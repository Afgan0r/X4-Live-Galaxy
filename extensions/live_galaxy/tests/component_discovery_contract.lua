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
    local api, calls = fake_api(17)
    local observation, err = discovery.new(api).read_observation(1)

    assert(observation == nil)
    assert(err == "enumeration_overflow")
    assert(calls.allocations == 0)
    assert(calls.convert == 0 and calls.metadata == 0 and calls.capacity == 0)
end

function cases.emits_sorted_real_station_facts_only_after_all_members_validate()
    local api, calls = fake_api()
    local observation = assert(discovery.new(api).read_observation(1))
    local facts = observation.runtime_facts

    assert(observation.entity_id == "sector:argon_prime")
    assert(facts.assets[1].id == "asset:station:10")
    assert(facts.assets[2].id == "asset:station:20")
    assert(calls.convert == 2 and calls.convert64 == 2 and calls.metadata == 2 and calls.capacity == 2)
    assert(calls.order[1] == "convert" and calls.order[2] == "convert64")
    assert(calls.order[3] == "convert" and calls.order[4] == "convert64")
    assert(calls.order[5] == "metadata" and calls.order[6] == "capacity")
end

function cases.serializes_the_existing_compact_station_envelope()
    local api = fake_api()
    local payload = assert(telemetry.produce_observation(discovery.new(api), 3))

    assert(payload:match('"entity_id":"sector:argon_prime"'))
    assert(payload:match('"x":%[{"i":"asset:station:10","p":"sector:argon_prime"},{"i":"asset:station:20","p":"sector:second_contact"}%]'))
    assert(payload:match('"c":%[{"i":"capacity:station:10","p":"asset:station:10","v":42},{"i":"capacity:station:20","p":"asset:station:20","v":24}%]'))
    assert(payload:match('"o":%[{"i":"ownership:station:10","p":"asset:station:10","n":"faction:argon"},{"i":"ownership:station:20","p":"asset:station:20","n":"faction:argon"}%]'))
end

function cases.rejects_a_syntactically_valid_owner_outside_the_declared_scope()
    local api = fake_api()
    local original_get_component_data = api.get_component_data
    api.get_component_data = function(_, station)
        local _, sector_id = original_get_component_data(api, station)
        return station == "station:20" and "faction:antigone" or "faction:argon", sector_id
    end

    local payload, err = telemetry.produce_observation(discovery.new(api))

    assert(payload == nil)
    assert(err == "facts_unsupported")
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

    local observation = assert(discovery.new(api).read_observation(1))

    assert(observation.runtime_facts.assets[1].id == "asset:station:10")
end

return cases
