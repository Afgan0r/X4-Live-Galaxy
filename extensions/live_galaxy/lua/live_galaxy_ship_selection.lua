local selection = {}
local source = require("live_galaxy.lua.live_galaxy_ship_source")
local core = require("live_galaxy.lua.live_galaxy_ship_collection")
local detail = require("live_galaxy.lua.live_galaxy_ship_details")
local budget_module = require("live_galaxy.lua.live_galaxy_ship_budget")
local profile = require("live_galaxy.lua.live_galaxy_ship_profile")
local function metrics(self)
    local b = self.budget
    return b and { calls = b.calls, allocation_bytes = b.allocation, steps = b.steps,
        duration_millis = (b.last or 0) - (b.started or 0), revision = self.revision,
        max_callback_duration_millis = b.max_callback_duration,
        max_callback_overrun_millis = b.max_callback_overrun,
        source_value_bytes = b.source_value_bytes,
        section = self.key, incarnation = self.incarnation } or nil
end
local function discard(self, carrier, reason)
    if self.collector then
        if self.collector.discard then self.collector:discard(carrier, reason)
        else self.collector:fail(carrier, reason) end
    end
    self.collector, self.pending, self.accepted = nil, nil, nil
    return { disposition = reason, capture_metrics = metrics(self) }
end
function selection.attach(adapter, options)
    local limits, err = profile.validate(options.heavy_limits)
    if not limits then return nil, err end
    local state = { options = options, limits = limits, clock = adapter,
        api = options.ship_api or source.runtime() }
    function adapter:feedback(context, carrier, status, control)
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
    self.budget = budget_module.new(self.api, self.limits)
    local clock = { begin_evidence = self.clock.begin_evidence, finish_evidence = self.clock.finish_evidence,
        clock_getter = function() return self.budget:clock(self.clock.clock_getter) end }
    local options = self.options
    if key == "ship_core" then
        local core_options = {}; for name, value in pairs(options) do core_options[name] = value end
        core_options.ship_api, core_options.work_budget = self.budget.api, self.budget
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
        local member = parent.members[ordinal + 1]
        if not member then return nil, "selection_unavailable" end
        self.collector = assert(detail.new({ ship_api = self.budget.api,
            max_inner = self.limits.max_inner_records, max_allocation_bytes = self.limits.max_allocation_bytes,
            source_scope = options.source_scope, expected_core = parent.cores[member],
            work_budget = self.budget, group = { key = key, members = { member },
                owner = options.faction_id, core_revision = parent.revision } }, clock))
    end
    self.key, self.revision, self.boundary, self.incarnation = key, status.collection_revision,
        context.source_boundary, status.producer_incarnation
    return true
end
function selection.advance(self, context, carrier, status)
    context.source_boundary = context.source_boundary or "runtime_start"
    local now = tonumber(status.monotonic_millis)
    if self.run_started and (not now or now - self.run_started >= self.limits.admission_window_millis) then
        return discard(self, carrier, "admission_window_exhausted")
    end
    if self.incarnation and (self.incarnation ~= status.producer_incarnation or self.boundary ~= context.source_boundary) then
        return discard(self, carrier, "source_boundary_changed")
    end
    if not self.collector or self.key ~= status.selection or self.collector.stage == "done" then
        local ok, err = start(self, context, carrier, status)
        if not ok then
            if err == "stale_parent" then carrier:fail_section("stale_parent") end
            return discard(self, carrier, err)
        end
    end
    local ok, err = self.budget:before(status)
    if not ok then return discard(self, carrier, err) end
    local success, result = pcall(self.collector.tick, self.collector, context, carrier, status)
    if not success then return discard(self, carrier, self.budget.reason or "source_failure") end
    ok, err = self.budget:after(carrier)
    if not ok then return discard(self, carrier, err) end
    if result.disposition == "sampled" and self.key == "ship_core" then
        -- Transfer owned, already bytewise-ordered membership; do not repeat an
        -- copy/sort after the source collector has completed.
        self.pending = { members = self.collector.identities, cores = self.collector.cores, revision = self.revision,
            boundary = self.boundary, incarnation = self.incarnation }
    end
    if result.disposition == "sampled" then result.capture_metrics = metrics(self) end
    return result
end
return selection
