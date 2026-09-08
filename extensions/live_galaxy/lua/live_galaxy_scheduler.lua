local scheduler = {}

local terminal = {
    [6] = "permanently_rejected", [7] = "ambiguous_commit",
    [8] = "retry_exhausted", [9] = "disconnected", [10] = "restart_required",
}

local function call(target, name, ...)
    if type(target) ~= "table" or type(target[name]) ~= "function" then
        return nil, "adapter_unavailable"
    end
    return target[name](target, ...)
end

function scheduler.tick(context, carrier, observation)
    if context == "telemetry_tick" then context = {} end
    if type(context) ~= "table" then return { disposition = "clock_unavailable" } end
    if context.reentry_guard then return { disposition = "reentry_suppressed" } end
    context.reentry_guard = true
    local function finish(disposition, code)
        context.reentry_guard = false
        return { disposition = disposition, code = code }
    end

    local progress, progress_status = call(carrier, "progress", 1)
    if progress == nil then return finish(progress_status, nil) end
    if terminal[progress] then return finish(terminal[progress], progress) end
    local control, control_error = call(carrier, "poll_control")
    if control == nil then return finish(control_error, nil) end
    if terminal[control] then return finish(terminal[control], control) end

    local begin, begin_error = call(observation, "begin_evidence")
    if begin == nil then return finish(begin_error, nil) end
    local reserved = call(carrier, "begin_section", begin)
    if reserved == -21 then return finish("producer_busy", reserved) end
    if reserved ~= 0 then return finish("reservation_failed", reserved) end

    local fact, source_error = call(observation, "capture")
    if fact == nil then
        call(carrier, "fail_section", source_error or "source_failure")
        return finish(source_error or "source_failure", nil)
    end
    local pushed = call(carrier, "push_record", fact)
    if pushed ~= 0 then
        call(carrier, "fail_section", "fact_rejected")
        return finish("fact_rejected", pushed)
    end
    local completion, completion_error = call(observation, "finish_evidence")
    if completion == nil then
        call(carrier, "fail_section", completion_error or "clock_unavailable")
        return finish(completion_error or "clock_unavailable", nil)
    end
    local closed = call(carrier, "finish_section", completion)
    if closed ~= 0 then return finish("finish_rejected", closed) end
    return finish("sampled", 0)
end

return scheduler
