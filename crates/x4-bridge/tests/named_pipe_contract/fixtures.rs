pub const HELLO: &str = r#"{"type":"hello","protocol_major":1,"game_build":"live-galaxy-x4-build-2","capabilities":["live-galaxy-observation-v2"],"generation":1}"#;
pub const OBSERVATION: &str = r#"{"type":"observation","scope":"runtime:sectors","entity_id":"sector:argon_prime","version":1,"quality":"fresh","runtime_facts":{"r":"x4_runtime","g":42,"q":"fresh","a":"available","s":[{"i":"sector:argon_prime"}],"x":[{"i":"asset:ship:1","p":"sector:argon_prime"}],"c":[{"i":"capacity:ship:storage","p":"asset:ship:1","v":42}],"o":[{"i":"ownership:ship:1","p":"asset:ship:1","n":"faction:argon"}]},"generation":1,"sequence":3}"#;
pub const MARKER: &str = r#"{"type":"complete_marker","scope":"runtime:sectors","version":1,"generation":1,"sequence":4}"#;
pub const HEARTBEAT: &str =
    r#"{"type":"heartbeat","scope":"runtime:sectors","version":1,"generation":1,"sequence":1}"#;
pub const HEALTH: &str = r#"{"type":"runtime_health","scope":"runtime:sectors","version":1,"status":"available","generation":1,"sequence":2}"#;
pub const MARKER_CONFIRMATION: &str =
    r#"{"type":"heartbeat","scope":"runtime:sectors","version":1,"generation":1,"sequence":5}"#;

pub fn observation(version: u64, sequence: u64) -> String {
    format!(
        r#"{{"type":"observation","scope":"runtime:sectors","entity_id":"sector:argon_prime","version":{version},"quality":"fresh","runtime_facts":{{"r":"x4_runtime","g":42,"q":"fresh","a":"available","s":[{{"i":"sector:argon_prime"}}],"x":[{{"i":"asset:ship:1","p":"sector:argon_prime"}}],"c":[{{"i":"capacity:ship:storage","p":"asset:ship:1","v":42}}],"o":[{{"i":"ownership:ship:1","p":"asset:ship:1","n":"faction:argon"}}]}},"generation":1,"sequence":{sequence}}}"#
    )
}

pub fn station_observation(station: u64, sector: &str, sequence: u64) -> String {
    format!(
        r#"{{"type":"observation","scope":"runtime:sectors","entity_id":"asset:station:{station}","version":2,"quality":"fresh","runtime_facts":{{"r":"x4_runtime","q":"fresh","a":"available","s":[{{"i":"{sector}"}}],"x":[{{"i":"asset:station:{station}","p":"{sector}"}}],"c":[{{"i":"capacity:station:{station}","p":"asset:station:{station}","v":42}}],"o":[{{"i":"ownership:station:{station}","p":"asset:station:{station}","n":"faction:argon"}}]}},"generation":1,"sequence":{sequence}}}"#
    )
}
