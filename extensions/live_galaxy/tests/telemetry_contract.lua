local normalize = require("live_galaxy_normalize")
local telemetry = require("live_galaxy_telemetry")

local cases = {}

function cases.normalizes_identity_source_time_version_and_quality()
    local section = normalize.normalize_section({
        entity_id = "sector:alpha",
        source = "x4_runtime",
        observed_at_unix_millis = 1725000000000,
        version = 1,
        quality = "partial",
    })

    assert(section.entity_id == "sector:alpha")
    assert(section.source == "x4_runtime")
    assert(section.observed_at_unix_millis == 1725000000000)
    assert(section.version == 1)
    assert(section.quality == "partial")
end

function cases.keeps_known_empty_unknown_and_unsupported_explicit()
    local qualities = { "known_empty", "unknown", "unsupported" }

    for index, quality in ipairs(qualities) do
        local section = normalize.normalize_section({
            entity_id = "sector:" .. index,
            source = "x4_runtime",
            observed_at_unix_millis = 1725000000000,
            version = 1,
            quality = quality,
        })
        assert(section.quality == quality)
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
                },
            }
        end,
    }

    local sections = telemetry.observe_runtime_scope(adapter, "sectors", 1)
    assert(#sections == 1)
    assert(sections[1].entity_id == "sector:dynamic")
end

return cases
