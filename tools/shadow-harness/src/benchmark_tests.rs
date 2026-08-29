use super::{BenchmarkFixture, BenchmarkProcess, PayloadProcess};
use mind_orchestration::ProviderFailure;

const ZYA: &str = r#"{"id":"one","track":"benchmark","faction":"ZYA","frozen_snapshot_identity":"s1","current_snapshot_identity":"s1","visible_fact_ids":["ZYA:military:fleet"],"allowed_capabilities":["DefenseAndMilitaryStrategy"],"policy_version":"p1","prompt_package_hash":"h1","provider_id":"codex","model_id":"model","generation_settings":"t0","prompt_payload":"one","expected_trajectory":"zero-cycle","expected_disposition":"accept","observation_identity":1,"max_candidate_bytes":8,"max_trade_offs":1,"frames":["{\"type\":\"observation\",\"scope\":\"ZYA\",\"entity_id\":\"ZYA:military:fleet\",\"observed_at_unix_millis\":1,\"version\":1,\"quality\":\"fresh\",\"content\":\"own\"}"]}"#;
const ARG: &str = r#"{"id":"two","track":"benchmark","faction":"ARG","frozen_snapshot_identity":"s2","current_snapshot_identity":"s2","visible_fact_ids":["ARG:economy:station"],"allowed_capabilities":["EconomyAndLogistics"],"policy_version":"p2","prompt_package_hash":"h2","provider_id":"codex","model_id":"model","generation_settings":"t1","prompt_payload":"two","expected_trajectory":"direct-agreement","expected_disposition":"accept","observation_identity":2,"max_candidate_bytes":9,"max_trade_offs":2,"frames":["{\"type\":\"observation\",\"scope\":\"ARG\",\"entity_id\":\"ARG:economy:station\",\"observed_at_unix_millis\":1,\"version\":1,\"quality\":\"fresh\",\"content\":\"own\"}"]}"#;

#[derive(Default)]
struct Capture(Vec<String>);
impl BenchmarkProcess for Capture {
    fn invoke(&mut self, input: &str) -> Result<Vec<u8>, ProviderFailure> {
        self.0.push(input.into());
        Ok(Vec::new())
    }
}

#[test]
fn fixtures_build_distinct_typed_requests_and_process_payloads() {
    let first = BenchmarkFixture::parse(ZYA.as_bytes()).unwrap();
    let second = BenchmarkFixture::parse(ARG.as_bytes()).unwrap();
    assert_ne!(first.request().unwrap(), second.request().unwrap());
    let mut capture = Capture::default();
    PayloadProcess::new(&mut capture, first.canonical_payload())
        .invoke("ignored")
        .unwrap();
    PayloadProcess::new(&mut capture, second.canonical_payload())
        .invoke("ignored")
        .unwrap();
    assert_ne!(capture.0[0], capture.0[1]);
}

#[test]
fn malformed_or_unsupported_fixture_fails_closed() {
    assert!(BenchmarkFixture::parse(br#"{"id":"bad","track":"benchmark"}"#).is_err());
    assert!(
        BenchmarkFixture::parse(ZYA.replace("DefenseAndMilitaryStrategy", "Nope").as_bytes())
            .is_err()
    );
}
