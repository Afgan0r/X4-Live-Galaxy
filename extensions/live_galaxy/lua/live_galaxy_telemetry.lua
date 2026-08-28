local telemetry = {}

local MAX_SECTIONS_PER_CYCLE = 1

function telemetry.produce_observation(adapter)
    if type(adapter) ~= "table" or type(adapter.read_observation) ~= "function" then
        return nil, "adapter_unavailable"
    end

    local observation, err = adapter.read_observation(MAX_SECTIONS_PER_CYCLE)
    if observation == nil then
        return nil, err or "observation_unavailable"
    end

    return observation
end

return telemetry
