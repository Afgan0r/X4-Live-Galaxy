local runtime = {}
local prefix = "live_galaxy/lua/"
local carrier_module = require(prefix .. "live_galaxy_carrier")
local observation_module = require(prefix .. "live_galaxy_observation")
local scheduler = require(prefix .. "live_galaxy_scheduler")

local active_carrier, active_observation
local initialized, callback_active = false, false
local last_diagnostic

local function diagnostic(event, detail)
    local value = event .. ":" .. tostring(detail):sub(1, 64)
    if value == last_diagnostic then return end
    last_diagnostic = value
    if type(DebugError) == "function" then
        DebugError("Live Galaxy Carrier B: event=" .. event .. " detail=" .. tostring(detail):sub(1, 64))
    end
end

function runtime.handle_tick(context)
    if callback_active then return false, "reentry_suppressed" end
    if active_carrier == nil or active_observation == nil then return false, "adapter_unavailable" end
    callback_active = true
    local ok, result = pcall(scheduler.tick, context, active_carrier, active_observation)
    callback_active = false
    if not ok or type(result) ~= "table" then
        diagnostic("callback_failure", ok and "invalid_result" or result)
        return false, "callback_failure"
    end
    if result.disposition ~= "producer_busy" then
        diagnostic("transition", result.disposition)
    end
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
