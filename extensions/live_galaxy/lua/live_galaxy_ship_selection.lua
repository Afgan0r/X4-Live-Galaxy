local selection = {}
local source = require("live_galaxy.lua.live_galaxy_ship_source")
local core = require("live_galaxy.lua.live_galaxy_ship_collection")
local detail = require("live_galaxy.lua.live_galaxy_ship_details")
local budget_module = require("live_galaxy.lua.live_galaxy_ship_budget")
local profile = require("live_galaxy.lua.live_galaxy_ship_profile")
local DETAIL_KEYS = { "ship_cargo:g0", "ship_crew:g0", "ship_loadout:g0" }
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
    self.collector, self.pending, self.accepted, self.snapshot, self.preflight_checked = nil, nil, nil, nil, nil
    -- A failed start performed no source capture. Never label previous capture
    -- counters with a new request or retain the old collector's failure stage.
    result.capture_metrics = not request and metrics(self) or nil
    return result
end
local function observe_qualification(self, carrier, status)
    local producer_state = type(status) == "table" and status.producer_state or nil
    if producer_state == "awaiting_compatibility" and self.producer_state
        and self.producer_state ~= producer_state then
        discard(self, carrier, "source_boundary_changed")
        self.run_started, self.budget = nil, nil
        self.key, self.revision, self.boundary, self.incarnation = nil, nil, nil, nil
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
            return discard(state, carrier, "source_boundary_changed")
        end
    end
    function adapter:advance(context, carrier, status) return selection.advance(state, context, carrier, status) end
    function adapter:discard(carrier, reason) return discard(state, carrier, reason) end
    return adapter
end
local function start(self, context, carrier, status)
    local key = status.selection
    local now = tonumber(status.monotonic_millis)
    if not now or not status.collection_revision or not status.collection_revision:match("^[1-9]%d*$") then return nil, "clock_unavailable" end
    self.run_started = self.run_started or now
    if now - self.run_started >= self.limits.admission_window_millis then return nil, "admission_window_exhausted" end
    local cached = self.snapshot and self.snapshot[key]
    self.budget = cached and self.snapshot.budget or budget_module.new(self.api, self.limits)
    local clock = { begin_evidence = self.clock.begin_evidence, finish_evidence = self.clock.finish_evidence,
        clock_getter = function() return self.budget:clock(self.clock.clock_getter) end }
    local options = self.options
    if key == "ship_core" then
        self.snapshot = nil
        local core_options = {}; for name, value in pairs(options) do core_options[name] = value end
        core_options.ship_api, core_options.work_budget, core_options.capture_only = self.budget.api, self.budget, true
        self.collector = assert(core.new(core_options, clock))
    else
        if not key:match("^ship_cargo:g%d+$") and not key:match("^ship_crew:g%d+$")
            and not key:match("^ship_loadout:g%d+$") then return nil, "selection_unavailable" end
        local parent = self.accepted
        local ordinal = key:match("^ship_%w+:g(%d+)$")
        ordinal = ordinal and tonumber(ordinal)
        if not parent or not ordinal or ordinal % 1 ~= 0 or ordinal > 65535
            or parent.incarnation ~= status.producer_incarnation or parent.boundary ~= context.source_boundary
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
    local parent = { members = self.collector.identities, cores = self.collector.cores,
        revision = self.revision, boundary = self.boundary, incarnation = self.incarnation,
        budget = self.budget }
    local clock = { begin_evidence = self.clock.begin_evidence, finish_evidence = self.clock.finish_evidence,
        clock_getter = function() return self.budget:clock(self.clock.clock_getter) end }
    for _, key in ipairs(DETAIL_KEYS) do
        local collector = assert(detail.new({ ship_api = self.budget.api, capture_only = true,
            max_inner = self.limits.max_inner_records, max_allocation_bytes = self.limits.max_allocation_bytes,
            source_scope = self.options.source_scope, expected_cores = parent.cores,
            work_budget = self.budget, group = { key = key, members = parent.members,
                owner = self.options.faction_id, core_revision = parent.revision } }, clock))
        local result = collector:tick(context, carrier, status)
        if result.disposition ~= "captured" then return nil, result end
        parent[key] = collector
    end
    local admitted, reason = self.budget:after(carrier)
    if not admitted then return nil, discard(self, carrier, reason) end
    self.budget.capture_duration_millis = self.budget.last - self.budget.callback_start
    self.preflight_checked = true
    self.snapshot = parent
    assert(self.collector:deliver())
    while true do
        local result = self.collector:step(context, carrier, status)
        if result.disposition ~= "collecting" then return result end
    end
end
function selection.advance(self, context, carrier, status)
    context.source_boundary = context.source_boundary or "runtime_start"
    observe_qualification(self, carrier, status)
    local now = tonumber(status.monotonic_millis)
    if self.run_started and (not now or now - self.run_started >= self.limits.admission_window_millis) then
        return discard(self, carrier, "admission_window_exhausted", status)
    end
    if self.incarnation and (self.incarnation ~= status.producer_incarnation or self.boundary ~= context.source_boundary) then
        return discard(self, carrier, "source_boundary_changed", status)
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
    local success, result = pcall(self.collector.tick, self.collector, context, carrier, status)
    if not success then return discard(self, carrier, self.budget.reason or "source_failure") end
    if result.disposition == "captured" and self.key == "ship_core" then
        local capture_success, captured, capture_result = pcall(capture_snapshot, self, context, carrier, status)
        if not capture_success then return discard(self, carrier, self.budget.reason or "source_failure") end
        if captured == nil then
            local refused = discard(self, carrier, capture_result.disposition)
            refused.rejection = capture_result.rejection
            return refused
        end
        result = captured
    end
    if self.preflight_checked then self.preflight_checked = nil
    else
        ok, err = self.budget:after(carrier)
        if not ok then return discard(self, carrier, err) end
    end
    if result.disposition == "sampled" and self.key == "ship_core" then
        -- Transfer owned, already bytewise-ordered membership; do not repeat an
        -- copy/sort after the source collector has completed.
        self.pending = { members = self.collector.identities, cores = self.collector.cores, revision = self.revision,
            boundary = self.boundary, incarnation = self.incarnation }
    elseif result.disposition == "sampled" and self.snapshot then
        self.snapshot[self.key] = nil
    end
    if result.disposition ~= "collecting" and result.disposition ~= "producer_busy" then
        result.capture_metrics = metrics(self)
    end
    return result
end
return selection
