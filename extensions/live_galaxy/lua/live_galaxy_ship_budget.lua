local budget = {}
local function integer(v) return type(v) == "number" and v >= 0 and v <= 9007199254740991 and v % 1 == 0 end
function budget.new(api, limits)
    local self = { calls = 0, allocation = 0, steps = 0, source_value_bytes = 0, v = limits }
    local wrapped = {}
    for name, fn in pairs(api) do
        wrapped[name] = function(_, ...)
            self.operation = name
            if name:match("_size$") then return fn(api, ...) end
            if name == "faction_string" then
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
            else
                if self.calls >= limits.max_native_calls then
                    self.reason = "native_call_limit"; error(self.reason, 0)
                end
                self.calls = self.calls + 1
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
    return getter()
end
function budget:sample_callback(carrier)
    -- Status-only progress(0) reads the owned monotonic clock between
    -- indivisible source or carrier operations. It cannot interrupt an
    -- operation which has already entered X4.
    local _, status = carrier:progress(0)
    local now = status and tonumber(status.monotonic_millis)
    if not integer(now) or now < self.callback_start then return nil, "clock_unavailable" end
    self.last = now
    self.max_callback_duration = math.max(self.max_callback_duration or 0, now - self.callback_start)
    self.max_callback_overrun = math.max(0, self.max_callback_duration - self.v.callback_budget_millis)
    if now - self.started > self.v.max_message_age_millis then return nil, "collection_overflow" end
    return now
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
