local root, mode, result_path = ...
package.path = root .. "/?.lua;" .. root .. "/extensions/?.lua;" .. package.path
local carrier = assert(require("extensions.live_galaxy.lua.live_galaxy_carrier").new({
    limits = { data_message_bytes = 4096, control_message_bytes = 512, max_records = 16,
        max_content_bytes = 2048, max_canonical_bytes = 4096, max_batches = 16, max_work = 129,
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
if mode == "heavy-ship-detail" then
    local revision = 2
    local function invalid_abi()
        local function record()
            return { profile = "ship_cargo", source_scope = "x4:faction:argon:ships",
                identity = "9007199254740993", owner = "argon", core_revision = "1",
                member_revision = "1", policy_version = 2, capture_start_millis = "20",
                capture_end_millis = "30", source_evidence = "x4-9.00-steam-23660954-ship-detail-source-v1",
                wares_outcome = "value", storage_outcome = "empty",
                wares = { { ware = "ore", amount_items = 1 } }, storage = {} }
        end
        for _, mutate in ipairs({
            function(v) v.wares[1].amount_items = "1" end,
            function(v) v.wares[1].amount_items = 9007199254740992 end,
            function(v) v.wares[1].amount_items = -1 end,
            function(v) v.extra = true end,
            function(v) setmetatable(v, {}) end,
            function(v) setmetatable(v.wares[1], {}) end,
            function(v) v.wares[3], v.wares[1] = v.wares[1], nil end,
            function(v) for i = 1, 17 do v.wares[i] = { ware = "ore" .. i, amount_items = i } end end,
        }) do
            local value = record(); mutate(value)
            assert(carrier:push_record(value) == -20, "strict cargo ABI must reject malformed nested input")
        end
    end
    source.cargo_wares = function(_, id) called(); return { energycells = 7, ore = 11 } end
    source.crew_capacity = function() called(); return 12 + revision end
    source.crew_count = function() called(); return 2 end
    source.crew_size = function() return 40 end
    source.crew_allocate = function() return {} end
    source.crew_fill = function()
        called(); return {
            { id = "service", amount_people = 7, reported_numtiers = 1, canhire = true, tiers = {} },
            { id = "passenger", amount_people = 2, reported_numtiers = 1, canhire = false, tiers = {} } }
    end
    source.crew_tier_size = function() return 16 end
    source.crew_tier_allocate = function() return {} end
    source.crew_tier_fill = function(_, id, role)
        called(); return { { name = "raw", skill_lower_threshold = -25, amount_people = role == "service" and 7 or 2 } }
    end
    source.physical_count = function() called(); return 1 end
    source.physical_component = function() called(); return "0" end
    source.physical_macro = function(_, id, kind) called(); return kind .. "_installed_macro" end
    source.physical_group = function() called(); return { path = "..", group = "" } end
    source.virtual_count = function(_, id, kind) called(); return kind == "thruster" and 1 or 0 end
    source.virtual_macro = function() called(); return "thruster_current_macro" end
    source.software_count = function() called(); return 1 end
    source.software_size = function() return 16 end
    source.software_allocate = function() return {} end
    source.software_fill = function() called(); return { { maximum = "software_max", current = "software_current" } } end
    source.missiles_count = function() called(); return 1 end
    source.missiles_size = function() return 24 end
    source.missiles_allocate = function() return {} end
    source.missiles_fill = function() called(); return { { ware = "missile_ware", macro_name = "missile_macro", amount_raw = -revision } } end
    source.units_count = function() called(); return 2 end
    source.units_size = function() return 24 end
    source.units_allocate = function() return {} end
    source.units_fill = function() called(); return {
        { macro_name = "drone_macro", category = "defence", amount_items = 7 },
        { macro_name = "unit_macro", category = "unfiltered_raw", amount_items = 2 } } end
    source.cargo_storage_count = function() called(); return 1 end
    source.cargo_storage_size = function() return 24 end
    source.cargo_storage_allocate = function() return {} end
    source.cargo_storage_fill = function()
        called(); return { { transport = "solid", capacity_cubic_metres = 1200,
            occupied_cubic_metres = 110 } }
    end
    local function copy(value)
        if type(value) ~= "table" then return value end
        local result = {}; for key, item in pairs(value) do result[key] = copy(item) end
        return result
    end
    local function invalid_detail(observation, family)
        local mutations = family == "ship_crew" and {
            function(v) v.capacity_people = "12" end,
            function(v) v.roles[1].tiers[1].skill_lower_threshold = 2147483648 end,
            function(v) v.roles[1].canhire = 1 end,
            function(v) setmetatable(v.roles[1], {}) end,
        } or {
            function(v) v.physical[1].component = 0 end,
            function(v) v.physical[1].slot = 0 end,
            function(v) v.missiles[1].amount_raw = "-3" end,
            function(v) v.onlydrones = true end,
            function(v) setmetatable(v.units[1], {}) end,
        }
        for _, mutate in ipairs(mutations) do
            local value = copy(observation.pending)
            value.profile, value.source_scope = family, "x4:faction:argon:ships"
            value.identity, value.owner = observation.group.members[1], "argon"
            value.core_revision, value.member_revision, value.policy_version = "1", "1", 2
            value.capture_start_millis, value.capture_end_millis = tostring(20 + revision), tostring(30 + revision)
            value.source_evidence = "x4-9.00-steam-23660954-ship-detail-source-v1"
            mutate(value)
            assert(carrier:push_record(value) == -20, "strict detail ABI must reject malformed nested input")
        end
    end
    for cycle = 1, 2 do
        for group_index = 0, 1 do
            local members = {}
            for i = 1, 4 do members[i] = "900719925474099" .. tostring(group_index * 4 + i + 1) end
            local detail_begin, detail_finish = {}, {}
            for k, v in pairs(begin) do detail_begin[k] = v end
            for k, v in pairs(finish) do detail_finish[k] = v end
            detail_begin.capture_start_millis = tostring(20 + revision)
            detail_finish.capture_end_millis = tostring(30 + revision)
            local observation = assert(require("live_galaxy.lua.live_galaxy_ship_details").new({
                ship_api = source, max_inner = 16, max_allocation_bytes = 2048,
                source_scope = "x4:faction:argon:ships", group = { key = "ship_cargo:g" .. group_index,
                    members = members, owner = "argon", core_revision = "1" } },
                { begin_evidence = function() return detail_begin end,
                    finish_evidence = function() return detail_finish end }))
            local sampled = false
            local deadline = host_monotonic_millis() + 15000
            repeat
                local _, status = carrier:progress(1)
                local control = carrier:poll_control()
                if not sampled and status and status.capacity:match("^available:")
                    and status.selection == "ship_cargo:g" .. group_index then
                    if observation.stage == "reserve" then invalid_abi() end
                    local result = observation:tick({ source_boundary = "runtime_start" }, carrier, status)
                    assert(result.disposition == "collecting" or result.disposition == "sampled"
                        or result.disposition == "producer_busy", result.disposition)
                    sampled = result.disposition == "sampled"
                end
                if sampled and control == 5 then break end
                assert(host_monotonic_millis() < deadline, "cargo commit watchdog")
                host_sleep(1)
            until false
            revision = revision + 1
        end
    end
    for _, family in ipairs({ "ship_crew", "ship_loadout" }) do
    for cycle = 1, 2 do
        for group_index = 0, 1 do
            local members = {}
            for i = 1, 4 do members[i] = "900719925474099" .. tostring(group_index * 4 + i + 1) end
            local detail_begin, detail_finish = {}, {}
            for k, v in pairs(begin) do detail_begin[k] = v end
            for k, v in pairs(finish) do detail_finish[k] = v end
            detail_begin.capture_start_millis, detail_finish.capture_end_millis = tostring(20 + revision), tostring(30 + revision)
            local key = family .. ":g" .. group_index
            local observation = assert(require("live_galaxy.lua.live_galaxy_ship_details").new({
                ship_api = source, max_inner = 16, max_allocation_bytes = 2048,
                source_scope = "x4:faction:argon:ships", group = { key = key,
                    members = members, owner = "argon", core_revision = "1" } },
                { begin_evidence = function() return detail_begin end, finish_evidence = function() return detail_finish end }))
            local sampled, deadline = false, host_monotonic_millis() + 15000
            repeat
                local _, status = carrier:progress(1)
                local control = carrier:poll_control()
                if not sampled and status and status.capacity:match("^available:") and status.selection == key then
                    if observation.stage == "record" and observation.index == 1 then invalid_detail(observation, family) end
                    local result = observation:tick({}, carrier, status)
                    assert(result.disposition == "collecting" or result.disposition == "sampled" or result.disposition == "producer_busy", result.disposition)
                    sampled = result.disposition == "sampled"
                end
                if sampled and control == 5 then break end
                assert(host_monotonic_millis() < deadline, "crew commit watchdog")
                host_sleep(1)
            until false
            revision = revision + 1
        end
    end
    end
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
