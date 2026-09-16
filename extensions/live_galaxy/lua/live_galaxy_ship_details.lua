local details = {}
local source = require("live_galaxy.lua.live_galaxy_ship_source")
local MAX_INTEGER = 9007199254740991
local function integer(v, maximum)
    return type(v) == "number" and v >= 0 and v <= maximum and v % 1 == 0
end
local function token(v)
    return type(v) == "string" and #v > 0 and #v <= 128 and v:match("^[%w_:%-]+$") ~= nil
end

function details.new(options, clock)
    local limit = options.max_inner
    if not integer(limit, 65535) or limit == 0
        or not integer(options.max_allocation_bytes, MAX_INTEGER)
        or options.max_allocation_bytes == 0 then return nil, "invalid_limits" end
    return setmetatable({ api = options.ship_api or source.runtime(), clock = clock,
        limit = limit, allocation = options.max_allocation_bytes, stage = "reserve",
        group = options.group, scope = options.source_scope }, { __index = details })
end

function details:fail(carrier, reason)
    if self.reserved then carrier:fail_section(reason) end
    self.stage, self.pending, self.buffer = "failed", nil, nil
    return { disposition = reason }
end

function details:tick(context, carrier, status)
    local group = self.group
    if self.stage == "failed" then return { disposition = "failed" } end
    if status.selection ~= group.key then return self:fail(carrier, "restart_required") end
    if self.stage == "reserve" then
        local begin, err = self.clock:begin_evidence()
        if not begin then return self:fail(carrier, err) end
        begin.section_key, begin.expected_records = group.key, #group.members
        begin.coverage, begin.consistency, begin.stable_identity = "partial", "observed_count_fill_only", true
        begin.source_epoch_status, begin.source_boundary = "unknown", context.source_boundary or "runtime_start"
        if carrier:begin_section(begin) ~= 0 then return self:fail(carrier, "reservation_failed") end
        self.reserved, self.index, self.capture_start = true, 1, begin.capture_start_millis
        self.stage = "wares"
    elseif self.stage == "wares" then
        local raw = self.api:cargo_wares(group.members[self.index])
        if type(raw) ~= "table" or getmetatable(raw) ~= nil then return self:fail(carrier, "cargo_unknown") end
        local wares = {}
        for ware, amount in pairs(raw) do
            if #wares >= self.limit or not token(ware) or not integer(amount, MAX_INTEGER) then
                return self:fail(carrier, "collection_overflow")
            end
            wares[#wares + 1] = { ware = ware, amount_items = amount }
        end
        table.sort(wares, function(a, b) return a.ware < b.ware end)
        self.pending, self.stage = { wares = wares }, "count"
    elseif self.stage == "count" then
        self.count = self.api:cargo_storage_count(group.members[self.index])
        self.size = self.api:cargo_storage_size()
        if not integer(self.count, self.limit) or not integer(self.size, self.allocation)
            or self.size == 0 or self.count > math.floor(self.allocation / self.size) then
            return self:fail(carrier, "collection_overflow")
        end
        self.stage = "allocate"
    elseif self.stage == "allocate" then
        self.buffer = self.api:cargo_storage_allocate(self.count)
        self.stage = "fill"
    elseif self.stage == "fill" then
        local rows = self.api:cargo_storage_fill(group.members[self.index], self.buffer, self.count)
        self.buffer = nil
        if type(rows) ~= "table" or #rows > self.count then return self:fail(carrier, "enumeration_incomplete") end
        local seen = {}
        for _, row in ipairs(rows) do
            if not token(row.transport) or seen[row.transport]
                or not integer(row.capacity_cubic_metres, 4294967295)
                or not integer(row.occupied_cubic_metres, row.capacity_cubic_metres) then
                return self:fail(carrier, "invalid_fact")
            end
            seen[row.transport] = true
        end
        table.sort(rows, function(a, b) return a.transport < b.transport end)
        self.pending.storage, self.stage = rows, "record"
    elseif self.stage == "record" then
        local finish, err = self.clock:finish_evidence()
        if not finish then return self:fail(carrier, err) end
        local record = self.pending
        record.profile, record.source_scope = "ship_cargo", self.scope
        record.identity, record.owner = group.members[self.index], group.owner
        record.core_revision, record.member_revision = group.core_revision, group.core_revision
        record.policy_version = 2
        record.capture_start_millis, record.capture_end_millis = self.capture_start, finish.capture_end_millis
        record.source_evidence = "x4-9.00-steam-23660954-ship-detail-source-v1"
        record.wares_outcome = #record.wares == 0 and "empty" or "value"
        record.storage_outcome = #record.storage == 0 and "empty" or "value"
        local code = carrier:push_record(record)
        if code == -21 then return { disposition = "producer_busy" } end
        if code ~= 0 then return self:fail(carrier, "fact_rejected") end
        self.index, self.pending = self.index + 1, nil
        self.stage = self.index > #group.members and "complete" or "wares"
    elseif self.stage == "complete" then
        local finish, err = self.clock:finish_evidence()
        if not finish then return self:fail(carrier, err) end
        finish.coverage, finish.consistency, finish.stable_identity = "partial", "observed_count_fill_only", true
        if carrier:finish_section(finish) ~= 0 then return self:fail(carrier, "finish_rejected") end
        self.stage, self.reserved = "done", false
        return { disposition = "sampled" }
    end
    return { disposition = "collecting" }
end

return details
