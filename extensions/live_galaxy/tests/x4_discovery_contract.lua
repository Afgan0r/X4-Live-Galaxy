package.path = package.path .. ";extensions/live_galaxy/lua/?.lua"

local component_discovery = require("live_galaxy_component_discovery")
local discovery = require("live_galaxy_x4_discovery")
local runtime = require("live_galaxy_runtime")
local cases = {}

local function fresh_runtime()
    package.loaded["live_galaxy_runtime"] = nil
    return require("live_galaxy_runtime")
end

local function station(sector_id, stable_id, capacity)
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
            capacity = { { id = "capacity:station:" .. stable_id, asset_id = asset_id, value = capacity } },
            ownership = {
                { id = "ownership:station:" .. stable_id, asset_id = asset_id, owner_id = "faction:argon" },
            },
        },
    }
end

local function station_generation(count)
    local observations = {}
    for index = 1, count do
        observations[index] = station(
            "sector:" .. string.format("%03d", index),
            string.format("%03d", index),
            index
        )
    end
    return observations
end

local function oversized_observation()
    local sectors, assets, capacity, ownership = {}, {}, {}, {}
    for index = 1, 16 do
        local suffix = string.format("%02d", index)
        local sector_id = "sector:" .. string.rep("s", 70) .. suffix
        local asset_id = "asset:" .. string.rep("a", 70) .. suffix
        sectors[index] = { id = sector_id }
        assets[index] = { id = asset_id, sector_id = sector_id }
        capacity[index] = { id = "capacity:" .. string.rep("c", 65) .. suffix, asset_id = asset_id, value = index }
        ownership[index] = {
            id = "ownership:" .. string.rep("o", 64) .. suffix,
            asset_id = asset_id,
            owner_id = "faction:argon",
        }
    end
    return {
        entity_id = assets[1].id,
        source = "x4_runtime",
        version = 1,
        quality = "fresh",
        runtime_facts = {
            source = "x4_runtime",
            quality = "fresh",
            availability = "available",
            sectors = sectors,
            assets = assets,
            capacity = capacity,
            ownership = ownership,
        },
    }
end

local function accepting_pipe(frames)
    return {
        write_raw = function(_, frame)
            frames[#frames + 1] = frame
            return true
        end,
        disconnect = function() end,
    }
end

function cases.sanitizes_embedded_version_without_exposing_unavailable_values()
    assert(runtime.sanitize_embedded_version(nil) == "unavailable")
    assert(runtime.sanitize_embedded_version(42) == "unavailable")
    assert(runtime.sanitize_embedded_version("") == "unavailable")
    assert(runtime.sanitize_embedded_version("Lua\1 5.4\255") == "Lua_ 5.4_")
    assert(#runtime.sanitize_embedded_version(string.rep("x", 65)) == 64)
end

function cases.production_discovery_never_selects_the_synthetic_sector_adapter()
    local adapter = discovery.new({
        get_clusters = function() return { "cluster:one" } end,
        get_sectors = function() return { "sector:alpha" } end,
        stable_id = function() return "100" end,
    })
    local observation, err = adapter.read_observation(1)

    assert(observation == nil)
    assert(err == "enumeration_unavailable")
end

function cases.runtime_drains_129_immutable_frames_through_bounded_fifo_resources()
    local isolated_runtime = fresh_runtime()
    isolated_runtime.set_discovery_adapter({
        read_observations = function() return station_generation(129) end,
    })
    local accepted = {}
    isolated_runtime.set_pipe_adapter(accepting_pipe(accepted))

    local completed
    for _ = 1, 400 do
        assert(select(2, isolated_runtime.handle_tick()) == "sent")
        local snapshot = isolated_runtime.fifo_snapshot()
        if snapshot.completed_generations == 1 then
            completed = snapshot
            break
        end
    end

    assert(completed ~= nil)
    assert(completed.depth_messages == 0 and completed.depth_bytes == 0)
    assert(completed.enqueued_messages == 129)
    assert(completed.local_handoff_messages == 129)
    assert(completed.enqueued_bytes == completed.local_handoff_bytes)
    assert(completed.max_depth_messages < 129)
    local observations, markers = 0, 0
    for _, frame in ipairs(accepted) do
        if frame:match('"type":"observation"') then observations = observations + 1 end
        if frame:match('"type":"complete_marker"') then markers = markers + 1 end
    end
    assert(observations == 129 and markers == 1)
end

function cases.runtime_retries_the_same_fifo_head_without_double_accounting()
    local isolated_runtime = fresh_runtime()
    isolated_runtime.set_discovery_adapter({
        read_observations = function() return station_generation(3) end,
    })
    local deferred, attempted = false, {}
    isolated_runtime.set_pipe_adapter({
        write_raw = function(_, frame)
            attempted[#attempted + 1] = frame
            if frame:match('"type":"observation"') and not deferred then
                deferred = true
                return false
            end
            return true
        end,
        disconnect = function() end,
    })

    for _ = 1, 3 do assert(select(2, isolated_runtime.handle_tick()) == "sent") end
    assert(select(2, isolated_runtime.handle_tick()) == "pipe_backpressure")
    local held = isolated_runtime.fifo_snapshot()
    assert(held.depth_messages > 0)
    assert(held.local_handoff_messages == 0)
    assert(select(2, isolated_runtime.handle_tick()) == "sent")
    local released = isolated_runtime.fifo_snapshot()
    assert(released.enqueued_messages == held.enqueued_messages)
    assert(released.enqueued_bytes == held.enqueued_bytes)
    assert(released.local_handoff_messages == 1)
    assert(attempted[#attempted] == attempted[#attempted - 1])
end

function cases.runtime_rejects_a_one_over_frame_before_fifo_accounting()
    local isolated_runtime = fresh_runtime()
    isolated_runtime.set_discovery_adapter({
        read_observations = function() return { oversized_observation() } end,
    })
    local accepted = {}
    isolated_runtime.set_pipe_adapter(accepting_pipe(accepted))

    for _ = 1, 4 do assert(select(2, isolated_runtime.handle_tick()) == "sent") end
    local snapshot = isolated_runtime.fifo_snapshot()
    assert(snapshot.enqueued_messages == 0 and snapshot.enqueued_bytes == 0)
    for _, frame in ipairs(accepted) do
        assert(not frame:match('"type":"observation"'))
        assert(not frame:match('"type":"complete_marker"'))
    end
end

function cases.runtime_discards_fifo_generation_and_restarts_after_reconnect()
    local isolated_runtime = fresh_runtime()
    isolated_runtime.set_discovery_adapter({
        read_observations = function() return station_generation(3) end,
    })
    local failed = false
    isolated_runtime.set_pipe_adapter({
        write_raw = function(_, frame)
            if frame:match('"type":"observation"') and not failed then
                failed = true
                error("connection lost")
            end
            return true
        end,
        disconnect = function() end,
    })

    for _ = 1, 4 do isolated_runtime.handle_tick() end
    local discarded = isolated_runtime.fifo_snapshot()
    assert(discarded.depth_messages == 0 and discarded.depth_bytes == 0)
    local hello, hello_kind, generation = isolated_runtime.next_payload()
    assert(hello_kind == "hello" and generation == 2)
    assert(hello:match('"generation":2'))
    local first, _, first_generation, first_sequence = isolated_runtime.next_payload()
    assert(first ~= nil and first_generation == 2 and first_sequence == 1)
end

function cases.runtime_never_marks_an_incomplete_or_health_only_generation()
    local isolated_runtime = fresh_runtime()
    isolated_runtime.set_discovery_adapter({
        read_observations = function() return nil, "facts_unsupported" end,
    })
    local accepted = {}
    isolated_runtime.set_pipe_adapter(accepting_pipe(accepted))

    for _ = 1, 20 do assert(select(2, isolated_runtime.handle_tick()) == "sent") end
    for _, frame in ipairs(accepted) do
        assert(not frame:match('"type":"complete_marker"'))
    end
end

function cases.runtime_emits_health_only_for_a_component_owner_scope_mismatch()
    local isolated_runtime = fresh_runtime()
    isolated_runtime.set_discovery_adapter(component_discovery.new({
        faction_id = "faction:argon",
        count_stations = function() return 1 end,
        new_buffer = function() return {} end,
        fill_stations = function(_, buffer) buffer[0] = "station:20" return 1 end,
        to_component = function(_, value) return value end,
        to_component64 = function() return "20" end,
        get_component_data = function() return "faction:antigone", "sector:second_contact" end,
        get_people_capacity = function() return 24 end,
    }))
    local accepted = {}
    isolated_runtime.set_pipe_adapter(accepting_pipe(accepted))

    for _ = 1, 8 do assert(select(2, isolated_runtime.handle_tick()) == "sent") end
    local unavailable = 0
    for _, frame in ipairs(accepted) do
        if frame:match('"status":"unavailable"') then unavailable = unavailable + 1 end
        assert(not frame:match('"type":"observation"'))
        assert(not frame:match('"type":"complete_marker"'))
    end
    assert(unavailable > 0)
end

function cases.runtime_emits_only_an_allowlisted_component_diagnostic_class()
    local previous_debug_error = DebugError
    local trace_events = {}
    package.loaded["live_galaxy/lua/live_galaxy_trace_config"] = {
        enabled = true,
        attempt_id = "d051-facts-class",
        max_frame_events = 64,
        version_diagnostic_enabled = false,
    }
    DebugError = function(message) trace_events[#trace_events + 1] = message end
    local isolated_runtime = fresh_runtime()
    local adapter = {
        read_observation = function() return nil, "facts_unsupported" end,
        diagnostic_class = function() return "owner_scope_mismatch" end,
    }

    assert(select(2, isolated_runtime.produce_discovery_payload(adapter, 1)) == "discovery_facts_unsupported")
    local class_events = 0
    for _, message in ipairs(trace_events) do
        if message:match("event=component_discovery_class detail=owner_scope_mismatch") then
            class_events = class_events + 1
        end
    end
    assert(class_events == 1)

    DebugError = previous_debug_error
    package.loaded["live_galaxy/lua/live_galaxy_trace_config"] = nil
    package.loaded["live_galaxy_runtime"] = nil
    require("live_galaxy_runtime")
end

function cases.runtime_discards_a_frame_that_crosses_a_restarted_pipe_connection()
    local writes, disconnects = 0, 0
    runtime.set_pipe_adapter({
        write_raw = function()
            writes = writes + 1
            if writes == 1 then error("first write lost connection") end
            return true
        end,
        disconnect = function() disconnects = disconnects + 1 end,
    })

    local sent, status = runtime.emit('{"type":"heartbeat"}')
    assert(not sent and status == "pipe_reconnect")
    assert(writes == 1 and disconnects == 1)
    runtime.set_pipe_adapter(nil)
end

return cases
