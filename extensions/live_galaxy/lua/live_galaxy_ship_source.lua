local source = {}
local MAX_SAFE_INTEGER = 9007199254740991

-- UniverseID never crosses Lua's floating-point number representation.
function source.identity(value)
    if type(value) == "number" then return nil end
    local raw = tostring(value)
    local decimal = raw:match("^(%d+)ULL$") or raw:match("^(%d+)$")
    if decimal == nil or decimal == "0" or decimal:sub(1, 1) == "0"
        or #decimal > 20 or #decimal == 20 and decimal > "18446744073709551615" then
        return nil
    end
    return decimal
end

local function class_token(value)
    if type(value) == "string" then return value end
    if type(value) ~= "number" or value < 0 or value > MAX_SAFE_INTEGER or value % 1 ~= 0 then
        return nil
    end
    if value == 0 then return "classid:0" end
    return "classid:" .. string.format("%.0f", value)
end

local function location_token(value)
    local raw = tostring(value)
    if raw == "0ULL" or raw == "0" then return "context:none" end
    local identity = source.identity(value)
    if identity == nil then return nil end
    return "sector:" .. identity
end

function source.runtime()
    local ffi = require("ffi")
    local C = ffi.C
    local api = {
        count_factions = function() return C.GetNumAllFactions(false) end,
        new_faction_buffer = function(_, count) return ffi.new("const char*[?]", count) end,
        fill_factions = function(_, buffer, count) return C.GetAllFactions(buffer, count, false) end,
        faction_string = function(_, pointer) return ffi.string(pointer) end,
        count_ships = function(_, faction) return C.GetNumAllFactionShips(faction) end,
        new_buffer = function(_, count) return ffi.new("UniverseID[?]", count) end,
        fill_ships = function(_, buffer, count, faction)
            return C.GetAllFactionShips(buffer, count, faction)
        end,
        read_core = function(_, identity)
            local component = ConvertStringToLuaID(identity)
            local owner, macro, raw_class = GetComponentData(component, "owner", "macro", "classid")
            local class = class_token(raw_class)
            if class == nil then
                return nil, { condition = "token_invalid", field = "class", observed_type = type(raw_class) }
            end
            local component64 = ConvertIDTo64Bit(component)
            local sector64 = C.GetContextByClass(component64, "sector", false)
            local location = location_token(sector64)
            if location == nil then
                return nil, { condition = "identity_invalid", field = "sectorid", observed_type = type(sector64) }
            end
            return { identity = identity, owner = owner, type = macro,
                class = class, location = location }
        end,
        cargo_wares = function(_, identity)
            return GetComponentData(ConvertStringToLuaID(identity), "cargo")
        end,
        crew_capacity = function(_, identity)
            return tonumber(C.GetPeopleCapacity(ConvertStringToLuaID(identity), true))
        end,
        crew_count = function() return tonumber(C.GetNumAllRoles()) end,
        crew_size = function() return ffi.sizeof("PeopleInfo") end,
        crew_allocate = function(_, count) return ffi.new("PeopleInfo[?]", count) end,
        crew_fill = function(_, identity, buffer, count)
            local returned = tonumber(C.GetPeople2(buffer, count, ConvertStringToLuaID(identity), true))
            if returned == nil or returned < 0 or returned > count then return nil end
            local rows = {}
            for i = 0, returned - 1 do
                rows[i + 1] = { id = ffi.string(buffer[i].id), amount_people = tonumber(buffer[i].amount),
                    reported_numtiers = tonumber(buffer[i].numtiers), canhire = buffer[i].canhire, tiers = {} }
            end
            return rows
        end,
        crew_tier_size = function() return ffi.sizeof("RoleTierData") end,
        crew_tier_allocate = function(_, count) return ffi.new("RoleTierData[?]", count) end,
        crew_tier_fill = function(_, identity, role, buffer, count)
            local returned = tonumber(C.GetRoleTiers(buffer, count, ConvertStringToLuaID(identity), role))
            if returned == nil or returned < 0 or returned > count then return nil end
            local rows = {}
            for i = 0, returned - 1 do
                rows[i + 1] = { name = ffi.string(buffer[i].name),
                    skill_lower_threshold = tonumber(buffer[i].skilllevel), amount_people = tonumber(buffer[i].amount) }
            end
            return rows
        end,
        cargo_storage_count = function(_, identity)
            return tonumber(C.GetNumCargoTransportTypes(ConvertStringToLuaID(identity), true))
        end,
        cargo_storage_size = function() return ffi.sizeof("StorageInfo") end,
        cargo_storage_allocate = function(_, count) return ffi.new("StorageInfo[?]", count) end,
        cargo_storage_fill = function(_, identity, buffer, count)
            local returned = tonumber(C.GetCargoTransportTypes(buffer, count,
                ConvertStringToLuaID(identity), true, false))
            if returned == nil or returned < 0 or returned > count then return nil end
            -- Copy every pointer-backed string before returning/yielding.
            local copied = {}
            for i = 0, returned - 1 do
                copied[i + 1] = { transport = ffi.string(buffer[i].transport),
                    capacity_cubic_metres = tonumber(buffer[i].capacity),
                    occupied_cubic_metres = tonumber(buffer[i].spaceused) }
            end
            return copied
        end,
    }
    require("live_galaxy.lua.live_galaxy_ship_loadout_source").extend(api, ffi, C, source.identity)
    return api
end

return source
