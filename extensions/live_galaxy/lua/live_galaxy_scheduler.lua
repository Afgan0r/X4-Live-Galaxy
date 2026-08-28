local scheduler = {}

local MAX_SLICES_PER_TICK = 1

function scheduler.save_suppressed(context)
    return type(context) == "table" and context.save_in_progress == true
end

function scheduler.sample_slice(producer, limit)
    if type(producer) ~= "table" or type(producer.sample) ~= "function" then
        return { disposition = "producer_unavailable" }
    end

    local requested = type(limit) == "number" and math.floor(limit) or 1
    local bounded = math.max(1, math.min(requested, MAX_SLICES_PER_TICK))
    local payload, err = producer:sample(bounded)
    if payload == nil then
        return { disposition = err or "observation_unavailable" }
    end
    return { disposition = "sampled", payload = payload }
end

function scheduler.enqueue_or_backpressure(bridge, payload)
    if type(bridge) ~= "table" or type(bridge.try_enqueue) ~= "function" then
        return { disposition = "bridge_unavailable" }
    end

    local accepted, reason = bridge:try_enqueue(payload)
    if accepted then
        return { disposition = "enqueued" }
    end
    if reason == "queue_saturated" then
        return { disposition = "backpressure" }
    end
    return { disposition = "bridge_unavailable" }
end

function scheduler.tick(context, producer, bridge)
    if scheduler.save_suppressed(context) then
        return { disposition = "save_suppressed" }
    end

    local sampled = scheduler.sample_slice(producer, MAX_SLICES_PER_TICK)
    if sampled.disposition ~= "sampled" then
        return sampled
    end
    return scheduler.enqueue_or_backpressure(bridge, sampled.payload)
end

return scheduler
