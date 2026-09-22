local selection = {}
local source, core = require("live_galaxy.lua.live_galaxy_ship_source"), require("live_galaxy.lua.live_galaxy_ship_collection")
local detail, budget_module = require("live_galaxy.lua.live_galaxy_ship_details"), require("live_galaxy.lua.live_galaxy_ship_budget")
local profile = require("live_galaxy.lua.live_galaxy_ship_profile")
local factions = require("live_galaxy.lua.live_galaxy_factions")
local function section(key, fallback)
    if key == "ship_core" then return "core", fallback, 0 end
    local faction = key:match("^ship_core:([%w_%-]+)$")
    if faction then return "core", faction, 0 end
    local family, scoped, group = key:match("^ship_(cargo):([%w_%-]+):g(%d+)$")
    if not family then family, scoped, group = key:match("^ship_(crew):([%w_%-]+):g(%d+)$") end
    if not family then family, scoped, group = key:match("^ship_(loadout):([%w_%-]+):g(%d+)$") end
    if not family then family, group = key:match("^ship_(cargo):g(%d+)$") end
    if not family then family, group = key:match("^ship_(crew):g(%d+)$") end
    if not family then family, group = key:match("^ship_(loadout):g(%d+)$") end
    return family, scoped or fallback, group and tonumber(group)
end
local function detail_keys(faction, scoped)
    local middle = scoped and ":" .. faction or ""
    return { "ship_cargo" .. middle .. ":g0", "ship_crew" .. middle .. ":g0",
        "ship_loadout" .. middle .. ":g0" }
end
local function metrics(self)
    local b = self.budget
    return b and { calls = b.calls, allocation_bytes = b.allocation, steps = b.steps,
        duration_millis = b.capture_duration_millis or (b.last or 0) - (b.started or 0), revision = self.revision,
        max_callback_duration_millis = b.max_callback_duration,
        max_callback_overrun_millis = b.max_callback_overrun,
        source_value_bytes = b.source_value_bytes,
        section = self.key, incarnation = self.incarnation } or nil
end
local function discard(self, carrier, reason, request)
    local result
    if self.collector then
        if self.collector.discard then result = self.collector:discard(carrier, reason)
        else result = self.collector:fail(carrier, reason) end
    end
    result = result or { disposition = reason, rejection = { stage = "selection", condition = reason } }
    if request then
        result.rejection = { stage = "selection", condition = reason, section = request.selection,
            revision = request.collection_revision, run = request.producer_incarnation }
    elseif self.budget then
        result.rejection.operation = self.budget.operation
        result.rejection.condition = self.budget.condition or result.rejection.condition
    end
    self.collector, self.pending, self.accepted, self.snapshot, self.snapshot_capture = nil, nil, nil, nil, nil
    -- A failed start performed no source capture. Never label previous capture
    -- counters with a new request or retain the old collector's failure stage.
    result.capture_metrics = not request and metrics(self) or nil
    return result
end
local function refresh_boundary(self)
    self.roster_history = self.roster
    self.roster = nil
    self.discovery_revision = (self.discovery_revision or 1) + 1
    self.key, self.revision, self.boundary, self.incarnation = nil, nil, nil, nil
    self.run_started, self.budget = nil, nil
end
local function census_record(entry, revision)
    return { profile = "faction_census", faction_id = entry.id,
        discovery_revision = revision, disposition = entry.disposition,
        reason = entry.reason, origin = entry.origin or "",
        source_evidence = entry.evidence or "", mind_candidate = entry.mind_candidate == true }
end
local function drive_census(self, context, carrier, status)
    local txn = self.census_txn
    if not txn then
        local revision = tonumber(status.collection_revision)
        local roster, err = factions.capture(self.api, self.options.faction_inventory or {},
            revision, { max_factions = math.floor(self.limits.max_allocation_bytes / 8),
                max_allocation_bytes = self.limits.max_allocation_bytes, pointer_bytes = 8 })
        if not roster then return discard(self, carrier, err, status) end
        local begin, begin_error = self.clock:begin_evidence()
        local finish, finish_error = self.clock:finish_evidence()
        if not begin or not finish then return discard(self, carrier, begin_error or finish_error, status) end
        begin.section_key, begin.expected_records = "faction_census", #roster.entries
        begin.coverage, begin.consistency, begin.stable_identity = "partial", "observed_count_fill_only", true
        begin.source_epoch_status, begin.source_boundary = context.source_epoch_status or "unknown", context.source_boundary
        finish.coverage, finish.consistency, finish.stable_identity = "partial", "observed_count_fill_only", true
        txn = { roster = roster, begin = begin, finish = finish, index = 1, stage = "begin" }
        self.census_txn = txn
    end
    if txn.stage == "begin" then
        local code = carrier:begin_section(txn.begin)
        if code == -21 then return { disposition = "producer_busy" } end
        if code ~= 0 then return discard(self, carrier, "reservation_failed", status) end
        txn.stage = #txn.roster.entries == 0 and "finish" or "record"
    end
    if txn.stage == "record" then
        local code = carrier:push_record(census_record(txn.roster.entries[txn.index], txn.roster.discovery_revision))
        if code == -21 then return { disposition = "producer_busy" } end
        if code ~= 0 then return discard(self, carrier, "fact_rejected", status) end
        txn.index = txn.index + 1
        if txn.index > #txn.roster.entries then txn.stage = "finish" end
    end
    if txn.stage == "finish" then
        if carrier:finish_section(txn.finish) ~= 0 then return discard(self, carrier, "finish_rejected", status) end
        self.roster, self.discovery_revision, self.census_txn = txn.roster, txn.roster.discovery_revision, nil
        return { disposition = "sampled", discovery_revision = self.discovery_revision,
            unknown_blocker = self.roster.unknown_blocker }
    end
    return { disposition = "collecting" }
end
local function observe_qualification(self, carrier, status)
    local producer_state = type(status) == "table" and status.producer_state or nil
    if producer_state == "awaiting_compatibility" and self.producer_state
        and self.producer_state ~= producer_state then
        discard(self, carrier, "source_boundary_changed")
        refresh_boundary(self)
    end
    if type(producer_state) == "string" then self.producer_state = producer_state end
end
function selection.attach(adapter, options)
    local limits, err = profile.validate(options.heavy_limits)
    if not limits then return nil, err end
    local state = { options = options, limits = limits, clock = adapter,
        api = options.ship_api or source.runtime() }
    function adapter:feedback(context, carrier, status, control)
        observe_qualification(state, carrier, status)
        if control == 5 and state.pending then
            if type(status) ~= "table" or not tonumber(status.monotonic_millis) then
                return discard(state, carrier, "clock_unavailable")
            end
            local pending = state.pending
            if pending.incarnation ~= status.producer_incarnation then return discard(state, carrier, "source_boundary_changed") end
            pending.accepted_at = tonumber(status.monotonic_millis)
            state.accepted, state.pending = pending, nil
        end
        if control == 9 or control == 10 or context.source_boundary ~= (state.boundary or context.source_boundary) then
            local result = discard(state, carrier, "source_boundary_changed")
            refresh_boundary(state)
            return result
        end
    end
    function adapter:advance(context, carrier, status) return selection.advance(state, context, carrier, status) end
    function adapter:discard(carrier, reason) return discard(state, carrier, reason) end
    return adapter
end
local function start(self, context, carrier, status)
    local key = status.selection
    local family, faction = section(key, self.options.faction_id)
    if not family or not faction then return nil, "selection_unavailable" end
    if self.roster and (not self.roster.by_id[faction]
        or self.roster.by_id[faction].disposition ~= "included") then return nil, "selection_unavailable" end
    self.faction = faction
    self.scoped = key ~= "ship_core" and key:match(":g%d+$") and key:match("^ship_%w+:[%w_%-]+:g%d+$") ~= nil
        or key:match("^ship_core:[%w_%-]+$") ~= nil
    local now = tonumber(status.monotonic_millis)
    if not now or not status.collection_revision or not status.collection_revision:match("^[1-9]%d*$") then return nil, "clock_unavailable" end
    self.run_started = self.run_started or now
    if now - self.run_started >= self.limits.admission_window_millis then return nil, "admission_window_exhausted" end
    local cached = self.snapshot and self.snapshot[key]
    self.budget = cached and self.snapshot.budget or budget_module.new(self.api, self.limits)
    local clock = { begin_evidence = self.clock.begin_evidence, finish_evidence = self.clock.finish_evidence,
        clock_getter = function() return self.budget:clock(self.clock.clock_getter) end }
    local options = {}; for name, value in pairs(self.options) do options[name] = value end
    options.faction_id, options.source_scope = faction, "x4:faction:" .. faction .. ":ships"
    if family == "core" then
        self.snapshot = nil
        local core_options = {}; for name, value in pairs(options) do core_options[name] = value end
        core_options.ship_api, core_options.work_budget, core_options.capture_only = self.budget.api, self.budget, true
        core_options.section_key = key
        self.collector = assert(core.new(core_options, clock))
    else
        if not family or family == "core" then return nil, "selection_unavailable" end
        local parent = self.accepted
        local ordinal = select(3, section(key, self.options.faction_id))
        if not parent or not ordinal or ordinal % 1 ~= 0 or ordinal > 65535
            or parent.incarnation ~= status.producer_incarnation or parent.boundary ~= context.source_boundary
            or parent.faction ~= faction
            or now - parent.accepted_at > self.limits.freshness_millis then return nil, "stale_parent" end
        if cached and self.snapshot.revision == parent.revision then
            self.collector = cached
            assert(self.collector:deliver())
            self.key, self.revision, self.boundary, self.incarnation = key, status.collection_revision,
                context.source_boundary, status.producer_incarnation
            return true
        end
        local first = ordinal * self.limits.group_members + 1
        local last = math.min(first + self.limits.group_members - 1, #parent.members)
        if first > last then return nil, "selection_unavailable" end
        local members, expected_cores = {}, {}
        for index = first, last do
            local member = parent.members[index]
            members[#members + 1] = member
            expected_cores[member] = parent.cores[member]
        end
        self.collector = assert(detail.new({ ship_api = self.budget.api,
            max_inner = self.limits.max_inner_records, max_allocation_bytes = self.limits.max_allocation_bytes,
            source_scope = options.source_scope, expected_cores = expected_cores,
            work_budget = self.budget, group = { key = key, members = members,
                owner = options.faction_id, core_revision = parent.revision } }, clock))
    end
    self.key, self.revision, self.boundary, self.incarnation = key, status.collection_revision,
        context.source_boundary, status.producer_incarnation
    return true
end
local function capture_snapshot(self, context, carrier, status)
    if not self.snapshot_capture then
        self.snapshot_capture = { parent = { members = self.collector.identities, cores = self.collector.cores,
            revision = self.revision, boundary = self.boundary, incarnation = self.incarnation,
            faction = self.faction,
            budget = self.budget }, detail_index = 1 }
    end
    local capture = self.snapshot_capture
    local parent = capture.parent
    if #parent.members == 0 then
        self.snapshot, self.snapshot_capture = parent, nil
        assert(self.collector:deliver())
        while true do
            local result = self.collector:step(context, carrier, status)
            if result.disposition ~= "collecting" then return result end
            if self.budget:should_yield(carrier) then return result end
        end
    end
    local clock = { begin_evidence = self.clock.begin_evidence, finish_evidence = self.clock.finish_evidence,
        clock_getter = function() return self.budget:clock(self.clock.clock_getter) end }
    local keys = detail_keys(parent.faction, self.scoped)
    while capture.detail_index <= #keys do
        local key = keys[capture.detail_index]
        if not capture.collector then
            capture.collector = assert(detail.new({ ship_api = self.budget.api, capture_only = true,
                max_inner = self.limits.max_inner_records, max_allocation_bytes = self.limits.max_allocation_bytes,
                source_scope = "x4:faction:" .. parent.faction .. ":ships", expected_cores = parent.cores,
                work_budget = self.budget, group = { key = key, members = parent.members,
                    owner = parent.faction, core_revision = parent.revision } }, clock))
        end
        local result = capture.collector:tick(context, carrier, status)
        if result.disposition == "collecting" then return result end
        if result.disposition ~= "captured" then return nil, result end
        parent[key], capture.collector = capture.collector, nil
        capture.detail_index = capture.detail_index + 1
        if self.budget:should_yield(carrier) then return { disposition = "collecting" } end
    end
    self.snapshot, self.snapshot_capture = parent, nil
    assert(self.collector:deliver())
    while true do
        local result = self.collector:step(context, carrier, status)
        if result.disposition ~= "collecting" then return result end
        if self.budget:should_yield(carrier) then return result end
    end
end
function selection.advance(self, context, carrier, status)
    context.source_boundary = context.source_boundary or "runtime_start"
    observe_qualification(self, carrier, status)
    if status.selection == "faction_census" then
        return drive_census(self, context, carrier, status)
    end
    local now = tonumber(status.monotonic_millis)
    if self.run_started and (not now or now - self.run_started >= self.limits.admission_window_millis) then
        return discard(self, carrier, "admission_window_exhausted", status)
    end
    if self.incarnation and (self.incarnation ~= status.producer_incarnation or self.boundary ~= context.source_boundary) then
        local result = discard(self, carrier, "source_boundary_changed", status)
        refresh_boundary(self)
        return result
    end
    if not self.collector or self.key ~= status.selection or self.collector.stage == "done" then
        local ok, err = start(self, context, carrier, status)
        if not ok then
            if err == "stale_parent" then carrier:fail_section("stale_parent") end
            return discard(self, carrier, err, status)
        end
    end
    local ok, err = self.budget:before(status)
    if not ok then return discard(self, carrier, err) end
    if self.options.faction_inventory and not self.roster then
        local roster, roster_error = factions.capture(self.budget.api,
            self.options.faction_inventory, self.discovery_revision or 1, {
                max_factions = math.floor(self.limits.max_allocation_bytes / 8),
                max_allocation_bytes = self.limits.max_allocation_bytes, pointer_bytes = 8,
            })
        if not roster then return discard(self, carrier, roster_error) end
        self.roster = roster
        self.discovery_revision = roster.discovery_revision
        if not roster.by_id[self.faction]
            or roster.by_id[self.faction].disposition ~= "included" then
            return discard(self, carrier, "selection_unavailable", status)
        end
    end
    local success, result = pcall(self.collector.tick, self.collector, context, carrier, status)
    if not success then return discard(self, carrier, self.budget.reason or "source_failure") end
    local active_family = section(self.key, self.options.faction_id)
    if active_family == "core" and (result.disposition == "captured" or self.snapshot_capture) then
        local capture_success, captured, capture_result = pcall(capture_snapshot, self, context, carrier, status)
        if not capture_success then return discard(self, carrier, self.budget.reason or "source_failure") end
        if captured == nil then
            local refused = discard(self, carrier, capture_result.disposition)
            refused.rejection = capture_result.rejection
            return refused
        end
        result = captured
    end
    ok, err = self.budget:after(carrier)
    if not ok then return discard(self, carrier, err) end
    if result.disposition == "sampled" and active_family == "core" then
        -- Transfer owned, already bytewise-ordered membership; do not repeat an
        -- copy/sort after the source collector has completed.
        self.pending = { members = self.collector.identities, cores = self.collector.cores, revision = self.revision,
            boundary = self.boundary, incarnation = self.incarnation, faction = self.faction }
    elseif result.disposition == "sampled" and self.snapshot then
        self.snapshot[self.key] = nil
    end
    if result.disposition ~= "collecting" and result.disposition ~= "producer_busy" then
        result.capture_metrics = metrics(self)
    end
    if self.roster then result.discovery_revision = self.roster.discovery_revision end
    return result
end
return selection
