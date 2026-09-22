use std::io::Write as _;
mod cargo_readback;
mod crew_readback;
mod detail_peer;
mod full_set_peer;
mod full_set_readback;
mod full_set_recovery;
mod peer;
mod readback;
mod scenarios;
mod wire;

use observation_application::LifecycleLimits;
use observation_domain::SectionKey;
use observation_ingest::{AggregateLimits, CandidateLimits, GenerationLimits};
use observation_persistence::PublicationLimits;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};
use x4_bridge::ProductionObservationSession;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [root, host, script, data, scenario, rest @ ..] = args.as_slice() else {
        return Err("expected root host script data scenario [limits]".into());
    };
    let database = data.join("observations.sqlite3");
    let receiver = if scenario == Path::new("heavy-ship-full-set") {
        session(&database)?
    } else {
        selected_session(&database)?
    };
    if scenario == Path::new("heavy-ship-full-set") {
        return scenarios::run_full_set(root, host, script, data, rest, &database, receiver);
    }
    if scenario == Path::new("heavy-ship-detail") {
        return scenarios::run_detail(root, host, script, data, &database, receiver);
    }
    run_core(root, host, script, data, &database, receiver)
}

fn run_core(
    root: &Path,
    host: &Path,
    script: &Path,
    data: &Path,
    database: &Path,
    mut receiver: ProductionObservationSession,
) -> Result<()> {
    let key = SectionKey::new("ship_core").ok_or("section key")?;
    let mut first = Host::start(root, host, script, data, "heavy-ship-core")?;
    let (identity1, completion1) = peer::serve(&mut receiver, 2, false)?;
    first.finish()?;
    drop(receiver);
    readback::verify(database, &[1, 2], &completion1.1)?;
    let mut receiver = selected_session(database)?;
    if receiver
        .next_revision(&key)
        .map_err(|e| format!("floor:{e:?}"))?
        != 3
    {
        return Err("durable restart floor".into());
    }
    peer::replay(&mut receiver, &completion1)?;
    drop(receiver);
    readback::verify(database, &[1, 2], &completion1.1)?;
    let mut receiver = selected_session(database)?;
    let mut second = Host::start(root, host, script, data, "heavy-ship-restart")?;
    let (identity2, completion2) = peer::serve(&mut receiver, 1, true)?;
    second.finish()?;
    if identity1 == identity2 {
        return Err("producer restart must change process incarnation".into());
    }
    peer::replay(&mut receiver, &completion2)?;
    drop(receiver);
    readback::verify(database, &[1, 2, 3], &completion2.1)?;
    writeln!(
        std::io::stdout(),
        "PASS heavy-ship-core actual_native=true revisions=1,2,3 connections=2 producer_processes=2 batches=24 replay=exact malformed=atomic independent_history_current=true"
    )?;
    Ok(())
}

fn session(database: &Path) -> Result<ProductionObservationSession> {
    let candidate =
        CandidateLimits::new(65536, 16, 16, 65536, 5000, 5000).ok_or("candidate limits")?;
    let aggregate = AggregateLimits::new(2, 131_072, 32, 32, 131_072).ok_or("aggregate limits")?;
    ProductionObservationSession::open(
        database,
        GenerationLimits::bounded(candidate, aggregate),
        publication_limits()?,
        LifecycleLimits::new(4096, 262_144, 262_144, 2).ok_or("lifecycle limits")?,
        4,
    )
    .map_err(|e| format!("session:{e:?}").into())
}

fn selected_session(database: &Path) -> Result<ProductionObservationSession> {
    let mut session = session(database)?;
    let inventory = [observation_domain::FactionOriginEvidence::new(
        "argon",
        observation_domain::FactionOrigin::Vanilla,
        Some(true),
        true,
        "local-fixture",
    )
    .map_err(|error| format!("inventory:{error:?}"))?];
    session
        .accept_faction_census(1, ["argon"], &inventory)
        .map_err(|error| format!("census:{error:?}"))?;
    session
        .select_ship_core("argon")
        .map_err(|e| format!("selection:{e:?}"))?;
    Ok(session)
}

fn publication_limits() -> Result<PublicationLimits> {
    PublicationLimits::new(16, 65536).ok_or_else(|| "publication limits".into())
}

struct Host(Child);
impl Host {
    fn start(
        root: &Path,
        executable: &Path,
        script: &Path,
        data: &Path,
        mode: &str,
    ) -> Result<Self> {
        Ok(Self(
            Command::new(executable)
                .current_dir(root)
                .arg(root)
                .arg(script)
                .arg(data.join(format!("{mode}.lua")))
                .arg(data.join("marker"))
                .arg(mode)
                .spawn()?,
        ))
    }
    fn finish(&mut self) -> Result<()> {
        finish_host(&mut self.0)
    }
}
impl Drop for Host {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn finish_host(child: &mut Child) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(20);
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if Instant::now() >= deadline {
            return Err("Lua host watchdog".into());
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    if status.success() {
        Ok(())
    } else {
        Err(format!("Lua host failed:{status}").into())
    }
}
