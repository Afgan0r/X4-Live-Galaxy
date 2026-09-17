local helper = require("test_helper")

describe("bounded faction core continuation", function()
    local fixture, module
    before_each(function()
        fixture = helper.new()
        module = fixture.load("live_galaxy_ship_collection")
    end)
    after_each(function() fixture.restore() end)

    local function environment(options)
        options = options or {}
        local calls, records, finished, failed, now = {}, {}, 0, 0, 0
        local api = {
            list_factions = function() calls[#calls + 1] = "census"; return { "argon" } end,
            count_ships = function() calls[#calls + 1] = "count"; return options.count or 2 end,
            new_buffer = function() calls[#calls + 1] = "allocate"; return {} end,
            fill_ships = function(_, buffer)
                calls[#calls + 1] = "fill"
                buffer[0], buffer[1] = "9007199254740995", options.second or "9007199254740993"
                return options.filled or 2
            end,
            read_core = function(_, id)
                calls[#calls + 1] = "core"
                local core = { identity = id, owner = "argon", type = "destroyer_macro",
                    class = "destroyer", location = "sector:1" }
                if options.change and calls[#calls - 1] == "core" then core[options.change] = "changed" end
                return core
            end,
        }
        local clock = {
            begin_evidence = function() return { capture_start_millis = "1" } end,
            finish_evidence = function() return { capture_end_millis = "2" } end,
        }
        local carrier = {
            begin_section = function() calls[#calls + 1] = "reserve"; return 0 end,
            push_record = function(_, record)
                if options.busy then return -21 end
                records[#records + 1] = record; return 0
            end,
            finish_section = function(_, evidence)
                assert.equals("partial", evidence.coverage)
                assert.equals("observed_count_fill_only", evidence.consistency)
                finished = finished + 1; return 0
            end,
            fail_section = function() failed = failed + 1; records = {}; return 0 end,
        }
        local limits = { max_records = 4, max_allocation_bytes = options.allocation or 128,
            max_work = options.work or 100, max_attempts = 2, max_age_millis = options.age or 1000,
            faction_pointer_bytes = 8 }
        local collector = assert(module.new({ faction_id = "argon", ship_api = api, ship_limits = limits }, clock))
        return {
            collector = collector, calls = calls,
            step = function(boundary, frozen)
                if not frozen then now = now + 1 end
                return collector:tick({ source_boundary = boundary or "runtime_start" }, carrier,
                    { selection = "ship_core", monotonic_millis = tostring(now), producer_incarnation = "1" })
            end,
            result = function() return records, finished, failed end,
        }
    end

    it("collects 129 owned identities synchronously without a population quota", function()
        local now, pushed, identity_reads, busy = 0, {}, 0, true
        local buffer = setmetatable({}, { __index = function(_, i)
            identity_reads = identity_reads + 1; return tostring(129 - i)
        end })
        local api = { list_factions = function() return { "argon" } end,
            count_ships = function() return 129 end, new_buffer = function() return buffer end,
            fill_ships = function() return 129 end,
            read_core = function(_, id) return { identity = id, owner = "argon", type = "ship",
                class = "ship", location = "sector:1" } end }
        local collector = assert(module.new({ faction_id = "argon", ship_api = api,
            ship_limits = { max_records = 4, max_allocation_bytes = 4096, max_work = 2000,
                max_attempts = 1, max_age_millis = 30000, faction_pointer_bytes = 8 } },
            { begin_evidence = function() return {} end, finish_evidence = function() return {} end }))
        local carrier = { begin_section = function() return 0 end,
            push_record = function(_, row)
                if busy then busy = false; return -21 end
                pushed[#pushed + 1] = row.identity; return 0
            end, finish_section = function() return 0 end, fail_section = function() end }
        local result
        for _ = 1, 2000 do
            now = now + 1
            result = collector:tick({}, carrier, { selection = "ship_core", monotonic_millis = tostring(now),
                producer_incarnation = "1" })
            assert.equals(129, identity_reads, "one complete census, never repeated on busy delivery")
            if result.disposition ~= "collecting" and result.disposition ~= "producer_busy" then break end
        end
        assert.equals("sampled", result.disposition, "byte-safe census must not hit the old count quota")
        assert.equals(129, #pushed)
        local expected = {}; for i = 1, 129 do expected[i] = tostring(i) end; table.sort(expected)
        assert.same(expected, pushed)
    end)

    it("rejects limits and selection before touching X4", function()
        assert.same({ nil, "selection_unavailable" }, { module.new({}, {}) })
        assert.same({ nil, "limits_unavailable" }, { module.new({ faction_id = "argon" }, {}) })
    end)

    for _, case in ipairs({
        { options = { filled = 1 }, reason = "enumeration_incomplete" },
        { options = { second = "9007199254740995" }, reason = "identity_invalid" },
        { options = { second = 9007199254740992 }, reason = "identity_invalid" },
        { options = { change = "owner" }, reason = "core_changed" },
        { options = { change = "location" }, reason = "core_changed" },
        { options = { work = 8 }, reason = "collection_overflow" },
        { options = { age = 3, busy = true }, reason = "collection_overflow" },
    }) do
        it("abandons the whole attempt for " .. case.reason, function()
            local env, result = environment(case.options), nil
            for _ = 1, 20 do
                result = env.step()
                if result.disposition == case.reason then break end
            end
            assert.equals(case.reason, result.disposition)
            local records, finished = env.result()
            assert.equals(0, #records)
            assert.equals(0, finished)
        end)
    end

    it("bounds allocation before requesting the ship buffer", function()
        local env = environment({ allocation = 8 })
        env.step()
        assert.same({ "census", "count" }, env.calls)
    end)

    it("does no source work on a frozen callback clock", function()
        local env = environment()
        env.step()
        local before = #env.calls
        assert.equals("clock_unavailable", env.step(nil, true).disposition)
        assert.equals(before, #env.calls)
    end)

    it("discards reserved source work on a load boundary", function()
        local env = environment({ busy = true })
        assert.equals("producer_busy", env.step().disposition)
        assert.equals("source_boundary_changed", env.step("game_loaded").disposition)
        local records, finished, failed = env.result()
        assert.same({ 0, 0, 1 }, { #records, finished, failed })
        env.step("game_loaded")
        assert.equals("reserve", env.calls[#env.calls])
    end)

    it("never turns an empty observed count into known-empty", function()
        local env, result = environment({ count = 0 }), nil
        for _ = 1, 12 do
            result = env.step()
            if result.disposition == "empty_unproven" then break end
        end
        assert.equals("empty_unproven", result.disposition)
        local records, finished = env.result()
        assert.same({ 0, 0 }, { #records, finished })
    end)
end)
