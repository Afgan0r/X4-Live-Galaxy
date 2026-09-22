local root, mode, result_path, marker = ...
local interleave = marker == "throughput-interleave"
local performance = marker == "performance"
local throughput = marker == "throughput" or interleave or performance
local population = throughput and 129 or 1
package.path = root .. "/?.lua;" .. root .. "/extensions/?.lua;" .. package.path
local options = assert(require("live_galaxy.lua.live_galaxy_config").options())
local carrier = assert(require("live_galaxy.lua.live_galaxy_carrier").new(options.carrier))
local timings, captures = {}, {}
if throughput then
    for _, name in ipairs({ "begin_section", "push_record", "finish_section", "progress" }) do
        local method = carrier[name]
        carrier[name] = function(self, ...)
            local before = host_monotonic_millis()
            local first, second = method(self, ...)
            timings[name] = timings[name] or {}
            timings[name][#timings[name] + 1] = host_monotonic_millis() - before
            return first, second
        end
    end
end
local revision, calls = 1, 0
local discovered_factions = interleave and { "argon", "teladi" } or { "argon" }
local active_faction = "argon"
local function called() calls = calls + 1 end
local function stop(family)
    if mode == "fail_" .. family then error("fixture_source_stop", 0) end
end
local source = {
    list_factions = function() called(); return discovered_factions end,
    count_factions = function() called(); return #discovered_factions end,
    new_faction_buffer = function() return {} end,
    fill_factions = function(_, buffer)
        called(); for index, faction in ipairs(discovered_factions) do buffer[index - 1] = faction end
        return #discovered_factions
    end,
    faction_string = function(_, value) return value end,
    count_ships = function(_, faction) called(); active_faction = faction; return population end,
    new_buffer = function() return {} end,
    fill_ships = function(_, buffer)
        called(); for i = 0, population - 1 do buffer[i] = "900719925474" .. string.format("%04d", i + 993) end
        return population
    end,
    read_core = function(_, identity) called(); stop("core"); return { identity = identity, owner = active_faction,
        type = "destroyer_macro", class = "destroyer", location = "sector:1" } end,
    cargo_wares = function()
        called(); if not throughput then return { ore = revision } end
        local rows = {}; for i = 1, 80 do rows["ware_" .. string.rep("x", 100) .. string.format("%03d", i)] = revision end
        return rows
    end,
    cargo_storage_count = function() called(); return 1 end,
    cargo_storage_size = function() return 24 end,
    cargo_storage_allocate = function() return {} end,
    cargo_storage_fill = function() called(); stop("cargo"); return { { transport = "solid",
        capacity_cubic_metres = 1200, occupied_cubic_metres = 110 } } end,
    crew_capacity = function() called(); return 12 + revision end,
    crew_count = function() called(); return throughput and 80 or 1 end,
    crew_size = function() return 40 end,
    crew_allocate = function() return {} end,
    crew_fill = function()
        called(); local rows = {}; for i = 1, throughput and 80 or 1 do
            rows[i] = { id = "service" .. (throughput and string.format("%03d", i) or ""), amount_people = 7,
                reported_numtiers = 1, canhire = true, tiers = {} }
        end; return rows
    end,
    crew_tier_size = function() return 16 end,
    crew_tier_allocate = function() return {} end,
    crew_tier_fill = function() called(); stop("crew"); return { { name = "raw",
        skill_lower_threshold = -25, amount_people = 7 } } end,
    physical_count = function() called(); return 1 end,
    physical_component = function() called(); return "0" end,
    physical_macro = function(_, _, kind) called(); return kind .. "_installed_macro" end,
    physical_group = function() called(); return { path = "..", group = "" } end,
    virtual_count = function(_, _, kind) called(); return kind == "thruster" and 1 or 0 end,
    virtual_macro = function() called(); return "thruster_current_macro" end,
    software_count = function() called(); return throughput and 80 or 1 end,
    software_size = function() return 16 end,
    software_allocate = function() return {} end,
    software_fill = function()
        called(); stop("loadout"); local rows = {}; for i = 1, throughput and 80 or 1 do
            rows[i] = { maximum = "software_max" .. (throughput and string.rep("x", 100) .. i or ""), current = "software_current" }
        end; return rows
    end,
    missiles_count = function() called(); return 1 end,
    missiles_size = function() return 24 end,
    missiles_allocate = function() return {} end,
    missiles_fill = function() called(); return { { ware = "missile_ware", macro_name = "missile_macro", amount_raw = -revision } } end,
    units_count = function() called(); return 1 end,
    units_size = function() return 24 end,
    units_allocate = function() return {} end,
    units_fill = function() called(); return { { macro_name = "unit_macro", category = "unfiltered_raw", amount_items = 2 } } end,
}
if throughput then
    for name, method in pairs(source) do
        source[name] = function(...)
            local before = host_monotonic_millis()
            local first, second, third = method(...)
            local key = "source_" .. name
            timings[key] = timings[key] or {}
            timings[key][#timings[key] + 1] = host_monotonic_millis() - before
            return first, second, third
        end
    end
end
options.observation.ship_api = source
options.observation.getter = function() error("clock sample must not run in ship mode") end
options.observation.clock_getter = function() return host_monotonic_millis() / 1000 end
local observation = assert(require("live_galaxy.lua.live_galaxy_observation").new(options.observation))
local scheduler = require("live_galaxy.lua.live_galaxy_scheduler")
local expected = interleave and 20 or (mode == "first" and 9 or 5)
local committed, last, revisions = 0, "none", {}
local pending_age_wait, synthetic_wait_count, synthetic_wait_total = nil, 0, 0
local started, callback_durations, commit_durations, busy, produced, production_times = host_monotonic_millis(), {}, {}, 0, 0, {}
local deadline = host_monotonic_millis() + options.observation.heavy_limits.admission_window_millis
local feedback = observation.feedback
function observation:feedback(context, active_carrier, status, control)
    feedback(self, context, active_carrier, status, control)
    revision = assert(tonumber(status.collection_revision))
    if control == 5 then
        committed = committed + 1; revisions[#revisions + 1] = revision
        commit_durations[#commit_durations + 1] = host_monotonic_millis() - (production_times[committed] or started)
        if interleave and committed == 2 then pending_age_wait = 31000 end
    end
end
while committed < expected do
    local before = host_monotonic_millis()
    local result = scheduler.tick({ source_boundary = "runtime_start" }, carrier, observation)
    callback_durations[#callback_durations + 1] = host_monotonic_millis() - before
    if result.disposition == "producer_busy" then busy = busy + 1 end
    if result.disposition == "sampled" then
        produced = produced + 1; production_times[produced] = host_monotonic_millis()
        local m = result.capture_metrics
        if m then captures[#captures + 1] = "capture section=" .. m.section .. " revision=" .. m.revision
            .. " duration_millis=" .. m.duration_millis .. " calls=" .. m.calls
            .. " allocation_bytes=" .. m.allocation_bytes .. " steps=" .. m.steps
            .. " max_callback_duration_millis=" .. m.max_callback_duration_millis
            .. " max_callback_overrun_millis=" .. m.max_callback_overrun_millis end
    end
    last = result.disposition
    assert(host_monotonic_millis() < deadline, "configured heavy watchdog: " .. last)
    assert(last ~= "source_failure" and last ~= "core_changed" and last ~= "native_call_limit"
        and last ~= "allocation_limit" and last ~= "callback_budget_exceeded",
        last .. " revision=" .. revision .. " committed=" .. committed .. " calls=" .. calls)
    if pending_age_wait then
        local delay = pending_age_wait; pending_age_wait = nil
        host_sleep(delay)
        synthetic_wait_count = synthetic_wait_count + 1; synthetic_wait_total = synthetic_wait_total + delay
    end
    host_sleep(1)
end
local file = assert(io.open(result_path, "wb"))
assert(file:write("committed=" .. committed .. "\ncalls=" .. calls .. "\nrevisions=" .. table.concat(revisions, ",") .. "\n"))
if throughput then
    assert(file:write("synthetic_wait_count=" .. synthetic_wait_count .. "\nsynthetic_wait_total_millis=" .. synthetic_wait_total .. "\n"))
    local function percentile(rows, fraction) table.sort(rows); return rows[math.max(1, math.ceil(#rows * fraction))] end
    assert(file:write("synthetic_core_records=" .. population .. "\nnested_records=80\nelapsed_millis=" .. (host_monotonic_millis() - started)
        .. "\ncallback_samples=" .. #callback_durations .. "\ncallback_p95_millis=" .. percentile(callback_durations, .95)
        .. "\ncallback_max_millis=" .. percentile(callback_durations, 1)
        .. "\nseal_to_commit_samples=" .. #commit_durations .. "\nseal_to_commit_p95_millis=" .. percentile(commit_durations, .95)
        .. "\nseal_to_commit_max_millis=" .. percentile(commit_durations, 1)
        .. "\nproducer_busy_pulses=" .. busy .. "\nfinal_backlog=" .. (produced - committed) .. "\n"))
    for name, rows in pairs(timings) do
        assert(file:write("stage=" .. name .. " samples=" .. #rows .. " p95_millis=" .. percentile(rows, .95)
            .. " max_millis=" .. percentile(rows, 1) .. "\n"))
    end
    for _, row in ipairs(captures) do assert(file:write(row .. "\n")) end
end
assert(file:close()); assert(carrier:close() == 0)
