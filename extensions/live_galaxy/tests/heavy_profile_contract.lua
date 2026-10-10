local helper = require("test_helper")
local function values()
    local file = assert(io.open("config/heavy-ship-experiment.json", "rb"))
    local text = assert(file:read("*a")); assert(file:close())
    local result = {}
    for key, value in text:gmatch('"([%w_]+)"%s*:%s*(%d+)') do result[key] = tonumber(value) end
    return result
end
describe("shared heavy profile and bounded game work", function()
    local fixture, profile, budget
    before_each(function()
        fixture = helper.new(); profile = fixture.load("live_galaxy_ship_profile")
        budget = fixture.load("live_galaxy_ship_budget")
    end)
    after_each(function() fixture.restore() end)
    it("reads a stable sector identity from the native component context", function()
        package.loaded.ffi = { C = { GetContextByClass = function(component, class, include_self)
            assert.equals("9007199254740993ULL", component)
            assert.equals("sector", class); assert.is_false(include_self)
            return "18446744073709551615ULL"
        end } }
        _G.ConvertStringToLuaID = function(id) assert.equals("9007199254740993", id); return "external-component" end
        _G.ConvertIDTo64Bit = function(component)
            assert.equals("external-component", component); return "9007199254740993ULL"
        end
        _G.GetComponentData = function(component, ...)
            assert.equals("external-component", component)
            assert.same({ "owner", "macro", "classid" }, { ... })
            return "argon", "macro", 123
        end
        local api = fixture.load("live_galaxy_ship_source").runtime()
        local wrapped = budget.new(api, values())
        assert(wrapped:before({ monotonic_millis = "10" }))
        local value, rejection = wrapped.api:read_core("9007199254740993")
        assert.is_nil(rejection)
        assert.same({ identity = "9007199254740993", owner = "argon", type = "macro",
            class = "classid:123", location = "sector:18446744073709551615" }, value)
        assert.equals("read_core", wrapped.operation)
    end)
    it("canonicalizes signed zero and rejects malformed component classes before sector lookup", function()
        for _, case in ipairs({
            { raw = 0, expected = "classid:0" }, { raw = -0.0, expected = "classid:0" },
            { raw = 1.5 }, { raw = -1 }, { raw = 9007199254740992 }, { raw = {} },
        }) do
            local context_calls = 0
            package.loaded.ffi = { C = { GetContextByClass = function()
                context_calls = context_calls + 1; return "1ULL"
            end } }
            _G.ConvertStringToLuaID = function() return "external-component" end
            _G.ConvertIDTo64Bit = function() return "9007199254740993ULL" end
            _G.GetComponentData = function() return "argon", "macro", case.raw end
            local api = fixture.load("live_galaxy_ship_source").runtime()
            local value, rejection = api:read_core("9007199254740993")
            if case.expected then
                assert.equals(case.expected, value.class); assert.is_nil(rejection)
                assert.equals(1, context_calls)
            else
                assert.is_nil(value)
                assert.same({ condition = "token_invalid", field = "class",
                    observed_type = type(case.raw) }, rejection)
                assert.equals(0, context_calls)
            end
        end
    end)
    it("propagates an invalid native sector identity through the budget wrapper", function()
        package.loaded.ffi = { C = { GetContextByClass = function() return 123 end } }
        _G.ConvertStringToLuaID = function() return "external-component" end
        _G.ConvertIDTo64Bit = function() return "9007199254740993ULL" end
        _G.GetComponentData = function() return "argon", "macro", "ship" end
        local api = fixture.load("live_galaxy_ship_source").runtime()
        local wrapped = budget.new(api, values())
        assert(wrapped:before({ monotonic_millis = "10" }))
        local value, rejection = wrapped.api:read_core("9007199254740993")
        assert.is_nil(value)
        assert.same({ condition = "identity_invalid", field = "sectorid", observed_type = "number" }, rejection)
        assert.equals("read_core", wrapped.operation)
    end)
    it("retains a ship with no sector context as an explicit location state", function()
        package.loaded.ffi = { C = { GetContextByClass = function() return "0ULL" end } }
        _G.ConvertStringToLuaID = function() return "external-component" end
        _G.ConvertIDTo64Bit = function() return "9007199254740993ULL" end
        _G.GetComponentData = function() return "argon", "macro", "ship" end
        local api = fixture.load("live_galaxy_ship_source").runtime()
        local value, rejection = api:read_core("9007199254740993")
        assert.is_nil(rejection)
        assert.same({ identity = "9007199254740993", owner = "argon", type = "macro",
            class = "ship", location = "context:none" }, value)
    end)
    it("materializes the same strict profile for source and actual native consumers", function()
        local v = values()
        local options = assert(profile.options(v, "argon"))
        assert.equals(v.max_inner_records, options.carrier.limits.max_inner_records)
        assert.equals(v.max_delivery_attempts, options.carrier.limits.max_attempts)
        assert.equals(v.max_candidate_raw_bytes, options.carrier.limits.max_content_bytes)
        assert.equals(v.max_collection_steps, options.observation.ship_limits.max_work)
        for _, key in ipairs({ "heavy_permits", "max_native_calls", "max_allocation_bytes", "admission_window_millis" }) do
            local bad = values(); bad[key] = 0; assert.is_nil(profile.options(bad, "argon"))
        end
        local bad = values(); bad.unknown = 1; assert.is_nil(profile.options(bad, "argon"))
        bad = values(); bad.experimental_profile = 2; assert.is_nil(profile.options(bad, "argon"))
        bad = values(); bad.max_aggregate_records = 127; assert.is_nil(profile.options(bad, "argon"))
        bad = values(); bad.max_aggregate_work = 131071; assert.is_nil(profile.options(bad, "argon"))
        assert.is_table(profile.options(v, "xenon"))
        assert.is_table(profile.options(v, "khaak"))
        assert.is_nil(profile.options(v, "player"))
    end)
    it("bounds cumulative allocation and native calls before entering the seam", function()
        local v = values(); v.max_total_allocation_bytes = 16; v.max_native_calls = 1
        local entered = 0
        local b = budget.new({ new_buffer = function() entered = entered + 1; return {} end,
            count_ships = function() entered = entered + 1; return 1 end }, v)
        assert(b:before({ monotonic_millis = "10" })); b.api:new_buffer(2)
        assert(b:before({ monotonic_millis = "11" }))
        assert.has_error(function() b.api:new_buffer(1) end, "allocation_limit")
        assert.equals(1, entered)
        local c = budget.new({ count_ships = function() entered = entered + 1; return 1 end }, v)
        assert(c:before({ monotonic_millis = "10" })); c.api:count_ships()
        assert.has_error(function() c.api:count_ships() end, "native_call_limit")
        assert(c:before({ monotonic_millis = "11" }))
        assert.has_error(function() c.api:count_ships() end, "native_call_limit")
        assert.equals(2, entered)
    end)
    it("refuses frozen/backward clocks and measures a synchronous overrun after return", function()
        local v, entered = values(), 0
        local b = budget.new({ count_ships = function() entered = entered + 1; return 1 end }, v)
        assert(b:before({ monotonic_millis = "10" })); b.api:count_ships()
        local carrier = { progress = function(_, work) assert.equals(0, work); return 0, { monotonic_millis = "13" } end }
        local ok, reason = b:after(carrier)
        assert.is_true(ok); assert.is_nil(reason); assert.equals(1, entered)
        assert.equals(3, b.max_callback_duration); assert.equals(1, b.max_callback_overrun)
        ok, reason = b:before({ monotonic_millis = "10" })
        assert.is_nil(ok); assert.equals("clock_unavailable", reason)
        assert.is_nil(b:before({ monotonic_millis = "9" })); assert.equals(1, entered)
    end)
    it("yields after elapsed callback time without counting operations", function()
        local v, now = values(), 10
        local b = budget.new({}, v)
        local carrier = { progress = function()
            return 0, { monotonic_millis = tostring(now) }
        end }
        assert(b:before({ monotonic_millis = tostring(now) }))
        now = now + v.callback_budget_millis - 1
        assert.is_false(b:should_yield(carrier))
        now = now + 1
        assert.is_true(b:should_yield(carrier))
        assert.equals(v.callback_budget_millis, b.max_callback_duration)
        assert.equals(0, b.max_callback_overrun)
    end)
    it("attributes observed step intervals and callback waits without extra status reads", function()
        local now, samples = 10, 0
        local b = budget.new({ count_ships = function() now = now + 3; return 1 end }, values())
        local carrier = { progress = function(_, work)
            assert.equals(0, work)
            samples = samples + 1; now = now + 2
            return 0, { monotonic_millis = tostring(now) }
        end }
        assert(b:before({ monotonic_millis = "10" }))
        b.api:count_ships()
        now = now + 1 -- Owned processing outside the source seam.
        assert.is_true(b:should_yield(carrier)); assert(b:after(carrier))
        now = now + 50
        assert(b:before({ monotonic_millis = tostring(now) }))
        now = now + 1
        assert.is_true(b:should_yield(carrier)); assert(b:after(carrier))
        assert.equals(2, samples, "instrumentation must not add carrier progress calls")
        assert.is_table(b.timing, "component attribution must accompany cumulative age")
        assert.same({ count = 1, total_millis = 6, max_millis = 6 }, b.timing.intervals.source)
        assert.same({ count = 1, total_millis = 3, max_millis = 3 }, b.timing.intervals.processing)
        assert.same({ count = 2, total_millis = 9, max_millis = 6 }, b.timing.callbacks)
        assert.same({ count = 1, total_millis = 50, max_millis = 50 }, b.timing.between_callbacks)
        assert.equals(1, b.timing.operations.source)
        assert.equals(2, b.timing.status_samples)
        assert.equals("included_unmeasured", b.timing.sampling_overhead)
        assert.equals("carrier_monotonic_millis", b.timing.clock)
    end)
    it("retains coarse interval bounds and rejects frozen callbacks without inventing elapsed work", function()
        local now, samples = 10, 0
        local b = budget.new({ count_ships = function() return 1 end }, values())
        local carrier = { progress = function(_, work)
            assert.equals(0, work); samples = samples + 1
            return 0, { monotonic_millis = tostring(now) }
        end }
        assert(b:before({ monotonic_millis = "10" }))
        b.api:count_ships(); assert.is_false(b:should_yield(carrier))
        local saved = b:timing_snapshot()
        now = 14; assert.is_true(b:should_yield(carrier)); assert(b:after(carrier))
        assert.same({ count = 1, total_millis = 0, max_millis = 0 }, saved.intervals.source)
        assert.same({ count = 1, total_millis = 4, max_millis = 4 }, b.timing.callbacks)
        assert.is_nil(b:before({ monotonic_millis = "14" }))
        assert.is_nil(b:before({ monotonic_millis = "13" }))
        assert.equals(1, b.timing.callbacks.count); assert.equals(0, b.timing.between_callbacks.count)
        assert.equals(2, samples)
        now = 15; assert(b:before({ monotonic_millis = "15" }))
        now = 14
        local ok, reason = b:after(carrier)
        assert.is_nil(ok); assert.equals("clock_unavailable", reason)
        assert.equals(1, b.timing.unsampled_callbacks)
        assert.equals(4, b.timing.callbacks.total_millis)
    end)
    it("reports an intra-callback clock regression without double-counting elapsed time", function()
        local now = 10
        local b = budget.new({}, values())
        local carrier = { progress = function() return 0, { monotonic_millis = tostring(now) } end }
        assert(b:before({ monotonic_millis = "10" }))
        now = 14; assert.is_true(b:should_yield(carrier))
        now = 13; assert.is_true(b:should_yield(carrier))
        now = 15; assert.is_true(b:should_yield(carrier))
        assert.equals(1, b.timing.clock_regressions)
        assert.equals(5, b.timing.intervals.processing.total_millis)
        assert.equals(5, b.timing.callbacks.total_millis)
    end)
    it("keeps entered operation counts and the unmeasured tail after an aborted getter", function()
        local b = budget.new({ count_ships = function() error("injected getter failure") end }, values())
        assert(b:before({ monotonic_millis = "10" }))
        assert.has_error(function() b.api:count_ships() end, "injected getter failure")
        local timing = b:timing_snapshot()
        assert.equals(1, timing.operations.source)
        assert.equals(0, timing.status_samples); assert.equals(1, timing.unsampled_callbacks)
        assert.is_true(timing.pending_operations)
        assert.equals(0, timing.intervals.source.count)
    end)
    it("requires committed core feedback and revalidates one full-faction detail group", function()
        local v, now, finishes, failures, reads = values(), 0, 0, 0, 0
        local identities = { "9007199254740993", "9007199254740995" }
        local options = assert(profile.options(v, "argon")).observation
        options.ship_api = {
            list_factions = function() return { "argon" } end,
            count_ships = function() return #identities end,
            new_buffer = function() return { [0] = identities[1], [1] = identities[2] } end,
            fill_ships = function() return #identities end,
            read_core = function(_, identity)
                reads = reads + 1
                return { identity = identity, owner = "argon", type = "ship_macro",
                    class = "destroyer", location = "sector:1" }
            end,
            cargo_wares = function() return { ore = 1 } end,
            cargo_storage_count = function() return 0 end,
            cargo_storage_size = function() return 24 end,
            cargo_storage_allocate = function() return {} end,
            cargo_storage_fill = function() return {} end,
            crew_capacity = function() return 0 end,
            crew_count = function() return 0 end,
            crew_size = function() return 40 end,
            crew_allocate = function() return {} end,
            crew_fill = function() return {} end,
            crew_tier_size = function() return 16 end,
            crew_tier_allocate = function() return {} end,
            crew_tier_fill = function() return {} end,
            physical_count = function() return 0 end,
            virtual_count = function() return 0 end,
            software_count = function() return 0 end,
            software_size = function() return 24 end,
            software_allocate = function() return {} end,
            software_fill = function() return {} end,
            missiles_count = function() return 0 end,
            missiles_size = function() return 24 end,
            missiles_allocate = function() return {} end,
            missiles_fill = function() return {} end,
            units_count = function() return 0 end,
            units_size = function() return 24 end,
            units_allocate = function() return {} end,
            units_fill = function() return {} end,
        }
        local adapter = { clock_getter = function() return now end,
            begin_evidence = function() return { capture_start_millis = tostring(now) } end,
            finish_evidence = function() return { capture_end_millis = tostring(now) } end }
        assert(fixture.load("live_galaxy_ship_selection").attach(adapter, options))
        local carrier = { begin_section = function() return 0 end, push_record = function() return 0 end,
            finish_section = function() finishes = finishes + 1; return 0 end,
            fail_section = function() failures = failures + 1; return 0 end,
            progress = function() return 0, { monotonic_millis = tostring(now) } end }
        local context = { source_boundary = "runtime_start" }
        local status = { selection = "ship_core", collection_revision = "1", producer_incarnation = "7",
            producer_state = "ready" }
        local function step()
            now = now + 50; status.monotonic_millis = tostring(now)
            return adapter:advance(context, carrier, status)
        end
        for _ = 1, 30 do if step().disposition == "sampled" then break end end
        assert.equals(1, finishes); assert.equals(10, reads)
        adapter:feedback(context, carrier, status, 5)
        status.selection, status.collection_revision = "ship_cargo:g0", "2"
        for _ = 1, 30 do if step().disposition == "sampled" then break end end
        assert.equals(2, finishes); assert.equals(10, reads); assert.equals(0, failures)
        status.selection = "ship_cargo:g1"
        status.collection_revision = "4"
        local refusal = step()
        assert.equals("selection_unavailable", refusal.disposition)
        assert.same({ stage = "selection", condition = "selection_unavailable", section = "ship_cargo:g1",
            revision = "4", run = "7" }, refusal.rejection)
        assert.is_nil(refusal.capture_metrics, "the refused request did not capture source values")
        assert.equals(10, reads, "a missing group never starts a source read")
        now = v.admission_window_millis + 1000
        assert.equals("admission_window_exhausted", step().disposition)
        adapter:feedback(context, carrier, status, 0)
        assert.equals("admission_window_exhausted", step().disposition,
            "ordinary polls never renew an exhausted experiment")
        status.producer_state = "awaiting_compatibility"
        adapter:feedback(context, carrier, status, 0)
        status.producer_state, status.selection, status.collection_revision = "ready", "ship_core", "5"
        assert.equals("sampled", step().disposition,
            "a fresh bridge qualification opens a new finite experiment without reloading X4")
    end)
end)
