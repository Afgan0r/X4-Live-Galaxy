local extension_path = "extensions/?.lua"
if not package.path:find(extension_path, 1, true) then
    package.path = package.path .. ";" .. extension_path
end

local runtime = { scope_active = false }
local carrier_module = require("live_galaxy.lua.live_galaxy_carrier")
local observation_module = require("live_galaxy.lua.live_galaxy_observation")
local scheduler = require("live_galaxy.lua.live_galaxy_scheduler")

local active_carrier, active_observation
local initialized, callback_active = false, false
local update_callback
local registered, registration_attempted, stopped, dispatch_active = false, false, false, false
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
    reservation_failed = true, result_shape = true, progress_rejected = true,
    collecting = true, core_changed = true, stale_parent = true, collection_overflow = true,
    allocation_limit = true, native_call_limit = true, callback_budget_exceeded = true,
    source_boundary_changed = true, selection_unavailable = true, admission_window_exhausted = true,
    identity_invalid = true, enumeration_incomplete = true, empty_unproven = true,
    cargo_unknown = true, failed = true,
    crew_unknown = true, invalid_capture_stage = true,
    update_registered = true, update_removed = true, update_unavailable = true,
    update_registration_failed = true, update_removal_failed = true,
    callback_busy = true, runtime_stopped = true, teardown_failed = true,
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
    -- Successor failures expose only bounded numeric codes, never native text.
    local carrier_code = rejection.carrier_code
    if rejection.condition == "progress_rejected" and type(carrier_code) == "number"
        and carrier_code >= -2147483648 and carrier_code <= -1 and carrier_code % 1 == 0 then
        text = text .. " carrier_code=" .. carrier_code
    end
    local section = rejection.section
    if type(section) == "string" and #section <= 128 and (section == "ship_core"
        or section:match("^ship_core:[%w_%-]+$") or section:match("^ship_cargo:[%w_%-]*:?g%d+$")
        or section:match("^ship_crew:[%w_%-]*:?g%d+$")
        or section:match("^ship_loadout:[%w_%-]*:?g%d+$")) then text = text .. " section=" .. section end
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
                .. " duration_millis=" .. metrics.duration_millis
            if metrics.max_callback_duration_millis ~= nil then
                text = text .. " max_callback_duration_millis=" .. metrics.max_callback_duration_millis
                    .. " max_callback_overrun_millis=" .. metrics.max_callback_overrun_millis
                    .. " source_value_bytes=" .. metrics.source_value_bytes
            end
            if metrics.timing then
                local timing = metrics.timing
                text = text .. " timing_clock=" .. timing.clock .. " timing_scope=" .. timing.scope
                    .. " timing_sampling_overhead=" .. timing.sampling_overhead
                    .. " timing_origin_section=" .. metrics.timing_origin_section
                    .. " timing_origin_revision=" .. metrics.timing_origin_revision
                    .. " timing_origin_run=" .. metrics.timing_origin_incarnation
                    .. " timing_status_samples=" .. timing.status_samples
                    .. " timing_clock_regressions=" .. timing.clock_regressions
                    .. " timing_unsampled_callbacks=" .. timing.unsampled_callbacks
                    .. " timing_pending_operations=" .. tostring(timing.pending_operations)
                for _, category in ipairs({ "source", "allocation", "clock", "processing", "delivery", "mixed" }) do
                    local value = timing.intervals[category]
                    local prefix = " observed_" .. category
                    text = text .. prefix .. "_interval_count=" .. value.count
                        .. prefix .. "_total_millis=" .. value.total_millis .. prefix .. "_max_millis=" .. value.max_millis
                        .. prefix .. "_operations=" .. timing.operations[category]
                end
                for _, category in ipairs({ "callbacks", "between_callbacks" }) do
                    local value = timing[category]
                    local prefix = " selection_" .. category
                    text = text .. prefix .. "_count=" .. value.count .. prefix .. "_total_millis=" .. value.total_millis
                        .. prefix .. "_max_millis=" .. value.max_millis
                end
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

local function discard_active(reason)
    if not active_observation then return true end
    if active_observation.discard then
        return pcall(active_observation.discard, active_observation, active_carrier, reason)
    elseif active_observation.collector then
        return pcall(active_observation.collector.discard, active_observation.collector, active_carrier, reason)
    end
    return true
end

function runtime.handle_tick(_, event_parameter)
    if stopped or rawget(_G, "LiveGalaxyObservationRuntime") ~= runtime then
        return false, "runtime_stopped"
    end
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
        boundary.reentry_guard = false
        local discarded = discard_active("callback_failure")
        diagnostic("callback_failure", ok and "invalid_result" or "exception", nil,
            not discarded and { operation = "discard", condition = "teardown_failed" } or nil)
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
    if stopped then return false, "runtime_stopped" end
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

local function dispatch_tick()
    if not initialized then
        local ok, reason = runtime.initialize()
        if not ok then return false, reason end
    end
    return runtime.handle_tick(nil, "telemetry_tick")
end

-- This exact closure is retained for removal. Never edit the engine callback
-- list during dispatch; teardown/reconstruction refuse that transition.
update_callback = function()
    if stopped or not registered or rawget(_G, "LiveGalaxyObservationRuntime") ~= runtime then
        return false, "runtime_stopped"
    end
    if not runtime.scope_active then return false, "scope_inactive" end
    if dispatch_active then return false, "reentry_suppressed" end
    dispatch_active = true
    local ok, accepted, reason = pcall(dispatch_tick)
    dispatch_active = false
    if not ok then
        diagnostic("callback_failure", "exception")
        return false, "callback_failure"
    end
    return accepted, reason
end

local function register_update()
    if registered then return true end
    if type(SetScript) ~= "function" or type(RemoveScript) ~= "function" then
        diagnostic("callback_registration", "update_unavailable")
        return false, "update_unavailable"
    end
    registration_attempted = true -- A throwing hook may already have registered it.
    if not pcall(SetScript, "onUpdate", update_callback) then
        diagnostic("callback_registration", "update_registration_failed")
        return false, "update_registration_failed"
    end
    registered = true
    diagnostic("callback_registration", "update_registered")
    return true
end

function runtime.handle_event(_, event_parameter)
    if stopped then return false, "runtime_stopped" end
    if event_parameter == "telemetry_game_loaded" then
        pending_boundary = boundaries.telemetry_game_loaded
    elseif event_parameter ~= "telemetry_game_start" then
        return false, "event_ignored"
    end
    runtime.scope_active = true
    -- Boundary cues do not run collection or a second transport pump.
    if dispatch_active then return true, "boundary_pending" end
    return register_update()
end

function runtime.shutdown()
    if dispatch_active then
        diagnostic("callback_teardown", "callback_busy")
        return false, "callback_busy"
    end
    stopped = true -- Any retained callback is inert even when removal fails.
    runtime.scope_active = false
    if registration_attempted then
        if not pcall(RemoveScript, "onUpdate", update_callback) then
            diagnostic("callback_teardown", "update_removal_failed")
            return false, "update_removal_failed"
        end
        registered, registration_attempted = false, false
        diagnostic("callback_teardown", "update_removed")
    end
    if active_carrier then
        local discarded = discard_active("source_boundary_changed")
        local ok, code = pcall(active_carrier.close, active_carrier)
        if not discarded or not ok or code ~= 0 then
            diagnostic("callback_teardown", "teardown_failed")
            return false, "teardown_failed"
        end
    end
    active_carrier, active_observation, initialized = nil, nil, false
    return true
end

local previous = rawget(_G, "LiveGalaxyObservationRuntime")
if previous then
    local live_scope = previous.scope_active
    local ok, reason = previous.shutdown()
    if not ok then error("Live Galaxy Carrier B: " .. reason) end
    -- Same-state reconstruction may retain an already live scope; a fresh VM
    -- still waits for an MD lifecycle event. This does not prove VM continuity.
    runtime.scope_active = live_scope
    pending_boundary = { source_epoch_status = "boundary_uncertain", source_boundary = "lua_reload" }
end
if type(RegisterEvent) ~= "function" then error("Live Galaxy Carrier B: RegisterEvent unavailable") end
-- Reuse the one event closure across module reconstruction. It resolves only
-- the current owner, without retaining an old runtime or its native resource.
runtime.event_callback = previous and previous.event_callback or function(name, parameter)
    local current = rawget(_G, "LiveGalaxyObservationRuntime")
    if current then return current.handle_event(name, parameter) end
end
_G.LiveGalaxyObservationRuntime = runtime
if not previous then RegisterEvent("live_galaxy_observation", runtime.event_callback) end
register_update()

return runtime
