//! Owned nonblocking Codex stdio execution.
use super::{
    Configuration, Diagnostic, ErrorCode, Phase, Result, RuntimeError, SUPPORTED_VERSION, protocol,
};
use serde_json::{Value, json};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    os::{
        fd::AsRawFd,
        unix::{
            fs::{MetadataExt, OpenOptionsExt},
            process::CommandExt,
        },
    },
    path::{Path, PathBuf},
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

const MAX_OUTPUT: usize = 1024 * 1024;
const MAX_WRITE: usize = 64 * 1024;
const MAX_BATCH: usize = 64;
const OVERALL: Duration = Duration::from_secs(30);
const VERSION: Duration = Duration::from_secs(5);
const INITIALIZE: Duration = Duration::from_secs(10);
const READ: Duration = Duration::from_secs(5);
const CLEANUP: Duration = Duration::from_secs(3);

fn io_error() -> RuntimeError {
    RuntimeError::new(
        ErrorCode::Process,
        "Codex could not start or its stdio connection failed. Check executable permissions and native CLI configuration outside the app.",
    )
}
fn timeout() -> RuntimeError {
    RuntimeError::new(
        ErrorCode::Timeout,
        "Codex setup exceeded its deadline. Inspect native CLI startup/configuration, then check again after cleanup settles.",
    )
}
fn limit() -> RuntimeError {
    RuntimeError::new(
        ErrorCode::OutputLimit,
        "Codex exceeded the bounded setup output budget. Inspect native CLI startup/configuration externally.",
    )
}

trait Discovery {
    fn path(&self) -> Vec<PathBuf>;
    fn home(&self) -> Option<PathBuf>;
    fn executable(&self, path: &Path) -> bool;
}
struct LocalDiscovery;
impl Discovery for LocalDiscovery {
    fn path(&self) -> Vec<PathBuf> {
        std::env::var_os("PATH")
            .map(|p| {
                std::env::split_paths(&p)
                    .filter(|p| p.is_absolute())
                    .collect()
            })
            .unwrap_or_default()
    }
    fn home(&self) -> Option<PathBuf> {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
    }
    fn executable(&self, path: &Path) -> bool {
        std::fs::metadata(path).is_ok_and(|m| m.is_file() && m.mode() & 0o111 != 0)
    }
}
fn discover(configuration: &Configuration, effects: &impl Discovery) -> Result<PathBuf> {
    if let Some(path) = &configuration.executable {
        let path = PathBuf::from(path);
        return if effects.executable(&path) {
            Ok(path)
        } else {
            Err(RuntimeError::new(
                ErrorCode::MissingExecutable,
                "The saved executable is unavailable or not executable. Correct it or reset the saved path; automatic discovery was not substituted.",
            ))
        };
    }
    let mut candidates = effects
        .path()
        .into_iter()
        .map(|p| p.join("codex"))
        .collect::<Vec<_>>();
    if let Some(home) = effects.home() {
        candidates.push(home.join(".local/bin/codex"));
    }
    candidates.extend([
        PathBuf::from("/opt/homebrew/bin/codex"),
        PathBuf::from("/usr/local/bin/codex"),
    ]);
    candidates.into_iter().find(|p|effects.executable(p)).ok_or_else(||RuntimeError::new(ErrorCode::MissingExecutable,"Codex was not found in PATH or standard locations. Install the supported official CLI or save its absolute executable path."))
}

#[derive(Debug, Clone)]
struct CommandSpec {
    executable: PathBuf,
    args: Vec<&'static str>,
    directory: PathBuf,
}
fn command_spec(executable: &Path, directory: &Path, version: bool) -> CommandSpec {
    CommandSpec {
        executable: executable.to_owned(),
        args: if version {
            vec!["--version"]
        } else {
            vec!["app-server"]
        },
        directory: directory.to_owned(),
    }
}

enum Chunk {
    Stdout(Vec<u8>),
    Stderr(usize),
    Idle,
    Eof,
}
trait Session {
    fn send(&mut self, bytes: &[u8]) -> Result<()>;
    fn poll(&mut self) -> Result<Chunk>;
    fn elapsed(&self) -> Duration;
    fn wait_tick(&mut self);
    fn cleanup(&mut self) -> bool;
    fn successful_exit(&self) -> bool;
}

fn account_output(total: &mut usize, count: usize) -> Result<()> {
    *total = total
        .checked_add(count)
        .filter(|n| *n <= MAX_OUTPUT)
        .ok_or_else(limit)?;
    Ok(())
}
fn rpc(
    session: &mut dyn Session,
    total: &mut usize,
    frames: &mut protocol::Frames,
    id: i64,
    call: (&str, Value),
    deadline: Duration,
    stop: &AtomicBool,
) -> Result<Value> {
    let (method, params) = call;
    session.send(&protocol::request(id, method, params))?;
    let mut result = None;
    loop {
        if stop.load(Ordering::SeqCst) || session.elapsed() >= deadline {
            return Err(timeout());
        }
        for _ in 0..MAX_BATCH {
            match session.poll()? {
                Chunk::Stdout(bytes) => {
                    account_output(total, bytes.len())?;
                    for frame in frames.push(&bytes)? {
                        if let Some(value) = protocol::response(frame, id)? {
                            if result.is_some() {
                                return Err(protocol::failure());
                            }
                            result = Some(value);
                        }
                    }
                }
                Chunk::Stderr(n) => account_output(total, n)?,
                Chunk::Eof => return Err(protocol::failure()),
                Chunk::Idle => break,
            }
        }
        if let Some(result) = result {
            return Ok(result);
        }
        session.wait_tick();
    }
}
fn version(session: &mut dyn Session, total: &mut usize, stop: &AtomicBool) -> Result<String> {
    let mut output = Vec::new();
    loop {
        if stop.load(Ordering::SeqCst) || session.elapsed() >= VERSION {
            return Err(timeout());
        }
        match session.poll()? {
            Chunk::Stdout(bytes) => {
                account_output(total, bytes.len())?;
                if output.len() + bytes.len() > protocol::MAX_FRAME {
                    return Err(limit());
                }
                output.extend(bytes);
            }
            Chunk::Stderr(n) => account_output(total, n)?,
            Chunk::Eof => break,
            Chunk::Idle => session.wait_tick(),
        }
    }
    let text = std::str::from_utf8(&output)
        .map_err(|_| protocol::failure())?
        .trim();
    let value = text
        .strip_prefix("codex-cli ")
        .filter(|v| {
            v.len() <= 64
                && !v.is_empty()
                && v.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b".-+".contains(&b))
        })
        .ok_or_else(protocol::failure)?;
    Ok(value.into())
}

fn metadata(
    session: &mut dyn Session,
    total: &mut usize,
    stop: &AtomicBool,
    budget: Duration,
    progress: &mut dyn FnMut(Phase),
) -> Result<protocol::Metadata> {
    let mut frames = protocol::Frames::default();
    progress(Phase::Initializing);
    let initialized = rpc(
        session,
        total,
        &mut frames,
        1,
        ("initialize", protocol::initialize_params()),
        INITIALIZE.min(budget.saturating_sub(CLEANUP)),
        stop,
    )?;
    protocol::validate_initialize(&initialized)?;
    session.send(&protocol::initialized())?;
    progress(Phase::Account);
    let deadline = (session.elapsed() + READ).min(budget.saturating_sub(CLEANUP));
    let account = rpc(
        session,
        total,
        &mut frames,
        2,
        ("account/read", json!({"refreshToken":false})),
        deadline,
        stop,
    )?;
    progress(Phase::Configuration);
    let deadline = (session.elapsed() + READ).min(budget.saturating_sub(CLEANUP));
    let configuration=rpc(session,total,&mut frames,3,("config/read",json!({"includeLayers":false,"cwd":null})),deadline,stop).map_err(|e|if e.code==ErrorCode::Protocol {RuntimeError::new(ErrorCode::Configuration,"Codex configuration could not be read. Inspect it with the official CLI, then check again.")} else {e})?;
    protocol::metadata(&account, &configuration)
}

pub(super) struct Runner {
    directory: PathBuf,
    stop: Arc<AtomicBool>,
    quarantine: Vec<Box<dyn Session>>,
    fixture: Option<String>,
}
impl Runner {
    pub fn new(directory: PathBuf, stop: Arc<AtomicBool>) -> Self {
        Self {
            directory,
            stop,
            quarantine: vec![],
            fixture: std::env::var("AGENT_WORKSPACE_CODEX_FIXTURE").ok(),
        }
    }
    fn start(&mut self, spec: &CommandSpec) -> Result<Box<dyn Session>> {
        if let Some(scenario) = &self.fixture {
            return FixtureSession::start(scenario, spec).map(|s| Box::new(s) as Box<dyn Session>);
        }
        let mut session = NativeSession::start(spec)?;
        if nonblocking(session.stdin.as_ref().expect("owned stdin"))
            .and_then(|()| nonblocking(&session.stdout))
            .and_then(|()| nonblocking(&session.stderr))
            .is_err()
        {
            if !session.cleanup() {
                self.quarantine.push(Box::new(session));
                return Err(RuntimeError::cleanup());
            }
            return Err(io_error());
        }
        session.pipes_ready = true;
        Ok(Box::new(session))
    }
    pub fn check(
        &mut self,
        configuration: &Configuration,
        id: Uuid,
        progress: &mut dyn FnMut(Phase),
    ) -> Diagnostic {
        let mut diagnostic = Diagnostic {
            check_id: id,
            revision: configuration.revision,
            timestamp_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .ok()
                .and_then(|d| u64::try_from(d.as_millis()).ok())
                .unwrap_or(0),
            executable: None,
            version: None,
            authentication: "Unknown".into(),
            provider: None,
            billing: "Unknown".into(),
            passed: false,
            error: None,
            cleanup_confirmed: true,
        };
        let mut total = 0;
        let outcome = (|| {
            if self.stop.load(Ordering::SeqCst) {
                return Err(timeout());
            }
            progress(Phase::Discovering);
            let executable = if self.fixture.is_some() {
                if self.fixture.as_deref() == Some("missing") {
                    return Err(RuntimeError::new(
                        ErrorCode::MissingExecutable,
                        "Synthetic fixture: Codex executable missing. Install Codex or reset its saved path.",
                    ));
                }
                PathBuf::from("/fixture/codex")
            } else {
                discover(configuration, &LocalDiscovery)?
            };
            let resolved = executable
                .to_str()
                .filter(|p| p.len() <= super::MAX_EXECUTABLE_BYTES)
                .ok_or_else(io_error)?;
            diagnostic.executable = Some(resolved.to_owned());
            let directory = self.directory.join("codex-setup");
            if self.fixture.is_none() {
                private_directory(&directory)?;
            }
            progress(Phase::Version);
            let mut session = self.start(&command_spec(&executable, &directory, true))?;
            let result = version(session.as_mut(), &mut total, &self.stop);
            let clean = session.cleanup();
            if !clean {
                self.quarantine.push(session);
                diagnostic.cleanup_confirmed = false;
                return Err(RuntimeError::cleanup());
            }
            if !session.successful_exit() {
                return Err(io_error());
            }
            let spent = session.elapsed();
            diagnostic.version = Some(result?);
            if diagnostic.version.as_deref() != Some(SUPPORTED_VERSION) {
                return Err(RuntimeError::new(
                    ErrorCode::IncompatibleVersion,
                    format!(
                        "This app supports Codex {SUPPORTED_VERSION}. Install that official CLI version, then check setup again."
                    ),
                ));
            }
            if self.stop.load(Ordering::SeqCst) {
                return Err(timeout());
            }
            let mut session = self.start(&command_spec(&executable, &directory, false))?;
            let result = metadata(
                session.as_mut(),
                &mut total,
                &self.stop,
                OVERALL.saturating_sub(spent),
                progress,
            );
            progress(Phase::Cleaning);
            let clean = session.cleanup();
            if !clean {
                self.quarantine.push(session);
                diagnostic.cleanup_confirmed = false;
                return Err(RuntimeError::cleanup());
            }
            if !session.successful_exit() {
                return Err(io_error());
            }
            let evidence = result?;
            diagnostic.authentication = evidence.authentication;
            diagnostic.provider = evidence.provider;
            diagnostic.billing = evidence.billing;
            Ok(())
        })();
        diagnostic.cleanup_confirmed &= self.quarantine.is_empty();
        diagnostic.passed = outcome.is_ok() && diagnostic.cleanup_confirmed;
        diagnostic.error = outcome.err();
        diagnostic
    }
    pub fn cleanup_confirmed(&self) -> bool {
        self.quarantine.is_empty()
    }
}

fn private_directory(directory: &Path) -> Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    match std::fs::DirBuilder::new().mode(0o700).create(directory) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
        Err(_) => return Err(io_error()),
    }
    crate::supervisor::check_private(directory, true).map_err(|_| io_error())
}

fn execution_lock(directory: &Path) -> Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(directory.join("execution.lock"))
        .map_err(|_| io_error())?;
    let metadata = file.metadata().map_err(|_| io_error())?;
    // SAFETY: geteuid has no preconditions and changes no process state.
    if !metadata.is_file()
        || metadata.nlink() != 1
        || metadata.mode() & 0o077 != 0
        || metadata.uid() != unsafe { libc::geteuid() }
    {
        return Err(io_error());
    }
    Ok(file)
}

struct NativeSession {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: ChildStdout,
    stderr: ChildStderr,
    start: Instant,
    lease: Option<File>,
    directory: PathBuf,
    write: Vec<u8>,
    stdout_eof: bool,
    stderr_eof: bool,
    stderr_next: bool,
    reaped: bool,
    successful_exit: bool,
    pipes_ready: bool,
    cleanup_start: Option<Instant>,
}
impl NativeSession {
    fn start(spec: &CommandSpec) -> Result<Self> {
        let lease = execution_lock(&spec.directory)?;
        lease.try_lock().map_err(|_| RuntimeError::cleanup())?;
        let fd = lease.as_raw_fd();
        let mut command = Command::new(&spec.executable);
        command
            .args(&spec.args)
            .current_dir(&spec.directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0);
        // SAFETY: only async-signal-safe fcntl runs between fork and exec. The
        // parent's lease is alive through spawn; inherited work retains its lock.
        unsafe {
            command.pre_exec(move || {
                let flags = libc::fcntl(fd, libc::F_GETFD);
                if flags < 0 || libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) < 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let mut child = command.spawn().map_err(|_| io_error())?;
        let stdin = child.stdin.take().expect("piped stdin");
        let stdout = child.stdout.take().expect("piped stdout");
        let stderr = child.stderr.take().expect("piped stderr");
        Ok(Self {
            child,
            stdin: Some(stdin),
            stdout,
            stderr,
            start: Instant::now(),
            lease: Some(lease),
            directory: spec.directory.clone(),
            write: vec![],
            stdout_eof: false,
            stderr_eof: false,
            stderr_next: false,
            reaped: false,
            successful_exit: false,
            pipes_ready: false,
            cleanup_start: None,
        })
    }
    fn signal(&self, signal: i32) -> bool {
        if self.reaped {
            return false;
        }
        let Ok(group) = i32::try_from(self.child.id()) else {
            return false;
        };
        // SAFETY: the retained, unreaped child pins the freshly created group ID.
        unsafe {
            libc::kill(-group, signal) == 0
                || io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
        }
    }
    fn exited_without_reaping(&self) -> bool {
        // SAFETY: waitid writes an initialized siginfo for our retained child;
        // WNOWAIT preserves PID/group authority until signaling is finished.
        unsafe {
            let mut info: libc::siginfo_t = std::mem::zeroed();
            libc::waitid(
                libc::P_PID,
                self.child.id(),
                &mut info,
                libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
            ) == 0
                && info.si_pid() != 0
        }
    }
}
fn nonblocking(pipe: &impl AsRawFd) -> io::Result<()> {
    // SAFETY: pipe owns a valid descriptor; flags change only this owned handle.
    unsafe {
        let flags = libc::fcntl(pipe.as_raw_fd(), libc::F_GETFL);
        if flags < 0 || libc::fcntl(pipe.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) < 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}
fn read_pipe(pipe: &mut impl Read, eof: &mut bool) -> Result<Option<Vec<u8>>> {
    if *eof {
        return Ok(None);
    }
    let mut bytes = [0; 4096];
    match pipe.read(&mut bytes) {
        Ok(0) => {
            *eof = true;
            Ok(None)
        }
        Ok(n) => Ok(Some(bytes[..n].to_vec())),
        Err(e)
            if matches!(
                e.kind(),
                io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
            ) =>
        {
            Ok(None)
        }
        Err(_) => Err(io_error()),
    }
}
impl Session for NativeSession {
    fn send(&mut self, bytes: &[u8]) -> Result<()> {
        if self
            .write
            .len()
            .checked_add(bytes.len())
            .is_none_or(|n| n > MAX_WRITE)
        {
            return Err(limit());
        }
        self.write.extend_from_slice(bytes);
        Ok(())
    }
    fn poll(&mut self) -> Result<Chunk> {
        if !self.write.is_empty() {
            match self.stdin.as_mut().ok_or_else(io_error)?.write(&self.write) {
                Ok(0) => return Err(io_error()),
                Ok(n) => {
                    self.write.drain(..n);
                }
                Err(e)
                    if matches!(
                        e.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                    ) => {}
                Err(_) => return Err(io_error()),
            }
        }
        self.stderr_next = !self.stderr_next;
        if self.stderr_next
            && let Some(bytes) = read_pipe(&mut self.stderr, &mut self.stderr_eof)?
        {
            return Ok(Chunk::Stderr(bytes.len()));
        }
        if let Some(bytes) = read_pipe(&mut self.stdout, &mut self.stdout_eof)? {
            return Ok(Chunk::Stdout(bytes));
        }
        if let Some(bytes) = read_pipe(&mut self.stderr, &mut self.stderr_eof)? {
            return Ok(Chunk::Stderr(bytes.len()));
        }
        if self.stdout_eof {
            Ok(Chunk::Eof)
        } else {
            Ok(Chunk::Idle)
        }
    }
    fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
    fn wait_tick(&mut self) {
        std::thread::sleep(Duration::from_millis(5));
    }
    fn successful_exit(&self) -> bool {
        self.successful_exit
    }
    fn cleanup(&mut self) -> bool {
        clean_owned(self)
    }
}

// The lifecycle decisions below are shared by the real process and controlled
// outcome doubles; a fixture does not implement timeout or signaling policy.
trait CleanupEffects {
    fn close_input(&mut self);
    fn drain(&mut self);
    fn elapsed(&self) -> Duration;
    fn tick(&mut self);
    fn reaped(&self) -> bool;
    fn exited(&self) -> bool;
    fn signal(&mut self, signal: i32) -> bool;
    fn reap(&mut self) -> bool;
    fn confirmed(&self) -> bool;
}
fn clean_owned(process: &mut impl CleanupEffects) -> bool {
    process.close_input();
    let mut term = false;
    let mut kill = false;
    loop {
        process.drain();
        let elapsed = process.elapsed();
        if !process.reaped() {
            if elapsed >= Duration::from_secs(1) && !term {
                if !process.signal(libc::SIGTERM) {
                    return false;
                }
                term = true;
            }
            if elapsed >= Duration::from_secs(2) && !kill {
                if !process.signal(libc::SIGKILL) {
                    return false;
                }
                kill = true;
            }
            if process.exited() && (!process.signal(libc::SIGKILL) || !process.reap()) {
                return false;
            }
        }
        if process.reaped() && process.confirmed() {
            return true;
        }
        if elapsed >= CLEANUP {
            return false;
        }
        process.tick();
    }
}
impl CleanupEffects for NativeSession {
    fn close_input(&mut self) {
        self.stdin.take();
        self.write.clear();
        self.cleanup_start = Some(Instant::now());
    }
    fn drain(&mut self) {
        if self.pipes_ready {
            for _ in 0..MAX_BATCH {
                let _ = read_pipe(&mut self.stdout, &mut self.stdout_eof);
                let _ = read_pipe(&mut self.stderr, &mut self.stderr_eof);
            }
        }
    }
    fn elapsed(&self) -> Duration {
        self.cleanup_start.expect("cleanup has started").elapsed()
    }
    fn tick(&mut self) {
        Session::wait_tick(self);
    }
    fn reaped(&self) -> bool {
        self.reaped
    }
    fn exited(&self) -> bool {
        self.exited_without_reaping()
    }
    fn signal(&mut self, signal: i32) -> bool {
        NativeSession::signal(self, signal)
    }
    fn reap(&mut self) -> bool {
        let Ok(status) = self.child.wait() else {
            return false;
        };
        self.successful_exit = status.success();
        self.reaped = true;
        self.lease.take();
        true
    }
    fn confirmed(&self) -> bool {
        let lease_free =
            execution_lock(&self.directory).is_ok_and(|lease| lease.try_lock().is_ok());
        let Ok(group) = i32::try_from(self.child.id()) else {
            return false;
        };
        // SAFETY: signal zero only observes. No signal is sent after reaping.
        let group_gone = unsafe {
            libc::kill(-group, 0) == -1
                && io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
        };
        lease_free && group_gone && self.stdout_eof && self.stderr_eof
    }
}

pub(super) fn boot_identity() -> String {
    #[cfg(target_os = "macos")]
    {
        let mut boot = libc::timeval {
            tv_sec: 0,
            tv_usec: 0,
        };
        let mut size = std::mem::size_of::<libc::timeval>();
        // SAFETY: nul-terminated name and writable timeval with its exact size.
        if unsafe {
            libc::sysctlbyname(
                c"kern.boottime".as_ptr(),
                std::ptr::from_mut(&mut boot).cast(),
                &mut size,
                std::ptr::null_mut(),
                0,
            )
        } == 0
        {
            return format!("{}:{}", boot.tv_sec, boot.tv_usec);
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(value) = std::fs::read_to_string("/proc/sys/kernel/random/boot_id") {
            return value.trim().to_owned();
        }
    }
    String::new() // Unknown boot identity fences surviving launch evidence.
}

/// Explicit, synthetic supervisor fixture mode. It shares command construction,
/// byte framing, metadata sequencing, budgets and cleanup decisions with stdio.
struct FixtureSession {
    scenario: String,
    output: std::collections::VecDeque<Chunk>,
    elapsed: Duration,
    sent: Vec<Value>,
}
impl FixtureSession {
    fn start(scenario: &str, spec: &CommandSpec) -> Result<Self> {
        if !matches!(
            scenario,
            "success"
                | "missing"
                | "version"
                | "auth"
                | "configuration"
                | "timeout"
                | "cleanup"
                | "malformed"
                | "oversized"
                | "exit"
        ) {
            return Err(RuntimeError::new(
                ErrorCode::InvalidRequest,
                "Unknown Codex fixture scenario. Use the documented private-state fixture instructions.",
            ));
        }
        let mut output = std::collections::VecDeque::new();
        if spec.args == ["--version"] {
            output.push_back(Chunk::Stdout(if scenario == "version" {
                b"codex-cli 0.0.0\n".to_vec()
            } else {
                format!("codex-cli {SUPPORTED_VERSION}\n").into_bytes()
            }));
            output.push_back(Chunk::Eof);
        }
        Ok(Self {
            scenario: scenario.into(),
            output,
            elapsed: Duration::ZERO,
            sent: vec![],
        })
    }
}
impl Session for FixtureSession {
    fn send(&mut self, bytes: &[u8]) -> Result<()> {
        let request: Value = serde_json::from_slice(bytes).map_err(|_| protocol::failure())?;
        if let Some(id) = request.get("id").and_then(Value::as_i64) {
            let mut response: Value = include_str!("fixtures/success.jsonl")
                .lines()
                .nth(usize::try_from(id - 1).map_err(|_| protocol::failure())?)
                .ok_or_else(protocol::failure)
                .and_then(|s| serde_json::from_str(s).map_err(|_| protocol::failure()))?;
            if self.scenario == "timeout" {
                self.sent.push(request);
                return Ok(());
            }
            if self.scenario == "auth" && id == 2 {
                response["result"]["account"] = Value::Null;
            }
            if self.scenario == "configuration" && id == 3 {
                response = json!({"id":id,"error":{"code":-32603,"message":"synthetic configuration failure"}});
            }
            let bytes = if self.scenario == "malformed" {
                b"invalid\n".to_vec()
            } else if self.scenario == "oversized" {
                vec![b'x'; protocol::MAX_FRAME + 1]
            } else {
                let mut b = serde_json::to_vec(&response).map_err(|_| protocol::failure())?;
                b.push(b'\n');
                b
            };
            self.output.push_back(Chunk::Stdout(bytes));
        }
        self.sent.push(request);
        Ok(())
    }
    fn poll(&mut self) -> Result<Chunk> {
        Ok(self.output.pop_front().unwrap_or(Chunk::Idle))
    }
    fn elapsed(&self) -> Duration {
        self.elapsed
    }
    fn wait_tick(&mut self) {
        self.elapsed += Duration::from_secs(1);
    }
    fn cleanup(&mut self) -> bool {
        self.scenario != "cleanup" || self.sent.is_empty()
    }
    fn successful_exit(&self) -> bool {
        self.scenario != "exit"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Locations {
        available: Vec<PathBuf>,
    }
    impl Discovery for Locations {
        fn path(&self) -> Vec<PathBuf> {
            vec![PathBuf::from("/path first"), PathBuf::from("/second")]
        }
        fn home(&self) -> Option<PathBuf> {
            Some(PathBuf::from("/home"))
        }
        fn executable(&self, p: &Path) -> bool {
            self.available.iter().any(|a| a == p)
        }
    }
    fn runner(scenario: &str) -> Runner {
        Runner {
            directory: "/private/fixture".into(),
            stop: Arc::new(AtomicBool::new(false)),
            quarantine: vec![],
            fixture: Some(scenario.into()),
        }
    }
    #[test]
    fn discovery_obeys_override_path_and_standard_locations_without_invalid_override_fallback() {
        let effects = Locations {
            available: vec!["/second/codex".into(), "/home/.local/bin/codex".into()],
        };
        assert_eq!(
            discover(&Configuration::default(), &effects).unwrap(),
            Path::new("/second/codex")
        );
        assert_eq!(
            discover(
                &Configuration {
                    revision: 1,
                    executable: Some("/invalid".into())
                },
                &effects
            )
            .unwrap_err()
            .code,
            ErrorCode::MissingExecutable
        );
        assert_eq!(
            discover(
                &Configuration::default(),
                &Locations {
                    available: vec!["/home/.local/bin/codex".into()]
                }
            )
            .unwrap(),
            Path::new("/home/.local/bin/codex")
        );
    }
    #[test]
    fn real_command_construction_passes_executable_and_arguments_directly() {
        let spec = command_spec(
            Path::new("/app with spaces/codex"),
            Path::new("/private/state"),
            false,
        );
        assert_eq!(spec.executable, Path::new("/app with spaces/codex"));
        assert_eq!(spec.args, ["app-server"]);
        assert_eq!(
            command_spec(&spec.executable, &spec.directory, true).args,
            ["--version"]
        );
    }
    #[test]
    fn fixture_success_uses_real_sequence_parser_and_cleanup() {
        let mut r = runner("success");
        let mut stages = vec![];
        let d = r.check(&Configuration::default(), Uuid::from_u128(7), &mut |p| {
            stages.push(p)
        });
        assert!(d.passed);
        assert!(d.cleanup_confirmed);
        assert_eq!(d.billing, "ChatGPT account");
        assert_eq!(
            stages,
            [
                Phase::Discovering,
                Phase::Version,
                Phase::Initializing,
                Phase::Account,
                Phase::Configuration,
                Phase::Cleaning
            ]
        );
    }
    #[test]
    fn primary_synthetic_faults_keep_failure_and_cleanup_distinct() {
        for (scenario, code) in [
            ("missing", ErrorCode::MissingExecutable),
            ("version", ErrorCode::IncompatibleVersion),
            ("auth", ErrorCode::Authentication),
            ("configuration", ErrorCode::Configuration),
            ("timeout", ErrorCode::Timeout),
            ("malformed", ErrorCode::Protocol),
            ("oversized", ErrorCode::OutputLimit),
            ("exit", ErrorCode::Process),
            ("cleanup", ErrorCode::CleanupUnconfirmed),
        ] {
            let d =
                runner(scenario).check(&Configuration::default(), Uuid::from_u128(7), &mut |_| {});
            assert!(!d.passed, "{scenario}");
            assert_eq!(d.error.unwrap().code, code, "{scenario}");
            assert_eq!(d.cleanup_confirmed, scenario != "cleanup");
        }
    }
    #[test]
    fn metadata_only_commands_never_request_threads_turns_refresh_or_login() {
        let spec = command_spec(Path::new("/codex"), Path::new("/private"), false);
        let mut session = FixtureSession::start("success", &spec).unwrap();
        metadata(
            &mut session,
            &mut 0,
            &AtomicBool::new(false),
            OVERALL,
            &mut |_| {},
        )
        .unwrap();
        let methods = session
            .sent
            .iter()
            .map(|v| v["method"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            methods,
            ["initialize", "initialized", "account/read", "config/read"]
        );
        assert_eq!(session.sent[2]["params"], json!({"refreshToken":false}));
        assert_eq!(
            session.sent[3]["params"],
            json!({"includeLayers":false,"cwd":null})
        );
    }
    #[test]
    fn output_budget_includes_stdout_and_stderr_and_deadlines_need_no_real_waits() {
        let mut count = MAX_OUTPUT - 1;
        account_output(&mut count, 1).unwrap();
        assert!(account_output(&mut count, 1).is_err());
        let spec = command_spec(Path::new("/codex"), Path::new("/private"), false);
        let mut session = FixtureSession::start("timeout", &spec).unwrap();
        assert_eq!(
            metadata(
                &mut session,
                &mut 0,
                &AtomicBool::new(false),
                OVERALL,
                &mut |_| {}
            )
            .err()
            .unwrap()
            .code,
            ErrorCode::Timeout
        );
        assert_eq!(session.elapsed, INITIALIZE);
    }

    struct CleanupScenario {
        time: Duration,
        exits: Duration,
        reaped: bool,
        confirmed: bool,
        signal_ok: bool,
        calls: Vec<&'static str>,
    }
    impl CleanupEffects for CleanupScenario {
        fn close_input(&mut self) {
            self.calls.push("close");
        }
        fn drain(&mut self) {
            self.calls.push("drain");
        }
        fn elapsed(&self) -> Duration {
            self.time
        }
        fn tick(&mut self) {
            self.time += Duration::from_secs(1);
        }
        fn reaped(&self) -> bool {
            self.reaped
        }
        fn exited(&self) -> bool {
            self.time >= self.exits
        }
        fn signal(&mut self, signal: i32) -> bool {
            assert!(!self.reaped, "reaped process granted signaling authority");
            self.calls.push(if signal == libc::SIGTERM {
                "term"
            } else {
                "kill"
            });
            self.signal_ok
        }
        fn reap(&mut self) -> bool {
            self.calls.push("reap");
            self.reaped = true;
            true
        }
        fn confirmed(&self) -> bool {
            self.confirmed
        }
    }
    fn cleanup_scenario() -> CleanupScenario {
        CleanupScenario {
            time: Duration::ZERO,
            exits: Duration::ZERO,
            reaped: false,
            confirmed: true,
            signal_ok: true,
            calls: vec![],
        }
    }
    #[test]
    fn cleanup_signals_only_before_reaping_and_requires_descendant_evidence() {
        let mut s = cleanup_scenario();
        assert!(clean_owned(&mut s));
        assert_eq!(s.calls, ["close", "drain", "kill", "reap"]);
        let mut s = cleanup_scenario();
        s.confirmed = false;
        assert!(!clean_owned(&mut s));
        assert_eq!(s.time, CLEANUP);
        assert_eq!(s.calls.iter().filter(|c| **c == "kill").count(), 1);
    }
    #[test]
    fn bounded_cleanup_escalates_and_retains_unconfirmed_stop() {
        let mut s = cleanup_scenario();
        s.exits = Duration::from_secs(2);
        assert!(clean_owned(&mut s));
        assert!(s.calls.contains(&"term"));
        assert!(s.calls.contains(&"kill"));
        let mut s = cleanup_scenario();
        s.exits = Duration::MAX;
        assert!(!clean_owned(&mut s));
        assert_eq!(s.time, CLEANUP);
        assert!(!s.reaped);
        let mut s = cleanup_scenario();
        s.signal_ok = false;
        assert!(!clean_owned(&mut s));
        assert!(!s.reaped);
    }
}
