local discovery = {}
local PRE_RUN_OWNER_MEMBER_EVIDENCE = 129
local MAX_FACT_STRING_BYTES = 96
local MAX_SAFE_INTEGER = 9007199254740991
local FACT_DIAGNOSTIC_CLASSES = {
    asset_identity_invalid = true, capacity_invalid = true, capacity_unavailable = true,
    metadata_unavailable = true, owner_invalid = true, owner_scope_empty = true,
    owner_scope_mismatch = true, sector_invalid = true,
}

local function callable(api, name)
    return type(api) == "table" and type(api[name]) == "function"
end
local function valid_integer(value)
    return type(value) == "number" and value >= 0 and value <= MAX_SAFE_INTEGER
        and value == value and value % 1 == 0
end
local function valid_positive_integer(value) return valid_integer(value) and value > 0 end
local function checked_multiply(left, right)
    if not valid_positive_integer(left) or not valid_positive_integer(right)
        or left > math.floor(MAX_SAFE_INTEGER / right) then return nil end
    return left * right
end

function discovery.derive_pre_run_native_policy(universe_id_bytes)
    local allocation_bytes = checked_multiply(PRE_RUN_OWNER_MEMBER_EVIDENCE, universe_id_bytes)
    local member_work = checked_multiply(4, PRE_RUN_OWNER_MEMBER_EVIDENCE)
    if allocation_bytes == nil or member_work == nil or member_work > MAX_SAFE_INTEGER - 2 then return nil end
    return { max_allocation_bytes = allocation_bytes, max_work_units = 2 + member_work }
end

local function validated_policy(api)
    if type(api) ~= "table" or not valid_positive_integer(api.universe_id_bytes)
        or type(api.native_policy) ~= "table" or getmetatable(api.native_policy) ~= nil then return nil end
    local fields = 0
    for key in pairs(api.native_policy) do
        if key ~= "max_allocation_bytes" and key ~= "max_work_units" then return nil end
        fields = fields + 1
    end
    if fields ~= 2 or not valid_positive_integer(api.native_policy.max_allocation_bytes)
        or not valid_positive_integer(api.native_policy.max_work_units) then return nil end
    return { universe_id_bytes = api.universe_id_bytes,
        max_allocation_bytes = api.native_policy.max_allocation_bytes,
        max_work_units = api.native_policy.max_work_units }
end

local function valid_id(value)
    return type(value) == "string" and #value > 0 and #value <= MAX_FACT_STRING_BYTES
        and value:match("^[%w_:%-]+$") ~= nil
end

local function runtime_api()
    local globals = _G
    local ffi = require("ffi")
    local C = ffi.C
    local universe_id_bytes = tonumber(ffi.sizeof("UniverseID"))
    return {
        count_stations = function(_, faction_id) return C.GetNumAllFactionStations(faction_id) end,
        new_buffer = function(_, count) return ffi.new("UniverseID[?]", count) end,
        fill_stations = function(_, buffer, count, faction_id)
            return C.GetAllFactionStations(buffer, count, faction_id)
        end,
        to_component = function(_, raw_id) return globals.ConvertStringToLuaID(tostring(raw_id)) end,
        to_component64 = function(_, component) return globals.ConvertIDTo64Bit(component) end,
        get_component_data = function(_, component)
            return globals.GetComponentData(component, "owner", "sector")
        end,
        get_people_capacity = function(_, component64) return C.GetPeopleCapacity(component64, "", false) end,
        canonical_owner_id = function(_, owner_id)
            if type(owner_id) ~= "string" or owner_id == "" then return nil end
            if owner_id:sub(1, 8) == "faction:" then return owner_id end
            return "faction:" .. owner_id
        end,
        faction_id = "faction:argon", native_faction_id = "argon",
        universe_id_bytes = universe_id_bytes,
        native_policy = discovery.derive_pre_run_native_policy(universe_id_bytes),
    }
end

local function api_available(api)
    return callable(api, "count_stations") and callable(api, "new_buffer")
        and callable(api, "fill_stations") and callable(api, "to_component")
        and callable(api, "to_component64") and callable(api, "get_component_data")
        and callable(api, "get_people_capacity")
end

local function section_from_candidate(candidate)
    return { entity_id = candidate.asset_id, source = "x4_runtime", version = 1, quality = "fresh",
        runtime_facts = { source = "x4_runtime", quality = "fresh", availability = "available",
            sectors = { { id = candidate.sector_id } },
            assets = { { id = candidate.asset_id, sector_id = candidate.sector_id } },
            capacity = { { id = "capacity:station:" .. candidate.stable_id,
                asset_id = candidate.asset_id, value = candidate.people_capacity } },
            ownership = { { id = "ownership:station:" .. candidate.stable_id,
                asset_id = candidate.asset_id, owner_id = candidate.owner_id } } } }
end

function discovery.new(api)
    local adapter, last_diagnostic_class = {}, nil
    local native_policy = validated_policy(api)
    local function unsupported(class)
        last_diagnostic_class = FACT_DIAGNOSTIC_CLASSES[class] and class or nil
        return nil, "facts_unsupported"
    end
    function adapter.diagnostic_class() return last_diagnostic_class end
    function adapter.read_observations(_, version)
        last_diagnostic_class = nil
        if native_policy == nil or not api_available(api) then return nil, "enumeration_unavailable" end
        local work_remaining = native_policy.max_work_units
        local function consume_work()
            if work_remaining < 1 then return false end
            work_remaining = work_remaining - 1
            return true
        end
        local native_faction_id = api.native_faction_id or api.faction_id
        if not consume_work() then return nil, "enumeration_overflow" end
        local count_ok, raw_count = pcall(api.count_stations, api, native_faction_id)
        local count = tonumber(raw_count)
        if not count_ok or not valid_integer(count) then return nil, "enumeration_unavailable" end
        if count == 0 then return unsupported("owner_scope_empty") end
        local allocation_bytes = checked_multiply(count, native_policy.universe_id_bytes)
        local member_work = checked_multiply(4, count)
        if allocation_bytes == nil or allocation_bytes > native_policy.max_allocation_bytes
            or member_work == nil or member_work > MAX_SAFE_INTEGER - 1
            or work_remaining < 1 + member_work then return nil, "enumeration_overflow" end
        local allocation_ok, buffer = pcall(api.new_buffer, api, count)
        if not allocation_ok or buffer == nil then return nil, "enumeration_unavailable" end
        if not consume_work() then return nil, "enumeration_overflow" end
        local fill_ok, raw_filled = pcall(api.fill_stations, api, buffer, count, native_faction_id)
        local filled = tonumber(raw_filled)
        if not fill_ok or not valid_integer(filled) or filled ~= count then return nil, "enumeration_incomplete" end
        local candidates, stable_ids = {}, {}
        for index = 0, count - 1 do
            if not consume_work() then return nil, "enumeration_overflow" end
            local component_ok, component = pcall(api.to_component, api, buffer[index])
            if not component_ok or component == nil or component == false then return nil, "identity_invalid" end
            if not consume_work() then return nil, "enumeration_overflow" end
            local id_ok, component64 = pcall(api.to_component64, api, component)
            local stable_id = id_ok and tostring(component64) or nil
            if not id_ok or component64 == nil or component64 == false
                or not valid_id(stable_id) or stable_ids[stable_id] then return nil, "identity_invalid" end
            stable_ids[stable_id] = true
            candidates[#candidates + 1] = { component = component, component64 = component64, stable_id = stable_id }
        end
        table.sort(candidates, function(left, right) return left.stable_id < right.stable_id end)
        for _, candidate in ipairs(candidates) do
            if not consume_work() then return nil, "enumeration_overflow" end
            local metadata_ok, raw_owner_id, sector_id = pcall(api.get_component_data, api, candidate.component)
            if not consume_work() then return nil, "enumeration_overflow" end
            local capacity_ok, people_capacity = pcall(api.get_people_capacity, api, candidate.component64)
            local owner_id = raw_owner_id
            if metadata_ok and callable(api, "canonical_owner_id") then
                local canonical_ok, canonical_owner_id = pcall(api.canonical_owner_id, api, raw_owner_id)
                if not canonical_ok then return unsupported("owner_invalid") end
                owner_id = canonical_owner_id
            end
            candidate.asset_id = "asset:station:" .. candidate.stable_id
            candidate.owner_id, candidate.sector_id, candidate.people_capacity = owner_id, sector_id, people_capacity
            if not metadata_ok then return unsupported("metadata_unavailable") end
            if not capacity_ok then return unsupported("capacity_unavailable") end
            if not valid_id(candidate.asset_id) then return unsupported("asset_identity_invalid") end
            if not valid_id(owner_id) then return unsupported("owner_invalid") end
            if owner_id ~= api.faction_id then return unsupported("owner_scope_mismatch") end
            if not valid_id(sector_id) then return unsupported("sector_invalid") end
            if not valid_integer(people_capacity) then return unsupported("capacity_invalid") end
        end
        local observations = {}
        for _, candidate in ipairs(candidates) do
            local observation = section_from_candidate(candidate)
            if valid_integer(version) and version >= 1 then observation.version = version end
            observations[#observations + 1] = observation
        end
        return observations
    end
    function adapter.read_observation(_, version)
        local observations, err = adapter.read_observations(adapter, version)
        if observations == nil then return nil, err end
        if #observations ~= 1 then return nil, "observation_batch_required" end
        return observations[1]
    end
    return adapter
end

function discovery.new_runtime_adapter() return discovery.new(runtime_api()) end
return discovery
