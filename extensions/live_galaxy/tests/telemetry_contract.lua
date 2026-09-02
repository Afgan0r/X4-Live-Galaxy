local normalize = require("live_galaxy_normalize")
local telemetry = require("live_galaxy_telemetry")

local cases = {}

local function runtime_facts(entity_id, x4_game_time)
    local asset_id = "asset:" .. entity_id
    return {
        source = "x4_runtime",
        x4_game_time = x4_game_time,
        quality = "fresh",
        availability = "available",
        sectors = { { id = entity_id } },
        assets = { { id = asset_id, sector_id = entity_id } },
        capacity = { { id = "capacity:" .. entity_id, asset_id = asset_id, value = 1 } },
        ownership = { { id = "ownership:" .. entity_id, asset_id = asset_id, owner_id = "owner:argon" } },
    }
end

function cases.normalizes_identity_source_game_time_version_and_quality()
    local section = normalize.normalize_section({
        entity_id = "sector:alpha",
        source = "x4_runtime",
        version = 1,
        quality = "fresh",
        runtime_facts = runtime_facts("sector:alpha", 3600),
    })

    assert(section.entity_id == "sector:alpha")
    assert(section.source == "x4_runtime")
    assert(section.version == 1)
    assert(section.quality == "fresh")
    assert(section.runtime_facts.source == "x4_runtime")
    assert(section.runtime_facts.x4_game_time == 3600)
    assert(section.runtime_facts.quality == "fresh")
    assert(section.runtime_facts.availability == "available")
    assert(section.runtime_facts.sectors[1].id == "sector:alpha")
    assert(section.runtime_facts.assets[1].sector_id == "sector:alpha")
    assert(section.runtime_facts.capacity[1].value == 1)
    assert(section.runtime_facts.ownership[1].owner_id == "owner:argon")
end

function cases.rejects_invalid_runtime_game_time()
    for _, game_time in ipairs({ -1, 1.5, "3600" }) do
        local section, err = normalize.normalize_section({
            entity_id = "sector:alpha",
            source = "x4_runtime",
            version = 1,
            quality = "fresh",
            runtime_facts = runtime_facts("sector:alpha", game_time),
        })
        assert(section == nil)
        assert(err == "runtime_facts_invalid")
    end
end

function cases.rejects_nonfresh_metadata_without_required_runtime_facts()
    local qualities = { "known_empty", "unknown", "unsupported" }

    for index, quality in ipairs(qualities) do
        local section, err = normalize.normalize_section({
            entity_id = "sector:" .. index,
            source = "x4_runtime",
            version = 1,
            quality = quality,
        })
        assert(section == nil)
        assert(err == "runtime_facts_invalid")
    end
end

function cases.discovers_only_the_supplied_runtime_scope()
    local adapter = {
        list_scope = function(_, scope, limit)
            assert(scope == "sectors")
            assert(limit == 1)
            return {
                {
                    entity_id = "sector:dynamic",
                    source = "x4_runtime",
                    version = 2,
                    quality = "fresh",
                    runtime_facts = runtime_facts("sector:dynamic", 7200),
                },
            }
        end,
    }

    local sections = telemetry.observe_runtime_scope(adapter, "sectors", 1)
    assert(#sections == 1)
    assert(sections[1].entity_id == "sector:dynamic")
    assert(sections[1].source == "x4_runtime")
    assert(sections[1].version == 2)
    assert(sections[1].quality == "fresh")
    assert(sections[1].runtime_facts.x4_game_time == 7200)
end

return cases
