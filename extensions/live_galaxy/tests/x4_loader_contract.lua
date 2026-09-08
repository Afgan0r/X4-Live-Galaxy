local dotted_prefix = "extensions.live_galaxy.lua."
local slash_prefix = "live_galaxy/lua/"
local source_root = "extensions/live_galaxy/lua/"

local modules = {
    "live_galaxy_carrier",
    "live_galaxy_component_discovery",
    "live_galaxy_normalize",
    "live_galaxy_observation",
    "live_galaxy_runtime",
    "live_galaxy_scheduler",
    "live_galaxy_telemetry",
    "live_galaxy_trace_config",
    "live_galaxy_x4_discovery",
}

local function relevant(name)
    return name:sub(1, #dotted_prefix) == dotted_prefix
        or name:sub(1, #slash_prefix) == slash_prefix
end

describe("X4 extension-qualified Lua loader", function()
    local saved_loaders, saved_path, saved_cpath, saved_loaded

    before_each(function()
        saved_loaders, saved_path, saved_cpath = package.loaders, package.path, package.cpath
        saved_loaded = {}
        for name, value in pairs(package.loaded) do
            if relevant(name) then saved_loaded[name] = value end
        end
        for name in pairs(package.loaded) do
            if relevant(name) then package.loaded[name] = nil end
        end
        package.path, package.cpath = "", ""
        package.loaders = {
            function(name)
                if name:sub(1, #dotted_prefix) ~= dotted_prefix then
                    return "\n\tX4 loader requires an extension-qualified module name"
                end
                local leaf = name:sub(#dotted_prefix + 1)
                if not leaf:match("^live_galaxy_[%w_]+$") then
                    return "\n\tX4 loader rejected an invalid module name"
                end
                local chunk, load_error = loadfile(source_root .. leaf .. ".lua")
                if chunk == nil then return "\n\t" .. tostring(load_error) end
                return chunk
            end,
        }
    end)

    after_each(function()
        for name in pairs(package.loaded) do
            if relevant(name) then package.loaded[name] = nil end
        end
        for name, value in pairs(saved_loaded) do package.loaded[name] = value end
        package.loaders, package.path, package.cpath = saved_loaders, saved_path, saved_cpath
    end)

    it("loads every production module without a slash search path", function()
        local slash_ok = pcall(require, slash_prefix .. "live_galaxy_carrier")
        assert.is_false(slash_ok)
        for _, name in ipairs(modules) do
            assert.is_table(require(dotted_prefix .. name))
        end
    end)
end)
