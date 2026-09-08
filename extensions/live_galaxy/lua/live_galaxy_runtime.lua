local runtime = {}
local prefix = "live_galaxy/lua/"
local carrier_module = require(prefix .. "live_galaxy_carrier")
local observation_module = require(prefix .. "live_galaxy_observation")
local scheduler = require(prefix .. "live_galaxy_scheduler")

local prior_generation = rawget(_G, "__live_galaxy_runtime_generation")
local reload_boundary_pending = type(prior_generation) == "number"
rawset(_G, "__live_galaxy_runtime_generation", (prior_generation or 0) + 1)

local active_carrier, active_observation
local initialized, callback_active = false, false
local last_diagnostic
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
}

local function diagnostic(event, detail)
    local safe = safe_details[detail] and detail or "unknown"
    local value = event .. ":" .. safe
    if value == last_diagnostic then return end
    last_diagnostic = value
    if type(DebugError) == "function" then
        DebugError("Live Galaxy Carrier B: event=" .. event .. " detail=" .. safe)
    end
end

function runtime.handle_tick(_, event_parameter)
    if callback_active then return false, "reentry_suppressed" end
    if active_carrier == nil or active_observation == nil then return false, "adapter_unavailable" end
    local boundaries = {
        telemetry_tick = { source_epoch_status = "unknown", source_boundary = "runtime_start" },
        telemetry_game_loaded = {
            source_epoch_status = "boundary_uncertain", source_boundary = "game_loaded",
        },
        telemetry_lua_reload = {
            source_epoch_status = "boundary_uncertain", source_boundary = "lua_reload",
        },
    }
    local boundary = boundaries[event_parameter]
    if boundary == nil then return false, "event_ignored" end
    if reload_boundary_pending and event_parameter == "telemetry_tick" then
        boundary = boundaries.telemetry_lua_reload
    end
    callback_active = true
    local ok, result = pcall(scheduler.tick, boundary, active_carrier, active_observation)
    callback_active = false
    if not ok or type(result) ~= "table" then
        diagnostic("callback_failure", ok and "invalid_result" or "exception")
        return false, "callback_failure"
    end
    if result.disposition ~= "producer_busy" then
        diagnostic("transition", result.disposition)
    end
    if result.disposition == "sampled" then reload_boundary_pending = false end
    return result.disposition == "sampled", result.disposition
end

function runtime.initialize(options)
    if initialized then return true, "already_initialized" end
    options = options or {}
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
    if type(RegisterEvent) == "function" then
        RegisterEvent("live_galaxy_observation", runtime.handle_tick)
    end
    diagnostic("initialized", "abi=2")
    return true, "initialized"
end

local function init()
    runtime.initialize()
end

if type(Register_OnLoad_Init) == "function" then
    Register_OnLoad_Init(init, "extensions.live_galaxy.lua.live_galaxy_runtime")
end

return runtime
