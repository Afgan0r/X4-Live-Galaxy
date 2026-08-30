package.path = package.path .. ";tests/x4-candidates/lua/?.lua"

local runner = require("live_galaxy_candidate_runner")
local phase_051 = require("phase_051_candidates")
local cases = {}

local function digest_value(payload)
    return string.rep("0", 56) .. string.format("%08x", #payload)
end

local function trusted_services()
    return {
        adapters = {
            ["count-fill-local-contract"] = function()
                return { actual_result = "count-fill-contract-valid", completeness = "complete", work_units = 1 }
            end,
        },
        digest = {
            hash = function(payload) return "sha256", digest_value(payload) end,
            verify = function(payload, algorithm, digest)
                return algorithm == "sha256" and digest == digest_value(payload)
            end,
        },
        watchdog = {
            invoke = function(_, _, _, callback)
                local ok, value = pcall(callback)
                if not ok then return "callback-error" end
                return "completed", value
            end,
        },
    }
end

function cases.candidate_authority_fields_never_execute()
    local called = false
    for _, field in ipairs({ "execute", "validate", "assess", "digest", "watchdog", "registry", "verdict" }) do
        local manifest = phase_051.single_success()
        manifest.candidates[1][field] = function() called = true end
        local output, reason = runner.run(manifest, trusted_services())
        assert(output == nil and reason == "candidate_schema_invalid")
    end
    assert(called == false)
end

function cases.candidate_metatable_authority_never_executes()
    local manifest = phase_051.single_success()
    local called = false
    setmetatable(manifest.candidates[1], {
        __index = function()
            called = true
            return function() called = true end
        end,
    })
    local output, reason = runner.run(manifest, trusted_services())
    assert(output == nil and reason == "candidate_schema_invalid")
    assert(called == false)
end

function cases.direct_native_and_dynamic_authority_fail_closed()
    local called = false
    for _, field in ipairs({ "ffi", "ffi.C", "module", "command", "executable" }) do
        local manifest = phase_051.single_success()
        manifest.candidates[1][field] = function() called = true end
        local output, reason = runner.run(manifest, trusted_services())
        assert(output == nil and reason == "candidate_schema_invalid")
    end
    local manifest = phase_051.single_success()
    manifest.candidates[1].adapter_id = "unregistered-adapter"
    assert(select(2, runner.run(manifest, trusted_services())) == "adapter_unregistered")
    assert(select(2, runner.run_native(manifest, trusted_services())) == "trusted_runtime_attestation_missing")
    assert(called == false)
end

function cases.runner_source_has_no_direct_native_binding()
    local file = assert(io.open("tests/x4-candidates/lua/live_galaxy_candidate_runner.lua", "rb"))
    local source = file:read("*a")
    file:close()
    source = source:gsub("%-%-[^\n]*", "")
    assert(not source:find('require%("ffi"%)'))
    assert(not source:find("ffi%.C"))
    assert(not source:find("loadstring"))
end

return cases
