local capture = {}
local function unsigned(value, maximum)
    return type(value) == "number" and value >= 0 and value <= maximum and value % 1 == 0
end
local function allocation(self, count, size)
    return unsigned(count, self.limit) and unsigned(size, self.allocation)
        and size > 0 and count <= math.floor(self.allocation / size)
end
-- One native getter or one checked allocation per callback; strings are copied
-- by the source before a callback can yield. No individual personnel are read.
function capture.step(self)
    local api, id, stage = self.api, self.group.members[self.index], self.stage
    if stage == "crew_capacity" then
        local capacity = api:crew_capacity(id)
        if not unsigned(capacity, 4294967295) then return nil, "crew_unknown" end
        self.pending = { capacity_people = capacity, capacity_outcome = capacity == 0 and "zero" or "value",
            includepilot = true }
        self.stage = "crew_count"
    elseif stage == "crew_count" then
        self.count = api:crew_count()
        if not allocation(self, self.count, api:crew_size()) then return nil, "collection_overflow" end
        self.stage = "crew_allocate"
    elseif stage == "crew_allocate" then
        self.buffer, self.stage = api:crew_allocate(self.count), "crew_fill"
    elseif stage == "crew_fill" then
        local rows = api:crew_fill(id, self.buffer, self.count)
        self.buffer = nil
        if type(rows) ~= "table" or #rows > self.count then return nil, "enumeration_incomplete" end
        local seen = {}
        for _, role in ipairs(rows) do
            if type(role.id) ~= "string" or not role.id:match("^[%w_:%-]+$") or seen[role.id]
                or not unsigned(role.amount_people, 4294967295)
                or not unsigned(role.reported_numtiers, self.limit) or type(role.canhire) ~= "boolean" then
                return nil, "invalid_fact"
            end
            seen[role.id] = true
        end
        table.sort(rows, function(a, b) return a.id < b.id end)
        self.pending.roles, self.role_index = rows, 1
        self.pending.roles_outcome = #rows == 0 and "empty" or "value"
        self.stage = #rows == 0 and "record" or "crew_tier_allocate"
    elseif stage == "crew_tier_allocate" then
        local count = self.pending.roles[self.role_index].reported_numtiers
        if not allocation(self, count, api:crew_tier_size()) then return nil, "collection_overflow" end
        self.buffer, self.stage = api:crew_tier_allocate(count), "crew_tier_fill"
    elseif stage == "crew_tier_fill" then
        local role = self.pending.roles[self.role_index]
        local tiers = api:crew_tier_fill(id, role.id, self.buffer, role.reported_numtiers)
        self.buffer = nil
        if type(tiers) ~= "table" or #tiers > role.reported_numtiers then return nil, "enumeration_incomplete" end
        for _, tier in ipairs(tiers) do
            if type(tier.name) ~= "string" or #tier.name == 0 or #tier.name > 128
                or tier.name:find("[|\n\r]") or type(tier.skill_lower_threshold) ~= "number"
                or tier.skill_lower_threshold < -2147483648 or tier.skill_lower_threshold > 2147483647
                or tier.skill_lower_threshold % 1 ~= 0 or not unsigned(tier.amount_people, 4294967295) then
                return nil, "invalid_fact"
            end
        end
        role.tiers, self.role_index = tiers, self.role_index + 1
        self.stage = self.role_index > #self.pending.roles and "record" or "crew_tier_allocate"
    else return nil, "invalid_capture_stage" end
    return true
end
return capture
