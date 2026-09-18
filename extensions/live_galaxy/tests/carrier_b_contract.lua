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
        local calls, fact, begin, finish, monotonic = {}, nil, nil, nil, 10000
        local function record(name, result)
            return function(_, value)
                calls[#calls + 1] = name
                if name == "push_record" then fact = value end
                if name == "begin_section" then begin = value end
                if name == "finish_section" then finish = value end
                if options.throw == name then error("injected " .. name) end
                local current = result
                if name == "progress" and type(options.progress_code) == "function" then
                    current = options.progress_code()
                end
                if current == nil then current = 0 end
                if name == "progress" then
                    monotonic = monotonic + (options.monotonic_step or 1)
                    local capacity = options.capacity or "available:0"
                    if type(capacity) == "function" then capacity = capacity() end
                    return current, "ready", "connected", tostring(monotonic),
                        capacity, "producer:1", options.selection or "carrier_b_realtime_sample",
                        options.remaining or "1", options.revision or "1"
                end
                return current
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

    it("collects one selected faction through resumable count fill core stages", function()
        local env = native({ selection = "ship_core", remaining = "4" })
        local source_calls = {}
        local api = {
            list_factions = function()
                source_calls[#source_calls + 1] = "census"
                return { "argon", "teladi" }
            end,
            count_ships = function(_, faction)
                source_calls[#source_calls + 1] = "count:" .. faction
                return 2
            end,
            new_buffer = function(_, count)
                source_calls[#source_calls + 1] = "allocate:" .. count
                return {}
            end,
            fill_ships = function(_, buffer, count, faction)
                source_calls[#source_calls + 1] = "fill:" .. count .. ":" .. faction
                buffer[0], buffer[1] = "9007199254740995", "9007199254740993"
                return 2
            end,
            read_core = function(_, identity)
                source_calls[#source_calls + 1] = "core:" .. identity
                return { identity = identity, owner = "argon", type = "destroyer_macro",
                    class = "destroyer", location = "sector:argon_prime" }
            end,
        }
        local carrier = assert(fixture.load("live_galaxy_carrier").new({ loadlib = env.loadlib }))
        local observation = assert(fixture.load("live_galaxy_observation").new({
            profile = "ship_core", faction_id = "argon", ship_api = api,
            ship_limits = { max_records = 4, max_allocation_bytes = 128,
                max_work = 100, max_attempts = 2, max_age_millis = 1000,
                faction_pointer_bytes = 8 },
            getter = function() return 0 end, clock_getter = function() return 1 end,
        }))
        local scheduler = fixture.load("live_galaxy_scheduler")
        local result
        for _ = 1, 20 do
            local before = #env.calls
            result = scheduler.tick("telemetry_tick", carrier, observation)
            assert.equals("progress", env.calls[before + 1])
            assert.equals("poll_control", env.calls[before + 2])
            if result.disposition == "sampled" then break end
        end

        assert.same({ "census", "count:argon", "allocate:2", "fill:2:argon",
            "core:9007199254740993", "core:9007199254740993",
            "core:9007199254740995", "core:9007199254740995" }, source_calls)
        assert.equals("sampled", result.disposition)
        assert.equals("ship_core", env.fact().profile)
        assert.equals("9007199254740995", env.fact().identity)
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
        local env, tick, getter_calls = native(), nil, 0
        package.loadlib = env.loadlib
        _G.GetCurRealTime = function()
            getter_calls = getter_calls + 1
            return 2
        end
        package.preload.ffi = function()
            return { C = {
                GetCurrentGameTime = function() return getter_calls == 0 and 10.25 or 10.5 end,
            } }
        end
        _G.RegisterEvent = function(_, callback) tick = callback end
        fixture.runtime()
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

    it("retains one game-loaded boundary until a sample is admitted", function()
        local progress = 0
        local env, tick = native({
            capacity = function()
                progress = progress + 1
                return progress == 1 and "occupied:1" or "available:0"
            end,
        }), nil
        package.loadlib = env.loadlib
        _G.GetCurRealTime = function() return 2 end
        package.preload.ffi = function()
            return { C = { GetCurrentGameTime = function() return 10 end } }
        end
        _G.RegisterEvent = function(_, callback) tick = callback end
        fixture.runtime()

        assert.same({ false, "producer_busy" }, {
            tick("live_galaxy_observation", "telemetry_game_loaded"),
        })
        assert.same({ true, "sampled" }, {
            tick("live_galaxy_observation", "telemetry_tick"),
        })
        assert.equals("game_loaded", env.begin().source_boundary)
        assert.same({ true, "sampled" }, {
            tick("live_galaxy_observation", "telemetry_tick"),
        })
        assert.equals("runtime_start", env.begin().source_boundary)
    end)

    it("redacts callback paths and control characters", function()
        local env, tick, diagnostic = native({ throw = "progress" }), nil, nil
        package.loadlib = env.loadlib
        _G.GetCurRealTime = function() return 2 end
        package.preload.ffi = function()
            return { C = { GetCurrentGameTime = function() return 0 end } }
        end
        _G.DebugError = function(value) diagnostic = value end
        _G.RegisterEvent = function(_, callback) tick = callback end
        fixture.runtime()
        tick("live_galaxy_observation", "telemetry_tick")
        assert.equals("Live Galaxy Carrier B: event=transition detail=operation_failure", diagnostic)
        assert.is_nil(diagnostic:match("[A-Z]:\\"))
        assert.is_nil(diagnostic:match("[%c]"))
    end)
    it("keeps valid work running while exposing logger failure and a bounded recovery gap", function()
        local env, tick, samples, recovered = native(), nil, 0, nil
        package.loadlib = env.loadlib
        _G.GetCurRealTime = function() samples = samples + 1; return 2 end
        package.preload.ffi = function() return { C = { GetCurrentGameTime = function() return 0 end } } end
        _G.DebugError = function() error("failed sink") end
        _G.RegisterEvent = function(_, callback) tick = callback end
        fixture.runtime()
        assert.same({ false, "diagnostic_failure" }, { tick("live_galaxy_observation", "telemetry_tick") })
        assert.same({ false, "diagnostic_failure" }, { tick("live_galaxy_observation", "telemetry_tick") })
        assert.equals(2, samples, "diagnostic failure does not cancel valid observation work")
        _G.DebugError = function(value) recovered = value end
        assert.same({ true, "sampled" }, { tick("live_galaxy_observation", "telemetry_tick") })
        assert.is_truthy(recovered:match("diagnostic_gap_count=3"))
    end)

    it("renders measured heavy callback duration and overrun in ordinary runtime diagnostics", function()
        local env, tick, diagnostic = native({ selection = "ship_core", remaining = "1", monotonic_step = 10 }), nil, nil
        local file = assert(io.open("config/heavy-ship-experiment.json", "rb"))
        local text = assert(file:read("*a")); assert(file:close())
        local values = {}; for key, value in text:gmatch('"([%w_]+)"%s*:%s*(%d+)') do values[key] = tonumber(value) end
        local options = assert(fixture.load("live_galaxy_ship_profile").options(values, "argon")).observation
        options.getter, options.clock_getter = function() return 0 end, function() return 1 end
        options.ship_api = {
            list_factions = function() return { "argon" } end, count_ships = function() return 1 end,
            new_buffer = function() return { [0] = "9007199254740993" } end, fill_ships = function() return 1 end,
            read_core = function(_, id) return { identity = id, owner = "argon", type = "macro", class = "ship", location = "sector:1" } end,
        }
        _G.RegisterEvent = function(_, callback) tick = callback end
        _G.DebugError = function(value) diagnostic = value end
        local runtime = fixture.runtime()
        assert(runtime.initialize({ carrier = { loadlib = env.loadlib }, observation = options }))
        assert.same({ true, "sampled" }, { tick("live_galaxy_observation", "telemetry_tick") })
        assert.is_truthy(diagnostic:match("max_callback_duration_millis=20"))
        assert.is_truthy(diagnostic:match("max_callback_overrun_millis=18"))
        assert.is_truthy(diagnostic:match("source_value_bytes=%d+"))
    end)

    for _, case in ipairs({
        { field = "owner", value = "PRIVATE_OWNER", condition = "value_mismatch", observed_type = "string" },
        { field = "type", value = "PRIVATE/path\n", condition = "token_invalid", observed_type = "string" },
        { field = "class", value = false, condition = "token_invalid", observed_type = "boolean" },
        { field = "location", value = {}, condition = "token_invalid", observed_type = "table" },
        { field = "sectorid", condition = "identity_invalid", observed_type = "number", absent = true },
        { field = "core", condition = "source_value_cycle", cycle = true },
    }) do
        it("retains rejected " .. case.field .. " context through ordinary runtime logging", function()
            local env, tick, diagnostic = native({ selection = "ship_core", remaining = "1", monotonic_step = 10 }), nil, nil
            local messages = {}
            local file = assert(io.open("config/heavy-ship-experiment.json", "rb"))
            local text = assert(file:read("*a")); assert(file:close())
            local values = {}; for key, value in text:gmatch('"([%w_]+)"%s*:%s*(%d+)') do values[key] = tonumber(value) end
            local options = assert(fixture.load("live_galaxy_ship_profile").options(values, "argon")).observation
            options.getter, options.clock_getter = function() return 0 end, function() return 1 end
            options.ship_api = {
                list_factions = function() return { "argon" } end, count_ships = function() return 1 end,
                new_buffer = function() return { [0] = "9007199254740993" } end, fill_ships = function() return 1 end,
                read_core = function(_, id)
                    if case.absent then return nil, { field = "sectorid", condition = "identity_invalid", observed_type = "number" } end
                    local core = { identity = id, owner = "argon", type = "macro", class = "ship", location = "sector:1" }
                    if case.cycle then core.child = core else core[case.field] = case.value end
                    return core
                end,
            }
            _G.RegisterEvent = function(_, callback) tick = callback end
            _G.DebugError = function(value) diagnostic = value; messages[#messages + 1] = value end
            local runtime = fixture.runtime()
            assert(runtime.initialize({ carrier = { loadlib = env.loadlib }, observation = options }))
            assert.same({ false, "invalid_fact" }, { tick("live_galaxy_observation", "telemetry_tick") })
            assert.is_truthy(diagnostic:match("stage=core"))
            assert.is_truthy(diagnostic:match("condition=" .. case.condition))
            if not case.cycle then
                assert.is_truthy(diagnostic:match("field=" .. case.field))
                assert.is_truthy(diagnostic:match("observed_type=" .. case.observed_type))
            else assert.is_truthy(diagnostic:match("operation=read_core")) end
            assert.is_truthy(diagnostic:match("section=ship_core"))
            assert.is_truthy(diagnostic:match("revision=1"))
            assert.is_truthy(diagnostic:match("attempt=1"))
            assert.is_truthy(diagnostic:match("calls=%d+"))
            assert.is_nil(diagnostic:match("PRIVATE"))
            assert.is_nil(diagnostic:match("9007199254740993"))
            assert.is_nil(diagnostic:match("[%c]"))
            if case.field == "owner" then
                local terminal
                for _ = 1, 10 do
                    local _, reason = tick("live_galaxy_observation", "telemetry_tick")
                    if reason == "retry_exhausted" then terminal = true; break end
                end
                assert.is_true(terminal)
                local count = #messages
                for _ = 1, 3 do assert.same({ false, "retry_exhausted" }, { tick("live_galaxy_observation", "telemetry_tick") }) end
                assert.equals(count, #messages, "unchanged terminal attempt is logged once")
            end
        end)
    end

    it("logs initial detail refusal against the request without fabricated capture metrics", function()
        local env, tick, diagnostic = native({ selection = "ship_cargo:g1", revision = "4" }), nil, nil
        local file = assert(io.open("config/heavy-ship-experiment.json", "rb"))
        local text = assert(file:read("*a")); assert(file:close())
        local values = {}; for key, value in text:gmatch('"([%w_]+)"%s*:%s*(%d+)') do values[key] = tonumber(value) end
        local options = assert(fixture.load("live_galaxy_ship_profile").options(values, "argon")).observation
        options.getter, options.clock_getter = function() return 0 end, function() return 1 end
        options.ship_api = { read_core = function() error("unexpected source capture") end }
        _G.RegisterEvent = function(_, callback) tick = callback end
        _G.DebugError = function(value) diagnostic = value end
        local runtime = fixture.runtime()
        assert(runtime.initialize({ carrier = { loadlib = env.loadlib }, observation = options }))
        assert.same({ false, "stale_parent" }, { tick("live_galaxy_observation", "telemetry_tick") })
        assert.is_truthy(diagnostic:match("stage=selection"))
        assert.is_truthy(diagnostic:match("section=ship_cargo:g1"))
        assert.is_truthy(diagnostic:match("revision=4"))
        assert.is_truthy(diagnostic:match("run=producer:1"))
        assert.is_nil(diagnostic:match("duration_millis"))
        assert.is_nil(diagnostic:match("ordinal="))
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
        local env, getter_calls = native({ progress_code = 2, capacity = "occupied:1" }), 0
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
        assert.same({ "loadlib", "open", "progress", "poll_control" }, env.calls)
    end)

    it("pumps frequently without touching observation sources before admission", function()
        local env = native({ capacity = "occupied:0" })
        local begin_calls, getter_calls, finish_calls = 0, 0, 0
        local carrier = assert(fixture.load("live_galaxy_carrier").new({ loadlib = env.loadlib }))
        local observation = {
            begin_evidence = function() begin_calls = begin_calls + 1 end,
            capture = function() getter_calls = getter_calls + 1 end,
            finish_evidence = function() finish_calls = finish_calls + 1 end,
        }
        local scheduler = fixture.load("live_galaxy_scheduler")
        for _ = 1, 100 do
            local result = scheduler.tick("telemetry_tick", carrier, observation)
            assert.equals("producer_busy", result.disposition)
        end
        assert.same({ 0, 0, 0 }, { begin_calls, getter_calls, finish_calls })
        assert.equals(200, #env.calls - 2)
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
        local env, tick = native(), nil
        package.loadlib = env.loadlib
        _G.GetCurRealTime = function() return 2 end
        package.preload.ffi = function()
            return { C = { GetCurrentGameTime = function() return 0 end } }
        end
        _G.RegisterEvent = function(name, callback)
            assert.equals("live_galaxy_observation", name)
            tick = callback
        end
        local runtime = fixture.runtime()
        assert.is_function(tick)
        assert.same({ true, "initialized" }, { runtime.initialize() })
        assert.same({ "loadlib", "open" }, env.calls)
        assert.same({ true, "already_initialized" }, { runtime.initialize() })
    end)
end)
