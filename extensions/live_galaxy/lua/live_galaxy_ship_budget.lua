local budget = {}
local function integer(v) return type(v) == "number" and v >= 0 and v <= 9007199254740991 and v % 1 == 0 end
local function aggregate() return { count = 0, total_millis = 0, max_millis = 0 } end
local function add(value, elapsed)
    value.count, value.total_millis = value.count + 1, value.total_millis + elapsed
    value.max_millis = math.max(value.max_millis, elapsed)
end
function budget.new(api, limits)
    local timing = { intervals = {}, operations = {}, callbacks = aggregate(), between_callbacks = aggregate(),
        status_samples = 0, clock_regressions = 0, unsampled_callbacks = 0,
        sampling_overhead = "included_unmeasured", scope = "selection_observed_intervals",
        clock = "carrier_monotonic_millis" }
    for _, category in ipairs({ "source", "allocation", "clock", "processing", "delivery", "mixed" }) do
        timing.intervals[category], timing.operations[category] = aggregate(), 0
    end
    local self = { calls = 0, allocation = 0, steps = 0, source_value_bytes = 0, v = limits, timing = timing }
    local wrapped = {}
    for name, fn in pairs(api) do
        wrapped[name] = function(_, ...)
            self.operation = name
            if name:match("_size$") then self:observe_operation("source"); return fn(api, ...) end
            if name == "faction_string" then
                self:observe_operation("source")
                local value = fn(api, ...); self:charge_source_value(value); return value
            end
            self.callback_operations = self.callback_operations + 1
            if name:match("_allocate$") or name == "new_buffer" or name == "new_faction_buffer" then
                local count = select(1, ...)
                local size = (name == "new_buffer" or name == "new_faction_buffer") and 8
                    or api[name:gsub("_allocate$", "_size")](api)
                if not integer(count) or not integer(size) or size == 0
                    or count > math.floor(limits.max_allocation_bytes / size)
                    or count * size > limits.max_total_allocation_bytes - self.allocation
                    or count * size > limits.max_total_bytes - self.allocation - self.source_value_bytes then
                    self.reason = "allocation_limit"; error(self.reason, 0)
                end
                self.allocation = self.allocation + count * size
                self:observe_operation("allocation")
            else
                if self.calls >= limits.max_native_calls then
                    self.reason = "native_call_limit"; error(self.reason, 0)
                end
                self.calls = self.calls + 1
                self:observe_operation("source")
            end
            local value, rejection = fn(api, ...)
            -- Native buffers are charged by capacity, never traversed here.
            if not name:match("_allocate$") and name ~= "new_buffer" and name ~= "new_faction_buffer" then
                self:charge_source_value(value)
            end
            return value, rejection
        end
    end
    self.api = wrapped
    return setmetatable(self, { __index = budget })
end
function budget:observe_operation(category)
    -- Categories describe an observed step interval, not exclusive getter CPU.
    -- Several classes before one existing sample are explicitly mixed.
    self.timing.operations[category] = self.timing.operations[category] + 1
    if self.interval_category and self.interval_category ~= category then self.interval_category = "mixed"
    else self.interval_category = self.interval_category or category end
end
function budget:charge_source_value(value, ancestors)
    -- Charge represented source values, not a claim about exact Lua heap size.
    -- Include keys and delimiters; cumulative charging is conservative across
    -- repeated reads. Native allocation and retained values share total bytes.
    local kind = type(value)
    if kind == "table" then
        ancestors = ancestors or {}
        if ancestors[value] then
            self.reason, self.condition = "invalid_fact", "source_value_cycle"
            error(self.reason, 0)
        end
        ancestors[value] = true
        for key, child in pairs(value) do
            self:step()
            self:charge_source_value(key, ancestors)
            self:charge_source_value(child, ancestors)
        end
        ancestors[value] = nil
        return
    end
    local bytes = (kind == "string" and #value or #tostring(value)) + 2
    if bytes > self.v.max_candidate_raw_bytes - self.source_value_bytes
        or bytes > self.v.max_total_bytes - self.allocation - self.source_value_bytes then
        self.reason = "allocation_limit"; error(self.reason, 0)
    end
    self.source_value_bytes = self.source_value_bytes + bytes
end
function budget:before(status)
    local now = tonumber(status.monotonic_millis)
    if not integer(now) or self.last and now <= self.last then return nil, "clock_unavailable" end
    self.started = self.started or now
    if now - self.started > self.v.max_message_age_millis or self.steps >= self.v.max_collection_steps then
        return nil, "collection_overflow"
    end
    if self.callback_end then add(self.timing.between_callbacks, math.max(0, now - self.callback_end)) end
    self.timing.callbacks.count = self.timing.callbacks.count + 1
    self.timing.unsampled_callbacks = self.timing.unsampled_callbacks + 1
    self.callback_sampled = false
    self.interval_start, self.interval_category, self.callback_elapsed = now, nil, 0
    self.last, self.callback_start, self.callback_operations, self.callback_yielded = now, now, 0, false
    return true
end
function budget:step()
    if self.steps >= self.v.max_collection_steps then
        self.reason = "collection_overflow"; error(self.reason, 0)
    end
    self.steps = self.steps + 1
end
function budget:clock(getter)
    self.operation = "clock_getter"
    if self.calls >= self.v.max_native_calls then
        self.reason = "native_call_limit"; error(self.reason, 0)
    end
    self.callback_operations, self.calls = self.callback_operations + 1, self.calls + 1
    self:observe_operation("clock")
    return getter()
end
function budget:sample_callback(carrier)
    -- Status-only progress(0) reads the owned monotonic clock between
    -- indivisible source or carrier operations. It cannot interrupt an
    -- operation which has already entered X4.
    local _, status = carrier:progress(0)
    self.timing.status_samples = self.timing.status_samples + 1
    local now = status and tonumber(status.monotonic_millis)
    if not integer(now) or now < self.callback_start then return nil, "clock_unavailable" end
    if not self.callback_sampled then
        self.timing.unsampled_callbacks = self.timing.unsampled_callbacks - 1
        self.callback_sampled = true
    end
    local elapsed = math.max(0, now - self.interval_start)
    if now < self.interval_start then self.timing.clock_regressions = self.timing.clock_regressions + 1 end
    add(self.timing.intervals[self.interval_category or "processing"], elapsed)
    self.interval_start, self.interval_category = math.max(now, self.interval_start), nil
    local callback_elapsed = math.max(self.callback_elapsed, now - self.callback_start)
    self.timing.callbacks.total_millis = self.timing.callbacks.total_millis + callback_elapsed - self.callback_elapsed
    self.timing.callbacks.max_millis = math.max(self.timing.callbacks.max_millis, callback_elapsed)
    self.callback_elapsed, self.callback_end = callback_elapsed, self.interval_start
    self.last = now
    self.max_callback_duration = math.max(self.max_callback_duration or 0, now - self.callback_start)
    self.max_callback_overrun = math.max(0, self.max_callback_duration - self.v.callback_budget_millis)
    if now - self.started > self.v.max_message_age_millis then return nil, "collection_overflow" end
    return now
end
function budget:timing_snapshot()
    -- The budget survives retained detail delivery; emitted diagnostics must
    -- keep their own values and identify this lifetime separately from request.
    local result = { pending_operations = self.interval_category ~= nil }
    for key, value in pairs(self.timing) do
        if type(value) ~= "table" then result[key] = value end
    end
    result.intervals, result.operations = {}, {}
    for category, value in pairs(self.timing.intervals) do
        result.intervals[category] = { count = value.count, total_millis = value.total_millis, max_millis = value.max_millis }
        result.operations[category] = self.timing.operations[category]
    end
    for _, key in ipairs({ "callbacks", "between_callbacks" }) do
        local value = self.timing[key]
        result[key] = { count = value.count, total_millis = value.total_millis, max_millis = value.max_millis }
    end
    return result
end
function budget:should_yield(carrier)
    local now, reason = self:sample_callback(carrier)
    if not now then
        self.reason = reason
        error(reason, 0)
    end
    self.callback_yielded = now - self.callback_start >= self.v.callback_budget_millis
    return self.callback_yielded
end
function budget:after(carrier)
    if self.callback_yielded then return true end
    local now, reason = self:sample_callback(carrier)
    if not now then return nil, reason end
    return true
end
return budget
