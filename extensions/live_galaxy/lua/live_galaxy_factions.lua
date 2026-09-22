local factions = {}
local MAX_INTEGER = 9007199254740991

local function integer(value)
    return type(value) == "number" and value >= 0 and value <= MAX_INTEGER and value % 1 == 0
end

local function token(value)
    return type(value) == "string" and #value > 0 and #value <= 128
        and value:match("^[%w_:%-]+$") ~= nil
end

local function classify(id, inventory)
    local evidence = inventory[id]
    if evidence == nil then
        return { id = id, disposition = "unknown", reason = "origin_unresolved",
            mind_candidate = false }
    end
    local disposition, reason
    if (evidence.origin == "vanilla" or evidence.origin == "dlc")
        and evidence.independent == true then
        disposition, reason = "included", "independent_first_party"
    elseif evidence.origin == "player" or evidence.origin == "modded" then
        disposition, reason = "excluded", "outside_mandatory_coverage"
    elseif evidence.independent == false then
        disposition, reason = "excluded", "not_independent"
    else
        disposition, reason = "unknown", "independence_unresolved"
    end
    return { id = id, origin = evidence.origin, independent = evidence.independent,
        disposition = disposition, reason = reason, evidence = evidence.evidence,
        mind_candidate = disposition == "included" and evidence.mind_candidate == true }
end

function factions.capture(api, inventory, revision, limits)
    if type(api) ~= "table" or type(inventory) ~= "table" or not integer(revision)
        or type(limits) ~= "table" or not integer(limits.max_factions)
        or not integer(limits.max_allocation_bytes) or not integer(limits.pointer_bytes)
        or limits.max_factions == 0 or limits.pointer_bytes == 0 then
        return nil, "invalid_contract"
    end
    local count = api:count_factions()
    if not integer(count) or count > limits.max_factions
        or count * limits.pointer_bytes > limits.max_allocation_bytes then
        return nil, "census_overflow"
    end
    local buffer = api:new_faction_buffer(count)
    if api:fill_factions(buffer, count) ~= count then return nil, "census_incomplete" end
    local entries, by_id, unknown = {}, {}, false
    for index = 0, count - 1 do
        local id = api:faction_string(buffer[index])
        if not token(id) then return nil, "invalid_faction" end
        if by_id[id] ~= nil then return nil, "duplicate_faction" end
        local entry = classify(id, inventory)
        entries[index + 1], by_id[id] = entry, entry
        unknown = unknown or entry.disposition == "unknown"
    end
    return { discovery_revision = revision, entries = entries, by_id = by_id,
        unknown_blocker = unknown }
end

return factions
