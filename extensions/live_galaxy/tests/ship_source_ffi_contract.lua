local helper = require("test_helper")

describe("ship detail FFI contract", function()
    local fixture
    before_each(function() fixture = helper.new() end)
    after_each(function() fixture.restore() end)

    it("passes UniverseID and exact selectors to every detail C call", function()
        local function object(value) assert.equals("64:42", value) end
        local C = {
            GetPeopleCapacity = function(value, macro, include_pilot)
                object(value); assert.equals("", macro); assert.is_false(include_pilot); return 4
            end,
            GetPeople2 = function(_, count, value, include_arriving)
                object(value); assert.equals(2, count); assert.is_true(include_arriving); return 0
            end,
            GetRoleTiers = function(_, count, value, role)
                object(value); assert.equals(2, count); assert.equals("service", role); return 0
            end,
            GetNumCargoTransportTypes = function(value, merge)
                object(value); assert.is_true(merge); return 0
            end,
            GetCargoTransportTypes = function(_, count, value, merge, after_orders)
                object(value); assert.equals(2, count); assert.is_true(merge)
                assert.is_false(after_orders); return 0
            end,
            GetNumUpgradeSlots = function(value, macro, kind)
                object(value); assert.equals("", macro); assert.equals("engine", kind); return 0
            end,
            GetUpgradeSlotCurrentComponent = function(value, kind, slot)
                object(value); assert.equals("engine", kind); assert.equals(1, slot); return "1ULL"
            end,
            GetUpgradeSlotCurrentMacro = function(value, zero, kind, slot)
                object(value); assert.equals(0, zero); assert.equals("engine", kind)
                assert.equals(1, slot); return "macro"
            end,
            GetUpgradeSlotGroup = function(value, macro, kind, slot)
                object(value); assert.equals("", macro); assert.equals("engine", kind)
                assert.equals(1, slot); return { path = "path", group = "group" }
            end,
            GetNumVirtualUpgradeSlots = function(value, macro, kind)
                object(value); assert.equals("", macro); assert.equals("engine", kind); return 0
            end,
            GetVirtualUpgradeSlotCurrentMacro = function(value, kind, slot)
                object(value); assert.equals("engine", kind); assert.equals(1, slot); return "virtual"
            end,
            GetNumSoftwareSlots = function(value, macro)
                object(value); assert.equals("", macro); return 0
            end,
            GetSoftwareSlots = function(_, count, value, macro)
                object(value); assert.equals(2, count); assert.equals("", macro); return 0
            end,
            GetNumMissileCargo = function(value) object(value); return 0 end,
            GetMissileCargo = function(_, count, value)
                object(value); assert.equals(2, count); return 0
            end,
            GetNumAllUnits = function(value, only_drones)
                object(value); assert.is_false(only_drones); return 0
            end,
            GetAllUnits = function(_, count, value, only_drones)
                object(value); assert.equals(2, count); assert.is_false(only_drones); return 0
            end,
        }
        package.loaded.ffi = { C = C, string = function(value) return value end,
            sizeof = function() return 1 end, new = function() return {} end }
        _G.ConvertStringToLuaID = function(value)
            assert.equals("42", value); return "lua:42"
        end
        _G.ConvertIDTo64Bit = function(value)
            assert.equals("lua:42", value); return "64:42"
        end
        _G.GetComponentData = function(value, selector)
            assert.equals("lua:42", value)
            assert.equals("cargo", selector)
            return { ware = 1 }
        end
        local api = fixture.load("live_galaxy_ship_source").runtime()
        assert.same({ ware = 1 }, api:cargo_wares("42"))
        assert.equals(4, api:crew_capacity("42"))
        assert.same({}, api:crew_fill("42", {}, 2))
        assert.same({}, api:crew_tier_fill("42", "service", {}, 2))
        assert.equals(0, api:cargo_storage_count("42"))
        assert.same({}, api:cargo_storage_fill("42", {}, 2))
        assert.equals(0, api:physical_count("42", "engine"))
        assert.equals("1", api:physical_component("42", "engine", 1))
        assert.equals("macro", api:physical_macro("42", "engine", 1))
        assert.same({ path = "path", group = "group" }, api:physical_group("42", "engine", 1))
        assert.equals(0, api:virtual_count("42", "engine"))
        assert.equals("virtual", api:virtual_macro("42", "engine", 1))
        assert.equals(0, api:software_count("42"))
        assert.same({}, api:software_fill("42", {}, 2))
        assert.equals(0, api:missiles_count("42"))
        assert.same({}, api:missiles_fill("42", {}, 2))
        assert.equals(0, api:units_count("42"))
        assert.same({}, api:units_fill("42", {}, 2))
    end)
end)
