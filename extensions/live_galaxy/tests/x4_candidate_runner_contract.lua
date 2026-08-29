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
    for index, line in ipairs(lines) do
        assert(#line <= contract.bounds.max_row_bytes)
        local row = decode_json(line)
        for _, field in ipairs(contract.required_fields) do assert(row[field] ~= nil, field) end
        assert(row.schema_version == contract.schema_version)
        assert(row.evidence_classification == contract.evidence_classification)
        assert(row.stage_id == contract.stage_order[((index - 1) % #contract.stage_order) + 1])
        assert(row.digest_algorithm == contract.digest_algorithm)
        assert(#row.record_digest == contract.digest_hex_length)
        assert(row.record_digest == digest_value(row.canonical_digest_payload))
        local payload = {}
        for key, value in pairs(row) do
            if key ~= "digest_algorithm" and key ~= "canonical_digest_payload" and key ~= "record_digest" then
                payload[key] = value
            end
        end
        assert(independent_encode(payload) == row.canonical_digest_payload)
    end
    return lines
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

return cases
