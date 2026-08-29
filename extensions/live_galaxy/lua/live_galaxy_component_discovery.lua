local discovery = {}

local MAX_OWNER_STATIONS = 16
local MAX_FACT_STRING_BYTES = 96
local MAX_SAFE_INTEGER = 9007199254740991
local FACT_DIAGNOSTIC_CLASSES = {
    asset_identity_invalid = true,
    capacity_invalid = true,
    capacity_unavailable = true,
    metadata_unavailable = true,
    owner_invalid = true,
    owner_scope_empty = true,
    owner_scope_mismatch = true,
    sector_invalid = true,
}

local function callable(api, name)
    return type(api) == "table" and type(api[name]) == "function"
end

local function valid_integer(value)
    return type(value) == "number" and value >= 0 and value <= MAX_SAFE_INTEGER
        and value == value and value % 1 == 0
end

local function valid_id(value)
    return type(value) == "string" and #value > 0 and #value <= MAX_FACT_STRING_BYTES
        and value:match("^[%w_:%-]+$") ~= nil
end

local function runtime_api()
    local globals = _G
    local ffi = require("ffi")
    local C = ffi.C
    return {
        count_stations = function(_, faction_id) return C.GetNumAllFactionStations(faction_id) end,
        new_buffer = function(_, count) return ffi.new("UniverseID[?]", count) end,
        fill_stations = function(_, buffer, count, faction_id)
            return C.GetAllFactionStations(buffer, count, faction_id)
        end,
        to_component = function(_, raw_id) return globals.ConvertStringToLuaID(tostring(raw_id)) end,
        to_component64 = function(_, component) return globals.ConvertIDTo64Bit(component) end,
        get_component_data = function(_, component)
            return globals.GetComponentData(component, "owner", "sector")
        end,
        get_people_capacity = function(_, component64)
            return C.GetPeopleCapacity(component64, "", false)
        end,
        canonical_owner_id = function(_, owner_id)
            if type(owner_id) ~= "string" or owner_id == "" then return nil end
            if owner_id:sub(1, 8) == "faction:" then return owner_id end
            return "faction:" .. owner_id
        end,
        faction_id = "faction:argon",
        native_faction_id = "argon",
    }
end

local function api_available(api)
    return callable(api, "count_stations") and callable(api, "new_buffer")
        and callable(api, "fill_stations") and callable(api, "to_component") and callable(api, "to_component64")
        and callable(api, "get_component_data") and callable(api, "get_people_capacity")
end

local function section_from_candidates(candidates)
    local sectors, assets, capacity, ownership, known_sectors = {}, {}, {}, {}, {}
    for _, candidate in ipairs(candidates) do
        if not known_sectors[candidate.sector_id] then
            known_sectors[candidate.sector_id] = true
            sectors[#sectors + 1] = { id = candidate.sector_id }
        end
        assets[#assets + 1] = { id = candidate.asset_id, sector_id = candidate.sector_id }
        capacity[#capacity + 1] = { id = "capacity:station:" .. candidate.stable_id,
            asset_id = candidate.asset_id, value = candidate.people_capacity }
        ownership[#ownership + 1] = { id = "ownership:station:" .. candidate.stable_id,
            asset_id = candidate.asset_id, owner_id = candidate.owner_id }
    end
    table.sort(sectors, function(left, right) return left.id < right.id end)
    return { entity_id = sectors[1].id, source = "x4_runtime", version = 1, quality = "fresh",
        runtime_facts = { source = "x4_runtime", quality = "fresh", availability = "available",
            sectors = sectors, assets = assets, capacity = capacity, ownership = ownership } }
end

function discovery.new(api)
    local adapter = {}
    local last_diagnostic_class

    local function unsupported(class)
        last_diagnostic_class = FACT_DIAGNOSTIC_CLASSES[class] and class or nil
        return nil, "facts_unsupported"
    end

    function adapter.diagnostic_class()
        return last_diagnostic_class
    end

    function adapter.read_observation(_, version)
        last_diagnostic_class = nil
        if not api_available(api) then return nil, "enumeration_unavailable" end
        local native_faction_id = api.native_faction_id or api.faction_id
        local count_ok, raw_count = pcall(api.count_stations, api, native_faction_id)
        local count = tonumber(raw_count)
        if not count_ok or not valid_integer(count) then return nil, "enumeration_unavailable" end
        if count > MAX_OWNER_STATIONS then return nil, "enumeration_overflow" end
        if count == 0 then return unsupported("owner_scope_empty") end

        local allocation_ok, buffer = pcall(api.new_buffer, api, count)
        if not allocation_ok or buffer == nil then return nil, "enumeration_unavailable" end
        local fill_ok, raw_filled = pcall(api.fill_stations, api, buffer, count, native_faction_id)
        local filled = tonumber(raw_filled)
        if not fill_ok or not valid_integer(filled) or filled ~= count then return nil, "enumeration_incomplete" end

        local candidates, stable_ids = {}, {}
        for index = 0, count - 1 do
            local component_ok, component = pcall(api.to_component, api, buffer[index])
            if not component_ok or component == nil or component == false then return nil, "identity_invalid" end
            local id_ok, component64 = pcall(api.to_component64, api, component)
            local stable_id = id_ok and tostring(component64) or nil
            if not id_ok or component64 == nil or component64 == false
                or not valid_id(stable_id) or stable_ids[stable_id] then
                return nil, "identity_invalid"
            end
            stable_ids[stable_id] = true
            candidates[#candidates + 1] = {
                component = component,
                component64 = component64,
                stable_id = stable_id,
            }
        end
        table.sort(candidates, function(left, right) return left.stable_id < right.stable_id end)

        for _, candidate in ipairs(candidates) do
            local metadata_ok, raw_owner_id, sector_id = pcall(
                api.get_component_data, api, candidate.component
            )
            local capacity_ok, people_capacity = pcall(
                api.get_people_capacity, api, candidate.component64
            )
            local owner_id = raw_owner_id
            if metadata_ok and callable(api, "canonical_owner_id") then
                local canonical_ok, canonical_owner_id = pcall(
                    api.canonical_owner_id, api, raw_owner_id
                )
                if not canonical_ok then return unsupported("owner_invalid") end
                owner_id = canonical_owner_id
            end
            candidate.asset_id = "asset:station:" .. candidate.stable_id
            candidate.owner_id, candidate.sector_id, candidate.people_capacity = owner_id, sector_id, people_capacity
            if not metadata_ok then return unsupported("metadata_unavailable") end
            if not capacity_ok then return unsupported("capacity_unavailable") end
            if not valid_id(candidate.asset_id) then return unsupported("asset_identity_invalid") end
            if not valid_id(owner_id) then return unsupported("owner_invalid") end
            if owner_id ~= api.faction_id then return unsupported("owner_scope_mismatch") end
            if not valid_id(sector_id) then return unsupported("sector_invalid") end
            if not valid_integer(people_capacity) then return unsupported("capacity_invalid") end
        end

        local observation = section_from_candidates(candidates)
        if valid_integer(version) and version >= 1 then observation.version = version end
        return observation
    end

    return adapter
end

function discovery.new_runtime_adapter()
    return discovery.new(runtime_api())
end

return discovery
