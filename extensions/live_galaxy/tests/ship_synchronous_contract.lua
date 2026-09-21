local helper = require("test_helper")

describe("synchronous selection capture with resumable delivery", function()
    local fixture
    before_each(function() fixture = helper.new() end)
    after_each(function() fixture.restore() end)

    local function add_empty_details(api, observed)
        local function hit() observed.count = observed.count + 1 end
        api.cargo_wares = function() hit(); return {} end
        api.cargo_storage_count = function() hit(); return 0 end
        api.cargo_storage_size = function() return 24 end
        api.cargo_storage_allocate = function() return {} end
        api.cargo_storage_fill = function() hit(); return {} end
        api.crew_capacity = function() hit(); return 0 end
        api.crew_count = function() hit(); return 0 end
        api.crew_size = function() return 40 end
        api.crew_allocate = function() return {} end
        api.crew_fill = function() hit(); return {} end
        api.crew_tier_size = function() return 16 end
        api.crew_tier_allocate = function() return {} end
        api.crew_tier_fill = function() hit(); return {} end
        api.physical_count = function() hit(); return 0 end
        api.virtual_count = function() hit(); return 0 end
        for _, family in ipairs({ "software", "missiles", "units" }) do
            api[family .. "_count"] = function() hit(); return 0 end
            api[family .. "_size"] = function() return 24 end
            api[family .. "_allocate"] = function() return {} end
            api[family .. "_fill"] = function() hit(); return {} end
        end
    end

    local function configured_case(mode)
        local file = assert(io.open("config/heavy-ship-experiment.json", "rb"))
        local text = assert(file:read("*a")); assert(file:close())
        local values = {}; for key, value in text:gmatch('"([%w_]+)"%s*:%s*(%d+)') do values[key] = tonumber(value) end
        if mode == "bytes" then
            for _, key in ipairs({ "complete_message_bytes", "max_candidate_raw_bytes", "max_candidate_records",
                "max_candidate_batches", "max_publication_records", "max_publication_content_bytes", "max_inner_records",
                "group_members" }) do values[key] = 200 end
        end
        local options = assert(fixture.load("live_galaxy_ship_profile").options(values, "argon")).observation
        local now, admitted, pushed, finished = 100, 0, 0, 0
        options.ship_api = {
            list_factions = function() return { "argon" } end, count_ships = function() return 2 end,
            new_buffer = function() return { [0] = "9007199254740993", [1] = "9007199254740995" } end,
            fill_ships = function() return 2 end,
            read_core = function(_, id)
                if mode == "failure" then error("native getter failed") end
                if mode == "age" then now = 30101 end
                if mode == "clock" then now = false end
                return { identity = id, owner = "argon", type = "macro", class = "ship", location = "sector:1" }
            end,
        }
        add_empty_details(options.ship_api, { count = 0 })
        local adapter = { begin_evidence = function() return { capture_start_millis = "1" } end,
            finish_evidence = function() return { capture_end_millis = "2" } end }
        assert(fixture.load("live_galaxy_ship_selection").attach(adapter, options))
        local carrier = {
            progress = function() return 0, { monotonic_millis = tostring(now) } end,
            begin_section = function() admitted = admitted + 1; return 0 end,
            push_record = function() pushed = pushed + 1; return 0 end,
            finish_section = function() finished = finished + 1; return 0 end,
            fail_section = function() return 0 end,
        }
        local result = adapter:advance({}, carrier, { selection = "ship_core", collection_revision = "1",
            producer_incarnation = "7", monotonic_millis = "100" })
        return result, admitted, pushed, finished
    end

    for _, case in ipairs({ { "age", "collection_overflow" }, { "clock", "clock_unavailable" }, { "bytes", "allocation_limit" },
        { "failure", "source_failure" } }) do
        it("refuses " .. case[1] .. " before native section admission or partial publication", function()
            local result, admitted, pushed, finished = configured_case(case[1])
            assert.equals(case[2], result.disposition)
            assert.same({ 0, 0, 0 }, { admitted, pushed, finished })
            if case[1] == "age" then assert.equals(30001, result.capture_metrics.max_callback_duration_millis) end
        end)
    end

    it("captures 950 exact independent cores in one callback beyond 2ms and never rereads on backpressure", function()
        local file = assert(io.open("config/heavy-ship-experiment.json", "rb"))
        local text = assert(file:read("*a")); assert(file:close())
        local values = {}; for key, value in text:gmatch('"([%w_]+)"%s*:%s*(%d+)') do values[key] = tonumber(value) end
        local options = assert(fixture.load("live_galaxy_ship_profile").options(values, "argon")).observation
        local now, reads, fills, finished, busy = 100, 0, 0, 0, true
        local reused, delivered, expected = {}, {}, {}
        for i = 1, 950 do expected[i] = "900719925474" .. string.format("%04d", i) end
        local detail_reads = { count = 0 }
        options.ship_api = {
            list_factions = function() return { "argon" } end,
            count_ships = function() return 950 end,
            new_buffer = function() return {} end,
            fill_ships = function(_, buffer, count)
                fills = fills + 1; for i = 0, count - 1 do buffer[i] = expected[count - i] end; return count
            end,
            read_core = function(_, id)
                reads = reads + 1; now = now + 1
                reused.identity, reused.owner, reused.type, reused.class, reused.location = id, "argon", "macro:" .. id, "ship", "sector:" .. id
                return reused
            end,
        }
        add_empty_details(options.ship_api, detail_reads)
        local adapter = {
            begin_evidence = function() return { capture_start_millis = tostring(now) } end,
            finish_evidence = function() return { capture_end_millis = tostring(now) } end,
            clock_getter = function() return now end,
        }
        assert(fixture.load("live_galaxy_ship_selection").attach(adapter, options))
        local carrier = {
            progress = function() return 0, { monotonic_millis = tostring(now) } end,
            begin_section = function() if busy then return -21 end; return 0 end,
            push_record = function(_, row)
                if busy then return -21 end
                delivered[#delivered + 1] = row; return 0
            end,
            finish_section = function() finished = finished + 1; return 0 end,
            fail_section = function(_, reason) error("unexpected refusal: " .. reason) end,
        }
        local status = { selection = "ship_core", collection_revision = "1", producer_incarnation = "7", monotonic_millis = "100" }
        local result = adapter:advance({}, carrier, status)
        assert.equals("producer_busy", result.disposition)
        assert.equals(4750, reads, "core and all detail revalidation reads must finish before returning")
        assert.is_true(detail_reads.count > 0, "all detail families must be captured in the same callback")
        assert.equals(1, fills); assert.equals(0, #delivered); assert.equals(0, finished)
        reused.location = "changed_after_capture"
        for _ = 1, 3 do
            now = now + 50; status.monotonic_millis = tostring(now)
            assert.equals("producer_busy", adapter:advance({}, carrier, status).disposition)
            assert.equals(4750, reads); assert.equals(1, fills)
        end
        busy = false; now = now + 50; status.monotonic_millis = tostring(now)
        result = adapter:advance({}, carrier, status)
        assert.equals("sampled", result.disposition); assert.equals(1, finished)
        assert.equals(950, #delivered); assert.equals(4750, reads)
        local captured_detail_reads = detail_reads.count
        assert.is_nil(adapter:feedback({ source_boundary = "runtime_start" }, carrier, status, 5))
        for revision, key in ipairs({ "ship_cargo:g0", "ship_crew:g0", "ship_loadout:g0" }) do
            now = now + 1
            status.selection, status.collection_revision, status.monotonic_millis = key, tostring(revision + 1), tostring(now)
            assert.equals("sampled", adapter:advance({}, carrier, status).disposition)
            assert.equals(4750, reads, "cached detail delivery must not reread X4 core state")
            assert.equals(captured_detail_reads, detail_reads.count, "cached detail delivery must not repeat getters")
        end
        assert.equals(3800, #delivered)
        for i, row in ipairs(delivered) do
            local member = expected[(i - 1) % 950 + 1]
            assert.equals(member, row.identity)
            if i <= 950 then
                assert.equals("macro:" .. member, row.type)
                assert.equals("sector:" .. member, row.location)
            end
        end
        assert.equals(4750, result.capture_metrics.max_callback_duration_millis)
        assert.equals(4748, result.capture_metrics.max_callback_overrun_millis)
    end)
end)
