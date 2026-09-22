local root, mode, result_path = ...
package.path = root .. "/?.lua;" .. root .. "/extensions/?.lua;" .. package.path

local config = assert(require("live_galaxy.lua.live_galaxy_config").options())
local faction_values = mode == "heavy-ship-full-set-restart"
    and { "argon", "player", "scaleplate", "custom_mod", "khaak", "xenon" }
    or { "custom_mod", "khaak", "player", "xenon", "scaleplate", "argon" }
local ships = {
    argon = { "9007199254740101", "9007199254740102" },
    scaleplate = { "9007199254740201" },
    xenon = {},
    khaak = { "9007199254740301" },
}
local owner = {}
for faction, values in pairs(ships) do
    for _, identity in ipairs(values) do owner[identity] = faction end
end
local reads = {}
local api = {
    count_factions = function() return #faction_values end,
    new_faction_buffer = function() return {} end,
    fill_factions = function(_, buffer, count)
        assert(count == #faction_values)
        for index, value in ipairs(faction_values) do buffer[index - 1] = value end
        return count
    end,
    faction_string = function(_, value) return value end,
    count_ships = function(_, faction) return #assert(ships[faction]) end,
    new_buffer = function() return {} end,
    fill_ships = function(_, buffer, count, faction)
        local values = assert(ships[faction]); assert(count == #values)
        for index, value in ipairs(values) do buffer[index - 1] = value end
        return count
    end,
    read_core = function(_, identity)
        reads[identity] = (reads[identity] or 0) + 1
        local location = identity == "9007199254740101" and reads[identity] % 2 == 0
            and "sector:changed" or "sector:stable"
        return { identity = identity, owner = owner[identity], type = "ship_macro",
            class = "ship", location = location }
    end,
    cargo_wares = function() return {} end,
    cargo_storage_count = function() return 0 end,
    cargo_storage_size = function() return 24 end,
    cargo_storage_allocate = function() return {} end,
    cargo_storage_fill = function() return {} end,
    crew_capacity = function() return 0 end,
    crew_count = function() return 0 end,
    crew_size = function() return 40 end,
    crew_allocate = function() return {} end,
    crew_fill = function() return {} end,
    crew_tier_size = function() return 16 end,
    crew_tier_allocate = function() return {} end,
    crew_tier_fill = function() return {} end,
    physical_count = function() return 0 end,
    virtual_count = function() return 0 end,
    software_count = function() return 0 end,
    software_size = function() return 24 end,
    software_allocate = function() return {} end,
    software_fill = function() return {} end,
    missiles_count = function() return 0 end,
    missiles_size = function() return 24 end,
    missiles_allocate = function() return {} end,
    missiles_fill = function() return {} end,
    units_count = function() return 0 end,
    units_size = function() return 24 end,
    units_allocate = function() return {} end,
    units_fill = function() return {} end,
}

local roster = assert(require("live_galaxy.lua.live_galaxy_factions").capture(
    api, config.observation.faction_inventory, 1,
    { max_factions = 64, max_allocation_bytes = 4096, pointer_bytes = 8 }))
assert(roster.by_id.player.disposition == "excluded")
assert(roster.by_id.custom_mod.disposition == "unknown" and roster.unknown_blocker)
for _, faction in ipairs({ "argon", "scaleplate", "xenon", "khaak" }) do
    assert(roster.by_id[faction].disposition == "included")
end

config.observation.ship_api = api
local carrier = assert(require("live_galaxy.lua.live_galaxy_carrier").new(config.carrier))
local native_begin = carrier.begin_section
function carrier:begin_section(value)
    local code = native_begin(self, value)
    if code ~= 0 then
        local fields = {}
        for key, item in pairs(value) do
            fields[#fields + 1] = key .. "=" .. type(item) .. ":" .. tostring(item)
        end
        table.sort(fields)
        io.stderr:write("FULL_SET_BEGIN code=" .. tostring(code) .. " " .. table.concat(fields, ",") .. "\n")
    end
    return code
end
local native_push = carrier.push_record
function carrier:push_record(value)
    local code = native_push(self, value)
    if code ~= 0 then
        local fields = {}
        for key, item in pairs(value) do fields[#fields + 1] = key .. "=" .. type(item) .. ":" .. tostring(item) end
        table.sort(fields)
        io.stderr:write("FULL_SET_PUSH code=" .. tostring(code) .. " " .. table.concat(fields, ",") .. "\n")
    end
    return code
end
local function clock() return math.floor(host_monotonic_millis()) end
local adapter = {
    clock_getter = clock,
    begin_evidence = function() return { capture_start_millis = tostring(clock()),
        capture_clock = "game_time_millis", quality = "unknown", availability = "available" } end,
    finish_evidence = function() return { capture_end_millis = tostring(clock()),
        success = true, quality = "unknown", availability = "available" } end,
}
assert(require("live_galaxy.lua.live_galaxy_ship_selection").attach(adapter, config.observation))
local context = { source_boundary = "runtime_start", source_epoch_status = "unknown" }
local target = mode == "heavy-ship-full-set-restart" and 26 or 10
local committed, deadline = 0, host_monotonic_millis() + 30000
while committed < target do
    local _, status = carrier:progress(1)
    local control = carrier:poll_control()
    adapter:feedback(context, carrier, status, control)
    if control == 5 then committed = committed + 1 end
    if status and status.capacity:match("^available:") then
        local result = adapter:advance(context, carrier, status)
        assert(result.disposition == "collecting" or result.disposition == "sampled"
            or result.disposition == "producer_busy" or result.disposition == "clock_unavailable",
            result.disposition .. ":" .. tostring(result.rejection and result.rejection.native_code))
    end
    assert(host_monotonic_millis() < deadline, "full-set watchdog")
    host_sleep(1)
end
local file = assert(io.open(result_path, "wb"))
assert(file:write(string.format("return {committed=%d,unknown_blocker=true}", committed)))
assert(file:close())
assert(carrier:close() == 0)
