local dotted_prefix = "live_galaxy.lua."
local extension_path = "extensions/?.lua"
local runtime_path = "extensions/live_galaxy/lua/live_galaxy_runtime.lua"

local modules = {
    "live_galaxy_carrier",
    "live_galaxy_component_discovery",
    "live_galaxy_normalize",
    "live_galaxy_observation",
    "live_galaxy_scheduler",
    "live_galaxy_telemetry",
    "live_galaxy_trace_config",
    "live_galaxy_x4_discovery",
}

local function relevant(name) return name:sub(1, #dotted_prefix) == dotted_prefix end

describe("X4 production Lua loader", function()
    local saved_path, saved_loaded

    before_each(function()
        saved_path = package.path
        saved_loaded = {}
        for name, value in pairs(package.loaded) do
            if relevant(name) then saved_loaded[name] = value end
        end
        for name in pairs(package.loaded) do
            if relevant(name) then package.loaded[name] = nil end
        end
        package.path = "tools/.cache/x4-default/?.lua"
    end)

    after_each(function()
        for name in pairs(package.loaded) do
            if relevant(name) then package.loaded[name] = nil end
        end
        for name, value in pairs(saved_loaded) do package.loaded[name] = value end
        package.path = saved_path
    end)

    it("adds the extension search path before loading local modules", function()
        assert.is_nil(package.path:find(extension_path, 1, true))
        local chunk, load_error = loadfile(runtime_path)
        assert.is_function(chunk, load_error)
        assert.is_table(chunk())
        assert.is_truthy(package.path:find(extension_path, 1, true))
        for _, name in ipairs(modules) do
            assert.is_table(require(dotted_prefix .. name), name)
        end
    end)
end)
