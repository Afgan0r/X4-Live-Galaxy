local capture = {}
local order = require("live_galaxy.lua.live_galaxy_ship_order")
local function unsigned(value, maximum)
    return type(value) == "number" and value >= 0 and value <= maximum and value % 1 == 0
end
local function allocation(self, count, size)
    return unsigned(count, self.limit) and unsigned(size, self.allocation)
        and size > 0 and count <= math.floor(self.allocation / size)
end
-- Synchronous aggregate capture; source-owned borrowed strings are copied by
-- the source seam. No individual personnel are read.
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
        self.rows, self.row_index, self.seen, self.stage = rows, 1, {}, "crew_validate"
    elseif stage == "crew_validate" then
        for i = self.row_index, #self.rows do
            if self.work_budget then self.work_budget:step() end
            local role = self.rows[i]
            local field, condition
            if type(role.id) ~= "string" or not role.id:match("^[%w_:%-]+$") then field, condition = "id", "token_invalid"
            elseif self.seen[role.id] then field, condition = "id", "duplicate"
            elseif not unsigned(role.amount_people, 4294967295) then field, condition = "amount_people", "integer_invalid"
            elseif not unsigned(role.reported_numtiers, self.limit) then field, condition = "reported_numtiers", "integer_invalid"
            elseif type(role.canhire) ~= "boolean" then field, condition = "canhire", "result_shape" end
            if field then
                return nil, "invalid_fact", { condition = condition, field = field, observed_type = type(role[field]) }
            end
            self.seen[role.id], self.row_index = true, i + 1
        end
        if self.row_index > #self.rows then
            self.sort, self.stage = order.new(self.rows, function(a, b) return a.id < b.id end), "crew_order"
        end
    elseif stage == "crew_order" then
        local done, err = self.sort()
        if err then return nil, "invalid_fact" end
        if done then
            self.pending.roles, self.role_index = self.rows, 1
            self.pending.roles_outcome = #self.rows == 0 and "empty" or "value"
            self.stage = #self.rows == 0 and "record" or "crew_tier_allocate"
            self.rows, self.seen, self.sort = nil, nil, nil
        end
    elseif stage == "crew_tier_allocate" then
        local count = self.pending.roles[self.role_index].reported_numtiers
        if not allocation(self, count, api:crew_tier_size()) then return nil, "collection_overflow" end
        self.buffer, self.stage = api:crew_tier_allocate(count), "crew_tier_fill"
    elseif stage == "crew_tier_fill" then
        local role = self.pending.roles[self.role_index]
        local tiers = api:crew_tier_fill(id, role.id, self.buffer, role.reported_numtiers)
        self.buffer = nil
        if type(tiers) ~= "table" or #tiers > role.reported_numtiers then return nil, "enumeration_incomplete" end
        self.rows, self.row_index, self.stage = tiers, 1, "crew_tier_validate"
    elseif stage == "crew_tier_validate" then
        for i = self.row_index, #self.rows do
            if self.work_budget then self.work_budget:step() end
            local tier = self.rows[i]
            local field, condition
            if type(tier.name) ~= "string" or #tier.name == 0 or #tier.name > 128 or tier.name:find("[|\n\r]") then
                field, condition = "name", "string_invalid"
            elseif type(tier.skill_lower_threshold) ~= "number" or tier.skill_lower_threshold < -2147483648
                or tier.skill_lower_threshold > 2147483647 or tier.skill_lower_threshold % 1 ~= 0 then
                field, condition = "skill_lower_threshold", "integer_invalid"
            elseif not unsigned(tier.amount_people, 4294967295) then field, condition = "amount_people", "integer_invalid" end
            if field then
                return nil, "invalid_fact", { condition = condition, field = field, observed_type = type(tier[field]) }
            end
            self.row_index = i + 1
        end
        if self.row_index > #self.rows then
            self.pending.roles[self.role_index].tiers, self.role_index = self.rows, self.role_index + 1
            self.rows = nil
            self.stage = self.role_index > #self.pending.roles and "record" or "crew_tier_allocate"
        end
    else return nil, "invalid_capture_stage" end
    return true
end
return capture
