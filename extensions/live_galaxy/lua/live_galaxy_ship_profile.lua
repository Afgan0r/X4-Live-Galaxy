local profile = {}
local fields = {
    "complete_message_bytes", "control_message_bytes", "max_candidate_raw_bytes", "max_candidate_records",
    "max_candidate_batches", "max_candidate_work", "max_message_age_millis", "max_message_inactivity_millis",
    "max_candidates", "max_aggregate_bytes", "max_aggregate_records", "max_aggregate_batches",
    "max_aggregate_work", "max_publication_records", "max_publication_content_bytes", "max_pending_bytes",
    "max_total_bytes", "max_lifecycle_work", "max_delivery_attempts", "max_blockers", "reconnect_attempts",
    "reconnect_delay_millis", "availability_interval_millis", "heavy_profile_version", "experimental_profile",
    "group_members", "max_inner_records", "max_allocation_bytes", "max_total_allocation_bytes",
    "max_native_calls", "max_collection_steps", "callback_budget_millis", "heavy_permits", "rate_interval_millis",
    "max_overrun_debt", "admission_window_millis", "freshness_millis", "retained_revisions",
}
function profile.validate(values)
    if type(values) ~= "table" or getmetatable(values) ~= nil then return nil, "invalid_limits" end
    local allowed, copy = {}, {}
    for _, key in ipairs(fields) do
        local value = values[key]
        if type(value) ~= "number" or value <= 0 or value > 9007199254740991 or value % 1 ~= 0 then return nil, "invalid_limits" end
        allowed[key], copy[key] = true, value
    end
    for key in pairs(values) do if not allowed[key] then return nil, "invalid_limits" end end
    local v = copy
    if v.heavy_profile_version ~= 1 or v.experimental_profile ~= 1 or v.group_members ~= 1 or v.heavy_permits ~= 1
        or v.max_delivery_attempts ~= 1 or v.reconnect_attempts ~= 1 or v.control_message_bytes ~= 512
        or v.availability_interval_millis ~= 5000 or v.complete_message_bytes > v.max_candidate_raw_bytes
        or v.retained_revisions < 2
        or v.complete_message_bytes > v.max_pending_bytes or v.max_pending_bytes > v.max_total_bytes
        or v.max_candidate_raw_bytes > v.max_aggregate_bytes or v.max_publication_records > v.max_candidate_records
        or v.max_publication_content_bytes > v.max_total_bytes or v.max_candidate_work > v.max_lifecycle_work
        or v.max_inner_records > v.max_candidate_records or v.max_allocation_bytes > v.max_total_allocation_bytes
        or v.max_total_allocation_bytes > v.max_total_bytes or v.max_native_calls > v.max_collection_steps
        or v.callback_budget_millis >= v.rate_interval_millis or v.max_message_age_millis > v.admission_window_millis
        or v.freshness_millis > v.admission_window_millis then return nil, "invalid_limits" end
    if v.max_candidate_records > v.max_aggregate_records or v.max_candidate_batches > v.max_aggregate_batches
        or v.max_candidate_work > v.max_aggregate_work or v.max_aggregate_bytes > v.max_total_bytes
        or v.max_message_inactivity_millis > v.max_message_age_millis then return nil, "invalid_limits" end
    if v.max_candidate_records ~= v.max_candidate_raw_bytes or v.max_candidate_batches ~= v.max_candidate_raw_bytes
        or v.max_aggregate_records ~= v.max_aggregate_bytes or v.max_aggregate_batches ~= v.max_aggregate_bytes
        or v.max_publication_records ~= v.max_publication_content_bytes
        or v.max_inner_records ~= v.complete_message_bytes then return nil, "invalid_limits" end
    return copy
end
function profile.options(values, faction)
    local v, err = profile.validate(values)
    if not v then return nil, err end
    if type(faction) ~= "string" or not faction:match("^[%w_%-]+$") or #faction > 64
        or faction == "player" or faction == "xenon" or faction == "khaak" then return nil, "selection_unavailable" end
    local scope = "x4:faction:" .. faction .. ":ships"
    return { carrier = { limits = {
        data_message_bytes = v.complete_message_bytes, control_message_bytes = v.control_message_bytes,
        max_records = v.max_candidate_records, max_content_bytes = v.max_candidate_raw_bytes,
        max_canonical_bytes = v.complete_message_bytes, max_batches = v.max_candidate_batches,
        max_work = v.max_candidate_work, max_age_millis = v.max_message_age_millis, pending_slots = 1,
        max_attempts = v.max_delivery_attempts, max_retry_age_millis = v.max_message_age_millis,
        availability_interval_millis = v.availability_interval_millis,
        heavy_profile_version = v.heavy_profile_version, max_inner_records = v.max_inner_records,
    }, source = { source_scope = scope, source_epoch_status = "unknown", source_boundary = "runtime_start" } },
    observation = { profile = "ship_core", faction_id = faction, source_scope = scope, heavy_limits = v,
        ship_limits = { max_records = v.max_candidate_records, max_allocation_bytes = v.max_allocation_bytes,
            max_work = v.max_collection_steps, max_attempts = 1, max_age_millis = v.max_message_age_millis,
            faction_pointer_bytes = 8 } } }
end
return profile
