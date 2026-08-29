local telemetry = {}

local normalize = require("live_galaxy/lua/live_galaxy_normalize")

local MAX_SECTIONS_PER_CYCLE = 1
local MAX_DISCOVERY_OBSERVATION_FRAMES = 64
local MAX_DISCOVERY_OBSERVATION_BYTES = 1800

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

function telemetry.produce_observations(adapter, version)
    if type(adapter) == "table" and type(adapter.read_observations) == "function" then
        local observations, err = adapter.read_observations(adapter, version)
        if type(observations) ~= "table" or #observations == 0
            or #observations > MAX_DISCOVERY_OBSERVATION_FRAMES then
            return nil, err or "observation_unavailable"
        end
        local serialized = {}
        for _, observation in ipairs(observations) do
            local frame, normalize_err = normalize.serialize_telemetry(observation)
            if frame == nil or #frame > MAX_DISCOVERY_OBSERVATION_BYTES then
                return nil, normalize_err or "observation_oversized"
            end
            serialized[#serialized + 1] = frame
        end
        return serialized
    end
    local observation, err = telemetry.produce_observation(adapter, version)
    if observation == nil then return nil, err end
    return { observation }
end

return telemetry
