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
            max_allocation_bytes = 48, source_scope = "x4:faction:argon:ships",
            group = { key = "ship_cargo:g0", owner = "argon", core_revision = "7",
                members = { "9007199254740993" } } }, {
            begin_evidence = function() return { capture_start_millis = "100" } end,
            finish_evidence = function() return { capture_end_millis = "110" } end,
        }))
        local last
        for _ = 1, 20 do
            local before = #calls
            last = collector:tick({}, carrier, { selection = "ship_cargo:g0" })
            assert.is_true(#calls - before <= 1, "one heavy getter/allocation per callback")
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
end)
