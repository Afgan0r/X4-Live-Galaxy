local details = {}
local source = require("live_galaxy.lua.live_galaxy_ship_source")
local crew_capture = require("live_galaxy.lua.live_galaxy_ship_crew_capture")
local loadout_capture = require("live_galaxy.lua.live_galaxy_ship_loadout_capture")
local order = require("live_galaxy.lua.live_galaxy_ship_order")
local MAX_INTEGER = 9007199254740991
local function integer(v, maximum)
    return type(v) == "number" and v >= 0 and v <= maximum and v % 1 == 0
end
local function token(v)
    return type(v) == "string" and #v > 0 and #v <= 128 and v:match("^[%w_:%-]+$") ~= nil
end
local function start_stage(kind)
    if kind == "ship_crew" then return "crew_capacity" end
    if kind == "ship_loadout" then return "loadout_init" end
    return "wares"
end

function details.new(options, clock)
    local limit = options.max_inner
    if not integer(limit, MAX_INTEGER) or limit == 0
        or not integer(options.max_allocation_bytes, MAX_INTEGER)
        or options.max_allocation_bytes == 0 then return nil, "invalid_limits" end
    local expected_cores = options.expected_cores
    if expected_cores == nil and type(options.expected_core) == "table" then
        expected_cores = { [options.expected_core.identity] = options.expected_core }
    end
    return setmetatable({ api = options.ship_api or source.runtime(), clock = clock,
        limit = limit, allocation = options.max_allocation_bytes, stage = "reserve",
        group = options.group, scope = options.source_scope, expected_cores = expected_cores,
        work_budget = options.work_budget,
        capture_only = options.capture_only,
        kind = options.group.key:match("^(ship_%w+):") }, { __index = details })
end

function details:deliver()
    if self.stage ~= "captured" then return false end
    self.capture_only = false
    self.finish, self.index, self.pending, self.stage = self.captured_finish, 1, self.records[1], "reserve_delivery"
    return true
end

function details:fail(carrier, reason, condition, field, observed_type)
    local rejection = { stage = self.stage, condition = condition or reason,
        field = field, observed_type = observed_type, ordinal = self.index }
    if not self.capture_only then carrier:fail_section(reason) end
    self.stage, self.pending, self.buffer = "failed", nil, nil
    return { disposition = reason, rejection = rejection }
end

function details:step(context, carrier, status)
    local group = self.group
    if self.stage == "done" then return { disposition = "producer_busy" } end
    if self.stage == "failed" then return { disposition = "failed" } end
    if status.selection ~= group.key and not (self.capture_only
        and (status.selection == "ship_core" or status.selection:match("^ship_core:[%w_%-]+$"))) then
        return self:fail(carrier, "restart_required")
    end
    if self.stage == "reserve" then
        local begin, err = self.clock:begin_evidence()
        if not begin then return self:fail(carrier, err) end
        begin.section_key, begin.expected_records = group.key, #group.members
        begin.coverage, begin.consistency, begin.stable_identity = "partial", "observed_count_fill_only", true
        begin.source_epoch_status, begin.source_boundary = "unknown", context.source_boundary or "runtime_start"
        self.begin, self.records, self.index, self.capture_start = begin, {}, 1, begin.capture_start_millis
        self.stage = start_stage(self.kind)
    elseif self.stage:match("^crew_") then
        local ok, err, rejection = crew_capture.step(self)
        if not ok then return self:fail(carrier, err, rejection and rejection.condition,
            rejection and rejection.field, rejection and rejection.observed_type) end
    elseif self.stage:match("^loadout_") then
        local ok, err, rejection = loadout_capture.step(self)
        if not ok then return self:fail(carrier, err, rejection and rejection.condition,
            rejection and rejection.field, rejection and rejection.observed_type) end
    elseif self.stage == "wares" then
        local raw = self.api:cargo_wares(group.members[self.index])
        if type(raw) ~= "table" then return self:fail(carrier, "cargo_unknown", "result_shape", "wares", type(raw)) end
        if getmetatable(raw) ~= nil then return self:fail(carrier, "cargo_unknown", "unexpected_metatable", "wares", type(raw)) end
        -- Copy the returned owned values into the frozen selection record.
        self.raw, self.pending, self.stage = raw, { wares = {} }, "ware_copy"
    elseif self.stage == "ware_copy" then
        while true do
            if self.work_budget then self.work_budget:step() end
            local ware, amount = next(self.raw, self.ware_key)
            if ware == nil then
                self.raw, self.ware_key = nil, nil
                self.sort, self.stage = order.new(self.pending.wares, function(a, b) return a.ware < b.ware end), "ware_order"
                break
            end
            if #self.pending.wares >= self.limit then return self:fail(carrier, "collection_overflow", "record_limit", "wares") end
            if not token(ware) then return self:fail(carrier, "collection_overflow", "token_invalid", "ware", type(ware)) end
            if not integer(amount, MAX_INTEGER) then return self:fail(carrier, "collection_overflow", "integer_invalid", "amount", type(amount)) end
            self.pending.wares[#self.pending.wares + 1], self.ware_key = { ware = ware, amount_items = amount }, ware
        end
    elseif self.stage == "ware_order" then
        local done, err = self.sort()
        if err then return self:fail(carrier, "invalid_fact") end
        if done then self.sort, self.stage = nil, "count" end
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
        self.rows, self.row_index, self.seen, self.stage = rows, 1, {}, "storage_validate"
    elseif self.stage == "storage_validate" then
        for i = self.row_index, #self.rows do
            if self.work_budget then self.work_budget:step() end
            local row = self.rows[i]
            if not token(row.transport) then return self:fail(carrier, "invalid_fact", "token_invalid", "transport", type(row.transport)) end
            if self.seen[row.transport] then return self:fail(carrier, "invalid_fact", "duplicate", "transport") end
            if not integer(row.capacity_cubic_metres, 4294967295) then
                return self:fail(carrier, "invalid_fact", "integer_invalid", "capacity_cubic_metres", type(row.capacity_cubic_metres))
            end
            if not integer(row.occupied_cubic_metres, row.capacity_cubic_metres) then
                return self:fail(carrier, "invalid_fact", "integer_invalid", "occupied_cubic_metres", type(row.occupied_cubic_metres))
            end
            self.seen[row.transport], self.row_index = true, i + 1
        end
        if self.row_index > #self.rows then
            self.sort, self.stage = order.new(self.rows, function(a, b) return a.transport < b.transport end), "storage_order"
        end
    elseif self.stage == "storage_order" then
        local done, err = self.sort()
        if err then return self:fail(carrier, "invalid_fact") end
        if done then
            self.pending.storage, self.rows, self.seen, self.sort, self.stage = self.rows, nil, nil, nil, "record"
        end
    elseif self.stage == "record" then
        local finish, err = self.clock:finish_evidence()
        if not finish then return self:fail(carrier, err) end
        local record = self.pending
        record.profile, record.source_scope = self.kind, self.scope
        record.identity, record.owner = group.members[self.index], group.owner
        record.core_revision, record.member_revision = group.core_revision, group.core_revision
        record.policy_version = 2
        record.capture_start_millis, record.capture_end_millis = self.capture_start, finish.capture_end_millis
        record.source_evidence = "x4-9.00-steam-23660954-ship-detail-source-v1"
        record.consistency, record.consistency_reason = "consistent", "none"
        if self.kind == "ship_cargo" then
            record.wares_outcome = #record.wares == 0 and "empty" or "value"
            record.storage_outcome = #record.storage == 0 and "empty" or "value"
        end
        self.records[self.index] = record
        self.index, self.pending = self.index + 1, nil
        if self.index > #group.members and self.expected_cores then
            self.index, self.stage = 1, "revalidate"
        elseif self.index > #group.members then
            self.stage = "capture_finish"
        else
            self.stage = start_stage(self.kind)
        end
    elseif self.stage == "revalidate" then
        local identity = group.members[self.index]
        local expected = self.expected_cores and self.expected_cores[identity]
        if type(expected) ~= "table" then return self:fail(carrier, "core_changed", "missing_parent", "core") end
        local current, source_rejection = self.api:read_core(identity)
        if type(current) ~= "table" then
            self.records[self.index].consistency = "possibly_stale"
            self.records[self.index].consistency_reason = "missing"
        else
            local reason
            if current.identity ~= expected.identity then reason = "missing"
            elseif current.owner ~= expected.owner then reason = "owner_changed"
            elseif current.location ~= expected.location then reason = "location_changed"
            elseif current.type ~= expected.type or current.class ~= expected.class then reason = "core_changed" end
            if reason then
                self.records[self.index].consistency = "possibly_stale"
                self.records[self.index].consistency_reason = reason
            end
        end
        self.index = self.index + 1
        if self.index > #group.members then self.stage = "capture_finish" end
    elseif self.stage == "capture_finish" then
        local finish, err = self.clock:finish_evidence()
        if not finish then return self:fail(carrier, err) end
        finish.coverage, finish.consistency, finish.stable_identity = "partial", "observed_count_fill_only", true
        if self.capture_only then
            self.captured_finish, self.stage = finish, "captured"
            return { disposition = "captured" }
        end
        self.finish, self.index, self.pending, self.stage = finish, 1, self.records[1], "reserve_delivery"
        if self.work_budget then
            local ok, reason = self.work_budget:after(carrier)
            if not ok then return self:fail(carrier, reason) end
        end
    elseif self.stage == "reserve_delivery" then
        local code = carrier:begin_section(self.begin)
        if code == -21 then return { disposition = "producer_busy" } end
        if code ~= 0 then return self:fail(carrier, "reservation_failed") end
        self.reserved, self.stage = true, "deliver"
    elseif self.stage == "deliver" then
        local code = carrier:push_record(self.pending)
        if code == -21 then return { disposition = "producer_busy" } end
        if code ~= 0 then return self:fail(carrier, "fact_rejected") end
        self.index = self.index + 1
        self.pending = self.records[self.index]
        self.stage = self.pending and "deliver" or "complete"
    elseif self.stage == "complete" then
        if carrier:finish_section(self.finish) ~= 0 then return self:fail(carrier, "finish_rejected") end
        self.stage, self.reserved = "done", false
        return { disposition = "sampled" }
    end
    if self.stage == "captured" then return { disposition = "captured" } end
    return { disposition = "collecting" }
end

function details:tick(context, carrier, status)
    while true do
        if self.work_budget then self.work_budget:step() end
        local result = self:step(context, carrier, status)
        if result.disposition ~= "collecting" then return result end
        if self.work_budget and self.work_budget:should_yield(carrier) then return result end
    end
end

return details
