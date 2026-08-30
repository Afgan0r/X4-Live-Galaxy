local function initialize()
    _G.live_galaxy_candidate_build = {
        build_id = "{{BUILD_ID}}",
        group_id = "{{GROUP_ID}}",
        developer_only = true,
        execution_ready_local_process = true,
        x4_execution = "human-triggered-pending",
    }
end

Register_OnLoad_Init(initialize, "live_galaxy_candidate_{{SAFE_GROUP_ID}}")
