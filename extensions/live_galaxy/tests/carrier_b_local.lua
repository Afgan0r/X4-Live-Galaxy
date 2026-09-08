local root, mode, result_path, marker_path, prior_token = ...
assert(type(root) == "string" and type(result_path) == "string")
package.path = root .. "/extensions/?.lua;" .. package.path

local function write(path, text)
    local file = assert(io.open(path, "wb"))
    assert(file:write(text))
    assert(file:close())
end

local function read(path)
    local file = assert(io.open(path, "rb"))
    local text = assert(file:read("*a"))
    assert(file:close())
    return text
end

local function open_carrier()
    local carrier = require("live_galaxy/lua/live_galaxy_carrier")
    return assert(carrier.new())
end

if mode == "reload" then
    local prior = read(result_path)
    local pending_state = assert(prior:match('pending_state="([^"]+)"'))
    local pending_incarnation = assert(prior:match('pending_incarnation="([^"]+)"'))
    local pending_owners = assert(prior:match("pending_owners=(%d+)"))
    local clock_calls = assert(prior:match("clock_calls=(%d+)"))
    local initializer = assert(package.loadlib(
        ".\\extensions\\live_galaxy\\ui_c_library_live_galaxy_carrier_64.txt",
        "luaopen_live_galaxy_carrier"))
    local api = assert(initializer())
    local stale = api.close(assert(prior_token))
    local fresh = open_carrier()
    write(result_path, string.format(
        "return {actual_native=true,stale=%d,fresh=%q,token=%q,getter_calls=1,clock_calls=%s,pending_state=%q,pending_incarnation=%q,pending_owners=%s}",
        stale, fresh.token, prior_token, clock_calls, pending_state,
        pending_incarnation, pending_owners))
    assert(fresh:close() == 0)
    return
end

local carrier = open_carrier()
local token = carrier.token
local getter_calls, clock_calls = 0, 0
local observation = assert(require("live_galaxy/lua/live_galaxy_observation").new({
    getter = function() getter_calls = getter_calls + 1; return 123.5 end,
    clock_getter = function()
        clock_calls = clock_calls + 1
        return clock_calls == 1 and 10.25 or 10.5
    end,
}))
local scheduler = require("live_galaxy/lua/live_galaxy_scheduler")
local sampled, last = false, "none"
local sample_deadline = host_monotonic_millis() + 10000
while host_monotonic_millis() < sample_deadline do
    local result = scheduler.tick({
        capture_start_millis = "1",
        capture_end_millis = "2",
    }, carrier, observation)
    last = result.disposition
    if last == "sampled" then sampled = true; break end
    write(marker_path, last)
    host_sleep(1)
end
local sample_status = carrier.current_status or {}
assert(sampled, string.format(
    "sample watchdog: last=%s producer=%s transport=%s monotonic=%s",
    last, tostring(sample_status.producer_state),
    tostring(sample_status.transport_state), tostring(sample_status.monotonic_millis)))

if mode == "pending-io-unload" then
    local pending, status = carrier:progress(1)
    assert(pending == 1, "pending handoff required")
    assert(type(status) == "table" and status.producer_state == "pending_start")
    local owners = tonumber(status.capacity:match(":(%d+)$"))
    assert(owners and owners > 0, "pending OS owner required")
    local generation = assert(token:match("^(%d+):"), "token generation")
    assert(status.producer_incarnation == "x4-producer-" .. generation, "pending identity")
    write(marker_path, "kill-bridge")
    host_sleep(500)
    local retry = carrier:progress(1)
    assert(retry == 1 or retry == 2, "retained pending bytes required")
    write(result_path, string.format(
        "return {actual_native=true,token=%q,getter_calls=%d,clock_calls=%d,sampled=true,pending_state=%q,pending_incarnation=%q,pending_owners=%d}",
        token, getter_calls, clock_calls, status.producer_state,
        status.producer_incarnation, owners))
    return
end

local committed = false
local commit_deadline = host_monotonic_millis() + 10000
while host_monotonic_millis() < commit_deadline do
    local progress, state = carrier:progress(1)
    local control = carrier:poll_control()
    if control ~= 3 then
        local trace = assert(io.open(marker_path, "ab"))
        trace:write(string.format("progress=%s state=%s control=%s\n",
            tostring(progress), tostring(state), tostring(control)))
        trace:close()
    end
    if control == 5 then committed = true; break end
    host_sleep(1)
end
local commit_status = carrier.current_status or {}
assert(committed, string.format(
    "commit watchdog: producer=%s transport=%s monotonic=%s",
    tostring(commit_status.producer_state), tostring(commit_status.transport_state),
    tostring(commit_status.monotonic_millis)))
write(result_path, string.format(
    "return {actual_native=true,token=%q,getter_calls=%d,clock_calls=%d,sampled=true}",
    token, getter_calls, clock_calls))
assert(carrier:close() == 0)
