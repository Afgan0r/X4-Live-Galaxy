use super::*;

pub fn run_full_set(
    root: &Path,
    host: &Path,
    script: &Path,
    data: &Path,
    rest: &[PathBuf],
    database: &Path,
    mut receiver: ProductionObservationSession,
) -> Result<()> {
    let limits = rest.first().ok_or("full-set limits path")?;
    let profile = x4_bridge::HeavyShipLimits::read(limits).ok_or("full-set limits")?;
    receiver
        .configure_heavy(profile.clone())
        .map_err(|e| format!("heavy profile:{e:?}"))?;
    receiver
        .select_ship_factions(["argon", "scaleplate", "xenon", "khaak"])
        .map_err(|e| format!("full-set selection:{e:?}"))?;
    let mut first = Host::start(root, host, script, data, "heavy-ship-full-set")?;
    let first_run = full_set_peer::serve(&mut receiver, &profile, 1, true)?;
    first.finish()?;
    let mut second = Host::start(root, host, script, data, "heavy-ship-full-set-restart")?;
    let second_run = full_set_peer::serve(&mut receiver, &profile, 2, false)?;
    second.finish()?;
    if first_run.incarnation == second_run.incarnation || first_run.blocked_skips != 1 {
        return Err("full-set restart or blocked-scope recovery missing".into());
    }
    let mut completions = first_run.completions;
    for (key, revisions) in second_run.completions {
        completions.entry(key).or_default().extend(revisions);
    }
    drop(receiver);
    full_set_readback::verify(database, &completions)?;
    writeln!(
        std::io::stdout(),
        "PASS heavy-ship-full-set actual_native=true factions=4 zero_ship=xenon blocked_skip=1 rotations=3 restart=true earlier_current=true unknown_blocker=true"
    )?;
    Ok(())
}

pub fn run_detail(
    root: &Path,
    host: &Path,
    script: &Path,
    data: &Path,
    database: &Path,
    mut receiver: ProductionObservationSession,
) -> Result<()> {
    let mut host = Host::start(root, host, script, data, "heavy-ship-detail")?;
    let (_, completions) = detail_peer::serve(&mut receiver)?;
    host.finish()?;
    drop(receiver);
    readback::verify_cargo(database)?;
    crew_readback::verify(database)?;
    let mut receiver = session(database)?;
    // Exact replay reconciles the current revision for each group. Earlier
    // retained revisions were independently read above, not republished.
    for (_, completion) in completions
        .iter()
        .enumerate()
        .filter(|(index, _)| matches!(index, 3 | 4 | 7 | 8 | 11 | 12))
    {
        peer::replay(&mut receiver, completion)?;
    }
    writeln!(
        std::io::stdout(),
        "PASS actual_chain_cargo_crew_loadout independent_history_current=true groups=2 revisions=2..13 replay=current_exact"
    )?;
    Ok(())
}
