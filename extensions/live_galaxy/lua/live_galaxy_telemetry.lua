local telemetry = {}

local normalize = require("live_galaxy/lua/live_galaxy_normalize")

local MAX_SECTIONS_PER_CYCLE = 1
local MAX_CANONICAL_ENCODED_FRAME_UTF8_BYTES = 1800
telemetry.MAX_CANONICAL_ENCODED_FRAME_UTF8_BYTES = MAX_CANONICAL_ENCODED_FRAME_UTF8_BYTES

local function bounded_limit(limit)
    if type(limit) ~= "number" or limit < 1 then
        return MAX_SECTIONS_PER_CYCLE
    end
    return math.min(math.floor(limit), MAX_SECTIONS_PER_CYCLE)
end

function telemetry.observe_runtime_scope(adapter, scope, limit)
    if type(adapter) ~= "table" or type(adapter.list_scope) ~= "function" then
        return nil, "adapter_unavailable"
    end
    if type(scope) ~= "string" or scope == "" then
        return nil, "scope_invalid"
    end

    local discovered, err = adapter:list_scope(scope, bounded_limit(limit))
    if type(discovered) ~= "table" then
        return nil, err or "scope_unavailable"
    end

    local sections = {}
    for _, raw_section in ipairs(discovered) do
        if #sections >= MAX_SECTIONS_PER_CYCLE then
            break
        end
        local section, normalize_err = normalize.normalize_section(raw_section)
        if section == nil then
            return nil, normalize_err
        end
        sections[#sections + 1] = section
    end
    table.sort(sections, function(left, right)
        return left.entity_id < right.entity_id
    end)
    return sections
end

function telemetry.produce_observation(adapter, version)
    if type(adapter) ~= "table" or type(adapter.read_observation) ~= "function" then
        return nil, "adapter_unavailable"
    end

    local observation, err = adapter.read_observation(MAX_SECTIONS_PER_CYCLE, version)
    if observation == nil then
        return nil, err or "observation_unavailable"
    end

    return normalize.serialize_telemetry(observation)
end

function telemetry.canonical_encoded_frame_utf8_bytes(frame)
    if type(frame) ~= "string" then return nil, "observation_invalid" end
    local bytes = #frame
    if bytes > MAX_CANONICAL_ENCODED_FRAME_UTF8_BYTES then
        return nil, "observation_oversized"
    end
    return bytes
end

function telemetry.produce_observation_source(adapter, version)
    if type(adapter) == "table" and type(adapter.read_observations) == "function" then
        local observations, err = adapter.read_observations(adapter, version)
        if type(observations) ~= "table" or #observations == 0 then
            return nil, err or "observation_unavailable"
        end
        local index = 0
        local source = {}
        function source.next_frame()
            index = index + 1
            local observation = observations[index]
            if observation == nil then return nil end
            local frame, normalize_err = normalize.serialize_telemetry(observation)
            if frame == nil then return nil, normalize_err end
            local bytes, bytes_err = telemetry.canonical_encoded_frame_utf8_bytes(frame)
            if bytes == nil then return nil, bytes_err end
            return frame, bytes
        end
        return source
    end
    local observation, err = telemetry.produce_observation(adapter, version)
    if observation == nil then return nil, err end
    local emitted = false
    local source = {}
    function source.next_frame()
        if emitted then return nil end
        emitted = true
        local bytes, bytes_err = telemetry.canonical_encoded_frame_utf8_bytes(observation)
        if bytes == nil then return nil, bytes_err end
        return observation, bytes
    end
    return source
end

function telemetry.produce_observations(adapter, version)
    local source, err = telemetry.produce_observation_source(adapter, version)
    if source == nil then return nil, err end
    local serialized = {}
    while true do
        local frame, frame_err = source.next_frame()
        if frame == nil then
            if frame_err ~= nil then return nil, frame_err end
            return serialized
        end
        serialized[#serialized + 1] = frame
    end
end

return telemetry
