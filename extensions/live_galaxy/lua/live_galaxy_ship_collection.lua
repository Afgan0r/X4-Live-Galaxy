local collection = {}
local source = require("live_galaxy.lua.live_galaxy_ship_source")
local order = require("live_galaxy.lua.live_galaxy_ship_order")
local MAX_INTEGER = 9007199254740991

local function integer(value)
    return type(value) == "number" and value >= 0 and value <= MAX_INTEGER and value % 1 == 0
end
local function token(value, limit)
    return type(value) == "string" and #value > 0 and #value <= limit
        and value:match("^[%w_:%-]+$") ~= nil
end
function collection.new(options, clock)
    if not token(options.faction_id, 64) or options.faction_id == "player"
        or options.faction_id == "xenon" or options.faction_id == "khaak" then
        return nil, "selection_unavailable"
    end
    local limits = options.ship_limits
    if type(limits) ~= "table" then return nil, "limits_unavailable" end
    local copied = {}
    for _, key in ipairs({ "max_records", "max_allocation_bytes", "max_work",
        "max_attempts", "max_age_millis", "faction_pointer_bytes" }) do
        if not integer(limits[key]) or limits[key] == 0 then return nil, "invalid_limits" end
        copied[key] = limits[key]
    end
    local api = options.ship_api or source.runtime()
    return setmetatable({ api = api, limits = copied, clock = clock, faction = options.faction_id,
        source_scope = options.source_scope or "x4:faction:" .. options.faction_id .. ":ships",
        stage = "census", work = 0, attempts = 1, work_budget = options.work_budget,
        capture_only = options.capture_only }, { __index = collection })
end

function collection:deliver()
    if self.stage ~= "captured" then return false end
    self.stage = "reserve"
    return true
end

function collection:discard(carrier, reason, condition, field, observed_type)
    local rejection = { stage = self.stage, condition = condition or reason,
        field = field, observed_type = observed_type, attempt = self.attempts,
        ordinal = self.index or self.copy_index }
    if self.reserved then carrier:fail_section(reason) end
    self.reserved, self.buffer, self.identities, self.pending = false, nil, nil, nil
    self.cores, self.records, self.begin, self.finish = nil, nil, nil, nil
    self.seen, self.copy_index, self.sort = nil, nil, nil
    self.faction_buffer, self.factions, self.boundary, self.incarnation = nil, nil, nil, nil
    self.attempts = self.attempts + 1
    self.stage = self.attempts <= self.limits.max_attempts and "census" or "halted"
    return { disposition = reason, rejection = rejection }
end

function collection:step(context, carrier, status)
    if self.stage == "halted" then return { disposition = "retry_exhausted" } end
    if status.selection ~= "ship_core" then return self:discard(carrier, "restart_required") end
    local boundary = context.source_boundary or "runtime_start"
    local incarnation = status.producer_incarnation
    if self.boundary and (self.boundary ~= boundary or self.incarnation ~= incarnation) then
        return self:discard(carrier, "source_boundary_changed")
    end
    self.boundary, self.incarnation = boundary, incarnation
    local now = tonumber(status.monotonic_millis)
    if not integer(now) then return self:discard(carrier, "clock_unavailable") end
    self.started = self.started or now
    if now - self.started > self.limits.max_age_millis or self.work >= self.limits.max_work then
        return self:discard(carrier, "collection_overflow")
    end
    self.last_step, self.work = now, self.work + 1
    if self.work_budget then self.work_budget:step() end
    local api, stage = self.api, self.stage
    if stage == "census" then
        if api.list_factions then
            self.factions = api:list_factions()
            if type(self.factions) ~= "table" or #self.factions
                > math.floor(self.limits.max_allocation_bytes / self.limits.faction_pointer_bytes) then
                return self:discard(carrier, "collection_overflow")
            end
            self.stage = "select"
        else
            self.faction_count = api:count_factions()
            if not integer(self.faction_count) or self.faction_count == 0
                or self.faction_count > math.floor(self.limits.max_allocation_bytes
                    / self.limits.faction_pointer_bytes) then
                return self:discard(carrier, "collection_overflow")
            end
            self.stage = "census_allocate"
        end
    elseif stage == "census_allocate" then
        self.faction_buffer = api:new_faction_buffer(self.faction_count)
        self.stage = "census_fill"
    elseif stage == "census_fill" then
        if api:fill_factions(self.faction_buffer, self.faction_count) ~= self.faction_count then
            return self:discard(carrier, "enumeration_incomplete")
        end
        -- Copy pointer-backed strings synchronously before yielding.
        self.factions = {}
        for i = 0, self.faction_count - 1 do
            self.factions[i + 1] = api:faction_string(self.faction_buffer[i])
        end
        self.faction_buffer, self.stage = nil, "select"
    elseif stage == "select" then
        if type(self.factions) ~= "table" then return self:discard(carrier, "invalid_fact") end
        self.seen, self.copy_index = self.seen or {}, self.copy_index or 1
        for i = self.copy_index, #self.factions do
            if self.work_budget then self.work_budget:step() end
            local faction = self.factions[i]
            if not token(faction, 64) then
                return self:discard(carrier, "invalid_fact", "token_invalid", "faction", type(faction))
            end
            if self.seen[faction] then return self:discard(carrier, "invalid_fact", "duplicate", "faction") end
            self.seen[faction], self.found, self.copy_index = true, self.found or faction == self.faction, i + 1
        end
        if self.copy_index > #self.factions then
            if not self.found then return self:discard(carrier, "selection_unavailable") end
            self.factions, self.seen, self.copy_index, self.found, self.stage = nil, nil, nil, nil, "count"
        end
    elseif stage == "count" then
        self.count = api:count_ships(self.faction)
        if not integer(self.count) or self.count > math.floor(self.limits.max_allocation_bytes / 8)
            or self.count * 3 + 4 > self.limits.max_work - self.work then
            return self:discard(carrier, "collection_overflow")
        end
        if self.count == 0 then return self:discard(carrier, "empty_unproven") end
        self.stage = "allocate"
    elseif stage == "allocate" then
        self.buffer = self.count > 0 and api:new_buffer(self.count) or {}
        self.stage = "fill"
    elseif stage == "fill" then
        if self.count > 0 and api:fill_ships(self.buffer, self.count, self.faction) ~= self.count then
            return self:discard(carrier, "enumeration_incomplete")
        end
        self.stage = "identities"
    elseif stage == "identities" then
        self.identities, self.seen, self.copy_index = self.identities or {}, self.seen or {}, self.copy_index or 0
        for i = self.copy_index, self.count - 1 do
            if self.work_budget then self.work_budget:step() end
            local id = source.identity(self.buffer[i])
            if id == nil then return self:discard(carrier, "identity_invalid", "identity_invalid", "identity", type(self.buffer[i])) end
            if self.seen[id] then return self:discard(carrier, "identity_invalid", "duplicate", "identity") end
            self.seen[id], self.identities[i + 1] = true, id
            self.copy_index = i + 1
        end
        if self.copy_index == self.count then
            self.sort = order.new(self.identities, function(a, b) return a < b end)
            self.buffer, self.seen, self.copy_index, self.stage = nil, nil, nil, "order"
        end
    elseif stage == "order" then
        local done, err = self.sort()
        if err then return self:discard(carrier, "identity_invalid") end
        if done then self.sort, self.stage = nil, "capture_begin" end
    elseif stage == "capture_begin" then
        local begin, err = self.clock:begin_evidence()
        if begin == nil then return self:discard(carrier, err) end
        begin.section_key, begin.expected_records = "ship_core", self.count
        begin.coverage, begin.consistency, begin.stable_identity = "partial", "observed_count_fill_only", true
        begin.source_epoch_status = context.source_epoch_status or "unknown"
        begin.source_boundary = boundary
        self.begin, self.index, self.cores, self.records, self.stage = begin, 1, {}, {}, "core"
    elseif stage == "reserve" then
        local code = carrier:begin_section(self.begin)
        if code == -21 then return { disposition = "producer_busy" } end
        if code ~= 0 then return self:discard(carrier, "reservation_failed") end
        self.reserved, self.index = true, 1
        self.stage = "record"
    elseif stage == "core" then
        local core, source_rejection = api:read_core(self.identities[self.index])
        if type(core) ~= "table" then
            return self:discard(carrier, "invalid_fact", source_rejection and source_rejection.condition or "result_shape",
                source_rejection and source_rejection.field or "core",
                source_rejection and source_rejection.observed_type or type(core))
        end
        for _, key in ipairs({ "identity", "owner", "type", "class", "location" }) do
            local valid = key == "identity" and core[key] == self.identities[self.index]
                or key == "owner" and token(core[key], 64)
                or key ~= "identity" and key ~= "owner" and token(core[key], key == "class" and 64 or 128)
            if not valid then
                return self:discard(carrier, "invalid_fact",
                    key == "identity" and "value_mismatch" or "token_invalid", key, type(core[key]))
            end
        end
        self.pending = { identity = core.identity, owner = self.faction, type = core.type,
            class = core.class, location = core.location, consistency = "consistent",
            consistency_reason = "none" }
        if core.owner ~= self.faction then
            self.pending.consistency, self.pending.consistency_reason = "possibly_stale", "owner_changed"
        end
        self.stage = "validate"
    elseif stage == "validate" then
        local current = api:read_core(self.pending.identity)
        if type(current) ~= "table" then
            self.pending.consistency, self.pending.consistency_reason = "possibly_stale", "missing"
        else
            local reason
            if current.identity ~= self.pending.identity then reason = "missing"
            elseif current.owner ~= self.pending.owner then reason = "owner_changed"
            elseif current.location ~= self.pending.location then reason = "location_changed"
            elseif current.type ~= self.pending.type or current.class ~= self.pending.class then reason = "core_changed" end
            if reason then
                self.pending.consistency, self.pending.consistency_reason = "possibly_stale", reason
            end
        end
        self.cores[self.pending.identity] = self.pending
        self.records[self.index] = self.pending
        self.pending, self.index = nil, self.index + 1
        self.stage = self.index > self.count and "capture_finish" or "core"
    elseif stage == "capture_finish" then
        local finish, err = self.clock:finish_evidence()
        if not finish then return self:discard(carrier, err) end
        finish.coverage, finish.consistency, finish.stable_identity = "partial", "observed_count_fill_only", true
        self.finish, self.index, self.stage = finish, 1, self.capture_only and "captured" or "reserve"
        if self.capture_only then return { disposition = "captured" } end
        if self.work_budget then
            local ok, reason = self.work_budget:after(carrier)
            if not ok then return self:discard(carrier, reason) end
        end
    elseif stage == "record" then
        self.pending = self.records[self.index]
        self.pending.profile, self.pending.source_scope = "ship_core", self.source_scope
        local code = carrier:push_record(self.pending)
        if code == -21 then return { disposition = "producer_busy" } end
        if code ~= 0 then return self:discard(carrier, "fact_rejected") end
        self.pending, self.index = nil, self.index + 1
        self.stage = self.index > self.count and "complete" or "record"
    elseif stage == "complete" then
        if carrier:finish_section(self.finish) ~= 0 then return self:discard(carrier, "finish_rejected") end
        self.reserved, self.stage = false, "done"
        return { disposition = "sampled", source_transition_accepted = true }
    elseif stage == "done" then
        self.cores = nil
        self.stage, self.started, self.last_step, self.work, self.attempts = "census", nil, nil, 0, 1
    end
    if stage == "captured" then return { disposition = "captured" } end
    return { disposition = "collecting" }
end

function collection:tick(context, carrier, status)
    local now = tonumber(status.monotonic_millis)
    if not integer(now) or self.last_callback and now <= self.last_callback then
        return { disposition = "clock_unavailable", rejection = { stage = self.stage,
            condition = "monotonic_not_advancing", field = "monotonic_millis", attempt = self.attempts } }
    end
    self.last_callback = now
    while true do
        local result = self:step(context, carrier, status)
        if result.disposition ~= "collecting" then return result end
    end
end

return collection
