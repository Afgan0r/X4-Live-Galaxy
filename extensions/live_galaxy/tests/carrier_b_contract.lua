local helper = require("test_helper")

describe("owned Carrier B adapter", function()
    local fixture, saved_loadlib
    before_each(function()
        fixture, saved_loadlib = helper.new(), package.loadlib
    end)
    after_each(function()
        package.loadlib = saved_loadlib
        fixture.restore()
    end)

    local function native(options)
        options = options or {}
        local calls, fact, begin, finish = {}, nil, nil, nil
        local function record(name, result)
            return function(_, value)
                calls[#calls + 1] = name
                if name == "push_record" then fact = value end
                if name == "begin_section" then begin = value end
                if name == "finish_section" then finish = value end
                if options.throw == name then error("injected " .. name) end
                if result == nil then result = 0 end
                if name == "progress" then
                    return result, "ready", "connected", "10001", "available", "producer:1"
                end
                return result
            end
        end
        local api = {
            abi_version = function() return options.abi or 2 end,
            open = function(version, limits, source)
                calls[#calls + 1] = "open"
                assert.equals(2, version)
                assert.equals(2048, limits.data_message_bytes)
                assert.equals("x4:carrier_b_acceptance", source.source_scope)
                return 0, "1:1"
            end,
            begin_section = record("begin_section", options.begin_code),
            push_record = record("push_record"),
            finish_section = record("finish_section"),
            fail_section = record("fail_section"),
            progress = record("progress", options.progress_code),
            poll_control = record("poll_control", options.control_code or 3),
            reset = record("reset"),
            close = record("close"),
        }
        if options.extra then api.raw_send = function() end end
        local loadlib = function(path, symbol)
            calls[#calls + 1] = "loadlib"
            assert.equals(".\\extensions\\live_galaxy\\ui_c_library_live_galaxy_carrier_64.txt", path)
            assert.equals("luaopen_live_galaxy_carrier", symbol)
            if options.load_failure then return nil, "missing" end
            return function()
                if options.init_failure then error("initializer") end
                return api
            end
        end
        return {
            calls = calls,
            api = api,
            loadlib = loadlib,
            fact = function() return fact end,
            begin = function() return begin end,
            finish = function() return finish end,
        }
    end

    it("loads ABI 2 and samples one reserved realtime fact", function()
        local env, getter_calls, clock_calls = native(), 0, 0
        local carrier = assert(fixture.load("live_galaxy_carrier").new({ loadlib = env.loadlib }))
        local observation = assert(fixture.load("live_galaxy_observation").new({
            getter = function()
                env.calls[#env.calls + 1] = "getter"
                getter_calls = getter_calls + 1
                return 123.5
            end,
            clock_getter = function()
                clock_calls = clock_calls + 1
                return clock_calls == 1 and 10.25 or 10.5
            end,
        }))
        local result = fixture.load("live_galaxy_scheduler").tick(
            "telemetry_tick", carrier, observation)
        assert.equals("sampled", result.disposition)
        assert.equals(1, getter_calls)
        assert.equals(2, clock_calls)
        assert.same({ "loadlib", "open", "progress", "poll_control", "begin_section",
            "getter", "push_record", "finish_section" }, env.calls)
        assert.same({
            entity_id = "x4:runtime:realtime_clock", observation_version = 1,
            getter = "GetCurRealTime", raw_value = "123.5", semantics = "opaque_runtime_number",
        }, env.fact())
        assert.equals("game_time_millis", env.begin().capture_clock)
        assert.equals("10250", env.begin().capture_start_millis)
        assert.equals("unknown", env.begin().source_epoch_status)
        assert.equals("runtime_start", env.begin().source_boundary)
        assert.equals("10500", env.finish().capture_end_millis)
    end)

    for _, value in ipairs({ false, "0", 1.5 }) do
        it("rejects malformed native result " .. tostring(value), function()
            local env = native({ progress_code = value })
            local carrier = assert(fixture.load("live_galaxy_carrier").new({ loadlib = env.loadlib }))
            local code, reason = carrier:progress(1)
            assert.is_nil(code)
            assert.equals("result_shape", reason)
        end)
    end

    it("drives the registered callback with the actual event arguments", function()
        local env, init, tick, getter_calls = native(), nil, nil, 0
        package.loadlib = env.loadlib
        package.preload.ffi = function()
            return { C = { GetCurRealTime = function()
                getter_calls = getter_calls + 1
                return 2
            end, GetCurrentGameTime = function() return getter_calls == 0 and 10.25 or 10.5 end } }
        end
        _G.Register_OnLoad_Init = function(callback) init = callback end
        _G.RegisterEvent = function(_, callback) tick = callback end
        fixture.runtime()
        init()
        assert.same({ true, "sampled" }, { tick("live_galaxy_observation", "telemetry_tick") })
        assert.equals(1, getter_calls)
        assert.equals("10250", env.begin().capture_start_millis)
        assert.equals("10500", env.finish().capture_end_millis)
        assert.same({ true, "sampled" }, {
            tick("live_galaxy_observation", "telemetry_game_loaded"),
        })
        assert.equals("boundary_uncertain", env.begin().source_epoch_status)
        assert.equals("game_loaded", env.begin().source_boundary)
    end)

    it("marks exactly the first collection after a Lua module reload", function()
        local env, init, tick = native(), nil, nil
        package.loadlib = env.loadlib
        package.preload.ffi = function()
            return { C = {
                GetCurRealTime = function() return 2 end,
                GetCurrentGameTime = function() return 10 end,
            } }
        end
        _G.Register_OnLoad_Init = function(callback) init = callback end
        _G.RegisterEvent = function(_, callback) tick = callback end

        fixture.runtime()
        init()
        tick("live_galaxy_observation", "telemetry_tick")
        assert.equals("runtime_start", env.begin().source_boundary)

        fixture.runtime()
        init()
        local actual_begin = env.api.begin_section
        env.api.begin_section = function() return -21 end
        assert.same({ false, "producer_busy" }, {
            tick("live_galaxy_observation", "telemetry_tick"),
        })
        assert.equals("runtime_start", env.begin().source_boundary)
        env.api.begin_section = actual_begin
        tick("live_galaxy_observation", "telemetry_tick")
        assert.equals("boundary_uncertain", env.begin().source_epoch_status)
        assert.equals("lua_reload", env.begin().source_boundary)
        tick("live_galaxy_observation", "telemetry_tick")
        assert.equals("unknown", env.begin().source_epoch_status)
        assert.equals("runtime_start", env.begin().source_boundary)
    end)

    it("redacts callback paths and control characters", function()
        local env, init, tick, diagnostic = native({ throw = "progress" }), nil, nil, nil
        package.loadlib = env.loadlib
        package.preload.ffi = function()
            return { C = { GetCurRealTime = function() return 2 end } }
        end
        _G.DebugError = function(value) diagnostic = value end
        _G.Register_OnLoad_Init = function(callback) init = callback end
        _G.RegisterEvent = function(_, callback) tick = callback end
        fixture.runtime()
        init()
        tick("live_galaxy_observation", "telemetry_tick")
        assert.equals("Live Galaxy Carrier B: event=transition detail=operation_failure", diagnostic)
        assert.is_nil(diagnostic:match("[A-Z]:\\"))
        assert.is_nil(diagnostic:match("[%c]"))
    end)

    for code, reason in pairs({
        [6] = "permanently_rejected", [7] = "ambiguous_commit",
        [8] = "retry_exhausted", [9] = "disconnected", [10] = "restart_required",
    }) do
        it("keeps terminal status " .. reason, function()
            local env = native({ progress_code = code })
            local carrier = assert(fixture.load("live_galaxy_carrier").new({ loadlib = env.loadlib }))
            local observation = assert(fixture.load("live_galaxy_observation").new({
                getter = function() return 1 end, clock_getter = function() return 0 end,
            }))
            local result = fixture.load("live_galaxy_scheduler").tick("telemetry_tick", carrier, observation)
            assert.equals(reason, result.disposition)
        end)
    end

    it("does not call the getter while immutable retry is pending", function()
        local env, getter_calls = native({ progress_code = 2, begin_code = -21 }), 0
        local carrier = assert(fixture.load("live_galaxy_carrier").new({ loadlib = env.loadlib }))
        local observation = assert(fixture.load("live_galaxy_observation").new({
            getter = function() getter_calls = getter_calls + 1; return 1 end,
            clock_getter = function() return 0 end,
        }))
        local result = fixture.load("live_galaxy_scheduler").tick({
            capture_start_millis = "1", capture_end_millis = "1",
        }, carrier, observation)
        assert.equals("producer_busy", result.disposition)
        assert.equals(0, getter_calls)
        assert.same({ "loadlib", "open", "progress", "poll_control", "begin_section" }, env.calls)
    end)

    for _, case in ipairs({
        { options = { load_failure = true }, expected = "loader_failure" },
        { options = { init_failure = true }, expected = "initializer_failure" },
        { options = { abi = 1 }, expected = "abi_mismatch" },
        { options = { extra = true }, expected = "operation_shape" },
    }) do
        it("fails closed for " .. case.expected, function()
            local env = native(case.options)
            local value, err = fixture.load("live_galaxy_carrier").new({ loadlib = env.loadlib })
            assert.is_nil(value)
            assert.equals(case.expected, err)
        end)
    end

    for _, value in ipairs({ "not-a-number", math.huge, -math.huge }) do
        it("rejects invalid getter value " .. tostring(value), function()
            local env = native()
            local carrier = assert(fixture.load("live_galaxy_carrier").new({ loadlib = env.loadlib }))
            local observation = assert(fixture.load("live_galaxy_observation").new({
                getter = function() return value end,
                clock_getter = function() return 0 end,
            }))
            local result = fixture.load("live_galaxy_scheduler").tick({
                capture_start_millis = "1", capture_end_millis = "1",
            }, carrier, observation)
            assert.equals("invalid_fact", result.disposition)
            assert.equals("fail_section", env.calls[#env.calls])
        end)
    end

    it("cleans the callback guard after an operation failure", function()
        local env = native({ throw = "progress" })
        local carrier = assert(fixture.load("live_galaxy_carrier").new({ loadlib = env.loadlib }))
        local context = { capture_start_millis = "1", capture_end_millis = "1" }
        local result = fixture.load("live_galaxy_scheduler").tick(context, carrier, {})
        assert.equals("operation_failure", result.disposition)
        assert.is_false(context.reentry_guard)
    end)

    it("closes exactly once through explicit close or Lua state collection", function()
        local function close_count(calls)
            local count = 0
            for _, call in ipairs(calls) do if call == "close" then count = count + 1 end end
            return count
        end
        local explicit = native()
        local carrier = assert(fixture.load("live_galaxy_carrier").new({ loadlib = explicit.loadlib }))
        assert.equals(0, carrier:close())
        carrier = nil
        collectgarbage("collect")
        assert.equals(1, close_count(explicit.calls))

        local finalized = native()
        carrier = assert(fixture.load("live_galaxy_carrier").new({ loadlib = finalized.loadlib }))
        carrier = nil
        collectgarbage("collect")
        collectgarbage("collect")
        assert.equals(1, close_count(finalized.calls))
    end)

    it("initializes once through normal require and registers one callback", function()
        local env, init, tick = native(), nil, nil
        package.loadlib = env.loadlib
        package.preload.ffi = function()
            return { C = { GetCurRealTime = function() return 2 end } }
        end
        _G.Register_OnLoad_Init = function(callback, alias)
            assert.equals("extensions.live_galaxy.lua.live_galaxy_runtime", alias)
            init = callback
        end
        _G.RegisterEvent = function(name, callback)
            assert.equals("live_galaxy_observation", name)
            tick = callback
        end
        local runtime = fixture.runtime()
        assert.is_function(init)
        init()
        assert.is_function(tick)
        assert.same({ "loadlib", "open" }, env.calls)
        assert.same({ true, "already_initialized" }, { runtime.initialize() })
    end)
end)
