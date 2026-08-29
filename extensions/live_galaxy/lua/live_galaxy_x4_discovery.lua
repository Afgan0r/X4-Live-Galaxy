local discovery = {}
local component_discovery = require("live_galaxy_component_discovery")

local MAX_SECTIONS_PER_CYCLE = 1
local MAX_SECTOR_SCAN = 16
local MAX_FACT_STRING_BYTES = 96
local MAX_SAFE_INTEGER = 9007199254740991
local MAX_PROBE_ATTEMPT_BYTES = 64

local function callable(api, name)
    return type(api) == "table" and type(api[name]) == "function"
end

local function valid_stable_id(value)
    return type(value) == "string"
        and #value <= MAX_FACT_STRING_BYTES
        and value:match("^%d+$") ~= nil
end

local function valid_fact_id(value)
    return type(value) == "string"
        and #value <= MAX_FACT_STRING_BYTES
        and value:match("^[%w_:%-]+$") ~= nil
end

local function valid_fact_value(value)
    return type(value) == "number"
        and value >= 0
        and value <= MAX_SAFE_INTEGER
        and value % 1 == 0
end

local function bounded_limit(limit)
    if type(limit) ~= "number" then return MAX_SECTIONS_PER_CYCLE end
    return math.max(1, math.min(math.floor(limit), MAX_SECTIONS_PER_CYCLE))
end

local function runtime_api()
    local globals = _G
    local probe_config = { enabled = false, attempt_id = "unset" }
    local config_ok, config = pcall(require, "live_galaxy/lua/live_galaxy_trace_config")
    if config_ok and type(config) == "table" then
        probe_config.enabled = config.capability_probe_enabled == true
        probe_config.attempt_id = config.capability_probe_attempt_id
    end
    probe_config.emit = function(vector)
        if type(globals.DebugError) ~= "function" then return false end
        local message = "Live Galaxy discovery: attempt_id=" .. vector.attempt_id
            .. " event=capability_vector metadata_type=" .. vector.metadata_type
            .. " owner_id_validity=" .. vector.owner_id_validity
            .. " sector_capacity=" .. vector.sector_capacity
            .. " first_cargo_ware_limit=" .. vector.first_cargo_ware_limit
        return pcall(globals.DebugError, message)
    end
    return {
        get_clusters = function() return globals.GetClusters(true) end,
        get_sectors = function(_, cluster) return globals.GetSectors(cluster) end,
        stable_id = function(_, sector) return tostring(globals.ConvertIDTo64Bit(sector)) end,
        get_component_data = function(_, sector)
            return globals.GetComponentData(sector, "name", "owner", "macro", "cargo")
        end,
        get_ware_production_limit = function(_, sector, ware)
            return globals.GetWareProductionLimit(sector, ware)
        end,
        get_people_capacity = function(_, sector)
            return globals.C.GetPeopleCapacity(globals.ConvertIDTo64Bit(sector), "", false)
        end,
        get_game_time = function()
            return globals.C.GetCurrentGameTime()
        end,
        capability_probe = probe_config,
    }
end

local function api_available(api)
    return callable(api, "get_clusters")
        and callable(api, "get_sectors")
        and callable(api, "stable_id")
        and callable(api, "get_component_data")
        and callable(api, "get_ware_production_limit")
        and callable(api, "get_people_capacity")
end

local function read_one_sector(api)
    local ok, clusters = pcall(api.get_clusters, api)
    if not ok or type(clusters) ~= "table" or type(clusters[1]) == "nil" then
        return nil, "scope_unavailable"
    end

    local sectors_ok, sectors = pcall(api.get_sectors, api, clusters[1])
    if not sectors_ok or type(sectors) ~= "table" then return nil, "scope_unavailable" end

    local candidates = {}
    for index, sector in ipairs(sectors) do
        if index > MAX_SECTOR_SCAN then return nil, "scope_incomplete" end
        local id_ok, stable_id = pcall(api.stable_id, api, sector)
        if id_ok and valid_stable_id(stable_id) then
            candidates[#candidates + 1] = { sector = sector, stable_id = stable_id }
        end
    end
    table.sort(candidates, function(left, right) return left.stable_id < right.stable_id end)
    if candidates[1] == nil then return nil, "identity_invalid" end
    return candidates[1]
end

local function valid_probe_attempt_id(value)
    return type(value) == "string"
        and value ~= ""
        and #value <= MAX_PROBE_ATTEMPT_BYTES
        and value:match("^[%w_%-]+$") ~= nil
end

local function classify_metadata(call_ok, name, macro)
    if not call_ok then return "call_error" end
    if type(name) ~= "string" or type(macro) ~= "string" then return "wrong_type" end
    if name == "" or macro == "" then return "invalid_value" end
    return "ok"
end

local function classify_owner(owner)
    if type(owner) ~= "string" then return "wrong_type" end
    if not valid_fact_id(owner) then return "invalid_value" end
    return "ok"
end

local function classify_capacity(call_ok, value)
    if not call_ok then return "call_error" end
    if type(value) ~= "number" then return "wrong_type" end
    if not valid_fact_value(value) then return "invalid_value" end
    return "ok"
end

local function classify_ware_limit(cargo, call_ok, value)
    if type(cargo) ~= "table" then return "wrong_type" end
    if next(cargo) == nil then return "not_applicable" end
    if not call_ok then return "call_error" end
    if type(value) ~= "number" then return "wrong_type" end
    if value < 0 or value ~= value then return "invalid_value" end
    return "ok"
end

local function legacy_new(api, capability_probe)
    local adapter = {}
    local probe = capability_probe or api.capability_probe
    local probe_consumed = false

    local function emit_capability_vector(metadata_type, owner_id_validity, sector_capacity, first_cargo_ware_limit)
        if probe_consumed or type(probe) ~= "table" or probe.enabled ~= true
            or not valid_probe_attempt_id(probe.attempt_id) or type(probe.emit) ~= "function" then
            return
        end
        probe_consumed = true
        pcall(probe.emit, {
            attempt_id = probe.attempt_id,
            metadata_type = metadata_type,
            owner_id_validity = owner_id_validity,
            sector_capacity = sector_capacity,
            first_cargo_ware_limit = first_cargo_ware_limit,
        })
    end

    function adapter:list_scope(scope, limit)
        if not api_available(api) then return nil, "adapter_unavailable" end
        if scope ~= "sectors" then return nil, "scope_unsupported" end
        if bounded_limit(limit) ~= MAX_SECTIONS_PER_CYCLE then return nil, "limit_invalid" end

        local candidate, candidate_err = read_one_sector(api)
        if candidate == nil then return nil, candidate_err end
        local metadata_ok, name, owner, macro, cargo = pcall(
            api.get_component_data, api, candidate.sector, "name", "owner", "macro", "cargo"
        )
        if not metadata_ok then return nil, "metadata_unavailable" end

        local capacity_ok, people_capacity = pcall(api.get_people_capacity, api, candidate.sector)
        local ware_capacity_ok = true
        local ware_limit_ok, ware_limit
        if type(cargo) == "table" then
            for ware in pairs(cargo) do
                ware_limit_ok, ware_limit = pcall(
                    api.get_ware_production_limit, api, candidate.sector, ware
                )
                ware_capacity_ok = ware_limit_ok
                    and type(ware_limit) == "number"
                    and ware_limit >= 0
                break
            end
        end
        local game_time
        if callable(api, "get_game_time") then
            local game_time_ok, raw_game_time = pcall(api.get_game_time, api)
            local numeric_game_time = tonumber(raw_game_time)
            if game_time_ok and numeric_game_time ~= nil and numeric_game_time == numeric_game_time
                and numeric_game_time >= 0 and numeric_game_time <= MAX_SAFE_INTEGER then
                game_time = math.floor(numeric_game_time)
            end
        end

        local sector_id = "sector:" .. candidate.stable_id
        local asset_id = "asset:" .. sector_id
        local complete = type(name) == "string" and name ~= ""
            and type(macro) == "string" and macro ~= ""
            and valid_fact_id(sector_id)
            and valid_fact_id(asset_id)
            and valid_fact_id(owner)
            and capacity_ok and valid_fact_value(people_capacity)
            and ware_capacity_ok
        if not complete then
            emit_capability_vector(
                classify_metadata(metadata_ok, name, macro),
                classify_owner(owner),
                classify_capacity(capacity_ok, people_capacity),
                classify_ware_limit(cargo, ware_limit_ok, ware_limit)
            )
            return nil, "facts_unsupported"
        end

        return {
            {
                entity_id = sector_id,
                source = "x4_runtime",
                version = 1,
                quality = "fresh",
                runtime_facts = {
                    source = "x4_runtime",
                    x4_game_time = game_time,
                    quality = "fresh",
                    availability = "available",
                    sectors = { { id = sector_id } },
                    assets = { { id = asset_id, sector_id = sector_id } },
                    capacity = {
                        { id = "capacity:" .. sector_id, asset_id = asset_id, value = people_capacity },
                    },
                    ownership = {
                        { id = "ownership:" .. sector_id, asset_id = asset_id, owner_id = owner },
                    },
                },
            },
        }
    end

    function adapter.read_observation(limit, version)
        local sections, err = adapter:list_scope("sectors", limit)
        if sections == nil then return nil, err end
        if type(version) == "number" and version >= 1 and version % 1 == 0 then
            sections[1].version = version
        end
        return sections[1]
    end

    return adapter
end

local function legacy_new_runtime_adapter()
    return discovery.new(runtime_api())
end

function discovery.new(api, capability_probe)
    if callable(api, "count_stations") then
        return component_discovery.new(api)
    end
    return legacy_new(api, capability_probe)
end

function discovery.new_runtime_adapter()
    return component_discovery.new_runtime_adapter()
end

return discovery
