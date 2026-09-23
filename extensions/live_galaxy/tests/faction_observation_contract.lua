local helper = require("test_helper")

local function values()
    local file = assert(io.open("config/heavy-ship-experiment.json", "rb"))
    local text = assert(file:read("*a"))
    assert(file:close())
    local result = {}
    for key, value in text:gmatch('"([%w_]+)"%s*:%s*(%d+)') do
        result[key] = tonumber(value)
    end
    return result
end

describe("dynamic faction observation contract", function()
    local fixture, profile, factions

    before_each(function()
        fixture = helper.new()
        profile = fixture.load("live_galaxy_ship_profile")
        factions = fixture.load("live_galaxy_factions")
    end)

    after_each(function()
        fixture.restore()
    end)

    for _, faction in ipairs({ "xenon", "khaak" }) do
        it("admits mandatory hostile observation subject " .. faction, function()
            local options, reason = profile.options(values(), faction)
            assert.is_nil(reason)
            assert.is_table(options)
            assert.equals("x4:faction:" .. faction .. ":ships", options.observation.source_scope)
        end)
    end

    local inventory = {
        argon = { origin = "vanilla", independent = true, mind_candidate = true, evidence = "source-v1" },
        scaleplate = { origin = "vanilla", independent = true, mind_candidate = false, evidence = "source-v1" },
        xenon = { origin = "vanilla", independent = true, mind_candidate = false, evidence = "source-v1" },
        khaak = { origin = "vanilla", independent = true, mind_candidate = false, evidence = "source-v1" },
        player = { origin = "player", independent = false, mind_candidate = false, evidence = "D-08" },
        visitor = { origin = "service", mind_candidate = false, evidence = "source-v1" },
    }

    local function api(values, filled)
        return {
            count_factions = function() return #values end,
            new_faction_buffer = function() return {} end,
            fill_factions = function(_, buffer, count)
                assert.equals(#values, count)
                for index, value in ipairs(values) do buffer[index - 1] = value end
                return filled or #values
            end,
            faction_string = function(_, value) return value end,
        }
    end

    it("preserves every included excluded and unknown identity", function()
        local values = { "xenon", "custom_mod", "argon", "player", "khaak", "scaleplate", "visitor" }
        local roster = assert(factions.capture(api(values), inventory, 9, {
            max_factions = 32, max_allocation_bytes = 4096, pointer_bytes = 8,
        }))
        assert.equals(9, roster.discovery_revision)
        assert.equals(#values, #roster.entries)
        assert.equals("included", roster.by_id.xenon.disposition)
        assert.equals("included", roster.by_id.khaak.disposition)
        assert.equals("excluded", roster.by_id.player.disposition)
        assert.equals("unknown", roster.by_id.custom_mod.disposition)
        assert.equals("unknown", roster.by_id.visitor.disposition)
        assert.is_false(roster.by_id.xenon.mind_candidate)
        assert.is_true(roster.unknown_blocker)
    end)

    it("atomically rejects count mismatch and duplicate identities", function()
        local roster, reason = factions.capture(api({ "argon" }, 0), inventory, 1, {
            max_factions = 32, max_allocation_bytes = 4096, pointer_bytes = 8,
        })
        assert.is_nil(roster)
        assert.equals("census_incomplete", reason)
        roster, reason = factions.capture(api({ "argon", "argon" }), inventory, 2, {
            max_factions = 32, max_allocation_bytes = 4096, pointer_bytes = 8,
        })
        assert.is_nil(roster)
        assert.equals("duplicate_faction", reason)
    end)

    it("configures one shared production profile for the discovered faction set", function()
        local options = assert(profile.options(values(), { "argon", "xenon", "khaak" }, inventory))
        assert.same({ "argon", "xenon", "khaak" }, options.observation.faction_ids)
        assert.equals(inventory, options.observation.faction_inventory)
        assert.equals("x4:faction:argon:ships", options.observation.source_scope)
    end)

    it("refreshes the active census and fences stale selections at a source boundary", function()
        local profile_options = assert(profile.options(values(), { "argon", "xenon" }, inventory))
        local options = profile_options.observation
        local census = { "argon" }
        options.ship_api = api(census)
        options.ship_api.count_ships = function() return 0 end
        options.ship_api.new_buffer = function() return {} end
        options.ship_api.fill_ships = function() return 0 end
        options.clock_getter = function() return 100 end
        local adapter = {
            begin_evidence = function() return { capture_start_millis = "100" } end,
            finish_evidence = function() return { capture_end_millis = "101" } end,
        }
        assert(fixture.load("live_galaxy_ship_selection").attach(adapter, options))
        local carrier = {
            progress = function() return 0, { monotonic_millis = "100" } end,
            begin_section = function() return 0 end,
            finish_section = function() return 0 end,
            fail_section = function() return 0 end,
        }
        local first = adapter:advance({ source_boundary = "runtime_start" }, carrier, {
            selection = "ship_core:argon", collection_revision = "1",
            producer_incarnation = "run-1", monotonic_millis = "100",
        })
        assert.equals("sampled", first.disposition)

        census[1], census[2] = "argon", "xenon"
        local stale = adapter:advance({ source_boundary = "game_loaded" }, carrier, {
            selection = "ship_core:xenon", collection_revision = "2",
            producer_incarnation = "run-1", monotonic_millis = "100",
        })
        assert.equals("source_boundary_changed", stale.disposition)
        local refreshed = adapter:advance({ source_boundary = "game_loaded" }, carrier, {
            selection = "ship_core:xenon", collection_revision = "2",
            producer_incarnation = "run-1", monotonic_millis = "100",
        })
        assert.equals("sampled", refreshed.disposition)
        assert.equals(2, refreshed.discovery_revision)
    end)

    it("restarts an interrupted census from record one after a boundary", function()
        local options = assert(profile.options(values(), { "argon", "xenon" }, inventory)).observation
        local discovered = { "argon", "scaleplate" }
        options.ship_api = api(discovered)
        local adapter = {
            begin_evidence = function() return { capture_start_millis = "100" } end,
            finish_evidence = function() return { capture_end_millis = "101" } end,
        }
        assert(fixture.load("live_galaxy_ship_selection").attach(adapter, options))
        local pushed, begins, resets = {}, 0, 0
        local carrier = {
            begin_section = function() begins = begins + 1; return 0 end,
            push_record = function(_, record)
                if #pushed == 1 and resets == 0 then return -21 end
                pushed[#pushed + 1] = { id = record.faction_id, revision = record.discovery_revision }
                return 0
            end,
            finish_section = function() return 0 end,
            reset = function() resets = resets + 1 end,
        }
        local request = { selection = "faction_census", collection_revision = "1",
            producer_incarnation = "run-1" }
        assert.equals("collecting", adapter:advance({ source_boundary = "runtime_start" }, carrier, request).disposition)
        assert.equals("producer_busy", adapter:advance({ source_boundary = "runtime_start" }, carrier, request).disposition)
        assert.same({ { id = "argon", revision = 1 } }, pushed)

        adapter:feedback({ source_boundary = "game_loaded" }, carrier, request, 9)
        discovered[1], discovered[2] = "xenon", nil
        request.collection_revision = "2"
        local outcome
        for _ = 1, 3 do
            outcome = adapter:advance({ source_boundary = "game_loaded" }, carrier, request)
            if outcome.disposition == "sampled" then break end
        end
        assert.equals("sampled", outcome.disposition)
        assert.equals(2, outcome.discovery_revision)
        assert.equals(2, begins)
        assert.equals(1, resets)
        assert.same({ { id = "argon", revision = 1 }, { id = "xenon", revision = 2 } }, pushed)
    end)
end)
