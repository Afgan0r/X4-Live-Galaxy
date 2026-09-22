local helper = require("test_helper")

local function values()
    local file = assert(io.open("config/heavy-ship-experiment.json", "rb"))
    local text = assert(file:read("*a"))
    assert(file:close())
    local result = {}
    for key, value in text:gmatch('"([%w_]+)"%s*:%s*(%d+)') do
        result[key] = tonumber(value)
    end
    return result
end

describe("dynamic faction observation contract", function()
    local fixture, profile

    before_each(function()
        fixture = helper.new()
        profile = fixture.load("live_galaxy_ship_profile")
    end)

    after_each(function()
        fixture.restore()
    end)

    for _, faction in ipairs({ "xenon", "khaak" }) do
        it("admits mandatory hostile observation subject " .. faction, function()
            local options, reason = profile.options(values(), faction)
            assert.is_nil(reason)
            assert.is_table(options)
            assert.equals("x4:faction:" .. faction .. ":ships", options.observation.source_scope)
        end)
    end
end)
