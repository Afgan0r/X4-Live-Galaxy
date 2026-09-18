local extension_path = "extensions/?.lua"
if not package.path:find(extension_path, 1, true) then
    package.path = package.path .. ";" .. extension_path
end

local runtime = {}
local carrier_module = require("live_galaxy.lua.live_galaxy_carrier")
local observation_module = require("live_galaxy.lua.live_galaxy_observation")
local scheduler = require("live_galaxy.lua.live_galaxy_scheduler")

local active_carrier, active_observation
local initialized, callback_active = false, false
local last_diagnostic
local diagnostic_gap_count = 0
local pending_boundary
local profile_identity = "clock"
local boundaries = {
    telemetry_tick = { source_epoch_status = "unknown", source_boundary = "runtime_start" },
    telemetry_game_loaded = {
        source_epoch_status = "boundary_uncertain", source_boundary = "game_loaded",
    },
}
local safe_details = {
    already_initialized = true, initialized = true, sampled = true,
    producer_busy = true, reentry_suppressed = true, adapter_unavailable = true,
    loader_unavailable = true, loader_failure = true, initializer_failure = true,
    initializer_shape = true, operation_shape = true, operation_missing = true,
    operation_failure = true, abi_mismatch = true, open_failure = true,
    getter_loader_failure = true, getter_unavailable = true, callback_failure = true,
    invalid_result = true, exception = true, permanently_rejected = true,
    ambiguous_commit = true, retry_exhausted = true, disconnected = true,
    restart_required = true, clock_unavailable = true, source_failure = true,
    invalid_fact = true, fact_rejected = true, finish_rejected = true,
    reservation_failed = true, result_shape = true,
    collecting = true, core_changed = true, stale_parent = true, collection_overflow = true,
    allocation_limit = true, native_call_limit = true, callback_budget_exceeded = true,
    source_boundary_changed = true, selection_unavailable = true, admission_window_exhausted = true,
    identity_invalid = true, enumeration_incomplete = true, empty_unproven = true,
    cargo_unknown = true, failed = true,
    crew_unknown = true, invalid_capture_stage = true,
}

-- Rejection metadata contains owned labels/types/ordinals, never source values.
local function rejection_text(rejection)
    if type(rejection) ~= "table" then return "" end
    local text = ""
    for _, key in ipairs({ "stage", "condition", "field", "observed_type", "operation" }) do
        local value = rejection[key]
        if type(value) == "string" and #value <= 64 and value:match("^[a-z_]+$") then
            text = text .. " " .. key .. "=" .. value
        end
    end
    for _, key in ipairs({ "attempt", "ordinal" }) do
        local value = rejection[key]
        if type(value) == "number" and value >= 0 and value <= 9007199254740991 and value % 1 == 0 then
            text = text .. " " .. key .. "=" .. value
        end
    end
    local section = rejection.section
    if type(section) == "string" and #section <= 32 and (section == "ship_core"
        or section:match("^ship_cargo:g%d+$") or section:match("^ship_crew:g%d+$")
        or section:match("^ship_loadout:g%d+$")) then text = text .. " section=" .. section end
    local revision, run = rejection.revision, rejection.run
    if type(revision) == "string" and #revision <= 20 and revision:match("^[1-9]%d*$") then
        text = text .. " revision=" .. revision
    end
    if type(run) == "string" and #run <= 64 and run:match("^[%w_:%-]+$") then text = text .. " run=" .. run end
    return text
end
local function diagnostic(event, detail, metrics, rejection)
    local safe = safe_details[detail] and detail or "unknown"
    local context_text = rejection_text(rejection)
    local value = event .. ":" .. safe .. context_text
    if metrics then value = value .. ":" .. metrics.section .. ":" .. metrics.revision .. ":" .. metrics.incarnation end
    if value == last_diagnostic and (not metrics or safe ~= "sampled") then return true end
    if type(DebugError) == "function" then
        local text = "Live Galaxy Carrier B: event=" .. event .. " detail=" .. safe
        if profile_identity ~= "clock" then text = text .. " profile=" .. profile_identity end
        if diagnostic_gap_count > 0 then text = text .. " diagnostic_gap_count=" .. diagnostic_gap_count end
        text = text .. context_text
        if metrics then
            text = text .. " section=" .. metrics.section .. " revision=" .. metrics.revision
                .. " run=" .. metrics.incarnation .. " calls=" .. metrics.calls
                .. " allocation_bytes=" .. metrics.allocation_bytes .. " steps=" .. metrics.steps
                .. " duration_millis=" .. metrics.duration_millis .. " backlog=1"
            if metrics.max_callback_duration_millis ~= nil then
                text = text .. " max_callback_duration_millis=" .. metrics.max_callback_duration_millis
                    .. " max_callback_overrun_millis=" .. metrics.max_callback_overrun_millis
                    .. " source_value_bytes=" .. metrics.source_value_bytes
            end
        end
        if not pcall(DebugError, text) then
            diagnostic_gap_count = math.min(diagnostic_gap_count + 1, 65535)
            return false
        end
        diagnostic_gap_count = 0
    end
    last_diagnostic = value
    return true
end

function runtime.handle_tick(_, event_parameter)
    local event_boundary = boundaries[event_parameter]
    if event_boundary == nil then return false, "event_ignored" end
    if event_parameter == "telemetry_game_loaded" then pending_boundary = event_boundary end
    if callback_active then return false, "reentry_suppressed" end
    if active_carrier == nil or active_observation == nil then return false, "adapter_unavailable" end
    local boundary = pending_boundary or event_boundary
    callback_active = true
    local ok, result = pcall(scheduler.tick, boundary, active_carrier, active_observation)
    callback_active = false
    if not ok or type(result) ~= "table" then
        diagnostic("callback_failure", ok and "invalid_result" or "exception")
        return false, "callback_failure"
    end
    if result.disposition == "sampled" and boundary == pending_boundary then
        pending_boundary = nil
    end
    if result.disposition ~= "producer_busy" and result.disposition ~= "collecting" then
        if not diagnostic("transition", result.disposition, result.capture_metrics, result.rejection) then return false, "diagnostic_failure" end
    end
    return result.disposition == "sampled", result.disposition
end

function runtime.initialize(options)
    if initialized then return true, "already_initialized" end
    if options == nil then
        local config = require("live_galaxy.lua.live_galaxy_config")
        profile_identity = config.profile_sha256 or "clock"
        local reason
        options, reason = config.options()
        if type(options) ~= "table" then return false, reason or "invalid_limits" end
    end
    local carrier, carrier_error = carrier_module.new(options.carrier)
    if carrier == nil then
        diagnostic("carrier_unavailable", carrier_error)
        return false, carrier_error
    end
    local observation, observation_error = observation_module.new(options.observation)
    if observation == nil then
        carrier:close()
        diagnostic("observation_unavailable", observation_error)
        return false, observation_error
    end
    active_carrier, active_observation = carrier, observation
    initialized = true
    diagnostic("initialized", "initialized")
    return true, "initialized"
end

local function dispatch(event_name, event_parameter)
    if not initialized then
        local ok, reason = runtime.initialize()
        if not ok then return false, reason end
    end
    return runtime.handle_tick(event_name, event_parameter)
end

if type(RegisterEvent) ~= "function" then
    error("Live Galaxy Carrier B: RegisterEvent unavailable")
end
RegisterEvent("live_galaxy_observation", dispatch)

return runtime
