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

function cases.emits_one_candidate_as_three_digest_bound_jsonl_stages()
    local jsonl, result = runner.run(phase_051.single_success(), execution_context())

    assert(type(jsonl) == "string", tostring(result))
    local lines = split_lines(jsonl)
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

return cases
