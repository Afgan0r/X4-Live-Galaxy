local candidates = {}

function candidates.single_success()
    return {
        schema_version = "runtime-evidence.v1",
        run_id = "run-p051-local-contract-001",
        game_version = "X4 9.00",
        mod_list = { "live_galaxy@0.1.0", "sn_mod_support_apis@195" },
        scenario_id = "phase-051-local-contract",
        prior_dossier_id = "contract-fixture-complete",
        prior_dossier_digest = string.rep("a", 64),
        build_id = "candidate-build-local-001",
        build_profile_digest = string.rep("b", 64),
        bounds = {
            max_steps = 3,
            max_work_units_per_step = 8,
            max_candidate_rows = 3,
            max_total_bytes = 32768,
            max_output_rows = 3,
        },
        candidates = {
            {
                id = "p051-native-count-fill-runtime",
                adapter_id = "count-fill-local-contract",
                source = "05.2-RESEARCH.md#phase-05.1-candidate-matrix",
                expected_result = "count-fill-contract-valid",
                bounds = { max_work_units = 8 },
            },
        },
    }
end

return candidates
