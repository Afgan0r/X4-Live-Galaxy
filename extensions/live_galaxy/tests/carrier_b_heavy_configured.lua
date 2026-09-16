local root, mode, result_path = ...
package.path = root .. "/?.lua;" .. root .. "/extensions/?.lua;" .. package.path
local options = assert(require("live_galaxy.lua.live_galaxy_config").options())
local carrier = assert(require("live_galaxy.lua.live_galaxy_carrier").new(options.carrier))
local revision, calls = 1, 0
local function called() calls = calls + 1 end
local function stop(family)
    if mode == "fail_" .. family then error("fixture_source_stop", 0) end
end
local source = {
    list_factions = function() called(); return { "argon" } end,
    count_ships = function() called(); return 1 end,
    new_buffer = function() return {} end,
    fill_ships = function(_, buffer) called(); buffer[0] = "9007199254740993"; return 1 end,
    read_core = function(_, identity) called(); stop("core"); return { identity = identity, owner = "argon",
        type = "destroyer_macro", class = "destroyer", location = "sector:1" } end,
    cargo_wares = function() called(); return { ore = revision } end,
    cargo_storage_count = function() called(); return 1 end,
    cargo_storage_size = function() return 24 end,
    cargo_storage_allocate = function() return {} end,
    cargo_storage_fill = function() called(); stop("cargo"); return { { transport = "solid",
        capacity_cubic_metres = 1200, occupied_cubic_metres = 110 } } end,
    crew_capacity = function() called(); return 12 + revision end,
    crew_count = function() called(); return 1 end,
    crew_size = function() return 40 end,
    crew_allocate = function() return {} end,
    crew_fill = function() called(); return { { id = "service", amount_people = 7,
        reported_numtiers = 1, canhire = true, tiers = {} } } end,
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
    software_count = function() called(); return 1 end,
    software_size = function() return 16 end,
    software_allocate = function() return {} end,
    software_fill = function() called(); stop("loadout"); return { { maximum = "software_max", current = "software_current" } } end,
    missiles_count = function() called(); return 1 end,
    missiles_size = function() return 24 end,
    missiles_allocate = function() return {} end,
    missiles_fill = function() called(); return { { ware = "missile_ware", macro_name = "missile_macro", amount_raw = -revision } } end,
    units_count = function() called(); return 1 end,
    units_size = function() return 24 end,
    units_allocate = function() return {} end,
    units_fill = function() called(); return { { macro_name = "unit_macro", category = "unfiltered_raw", amount_items = 2 } } end,
}
options.observation.ship_api = source
options.observation.getter = function() error("clock sample must not run in ship mode") end
options.observation.clock_getter = function() return host_monotonic_millis() / 1000 end
local observation = assert(require("live_galaxy.lua.live_galaxy_observation").new(options.observation))
local scheduler = require("live_galaxy.lua.live_galaxy_scheduler")
local expected = mode == "first" and 8 or 4
local committed, last, revisions = 0, "none", {}
local deadline = host_monotonic_millis() + options.observation.heavy_limits.admission_window_millis
local feedback = observation.feedback
function observation:feedback(context, active_carrier, status, control)
    feedback(self, context, active_carrier, status, control)
    revision = assert(tonumber(status.collection_revision))
    if control == 5 then committed = committed + 1; revisions[#revisions + 1] = revision end
end
while committed < expected do
    local result = scheduler.tick({ source_boundary = "runtime_start" }, carrier, observation)
    last = result.disposition
    assert(host_monotonic_millis() < deadline, "configured heavy watchdog: " .. last)
    assert(last ~= "source_failure" and last ~= "core_changed" and last ~= "native_call_limit"
        and last ~= "allocation_limit" and last ~= "callback_budget_exceeded", last)
    host_sleep(1)
end
local file = assert(io.open(result_path, "wb"))
assert(file:write("committed=" .. committed .. "\ncalls=" .. calls .. "\nrevisions=" .. table.concat(revisions, ",") .. "\n"))
assert(file:close()); assert(carrier:close() == 0)
