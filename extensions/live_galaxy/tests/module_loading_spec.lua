local helper = require("test_helper")
local json = require("dkjson")
local lfs = require("lfs")
local pipe_name = "extensions.sn_mod_support_apis.ui.named_pipes.Interface"

describe("shipped Lua syntax #syntax", function()
    local function check_directory(directory)
        local entries = {}
        for name in lfs.dir(directory) do
            if name ~= "." and name ~= ".." and name ~= "tests" then entries[#entries + 1] = name end
        end
        table.sort(entries)
        for _, name in ipairs(entries) do
            local path = directory .. "/" .. name
            if lfs.attributes(path, "mode") == "directory" then check_directory(path)
            elseif name:match("%.lua$") then
                it("compiles " .. path, function()
                    local chunk, err = loadfile(path)
                    assert.is_function(chunk, err)
                end)
            end
        end
    end
    check_directory("extensions/live_galaxy")
end)

describe("real product modules #loading", function()
    local fixture, unexpected
    after_each(function()
        if fixture then fixture.restore() end
        assert.same({}, unexpected)
    end)
    before_each(function()
        unexpected = {}
        fixture = helper.new()
    end)

    local function environment(options)
        options = options or {}
        local calls, order, frames = {}, {}, {}
        local env = { calls = calls, order = order, frames = frames }
        local function observed(name, callback)
            calls[name] = spy.new(function(...)
                order[#order + 1] = name
                return callback(...)
            end)
            -- luassert spies are callable tables; the real adapter requires functions.
            return function(...) return calls[name](...) end
        end
        local function strict(values)
            return setmetatable(values, { __index = function(_, name)
                unexpected[#unexpected + 1] = name
                error("Unexpected external call: " .. name)
            end })
        end
        local buffer = {}
        env.buffer = buffer
        local C = strict({
            GetNumAllFactionStations = observed("count", function(faction)
                assert.equals("argon", faction)
                if options.count_error then error("native count failure") end
                return options.count or 2
            end),
            GetAllFactionStations = observed("fill", function(output, count, faction)
                assert.equals(buffer, output)
                assert.equals(2, count)
                assert.equals("argon", faction)
                output[0], output[1] = 20, 10
                return 2
            end),
            GetPeopleCapacity = observed("capacity", function(component, race, include_docked)
                assert.is_number(component)
                assert.equals("", race)
                assert.is_false(include_docked)
                if options.capacity_error then error("private native failure") end
                return component * 2
            end),
        })
        local ffi = strict({
            C = C,
            sizeof = observed("sizeof", function(kind)
                assert.equals("UniverseID", kind)
                return 8
            end),
            new = observed("allocate", function(kind, count)
                assert.equals("UniverseID[?]", kind)
                assert.equals(2, count)
                return buffer
            end),
        })
        package.preload.ffi = function() return ffi end
        _G.ConvertStringToLuaID = observed("convert", function(raw)
            assert.is_string(raw)
            assert.is_true(raw == "10" or raw == "20")
            return "station:" .. raw
        end)
        _G.ConvertIDTo64Bit = observed("convert64", function(component)
            assert.is_true(component == "station:10" or component == "station:20")
            return tonumber(component:match("%d+"))
        end)
        _G.GetComponentData = observed("metadata", function(component, owner, sector)
            assert.equals("owner", owner)
            assert.equals("sector", sector)
            if options.metadata_error then error("private metadata failure") end
            return "argon", "sector:" .. component:match("%d+")
        end)
        _G.DebugError = observed("debug", function() end)
        _G.Register_OnLoad_Init = observed("onload", function(callback, alias)
            assert.is_function(callback)
            assert.equals("extensions.live_galaxy.lua.live_galaxy_runtime", alias)
            env.init = callback
        end)
        _G.RegisterEvent = observed("register", function(name, callback)
            assert.equals("live_galaxy_observation", name)
            assert.is_function(callback)
            env.tick = callback
        end)
        local pipes = strict({
            _Write_Pipe_Raw = observed("write", function(name, payload)
                assert.equals("live_galaxy", name)
                frames[#frames + 1] = payload
                if options.write_error then error("pipe write failure") end
                return not options.backpressure
            end),
            Disconnect_Pipe = observed("disconnect", function(name)
                assert.equals("live_galaxy", name)
                if options.disconnect_error then error("disconnect failure") end
            end),
        })
        package.preload[pipe_name] = function() return pipes end
        env.pipes = pipes
        return env
    end

    it("executes the real native adapter with exact zero-based and identity contracts", function()
        local env = environment()
        local adapter = fixture.load("live_galaxy_component_discovery").new_runtime_adapter()
        local observations = assert(adapter.read_observations(1, 7))
        assert.equals("asset:station:10", observations[1].entity_id)
        assert.equals("asset:station:20", observations[2].entity_id)
        assert.equals(20, observations[1].runtime_facts.capacity[1].value)
        assert.equals("faction:argon", observations[1].runtime_facts.ownership[1].owner_id)
        assert.same({ "sizeof", "count", "allocate", "fill", "convert", "convert64",
            "convert", "convert64", "metadata", "capacity", "metadata", "capacity" }, env.order)
        -- The spy snapshots the initially empty buffer before native fill mutates it.
        assert.spy(env.calls.fill).was.called_with({}, 2, "argon")
        assert.spy(env.calls.convert).was.called_with("20")
        assert.spy(env.calls.convert64).was.called_with("station:10")
        assert.spy(env.calls.metadata).was.called_with("station:10", "owner", "sector")
        assert.spy(env.calls.capacity).was.called_with(10, "", false)
        assert.spy(env.calls.capacity).was.called(2)
        assert.spy(env.calls.write).was_not.called()
    end)

    it("rejects an excessive native count before allocation or further native work", function()
        local env = environment({ count = 130 })
        local adapter = fixture.load("live_galaxy_component_discovery").new_runtime_adapter()
        local observations, err = adapter.read_observations(1)
        assert.is_nil(observations)
        assert.equals("enumeration_overflow", err)
        assert.spy(env.calls.count).was.called(1)
        for _, name in ipairs({ "allocate", "fill", "convert", "metadata", "capacity" }) do
            assert.spy(env.calls[name]).was_not.called()
        end
    end)

    for _, stage in ipairs({ "count", "metadata", "capacity" }) do
        it("contains a thrown external " .. stage .. " failure", function()
            local env = environment({ [stage .. "_error"] = true })
            local adapter = fixture.load("live_galaxy_component_discovery").new_runtime_adapter()
            local observations, err = adapter.read_observations(1)
            assert.is_nil(observations)
            assert.equals(stage == "count" and "enumeration_unavailable" or "facts_unsupported", err)
            assert.spy(env.calls[stage]).was.called(1)
            assert.spy(env.calls.write).was_not.called()
            if stage == "count" then assert.spy(env.calls.allocate).was_not.called() end
        end)
    end

    it("runs real initialization and observation callbacks through delayed native and pipe imports", function()
        local env = environment()
        local runtime = fixture.runtime()
        assert.spy(env.calls.onload).was.called(1)
        assert.is_nil(package.loaded.ffi)
        assert.is_nil(package.loaded[pipe_name])
        env.init()
        assert.spy(env.calls.register).was.called_with("live_galaxy_observation", runtime.handle_tick)
        for _ = 1, 6 do assert.is_true(env.tick()) end
        local observations, markers = {}, 0
        for _, frame in ipairs(env.frames) do
            local decoded, _, err = json.decode(frame)
            assert.is_nil(err)
            if decoded.type == "observation" then observations[#observations + 1] = decoded end
            if decoded.type == "complete_marker" then markers = markers + 1 end
        end
        assert.equals(2, #observations)
        assert.equals("asset:station:10", observations[1].entity_id)
        assert.equals(1, markers)
        assert.spy(env.calls.count).was.called(1)
        assert.spy(env.calls.write).was.called(6)
        assert.spy(env.calls.disconnect).was_not.called()
    end)

    it("propagates real lazy pipe backpressure without disconnecting", function()
        local env = environment({ backpressure = true })
        local ok, status = fixture.runtime().emit("frame")
        assert.is_false(ok)
        assert.equals("pipe_backpressure", status)
        assert.spy(env.calls.write).was.called_with("live_galaxy", "frame")
        assert.spy(env.calls.disconnect).was_not.called()
    end)

    for _, disconnect_error in ipairs({ false, true }) do
        it("contains pipe write failure with disconnect failure=" .. tostring(disconnect_error), function()
            local env = environment({ write_error = true, disconnect_error = disconnect_error })
            local ok, status = fixture.runtime().emit("frame")
            assert.is_false(ok)
            assert.equals("pipe_reconnect", status)
            assert.spy(env.calls.write).was.called(1)
            assert.spy(env.calls.disconnect).was.called_with("live_galaxy")
            assert.spy(env.calls.disconnect).was.called(1)
        end)
    end

    it("reports absent external pipe modules and unavailable pipe callbacks", function()
        local runtime = fixture.runtime()
        local ok, status = runtime.emit("frame")
        assert.is_false(ok)
        assert.equals("pipe_unavailable", status)
        package.preload[pipe_name] = function() return {} end
        ok, status = runtime.emit("frame")
        assert.is_false(ok)
        assert.equals("pipe_unavailable", status)
    end)

    it("loads without the optional X4 initialization registration callback", function()
        local env = environment()
        _G.Register_OnLoad_Init = nil
        assert.is_table(fixture.runtime())
        assert.spy(env.calls.onload).was_not.called()
        assert.spy(env.calls.register).was_not.called()
        assert.spy(env.calls.write).was_not.called()
    end)

    it("fails a missing product module through the standard require path", function()
        package.path, package.cpath = "tools/.cache/absent-module/?.lua", ""
        local ok, err = pcall(fixture.load, "live_galaxy_runtime")
        assert.is_false(ok)
        assert.matches("live_galaxy_runtime", err, 1, true)
        assert.is_nil(package.loaded["live_galaxy/lua/live_galaxy_runtime"])
    end)

    it("restores config caches paths and external globals after a protected failure", function()
        local initial_path, initial_cpath = package.path, package.cpath
        local env = environment()
        fixture.trace_config({ enabled = true, attempt_id = "first-seed" })
        local first = fixture.runtime()
        local ok = pcall(function() error("intentional case failure") end)
        assert.is_false(ok)
        fixture.restore()
        assert.equals(initial_path, package.path)
        assert.equals(initial_cpath, package.cpath)
        fixture = helper.new()
        assert.is_nil(rawget(_G, "DebugError"))
        assert.is_nil(package.preload.ffi)
        assert.is_nil(package.preload[pipe_name])
        local second = fixture.runtime()
        assert.is_not.equal(first, second)
        assert.is_false(fixture.load("live_galaxy_trace_config").enabled)
        assert.equals("unset", fixture.load("live_galaxy_trace_config").attempt_id)
        assert.equals("pipe_unavailable", select(2, second.emit("frame")))
        assert.spy(env.calls.write).was_not.called()
    end)
end)
