local helper = {}
local prefix = "live_galaxy/lua/"
local pipe_module = "extensions.sn_mod_support_apis.ui.named_pipes.Interface"
local globals = {
    "DebugError", "Register_OnLoad_Init", "RegisterEvent",
    "ConvertStringToLuaID", "ConvertIDTo64Bit", "GetComponentData",
}

local function relevant(name)
    return name:sub(1, #prefix) == prefix
        or name == "extensions.live_galaxy.lua.live_galaxy_runtime"
        or name == "ffi" or name == pipe_module
end

local function snapshot(source)
    local copy = {}
    for key, value in pairs(source) do copy[key] = value end
    return copy
end

local function restore_entries(target, saved)
    for key in pairs(target) do if relevant(key) then target[key] = nil end end
    for key, value in pairs(saved) do if relevant(key) then target[key] = value end end
end

function helper.new()
    local loaded, preloads = snapshot(package.loaded), snapshot(package.preload)
    local path, cpath = package.path, package.cpath
    local saved_globals, changed_tables = {}, {}
    for _, name in ipairs(globals) do
        saved_globals[name] = rawget(_G, name)
        rawset(_G, name, nil)
    end
    for name in pairs(package.loaded) do if relevant(name) then package.loaded[name] = nil end end
    package.preload.ffi = nil
    package.preload[pipe_module] = nil

    local fixture = {}
    function fixture.load(name) return require(prefix .. name) end
    function fixture.runtime()
        package.loaded[prefix .. "live_galaxy_runtime"] = nil
        package.loaded["extensions.live_galaxy.lua.live_galaxy_runtime"] = nil
        return fixture.load("live_galaxy_runtime")
    end
    function fixture.trace_config(fields)
        local config = fixture.load("live_galaxy_trace_config")
        if not changed_tables[config] then changed_tables[config] = snapshot(config) end
        for key, value in pairs(fields) do config[key] = value end
        return config
    end
    function fixture.restore()
        for config, saved in pairs(changed_tables) do
            for key in pairs(config) do config[key] = nil end
            for key, value in pairs(saved) do config[key] = value end
        end
        for _, name in ipairs(globals) do rawset(_G, name, saved_globals[name]) end
        restore_entries(package.loaded, loaded)
        restore_entries(package.preload, preloads)
        package.path, package.cpath = path, cpath
    end
    return fixture
end

return helper
