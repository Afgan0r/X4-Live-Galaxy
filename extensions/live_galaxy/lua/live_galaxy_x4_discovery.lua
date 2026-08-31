local discovery = {}
local component_discovery = require("live_galaxy/lua/live_galaxy_component_discovery")

function discovery.new(api)
    return component_discovery.new(api)
end

function discovery.new_runtime_adapter()
    return component_discovery.new_runtime_adapter()
end

return discovery
