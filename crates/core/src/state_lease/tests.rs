use super::*;
use crate::private_state::validate_private;

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
        regular: false,
        links: 1,
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
fn state_privacy_rejects_wrong_owner_public_modes_and_symlinks() {
    let path = Path::new("/private/state");
    validate_private(path, &directory()).unwrap();
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
            validate_private(path, &invalid).unwrap_err().kind(),
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

#[test]
fn native_lease_admission_stops_at_failure_and_preserves_lock_errors() {
    use std::cell::RefCell;
    for failure in ["prepare", "open", "lock", "none"] {
        let calls = RefCell::new(Vec::new());
        let call = |name| {
            calls.borrow_mut().push(name);
            if name == failure {
                Err(io::Error::from_raw_os_error(libc::EACCES))
            } else {
                Ok(())
            }
        };
        let result = admit(
            Path::new("/private/state"),
            |_| call("prepare"),
            |_| call("open"),
            |_| call("lock"),
        );
        let expected = match failure {
            "prepare" => 1,
            "open" => 2,
            _ => 3,
        };
        assert_eq!(*calls.borrow(), ["prepare", "open", "lock"][..expected]);
        if failure == "none" {
            assert!(result.is_ok());
        } else {
            assert_eq!(result.unwrap_err().raw_os_error(), Some(libc::EACCES));
        }
    }
    let error = admit(
        Path::new("/private/state"),
        |_| Ok(()),
        |_| Ok(()),
        |_| Err(io::ErrorKind::WouldBlock.into()),
    )
    .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
    assert!(error.to_string().contains("never remove owner.lock"));
}
