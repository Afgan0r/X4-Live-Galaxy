local budget = {}
local function integer(v) return type(v) == "number" and v >= 0 and v <= 9007199254740991 and v % 1 == 0 end
function budget.new(api, limits)
    local self = { calls = 0, allocation = 0, steps = 0, v = limits }
    local wrapped = {}
    for name, fn in pairs(api) do
        wrapped[name] = function(_, ...)
            if name:match("_size$") or name == "faction_string" then return fn(api, ...) end
            if self.callback_operations >= limits.heavy_permits then
                self.reason = "native_call_limit"; error(self.reason, 0)
            end
            self.callback_operations = self.callback_operations + 1
            if name:match("_allocate$") or name == "new_buffer" or name == "new_faction_buffer" then
                local count = select(1, ...)
                local size = (name == "new_buffer" or name == "new_faction_buffer") and 8
                    or api[name:gsub("_allocate$", "_size")](api)
                if not integer(count) or not integer(size) or size == 0
                    or count > math.floor(limits.max_allocation_bytes / size)
                    or count * size > limits.max_total_allocation_bytes - self.allocation then
                    self.reason = "allocation_limit"; error(self.reason, 0)
                end
                self.allocation = self.allocation + count * size
            else
                if self.calls >= limits.max_native_calls then
                    self.reason = "native_call_limit"; error(self.reason, 0)
                end
                self.calls = self.calls + 1
            end
            return fn(api, ...)
        end
    end
    self.api = wrapped
    return setmetatable(self, { __index = budget })
end
function budget:before(status)
    local now = tonumber(status.monotonic_millis)
    if not integer(now) or self.last and now <= self.last then return nil, "clock_unavailable" end
    self.started = self.started or now
    if now - self.started > self.v.max_message_age_millis or self.steps >= self.v.max_collection_steps then
        return nil, "collection_overflow"
    end
    self.last, self.callback_start, self.callback_operations = now, now, 0
    self.steps = self.steps + 1
    return true
end
function budget:clock(getter)
    if self.callback_operations >= self.v.heavy_permits or self.calls >= self.v.max_native_calls then
        self.reason = "native_call_limit"; error(self.reason, 0)
    end
    self.callback_operations, self.calls = self.callback_operations + 1, self.calls + 1
    return getter()
end
function budget:after(carrier)
    -- Status-only progress(0) reads the owned monotonic clock after return.
    -- It cannot interrupt a synchronous getter which has already entered X4.
    local _, status = carrier:progress(0)
    local now = status and tonumber(status.monotonic_millis)
    if not integer(now) or now < self.callback_start then return nil, "clock_unavailable" end
    if now - self.callback_start > self.v.callback_budget_millis then return nil, "callback_budget_exceeded" end
    return true
end
return budget
