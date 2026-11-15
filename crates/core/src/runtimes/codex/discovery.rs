//! Bounded executable discovery. Credential scope is never inferred from shell output.
use super::process::Launch;
use crate::runtimes::{SetupFailure, SetupProblem};
use std::{
    ffi::OsString,
    future::Future,
    path::{Path, PathBuf},
};

pub(super) trait Search: Sync {
    fn resolve(&self, path: &Path) -> impl Future<Output = Option<PathBuf>> + Send;
    fn shell_path(&self) -> impl Future<Output = Option<OsString>> + Send;
}

pub(super) async fn discover(
    launch: &Launch,
    selected: Option<&Path>,
    search: &impl Search,
) -> Result<(Launch, &'static str), SetupFailure> {
    // Explicit choices never silently fall back to another installation.
    if let Some(path) = selected {
        let (mut found, source) = resolve(launch, path, "selected", search).await?;
        if !launch.fixture
            && let Some(paths) = search.shell_path().await
        {
            found.environment.retain(|(name, _)| name != "PATH");
            found.environment.push(("PATH".into(), paths));
        }
        return Ok((found, source));
    }
    if launch.fixture {
        return resolve(launch, &launch.program, "fixture", search).await;
    }
    if let Some(path) = launch
        .environment
        .iter()
        .find(|(key, _)| key == "PATH")
        .map(|(_, value)| value)
        && let Some(found) = in_path(launch, path, "path", search).await
    {
        return Ok(found);
    }
    if let Some(path) = search.shell_path().await
        && let Some(found) = in_path(launch, &path, "shell", search).await
    {
        return Ok(found);
    }
    let home = launch
        .environment
        .iter()
        .find(|(key, _)| key == "HOME")
        .map(|(_, value)| PathBuf::from(value));
    for path in [
        Some(PathBuf::from("/opt/homebrew/bin/codex")),
        Some(PathBuf::from("/usr/local/bin/codex")),
        home.map(|home| home.join(".local/bin/codex")),
    ]
    .into_iter()
    .flatten()
    {
        if let Ok(found) = resolve(launch, &path, "conventional", search).await {
            return Ok(found);
        }
    }
    Err(missing())
}
async fn in_path(
    launch: &Launch,
    paths: &std::ffi::OsStr,
    source: &'static str,
    search: &impl Search,
) -> Option<(Launch, &'static str)> {
    // Ignore relative/empty entries: discovery must not inspect project directories.
    for path in std::env::split_paths(paths)
        .filter(|path| path.is_absolute())
        .take(256)
    {
        if let Ok((mut found, source)) = resolve(launch, &path.join("codex"), source, search).await
        {
            found.environment.retain(|(name, _)| name != "PATH");
            found
                .environment
                .push(("PATH".into(), paths.to_os_string()));
            return Some((found, source));
        }
    }
    None
}
async fn resolve(
    launch: &Launch,
    path: &Path,
    source: &'static str,
    search: &impl Search,
) -> Result<(Launch, &'static str), SetupFailure> {
    let program = search.resolve(path).await.ok_or_else(missing)?;
    let mut found = launch.clone();
    found.program = program;
    Ok((found, source))
}
fn missing() -> SetupFailure {
    SetupFailure::at(
        SetupProblem::MissingExecutable,
        "Codex could not be found at the selected location or in the searched executable paths",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    struct Paths {
        paths: Mutex<Vec<PathBuf>>,
        shell: OsString,
        available: Vec<(PathBuf, PathBuf)>,
    }
    impl Search for Paths {
        async fn resolve(&self, path: &Path) -> Option<PathBuf> {
            self.paths.lock().unwrap().push(path.to_owned());
            self.available
                .iter()
                .find(|(logical, _)| logical == path)
                .map(|(_, resolved)| resolved.clone())
        }
        async fn shell_path(&self) -> Option<OsString> {
            Some(self.shell.clone())
        }
    }
    fn launch() -> Launch {
        Launch {
            program: "codex".into(),
            directory: "/neutral".into(),
            environment: vec![
                ("PATH".into(), "/gui".into()),
                ("CODEX_HOME".into(), "/credentials".into()),
            ],
            fixture: false,
        }
    }
    #[test]
    fn missing_saved_selection_never_uses_an_available_automatic_installation() {
        let search = Paths {
            paths: Mutex::default(),
            shell: "/shell".into(),
            available: vec![("/gui/codex".into(), "/actual/codex".into())],
        };
        run(async {
            assert!(
                discover(&launch(), Some(Path::new("/gone/codex")), &search)
                    .await
                    .is_err()
            );
        });
        assert_eq!(
            *search.paths.lock().unwrap(),
            [PathBuf::from("/gone/codex")]
        );
    }
    #[test]
    fn fresh_shell_lookup_and_symlink_resolution_preserve_credential_scope() {
        let search = Paths {
            paths: Mutex::default(),
            shell: "/shell".into(),
            available: vec![("/shell/codex".into(), "/version-two/codex".into())],
        };
        run(async {
            let (found, source) = discover(&launch(), None, &search).await.unwrap();
            assert_eq!(source, "shell");
            assert_eq!(found.program, Path::new("/version-two/codex"));
            assert_eq!(found.credential_scope(), "/credentials");
            assert!(
                found
                    .environment
                    .contains(&("PATH".into(), "/shell".into()))
            );
            let missing = Paths {
                paths: Mutex::default(),
                shell: "/shell".into(),
                available: vec![],
            };
            assert!(discover(&launch(), None, &missing).await.is_err());
        });
    }
    #[test]
    fn check_again_finds_new_installations_and_resolves_updated_links() {
        run(async {
            let missing = Paths {
                paths: Mutex::default(),
                shell: "/shell".into(),
                available: vec![],
            };
            let template = launch();
            assert!(discover(&template, None, &missing).await.is_err());
            for version in ["/v1/codex", "/v2/codex"] {
                let installed = Paths {
                    paths: Mutex::default(),
                    shell: "/shell".into(),
                    available: vec![
                        ("/gui/codex".into(), version.into()),
                        ("/shell/codex".into(), "/other/codex".into()),
                    ],
                };
                let (found, source) = discover(&template, None, &installed).await.unwrap();
                assert_eq!(source, "path");
                assert_eq!(found.program, Path::new(version));
            }
        });
    }
    fn run(f: impl Future<Output = ()>) {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(f);
    }
}
