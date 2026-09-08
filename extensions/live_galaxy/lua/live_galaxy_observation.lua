local observation = {}

local function finite(value)
    return type(value) == "number" and value == value and value ~= math.huge and value ~= -math.huge
end

local function game_time_millis(getter)
    local ok, seconds = pcall(getter)
    if not ok or not finite(seconds) or seconds < 0 or seconds > 9007199254740 then
        return nil, "clock_unavailable"
    end
    return tostring(math.floor(seconds * 1000))
end

function observation.new(options)
    options = options or {}
    local getter, clock_getter = options.getter, options.clock_getter
    if getter == nil or clock_getter == nil then
        local ok, ffi = pcall(require, "ffi")
        if not ok or type(ffi) ~= "table" or type(ffi.C) ~= "userdata" and type(ffi.C) ~= "table" then
            return nil, "getter_loader_failure"
        end
        if getter == nil then getter = function() return ffi.C.GetCurRealTime() end end
        if clock_getter == nil then
            clock_getter = function() return ffi.C.GetCurrentGameTime() end
        end
    end
    if type(getter) ~= "function" then return nil, "getter_unavailable" end
    if type(clock_getter) ~= "function" then return nil, "clock_unavailable" end
    return {
        getter = getter,
        clock_getter = clock_getter,
        begin_evidence = observation.begin_evidence,
        finish_evidence = observation.finish_evidence,
        capture = observation.capture,
    }
end

function observation:begin_evidence()
    local capture_start, err = game_time_millis(self.clock_getter)
    if capture_start == nil then return nil, err end
    return {
        section_key = "carrier_b_realtime_sample",
        expected_records = 1,
        capture_start_millis = capture_start,
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

function observation:finish_evidence()
    local capture_end, err = game_time_millis(self.clock_getter)
    if capture_end == nil then return nil, err end
    return {
        capture_end_millis = capture_end,
        success = true,
        quality = "unknown",
        availability = "available",
        coverage = "point_measurement",
        consistency = "unknown",
        stable_identity = false,
    }
end

return observation
