local source = {}
function source.extend(api, ffi, C, identity_string)
    local function object(id) return ConvertIDTo64Bit(ConvertStringToLuaID(id)) end
    api.physical_count = function(_, id, kind) return tonumber(C.GetNumUpgradeSlots(object(id), "", kind)) end
    api.physical_component = function(_, id, kind, slot)
        local raw = C.GetUpgradeSlotCurrentComponent(object(id), kind, slot)
        if tostring(raw):match("^0U?L?L?$") then return "0" end
        return identity_string(raw)
    end
    api.physical_macro = function(_, id, kind, slot)
        return ffi.string(C.GetUpgradeSlotCurrentMacro(object(id), 0, kind, slot))
    end
    api.physical_group = function(_, id, kind, slot)
        local raw = C.GetUpgradeSlotGroup(object(id), "", kind, slot)
        -- Returned by value; copy both pointer-backed strings before yielding.
        return { path = ffi.string(raw.path), group = ffi.string(raw.group) }
    end
    api.virtual_count = function(_, id, kind) return tonumber(C.GetNumVirtualUpgradeSlots(object(id), "", kind)) end
    api.virtual_macro = function(_, id, kind, slot)
        return ffi.string(C.GetVirtualUpgradeSlotCurrentMacro(object(id), kind, slot))
    end
    api.software_count = function(_, id) return tonumber(C.GetNumSoftwareSlots(object(id), "")) end
    api.software_size = function() return ffi.sizeof("SoftwareSlot") end
    api.software_allocate = function(_, count) return ffi.new("SoftwareSlot[?]", count) end
    api.software_fill = function(_, id, buffer, count)
        local n = tonumber(C.GetSoftwareSlots(buffer, count, object(id), ""))
        if n == nil or n < 0 or n > count then return nil end
        local rows = {}
        for i = 0, n - 1 do rows[i + 1] = { maximum = ffi.string(buffer[i].max), current = ffi.string(buffer[i].current) } end
        return rows
    end
    api.missiles_count = function(_, id) return tonumber(C.GetNumMissileCargo(object(id))) end
    api.missiles_size = function() return ffi.sizeof("UIWareInfo") end
    api.missiles_allocate = function(_, count) return ffi.new("UIWareInfo[?]", count) end
    api.missiles_fill = function(_, id, buffer, count)
        local n = tonumber(C.GetMissileCargo(buffer, count, object(id)))
        if n == nil or n < 0 or n > count then return nil end
        local rows = {}
        for i = 0, n - 1 do rows[i + 1] = { ware = ffi.string(buffer[i].ware), macro_name = ffi.string(buffer[i].macro), amount_raw = tonumber(buffer[i].amount) } end
        return rows
    end
    api.units_count = function(_, id) return tonumber(C.GetNumAllUnits(object(id), false)) end
    api.units_size = function() return ffi.sizeof("UnitData") end
    api.units_allocate = function(_, count) return ffi.new("UnitData[?]", count) end
    api.units_fill = function(_, id, buffer, count)
        local n = tonumber(C.GetAllUnits(buffer, count, object(id), false))
        if n == nil or n < 0 or n > count then return nil end
        local rows = {}
        for i = 0, n - 1 do rows[i + 1] = { macro_name = ffi.string(buffer[i].macro), category = ffi.string(buffer[i].category), amount_items = tonumber(buffer[i].amount) } end
        return rows
    end
end
return source
