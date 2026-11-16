//! Disposable actual OS processes, deliberately outside application units.
use super::*;
use serde_json::{Value, json};
use std::{path::Path, process::Stdio};

fn require(ok: bool, message: &str) -> io::Result<()> {
    if ok {
        Ok(())
    } else {
        Err(io::Error::other(message))
    }
}
fn command(fixture: &Path, mode: &str, root: &Path) -> Command {
    let mut command = Command::new("/usr/bin/python3");
    command
        .arg(fixture)
        .arg(mode)
        .arg(root)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .process_group(0);
    command
}
async fn marker(root: &Path, mode: &str) -> io::Result<i32> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Ok(value) = std::fs::read_to_string(root.join(format!("{mode}.pid"))) {
            return value.parse().map_err(io::Error::other);
        }
        require(Instant::now() < deadline, "fixture marker timed out")?;
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
fn alive(pid: i32) -> bool {
    // SAFETY: signal zero observes a disposable fixture only; no signaling authority is inferred.
    unsafe { libc::kill(pid, 0) == 0 }
}
pub(crate) async fn parent_loss_owner(root: &Path, fixture: &Path) -> io::Result<()> {
    let owner = Processes::new();
    let _child = owner.spawn(command(fixture, "parent-loss", root))?;
    marker(root, "parent-loss").await?;
    // Deliberate native-parent failure: skips all destructors/cleanup. Fixture exits via a control file.
    std::process::exit(0);
}
pub(crate) async fn run(root: &Path, fixture: &Path) -> io::Result<Value> {
    use std::os::unix::fs::DirBuilderExt;
    std::fs::DirBuilder::new().mode(0o700).create(root)?;
    let root = std::fs::canonicalize(root)?;
    let fixture = std::fs::canonicalize(fixture)?;
    let mut checks = Vec::new();
    let owner = Processes::new();
    for (mode, input, expected) in [
        ("finite", None, Some((true, b"partial\n".as_slice()))),
        (
            "nonzero",
            None,
            Some((false, b"exact failure\n".as_slice())),
        ),
        (
            "stdin",
            Some(vec![b'i'; 1024]),
            Some((true, b"1024".as_slice())),
        ),
        ("saturate", None, None),
    ] {
        let mut cmd = std::process::Command::new("/usr/bin/python3");
        use std::os::unix::process::CommandExt;
        cmd.arg(&fixture)
            .arg(mode)
            .arg(&root)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .stdin(Stdio::null())
            .stderr(Stdio::piped())
            .process_group(0);
        cmd.stdout(Stdio::piped());
        let (processes, data) = (owner.clone(), input);
        let outcome = tokio::task::spawn_blocking(move || processes.git(cmd, data.as_deref()))
            .await
            .map_err(io::Error::other)?;
        match expected {
            Some((success, bytes)) => require(
                outcome.is_ok_and(|actual| actual == (success, bytes.to_vec())),
                mode,
            )?,
            None => require(
                matches!(outcome, Err(GitFailure::Effect(error)) if error.kind() == io::ErrorKind::InvalidData),
                "saturation must reject oversized output",
            )?,
        }
        checks.push(json!({"scenario":mode,"passed":true}));
    }
    let mut linger = std::process::Command::new("/usr/bin/python3");
    use std::os::unix::process::CommandExt;
    linger
        .arg(&fixture)
        .arg("linger-zero")
        .arg(&root)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0);
    let processes = owner.clone();
    let outcome = tokio::task::spawn_blocking(move || processes.git(linger, None))
        .await
        .map_err(io::Error::other)?;
    require(
        matches!(outcome, Err(GitFailure::Effect(error)) if error.kind() == io::ErrorKind::TimedOut),
        "zero-exit cleanup must retain the natural-exit timeout",
    )?;
    checks.push(json!({"scenario":"natural-exit-timeout-with-zero-term-exit", "passed":true}));
    for abandoned in [false, true] {
        let pid_file = root.join("descendant.pid");
        if pid_file.exists() {
            std::fs::remove_file(&pid_file)?;
        }
        let child = owner.spawn(command(&fixture, "descendant", &root))?;
        let pid = marker(&root, "descendant").await?;
        require(alive(pid), "owned descendant did not start")?;
        if abandoned {
            drop(child);
        } else {
            child.finish(&|| {}).await?;
        }
        // For abandonment, completion is verified by the owner drain below.
        checks.push(json!({"scenario":"leader-exit-with-descendant", "abandoned_waiter":abandoned,"descendant_pid":pid,"passed":true}));
    }
    owner.finish().await?;
    checks.push(json!({"scenario":"owned-process-drain","passed":true}));
    let escaped_owner = Processes::new();
    let child = escaped_owner.spawn(command(&fixture, "escaped", &root))?;
    let escaped = marker(&root, "escaped").await?;
    let release = root.join("escaped.release");
    let release_task = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(3)).await;
        std::fs::write(release, b"voluntary fixture cleanup")
    });
    let uncertain = std::sync::atomic::AtomicBool::new(false);
    child
        .finish(&|| {
            uncertain.store(true, std::sync::atomic::Ordering::SeqCst);
        })
        .await?;
    release_task.await.map_err(io::Error::other)??;
    require(
        uncertain.load(std::sync::atomic::Ordering::SeqCst),
        "escaped pipe must report uncertainty",
    )?;
    require(
        escaped_owner.finish().await.is_err(),
        "escaped diagnostic must survive physical completion",
    )?;
    checks.push(json!({"scenario":"escaped-descendant-pipe", "pid":escaped,"passed":true,"limit":"group quiescence did not stop escaped work; release file ended it voluntarily; cleanup stayed unknown"}));
    let status = tokio::process::Command::new(std::env::current_exe()?)
        .arg("--parent-loss-owner")
        .arg(&root)
        .arg(&fixture)
        .status()
        .await?;
    require(status.success(), "disposable parent did not exit")?;
    let orphan = marker(&root, "parent-loss").await?;
    require(
        alive(orphan),
        "expected survivor observation after native parent loss",
    )?;
    std::fs::write(
        root.join("parent-loss.release"),
        b"voluntary fixture cleanup",
    )?;
    checks.push(json!({"scenario":"native-parent-loss", "survivor_pid":orphan,"passed":true,"limit":"survival observed; no signaling from saved PID; fixture stopped voluntarily"}));
    Ok(
        json!({"root":root,"fixture":fixture,"checks":checks,"provider":"none","authentication":"none","containment":"macOS owned groups only; escaped descendants and native parent loss are recovery cases"}),
    )
}
