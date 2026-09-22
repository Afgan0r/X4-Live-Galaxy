local root, mode, result_path = ...
package.path = root .. "/?.lua;" .. root .. "/extensions/?.lua;" .. package.path
local module = require("extensions.live_galaxy.lua.live_galaxy_carrier")
local dll_path = assert(os.getenv("X4_SHIP_ABI_DLL"))
local function loadlib(_, name) return package.loadlib(dll_path, name) end
local initialize = assert(loadlib(module.dll_path, module.initializer))
local api = initialize()
assert(api.abi_version() == 2)
local limits = { data_message_bytes = 4096, control_message_bytes = 512, max_records = 129,
    max_content_bytes = 512, max_canonical_bytes = 4096, max_batches = 129, max_work = 129,
    max_age_millis = 5000, pending_slots = 1, max_attempts = 2, max_retry_age_millis = 5000,
    availability_interval_millis = 5000 }
local heavy = mode == "heavy_profile"
if heavy then
    limits.heavy_profile_version, limits.max_inner_records = 1, 17
    limits.max_attempts, limits.max_age_millis = 1, 30000
end
local boundary = (mode == "incompatible" or heavy) and "runtime_start" or mode
local source = { source_scope = "x4:faction:argon:ships", source_epoch_status = "boundary_uncertain",
    source_boundary = boundary }
local function copy(value)
    local result = {}; for key, field in pairs(value) do result[key] = field end; return result
end
local original_limits, original_source = copy(limits), copy(source)
-- Open goes through registered operations. Invalid configuration cannot start a worker.
assert(api.open(1, limits, source) == -10)
for key, value in pairs({ pending_slots = 2, max_attempts = 3,
    availability_interval_millis = 4999, max_canonical_bytes = 4095, max_records = 0,
    max_batches = 128, max_work = 0, max_retry_age_millis = 0 }) do
    local bad = copy(limits); bad[key] = value
    assert(api.open(2, bad, source) == -20, "invalid configured limit " .. key)
end
if heavy then
    for key in pairs(limits) do
        local bad = copy(limits); bad[key] = nil
        assert(api.open(2, bad, source) == -20, "missing heavy limit " .. key)
    end
    for key, value in pairs({ heavy_profile_version = 2, max_inner_records = 0 }) do
        local bad = copy(limits); bad[key] = value
        assert(api.open(2, bad, source) == -20, "invalid heavy limit " .. key)
    end
    local too_large = copy(limits); too_large.max_inner_records = limits.max_records + 1
    assert(api.open(2, too_large, source) == -20, "heavy inner bound exceeds records")
    local unknown = copy(limits); unknown.extra = 1
    assert(api.open(2, unknown, source) == -20, "heavy limit table remains exact-key")
end
local bad_source = copy(source); bad_source.source_scope = ""
assert(api.open(2, limits, bad_source) == -20)
local carrier, failure, failure_code = module.new({ limits = limits, source = source, loadlib = loadlib })
assert(carrier, tostring(failure) .. ":" .. tostring(failure_code) .. ":" .. tostring(source.source_boundary))
source.source_scope, source.source_boundary = "mutated", "mutated"
limits.max_records, limits.data_message_bytes = 1, 1
local function await(predicate)
    local deadline = host_monotonic_millis() + 8000
    repeat
        local progress, status = carrier:progress(1)
        local control = carrier:poll_control()
        if predicate(control, progress, status) then return status end
        host_sleep(1)
    until host_monotonic_millis() >= deadline
    error("ABI watchdog")
end
local begin = { section_key = "ship_core", expected_records = 1, capture_start_millis = "12",
    capture_clock = "game_time_millis", quality = "unknown", availability = "available",
    coverage = "partial", consistency = "observed_count_fill_only", stable_identity = true,
    source_epoch_status = "boundary_uncertain", source_boundary = boundary }
if mode == "incompatible" then
    await(function(control) return control == 10 end)
    assert(carrier:begin_section(begin) ~= 0)
    local _, status = carrier:progress(1)
    assert(status.producer_state == "incompatible", "incompatible profile must latch restart-required state")
    assert(carrier:close() == 0)
else
    local status = await(function(_, _, status) return status and status.capacity:match("^available:") end)
    assert(status.selection == "ship_core" and status.remaining_capacity == "129")
    for key in pairs(begin) do
        local bad = copy(begin); bad[key] = nil
        assert(carrier:begin_section(bad) == -20, "missing begin key " .. key)
    end
    local extra = copy(begin); extra.extra = true
    assert(carrier:begin_section(extra) == -20)
    assert(carrier:begin_section(begin) == 0)
    begin.capture_start_millis, begin.source_boundary = "999", "mutated"
    local record = { profile = "ship_core", source_scope = "x4:faction:argon:ships",
        identity = "9007199254740993", owner = "argon", type = "destroyer_macro",
        class = "destroyer", location = "sector:1", consistency = "consistent",
        consistency_reason = "none" }
    for key in pairs(record) do
        local bad = copy(record); bad[key] = nil
        assert(carrier:push_record(bad) == -20, "missing ship key " .. key)
    end
    for _, value in ipairs({ 9007199254740992, 0/0, math.huge, -math.huge,
        "9.007199254740993e15", "09007199254740993", "12345678901234567890123456789012345" }) do
        local bad = copy(record); bad.identity = value
        local code = carrier:push_record(bad)
        assert(code == -20, "precision-losing/noncanonical identity " .. tostring(value) .. ":" .. tostring(code))
    end
    for key, value in pairs({ profile = "ship_core_v2", source_scope = "", owner = "",
        type = "", class = "", location = "" }) do
        local bad = copy(record); bad[key] = value
        assert(carrier:push_record(bad) == -20, "invalid ship field " .. key)
    end
    for consistency, reason in pairs({ consistent = "owner_changed", possibly_stale = "none" }) do
        local bad = copy(record); bad.consistency, bad.consistency_reason = consistency, reason
        assert(carrier:push_record(bad) == -20, "invalid core consistency pair")
    end
    extra = copy(record); extra.extra = true
    assert(carrier:push_record(extra) == -20)
    assert(carrier:push_record(setmetatable(copy(record), {})) == -20)
    assert(carrier:push_record(record) == 0)
    for key in pairs(record) do record[key] = "mutated-after-success" end
    local finish = { capture_end_millis = "13", success = true, quality = "unknown",
        availability = "available", coverage = "partial", consistency = "observed_count_fill_only",
        stable_identity = true }
    for key in pairs(finish) do
        local bad = copy(finish); bad[key] = nil
        assert(carrier:finish_section(bad) == -20, "missing finish key " .. key)
    end
    extra = copy(finish); extra.extra = true
    assert(carrier:finish_section(extra) == -20)
    assert(carrier:finish_section(setmetatable(copy(finish), {})) == -20)
    assert(carrier:finish_section(finish) == 0)
    finish.capture_end_millis = "999"
    await(function(control) return control == 5 end)
    require("extensions.live_galaxy.tests.ship_abi_lifecycle")(
        module, api, carrier, loadlib, original_limits, original_source, boundary)
end
local file = assert(io.open(result_path, "wb")); assert(file:write("SHIP_ABI_PASS")); assert(file:close())
