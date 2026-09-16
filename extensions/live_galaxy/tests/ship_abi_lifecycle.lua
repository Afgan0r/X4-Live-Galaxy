return function(module, api, carrier, loadlib, limits, source, boundary)
    local begin = { section_key = "ship_core", expected_records = 1, capture_start_millis = "20",
        capture_clock = "game_time_millis", quality = "unknown", availability = "available",
        coverage = "partial", consistency = "observed_count_fill_only", stable_identity = true,
        source_epoch_status = "boundary_uncertain", source_boundary = boundary }
    local record = { profile = "ship_core", source_scope = source.source_scope,
        identity = "9007199254740995", owner = "argon", type = "destroyer_macro",
        class = "destroyer", location = "sector:2" }
    local finish = { capture_end_millis = "21", success = true, quality = "unknown",
        availability = "available", coverage = "partial", consistency = "observed_count_fill_only",
        stable_identity = true }
    -- Discard a reserved partial capture. No old record may reach the replacement stream.
    local reservation_deadline = host_monotonic_millis() + 8000
    repeat
        local _, status = carrier:progress(1); carrier:poll_control(); host_sleep(1)
        if status and status.capacity:match("^available:") then break end
        assert(host_monotonic_millis() < reservation_deadline, "new reservation watchdog")
    until false
    assert(carrier:begin_section(begin) == 0)
    assert(carrier:push_record(record) == 0)
    local old = carrier.token
    assert(carrier:reset("source_boundary_changed") == 0)
    local marker_path = assert(os.getenv("X4_SHIP_ABI_MARKER"))
    local marker = assert(io.open(marker_path, "wb"))
    assert(marker:write("reset-accepted")); assert(marker:close())
    repeat
        local released = io.open(marker_path .. ".released", "rb")
        if released then released:close(); break end
        host_sleep(1)
        assert(host_monotonic_millis() < reservation_deadline, "old peer release watchdog")
    until false
    local fresh, failure, code = module.new({ limits = limits, source = source, loadlib = loadlib })
    assert(fresh, tostring(failure) .. ":" .. tostring(code))
    assert(fresh.token ~= old)
    assert(api.begin_section(old, begin) == -16)
    assert(api.push_record(old, record) == -16)
    assert(api.finish_section(old, finish) == -16)
    assert(api.fail_section(old, "late_callback") == -16)
    assert(api.progress(old, 1) == -16, "stale progress must reject before replacement mutation")
    assert(select("#", api.progress(old, 1)) == 1, "stale progress must not return replacement state")
    assert(api.poll_control(old) == -16)
    assert(api.reset(old, "late_reset") == -16)
    assert(api.close(old) == -16) -- Stale close must preserve the replacement.
    assert(fresh:begin_section(begin) == -21, "fresh baseline required")
    local deadline = host_monotonic_millis() + 8000
    local status
    repeat
        local _, current = fresh:progress(1); fresh:poll_control(); status = current
        host_sleep(1)
        assert(host_monotonic_millis() < deadline, "fresh handshake watchdog")
    until status and status.capacity:match("^available:")
    local calls = 0
    local function called() calls = calls + 1 end
    local source_api = {
        list_factions = function() called(); return { "argon" } end,
        count_ships = function() called(); return 1 end,
        new_buffer = function() called(); return {} end,
        fill_ships = function(_, buffer) called(); buffer[0] = record.identity; return 1 end,
        read_core = function(_, identity)
            called(); assert(identity == record.identity)
            return { identity = identity, owner = record.owner, type = record.type,
                class = record.class, location = record.location }
        end,
    }
    local collector = assert(require("live_galaxy.lua.live_galaxy_ship_collection").new({
        faction_id = "argon", source_scope = source.source_scope, ship_api = source_api,
        ship_limits = { max_records = 129, max_allocation_bytes = 2048, max_work = 129,
            max_attempts = 2, max_age_millis = 5000, faction_pointer_bytes = 8 },
    }, { begin_evidence = function() return begin end, finish_evidence = function() return finish end }))
    local sampled = false
    repeat
        local _, current = fresh:progress(1)
        local control = fresh:poll_control()
        if not sampled and current and current.capacity:match("^available:") then
            local outcome = collector:tick({ source_boundary = boundary,
                source_epoch_status = "boundary_uncertain" }, fresh, current)
            sampled = outcome.disposition == "sampled"
        end
        if sampled and control == 5 then break end
        host_sleep(1)
        assert(host_monotonic_millis() < deadline, "collector/native lifecycle watchdog")
    until false
    assert(calls == 6, "one fresh census, allocation, fill and stable core validation")
    assert(fresh:close() == 0)
end
