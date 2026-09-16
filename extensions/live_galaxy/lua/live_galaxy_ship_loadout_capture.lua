local capture = {}
local physical_kinds = { "engine", "shield", "turret", "weapon" }
local virtual_kinds = { "engine", "shield", "thruster", "turret", "weapon" }
local arrays = { "software", "missiles", "units" }
local function integer(v, max) return type(v) == "number" and v >= 0 and v <= max and v % 1 == 0 end
local function string(v) return type(v) == "string" and #v <= 128 and not v:find("[|\n\r]") end
local function next_kind(self, virtual)
    self.kind_index, self.slot = self.kind_index + 1, 1
    local kinds = virtual and virtual_kinds or physical_kinds
    if self.kind_index <= #kinds then self.stage = virtual and "loadout_virtual_count" or "loadout_physical_count"
    elseif not virtual then self.kind_index, self.stage = 1, "loadout_virtual_count"
    else self.array_index, self.stage = 1, "loadout_array_count" end
end
local function arrays_step(self, id)
    local name, api = arrays[self.array_index], self.api
    if self.stage == "loadout_array_count" then
        self.count = api[name .. "_count"](api, id)
        local size = api[name .. "_size"](api)
        if not integer(self.count, self.limit) or not integer(size, self.allocation)
            or size == 0 or self.count > math.floor(self.allocation / size) then return nil, "collection_overflow" end
        self.stage = "loadout_array_allocate"
    elseif self.stage == "loadout_array_allocate" then
        self.buffer, self.stage = api[name .. "_allocate"](api, self.count), "loadout_array_fill"
    else
        local rows = api[name .. "_fill"](api, id, self.buffer, self.count)
        self.buffer = nil
        if type(rows) ~= "table" or #rows > self.count then return nil, "enumeration_incomplete" end
        self.pending[name], self.pending[name .. "_outcome"] = rows, #rows == 0 and "empty" or "value"
        self.array_index = self.array_index + 1
        self.stage = self.array_index > #arrays and "record" or "loadout_array_count"
    end
    return true
end
function capture.step(self)
    local api, id, stage = self.api, self.group.members[self.index], self.stage
    if stage == "loadout_init" then
        self.pending = { physical = {}, virtual_slots = {}, onlydrones = false }
        self.kind_index, self.slot, self.stage = 1, 1, "loadout_physical_count"
    elseif stage == "loadout_physical_count" then
        self.count = api:physical_count(id, physical_kinds[self.kind_index])
        if not integer(self.count, self.limit - #self.pending.physical) then return nil, "collection_overflow" end
        if self.count == 0 then next_kind(self, false) else self.stage = "loadout_component" end
    elseif stage == "loadout_component" then
        local component = api:physical_component(id, physical_kinds[self.kind_index], self.slot)
        if type(component) ~= "string" or not component:match("^%d+$") or #component > 20 then return nil, "invalid_fact" end
        self.slot_pending = { kind = physical_kinds[self.kind_index], slot = self.slot, component = component }
        self.stage = "loadout_macro"
    elseif stage == "loadout_macro" then
        local macro = api:physical_macro(id, physical_kinds[self.kind_index], self.slot)
        if not string(macro) then return nil, "invalid_fact" end
        self.slot_pending.macro_name, self.stage = macro, "loadout_group"
    elseif stage == "loadout_group" then
        local group = api:physical_group(id, physical_kinds[self.kind_index], self.slot)
        if type(group) ~= "table" or not string(group.path) or not string(group.group) then return nil, "invalid_fact" end
        self.slot_pending.path, self.slot_pending.group = group.path, group.group
        self.pending.physical[#self.pending.physical + 1] = self.slot_pending
        self.slot_pending, self.slot = nil, self.slot + 1
        if self.slot > self.count then next_kind(self, false) else self.stage = "loadout_component" end
    elseif stage == "loadout_virtual_count" then
        self.count = api:virtual_count(id, virtual_kinds[self.kind_index])
        if not integer(self.count, self.limit - #self.pending.virtual_slots) then return nil, "collection_overflow" end
        if self.count == 0 then next_kind(self, true) else self.stage = "loadout_virtual_macro" end
    elseif stage == "loadout_virtual_macro" then
        local kind = virtual_kinds[self.kind_index]
        local macro = api:virtual_macro(id, kind, self.slot)
        if not string(macro) then return nil, "invalid_fact" end
        self.pending.virtual_slots[#self.pending.virtual_slots + 1] = { kind = kind, slot = self.slot, macro_name = macro }
        self.slot = self.slot + 1
        if self.slot > self.count then next_kind(self, true) end
    elseif stage:match("^loadout_array_") then
        return arrays_step(self, id)
    else return nil, "invalid_capture_stage" end
    self.pending.physical_outcome = #self.pending.physical == 0 and "empty" or "value"
    self.pending.virtual_outcome = #self.pending.virtual_slots == 0 and "empty" or "value"
    return true
end
return capture
