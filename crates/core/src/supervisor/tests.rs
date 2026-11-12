use super::*;

#[test]
fn state_creation_always_validates_existing_or_new_directories() {
    use std::cell::Cell;
    for created in [Ok(()), Err(io::ErrorKind::AlreadyExists.into())] {
        let checked = Cell::new(false);
        let result = prepare_state_with(
            Path::new("/private/state"),
            |_| created,
            |_| {
                checked.set(true);
                Err(io::ErrorKind::PermissionDenied.into())
            },
        );
        assert!(checked.get());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::PermissionDenied);
    }
    let result = prepare_state_with(
        Path::new("/private/state"),
        |_| Err(io::ErrorKind::PermissionDenied.into()),
        |_| panic!("must not continue after creation failure"),
    );
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::PermissionDenied);
}

fn directory() -> FileEvidence {
    FileEvidence {
        owned_by_user: true,
        mode: 0o700,
        directory: true,
        socket: false,
        regular: false,
        links: 1,
    }
}
fn socket() -> FileEvidence {
    FileEvidence {
        mode: 0o600,
        directory: false,
        socket: true,
        ..directory()
    }
}
fn lock() -> FileEvidence {
    FileEvidence {
        mode: 0o600,
        directory: false,
        regular: true,
        ..directory()
    }
}

#[test]
fn state_and_socket_privacy_reject_wrong_owner_public_modes_and_symlinks() {
    let path = Path::new("/private/state");
    validate_private(path, &directory(), true).unwrap();
    validate_private(path, &socket(), false).unwrap();
    for invalid in [
        FileEvidence {
            owned_by_user: false,
            ..directory()
        },
        FileEvidence {
            mode: 0o755,
            ..directory()
        },
        FileEvidence {
            directory: false,
            ..directory()
        },
    ] {
        assert_eq!(
            validate_private(path, &invalid, true).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
    }
    for invalid in [
        FileEvidence {
            owned_by_user: false,
            ..socket()
        },
        FileEvidence {
            mode: 0o666,
            ..socket()
        },
        FileEvidence {
            socket: false,
            ..socket()
        },
    ] {
        assert_eq!(
            validate_private(path, &invalid, false).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
    }
}

#[test]
fn owner_lock_must_be_private_owned_regular_and_have_one_link() {
    validate_lock(&lock()).unwrap();
    for invalid in [
        FileEvidence {
            owned_by_user: false,
            ..lock()
        },
        FileEvidence {
            mode: 0o644,
            ..lock()
        },
        FileEvidence {
            regular: false,
            ..lock()
        },
        FileEvidence { links: 2, ..lock() },
    ] {
        assert_eq!(
            validate_lock(&invalid).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
    }
}

struct OwnerScenario {
    calls: Vec<&'static str>,
    acquired: io::Result<()>,
    existing_socket: io::Result<FileEvidence>,
    fail_at: Option<&'static str>,
}
impl OwnerScenario {
    fn new() -> Self {
        Self {
            calls: vec![],
            acquired: Ok(()),
            existing_socket: Err(io::ErrorKind::NotFound.into()),
            fail_at: None,
        }
    }
    fn call(&mut self, call: &'static str) -> io::Result<()> {
        self.calls.push(call);
        if self.fail_at == Some(call) {
            Err(io::ErrorKind::PermissionDenied.into())
        } else {
            Ok(())
        }
    }
}
impl OwnerEffects for OwnerScenario {
    type Lock = ();
    type Listener = ();
    fn prepare(&mut self, _: &Path) -> io::Result<()> {
        self.call("prepare")
    }
    fn open_lock(&mut self, _: &Path) -> io::Result<()> {
        self.call("open_lock")
    }
    fn acquire(&mut self, _: &()) -> io::Result<()> {
        self.call("acquire")?;
        std::mem::replace(&mut self.acquired, Ok(()))
    }
    fn socket_info(&mut self, socket: &Path) -> io::Result<FileEvidence> {
        assert_eq!(socket, Path::new("/private/state/supervisor.sock"));
        self.call("socket_info")?;
        std::mem::replace(
            &mut self.existing_socket,
            Err(io::ErrorKind::NotFound.into()),
        )
    }
    fn remove_socket(&mut self, _: &Path) -> io::Result<()> {
        self.call("remove_socket")
    }
    fn bind_socket(&mut self, _: &Path) -> io::Result<()> {
        self.call("bind_socket")
    }
    fn make_private(&mut self, _: &Path) -> io::Result<()> {
        self.call("make_private")
    }
}

#[test]
fn competing_owner_is_rejected_before_touching_the_endpoint() {
    let mut scenario = OwnerScenario::new();
    scenario.acquired = Err(io::ErrorKind::WouldBlock.into());
    let error = admit_owner(Path::new("/private/state"), &mut scenario).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::AddrInUse);
    assert!(error.to_string().contains("do not remove owner.lock"));
    assert_eq!(scenario.calls, ["prepare", "open_lock", "acquire"]);
}

#[test]
fn admitted_owner_alone_may_replace_a_private_stale_socket() {
    let mut scenario = OwnerScenario::new();
    scenario.existing_socket = Ok(socket());
    admit_owner(Path::new("/private/state"), &mut scenario).unwrap();
    assert_eq!(
        scenario.calls,
        [
            "prepare",
            "open_lock",
            "acquire",
            "socket_info",
            "remove_socket",
            "bind_socket",
            "make_private"
        ]
    );
    let mut absent = OwnerScenario::new();
    admit_owner(Path::new("/private/state"), &mut absent).unwrap();
    assert!(!absent.calls.contains(&"remove_socket"));
}

#[test]
fn unsafe_or_unreadable_socket_is_retained_and_not_replaced() {
    for existing in [
        Ok(FileEvidence {
            mode: 0o666,
            ..socket()
        }),
        Ok(FileEvidence {
            socket: false,
            ..socket()
        }),
        Err(io::ErrorKind::PermissionDenied.into()),
    ] {
        let mut scenario = OwnerScenario::new();
        scenario.existing_socket = existing;
        assert!(admit_owner(Path::new("/private/state"), &mut scenario).is_err());
        assert_eq!(
            scenario.calls,
            ["prepare", "open_lock", "acquire", "socket_info"]
        );
    }
}

#[test]
fn admission_stops_on_each_filesystem_failure() {
    let operations = [
        "prepare",
        "open_lock",
        "acquire",
        "socket_info",
        "remove_socket",
        "bind_socket",
        "make_private",
    ];
    for (index, operation) in operations.iter().enumerate() {
        let mut scenario = OwnerScenario::new();
        scenario.existing_socket = Ok(socket());
        scenario.fail_at = Some(operation);
        assert!(admit_owner(Path::new("/private/state"), &mut scenario).is_err());
        assert_eq!(scenario.calls, operations[..=index]);
    }
}

#[test]
fn owner_probe_distinguishes_held_lock_from_access_failure() {
    for (outcome, expected) in [
        (Ok(()), Ok(false)),
        (Err(io::ErrorKind::WouldBlock.into()), Ok(true)),
        (
            Err(io::ErrorKind::PermissionDenied.into()),
            Err(io::ErrorKind::PermissionDenied),
        ),
    ] {
        let mut scenario = OwnerScenario::new();
        scenario.acquired = outcome;
        let actual =
            probe_owner(Path::new("/private/state"), &mut scenario).map_err(|error| error.kind());
        assert_eq!(actual, expected);
        assert_eq!(scenario.calls, ["open_lock", "acquire"]);
    }
}
