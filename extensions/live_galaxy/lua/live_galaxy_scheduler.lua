local scheduler = {}

local terminal = { [6] = true, [7] = true, [8] = true, [9] = true, [10] = true }

local function call(target, name, ...)
    if type(target) ~= "table" or type(target[name]) ~= "function" then
        return nil, "adapter_unavailable"
    end
    return target[name](target, ...)
end

function scheduler.tick(context, carrier, observation)
    if type(context) ~= "table" then return { disposition = "clock_unavailable" } end
    if context.reentry_guard then return { disposition = "reentry_suppressed" } end
    context.reentry_guard = true
    local function finish(disposition, code)
        context.reentry_guard = false
        return { disposition = disposition, code = code }
    end

    local progress, progress_error = call(carrier, "progress", 1)
    if progress == nil then return finish(progress_error, nil) end
    if terminal[progress] then return finish("producer_terminal", progress) end
    local control, control_error = call(carrier, "poll_control")
    if control == nil then return finish(control_error, nil) end
    if terminal[control] then return finish("producer_terminal", control) end

    local begin, begin_error = observation.begin_evidence(context)
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
    local completion, completion_error = observation.finish_evidence(context)
    if completion == nil then
        call(carrier, "fail_section", completion_error or "clock_unavailable")
        return finish(completion_error or "clock_unavailable", nil)
    end
    local closed = call(carrier, "finish_section", completion)
    if closed ~= 0 then return finish("finish_rejected", closed) end
    return finish("sampled", 0)
end

return scheduler
