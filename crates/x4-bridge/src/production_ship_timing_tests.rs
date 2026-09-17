#[test]
fn accelerated_game_capture_does_not_inflate_receiver_wall_cost() {
    let receiver_start = 10_000;
    let receiver_commit = 15_000;
    let content = "capture_start=0\ncapture_end=30000";
    let observed = super::capture_duration(content).expect("valid game capture");
    assert!(
        !crate::production_ship_cursor::refresh_required(receiver_commit, 0, 30_000, observed, 25),
        "five receiver-wall seconds must not become thirty game seconds and force core refresh"
    );
    assert_eq!(observed, receiver_commit - receiver_start);
}
