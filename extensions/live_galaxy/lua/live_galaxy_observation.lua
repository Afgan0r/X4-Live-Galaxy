local observation = {}

local function finite(value)
    return type(value) == "number" and value == value and value ~= math.huge and value ~= -math.huge
end

function observation.new(options)
    options = options or {}
    local getter = options.getter
    if getter == nil then
        local ok, ffi = pcall(require, "ffi")
        if not ok or type(ffi) ~= "table" or type(ffi.C) ~= "userdata" and type(ffi.C) ~= "table" then
            return nil, "getter_loader_failure"
        end
        getter = function() return ffi.C.GetCurRealTime() end
    end
    if type(getter) ~= "function" then return nil, "getter_unavailable" end
    return {
        getter = getter,
        begin_evidence = observation.begin_evidence,
        finish_evidence = observation.finish_evidence,
        capture = observation.capture,
    }
end

function observation.begin_evidence(context)
    if type(context) ~= "table" or type(context.capture_start_millis) ~= "string"
        or not context.capture_start_millis:match("^%d+$") then
        return nil, "clock_unavailable"
    end
    return {
        section_key = "carrier_b_realtime_sample",
        expected_records = 1,
        capture_start_millis = context.capture_start_millis,
        capture_clock = "game_time_millis",
        quality = "unknown",
        availability = "available",
        coverage = "point_measurement",
        consistency = "unknown",
        stable_identity = false,
    }
end

function observation:capture()
    local ok, value = pcall(self.getter)
    if not ok then return nil, "source_failure" end
    if not finite(value) then return nil, "invalid_fact" end
    local raw = tostring(value)
    if raw == "" or #raw > 96 then return nil, "invalid_fact" end
    return {
        entity_id = "x4:runtime:realtime_clock",
        observation_version = 1,
        getter = "GetCurRealTime",
        raw_value = raw,
        semantics = "opaque_runtime_number",
    }
end

function observation.finish_evidence(context)
    if type(context) ~= "table" or type(context.capture_end_millis) ~= "string"
        or not context.capture_end_millis:match("^%d+$") then
        return nil, "clock_unavailable"
    end
    return {
        capture_end_millis = context.capture_end_millis,
        success = true,
        quality = "unknown",
        availability = "available",
        coverage = "point_measurement",
        consistency = "unknown",
        stable_identity = false,
    }
end

return observation
