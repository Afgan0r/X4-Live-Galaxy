local root, mode, result_path, marker_path, prior_token = ...
assert(type(root) == "string" and type(result_path) == "string")
package.path = root .. "/extensions/?.lua;" .. package.path

local function write(path, text)
    local file = assert(io.open(path, "wb"))
    assert(file:write(text))
    assert(file:close())
end

local function open_carrier()
    local carrier = require("live_galaxy/lua/live_galaxy_carrier")
    return assert(carrier.new())
end

if mode == "reload" then
    local initializer = assert(package.loadlib(
        ".\\extensions\\live_galaxy\\ui_c_library_live_galaxy_carrier_64.txt",
        "luaopen_live_galaxy_carrier"))
    local api = assert(initializer())
    local stale = api.progress(assert(prior_token), 1)
    local fresh = open_carrier()
    write(result_path, string.format("return {stale=%d,fresh=%q,token=%q,getter_calls=1}",
        stale, fresh.token, prior_token))
    assert(fresh:close() == 0)
    return
end

local carrier = open_carrier()
local token = carrier.token
local getter_calls = 0
local observation = assert(require("live_galaxy/lua/live_galaxy_observation").new({
    getter = function() getter_calls = getter_calls + 1; return 123.5 end,
}))
local scheduler = require("live_galaxy/lua/live_galaxy_scheduler")
local sampled, last = false, "none"
for index = 1, 200000 do
    local result = scheduler.tick({
        capture_start_millis = tostring(index),
        capture_end_millis = tostring(index + 1),
    }, carrier, observation)
    last = result.disposition
    if last == "sampled" then sampled = true; break end
    if index % 10000 == 0 then write(marker_path, last) end
    host_sleep(0)
end
assert(sampled, "sample watchdog: " .. last)

if mode == "pending-io-unload" then
    write(marker_path, "kill-bridge")
    host_sleep(500)
    carrier:progress(1)
    write(result_path, string.format(
        "return {actual_native=true,token=%q,getter_calls=%d,sampled=true}",
        token, getter_calls))
    return
end

local committed = false
for _ = 1, 200000 do
    local progress, state = carrier:progress(1)
    local control = carrier:poll_control()
    if control ~= 3 then
        local trace = assert(io.open(marker_path, "ab"))
        trace:write(string.format("progress=%s state=%s control=%s\n",
            tostring(progress), tostring(state), tostring(control)))
        trace:close()
    end
    if control == 5 then committed = true; break end
    host_sleep(0)
end
assert(committed, "commit watchdog")
write(result_path, string.format(
    "return {actual_native=true,token=%q,getter_calls=%d,sampled=true}",
    token, getter_calls))
assert(carrier:close() == 0)
