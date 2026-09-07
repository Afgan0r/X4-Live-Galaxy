local carrier = {}

local DLL_PATH = ".\\extensions\\live_galaxy\\ui_c_library_live_galaxy_carrier_64.txt"
local INITIALIZER = "luaopen_live_galaxy_carrier"
local OPERATIONS = {
    "abi_version", "open", "begin_section", "push_record", "finish_section",
    "fail_section", "progress", "poll_control", "reset", "close",
}
local LIMITS = {
    data_message_bytes = 2048,
    control_message_bytes = 512,
    max_records = 1,
    max_content_bytes = 96,
    max_canonical_bytes = 2048,
    max_batches = 1,
    max_work = 1,
    max_age_millis = 5000,
    pending_slots = 1,
    max_attempts = 2,
    max_retry_age_millis = 5000,
    availability_interval_millis = 5000,
}
local SOURCE = {
    source_scope = "x4:carrier_b_acceptance",
    source_epoch_status = "unknown",
    source_boundary = "runtime_start",
}

local function exact_api(api)
    if type(api) ~= "table" then return false, "initializer_shape" end
    local expected = {}
    for _, name in ipairs(OPERATIONS) do expected[name] = true end
    local count = 0
    for name, value in pairs(api) do
        if not expected[name] or type(value) ~= "function" then return false, "operation_shape" end
        count = count + 1
    end
    if count ~= #OPERATIONS then return false, "operation_missing" end
    local ok, version = pcall(api.abi_version)
    if not ok then return false, "operation_failure" end
    if version ~= 2 then return false, "abi_mismatch" end
    return true
end

local function invoke(self, name, ...)
    if self.closed then return nil, "closed" end
    local ok, code, detail = pcall(self.api[name], self.token, ...)
    if not ok then return nil, "operation_failure" end
    if type(code) ~= "number" or code % 1 ~= 0 then return nil, "result_shape" end
    return code, detail
end

local wrapper = {}
wrapper.__index = wrapper
function wrapper:begin_section(value) return invoke(self, "begin_section", value) end
function wrapper:push_record(value) return invoke(self, "push_record", value) end
function wrapper:finish_section(value) return invoke(self, "finish_section", value) end
function wrapper:fail_section(reason) return invoke(self, "fail_section", reason) end
function wrapper:progress(work) return invoke(self, "progress", work) end
function wrapper:poll_control() return invoke(self, "poll_control") end
function wrapper:reset(reason) return invoke(self, "reset", reason) end
function wrapper:close()
    if self.closed then return 0 end
    local code, detail = invoke(self, "close")
    if code == 0 then
        self.closed = true
        self.guard_state.active = false
    end
    return code, detail
end

local function close_guard(api, token)
    local state = { active = true }
    local guard = newproxy(true)
    getmetatable(guard).__gc = function()
        if not state.active then return end
        state.active = false
        pcall(api.close, token)
    end
    return guard, state
end

function carrier.new(options)
    options = options or {}
    local loadlib = options.loadlib or package.loadlib
    if type(loadlib) ~= "function" then return nil, "loader_unavailable" end
    local initialize, load_error = loadlib(DLL_PATH, INITIALIZER)
    if type(initialize) ~= "function" then return nil, "loader_failure", load_error end
    local ok, api = pcall(initialize)
    if not ok then return nil, "initializer_failure" end
    local valid, reason = exact_api(api)
    if not valid then return nil, reason end
    local open_ok, code, token = pcall(api.open, 2, options.limits or LIMITS, options.source or SOURCE)
    if not open_ok then return nil, "operation_failure" end
    if code ~= 0 or type(token) ~= "string" or not token:match("^%d+:%d+$") then
        return nil, "open_failure", code
    end
    local guard, guard_state = close_guard(api, token)
    return setmetatable({
        api = api, token = token, closed = false,
        close_guard = guard, guard_state = guard_state,
    }, wrapper)
end

carrier.dll_path = DLL_PATH
carrier.initializer = INITIALIZER
return carrier
