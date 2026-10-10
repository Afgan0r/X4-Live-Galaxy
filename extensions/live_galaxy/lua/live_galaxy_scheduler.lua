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
local function discard(observation, carrier, reason)
    if observation.discard then pcall(observation.discard, observation, carrier, reason)
    elseif observation.collector then pcall(observation.collector.discard, observation.collector, carrier, reason) end
end

function scheduler.tick(context, carrier, observation)
    if context == "telemetry_tick" then
        context = { source_epoch_status = "unknown", source_boundary = "runtime_start" }
    end
    if type(context) ~= "table" then return { disposition = "clock_unavailable" } end
    if context.reentry_guard then return { disposition = "reentry_suppressed" } end
    context.reentry_guard = true
    local function finish(disposition, code, status)
        context.reentry_guard = false
        local rejection
        if type(status) == "table" then
            rejection = { stage = "scheduler", condition = disposition,
                section = status.selection, revision = status.collection_revision,
                run = status.producer_incarnation }
        end
        return { disposition = disposition, code = code, rejection = rejection }
    end

    local progress, progress_status = call(carrier, "progress", 1)
    if progress == nil then return finish(progress_status, nil) end
    local control, control_error = call(carrier, "poll_control")
    if control == nil then return finish(control_error, nil) end
    if control == 4 and not terminal[progress] then
        -- Received seals one immutable successor. Hand it off now, without
        -- polling again or waiting for its acknowledgement in this callback.
        local prior_status = progress_status
        progress, progress_status = call(carrier, "progress", 1)
        if progress == nil or progress < 0 then
            local reason = progress == nil and progress_status or "progress_rejected"
            discard(observation, carrier, reason)
            -- A failed operation may supply no snapshot. Retain only owned
            -- correlation from the prior one; never use it for admission.
            local correlation = type(progress_status) == "table" and progress_status or prior_status
            local result = finish(reason, progress, correlation)
            if result.rejection then
                result.rejection.operation = "progress"
                result.rejection.carrier_code = progress
            end
            return result
        end
    end
    if terminal[progress] then
        discard(observation, carrier, terminal[progress])
        -- A replacement core intent may recover a paused heavy producer.
        -- Poll control before returning so that pause cannot starve recovery.
        return finish(terminal[progress], progress, progress_status)
    end
    if type(observation.feedback) == "function" then
        local ok, result = pcall(observation.feedback, observation, context, carrier, progress_status, control)
        if not ok then discard(observation, carrier, "source_failure"); return finish("source_failure", nil, progress_status) end
        if result then context.reentry_guard = false; return result end
    end
    if terminal[control] then
        discard(observation, carrier, terminal[control])
        return finish(terminal[control], control, progress_status)
    end
    if type(progress_status) ~= "table"
        or type(progress_status.capacity) ~= "string"
        or not progress_status.capacity:match("^available:%d+$") then
        return finish("producer_busy", progress, progress_status)
    end

    if type(observation.advance) == "function" then
        local ok, result = pcall(observation.advance, observation, context, carrier, progress_status)
        if not ok or type(result) ~= "table" then
            if observation.discard then observation:discard(carrier, "source_failure")
            elseif observation.collector then observation.collector:discard(carrier, "source_failure") end
            return finish("source_failure", nil, progress_status)
        end
        context.reentry_guard = false
        return result
    end

    local begin, begin_error = call(observation, "begin_evidence")
    if begin == nil then return finish(begin_error, nil, progress_status) end
    begin.source_epoch_status = context.source_epoch_status or "unknown"
    begin.source_boundary = context.source_boundary or "runtime_start"
    local reserved = call(carrier, "begin_section", begin)
    if reserved == -21 then return finish("producer_busy", reserved, progress_status) end
    if reserved ~= 0 then return finish("reservation_failed", reserved, progress_status) end

    local fact, source_error = call(observation, "capture")
    if fact == nil then
        call(carrier, "fail_section", source_error or "source_failure")
        return finish(source_error or "source_failure", nil, progress_status)
    end
    local pushed = call(carrier, "push_record", fact)
    if pushed ~= 0 then
        call(carrier, "fail_section", "fact_rejected")
        return finish("fact_rejected", pushed, progress_status)
    end
    local completion, completion_error = call(observation, "finish_evidence")
    if completion == nil then
        call(carrier, "fail_section", completion_error or "clock_unavailable")
        return finish(completion_error or "clock_unavailable", nil, progress_status)
    end
    local closed = call(carrier, "finish_section", completion)
    if closed ~= 0 then return finish("finish_rejected", closed, progress_status) end
    return finish("sampled", 0, progress_status)
end

return scheduler
