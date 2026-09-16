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
    return setmetatable({ api = options.ship_api or source.runtime(), clock = clock,
        limit = limit, allocation = options.max_allocation_bytes, stage = "reserve",
        group = options.group, scope = options.source_scope, expected_core = options.expected_core,
        kind = options.group.key:match("^(ship_%w+):g") }, { __index = details })
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
        self.stage = start_stage(self.kind)
    elseif self.stage:match("^crew_") then
        local ok, err = crew_capture.step(self)
        if not ok then return self:fail(carrier, err) end
    elseif self.stage:match("^loadout_") then
        local ok, err = loadout_capture.step(self)
        if not ok then return self:fail(carrier, err) end
    elseif self.stage == "wares" then
        local raw = self.api:cargo_wares(group.members[self.index])
        if type(raw) ~= "table" or getmetatable(raw) ~= nil then return self:fail(carrier, "cargo_unknown") end
        -- This table is owned Lua data. Unlike borrowed ffi strings it can be
        -- validated/copied incrementally after the indivisible getter.
        self.raw, self.pending, self.stage = raw, { wares = {} }, "ware_copy"
    elseif self.stage == "ware_copy" then
        for _ = 1, 32 do
            local ware, amount = next(self.raw, self.ware_key)
            if ware == nil then
                self.raw, self.ware_key = nil, nil
                self.sort, self.stage = order.new(self.pending.wares, function(a, b) return a.ware < b.ware end), "ware_order"
                break
            end
            if #self.pending.wares >= self.limit or not token(ware) or not integer(amount, MAX_INTEGER) then
                return self:fail(carrier, "collection_overflow")
            end
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
        for i = self.row_index, math.min(self.row_index + 31, #self.rows) do
            local row = self.rows[i]
            if not token(row.transport) or self.seen[row.transport]
                or not integer(row.capacity_cubic_metres, 4294967295)
                or not integer(row.occupied_cubic_metres, row.capacity_cubic_metres) then
                return self:fail(carrier, "invalid_fact")
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
        if self.kind == "ship_cargo" then
            record.wares_outcome = #record.wares == 0 and "empty" or "value"
            record.storage_outcome = #record.storage == 0 and "empty" or "value"
        end
        local code = carrier:push_record(record)
        if code == -21 then return { disposition = "producer_busy" } end
        if code ~= 0 then return self:fail(carrier, "fact_rejected") end
        self.index, self.pending = self.index + 1, nil
        self.stage = self.index > #group.members and "complete" or start_stage(self.kind)
    elseif self.stage == "revalidate" then
        local current = self.api:read_core(self.expected_core.identity)
        if type(current) ~= "table" then return self:fail(carrier, "core_changed") end
        for _, key in ipairs({ "identity", "owner", "type", "class", "location" }) do
            if current[key] ~= self.expected_core[key] then return self:fail(carrier, "core_changed") end
        end
        self.validated, self.stage = true, "complete"
    elseif self.stage == "complete" then
        if self.expected_core and not self.validated then
            self.stage = "revalidate"
            return { disposition = "collecting" }
        end
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
