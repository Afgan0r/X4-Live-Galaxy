package.path = package.path .. ";extensions/live_galaxy/lua/?.lua"

local discovery = require("live_galaxy_component_discovery")
local telemetry = require("live_galaxy_telemetry")
local cases = {}

local function fake_api(count)
    local calls = {
        convert = 0,
        convert64 = 0,
        metadata = 0,
        capacity = 0,
        allocations = 0,
        order = {},
    }
    local stations = { "station:20", "station:10" }
    return {
        faction_id = "faction:argon",
        count_stations = function() return count or #stations end,
        new_buffer = function(_, size)
            calls.allocations = calls.allocations + 1
            return {}
        end,
        fill_stations = function(_, buffer, size)
            for index = 0, size - 1 do buffer[index] = stations[index + 1] end
            return size
        end,
        to_component = function(_, raw_station)
            calls.convert = calls.convert + 1
            calls.order[#calls.order + 1] = "convert"
            return raw_station
        end,
        to_component64 = function(_, station)
            calls.convert64 = calls.convert64 + 1
            calls.order[#calls.order + 1] = "convert64"
            return station:match("%d+")
        end,
        get_component_data = function(_, station)
            calls.metadata = calls.metadata + 1
            calls.order[#calls.order + 1] = "metadata"
            return "faction:argon",
                station == "station:10" and "sector:argon_prime" or "sector:second_contact"
        end,
        get_people_capacity = function(_, component64)
            calls.capacity = calls.capacity + 1
            calls.order[#calls.order + 1] = "capacity"
            assert(component64 == "10" or component64 == "20")
            return component64 == "10" and 42 or 24
        end,
    }, calls
end

function cases.validates_the_complete_owner_scope_before_component_reads()
    local api, calls = fake_api(65)
    local observation, err = discovery.new(api).read_observation(1)

    assert(observation == nil)
    assert(err == "enumeration_overflow")
    assert(calls.allocations == 0)
    assert(calls.convert == 0 and calls.metadata == 0 and calls.capacity == 0)
end

function cases.emits_sorted_real_station_frames_only_after_all_members_validate()
    local api, calls = fake_api()
    local observations = assert(discovery.new(api).read_observations(1))
    local first, second = observations[1].runtime_facts, observations[2].runtime_facts

    assert(observations[1].entity_id == "asset:station:10")
    assert(observations[2].entity_id == "asset:station:20")
    assert(first.assets[1].id == "asset:station:10")
    assert(second.assets[1].id == "asset:station:20")
    assert(calls.convert == 2 and calls.convert64 == 2 and calls.metadata == 2 and calls.capacity == 2)
    assert(calls.order[1] == "convert" and calls.order[2] == "convert64")
    assert(calls.order[3] == "convert" and calls.order[4] == "convert64")
    assert(calls.order[5] == "metadata" and calls.order[6] == "capacity")
end

function cases.uses_a_native_faction_token_and_canonicalizes_owner_facts()
    local api = fake_api(1)
    local counted_faction, filled_faction
    api.native_faction_id = "argon"
    api.count_stations = function(_, faction_id)
        counted_faction = faction_id
        return 1
    end
    api.fill_stations = function(_, buffer, _, faction_id)
        filled_faction = faction_id
        buffer[0] = "station:10"
        return 1
    end
    api.get_component_data = function() return "argon", "sector:argon_prime" end
    api.canonical_owner_id = function(_, owner_id) return "faction:" .. owner_id end

    local observation = assert(discovery.new(api).read_observations(1))[1]

    assert(counted_faction == "argon")
    assert(filled_faction == "argon")
    assert(observation.runtime_facts.ownership[1].owner_id == "faction:argon")
end

function cases.serializes_each_station_in_its_own_compact_envelope()
    local api = fake_api()
    local payloads = assert(telemetry.produce_observations(discovery.new(api), 3))
    local payload = payloads[1]

    assert(#payloads == 2)
    assert(payload:match('"entity_id":"asset:station:10"'))
    assert(#payload <= 1800)
    assert(payload:match('"x":%[{"i":"asset:station:10","p":"sector:argon_prime"}%]'))
    assert(payload:match('"c":%[{"i":"capacity:station:10","p":"asset:station:10","v":42}%]'))
    assert(payload:match('"o":%[{"i":"ownership:station:10","p":"asset:station:10","n":"faction:argon"}%]'))
end

function cases.accepts_the_new_64_station_bound_without_aggregate_frames()
    local stations = {}
    for index = 1, 64 do stations[index] = string.format("station:%02d", index) end
    local api = {
        faction_id = "faction:argon",
        count_stations = function() return #stations end,
        new_buffer = function() return {} end,
        fill_stations = function(_, buffer)
            for index, station in ipairs(stations) do buffer[index - 1] = station end
            return #stations
        end,
        to_component = function(_, station) return station end,
        to_component64 = function(_, station) return station:match("%d+") end,
        get_component_data = function(_, station)
            return "faction:argon", "sector:station_" .. station:match("%d+")
        end,
        get_people_capacity = function() return 1 end,
    }

    local observations = assert(discovery.new(api).read_observations(4))

    assert(#observations == 64)
    assert(observations[1].entity_id == "asset:station:01")
    assert(observations[64].entity_id == "asset:station:64")
    for _, observation in ipairs(observations) do
        assert(#observation.runtime_facts.assets == 1)
        assert(#observation.runtime_facts.capacity == 1)
        assert(#observation.runtime_facts.ownership == 1)
    end
    local payloads = assert(telemetry.produce_observations(discovery.new(api), 4))
    for _, payload in ipairs(payloads) do assert(#payload <= 1800) end
end

local function station_observation(index)
    local stable_id = string.format("%03d", index)
    local sector_id = "sector:station_" .. stable_id
    local asset_id = "asset:station:" .. stable_id
    return {
        entity_id = asset_id,
        source = "x4_runtime",
        version = 1,
        quality = "fresh",
        runtime_facts = {
            source = "x4_runtime",
            quality = "fresh",
            availability = "available",
            sectors = { { id = sector_id } },
            assets = { { id = asset_id, sector_id = sector_id } },
            capacity = { { id = "capacity:station:" .. stable_id, asset_id = asset_id, value = index } },
            ownership = {
                { id = "ownership:station:" .. stable_id, asset_id = asset_id, owner_id = "faction:argon" },
            },
        },
    }
end

function cases.streams_129_station_frames_in_order_without_an_aggregate_ceiling()
    local observations = {}
    for index = 1, 129 do observations[index] = station_observation(index) end
    local adapter = { read_observations = function() return observations end }

    local source = assert(telemetry.produce_observation_source(adapter, 7))
    local payloads = {}
    while true do
        local frame = source.next_frame()
        if frame == nil then break end
        payloads[#payloads + 1] = frame
    end

    assert(#payloads == 129)
    assert(payloads[1]:match('"entity_id":"asset:station:001"'))
    assert(payloads[129]:match('"entity_id":"asset:station:129"'))
    local aggregate_bytes = 0
    for _, payload in ipairs(payloads) do aggregate_bytes = aggregate_bytes + #payload end
    assert(aggregate_bytes > telemetry.MAX_CANONICAL_ENCODED_FRAME_UTF8_BYTES)
end

function cases.enforces_the_canonical_encoded_frame_ceiling_per_frame()
    local ceiling = telemetry.MAX_CANONICAL_ENCODED_FRAME_UTF8_BYTES
    assert(ceiling == 1800)
    assert(telemetry.canonical_encoded_frame_utf8_bytes(string.rep("x", ceiling)) == ceiling)
    local bytes, err = telemetry.canonical_encoded_frame_utf8_bytes(string.rep("x", ceiling + 1))
    assert(bytes == nil)
    assert(err == "observation_oversized")
end

function cases.rejects_a_syntactically_valid_owner_outside_the_declared_scope()
    local api = fake_api()
    local original_get_component_data = api.get_component_data
    api.get_component_data = function(_, station)
        local _, sector_id = original_get_component_data(api, station)
        return station == "station:20" and "faction:antigone" or "faction:argon", sector_id
    end

    local adapter = discovery.new(api)
    local payload, err = telemetry.produce_observation(adapter)

    assert(payload == nil)
    assert(err == "facts_unsupported")
    assert(adapter.diagnostic_class() == "owner_scope_mismatch")
end

function cases.classifies_closed_post_enumeration_failures_without_changing_the_error()
    local failures = {
        {
            class = "metadata_unavailable",
            apply = function(api)
                api.get_component_data = function() error("private native text") end
            end,
        },
        {
            class = "owner_invalid",
            apply = function(api)
                api.get_component_data = function() return "invalid owner!", "sector:argon_prime" end
            end,
        },
        {
            class = "owner_scope_mismatch",
            apply = function(api)
                api.get_component_data = function() return "faction:antigone", "sector:argon_prime" end
            end,
        },
        {
            class = "sector_invalid",
            apply = function(api)
                api.get_component_data = function() return "faction:argon", "invalid sector!" end
            end,
        },
        {
            class = "capacity_unavailable",
            apply = function(api)
                api.get_people_capacity = function() error("private native text") end
            end,
        },
        {
            class = "capacity_invalid",
            apply = function(api)
                api.get_people_capacity = function() return "private value" end
            end,
        },
    }

    for _, failure in ipairs(failures) do
        local api = fake_api()
        failure.apply(api)
        local adapter = discovery.new(api)
        local payload, err = telemetry.produce_observation(adapter)

        assert(payload == nil)
        assert(err == "facts_unsupported")
        assert(adapter.diagnostic_class() == failure.class)
    end
end

function cases.clears_the_closed_diagnostic_class_after_a_successful_retry()
    local api = fake_api()
    api.get_people_capacity = function() return "private value" end
    local adapter = discovery.new(api)

    assert(select(2, adapter.read_observation(1)) == "facts_unsupported")
    assert(adapter.diagnostic_class() == "capacity_invalid")

    api.get_people_capacity = function(_, component64)
        return component64 == "10" and 42 or 24
    end
    assert(adapter.read_observations(1) ~= nil)
    assert(adapter.diagnostic_class() == nil)
end

function cases.classifies_an_actual_empty_owner_scope_without_changing_the_error()
    local api = fake_api()
    api.count_stations = function() return 0 end
    local adapter = discovery.new(api)
    local payload, err = telemetry.produce_observation(adapter)

    assert(payload == nil)
    assert(err == "facts_unsupported")
    assert(adapter.diagnostic_class() == "owner_scope_empty")
end

function cases.suppresses_every_invalid_stage_without_partial_observation()
    local api = fake_api()
    api.fill_stations = function() return 1 end
    local payload, err = telemetry.produce_observation(discovery.new(api))

    assert(payload == nil)
    assert(err == "enumeration_incomplete")
end

function cases.fails_closed_when_count_call_throws()
    local api, calls = fake_api()
    api.count_stations = function() error("native count unavailable") end
    local payload, err = telemetry.produce_observation(discovery.new(api))

    assert(payload == nil)
    assert(err == "enumeration_unavailable")
    assert(calls.allocations == 0)
end

function cases.fails_closed_when_component_metadata_call_throws()
    local api = fake_api()
    api.get_component_data = function() error("native metadata unavailable") end
    local payload, err = telemetry.produce_observation(discovery.new(api))

    assert(payload == nil)
    assert(err == "facts_unsupported")
end

function cases.fails_closed_when_component_capacity_call_throws()
    local api = fake_api()
    api.get_people_capacity = function() error("native capacity unavailable") end
    local payload, err = telemetry.produce_observation(discovery.new(api))

    assert(payload == nil)
    assert(err == "facts_unsupported")
end

function cases.fails_closed_when_a_raw_universe_id_cannot_be_converted()
    local api = fake_api()
    api.to_component = function() return nil end
    local payload, err = telemetry.produce_observation(discovery.new(api))

    assert(payload == nil)
    assert(err == "identity_invalid")
end

function cases.fails_closed_before_capacity_when_component64_conversion_fails()
    local api, calls = fake_api()
    api.to_component64 = function()
        calls.convert64 = calls.convert64 + 1
        error("native conversion unavailable")
    end
    local payload, err = telemetry.produce_observation(discovery.new(api))

    assert(payload == nil)
    assert(err == "identity_invalid")
    assert(calls.convert64 == 1)
    assert(calls.capacity == 0)
end

function cases.normalizes_native_count_and_fill_before_validation()
    local api = fake_api()
    api.count_stations = function() return "2" end
    api.fill_stations = function(_, buffer, size)
        buffer[0], buffer[1] = "station:20", "station:10"
        return "2"
    end

    local observation = assert(discovery.new(api).read_observations(1))[1]

    assert(observation.runtime_facts.assets[1].id == "asset:station:10")
end

return cases
