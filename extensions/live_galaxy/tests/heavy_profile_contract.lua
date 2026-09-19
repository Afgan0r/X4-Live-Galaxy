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
            return "argon", "macro", "ship"
        end
        local api = fixture.load("live_galaxy_ship_source").runtime()
        local wrapped = budget.new(api, values())
        assert(wrapped:before({ monotonic_millis = "10" }))
        local value, rejection = wrapped.api:read_core("9007199254740993")
        assert.is_nil(rejection)
        assert.same({ identity = "9007199254740993", owner = "argon", type = "macro",
            class = "ship", location = "sector:18446744073709551615" }, value)
        assert.equals("read_core", wrapped.operation)
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
        assert.is_nil(profile.options(v, "xenon"))
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
    it("requires committed core feedback and revalidates the configured single-member detail", function()
        local v, now, finishes, failures, reads = values(), 0, 0, 0, 0
        local fact = { identity = "9007199254740993", owner = "argon", type = "ship_macro",
            class = "destroyer", location = "sector:1" }
        local options = assert(profile.options(v, "argon")).observation
        options.ship_api = {
            list_factions = function() return { "argon" } end,
            count_ships = function() return 1 end,
            new_buffer = function() return { [0] = fact.identity } end,
            fill_ships = function() return 1 end,
            read_core = function() reads = reads + 1; return fact end,
            cargo_wares = function() return { ore = 1 } end,
            cargo_storage_count = function() return 0 end,
            cargo_storage_size = function() return 24 end,
            cargo_storage_allocate = function() return {} end,
            cargo_storage_fill = function() return {} end,
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
        local status = { selection = "ship_core", collection_revision = "1", producer_incarnation = "7" }
        local function step()
            now = now + 50; status.monotonic_millis = tostring(now)
            return adapter:advance(context, carrier, status)
        end
        for _ = 1, 30 do if step().disposition == "sampled" then break end end
        assert.equals(1, finishes); assert.equals(2, reads)
        adapter:feedback(context, carrier, status, 5)
        status.selection, status.collection_revision = "ship_cargo:g0", "2"
        for _ = 1, 30 do if step().disposition == "sampled" then break end end
        assert.equals(2, finishes); assert.equals(3, reads); assert.equals(0, failures)
        status.selection = "ship_cargo:g1"
        status.collection_revision = "4"
        local refusal = step()
        assert.equals("selection_unavailable", refusal.disposition)
        assert.same({ stage = "selection", condition = "selection_unavailable", section = "ship_cargo:g1",
            revision = "4", run = "7" }, refusal.rejection)
        assert.is_nil(refusal.capture_metrics, "the refused request did not capture source values")
        assert.equals(3, reads, "a missing group never starts a source read")
        now = v.admission_window_millis + 1000
        assert.equals("admission_window_exhausted", step().disposition)
    end)
end)
