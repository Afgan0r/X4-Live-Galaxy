local root, mode, result_path = ...
package.path = root .. "/?.lua;" .. root .. "/extensions/?.lua;" .. package.path
local carrier = assert(require("extensions.live_galaxy.lua.live_galaxy_carrier").new({
    limits = { data_message_bytes = 4096, control_message_bytes = 512, max_records = 16,
        max_content_bytes = 512, max_canonical_bytes = 4096, max_batches = 16, max_work = 129,
        max_age_millis = 5000, pending_slots = 1, max_attempts = 2, max_retry_age_millis = 5000,
        availability_interval_millis = 5000 },
    source = { source_scope = "x4:faction:argon:ships", source_epoch_status = "unknown",
        source_boundary = "runtime_start" },
}))
local begin = { section_key = "ship_core", expected_records = 8, capture_start_millis = "10",
    capture_clock = "game_time_millis", quality = "unknown", availability = "available",
    coverage = "partial", consistency = "observed_count_fill_only", stable_identity = true,
    source_epoch_status = "unknown", source_boundary = "runtime_start" }
local finish = { capture_end_millis = "11", success = true, quality = "unknown",
    availability = "available", coverage = "partial", consistency = "observed_count_fill_only",
    stable_identity = true }
local getters, stages, collections = 0, 0, 0
local function called() getters = getters + 1 end
local source = {
    list_factions = function() called(); return { "argon", "teladi" } end,
    count_ships = function(_, faction) called(); assert(faction == "argon"); return 8 end,
    new_buffer = function(_, count) called(); assert(count == 8); return {} end,
    fill_ships = function(_, buffer, count, faction)
        called(); assert(count == 8 and faction == "argon")
        for i = 0, 7 do buffer[i] = "900719925474099" .. tostring(9 - i) end
        return 8
    end,
    read_core = function(_, identity)
        called(); return { identity = identity, owner = "argon", type = "destroyer_macro",
            class = "destroyer", location = "sector:" .. tostring(collections + 1) }
    end,
}
local function collector()
    return assert(require("live_galaxy.lua.live_galaxy_ship_collection").new({ faction_id = "argon",
        source_scope = "x4:faction:argon:ships", ship_api = source,
        ship_limits = { max_records = 16, max_allocation_bytes = 2048, max_work = 129,
            max_attempts = 2, max_age_millis = 5000, faction_pointer_bytes = 8 } },
        { begin_evidence = function() return begin end, finish_evidence = function() return finish end }))
end
local expected = mode == "heavy-ship-core" and 2 or 1
for collection = 1, expected do
    local observation, sampled = collector(), false
    local deadline = host_monotonic_millis() + 15000
    while true do
        local _, status = carrier:progress(1)
        local control = carrier:poll_control()
        if not sampled and status and status.capacity:match("^available:") then
            stages = stages + 1
            local result = observation:tick({ source_boundary = "runtime_start" }, carrier, status)
            assert(result.disposition == "collecting" or result.disposition == "sampled"
                or result.disposition == "clock_unavailable", result.disposition)
            sampled = result.disposition == "sampled"
        end
        if sampled and control == 5 then break end
        assert(host_monotonic_millis() < deadline, "heavy commit watchdog")
        host_sleep(1)
    end
    collections = collections + 1
    assert(getters == 20 * collections, "getter work must be exactly bounded per census")
    assert(stages > 0 and stages <= 129 * collections, "stage work bound")
end
if mode == "heavy-ship-restart" then
    local deadline = host_monotonic_millis() + 15000
    repeat
        local _, status = carrier:progress(1); carrier:poll_control(); host_sleep(1)
        if status and status.capacity:match("^available:") then break end
        assert(host_monotonic_millis() < deadline, "malformed reservation watchdog")
    until false
    begin.expected_records = 1
    assert(carrier:begin_section(begin) == 0)
    -- Wrong-owner core crosses the real ABI, then receiver authority must reject it.
    assert(carrier:push_record({ profile = "ship_core", source_scope = "x4:faction:argon:ships",
        identity = "9007199254740993", owner = "teladi", type = "destroyer_macro",
        class = "destroyer", location = "sector:4" }) == 0)
    assert(carrier:finish_section(finish) == 0)
    repeat
        carrier:progress(1)
        if carrier:poll_control() == 6 then break end
        host_sleep(1)
        assert(host_monotonic_millis() < deadline, "malformed rejection watchdog")
    until false
end
local file = assert(io.open(result_path, "wb"))
assert(file:write(string.format("return {actual_native=true,getter_calls=%d,stages=%d,collections=%d}", getters, stages, collections)))
assert(file:close()); assert(carrier:close() == 0)
