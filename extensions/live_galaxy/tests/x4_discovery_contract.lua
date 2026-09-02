local helper = require("test_helper")
local component_discovery, discovery, runtime, fixture
describe("x4_discovery", function()
after_each(function() if fixture then fixture.restore() end end)
before_each(function()
    fixture = helper.new()
    component_discovery = fixture.load("live_galaxy_component_discovery")
    discovery = fixture.load("live_galaxy_x4_discovery")
    runtime = fixture.runtime()
end)

local function fresh_runtime()
    return fixture.runtime()
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

it("sanitizes embedded version without exposing unavailable values", function()
    assert(runtime.sanitize_embedded_version(nil) == "unavailable")
    assert(runtime.sanitize_embedded_version(42) == "unavailable")
    assert(runtime.sanitize_embedded_version("") == "unavailable")
    assert(runtime.sanitize_embedded_version("Lua\1 5.4\255") == "Lua_ 5.4_")
    assert(#runtime.sanitize_embedded_version(string.rep("x", 65)) == 64)
end)

it("production discovery never selects the synthetic sector adapter", function()
    local adapter = discovery.new({
        get_clusters = function() return { "cluster:one" } end,
        get_sectors = function() return { "sector:alpha" } end,
        stable_id = function() return "100" end,
    })
    local observation, err = adapter.read_observation(1)

    assert(observation == nil)
    assert(err == "enumeration_unavailable")
end)

it("runtime drains 129 immutable frames through bounded fifo resources", function()
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
end)

it("runtime retries the same fifo head without double accounting", function()
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
end)

it("runtime rejects a one over frame before fifo accounting", function()
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
end)

it("runtime discards fifo generation and restarts after reconnect", function()
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
end)

it("runtime never marks an incomplete or health only generation", function()
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
end)

it("runtime emits health only for a component owner scope mismatch", function()
    local isolated_runtime = fresh_runtime()
    local metadata = spy.new(function() return "faction:antigone", "sector:second_contact" end)
    isolated_runtime.set_discovery_adapter(component_discovery.new({
        faction_id = "faction:argon",
        universe_id_bytes = 8,
        native_policy = { max_allocation_bytes = 8, max_work_units = 6 },
        count_stations = function() return 1 end,
        new_buffer = function() return {} end,
        fill_stations = function(_, buffer) buffer[0] = "station:20" return 1 end,
        to_component = function(_, value) return value end,
        to_component64 = function() return "20" end,
        get_component_data = function(...) return metadata(...) end,
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
    assert.spy(metadata).was.called()
end)

it("runtime emits only an allowlisted component diagnostic class", function()
    local trace_events = {}
    fixture.trace_config({
        enabled = true,
        attempt_id = "d051-facts-class",
        max_frame_events = 64,
        version_diagnostic_enabled = false,
    })
    _G.DebugError = function(message) trace_events[#trace_events + 1] = message end
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

end)

it("runtime discards a frame that crosses a restarted pipe connection", function()
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
end)

end)
