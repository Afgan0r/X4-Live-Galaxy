local source = {}

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

function source.runtime()
    local ffi = require("ffi")
    local C = ffi.C
    return {
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
            local owner, macro, class, sector = GetComponentData(
                component, "owner", "macro", "classid", "sectorid")
            local sector_identity = source.identity(sector)
            if sector_identity == nil then return nil end
            return { identity = identity, owner = owner, type = macro,
                class = class, location = "sector:" .. sector_identity }
        end,
        cargo_wares = function(_, identity)
            return GetComponentData(ConvertStringToLuaID(identity), "cargo")
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
end

return source
