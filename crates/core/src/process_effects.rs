//! Shared physical process mechanics; domain commands and reconciliation stay local.
use std::{io, time::Duration};
use tokio::{process::Child, time::Instant};

use futures_util::FutureExt;
use std::sync::{Arc, Mutex};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    process::{ChildStdin, ChildStdout, Command},
    sync::{Notify, Semaphore, oneshot},
    task::JoinSet,
};

/// A core-local completion owner. No process result belongs to its async waiter.
#[derive(Clone)]
pub(crate) struct Processes(Arc<Owner>);
struct Owner {
    ordinary: Arc<Semaphore>,
    cleanup: Arc<Semaphore>,
    tasks: Mutex<JoinSet<()>>,
    changed: Arc<Notify>,
    faults: Mutex<Faults>,
    closed: std::sync::atomic::AtomicBool,
    reaping: std::sync::atomic::AtomicBool,
    reaper: tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>,
}
#[derive(Default)]
struct Faults {
    messages: Vec<String>,
    omitted: usize,
}
impl Faults {
    fn record(&mut self, message: String) {
        if self.messages.len() < 32 {
            self.messages.push(message);
        } else {
            self.omitted = self.omitted.saturating_add(1);
        }
    }
    fn diagnostic(&self) -> Option<String> {
        if self.messages.is_empty() {
            None
        } else {
            Some(format!(
                "{}; additional physical faults: {}",
                self.messages.join("; "),
                self.omitted
            ))
        }
    }
}

#[derive(Debug)]
pub(crate) enum GitFailure {
    Effect(io::Error),
    Cleanup {
        original: Option<String>,
        cleanup: Option<String>,
    },
}
impl From<io::Error> for GitFailure {
    fn from(error: io::Error) -> Self {
        Self::Effect(error)
    }
}

impl Processes {
    pub(crate) fn new() -> Self {
        let owner = Arc::new(Owner {
            ordinary: Arc::new(Semaphore::new(8)),
            cleanup: Arc::new(Semaphore::new(2)),
            tasks: Mutex::new(JoinSet::new()),
            changed: Arc::new(Notify::new()),
            faults: Mutex::new(Faults::default()),
            closed: std::sync::atomic::AtomicBool::new(false),
            reaping: std::sync::atomic::AtomicBool::new(false),
            reaper: tokio::sync::Mutex::new(None),
        });
        Self(owner)
    }
    fn ensure_reaper(&self) {
        let owner = &self.0;
        if owner
            .reaping
            .swap(true, std::sync::atomic::Ordering::SeqCst)
        {
            return;
        }
        let mut installed = owner
            .reaper
            .try_lock()
            .expect("reaper installation is synchronous");
        if installed.is_some() {
            return;
        }
        let weak = Arc::downgrade(owner);
        let reaper = tokio::spawn(async move {
            loop {
                let Some(owner) = weak.upgrade() else {
                    break;
                };
                let changed = owner.changed.notified();
                let joined = std::future::poll_fn(|cx| {
                    owner
                        .tasks
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .poll_join_next(cx)
                })
                .await;
                match joined {
                    Some(Ok(())) => {
                        owner.changed.notify_waiters();
                    }
                    Some(Err(error)) => owner
                        .faults
                        .lock()
                        .unwrap()
                        .record(format!("Physical process task {}: {error}", error.id())),
                    None if owner.closed.load(std::sync::atomic::Ordering::SeqCst) => break,
                    None => changed.await,
                }
            }
        });
        *installed = Some(reaper);
    }
    pub(crate) fn spawn(&self, command: Command) -> io::Result<Process> {
        self.spawn_inner(command, false, None)
    }
    pub(crate) fn spawn_until(&self, command: Command, deadline: Instant) -> io::Result<Process> {
        self.spawn_inner(command, false, Some(deadline))
    }
    fn spawn_inner(
        &self,
        mut command: Command,
        finite: bool,
        exit_deadline: Option<Instant>,
    ) -> io::Result<Process> {
        let permit = self.0.ordinary.clone().try_acquire_owned().map_err(|_| {
            io::Error::new(
                io::ErrorKind::WouldBlock,
                "Process capacity unavailable; no child started",
            )
        })?;
        self.ensure_reaper();
        let mut child = command.spawn()?;
        let permit = Arc::new(PhysicalLease {
            _permit: Some(permit),
            changed: self.0.changed.clone(),
        });
        let stdin = child.stdin.take().map(|io| Pipe {
            io,
            _lease: permit.clone(),
        });
        let stdout = child.stdout.take().map(|io| Pipe {
            io,
            _lease: permit.clone(),
        });
        let mut stderr = child.stderr.take();
        let finite_stderr = if finite {
            stderr.take().map(|io| Pipe {
                io,
                _lease: permit.clone(),
            })
        } else {
            None
        };
        let completion = self.retain(child.id(), child, stderr, permit, exit_deadline);
        Ok(Process {
            stdin,
            stdout,
            stderr: finite_stderr,
            completion,
        })
    }
    fn retain<G, R>(
        &self,
        id: Option<u32>,
        mut child: G,
        stderr: Option<R>,
        permit: Arc<PhysicalLease>,
        exit_deadline: Option<Instant>,
    ) -> Completion
    where
        G: OwnedGroup + Send + 'static,
        R: AsyncRead + Unpin + Send + 'static,
    {
        self.ensure_reaper();
        let (stop, stopped) = oneshot::channel();
        let (done, result) = oneshot::channel();
        let (uncertain, uncertainty) = tokio::sync::watch::channel(false);
        let owner = self.0.clone();
        self.0.tasks.lock().unwrap_or_else(|e| e.into_inner()).spawn(async move {

            let record = |message: String| owner.faults.lock().unwrap().record(format!("Process {id:?}: {message}"));
            let report = || {
                if !uncertain.send_replace(true) { record("physical cleanup uncertain; ownership retained".into()); }
            };
            let physical = async {
                let (cleaning, cleanup_started) = oneshot::channel();
                let drain = async {
                    let read = async {
                        if let Some(mut stderr) = stderr { tokio::io::copy(&mut stderr, &mut tokio::io::sink()).await.map(|_| ()) }
                        else { Ok(()) }
                    };
                    tokio::pin!(read);
                    tokio::select! {
                        result = &mut read => result,
                        _ = cleanup_started => {
                            match tokio::time::timeout(Duration::from_secs(2), &mut read).await {
                                Ok(result) => result,
                                Err(_) => { report(); read.await }
                            }
                        }
                    }
                };
                let lifecycle = async {
                    let _ = stopped.await;
                        let _cleanup = owner.cleanup.clone().acquire_owned().await.expect("cleanup reserve stays open");
                    settle_group(&mut child, exit_deadline, &report, &record).await;
                    let _ = cleaning.send(());
                };
                let ((), drained) = tokio::join!(lifecycle, drain);
                if let Err(error) = drained {
                    report();
                    owner.faults.lock().unwrap().record(format!("Process {id:?} stderr: {error}"));
                }
                reap(&mut child, &report, &record).await
            };
            // Child is outside this unwind boundary, so a failure cannot bypass cleanup.
            let status = match std::panic::AssertUnwindSafe(physical).catch_unwind().await {
                Ok(status) => status,
                Err(_) => {
                    report();
                    owner.faults.lock().unwrap().record(format!("Process {id:?} lifecycle panicked"));
                    settle_group(&mut child, exit_deadline, &report, &record).await;
                    reap(&mut child, &report, &record).await
                }
            };
            if let Err(error) = &status { owner.faults.lock().unwrap().record(format!("Process {id:?}: {error}")); }
            if *uncertain.borrow() { owner.faults.lock().unwrap().record(format!("Process {id:?} cleanup was uncertain")); }
            drop(permit); // child and pipe completion, independent of the result receiver
            let _ = done.send(status);
            owner.changed.notify_waiters();
        });
        self.0.changed.notify_one();
        Completion {
            stop: Some(stop),
            result,
            uncertainty,
        }
    }
    /// Finite Git bridge for already bounded blocking workspace workers. No runtime/block_on.
    pub(crate) fn git(
        &self,
        mut command: std::process::Command,
        input: Option<&[u8]>,
    ) -> Result<(bool, Vec<u8>), GitFailure> {
        if input.is_some_and(|bytes| bytes.len() > 1024) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Git command input exceeds its bounded budget",
            )
            .into());
        }
        if input.is_some() {
            command.stdin(std::process::Stdio::piped());
        }
        // Reserve before task creation/spawn; the physical owner keeps the permit.
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut process = self.spawn_inner(command.into(), true, Some(deadline))?;
        let stdin = process.stdin.take();
        let stdout = process
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("Git stdout unavailable"))?;
        let stderr = process
            .stderr
            .take()
            .ok_or_else(|| io::Error::other("Git stderr unavailable"))?;
        let input = input.map(<[u8]>::to_vec);
        let (send, receive) = oneshot::channel();
        self.0
            .tasks
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .spawn(async move {
                let deadline = Instant::now() + Duration::from_secs(2);
                let output = std::panic::AssertUnwindSafe(finite_output(
                    stdout, stderr, stdin, input, deadline,
                ))
                .catch_unwind()
                .await
                .unwrap_or_else(|_| Err(io::Error::other("Git pipe task panicked")));
                let uncertain = std::sync::atomic::AtomicBool::new(false);
                let status = process
                    .finish(&|| {
                        uncertain.store(true, std::sync::atomic::Ordering::SeqCst);
                    })
                    .await;
                let result = if uncertain.load(std::sync::atomic::Ordering::SeqCst) {
                    Err(GitFailure::Cleanup {
                        original: output.as_ref().err().map(ToString::to_string),
                        cleanup: status.as_ref().err().map(ToString::to_string),
                    })
                } else {
                    match status {
                        Ok(status) => output
                            .map(|(out, err)| {
                                (status.success(), if status.success() { out } else { err })
                            })
                            .map_err(GitFailure::Effect),
                        Err(error) => Err(GitFailure::Cleanup {
                            original: output.err().map(|e| e.to_string()),
                            cleanup: Some(error.to_string()),
                        }),
                    }
                };
                let _ = send.send(result);
            });
        self.0.changed.notify_one();
        receive
            .blocking_recv()
            .map_err(io::Error::other)
            .map_err(GitFailure::Effect)?
    }

    pub(crate) fn diagnostic(&self) -> Option<String> {
        self.0.faults.lock().unwrap().diagnostic()
    }

    pub(crate) async fn finish(&self) -> io::Result<()> {
        self.0.ordinary.close();
        self.0
            .closed
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.0.changed.notify_one();
        let mut reaper = self.0.reaper.lock().await;
        if let Some(task) = reaper.as_mut() {
            task.await.map_err(io::Error::other)?;
            reaper.take();
        }
        loop {
            let changed = self.0.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if self.0.ordinary.available_permits() == 8 {
                break;
            }
            changed.await;
        }
        self.diagnostic()
            .map_or(Ok(()), |fault| Err(io::Error::other(fault)))
    }
}

struct PhysicalLease {
    _permit: Option<tokio::sync::OwnedSemaphorePermit>,
    changed: Arc<Notify>,
}
impl Drop for PhysicalLease {
    fn drop(&mut self) {
        self._permit.take();
        self.changed.notify_one();
    }
}
/// Taking a pipe transfers physical capacity with it, even past its logical waiter.
pub(crate) struct Pipe<T> {
    io: T,
    _lease: Arc<PhysicalLease>,
}
impl<T: AsyncRead + Unpin> AsyncRead for Pipe<T> {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        std::pin::Pin::new(&mut self.get_mut().io).poll_read(cx, buf)
    }
}
impl<T: tokio::io::AsyncWrite + Unpin> tokio::io::AsyncWrite for Pipe<T> {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        bytes: &[u8],
    ) -> std::task::Poll<io::Result<usize>> {
        std::pin::Pin::new(&mut self.get_mut().io).poll_write(cx, bytes)
    }
    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        std::pin::Pin::new(&mut self.get_mut().io).poll_flush(cx)
    }
    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        std::pin::Pin::new(&mut self.get_mut().io).poll_shutdown(cx)
    }
}

pub(crate) struct Process {
    pub(crate) stdin: Option<Pipe<ChildStdin>>,
    pub(crate) stdout: Option<Pipe<ChildStdout>>,
    stderr: Option<Pipe<tokio::process::ChildStderr>>,
    completion: Completion,
}
struct Completion {
    stop: Option<oneshot::Sender<()>>,
    result: oneshot::Receiver<io::Result<std::process::ExitStatus>>,
    uncertainty: tokio::sync::watch::Receiver<bool>,
}
impl Process {
    pub(crate) async fn finish(
        mut self,
        uncertain: &impl Fn(),
    ) -> io::Result<std::process::ExitStatus> {
        drop(self.stdin.take());
        drop(self.stdout.take());
        self.completion.finish(uncertain).await
    }
}
impl Completion {
    async fn finish(mut self, uncertain: &impl Fn()) -> io::Result<std::process::ExitStatus> {
        self.stop.take();
        loop {
            if *self.uncertainty.borrow_and_update() {
                uncertain();
            }
            tokio::select! {
                result = &mut self.result => {
                    if *self.uncertainty.borrow() { uncertain(); }
                    return result.map_err(io::Error::other)?;
                }
                changed = self.uncertainty.changed() => { if changed.is_err() { return self.result.await.map_err(io::Error::other)?; } }
            }
        }
    }
}

trait OwnedGroup: Cleanup {
    fn wait(
        &mut self,
    ) -> impl std::future::Future<Output = io::Result<std::process::ExitStatus>> + Send;
}
impl OwnedGroup for Child {
    async fn wait(&mut self) -> io::Result<std::process::ExitStatus> {
        Child::wait(self).await
    }
}

async fn reap(
    child: &mut impl OwnedGroup,
    uncertain: &impl Fn(),
    record: &impl Fn(String),
) -> io::Result<std::process::ExitStatus> {
    loop {
        match child.wait().await {
            Ok(status) => return Ok(status),
            Err(error) => {
                record(format!("child wait failed; ownership retained: {error}"));
                uncertain();
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

async fn settle_group(
    child: &mut (impl Cleanup + Send),
    exit_deadline: Option<Instant>,
    uncertain: &impl Fn(),
    record: &impl Fn(String),
) {
    let report = cleanup(child, exit_deadline).await;
    for error in &report.errors {
        record(error.clone());
    }
    if !report.errors.is_empty() {
        uncertain();
    }
    if !report.quiescent {
        uncertain();
        // Keep the unreaped leader and permit while group evidence is unknown.
        while !child
            .quiescent(Duration::from_millis(100))
            .await
            .unwrap_or(false)
        {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

async fn finite_output<R, E, W>(
    stdout: R,
    stderr: E,
    stdin: Option<W>,
    input: Option<Vec<u8>>,
    deadline: Instant,
) -> io::Result<(Vec<u8>, Vec<u8>)>
where
    R: AsyncRead + Unpin,
    E: AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
    let write = async {
        if let (Some(mut stdin), Some(input)) = (stdin, input) {
            tokio::time::timeout_at(deadline, async {
                stdin.write_all(&input).await?;
                stdin.shutdown().await
            })
            .await
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Git input timed out; transmission may be partial",
                )
            })??;
        }
        Ok::<_, io::Error>(())
    };
    let (written, output, errors) = tokio::join!(
        write,
        collect(stdout, 256 * 1024, deadline),
        collect(stderr, 256 * 1024, deadline)
    );
    written?;
    Ok((output?, errors?))
}

/// Finite readers own a bounded vector; EOF, overflow and deadline are distinct.
pub(crate) async fn collect(
    mut read: impl AsyncRead + Unpin,
    limit: usize,
    deadline: Instant,
) -> io::Result<Vec<u8>> {
    tokio::time::timeout_at(deadline, async {
        let mut bytes = Vec::new();
        let mut buffer = [0u8; 2048];
        loop {
            let count = read.read(&mut buffer).await?;
            if count == 0 {
                return Ok(bytes);
            }
            if count > limit.saturating_sub(bytes.len()) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Process output exceeded its bounded read budget",
                ));
            }
            bytes.extend_from_slice(&buffer[..count]);
        }
    })
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "Process output timed out"))?
}

pub(crate) trait Cleanup {
    fn signal(&mut self, signal: i32) -> io::Result<()>;
    fn quiescent(
        &mut self,
        budget: Duration,
    ) -> impl std::future::Future<Output = io::Result<bool>> + Send;
}
impl Cleanup for Child {
    fn signal(&mut self, signal: i32) -> io::Result<()> {
        let pid = self
            .id()
            .and_then(|pid| i32::try_from(pid).ok())
            .ok_or_else(|| io::Error::other("owned child identity unavailable"))?;
        // SAFETY: process_group(0) created this group, and the unreaped child
        // handle still owns its identity. Never signal from a stored/reaped PID.
        if unsafe { libc::kill(-pid, signal) } == 0 {
            Ok(())
        } else {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::ESRCH) {
                Ok(())
            } else {
                Err(error)
            }
        }
    }
    async fn quiescent(&mut self, budget: Duration) -> io::Result<bool> {
        let pid = self
            .id()
            .and_then(|pid| i32::try_from(pid).ok())
            .ok_or_else(|| io::Error::other("owned group leader unavailable"))?;
        let deadline = Instant::now() + budget;
        loop {
            if group_quiescent(pid)? {
                return Ok(true);
            }
            if Instant::now() >= deadline {
                return Ok(false);
            }
            tokio::time::sleep_until((Instant::now() + Duration::from_millis(20)).min(deadline))
                .await;
        }
    }
}
struct CleanupReport {
    quiescent: bool,
    errors: Vec<String>,
}
async fn cleanup(child: &mut impl Cleanup, exit_deadline: Option<Instant>) -> CleanupReport {
    let mut report = CleanupReport {
        quiescent: false,
        errors: Vec::new(),
    };
    for signal in [None, Some(libc::SIGTERM), Some(libc::SIGKILL)] {
        if let Some(signal) = signal
            && let Err(error) = child.signal(signal)
        {
            report
                .errors
                .push(format!("group signal {signal}: {error}"));
        }
        let budget = if signal.is_none() {
            exit_deadline
                .map(|deadline| deadline.saturating_duration_since(Instant::now()))
                .unwrap_or(Duration::from_secs(2))
        } else {
            Duration::from_secs(2)
        };
        match child.quiescent(budget).await {
            Ok(true) => {
                report.quiescent = true;
                break;
            }
            Ok(false) => {}
            Err(error) => report.errors.push(format!("group quiescence: {error}")),
        }
    }
    report
}

#[cfg(target_os = "macos")]
fn group_quiescent(group: i32) -> io::Result<bool> {
    let group_id = u32::try_from(group).map_err(io::Error::other)?;
    let mut status = std::mem::MaybeUninit::<libc::siginfo_t>::zeroed();
    // SAFETY: correctly aligned initialized storage; P_PID targets the retained
    // direct child only. WNOWAIT observes exit without relinquishing its identity.
    if unsafe {
        libc::waitid(
            libc::P_PID,
            group_id,
            status.as_mut_ptr(),
            libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
        )
    } != 0
    {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful waitid initialized this record; si_pid is its PID field.
    let exited = unsafe { status.assume_init().si_pid() } == group;
    if !exited {
        return Ok(false);
    }
    let mut pids = [0i32; 512];
    let capacity = i32::try_from(std::mem::size_of_val(&pids)).expect("bounded PID array");
    // SAFETY: initialized writable array with the exact supplied byte capacity.
    // PROC_PGRP_ONLY=2 comes from the selected macOS SDK sys/proc_info.h.
    let bytes = unsafe {
        libc::proc_listpids(
            2,
            u32::try_from(group).map_err(io::Error::other)?,
            pids.as_mut_ptr().cast(),
            capacity,
        )
    };
    if bytes <= 0 || bytes >= capacity || bytes % 4 != 0 {
        return Err(io::Error::other("owned group enumeration incomplete"));
    }
    let count = usize::try_from(bytes / 4).map_err(io::Error::other)?;
    let mut leader_seen = false;
    for pid in &pids[..count] {
        if *pid == group {
            leader_seen = true;
            continue;
        }
        let mut info = std::mem::MaybeUninit::<libc::proc_bsdinfo>::uninit();
        let size =
            i32::try_from(std::mem::size_of::<libc::proc_bsdinfo>()).expect("fixed OS record");
        // SAFETY: aligned storage of exactly the structure size required by this
        // SDK flavor. Initialized fields are read only after exact byte success.
        let read = unsafe {
            libc::proc_pidinfo(
                *pid,
                libc::PROC_PIDTBSDINFO,
                0,
                info.as_mut_ptr().cast(),
                size,
            )
        };
        if read != size {
            let error = io::Error::last_os_error();
            // macOS hides zombies from proc_pidinfo (ESRCH), even though they
            // remain in proc_listpids. They own no running work or open pipes.
            if error.raw_os_error() == Some(libc::ESRCH) {
                continue;
            }
            return Err(error);
        }
        // SAFETY: proc_pidinfo returned the entire initialized proc_bsdinfo.
        let info = unsafe { info.assume_init() };
        if info.pbi_pgid != u32::try_from(group).map_err(io::Error::other)? {
            return Err(io::Error::other(
                "owned group membership changed during inspection",
            ));
        }
        leader_seen |= *pid == group;
        if info.pbi_status != libc::SZOMB {
            return Ok(false);
        }
    }
    if !leader_seen {
        return Err(io::Error::other("unreaped owned group leader missing"));
    }
    Ok(true)
}
#[cfg(not(target_os = "macos"))]
fn group_quiescent(_group: i32) -> io::Result<bool> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "process-group qualification currently supports macOS only",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncWriteExt, duplex};

    struct Fake {
        waits: std::collections::VecDeque<io::Result<bool>>,
        signals: Vec<i32>,
        fail_signal: bool,
    }
    impl Cleanup for Fake {
        fn signal(&mut self, signal: i32) -> io::Result<()> {
            self.signals.push(signal);
            if self.fail_signal {
                Err(io::Error::other("synthetic"))
            } else {
                Ok(())
            }
        }
        async fn quiescent(&mut self, _: Duration) -> io::Result<bool> {
            self.waits.pop_front().unwrap()
        }
    }
    #[test]
    fn cleanup_escalates_only_while_owned_and_retains_uncertainty() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .start_paused(true)
            .build()
            .unwrap()
            .block_on(async {
                let mut fake = Fake {
                    waits: [Ok(false), Ok(false), Ok(true)].into(),
                    signals: vec![],
                    fail_signal: false,
                };
                assert!(cleanup(&mut fake, None).await.quiescent);
                assert_eq!(fake.signals, [libc::SIGTERM, libc::SIGKILL]);
                let mut fake = Fake {
                    waits: [Ok(false), Err(io::Error::other("group unknown")), Ok(false)].into(),
                    signals: vec![],
                    fail_signal: true,
                };
                let report = cleanup(&mut fake, None).await;
                assert!(!report.quiescent);
                assert_eq!(report.errors.len(), 3);
                assert_eq!(fake.signals, [libc::SIGTERM, libc::SIGKILL]);
            });
    }
    struct Group {
        started: Option<oneshot::Sender<()>>,
        release: Option<oneshot::Receiver<()>>,
        panic_once: bool,
        wait_error: bool,
    }
    impl Cleanup for Group {
        fn signal(&mut self, _: i32) -> io::Result<()> {
            Ok(())
        }
        async fn quiescent(&mut self, _: Duration) -> io::Result<bool> {
            if std::mem::take(&mut self.panic_once) {
                panic!("synthetic cleanup panic");
            }
            if let Some(started) = self.started.take() {
                let _ = started.send(());
            }
            if let Some(release) = self.release.take() {
                let _ = release.await;
            }
            Ok(true)
        }
    }
    impl OwnedGroup for Group {
        async fn wait(&mut self) -> io::Result<std::process::ExitStatus> {
            use std::os::unix::process::ExitStatusExt;
            if std::mem::take(&mut self.wait_error) {
                Err(io::Error::other("synthetic wait failure"))
            } else {
                Ok(std::process::ExitStatus::from_raw(0))
            }
        }
    }
    fn retained(
        owner: &Processes,
        panic_once: bool,
        wait_error: bool,
        stderr: Option<tokio::io::DuplexStream>,
    ) -> (Completion, oneshot::Receiver<()>, oneshot::Sender<()>) {
        let permit = owner.0.ordinary.clone().try_acquire_owned().unwrap();
        let permit = Arc::new(PhysicalLease {
            _permit: Some(permit),
            changed: owner.0.changed.clone(),
        });
        let (started, entered) = oneshot::channel();
        let (release, released) = oneshot::channel();
        let completion = owner.retain(
            Some(42),
            Group {
                started: Some(started),
                release: Some(released),
                panic_once,
                wait_error,
            },
            stderr,
            permit,
            None,
        );
        (completion, entered, release)
    }
    #[tokio::test(start_paused = true)]
    async fn abandoned_waiter_keeps_capacity_until_physical_completion_and_reaper_observes_errors()
    {
        for wait_error in [false, true] {
            let owner = Processes::new();
            let (completion, entered, release) = retained(&owner, false, wait_error, None);
            drop(completion);
            entered.await.unwrap();
            assert_eq!(owner.0.ordinary.available_permits(), 7);
            release.send(()).unwrap();
            let result = owner.finish().await;
            assert_eq!(owner.0.ordinary.available_permits(), 8);
            if wait_error {
                assert!(
                    result
                        .unwrap_err()
                        .to_string()
                        .contains("synthetic wait failure")
                );
            } else {
                result.unwrap();
            }
        }
    }
    #[tokio::test]
    async fn panic_runs_cleanup_and_preserves_failure_after_recovery() {
        let owner = Processes::new();
        let (completion, entered, release) = retained(&owner, true, false, None);
        drop(completion);
        entered.await.unwrap();
        assert_eq!(owner.0.ordinary.available_permits(), 7);
        release.send(()).unwrap();
        assert!(
            owner
                .finish()
                .await
                .unwrap_err()
                .to_string()
                .contains("lifecycle panicked")
        );
    }
    #[tokio::test(start_paused = true)]
    async fn partial_input_timeout_preserves_uncertainty_without_replaying_bytes() {
        let (write, mut peer) = duplex(1);
        let task = tokio::spawn(finite_output(
            tokio::io::empty(),
            tokio::io::empty(),
            Some(write),
            Some(b"input".to_vec()),
            Instant::now() + Duration::from_secs(2),
        ));
        let error = task.await.unwrap().unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(error.to_string().contains("partial"));
        let mut received = Vec::new();
        peer.read_to_end(&mut received).await.unwrap();
        assert_eq!(received, b"i");
    }
    #[tokio::test]
    async fn taken_pipe_keeps_capacity_after_leader_and_async_waiter_complete() {
        let owner = Processes::new();
        let permit = owner.0.ordinary.clone().try_acquire_owned().unwrap();
        let lease = Arc::new(PhysicalLease {
            _permit: Some(permit),
            changed: owner.0.changed.clone(),
        });
        let pipe = Pipe {
            io: tokio::io::empty(),
            _lease: lease.clone(),
        };
        drop(lease);
        let drain = tokio::spawn({
            let owner = owner.clone();
            async move { owner.finish().await }
        });
        assert_eq!(owner.0.ordinary.available_permits(), 7);
        assert!(!drain.is_finished());
        drop(pipe);
        drain.await.unwrap().unwrap();
        assert_eq!(owner.0.ordinary.available_permits(), 8);
    }

    #[tokio::test(start_paused = true)]
    async fn pipe_outliving_quiescent_group_retains_capacity_and_reports_unknown() {
        let owner = Processes::new();
        let (send, receive) = duplex(8);
        let (completion, entered, release) = retained(&owner, false, false, Some(receive));
        let mut unknown = completion.uncertainty.clone();
        let task = tokio::spawn(async move { completion.finish(&|| {}).await });
        entered.await.unwrap();
        release.send(()).unwrap();
        unknown.changed().await.unwrap();
        assert!(*unknown.borrow());
        assert_eq!(owner.0.ordinary.available_permits(), 7);
        assert!(!task.is_finished());
        drop(send);
        task.await.unwrap().unwrap();
        assert!(
            owner
                .finish()
                .await
                .unwrap_err()
                .to_string()
                .contains("uncertain")
        );
    }
    #[tokio::test]
    async fn saturated_ordinary_capacity_leaves_cleanup_reserve_and_rejects_before_spawn() {
        let owner = Processes::new();
        let permits: Vec<_> = (0..8)
            .map(|_| owner.0.ordinary.clone().try_acquire_owned().unwrap())
            .collect();
        let error = owner
            .spawn(Command::new("/must-not-be-spawned"))
            .err()
            .unwrap();
        assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
        assert!(owner.0.cleanup.clone().try_acquire_many_owned(2).is_ok());
        drop(permits);
        owner.finish().await.unwrap();
    }

    #[tokio::test(start_paused = true)]
    async fn partial_output_requires_eof_and_keeps_one_absolute_deadline() {
        let (mut send, receive) = duplex(8);
        let deadline = Instant::now() + Duration::from_secs(2);
        let task = tokio::spawn(collect(receive, 8, deadline));
        send.write_all(b"part").await.unwrap();
        tokio::task::yield_now().await;
        assert!(!task.is_finished());
        tokio::time::advance(Duration::from_secs(1)).await;
        send.write_all(b"ial").await.unwrap();
        tokio::time::advance(Duration::from_secs(1)).await;
        assert_eq!(
            task.await.unwrap().unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
    }

    #[tokio::test]
    async fn exact_limit_is_complete_only_at_eof_and_overflow_is_not_truncated() {
        for (bytes, limit, expected) in [
            (b"four".as_slice(), 4, true),
            (b"five!".as_slice(), 4, false),
        ] {
            let (mut send, receive) = duplex(8);
            send.write_all(bytes).await.unwrap();
            drop(send);
            let result = collect(receive, limit, Instant::now() + Duration::from_secs(2)).await;
            if expected {
                assert_eq!(result.unwrap(), bytes);
            } else {
                assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidData);
            }
        }
    }
}

#[cfg(feature = "qualification")]
#[path = "process_effects/qualification.rs"]
pub(crate) mod qualification;
