package.path = package.path .. ";extensions/live_galaxy/lua/?.lua"

local discovery = require("live_galaxy_component_discovery")
local telemetry = require("live_galaxy_telemetry")
local cases = {}

local function fake_api(count)
    local calls = { convert = 0, metadata = 0, capacity = 0, allocations = 0, order = {} }
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
        stable_id = function(_, station)
            return station:match("%d+")
        end,
        get_component_data = function(_, station)
            calls.metadata = calls.metadata + 1
            calls.order[#calls.order + 1] = "metadata"
            return station == "station:10" and "faction:argon" or "faction:antigone",
                station == "station:10" and "sector:argon_prime" or "sector:second_contact"
        end,
        get_people_capacity = function(_, station)
            calls.capacity = calls.capacity + 1
            calls.order[#calls.order + 1] = "capacity"
            return station == "station:10" and 42 or 24
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
    assert(calls.convert == 2 and calls.metadata == 2 and calls.capacity == 2)
    assert(calls.order[1] == "convert" and calls.order[2] == "convert")
end

function cases.serializes_the_existing_compact_station_envelope()
    local api = fake_api()
    local payload = assert(telemetry.produce_observation(discovery.new(api), 3))

    assert(payload:match('"entity_id":"sector:argon_prime"'))
    assert(payload:match('"x":%[{"i":"asset:station:10","p":"sector:argon_prime"},{"i":"asset:station:20","p":"sector:second_contact"}%]'))
    assert(payload:match('"c":%[{"i":"capacity:station:10","p":"asset:station:10","v":42},{"i":"capacity:station:20","p":"asset:station:20","v":24}%]'))
    assert(payload:match('"o":%[{"i":"ownership:station:10","p":"asset:station:10","n":"faction:argon"},{"i":"ownership:station:20","p":"asset:station:20","n":"faction:antigone"}%]'))
end

function cases.suppresses_every_invalid_stage_without_partial_observation()
    local api = fake_api()
    api.fill_stations = function() return 1 end
    local payload, err = telemetry.produce_observation(discovery.new(api))

    assert(payload == nil)
    assert(err == "enumeration_incomplete")
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
