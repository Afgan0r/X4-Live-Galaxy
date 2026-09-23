pub fn scope(section: &str) -> String {
    observation_domain::ShipSectionIdentity::parse(section)
        .and_then(|identity| identity.faction().map(str::to_owned))
        .map_or_else(String::new, |faction| format!("x4:faction:{faction}:ships"))
}

pub fn attempt(message: &str) -> u64 {
    message
        .strip_prefix("attempt-")
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}
