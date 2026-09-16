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
    }
end

return source
