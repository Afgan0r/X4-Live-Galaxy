local normalize = {}

local valid_quality = {
    fresh = true,
    known_empty = true,
    unknown = true,
    partial = true,
    stale = true,
    unsupported = true,
}

local function valid_nonempty_string(value)
    return type(value) == "string" and value ~= ""
end

local function valid_entity_id(value)
    return valid_nonempty_string(value) and string.match(value, "^[%w_:%-]+$") ~= nil
end

local function valid_positive_integer(value)
    return type(value) == "number" and value >= 1 and value % 1 == 0
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
    if not valid_positive_integer(raw.observed_at_unix_millis) then
        return nil, "observed_at_invalid"
    end
    if not valid_positive_integer(raw.version) then
        return nil, "version_invalid"
    end
    if not valid_quality[raw.quality] then
        return nil, "quality_unsupported"
    end

    return {
        entity_id = raw.entity_id,
        source = raw.source,
        observed_at_unix_millis = raw.observed_at_unix_millis,
        version = raw.version,
        quality = raw.quality,
    }
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
        '{"entity_id":"%s","observed_at_unix_millis":%d,"version":%d,"quality":"%s"}',
        normalized.entity_id,
        normalized.observed_at_unix_millis,
        normalized.version,
        normalized.quality
    )
end

return normalize
