local collection = {}
local source = require("live_galaxy.lua.live_galaxy_ship_source")
local MAX_INTEGER = 9007199254740991

local function integer(value)
    return type(value) == "number" and value >= 0 and value <= MAX_INTEGER and value % 1 == 0
end
local function token(value, limit)
    return type(value) == "string" and #value > 0 and #value <= limit
        and value:match("^[%w_:%-]+$") ~= nil
end
local function ordered(left, right)
    if #left ~= #right then return #left < #right end
    return left < right
end
local function same_core(left, right)
    for _, key in ipairs({ "identity", "owner", "type", "class", "location" }) do
        if left[key] ~= right[key] then return false end
    end
    return true
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
        stage = "census", work = 0, attempts = 1 }, { __index = collection })
end

function collection:discard(carrier, reason)
    if self.reserved then carrier:fail_section(reason) end
    self.reserved, self.buffer, self.identities, self.pending = false, nil, nil, nil
    self.cores = nil
    self.faction_buffer, self.factions, self.boundary, self.incarnation = nil, nil, nil, nil
    self.attempts = self.attempts + 1
    self.stage = self.attempts <= self.limits.max_attempts and "census" or "halted"
    return { disposition = reason }
end

function collection:tick(context, carrier, status)
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
    if self.last_step and now <= self.last_step then return { disposition = "clock_unavailable" } end
    self.started = self.started or now
    if now - self.started > self.limits.max_age_millis or self.work >= self.limits.max_work then
        return self:discard(carrier, "collection_overflow")
    end
    self.last_step, self.work = now, self.work + 1
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
        local found, seen = false, {}
        if type(self.factions) ~= "table" then return self:discard(carrier, "invalid_fact") end
        for _, faction in ipairs(self.factions) do
            if not token(faction, 64) or seen[faction] then return self:discard(carrier, "invalid_fact") end
            seen[faction], found = true, found or faction == self.faction
        end
        self.factions = nil
        if not found then return self:discard(carrier, "selection_unavailable") end
        self.stage = "count"
    elseif stage == "count" then
        self.count = api:count_ships(self.faction)
        if not integer(self.count) or self.count > self.limits.max_records
            or self.count > math.floor(self.limits.max_allocation_bytes / 8)
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
        self.identities = {}
        local seen = {}
        for i = 0, self.count - 1 do
            local id = source.identity(self.buffer[i])
            if id == nil or seen[id] then return self:discard(carrier, "identity_invalid") end
            seen[id], self.identities[i + 1] = true, id
        end
        table.sort(self.identities, ordered)
        self.buffer, self.stage = nil, "reserve"
    elseif stage == "reserve" then
        local begin, err = self.clock:begin_evidence()
        if begin == nil then return self:discard(carrier, err) end
        begin.section_key, begin.expected_records = "ship_core", self.count
        begin.coverage, begin.consistency, begin.stable_identity = "partial", "observed_count_fill_only", true
        begin.source_epoch_status = context.source_epoch_status or "unknown"
        begin.source_boundary = boundary
        local code = carrier:begin_section(begin)
        if code == -21 then return { disposition = "producer_busy" } end
        if code ~= 0 then return self:discard(carrier, "reservation_failed") end
        self.reserved, self.index = true, 1
        self.stage = self.count == 0 and "complete" or "core"
    elseif stage == "core" then
        local core = api:read_core(self.identities[self.index])
        if type(core) ~= "table" or core.identity ~= self.identities[self.index]
            or core.owner ~= self.faction or not token(core.type, 128)
            or not token(core.class, 64) or not token(core.location, 128) then
            return self:discard(carrier, "invalid_fact")
        end
        self.pending = { identity = core.identity, owner = core.owner, type = core.type,
            class = core.class, location = core.location }
        self.stage = "validate"
    elseif stage == "validate" then
        local current = api:read_core(self.pending.identity)
        if type(current) ~= "table" or not same_core(self.pending, current) then
            return self:discard(carrier, "core_changed")
        end
        self.stage = "record"
    elseif stage == "record" then
        self.pending.profile, self.pending.source_scope = "ship_core", self.source_scope
        local code = carrier:push_record(self.pending)
        if code == -21 then return { disposition = "producer_busy" } end
        if code ~= 0 then return self:discard(carrier, "fact_rejected") end
        self.cores = self.cores or {}
        self.cores[self.pending.identity] = { identity = self.pending.identity, owner = self.pending.owner,
            type = self.pending.type, class = self.pending.class, location = self.pending.location }
        self.pending, self.index = nil, self.index + 1
        self.stage = self.index > self.count and "complete" or "core"
    elseif stage == "complete" then
        local finish, err = self.clock:finish_evidence()
        if finish == nil then return self:discard(carrier, err) end
        finish.coverage, finish.consistency, finish.stable_identity = "partial", "observed_count_fill_only", true
        if carrier:finish_section(finish) ~= 0 then return self:discard(carrier, "finish_rejected") end
        self.reserved, self.stage = false, "done"
        return { disposition = "sampled", source_transition_accepted = true }
    elseif stage == "done" then
        self.cores = nil
        self.stage, self.started, self.last_step, self.work, self.attempts = "census", nil, nil, 0, 1
    end
    return { disposition = "collecting" }
end

return collection
