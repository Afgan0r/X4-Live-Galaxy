local lfs = require("lfs")

describe("shipped Lua syntax #syntax", function()
    local function check_directory(directory)
        local entries = {}
        for name in lfs.dir(directory) do
            if name ~= "." and name ~= ".." and name ~= "tests" then entries[#entries + 1] = name end
        end
        table.sort(entries)
        for _, name in ipairs(entries) do
            local path = directory .. "/" .. name
            if lfs.attributes(path, "mode") == "directory" then check_directory(path)
            elseif name:match("%.lua$") then
                it("compiles " .. path, function()
                    local chunk, err = loadfile(path)
                    assert.is_function(chunk, err)
                end)
            end
        end
    end
    check_directory("extensions/live_galaxy")
end)
