local helper = require("test_helper")
local scheduler, fixture

describe("scheduler", function()
after_each(function() if fixture then fixture.restore() end end)
before_each(function()
    fixture = helper.new()
    scheduler = fixture.load("live_galaxy_scheduler")
end)

it("samples one bounded slice per tick", function()
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
end)

it("marks backpressure and unavailable without waiting", function()
    local saturated = scheduler.enqueue_or_backpressure({
        try_enqueue = function()
            return false, "queue_saturated"
        end,
    }, "telemetry")
    local unavailable = scheduler.enqueue_or_backpressure(nil, "telemetry")

    assert(saturated.disposition == "backpressure")
    assert(unavailable.disposition == "bridge_unavailable")
end)

it("suppresses during save sensitive windows", function()
    assert(scheduler.save_suppressed({ save_in_progress = true }))
    assert(not scheduler.save_suppressed({ save_in_progress = false }))
end)

end)
