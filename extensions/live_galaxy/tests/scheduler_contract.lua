local scheduler = require("live_galaxy_scheduler")

local cases = {}

function cases.samples_one_bounded_slice_per_tick()
    local calls = 0
    local result = scheduler.sample_slice({
        sample = function(_, limit)
            calls = calls + 1
            assert(limit == 1)
            return "telemetry"
        end,
    }, 1)

    assert(calls == 1)
    assert(result.disposition == "sampled")
end

function cases.marks_backpressure_and_unavailable_without_waiting()
    local saturated = scheduler.enqueue_or_backpressure({
        try_enqueue = function()
            return false, "queue_saturated"
        end,
    }, "telemetry")
    local unavailable = scheduler.enqueue_or_backpressure(nil, "telemetry")

    assert(saturated.disposition == "backpressure")
    assert(unavailable.disposition == "bridge_unavailable")
end

function cases.suppresses_during_save_sensitive_windows()
    assert(scheduler.save_suppressed({ save_in_progress = true }))
    assert(not scheduler.save_suppressed({ save_in_progress = false }))
end

return cases
