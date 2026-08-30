package.path = package.path .. ";tests/x4-candidates/lua/?.lua"

local runner = require("live_galaxy_candidate_runner")
local phase_051 = require("phase_051_candidates")
local cases = {}

local function digest_value(payload)
    return string.rep("0", 56) .. string.format("%08x", #payload)
end

local function digest_adapter()
    return {
        hash = function(payload)
            return "sha256", digest_value(payload)
        end,
        verify = function(payload, algorithm, digest)
            return algorithm == "sha256" and digest == digest_value(payload)
        end,
    }
end

local function fixed_vector_adapter()
    local vectors = {
        [""] = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ["abc"] = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ["hello world"] = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
    }
    return {
        hash = function(payload) return "sha256", vectors[payload] end,
        verify = function(payload, algorithm, digest)
            return algorithm == "sha256" and vectors[payload] ~= nil and vectors[payload] == digest
        end,
    }
end

local function execution_context()
    return {
        adapters = {
            count_fill = function()
                return {
                    actual_result = "count-fill-contract-valid",
                    completeness = "complete",
                    elapsed_real_ms = 7,
                    elapsed_game_ms = 11,
                    seta_state = "inactive",
                    work_units = 2,
                    observations = { "count-valid", "fill-valid" },
                }
            end,
        },
        digest = digest_adapter(),
        timeout_markers = {},
    }
end

local function split_lines(value)
    local lines = {}
    for line in value:gmatch("([^\n]+)") do lines[#lines + 1] = line end
    return lines
end

local function decode_json(source)
    local position = 1
    local function skip_space()
        while source:sub(position, position):match("%s") do position = position + 1 end
    end
    local parse_value
    local function parse_string()
        assert(source:sub(position, position) == '"')
        position = position + 1
        local parts = {}
        while true do
            local character = source:sub(position, position)
            assert(character ~= "", "unterminated JSON string")
            if character == '"' then position = position + 1; return table.concat(parts) end
            if character == "\\" then
                local escaped = source:sub(position + 1, position + 1)
                local values = { ['"'] = '"', ['\\'] = '\\', ['/'] = '/', b = '\b', f = '\f',
                    n = '\n', r = '\r', t = '\t' }
                if escaped == "u" then
                    local code = tonumber(source:sub(position + 2, position + 5), 16)
                    assert(code and code < 128, "fixture JSON uses unsupported unicode escape")
                    parts[#parts + 1] = string.char(code)
                    position = position + 6
                else
                    assert(values[escaped], "invalid JSON escape")
                    parts[#parts + 1] = values[escaped]
                    position = position + 2
                end
            else
                parts[#parts + 1] = character
                position = position + 1
            end
        end
    end
    local function parse_array()
        position = position + 1
        local result = {}
        skip_space()
        if source:sub(position, position) == "]" then position = position + 1; return result end
        while true do
            result[#result + 1] = parse_value()
            skip_space()
            local delimiter = source:sub(position, position)
            if delimiter == "]" then position = position + 1; return result end
            assert(delimiter == ",", "invalid JSON array")
            position = position + 1
        end
    end
    local function parse_object()
        position = position + 1
        local result = {}
        skip_space()
        if source:sub(position, position) == "}" then position = position + 1; return result end
        while true do
            skip_space()
            local key = parse_string()
            skip_space()
            assert(source:sub(position, position) == ":", "invalid JSON object")
            position = position + 1
            result[key] = parse_value()
            skip_space()
            local delimiter = source:sub(position, position)
            if delimiter == "}" then position = position + 1; return result end
            assert(delimiter == ",", "invalid JSON object")
            position = position + 1
        end
    end
    parse_value = function()
        skip_space()
        local character = source:sub(position, position)
        if character == '"' then return parse_string() end
        if character == "[" then return parse_array() end
        if character == "{" then return parse_object() end
        local literal = source:sub(position)
        if literal:sub(1, 4) == "true" then position = position + 4; return true end
        if literal:sub(1, 5) == "false" then position = position + 5; return false end
        local number = literal:match("^-?%d+%.?%d*[eE]?[+-]?%d*")
        assert(number and number ~= "", "invalid JSON value")
        position = position + #number
        return assert(tonumber(number))
    end
    local result = parse_value()
    skip_space()
    assert(position > #source, "trailing JSON data")
    return result
end

local function independent_escape(value)
    return value:gsub('[%z\1-\31\\"]', function(character)
        local escapes = { ['"'] = '\\"', ['\\'] = '\\\\', ['\b'] = '\\b',
            ['\f'] = '\\f', ['\n'] = '\\n', ['\r'] = '\\r', ['\t'] = '\\t' }
        return escapes[character] or string.format("\\u%04x", character:byte())
    end)
end

local function independent_encode(value)
    if type(value) == "string" then return '"' .. independent_escape(value) .. '"' end
    if type(value) == "number" then return tostring(value) end
    if type(value) == "boolean" then return value and "true" or "false" end
    assert(type(value) == "table")
    local count, array = 0, true
    for key in pairs(value) do
        if type(key) ~= "number" or key < 1 or key % 1 ~= 0 then array = false; break end
        if key > count then count = key end
    end
    local parts = {}
    if array then
        for index = 1, count do parts[#parts + 1] = independent_encode(value[index]) end
        return "[" .. table.concat(parts, ",") .. "]"
    end
    local keys = {}
    for key in pairs(value) do keys[#keys + 1] = key end
    table.sort(keys)
    for _, key in ipairs(keys) do
        parts[#parts + 1] = '"' .. independent_escape(key) .. '":' .. independent_encode(value[key])
    end
    return "{" .. table.concat(parts, ",") .. "}"
end

local function load_contract()
    local file = assert(io.open("tools/x4-verification/contracts/runtime-evidence.v1.json", "rb"))
    local content = file:read("*a")
    file:close()
    return decode_json(content)
end

local function validate_jsonl(jsonl)
    local contract = load_contract()
    local lines = split_lines(jsonl)
    assert(#lines <= contract.bounds.max_output_rows)
    assert(#jsonl <= contract.bounds.max_total_bytes)
    local rows = {}
    local previous_candidate
    for index, line in ipairs(lines) do
        assert(#line <= contract.bounds.max_row_bytes)
        local row = decode_json(line)
        rows[index] = row
        for _, field in ipairs(contract.required_fields) do assert(row[field] ~= nil, field) end
        assert(row.schema_version == contract.schema_version)
        assert(row.evidence_classification == contract.evidence_classification)
        assert(row.stage_id == contract.stage_order[((index - 1) % #contract.stage_order) + 1])
        assert(row.digest_algorithm == contract.digest_algorithm)
        assert(#row.record_digest == contract.digest_hex_length)
        assert(row.record_digest == digest_value(row.canonical_digest_payload))
        local allowed_reason = false
        for _, reason in ipairs(contract.failure_reasons) do
            if row.failure_reason == reason then allowed_reason = true end
        end
        assert(allowed_reason)
        assert((row.failure_point == "none") == (row.failure_reason == "none"))
        local stage_index = ((index - 1) % #contract.stage_order) + 1
        if stage_index == 1 then
            assert(row.contract_verdict == "not_run" and row.effect_verdict == "not_run")
            if previous_candidate ~= nil then assert(previous_candidate < row.candidate_id) end
            previous_candidate = row.candidate_id
        elseif stage_index == 2 then
            assert(row.effect_verdict == "not_run")
            assert(row.candidate_id == previous_candidate)
        else
            assert(row.candidate_id == previous_candidate)
            if row.effect_verdict == "pass" then assert(row.actual_result == row.expected_result) end
        end
        local payload = {}
        for key, value in pairs(row) do
            if key ~= "digest_algorithm" and key ~= "canonical_digest_payload" and key ~= "record_digest" then
                payload[key] = value
            end
        end
        assert(independent_encode(payload) == row.canonical_digest_payload)
    end
    return lines, rows
end

local function successful_candidate(id, actual_result)
    return {
        id = id,
        source = "05.2-RESEARCH.md#phase-05.1-candidate-matrix",
        expected_result = "expected-result",
        execute = function()
            return { actual_result = actual_result or "expected-result", completeness = "complete",
                work_units = 2, observations = { "observation" } }
        end,
        validate = function(result) return type(result) == "table" and result.completeness == "complete" end,
    }
end


function cases.fixed_sha256_vectors_reject_same_length_tampering()
    local ok, err = runner.verify_digest_adapter(fixed_vector_adapter(), {
        { payload = "", sha256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855" },
        { payload = "abc", same_length_tamper = "abd", sha256 = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad" },
        { payload = "hello world", same_length_tamper = "hello worle", sha256 = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9" },
    })
    assert(ok == true and err == nil)

    local accepted, reason = runner.verify_digest_adapter({
        hash = function(payload) return "sha256", string.rep("0", 56) .. string.format("%08x", #payload) end,
        verify = function(payload, _, digest) return digest:sub(-8) == string.format("%08x", #payload) end,
    }, {
        { payload = "abc", same_length_tamper = "abd", sha256 = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad" },
    })
    assert(accepted == nil and reason == "digest_vector_mismatch")
end

local function multi_manifest(first)
    local manifest = phase_051.single_success()
    manifest.bounds.max_output_rows = 6
    manifest.bounds.max_total_bytes = 65536
    manifest.candidates = { successful_candidate("candidate-z-later"), first }
    return manifest
end

function cases.emits_one_candidate_as_three_digest_bound_jsonl_stages()
    local jsonl, result = runner.run(phase_051.single_success(), execution_context())

    assert(type(jsonl) == "string", tostring(result))
    local lines = validate_jsonl(jsonl)
    assert(#lines == 3)
    assert(lines[1]:match('"stage_id":"execution"'))
    assert(lines[2]:match('"stage_id":"contract"'))
    assert(lines[3]:match('"stage_id":"effect"'))
    assert(lines[3]:match('"execution_verdict":"pass"'))
    assert(lines[3]:match('"contract_verdict":"pass"'))
    assert(lines[3]:match('"effect_verdict":"pass"'))
    assert(lines[3]:match('"schema_version":"runtime%-evidence%.v1"'))
    assert(lines[3]:match('"record_digest":"[0-9a-f]+"'))
    assert(result.candidate_count == 1)
    assert(result.output_rows == 3)
    assert(result.total_bytes == #jsonl)
end

function cases.isolates_exceptions_malformed_results_and_work_unit_exhaustion()
    local failures = {
        { expected = "execution_exception", execute = function() error("private native text") end },
        { expected = "malformed_result", execute = function() return "invalid" end },
        { expected = "work_units_exceeded", execute = function()
            return { actual_result = "expected-result", completeness = "complete", work_units = 9 }
        end },
    }
    for _, failure in ipairs(failures) do
        local first = successful_candidate("candidate-a-first")
        first.execute = failure.execute
        local jsonl, result = runner.run(multi_manifest(first), execution_context())
        assert(type(jsonl) == "string", tostring(result))
        local _, rows = validate_jsonl(jsonl)
        assert(rows[1].candidate_id == "candidate-a-first")
        assert(rows[1].execution_verdict == "fail")
        assert(rows[1].actual_result == failure.expected)
        assert(rows[1].failure_reason == failure.expected)
        assert(rows[1].failure_point == "execution")
        assert(rows[4].candidate_id == "candidate-z-later")
        assert(rows[6].effect_verdict == "pass")
    end
end

function cases.uses_an_external_timeout_marker_without_invoking_the_stage()
    local calls = 0
    local first = successful_candidate("candidate-a-timeout")
    first.execute = function() calls = calls + 1; return {} end
    local context = execution_context()
    context.timeout_markers = { [first.id] = { execution = true } }
    local jsonl, result = runner.run(multi_manifest(first), context)
    assert(type(jsonl) == "string", tostring(result))
    local _, rows = validate_jsonl(jsonl)
    assert(calls == 0)
    assert(rows[1].actual_result == "timeout_marker")
    assert(rows[1].failure_reason == "timeout_marker")
    assert(rows[1].failure_point == "execution")
    assert(rows[4].candidate_id == "candidate-z-later")
    assert(rows[6].effect_verdict == "pass")
end

function cases.never_passes_a_valid_but_unexpected_effect()
    local manifest = phase_051.single_success()
    manifest.candidates = { successful_candidate("candidate-unexpected", "valid-unexpected-result") }
    local jsonl, result = runner.run(manifest, execution_context())
    assert(type(jsonl) == "string", tostring(result))
    local _, rows = validate_jsonl(jsonl)
    assert(rows[3].execution_verdict == "pass")
    assert(rows[3].contract_verdict == "pass")
    assert(rows[3].effect_verdict == "mismatch")
    assert(rows[3].failure_point == "effect")
    assert(rows[3].failure_reason == "effect_mismatch")
end

function cases.records_protected_contract_and_effect_failure_reasons_then_continues()
    local contract_failure = successful_candidate("candidate-a-contract")
    contract_failure.validate = function() error("private contract detail") end
    local jsonl = assert(runner.run(multi_manifest(contract_failure), execution_context()))
    local _, rows = validate_jsonl(jsonl)
    assert(rows[2].contract_verdict == "fail")
    assert(rows[2].failure_reason == "contract_exception")
    assert(rows[6].effect_verdict == "pass")

    local effect_failure = successful_candidate("candidate-a-effect")
    effect_failure.assess = function() error("private effect detail") end
    jsonl = assert(runner.run(multi_manifest(effect_failure), execution_context()))
    _, rows = validate_jsonl(jsonl)
    assert(rows[3].effect_verdict == "fail")
    assert(rows[3].failure_reason == "effect_exception")
    assert(rows[6].effect_verdict == "pass")
end

function cases.rejects_missing_identity_bounds_and_digest_failures_with_exact_codes()
    local manifest = phase_051.single_success()
    manifest.run_id = nil
    assert(select(2, runner.run(manifest, execution_context())) == "run_id_invalid")

    local bound_cases = {
        { field = "max_steps", value = 2, reason = "stage_row_bound_invalid" },
        { field = "max_candidate_rows", value = 2, reason = "stage_row_bound_invalid" },
        { field = "max_output_rows", value = 2, reason = "output_rows_exceeded" },
        { field = "max_row_bytes", value = 1, reason = "row_bytes_exceeded" },
        { field = "max_total_bytes", value = 1, reason = "total_bytes_exceeded" },
    }
    for _, case in ipairs(bound_cases) do
        manifest = phase_051.single_success()
        manifest.bounds[case.field] = case.value
        assert(select(2, runner.run(manifest, execution_context())) == case.reason)
    end

    local context = execution_context()
    context.digest.verify = function() return false end
    assert(select(2, runner.run(phase_051.single_success(), context)) == "digest_mismatch")

    context = execution_context()
    context.digest = nil
    assert(select(2, runner.run(phase_051.single_success(), context)) == "digest_adapter_missing")

    context = execution_context()
    context.digest.hash = function() error("hash unavailable") end
    assert(select(2, runner.run(phase_051.single_success(), context)) == "digest_result_invalid")
end

function cases.independent_contract_rejects_collapsed_verdicts_and_noncanonical_order()
    local jsonl = assert(runner.run(phase_051.single_success(), execution_context()))
    local lines, rows = validate_jsonl(jsonl)
    rows[1].contract_verdict = "pass"
    local payload = {}
    for key, value in pairs(rows[1]) do
        if key ~= "digest_algorithm" and key ~= "canonical_digest_payload" and key ~= "record_digest" then
            payload[key] = value
        end
    end
    rows[1].canonical_digest_payload = independent_encode(payload)
    rows[1].record_digest = digest_value(rows[1].canonical_digest_payload)
    lines[1] = independent_encode(rows[1])
    assert(not pcall(validate_jsonl, table.concat(lines, "\n")))

    lines = split_lines(jsonl)
    lines[1], lines[2] = lines[2], lines[1]
    assert(not pcall(validate_jsonl, table.concat(lines, "\n")))
end

return cases
