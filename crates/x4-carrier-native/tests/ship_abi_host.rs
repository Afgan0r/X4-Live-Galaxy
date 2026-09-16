#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "real ABI fixtures fail immediately when their contract is violated"
)]

use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
fn prepared_host(repo: &Path) -> PathBuf {
    if let Some(path) = std::env::var_os("X4_CARRIER_LUA_HOST") {
        return path.into();
    }
    let common = Command::new("git")
        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .current_dir(repo)
        .output()
        .expect("git common directory");
    assert!(common.status.success());
    let common = PathBuf::from(String::from_utf8(common.stdout).unwrap().trim());
    let cache = common
        .parent()
        .unwrap()
        .join("tools/.cache/carrier-b-local");
    let mut hosts: Vec<_> = std::fs::read_dir(cache)
        .expect("prepare existing carrier-b-local host first")
        .map(|entry| entry.unwrap().path().join("carrier-b-host.exe"))
        .filter(|path| path.is_file())
        .collect();
    hosts.sort();
    hosts
        .pop()
        .expect("prepared carrier-b-local host required; no install in tests")
}

pub(super) struct Host {
    child: Child,
    run: PathBuf,
}
impl Host {
    pub(super) fn start(mode: &str) -> Self {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let run = repo
            .join("tools/.cache/ship-abi")
            .join(format!("{}-{mode}", std::process::id()));
        std::fs::create_dir_all(run.join("extensions/live_galaxy")).unwrap();
        let dll = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("x4_carrier_native.dll");
        std::fs::copy(
            dll,
            run.join("extensions/live_galaxy/ui_c_library_live_galaxy_carrier_64.txt"),
        )
        .unwrap();
        let child = Command::new(prepared_host(repo))
            .env("X4_SHIP_ABI_MARKER", run.join("marker.txt"))
            .env(
                "X4_SHIP_ABI_DLL",
                run.join("extensions/live_galaxy/ui_c_library_live_galaxy_carrier_64.txt"),
            )
            .current_dir(&run)
            .arg(repo)
            .arg(repo.join("extensions/live_galaxy/tests/ship_abi_operations.lua"))
            .arg(run.join("result.lua"))
            .arg(run.join("marker.txt"))
            .arg(mode)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        Self { child, run }
    }
    pub(super) fn wait_replacement(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !self.run.join("marker.txt").is_file() {
            if self.child.try_wait().unwrap().is_some() {
                self.finish();
            }
            assert!(Instant::now() < deadline, "replacement open watchdog");
            std::thread::yield_now();
        }
    }
    pub(super) fn release_peer(&self) {
        std::fs::write(self.run.join("marker.txt.released"), "released").unwrap();
    }
    pub(super) fn finish(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while self.child.try_wait().unwrap().is_none() {
            assert!(Instant::now() < deadline, "Lua ABI host watchdog");
            std::thread::yield_now();
        }
        let mut error = String::new();
        use std::io::Read as _;
        self.child
            .stderr
            .as_mut()
            .unwrap()
            .read_to_string(&mut error)
            .unwrap();
        self.child
            .stdout
            .as_mut()
            .unwrap()
            .read_to_string(&mut error)
            .unwrap();
        assert!(
            self.child.try_wait().unwrap().unwrap().success(),
            "Lua operations failed: {error}"
        );
        assert_eq!(
            std::fs::read_to_string(self.run.join("result.lua")).unwrap(),
            "SHIP_ABI_PASS"
        );
    }
}
impl Drop for Host {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if std::thread::panicking() {
            use std::io::Read as _;
            let mut output = String::new();
            if let Some(stderr) = self.child.stderr.as_mut() {
                let _ = stderr.read_to_string(&mut output);
            }
            eprintln!("Lua host diagnostic: {output}");
        }
        // Only this owned ignored fixture directory is removed; never any repository source.
        let _ = std::fs::remove_dir_all(&self.run);
    }
}
