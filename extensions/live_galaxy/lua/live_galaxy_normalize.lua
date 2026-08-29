local normalize = {}
local MAX_RUNTIME_FACTS_PER_CLASS = 16
local MAX_RUNTIME_FACT_STRING_BYTES = 96
local MAX_SAFE_INTEGER = 9007199254740991

local valid_quality = {
    fresh = true,
    known_empty = true,
    unknown = true,
    partial = true,
    stale = true,
    unsupported = true,
}

local function valid_nonempty_string(value)
    return type(value) == "string" and value ~= "" and #value <= MAX_RUNTIME_FACT_STRING_BYTES
end

local function valid_entity_id(value)
    return valid_nonempty_string(value) and string.match(value, "^[%w_:%-]+$") ~= nil
end

local function valid_positive_integer(value)
    return type(value) == "number" and value >= 1 and value <= MAX_SAFE_INTEGER and value % 1 == 0
end

local function valid_nonnegative_integer(value)
    return type(value) == "number" and value >= 0 and value <= MAX_SAFE_INTEGER and value % 1 == 0
end

local function sorted_unique(items, key)
    if type(items) ~= "table" or #items < 1 or #items > MAX_RUNTIME_FACTS_PER_CLASS then
        return false
    end
    local previous
    for _, item in ipairs(items) do
        local current = item[key]
        if not valid_entity_id(current) or (previous ~= nil and previous >= current) then
            return false
        end
        previous = current
    end
    return true
end

local function has_item(items, key, expected)
    for _, item in ipairs(items) do
        if item[key] == expected then return true end
    end
    return false
end

local function normalize_runtime_facts(raw, entity_id)
    if type(raw) ~= "table" or raw.source ~= "x4_runtime"
        or raw.quality ~= "fresh" or raw.availability ~= "available"
        or not sorted_unique(raw.sectors, "id")
        or not sorted_unique(raw.assets, "id")
        or not sorted_unique(raw.capacity, "id")
        or not sorted_unique(raw.ownership, "id")
        or not has_item(raw.sectors, "id", entity_id) then
        return nil, "runtime_facts_invalid"
    end
    for _, item in ipairs(raw.assets) do
        if not valid_entity_id(item.sector_id) or not has_item(raw.sectors, "id", item.sector_id) then
            return nil, "runtime_facts_invalid"
        end
    end
    for _, item in ipairs(raw.capacity) do
        if not valid_entity_id(item.asset_id) or not valid_nonnegative_integer(item.value)
            or not has_item(raw.assets, "id", item.asset_id) then
            return nil, "runtime_facts_invalid"
        end
    end
    for _, item in ipairs(raw.ownership) do
        if not valid_entity_id(item.asset_id) or not valid_entity_id(item.owner_id)
            or not has_item(raw.assets, "id", item.asset_id) then
            return nil, "runtime_facts_invalid"
        end
    end
    if raw.x4_game_time ~= nil and not valid_nonnegative_integer(raw.x4_game_time) then
        return nil, "runtime_facts_invalid"
    end
    return raw
end

function normalize.normalize_section(raw)
    if type(raw) ~= "table" then
        return nil, "section_invalid"
    end
    if not valid_entity_id(raw.entity_id) then
        return nil, "entity_id_invalid"
    end
    if not valid_nonempty_string(raw.source) then
        return nil, "source_invalid"
    end
    if not valid_positive_integer(raw.version) then
        return nil, "version_invalid"
    end
    if not valid_quality[raw.quality] then
        return nil, "quality_unsupported"
    end

    local runtime_facts, facts_err = normalize_runtime_facts(
        raw.runtime_facts, raw.entity_id
    )
    if runtime_facts == nil then return nil, facts_err end
    return {
        entity_id = raw.entity_id,
        source = raw.source,
        version = raw.version,
        quality = raw.quality,
        runtime_facts = runtime_facts,
    }
end

local function serialize_runtime_facts(facts)
    local sectors, assets, capacity, ownership = {}, {}, {}, {}
    for _, item in ipairs(facts.sectors) do
        sectors[#sectors + 1] = string.format('{"i":"%s"}', item.id)
    end
    for _, item in ipairs(facts.assets) do
        assets[#assets + 1] = string.format('{"i":"%s","p":"%s"}', item.id, item.sector_id)
    end
    for _, item in ipairs(facts.capacity) do
        capacity[#capacity + 1] = string.format(
            '{"i":"%s","p":"%s","v":%.0f}', item.id, item.asset_id, item.value
        )
    end
    for _, item in ipairs(facts.ownership) do
        ownership[#ownership + 1] = string.format(
            '{"i":"%s","p":"%s","n":"%s"}', item.id, item.asset_id, item.owner_id
        )
    end
    local game_time = facts.x4_game_time == nil and "" or string.format(',"g":%.0f', facts.x4_game_time)
    return string.format(
        '{"r":"x4_runtime"%s,"q":"fresh","a":"available","s":[%s],"x":[%s],"c":[%s],"o":[%s]}',
        game_time, table.concat(sectors, ","), table.concat(assets, ","), table.concat(capacity, ","), table.concat(ownership, ",")
    )
end

function normalize.serialize_telemetry(section)
    local normalized, err = normalize.normalize_section(section)
    if normalized == nil then
        return nil, err
    end
    if normalized.source ~= "x4_runtime" then
        return nil, "source_unsupported"
    end

    return string.format(
        '{"entity_id":"%s","version":%d,"quality":"%s","runtime_facts":%s}',
        normalized.entity_id,
        normalized.version,
        normalized.quality,
        serialize_runtime_facts(normalized.runtime_facts)
    )
end

return normalize
