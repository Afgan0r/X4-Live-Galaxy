local runner = {}

local SCHEMA_VERSION = "runtime-evidence.v1"
local EVIDENCE_CLASSIFICATION = "local-contract-only"
local STAGES = { "execution", "contract", "effect" }
local CANDIDATE_FIELDS = {
    adapter_id = true, bounds = true, expected_result = true, id = true, source = true,
}
local MANIFEST_FIELDS = {
    bounds = true, build_id = true, build_profile_digest = true, candidates = true,
    game_version = true, mod_list = true, prior_dossier_digest = true,
    prior_dossier_id = true, run_id = true, scenario_id = true, schema_version = true,
}
local OUTCOME_FIELDS = {
    actual_result = true, completeness = true, elapsed_game_ms = true,
    elapsed_real_ms = true, observations = true, seta_state = true, work_units = true,
}
local HARD_BOUNDS = {
    max_candidates = 16, max_steps = 3, max_work_units_per_step = 64,
    max_candidate_rows = 3, max_observations_per_step = 32, max_mods = 64,
    max_string_bytes = 256, max_row_bytes = 4096, max_total_bytes = 65536,
    max_output_rows = 48,
}

local function is_array(value)
    if type(value) ~= "table" or getmetatable(value) ~= nil then return false, 0 end
    local count = 0
    for key in pairs(value) do
        if type(key) ~= "number" or key < 1 or key % 1 ~= 0 then return false end
        if key > count then count = key end
    end
    for index = 1, count do if value[index] == nil then return false end end
    return true, count
end

local function escape_json(value)
    return value:gsub('[%z\1-\31\\"]', function(character)
        local escapes = { ['"'] = '\\"', ['\\'] = '\\\\', ['\b'] = '\\b',
            ['\f'] = '\\f', ['\n'] = '\\n', ['\r'] = '\\r', ['\t'] = '\\t' }
        return escapes[character] or string.format("\\u%04x", character:byte())
    end)
end

local function canonical_json(value)
    local kind = type(value)
    if kind == "string" then return '"' .. escape_json(value) .. '"' end
    if kind == "number" then
        if value ~= value or value == math.huge or value == -math.huge then return nil, "non_finite_number" end
        return tostring(value)
    end
    if kind == "boolean" then return value and "true" or "false" end
    if kind ~= "table" then return nil, "unsupported_json_type" end
    local array, count = is_array(value)
    local parts = {}
    if array then
        for index = 1, count do
            local encoded, err = canonical_json(value[index])
            if encoded == nil then return nil, err end
            parts[#parts + 1] = encoded
        end
        return "[" .. table.concat(parts, ",") .. "]"
    end
    local keys = {}
    for key in pairs(value) do
        if type(key) ~= "string" then return nil, "object_key_invalid" end
        keys[#keys + 1] = key
    end
    table.sort(keys)
    for _, key in ipairs(keys) do
        local encoded, err = canonical_json(value[key])
        if encoded == nil then return nil, err end
        parts[#parts + 1] = '"' .. escape_json(key) .. '":' .. encoded
    end
    return "{" .. table.concat(parts, ",") .. "}"
end

local function valid_string(value)
    return type(value) == "string" and value ~= "" and #value <= HARD_BOUNDS.max_string_bytes
end

local function valid_digest(value)
    return type(value) == "string" and #value == 64 and value:match("^[0-9a-f]+$") ~= nil
end

local function copy_sorted_strings(values, maximum)
    local dense, count = is_array(values)
    if not dense or count > maximum then return nil end
    local result = {}
    for index = 1, count do
        local value = values[index]
        if not valid_string(value) then return nil end
        result[index] = value
    end
    table.sort(result)
    return result
end

local function validate_services(context)
    if type(context) ~= "table" or type(context.adapters) ~= "table" then
        return nil, "adapter_registry_missing"
    end
    if type(context.digest) ~= "table" or type(context.digest.hash) ~= "function"
        or type(context.digest.verify) ~= "function" then
        return nil, "digest_adapter_missing"
    end
    if type(context.watchdog) ~= "table" or type(context.watchdog.invoke) ~= "function" then
        return nil, "watchdog_adapter_missing"
    end
    return true
end

local function validate_manifest(manifest)
    if type(manifest) ~= "table" or getmetatable(manifest) ~= nil
        or manifest.schema_version ~= SCHEMA_VERSION then
        return nil, "schema_version_invalid"
    end
    for field in pairs(manifest) do
        if not MANIFEST_FIELDS[field] then return nil, "manifest_schema_invalid" end
    end
    for _, field in ipairs({ "run_id", "game_version", "scenario_id", "prior_dossier_id", "build_id" }) do
        if not valid_string(manifest[field]) then return nil, field .. "_invalid" end
    end
    if not valid_digest(manifest.prior_dossier_digest) then return nil, "prior_dossier_digest_invalid" end
    if not valid_digest(manifest.build_profile_digest) then return nil, "build_profile_digest_invalid" end
    local dense_candidates, candidate_count = is_array(manifest.candidates)
    if not dense_candidates or candidate_count == 0
        or candidate_count > HARD_BOUNDS.max_candidates then return nil, "candidate_count_invalid" end
    local mods = copy_sorted_strings(manifest.mod_list, HARD_BOUNDS.max_mods)
    if mods == nil then return nil, "mod_list_invalid" end
    return mods, nil, candidate_count
end

local function effective_bounds(requested)
    if type(requested) ~= "table" or getmetatable(requested) ~= nil then
        return nil, "bounds_missing"
    end
    for name in pairs(requested) do
        if HARD_BOUNDS[name] == nil then return nil, "bounds_schema_invalid" end
    end
    local bounds = {}
    for name, hard_limit in pairs(HARD_BOUNDS) do
        local value = requested[name] or hard_limit
        if type(value) ~= "number" or value % 1 ~= 0 or value < 1 or value > hard_limit then
            return nil, name .. "_invalid"
        end
        bounds[name] = value
    end
    if bounds.max_steps ~= #STAGES or bounds.max_candidate_rows ~= #STAGES then
        return nil, "stage_row_bound_invalid"
    end
    return bounds
end

local function make_payload(manifest, mods, candidate, stage, state, outcome)
    return {
        actual_result = state.actual_result, build_id = manifest.build_id,
        build_profile_digest = manifest.build_profile_digest, candidate_id = candidate.id,
        candidate_source = candidate.source, completeness = state.completeness,
        contract_verdict = state.contract_verdict, effect_verdict = state.effect_verdict,
        elapsed_game_ms = outcome.elapsed_game_ms or 0, elapsed_real_ms = outcome.elapsed_real_ms or 0,
        evidence_classification = EVIDENCE_CLASSIFICATION, execution_verdict = state.execution_verdict,
        expected_result = candidate.expected_result, failure_point = state.failure_point,
        failure_reason = state.failure_reason,
        game_version = manifest.game_version, mod_list = mods,
        observation_count = outcome.observation_count or 0,
        prior_dossier_digest = manifest.prior_dossier_digest, prior_dossier_id = manifest.prior_dossier_id,
        run_id = manifest.run_id, scenario_id = manifest.scenario_id, schema_version = SCHEMA_VERSION,
        seta_state = outcome.seta_state or "not_applicable", stage_id = stage,
        work_units = outcome.work_units or 0,
    }
end

local function digest_row(payload, digest_adapter)
    local canonical, canonical_err = canonical_json(payload)
    if canonical == nil then return nil, canonical_err end
    local ok, algorithm, digest = pcall(digest_adapter.hash, canonical)
    if not ok or algorithm ~= "sha256" or not valid_digest(digest) then return nil, "digest_result_invalid" end
    local verified, matches = pcall(digest_adapter.verify, canonical, algorithm, digest)
    if not verified or matches ~= true then return nil, "digest_mismatch" end
    payload.digest_algorithm = algorithm
    payload.canonical_digest_payload = canonical
    payload.record_digest = digest
    return canonical_json(payload)
end

local function normalize_outcome(value, bounds, candidate_bounds)
    if type(value) ~= "table" or not valid_string(value.actual_result) then return nil, "malformed_result" end
    for field in pairs(value) do
        if not OUTCOME_FIELDS[field] then return nil, "malformed_result" end
    end
    local completeness = value.completeness
    if completeness ~= "complete" and completeness ~= "partial" and completeness ~= "unknown"
        and completeness ~= "not_applicable" then return nil, "completeness_invalid" end
    local observations = copy_sorted_strings(value.observations or {}, bounds.max_observations_per_step)
    if observations == nil then return nil, "observations_invalid" end
    local seta_state = value.seta_state or "not_applicable"
    if seta_state ~= "active" and seta_state ~= "inactive" and seta_state ~= "unknown"
        and seta_state ~= "not_applicable" then return nil, "seta_state_invalid" end
    local work_units = value.work_units or 0
    if type(work_units) ~= "number" or work_units % 1 ~= 0 or work_units < 0
        or work_units > bounds.max_work_units_per_step
        or work_units > candidate_bounds.max_work_units then return nil, "work_units_exceeded" end
    for _, field in ipairs({ "elapsed_real_ms", "elapsed_game_ms" }) do
        local number = value[field] or 0
        if type(number) ~= "number" or number % 1 ~= 0 or number < 0
            or number > 9007199254740991 then return nil, field .. "_invalid" end
    end
    return {
        actual_result = value.actual_result, completeness = completeness,
        elapsed_real_ms = value.elapsed_real_ms or 0, elapsed_game_ms = value.elapsed_game_ms or 0,
        seta_state = seta_state, work_units = work_units,
        observation_count = #observations,
    }
end

local function validate_candidate_bounds(value, hard_bounds)
    if type(value) ~= "table" or type(value.max_work_units) ~= "number"
        or value.max_work_units % 1 ~= 0 or value.max_work_units < 1
        or value.max_work_units > hard_bounds.max_work_units_per_step then
        return nil
    end
    for field in pairs(value) do
        if field ~= "max_work_units" then return nil end
    end
    return { max_work_units = value.max_work_units }
end

local function ordered_candidates(candidates, candidate_count, context, bounds)
    local ordered, seen = {}, {}
    for index = 1, candidate_count do
        local candidate = candidates[index]
        if type(candidate) ~= "table" or getmetatable(candidate) ~= nil then
            return nil, "candidate_schema_invalid"
        end
        for field in pairs(candidate) do
            if not CANDIDATE_FIELDS[field] then return nil, "candidate_schema_invalid" end
        end
        if not valid_string(candidate.id) or seen[candidate.id]
            or not valid_string(candidate.adapter_id) or not valid_string(candidate.source)
            or not valid_string(candidate.expected_result)
            or type(candidate.bounds) ~= "table" or getmetatable(candidate.bounds) ~= nil
            or validate_candidate_bounds(candidate.bounds, bounds) == nil then
            return nil, "candidate_schema_invalid"
        end
        if type(context.adapters[candidate.adapter_id]) ~= "function" then
            return nil, "adapter_unregistered"
        end
        seen[candidate.id] = true
        ordered[#ordered + 1] = candidate
    end
    table.sort(ordered, function(left, right) return left.id < right.id end)
    return ordered
end

local function invoke_bounded_adapter(candidate, context, bounds)
    -- Reserve the stage's unit before invoking cooperative local-contract work.
    -- This synchronous hook cannot preempt a blocking ffi.C/native call; build
    -- retention keeps that boundary runtime-pending until terminable isolation
    -- has X4-faithful evidence.
    local reserved_work_units = 1
    if reserved_work_units > bounds.max_work_units_per_step then return nil, "timeout_marker" end
    local adapter = context.adapters[candidate.adapter_id]
    local invoked, status, value = pcall(context.watchdog.invoke, candidate.id, "execution",
        reserved_work_units, adapter)
    if not invoked or status == "timeout" then return nil, "timeout_marker" end
    if status == "callback-error" then return nil, "execution_exception" end
    if status ~= "completed" then return nil, "timeout_marker" end
    return value
end

local function execute_candidate(candidate, context, bounds)
    local state = { actual_result = "not_run", completeness = "unknown", failure_point = "none", failure_reason = "none",
        execution_verdict = "not_run", contract_verdict = "not_run", effect_verdict = "not_run" }
    local outcomes = {}
    local raw, execution_error = invoke_bounded_adapter(candidate, context, bounds)
    if raw == nil then
        local reason = execution_error == "execution_exception" and execution_error or "timeout_marker"
        state.execution_verdict, state.failure_point, state.failure_reason, state.actual_result =
            "fail", "execution", reason, reason
        outcomes.execution = {}
        return state, outcomes
    end
    local outcome, reason = normalize_outcome(raw, bounds, candidate.bounds)
    if outcome == nil then
        state.execution_verdict, state.failure_point, state.failure_reason, state.actual_result =
            "fail", "execution", reason, reason
        outcomes.execution = {}
        return state, outcomes
    end
    state.execution_verdict = "pass"
    state.actual_result, state.completeness = outcome.actual_result, outcome.completeness
    outcomes.execution = outcome
    outcomes.contract = { work_units = 1 }
    if outcome.completeness ~= "complete" then
        state.contract_verdict, state.failure_point, state.failure_reason = "fail", "contract", "contract_rejected"
        return state, outcomes
    end
    state.contract_verdict = "pass"
    outcomes.effect = { work_units = 1 }
    local matches = state.actual_result == candidate.expected_result
    state.effect_verdict = matches == true and "pass" or "mismatch"
    if state.effect_verdict ~= "pass" then
        state.failure_point, state.failure_reason = "effect", "effect_mismatch"
    end
    return state, outcomes
end

function runner.run(manifest, context)
    local services_ok, services_err = validate_services(context)
    if services_ok == nil then return nil, services_err end
    local mods, manifest_err, candidate_count = validate_manifest(manifest)
    if mods == nil then return nil, manifest_err end
    local bounds, bounds_err = effective_bounds(manifest.bounds)
    if bounds == nil then return nil, bounds_err end
    local candidates, candidates_err = ordered_candidates(manifest.candidates, candidate_count, context, bounds)
    if candidates == nil then return nil, candidates_err end
    if #candidates * #STAGES > bounds.max_output_rows then return nil, "output_rows_exceeded" end
    local lines = {}
    for _, candidate in ipairs(candidates) do
        local final_state, outcomes = execute_candidate(candidate, context, bounds)
        local visible = { actual_result = "not_run", completeness = "unknown", failure_point = "none", failure_reason = "none",
            execution_verdict = "not_run", contract_verdict = "not_run", effect_verdict = "not_run" }
        for _, stage in ipairs(STAGES) do
            if stage == "execution" then
                visible.execution_verdict = final_state.execution_verdict
                visible.actual_result, visible.completeness = final_state.actual_result, final_state.completeness
            elseif stage == "contract" and final_state.execution_verdict == "pass" then
                visible.contract_verdict = final_state.contract_verdict
            elseif stage == "effect" and final_state.contract_verdict == "pass" then
                visible.effect_verdict = final_state.effect_verdict
            end
            if final_state.failure_point == stage then
                visible.failure_point, visible.failure_reason = stage, final_state.failure_reason
            end
            local row, row_err = digest_row(make_payload(manifest, mods, candidate, stage, visible,
                outcomes[stage] or {}), context.digest)
            if row == nil then return nil, row_err end
            if #row > bounds.max_row_bytes then return nil, "row_bytes_exceeded" end
            lines[#lines + 1] = row
        end
    end
    local jsonl = table.concat(lines, "\n")
    if #jsonl > bounds.max_total_bytes then return nil, "total_bytes_exceeded" end
    return jsonl, { candidate_count = #candidates, output_rows = #lines, total_bytes = #jsonl }
end

function runner.run_native()
    return nil, "trusted_runtime_attestation_missing"
end

function runner.verify_digest_adapter(adapter, vectors)
    local dense_vectors, vector_count = is_array(vectors)
    if type(adapter) ~= "table" or type(adapter.hash) ~= "function"
        or type(adapter.verify) ~= "function" or not dense_vectors or vector_count == 0 then
        return nil, "digest_adapter_missing"
    end
    for index = 1, vector_count do
        local vector = vectors[index]
        if type(vector) ~= "table" or type(vector.payload) ~= "string" or not valid_digest(vector.sha256) then
            return nil, "digest_vector_invalid"
        end
        local ok, algorithm, digest = pcall(adapter.hash, vector.payload)
        if not ok or algorithm ~= "sha256" or digest ~= vector.sha256 then return nil, "digest_vector_mismatch" end
        local verified, matches = pcall(adapter.verify, vector.payload, algorithm, digest)
        if not verified or matches ~= true then return nil, "digest_vector_mismatch" end
        if type(vector.same_length_tamper) == "string" and #vector.same_length_tamper == #vector.payload then
            local tampered, accepted = pcall(adapter.verify, vector.same_length_tamper, algorithm, digest)
            if tampered and accepted == true then return nil, "digest_tamper_accepted" end
        end
    end
    return true
end

runner.canonical_json = canonical_json

return runner
