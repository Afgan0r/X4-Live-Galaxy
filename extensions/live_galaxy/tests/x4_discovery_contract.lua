package.path = package.path .. ";extensions/live_galaxy/lua/?.lua"

local discovery = require("live_galaxy_x4_discovery")
local telemetry = require("live_galaxy_telemetry")
local runtime = require("live_galaxy_runtime")

local cases = {}

function cases.sanitizes_embedded_version_without_exposing_unavailable_values()
    assert(runtime.sanitize_embedded_version(nil) == "unavailable")
    assert(runtime.sanitize_embedded_version(42) == "unavailable")
    assert(runtime.sanitize_embedded_version("") == "unavailable")
    assert(runtime.sanitize_embedded_version("Lua\1 5.4\255") == "Lua_ 5.4_")
    assert(#runtime.sanitize_embedded_version(string.rep("x", 65)) == 64)
end

local function fake_api()
    local calls = { sectors = 0, metadata = 0, capacity = 0, ware_limit = 0 }
    local api = {
        get_clusters = function() return { "cluster:one" } end,
        get_sectors = function()
            calls.sectors = calls.sectors + 1
            return { "sector:zeta", "sector:alpha" }
        end,
        stable_id = function(_, sector)
            return sector == "sector:alpha" and "100" or "200"
        end,
        get_component_data = function(_, sector, ...)
            calls.metadata = calls.metadata + 1
            assert(sector == "sector:alpha")
            assert(select(1, ...) == "name")
            return "Alpha", "owner:argon", "macro:sector", { energycells = 20 }
        end,
        get_ware_production_limit = function(_, sector, ware)
            calls.ware_limit = calls.ware_limit + 1
            assert(sector == "sector:alpha")
            assert(ware == "energycells")
            return 100
        end,
        get_people_capacity = function(_, sector)
            calls.capacity = calls.capacity + 1
            assert(sector == "sector:alpha")
            return 42
        end,
        get_game_time = function() return 42 end,
    }
    return api, calls
end

function cases.produces_the_canonical_v2_runtime_facts_from_fake_discovery()
    local api, calls = fake_api()
    local adapter = discovery.new(api)
    local sections = telemetry.observe_runtime_scope(adapter, "sectors", 999)

    assert(#sections == 1)
    assert(sections[1].entity_id == "sector:100")
    assert(sections[1].source == "x4_runtime")
    assert(sections[1].quality == "fresh")
    assert(sections[1].runtime_facts.sectors[1].id == "sector:100")
    assert(sections[1].runtime_facts.assets[1].id == "asset:sector:100")
    assert(sections[1].runtime_facts.assets[1].sector_id == "sector:100")
    assert(sections[1].runtime_facts.capacity[1].id == "capacity:sector:100")
    assert(sections[1].runtime_facts.capacity[1].asset_id == "asset:sector:100")
    assert(sections[1].runtime_facts.capacity[1].value == 42)
    assert(sections[1].runtime_facts.ownership[1].id == "ownership:sector:100")
    assert(sections[1].runtime_facts.ownership[1].asset_id == "asset:sector:100")
    assert(sections[1].runtime_facts.ownership[1].owner_id == "owner:argon")
    assert(calls.sectors == 1)
    assert(calls.metadata == 1)
    assert(calls.ware_limit == 1)
    assert(calls.capacity == 1)
end

function cases.serializes_the_exact_admitted_v2_fact_envelope()
    local api = fake_api()
    local payload = telemetry.produce_observation(discovery.new(api))

    assert(payload:match('"entity_id":"sector:100"'))
    assert(not payload:match("observed_at_unix_millis"))
    assert(not payload:match("live_galaxy"))
    assert(payload:match('"quality":"fresh"'))
    assert(payload:match('"runtime_facts":{"r":"x4_runtime","g":42,"q":"fresh","a":"available","s":%[{"i":"sector:100"}%]'))
    assert(payload:match('"x":%[{"i":"asset:sector:100","p":"sector:100"}%]'))
    assert(payload:match('"c":%[{"i":"capacity:sector:100","p":"asset:sector:100","v":42}%]'))
    assert(payload:match('"o":%[{"i":"ownership:sector:100","p":"asset:sector:100","n":"owner:argon"}%]'))
end

function cases.keeps_unavailable_unsupported_and_invalid_values_explicit()
    local unavailable = discovery.new({})
    local value, err = unavailable.read_observation(1)
    assert(value == nil)
    assert(err == "adapter_unavailable")

    local unsupported = discovery.new({
        get_clusters = function() return { "cluster:one" } end,
        get_sectors = function() return { "sector:alpha" } end,
        stable_id = function() return "100" end,
        get_component_data = function() return nil, nil, nil, {} end,
        get_ware_production_limit = function() return nil end,
        get_people_capacity = function() return nil end,
        get_game_time = function() return 42 end,
    })
    local section, unsupported_err = unsupported.read_observation(1)
    assert(section == nil)
    assert(unsupported_err == "facts_unsupported")

    local invalid_capacity = discovery.new({
        get_clusters = function() return { "cluster:one" } end,
        get_sectors = function() return { "sector:alpha" } end,
        stable_id = function() return "100" end,
        get_component_data = function() return "Alpha", "owner", "macro", { energycells = 1 } end,
        get_ware_production_limit = function() return "not-a-number" end,
        get_people_capacity = function() return 1 end,
        get_game_time = function() return 42 end,
    })
    section, unsupported_err = invalid_capacity.read_observation(1)
    assert(section == nil)
    assert(unsupported_err == "facts_unsupported")

    local invalid = discovery.new({
        get_clusters = function() return { "cluster:one" } end,
        get_sectors = function() return { "sector:alpha" } end,
        stable_id = function() return "not valid!" end,
        get_component_data = function() return "Alpha", "owner", "macro", {} end,
        get_ware_production_limit = function() return 1 end,
        get_people_capacity = function() return 1 end,
        get_game_time = function() return 42 end,
    })
    value, err = invalid.read_observation(1)
    assert(value == nil)
    assert(err == "identity_invalid")
end

function cases.suppresses_a_metadata_only_observation_when_ownership_is_missing()
    local missing_owner = discovery.new({
        get_clusters = function() return { "cluster:one" } end,
        get_sectors = function() return { "sector:alpha" } end,
        stable_id = function() return "100" end,
        get_component_data = function() return "Alpha", nil, "macro", {} end,
        get_ware_production_limit = function() return 1 end,
        get_people_capacity = function() return 1 end,
        get_game_time = function() return 42 end,
    })

    local payload, err = telemetry.produce_observation(missing_owner)

    assert(payload == nil)
    assert(err == "facts_unsupported")
end

function cases.capability_probe_is_disabled_by_default_and_retains_unsupported()
    local events = {}
    local adapter = discovery.new({
        get_clusters = function() return { "cluster:one" } end,
        get_sectors = function() return { "sector:alpha" } end,
        stable_id = function() return "100" end,
        get_component_data = function() return nil, nil, nil, {} end,
        get_ware_production_limit = function() return nil end,
        get_people_capacity = function() return nil end,
    }, {
        enabled = false,
        attempt_id = "d09-probe",
        emit = function(event) events[#events + 1] = event end,
    })

    local observation, err = adapter.read_observation(1)

    assert(observation == nil)
    assert(err == "facts_unsupported")
    assert(#events == 0)
end

function cases.capability_probe_emits_one_closed_privacy_safe_vector()
    local events = {}
    local adapter = discovery.new({
        get_clusters = function() return { "cluster:one" } end,
        get_sectors = function() return { "sector:alpha" } end,
        stable_id = function() return "100" end,
        get_component_data = function() return "Alpha", "owner:argon", "macro:sector", { energycells = 20 } end,
        get_people_capacity = function() return "invalid-capacity" end,
        get_ware_production_limit = function() error("native ware failure: energycells") end,
    }, {
        enabled = true,
        attempt_id = "d09-probe",
        emit = function(event) events[#events + 1] = event end,
    })

    local observation, err = adapter.read_observation(1)

    assert(observation == nil)
    assert(err == "facts_unsupported")
    assert(#events == 1)
    assert(events[1].attempt_id == "d09-probe")
    assert(events[1].metadata_type == "ok")
    assert(events[1].owner_id_validity == "ok")
    assert(events[1].sector_capacity == "wrong_type")
    assert(events[1].first_cargo_ware_limit == "call_error")
    assert(events[1].sector == nil)
    assert(events[1].owner == nil)
    assert(events[1].name == nil)
    assert(events[1].macro == nil)
    assert(events[1].ware == nil)
    assert(events[1].value == nil)
    assert(events[1].error == nil)

    observation, err = adapter.read_observation(1)
    assert(observation == nil)
    assert(err == "facts_unsupported")
    assert(#events == 1)
end

function cases.capability_probe_handles_empty_cargo_and_invalid_attempts_without_emission()
    local events = {}
    local adapter = discovery.new({
        get_clusters = function() return { "cluster:one" } end,
        get_sectors = function() return { "sector:alpha" } end,
        stable_id = function() return "100" end,
        get_component_data = function() return "Alpha", "owner:argon", "macro:sector", {} end,
        get_people_capacity = function() return 42 end,
        get_ware_production_limit = function() return 99 end,
    }, {
        enabled = true,
        attempt_id = "",
        emit = function(event) events[#events + 1] = event end,
    })

    local observation, err = adapter.read_observation(1)
    assert(observation ~= nil)
    assert(err == nil)
    assert(#events == 0)
end

function cases.capability_probe_uses_not_applicable_only_for_empty_cargo()
    local events = {}
    local adapter = discovery.new({
        get_clusters = function() return { "cluster:one" } end,
        get_sectors = function() return { "sector:alpha" } end,
        stable_id = function() return "100" end,
        get_component_data = function() return "", "owner:argon", "macro:sector", {} end,
        get_people_capacity = function() return 42 end,
        get_ware_production_limit = function() error("must not be called") end,
    }, {
        enabled = true,
        attempt_id = "d09-empty-cargo",
        emit = function(event) events[#events + 1] = event end,
    })

    local observation, err = adapter.read_observation(1)

    assert(observation == nil)
    assert(err == "facts_unsupported")
    assert(#events == 1)
    assert(events[1].first_cargo_ware_limit == "not_applicable")
end

function cases.capability_probe_suppresses_a_failed_trace_write_after_one_attempt()
    local writes = 0
    local adapter = discovery.new({
        get_clusters = function() return { "cluster:one" } end,
        get_sectors = function() return { "sector:alpha" } end,
        stable_id = function() return "100" end,
        get_component_data = function() return nil, nil, nil, {} end,
        get_people_capacity = function() return nil end,
        get_ware_production_limit = function() return nil end,
    }, {
        enabled = true,
        attempt_id = "d09-write-failure",
        emit = function()
            writes = writes + 1
            error("trace backend unavailable")
        end,
    })

    assert(select(2, adapter.read_observation(1)) == "facts_unsupported")
    assert(select(2, adapter.read_observation(1)) == "facts_unsupported")
    assert(writes == 1)
end

function cases.runtime_wraps_discovery_without_the_retired_probe_payload()
    local body = runtime.produce_discovery_payload(discovery.new(fake_api()))

    assert(body:match('"entity_id":"sector:100"'))
    assert(not body:match("runtime_probe"))
    assert(not body:match("sector:live_galaxy"))
end

function cases.runtime_suppresses_observation_when_discovery_fails()
    local observation, err = runtime.produce_discovery_payload(discovery.new({}))

    assert(observation == nil)
    assert(err == "discovery_adapter_unavailable")
end

function cases.runtime_advances_past_a_failed_discovery_without_an_observation_frame()
    runtime.set_discovery_adapter(discovery.new({}))

    local hello, hello_kind = runtime.next_payload()
    local heartbeat, heartbeat_kind, _, heartbeat_sequence = runtime.next_payload()
    local health, health_kind, _, health_sequence = runtime.next_payload()
    local unavailable, unavailable_kind, _, unavailable_sequence = runtime.next_payload()
    local complete, complete_kind, _, complete_sequence = runtime.next_payload()
    local next_heartbeat, next_heartbeat_kind, _, next_heartbeat_sequence = runtime.next_payload()

    assert(hello_kind == "hello")
    assert(hello:match('"type":"hello"'))
    assert(hello:match('"game_build":"live%-galaxy%-x4%-build%-2"'))
    assert(hello:match('"live%-galaxy%-observation%-v2"'))
    assert(heartbeat_kind == "heartbeat" and heartbeat_sequence == 1)
    assert(health_kind == "runtime_health" and health_sequence == 2)
    assert(unavailable_kind == "runtime_health" and unavailable_sequence == 3)
    assert(unavailable:match('"status":"unavailable"'))
    assert(not unavailable:match('"type":"observation"'))
    assert(complete_kind == "complete_marker" and complete_sequence == 4)
    assert(next_heartbeat_kind == "heartbeat" and next_heartbeat_sequence == 5)
    assert(heartbeat ~= nil and health ~= nil and complete ~= nil and next_heartbeat ~= nil)

    runtime.set_discovery_adapter(nil)
end

function cases.runtime_discards_a_frame_that_crosses_a_restarted_pipe_connection()
    local writes, disconnects = 0, 0
    runtime.set_pipe_adapter({
        write_raw = function(_, _)
            writes = writes + 1
            if writes == 1 then error("first write lost connection") end
            return true
        end,
        disconnect = function(_) disconnects = disconnects + 1 end,
    })

    local sent, status = runtime.emit('{"type":"heartbeat"}')

    assert(not sent)
    assert(status == "pipe_reconnect")
    assert(writes == 1)
    assert(disconnects == 1)
    runtime.set_pipe_adapter(nil)
end

function cases.runtime_restarts_with_hello_after_a_pipe_write_failure()
    local writes = {}
    runtime.set_pipe_adapter({
        write_raw = function(_, payload)
            writes[#writes + 1] = payload
            if #writes == 1 then error("bridge connection closed") end
            return true
        end,
        disconnect = function(_) end,
    })

    local first_ok, first_status = runtime.handle_tick()
    local second_ok, second_status = runtime.handle_tick()

    assert(not first_ok)
    assert(first_status == "pipe_reconnect")
    assert(second_ok)
    assert(second_status == "sent")
    assert(#writes == 2)
    assert(writes[2]:match('"type":"hello"'))
    runtime.set_pipe_adapter({
        write_raw = function() error("reset connection state") end,
        disconnect = function() end,
    })
    assert(select(2, runtime.handle_tick()) == "pipe_reconnect")
    runtime.set_pipe_adapter(nil)
end

function cases.bounds_the_sector_scan_before_stable_identity_work()
    local maximum_calls = 0
    local maximum_sectors = {}
    for index = 1, 16 do maximum_sectors[index] = "sector:" .. index end
    local maximum = discovery.new({
        get_clusters = function() return { "cluster:one" } end,
        get_sectors = function() return maximum_sectors end,
        stable_id = function(_, sector)
            maximum_calls = maximum_calls + 1
            return sector:match("%d+")
        end,
        get_component_data = function() return "Alpha", "owner", "macro", {} end,
        get_ware_production_limit = function() return 1 end,
        get_people_capacity = function() return 1 end,
        get_game_time = function() return 42 end,
    })
    assert(maximum.read_observation(1) ~= nil)
    assert(maximum_calls == 16)

    local over_limit_calls = 0
    maximum_sectors[17] = "sector:17"
    local over_limit = discovery.new({
        get_clusters = function() return { "cluster:one" } end,
        get_sectors = function() return maximum_sectors end,
        stable_id = function(_, sector)
            over_limit_calls = over_limit_calls + 1
            return sector:match("%d+")
        end,
        get_component_data = function() return "Alpha", "owner", "macro", {} end,
        get_ware_production_limit = function() return 1 end,
        get_people_capacity = function() return 1 end,
        get_game_time = function() return 42 end,
    })
    local section, err = over_limit.read_observation(1)
    assert(section == nil)
    assert(err == "scope_incomplete")
    assert(over_limit_calls == 16)
end

return cases
