local normalize = require("live_galaxy_normalize")
local telemetry = require("live_galaxy_telemetry")

local cases = {}

local function runtime_facts(entity_id, observed_at_unix_millis)
    local asset_id = "asset:" .. entity_id
    return {
        source = "x4_runtime",
        observed_at_unix_millis = observed_at_unix_millis,
        quality = "fresh",
        availability = "available",
        sectors = { { id = entity_id } },
        assets = { { id = asset_id, sector_id = entity_id } },
        capacity = { { id = "capacity:" .. entity_id, asset_id = asset_id, value = 1 } },
        ownership = { { id = "ownership:" .. entity_id, asset_id = asset_id, owner_id = "owner:argon" } },
    }
end

function cases.normalizes_identity_source_time_version_and_quality()
    local section = normalize.normalize_section({
        entity_id = "sector:alpha",
        source = "x4_runtime",
        observed_at_unix_millis = 1725000000000,
        version = 1,
        quality = "fresh",
        runtime_facts = runtime_facts("sector:alpha", 1725000000000),
    })

    assert(section.entity_id == "sector:alpha")
    assert(section.source == "x4_runtime")
    assert(section.observed_at_unix_millis == 1725000000000)
    assert(section.version == 1)
    assert(section.quality == "fresh")
end

function cases.rejects_nonfresh_metadata_without_required_runtime_facts()
    local qualities = { "known_empty", "unknown", "unsupported" }

    for index, quality in ipairs(qualities) do
        local section, err = normalize.normalize_section({
            entity_id = "sector:" .. index,
            source = "x4_runtime",
            observed_at_unix_millis = 1725000000000,
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
                    observed_at_unix_millis = 1725000000000,
                    version = 2,
                    quality = "fresh",
                    runtime_facts = runtime_facts("sector:dynamic", 1725000000000),
                },
            }
        end,
    }

    local sections = telemetry.observe_runtime_scope(adapter, "sectors", 1)
    assert(#sections == 1)
    assert(sections[1].entity_id == "sector:dynamic")
end

return cases
