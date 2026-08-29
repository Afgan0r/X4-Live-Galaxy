local runtime = {}
local PIPE_NAME = "live_galaxy"
local MAX_PIPE_PAYLOAD_BYTES = 2048
package.path = package.path .. ";extensions/?.lua"
local LOCAL_MODULE_PREFIX = "live_galaxy/lua/"
local DEFAULT_TRACE_MAX_EVENTS = 64
local VERSION_DIAGNOSTIC_MAX_BYTES = 64
local DISCOVERY_DIAGNOSTIC_CLASSES = {
    asset_identity_invalid = true,
    capacity_invalid = true,
    capacity_unavailable = true,
    metadata_unavailable = true,
    owner_invalid = true,
    owner_scope_empty = true,
    owner_scope_mismatch = true,
    sector_invalid = true,
}

local function require_live_galaxy_module(name)
    return require(LOCAL_MODULE_PREFIX .. name)
end

local TRACE_CONFIG_MODULE = LOCAL_MODULE_PREFIX .. "live_galaxy_trace_config"

local discovery = require_live_galaxy_module("live_galaxy_x4_discovery")
local telemetry = require_live_galaxy_module("live_galaxy_telemetry")

local function load_trace_config()
    local ok, config = pcall(require, TRACE_CONFIG_MODULE)
    if not ok or type(config) ~= "table" then
        return false, "unset", DEFAULT_TRACE_MAX_EVENTS, "unavailable", false
    end
    local enabled = config.enabled == true
    local attempt_id = type(config.attempt_id) == "string" and config.attempt_id or "unset"
    local max_events = tonumber(config.max_frame_events) or DEFAULT_TRACE_MAX_EVENTS
    local status = enabled and "enabled" or "disabled"
    return enabled, attempt_id:sub(1, 64), math.max(1, math.min(max_events, DEFAULT_TRACE_MAX_EVENTS)), status,
        config.version_diagnostic_enabled == true
end

local trace_enabled, trace_attempt_id, trace_max_events, trace_config_status, version_diagnostic_enabled = load_trace_config()
local trace_frame_events = 0
local version_diagnostic_emitted = false

local function trace(event, detail, is_frame)
    if is_frame then
        if not trace_enabled or trace_frame_events >= trace_max_events then
            return
        end
        trace_frame_events = trace_frame_events + 1
    end
    if type(DebugError) == "function" then
        DebugError("Live Galaxy runtime: attempt_id=" .. trace_attempt_id
            .. " hop=lua event=" .. event .. " detail=" .. detail)
    end
end

function runtime.sanitize_embedded_version(value)
    if type(value) ~= "string" or value == "" then
        return "unavailable"
    end
    return value:gsub("[^ -~]", "_"):sub(1, VERSION_DIAGNOSTIC_MAX_BYTES)
end

local function trace_embedded_version_once()
    if not version_diagnostic_enabled or version_diagnostic_emitted then
        return
    end
    version_diagnostic_emitted = true
    trace("embedded_lua_version", runtime.sanitize_embedded_version(_VERSION))
end

local pipe_adapter

local function resolve_pipe_adapter()
    if type(pipe_adapter) == "table" then return pipe_adapter end
    local ok_api, pipes = pcall(require, "extensions.sn_mod_support_apis.ui.named_pipes.Interface")
    if not ok_api or type(pipes) ~= "table" then return nil end
    if type(pipes._Write_Pipe_Raw) ~= "function" or type(pipes.Disconnect_Pipe) ~= "function" then
        return nil
    end
    return { write_raw = pipes._Write_Pipe_Raw, disconnect = pipes.Disconnect_Pipe }
end

function runtime.emit(payload)
    local adapter = resolve_pipe_adapter()
    if adapter == nil then
        trace("pipe_api_unavailable", "named pipe module missing")
        return false, "pipe_unavailable"
    end
    if type(adapter.write_raw) ~= "function" or type(adapter.disconnect) ~= "function"
        or type(payload) ~= "string" or #payload > MAX_PIPE_PAYLOAD_BYTES then
        trace("payload_rejected", "invalid writer or payload")
        return false, "pipe_rejected"
    end

    local invocation_ok, write_result = pcall(adapter.write_raw, PIPE_NAME, payload)
    if not invocation_ok then
        pcall(adapter.disconnect, PIPE_NAME)
        trace("pipe_reconnect_required", "discarded frame after write failure")
        return false, "pipe_reconnect"
    end
    if write_result == false then
        trace("pipe_write_deferred", "writer reported bounded pipe backpressure")
        return false, "pipe_backpressure"
    end
    trace("pipe_write_succeeded", "bytes=" .. #payload, true)
    return true, "sent"
end

local generation, sequence, observation_version, connected = 0, 0, 1, false
local discovery_incomplete = false
local discovery_frames, discovery_frame_index
local MAX_COUNTER = 9007199254740991
local discovery_adapter

local function discard_discovery_frames()
    discovery_frames, discovery_frame_index = nil, nil
end

local function payload(kind, extra)
    if sequence == MAX_COUNTER then return nil end
    sequence = sequence + 1
    return '{"type":"' .. kind .. '","scope":"runtime:sectors","version":' .. observation_version .. ',"generation":' .. generation .. ',"sequence":' .. sequence .. extra .. '}', kind, generation, sequence
end

function runtime.next_payload()
    if not connected then
        if generation == MAX_COUNTER then return nil end
        generation, sequence, connected = generation + 1, 0, true
        return '{"type":"hello","protocol_major":1,"game_build":"live-galaxy-x4-build-2","capabilities":["live-galaxy-observation-v2"],"generation":' .. generation .. '}', "hello", generation, 0
    end
    if discovery_frames then
        if discovery_frame_index <= #discovery_frames then
            local frame = discovery_frames[discovery_frame_index]
            discovery_frame_index = discovery_frame_index + 1
            return payload("observation", "," .. frame:sub(2, -2))
        end
        discard_discovery_frames()
        local marker, kind, current_generation, current_sequence = payload("complete_marker", "")
        if marker ~= nil and observation_version < MAX_COUNTER then observation_version = observation_version + 1 end
        return marker, kind, current_generation, current_sequence
    end
    local step = (sequence % 4) + 1
    if step == 1 then return payload("heartbeat", "") end
    if step == 2 then return payload("runtime_health", ',"status":"available"') end
    if step == 3 then
        local adapter = discovery_adapter or discovery.new_runtime_adapter()
        local frames, err = telemetry.produce_observations(adapter, observation_version)
        if frames == nil then
            trace("discovery_unavailable", tostring(err))
            discovery_incomplete = true
            return payload("runtime_health", ',"status":"unavailable"')
        end
        discovery_frames, discovery_frame_index = frames, 2
        return payload("observation", "," .. frames[1]:sub(2, -2))
    end
    if discovery_incomplete then
        discovery_incomplete = false
        return payload("runtime_health", ',"status":"unavailable"')
    end
    local marker, kind, current_generation, current_sequence = payload("complete_marker", "")
    if marker ~= nil and observation_version < MAX_COUNTER then
        observation_version = observation_version + 1
    end
    return marker, kind, current_generation, current_sequence
end

function runtime.set_discovery_adapter(adapter)
    discovery_adapter = adapter
end

function runtime.set_pipe_adapter(adapter)
    pipe_adapter = adapter
end

function runtime.produce_discovery_payload(adapter, version)
    local observation, err = telemetry.produce_observation(adapter, version)
    if observation == nil then
        if trace_enabled and err == "facts_unsupported" and type(adapter) == "table"
            and type(adapter.diagnostic_class) == "function" then
            local diagnostic_ok, diagnostic_class = pcall(adapter.diagnostic_class, adapter)
            if diagnostic_ok and DISCOVERY_DIAGNOSTIC_CLASSES[diagnostic_class] then
                trace("component_discovery_class", diagnostic_class)
            end
        end
        trace("discovery_unavailable", tostring(err))
        return nil, "discovery_" .. tostring(err)
    end
    local body = observation:sub(2, -2)
    if body == "" or body:find("sector:live_galaxy", 1, true) then
        trace("discovery_invalid", "missing or retired observation identity")
        return nil, "discovery_invalid"
    end
    return body
end

function runtime.handle_tick()
    local value, kind, current_generation, current_sequence = runtime.next_payload()
    if not value then return false, kind or "counter_exhausted" end
    trace("frame_created", "type=" .. kind .. " generation=" .. current_generation
        .. " sequence=" .. current_sequence .. " bytes=" .. #value, true)
    local ok, status = runtime.emit(value)
    if not ok and (status == "pipe_unavailable" or status == "pipe_reconnect") then
        connected = false
        discard_discovery_frames()
    end
    trace("telemetry_tick", "status=" .. status, true)
    return ok, status
end

local function init()
    trace("trace_config_loaded", "status=" .. trace_config_status
        .. " enabled=" .. tostring(trace_enabled))
    trace_embedded_version_once()
    RegisterEvent("live_galaxy_observation", runtime.handle_tick)
    trace("handler_registered", "event=live_galaxy_observation")
end

if type(Register_OnLoad_Init) ~= "function" then
    trace("lifecycle_unavailable", "Register_OnLoad_Init missing")
    return runtime
end

Register_OnLoad_Init(init, "extensions.live_galaxy.lua.live_galaxy_runtime")

return runtime
