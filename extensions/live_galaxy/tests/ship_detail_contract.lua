local helper = require("test_helper")

describe("source-faithful resumable cargo", function()
    local fixture, module
    before_each(function()
        fixture = helper.new()
        module = fixture.load("live_galaxy_ship_details")
    end)
    after_each(function() fixture.restore() end)

    local function run(options)
        options = options or {}
        local calls, records, finished, failed = {}, {}, 0, 0
        local api = {
            read_core = function() calls[#calls + 1] = "revalidate"; return options.current_core end,
            cargo_wares = function() calls[#calls + 1] = "wares"; return options.wares or { ore = 17 } end,
            cargo_storage_count = function() calls[#calls + 1] = "count"; return options.count or 1 end,
            cargo_storage_size = function() return options.size or 24 end,
            cargo_storage_allocate = function() calls[#calls + 1] = "allocate"; return {} end,
            cargo_storage_fill = function()
                calls[#calls + 1] = "fill"
                return options.rows or { { transport = "solid", capacity_cubic_metres = 1000,
                    occupied_cubic_metres = 170 } }
            end,
        }
        local carrier = {
            begin_section = function() return 0 end,
            push_record = function(_, record) records[#records + 1] = record; return 0 end,
            finish_section = function() finished = finished + 1; return 0 end,
            fail_section = function() failed = failed + 1; return 0 end,
        }
        local collector = assert(module.new({ ship_api = api, max_inner = 2,
            expected_core = options.expected_core,
            max_allocation_bytes = 48, source_scope = "x4:faction:argon:ships",
            group = { key = "ship_cargo:g0", owner = "argon", core_revision = "7",
                members = { "9007199254740993" } } }, {
            begin_evidence = function() return { capture_start_millis = "100" } end,
            finish_evidence = function() return { capture_end_millis = "110" } end,
        }))
        local last
        for _ = 1, 20 do
            last = collector:tick({}, carrier, { selection = "ship_cargo:g0" })
            if last.disposition ~= "collecting" then break end
        end
        return calls, records, finished, failed, last
    end

    it("preserves item amount, volume and independent capture metadata", function()
        local calls, records, finished, failed = run()
        assert.same({ "wares", "count", "allocate", "fill" }, calls)
        assert.equals(1, finished); assert.equals(0, failed)
        assert.equals("9007199254740993", records[1].identity)
        assert.equals("7", records[1].core_revision)
        assert.equals("100", records[1].capture_start_millis)
        assert.equals("110", records[1].capture_end_millis)
        assert.equals(17, records[1].wares[1].amount_items)
        assert.equals(170, records[1].storage[1].occupied_cubic_metres)
    end)
    it("preserves the specific cargo field before failure cleanup", function()
        local _, records, finished, failed, outcome = run({ wares = { ore = "PRIVATE_AMOUNT" } })
        assert.equals("collection_overflow", outcome.disposition)
        assert.same({ stage = "ware_copy", condition = "integer_invalid", field = "amount",
            observed_type = "string", ordinal = 1 }, outcome.rejection)
        assert.equals(0, #records); assert.equals(0, finished); assert.equals(1, failed)
    end)
    it("captures eighty owned wares in one callback and resumes a busy handoff without getters", function()
        local raw, pushed, busy, pulses, reads, finishes = {}, nil, true, 0, 0, 0
        for i = 1, 80 do raw["ware" .. string.format("%03d", i)] = i end
        local collector = assert(module.new({ max_inner = 4096, max_allocation_bytes = 4096,
            source_scope = "x4:faction:argon:ships", group = { key = "ship_cargo:g0",
                owner = "argon", core_revision = "7", members = { "1" } }, ship_api = {
                cargo_wares = function() reads = reads + 1; return raw end, cargo_storage_count = function() return 0 end,
                cargo_storage_size = function() return 24 end, cargo_storage_allocate = function() return {} end,
                cargo_storage_fill = function() return {} end } }, {
                begin_evidence = function() return { capture_start_millis = "1" } end,
                finish_evidence = function() finishes = finishes + 1; return { capture_end_millis = "2" } end }))
        local carrier = { begin_section = function() return 0 end, finish_section = function() return 0 end,
            fail_section = function() error("valid eighty-row data must not fail") end,
            push_record = function(_, row) if busy then busy = false; return -21 end; pushed = row; return 0 end }
        local result
        result = collector:tick({}, carrier, { selection = "ship_cargo:g0" })
        assert.equals("producer_busy", result.disposition)
        assert.equals(80, #collector.pending.wares, "capture must finish in the first callback")
        assert.equals(1, reads)
        raw.ware001 = 999
        for _ = 1, 200 do
            result = collector:tick({}, carrier, { selection = "ship_cargo:g0" })
            pulses = pulses + 1
            if result.disposition == "sampled" then break end
            assert.is_true(result.disposition == "collecting" or result.disposition == "producer_busy")
        end
        assert.equals("sampled", result.disposition); assert.equals(1, pulses)
        assert.equals(1, reads); assert.equals(2, finishes, "no evidence getter on delivery retry")
        assert.equals(80, #pushed.wares)
        for i = 1, 80 do assert.equals(i, pushed.wares[i].amount_items) end
        assert.equals("producer_busy", collector:tick({}, carrier, { selection = "ship_cargo:g0" }).disposition)
        assert.equals(1, reads, "a finished collector waits for the next selection without looping")
    end)
    it("aborts a reserved native section when cached delivery fails", function()
        local failures = 0
        local collector = assert(module.new({ capture_only = true, max_inner = 2,
            max_allocation_bytes = 48, source_scope = "x4:faction:argon:ships",
            group = { key = "ship_cargo:g0", owner = "argon", core_revision = "7",
                members = { "9007199254740993" } }, ship_api = {
                cargo_wares = function() return {} end, cargo_storage_count = function() return 0 end,
                cargo_storage_size = function() return 24 end, cargo_storage_allocate = function() return {} end,
                cargo_storage_fill = function() return {} end } }, {
                begin_evidence = function() return { capture_start_millis = "100" } end,
                finish_evidence = function() return { capture_end_millis = "110" } end,
            }))
        local carrier = { begin_section = function() return 0 end, push_record = function() return -20 end,
            finish_section = function() return 0 end,
            fail_section = function() failures = failures + 1; return 0 end }
        assert.equals("captured", collector:tick({}, carrier, { selection = "ship_core" }).disposition)
        assert.is_true(collector:deliver())
        assert.equals("fact_rejected", collector:tick({}, carrier, { selection = "ship_cargo:g0" }).disposition)
        assert.equals(1, failures)
    end)
    it("rejects overflow before allocation or fill", function()
        for _, options in ipairs({ { count = 3 }, { size = 49 },
            { wares = { ore = 9007199254740992 } }, { wares = { a = 1, b = 2, c = 3 } } }) do
            local calls, records, finished, failed = run(options)
            assert.equals(0, #records); assert.equals(0, finished); assert.equals(1, failed)
            for _, call in ipairs(calls) do assert.is_not.equal("fill", call) end
        end
    end)
    it("uses returned storage rows and rejects malformed dimensions", function()
        local _, records, finished = run({ count = 2 })
        assert.equals(1, #records[1].storage); assert.equals(1, finished)
        local _, rejected, completed, failed = run({ rows = {
            { transport = "solid", capacity_cubic_metres = 1000, occupied_cubic_metres = 1001 } } })
        assert.equals(0, #rejected); assert.equals(0, completed); assert.equals(1, failed)
    end)
    it("marks only the changed ship stale without discarding the detail batch", function()
        local expected = { identity = "9007199254740993", owner = "argon", type = "ship_macro",
            class = "destroyer", location = "sector:argon_prime" }
        local _, _, completed, failed = run({ expected_core = expected, current_core = expected })
        assert.equals(1, completed); assert.equals(0, failed)
        local reasons = { identity = "missing", owner = "owner_changed", type = "core_changed",
            class = "core_changed", location = "location_changed" }
        for _, key in ipairs({ "identity", "owner", "type", "class", "location" }) do
            local changed = {}; for name, value in pairs(expected) do changed[name] = value end
            changed[key] = "changed"
            local _, records, finish, failures, last = run({ expected_core = expected, current_core = changed })
            assert.equals(1, #records)
            assert.equals("possibly_stale", records[1].consistency)
            assert.equals(reasons[key], records[1].consistency_reason)
            assert.equals(1, finish); assert.equals(0, failures); assert.equals("sampled", last.disposition)
        end
        local _, missing, finish, failures, last = run({ expected_core = expected, current_core = nil })
        assert.equals("possibly_stale", missing[1].consistency)
        assert.equals("missing", missing[1].consistency_reason)
        assert.equals(1, finish); assert.equals(0, failures); assert.equals("sampled", last.disposition)
    end)
end)

describe("resumable aggregate crew and installed state", function()
    local fixture, module
    before_each(function() fixture = helper.new(); module = fixture.load("live_galaxy_ship_details") end)
    after_each(function() fixture.restore() end)
    local function run(family, api, maximum)
        local record, failure, calls = nil, nil, 0
        local wrapped = {}
        for name, fn in pairs(api) do
            wrapped[name] = function(...)
                if not name:match("_size$") then calls = calls + 1 end
                return fn(...)
            end
        end
        local carrier = {
            begin_section = function() return 0 end,
            push_record = function(_, value) record = value; return 0 end,
            finish_section = function() return 0 end,
            fail_section = function(_, reason) failure = reason; return 0 end,
        }
        local key = family .. ":g0"
        local collector = assert(module.new({ ship_api = wrapped, max_inner = maximum or 8,
            max_allocation_bytes = 256, source_scope = "x4:faction:argon:ships",
            group = { key = key, owner = "argon", core_revision = "7", members = { "9007199254740993" } } }, {
            begin_evidence = function() return { capture_start_millis = "100" } end,
            finish_evidence = function() return { capture_end_millis = "110" } end,
        }))
        for _ = 1, 100 do
            local result = collector:tick({}, carrier, { selection = key })
            if result.disposition ~= "collecting" then return record, failure, result end
        end
        error("bounded fixture did not finish")
    end
    local function crew()
        return {
            crew_capacity = function() return 12 end, crew_count = function() return 2 end,
            crew_size = function() return 40 end, crew_allocate = function() return {} end,
            crew_fill = function() return {
                { id = "service", amount_people = 7, reported_numtiers = 1, canhire = true },
                { id = "passenger", amount_people = 2, reported_numtiers = 0, canhire = false } } end,
            crew_tier_size = function() return 16 end, crew_tier_allocate = function() return {} end,
            crew_tier_fill = function(_, _, role) return role == "service" and {
                { name = "Raw tier", skill_lower_threshold = -25, amount_people = 7 } } or {} end,
        }
    end
    it("keeps non-hireable roles, absent pilot and signed qualification tiers", function()
        local record, failure, result = run("ship_crew", crew())
        assert.is_nil(failure); assert.equals("sampled", result.disposition)
        assert.is_true(record.includepilot); assert.equals(12, record.capacity_people)
        assert.equals(2, #record.roles); assert.equals("passenger", record.roles[1].id)
        assert.is_false(record.roles[1].canhire); assert.equals(0, #record.roles[1].tiers)
        assert.equals(-25, record.roles[2].tiers[1].skill_lower_threshold)
    end)
    it("refuses crew count before allocation and raw tier overflow before publication", function()
        local api = crew(); api.crew_count = function() return 9 end
        local record, failure = run("ship_crew", api)
        assert.is_nil(record); assert.equals("collection_overflow", failure)
        api = crew(); api.crew_tier_fill = function(_, _, role) return role == "service" and {
            { name = "raw", skill_lower_threshold = 2147483648, amount_people = 7 } } or {} end
        record, failure = run("ship_crew", api)
        assert.is_nil(record); assert.equals("invalid_fact", failure)
    end)
    local function loadout()
        local api = {
            physical_count = function() return 1 end, physical_component = function() return "0" end,
            physical_macro = function(_, _, kind) return kind .. "_installed_macro" end,
            physical_group = function() return { path = "..", group = "" } end,
            virtual_count = function(_, _, kind) return kind == "thruster" and 1 or 0 end,
            virtual_macro = function() return "thruster_macro" end,
        }
        for _, family in ipairs({ "software", "missiles", "units" }) do
            api[family .. "_count"] = function() return 1 end
            api[family .. "_size"] = function() return 24 end
            api[family .. "_allocate"] = function() return {} end
        end
        api.software_fill = function() return { { maximum = "max", current = "current" } } end
        api.missiles_fill = function() return { { ware = "missile", macro_name = "macro", amount_raw = -3 } } end
        api.units_fill = function() return { { macro_name = "unit", category = "unfiltered_raw", amount_items = 2 } } end
        return api
    end
    it("keeps installed macro-only pseudo-groups and every distinct field family", function()
        local record, failure, result = run("ship_loadout", loadout())
        assert.is_nil(failure); assert.equals("sampled", result.disposition)
        assert.equals(4, #record.physical); assert.equals("0", record.physical[1].component)
        assert.equals("..", record.physical[1].path); assert.equals("", record.physical[1].group)
        assert.equals("thruster", record.virtual_slots[1].kind)
        assert.equals("current", record.software[1].current)
        assert.equals(-3, record.missiles[1].amount_raw)
        assert.equals("unfiltered_raw", record.units[1].category); assert.is_false(record.onlydrones)
    end)
    it("refuses cumulative physical slots and array allocation overflow", function()
        local record, failure = run("ship_loadout", loadout(), 3)
        assert.is_nil(record); assert.equals("collection_overflow", failure)
        local api = loadout(); api.software_size = function() return 257 end
        record, failure = run("ship_loadout", api)
        assert.is_nil(record); assert.equals("collection_overflow", failure)
    end)
end)
