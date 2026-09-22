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
end)
