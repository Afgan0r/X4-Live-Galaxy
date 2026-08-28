local runtime = {}
local PIPE_NAME = "live_galaxy"

function runtime.emit(payload)
    local ok_api, api = pcall(require, "extensions.sn_mod_support_apis.ui.named_pipes")
    if not ok_api or type(api) ~= "table" or type(api.Interface) ~= "table" then
        return false, "pipe_unavailable"
    end
    local writer = api.Interface._Write_Pipe_Raw
    if type(writer) ~= "function" or type(payload) ~= "string" or #payload > 512 then
        return false, "pipe_rejected"
    end
    local ok_write = pcall(writer, PIPE_NAME, payload)
    if not ok_write then
        return false, "pipe_unavailable"
    end
    return true, "sent"
end

local generation, sequence, connected = 0, 0, false
local MAX_COUNTER = 9007199254740991

local function payload(kind, extra)
    if sequence == MAX_COUNTER then return nil end
    sequence = sequence + 1
    return '{"type":"' .. kind .. '","scope":"runtime:sectors","version":1,"generation":' .. generation .. ',"sequence":' .. sequence .. extra .. '}'
end

function runtime.next_payload()
    if not connected then
        if generation == MAX_COUNTER then return nil end
        generation, sequence, connected = generation + 1, 0, true
        return '{"type":"hello","protocol_major":1,"game_build":"live-galaxy-x4-build-1","capabilities":["live-galaxy-observation-v1"],"generation":' .. generation .. '}'
    end
    local step = (sequence % 4) + 1
    if step == 1 then return payload("heartbeat", "") end
    if step == 2 then return payload("runtime_health", ',"status":"available"') end
    if step == 3 then return payload("observation", ',"entity_id":"sector:live_galaxy","observed_at_unix_millis":1,"quality":"unknown","content":"runtime_probe"') end
    return payload("complete_marker", "")
end

local function init()
    RegisterEvent("live_galaxy_observation", function()
        local value = runtime.next_payload()
        if not value then return false, "counter_exhausted" end
        local ok, status = runtime.emit(value)
        if not ok and status == "pipe_unavailable" then connected = false end
        return ok, status
    end)
end

Register_OnLoad_Init(init, "extensions.live_galaxy.lua.live_galaxy_runtime")

return runtime
