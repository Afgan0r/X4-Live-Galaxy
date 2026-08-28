local telemetry = {}

local normalize = require("live_galaxy_normalize")

local MAX_SECTIONS_PER_CYCLE = 1

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

function telemetry.produce_observation(adapter)
    if type(adapter) ~= "table" or type(adapter.read_observation) ~= "function" then
        return nil, "adapter_unavailable"
    end

    local observation, err = adapter.read_observation(MAX_SECTIONS_PER_CYCLE)
    if observation == nil then
        return nil, err or "observation_unavailable"
    end

    return normalize.serialize_telemetry(observation)
end

return telemetry
