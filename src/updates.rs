//! Startup update check against the project's GitHub releases.
//!
//! The `releases/latest` web URL redirects to the tag of the newest release;
//! reading that redirect avoids both JSON parsing and API rate limits.

pub const REPO_URL: &str = "https://github.com/adamwhiles/readfence";

const RELEASES_LATEST_URL: &str = "https://github.com/adamwhiles/readfence/releases/latest";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateInfo {
    pub version: String,
    pub url: String,
    /// The git tag of the release, exactly as it appears in download URLs.
    pub tag: String,
}

/// What a finished update check learned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateCheckOutcome {
    Available(UpdateInfo),
    UpToDate,
    Failed,
    /// Flatpak installs update through the store; pointing those users at
    /// GitHub downloads would be wrong, so the check is skipped entirely.
    StoreManaged,
}

/// The state the updates menu reports; advanced by launch, periodic, and
/// manual checks alike.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateStatus {
    Unknown,
    Checking,
    UpToDate,
    Available,
    Failed,
    StoreManaged,
}

pub async fn check_for_updates() -> UpdateCheckOutcome {
    if std::path::Path::new("/.flatpak-info").exists() {
        return UpdateCheckOutcome::StoreManaged;
    }

    match tokio::task::spawn_blocking(fetch_latest_release).await {
        Ok(Some(info)) => {
            if is_newer(&info.version, env!("CARGO_PKG_VERSION")) {
                UpdateCheckOutcome::Available(info)
            } else {
                UpdateCheckOutcome::UpToDate
            }
        }
        _ => UpdateCheckOutcome::Failed,
    }
}

fn fetch_latest_release() -> Option<UpdateInfo> {
    let agent = ureq::AgentBuilder::new()
        .redirects(0)
        .timeout(std::time::Duration::from_secs(10))
        .user_agent(concat!("readfence/", env!("CARGO_PKG_VERSION")))
        .build();

    // Depending on configuration ureq surfaces a redirect either as a
    // response or as a status error; accept both.
    let response = match agent.get(RELEASES_LATEST_URL).call() {
        Ok(response) => response,
        Err(ureq::Error::Status(_, response)) => response,
        Err(_) => return None,
    };

    parse_release_location(response.header("Location")?)
}

fn parse_release_location(location: &str) -> Option<UpdateInfo> {
    // Without any release yet, GitHub redirects to /releases instead of a tag.
    let (_, tag) = location.split_once("/tag/")?;
    let tag = tag.trim();
    let version = tag.trim_start_matches('v');
    if version.is_empty() {
        return None;
    }

    Some(UpdateInfo {
        version: version.to_string(),
        url: location.to_string(),
        tag: tag.to_string(),
    })
}

/// Where an in-place install of a release stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallState {
    Idle,
    Running,
    /// The executable on disk is the new release; relaunching the recorded
    /// path finishes the update. The path is captured before the swap —
    /// afterwards `/proc/self/exe` reads "… (deleted)" on Linux.
    Done(std::path::PathBuf),
    Failed(String),
}

/// Downloads the release build for this platform, verifies its checksum,
/// and swaps it in for the running executable. Returns the executable's
/// path for the relaunch.
pub async fn install_update(info: UpdateInfo) -> Result<std::path::PathBuf, String> {
    tokio::task::spawn_blocking(move || download_and_install(&info))
        .await
        .map_err(|_| "the update task crashed".to_string())?
}

fn download_and_install(info: &UpdateInfo) -> Result<std::path::PathBuf, String> {
    let target = release_target().ok_or("no prebuilt release for this platform")?;
    let asset = release_asset_name(target);
    let base = format!("{REPO_URL}/releases/download/{}", info.tag);

    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(300))
        .user_agent(concat!("readfence/", env!("CARGO_PKG_VERSION")))
        .build();

    let archive = fetch_bytes(&agent, &format!("{base}/{asset}"), 256 * 1024 * 1024)?;
    let checksum = fetch_bytes(&agent, &format!("{base}/{asset}.sha256"), 4096)?;
    verify_sha256(&archive, &String::from_utf8_lossy(&checksum))?;

    let binary = extract_binary(&archive)?;
    replace_current_exe(&binary)
}

fn replace_current_exe(binary: &[u8]) -> Result<std::path::PathBuf, String> {
    let exe = std::env::current_exe()
        .map_err(|error| format!("couldn't locate the running executable: {error}"))?;
    let directory = exe
        .parent()
        .ok_or("couldn't locate the running executable")?;

    // Staging next to the executable keeps the final swap on one filesystem.
    let staging = directory.join(format!(".readfence-update-{}", std::process::id()));
    std::fs::write(&staging, binary)
        .map_err(|error| format!("couldn't stage the update (is the folder writable?): {error}"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Err(error) =
            std::fs::set_permissions(&staging, std::fs::Permissions::from_mode(0o755))
        {
            let _ = std::fs::remove_file(&staging);
            return Err(format!("couldn't stage the update: {error}"));
        }
    }

    let result = self_replace::self_replace(&staging)
        .map_err(|error| format!("couldn't replace the app binary: {error}"));
    let _ = std::fs::remove_file(&staging);
    result.map(|()| exe)
}

/// The cargo-dist target triple releases are published under.
fn release_target() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => Some("x86_64-unknown-linux-gnu"),
        ("linux", "aarch64") => Some("aarch64-unknown-linux-gnu"),
        ("macos", "x86_64") => Some("x86_64-apple-darwin"),
        ("macos", "aarch64") => Some("aarch64-apple-darwin"),
        ("windows", "x86_64") => Some("x86_64-pc-windows-msvc"),
        _ => None,
    }
}

fn release_asset_name(target: &str) -> String {
    let extension = if cfg!(windows) { "zip" } else { "tar.xz" };
    format!("readfence-{target}.{extension}")
}

fn fetch_bytes(agent: &ureq::Agent, url: &str, limit: u64) -> Result<Vec<u8>, String> {
    let response = agent
        .get(url)
        .call()
        .map_err(|error| format!("download failed: {error}"))?;
    let mut bytes = Vec::new();
    std::io::Read::read_to_end(
        &mut std::io::Read::take(response.into_reader(), limit),
        &mut bytes,
    )
    .map_err(|error| format!("download failed: {error}"))?;
    Ok(bytes)
}

/// Checks `data` against a `sha256sum`-style line (`<hex> *<filename>`).
fn verify_sha256(data: &[u8], expected_line: &str) -> Result<(), String> {
    use sha2::Digest;

    let expected = expected_line
        .split_whitespace()
        .next()
        .filter(|token| token.len() == 64 && token.chars().all(|c| c.is_ascii_hexdigit()))
        .ok_or("the release checksum file is malformed")?;

    let actual: String = sha2::Sha256::digest(data)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();

    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err("the downloaded update failed checksum verification".to_string())
    }
}

/// Pulls the `readfence` binary out of a release `.tar.xz`.
#[cfg(not(windows))]
fn extract_binary(archive: &[u8]) -> Result<Vec<u8>, String> {
    let mut tarball = Vec::new();
    lzma_rs::xz_decompress(&mut std::io::Cursor::new(archive), &mut tarball)
        .map_err(|error| format!("couldn't decompress the update: {error:?}"))?;

    let mut entries = tar::Archive::new(std::io::Cursor::new(&tarball));
    for entry in entries
        .entries()
        .map_err(|error| format!("couldn't read the update archive: {error}"))?
    {
        let mut entry =
            entry.map_err(|error| format!("couldn't read the update archive: {error}"))?;
        let is_binary = entry.header().entry_type().is_file()
            && entry
                .path()
                .ok()
                .is_some_and(|path| path.file_name().is_some_and(|name| name == "readfence"));
        if is_binary {
            let mut bytes = Vec::new();
            std::io::Read::read_to_end(&mut entry, &mut bytes)
                .map_err(|error| format!("couldn't read the update archive: {error}"))?;
            return Ok(bytes);
        }
    }
    Err("no readfence binary in the update archive".to_string())
}

/// Pulls `readfence.exe` out of a release `.zip`.
#[cfg(windows)]
fn extract_binary(archive: &[u8]) -> Result<Vec<u8>, String> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(archive))
        .map_err(|error| format!("couldn't read the update archive: {error}"))?;
    for index in 0..zip.len() {
        let mut file = zip
            .by_index(index)
            .map_err(|error| format!("couldn't read the update archive: {error}"))?;
        let is_binary = file
            .name()
            .rsplit(['/', '\\'])
            .next()
            .is_some_and(|name| name.eq_ignore_ascii_case("readfence.exe"));
        if is_binary {
            let mut bytes = Vec::new();
            std::io::Read::read_to_end(&mut file, &mut bytes)
                .map_err(|error| format!("couldn't read the update archive: {error}"))?;
            return Ok(bytes);
        }
    }
    Err("no readfence binary in the update archive".to_string())
}

fn is_newer(candidate: &str, current: &str) -> bool {
    match (parse_version(candidate), parse_version(current)) {
        (Some(candidate), Some(current)) => candidate > current,
        _ => false,
    }
}

/// Reads the leading `major.minor.patch` numbers, tolerating a `v` prefix
/// and trailing pre-release or build suffixes.
fn parse_version(version: &str) -> Option<(u64, u64, u64)> {
    let mut parts = version.trim().trim_start_matches('v').splitn(3, '.');
    let component = |part: Option<&str>| -> Option<u64> {
        let digits: String = part?.chars().take_while(char::is_ascii_digit).collect();
        digits.parse().ok()
    };
    Some((
        component(parts.next())?,
        component(parts.next())?,
        component(parts.next())?,
    ))
}

#[cfg(test)]
mod tests {
    use super::{is_newer, parse_release_location, parse_version, verify_sha256};

    #[test]
    fn parses_release_redirects() {
        let info =
            parse_release_location("https://github.com/adamwhiles/readfence/releases/tag/0.4.1")
                .unwrap();
        assert_eq!(info.version, "0.4.1");
        assert_eq!(info.tag, "0.4.1");

        let info =
            parse_release_location("https://github.com/adamwhiles/readfence/releases/tag/v1.2.0")
                .unwrap();
        assert_eq!(info.version, "1.2.0");
        assert_eq!(info.tag, "v1.2.0");

        assert!(
            parse_release_location("https://github.com/adamwhiles/readfence/releases").is_none()
        );
    }

    #[test]
    fn verifies_checksums() {
        // sha256 of the empty input.
        let empty = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert!(verify_sha256(b"", &format!("{empty} *readfence.tar.xz")).is_ok());
        assert!(verify_sha256(b"", &empty.to_uppercase()).is_ok());
        assert!(verify_sha256(b"content", &format!("{empty} *readfence.tar.xz")).is_err());
        assert!(verify_sha256(b"", "not a checksum").is_err());
        assert!(verify_sha256(b"", "").is_err());
    }

    #[test]
    fn parses_release_tags() {
        assert_eq!(parse_version("0.3.4"), Some((0, 3, 4)));
        assert_eq!(parse_version("v1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version("1.2.3-rc1"), Some((1, 2, 3)));
        assert_eq!(parse_version("not a version"), None);
        assert_eq!(parse_version("1.2"), None);
    }

    #[test]
    fn compares_versions() {
        assert!(is_newer("0.3.5", "0.3.4"));
        assert!(is_newer("0.4.0", "0.3.9"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(!is_newer("0.3.4", "0.3.4"));
        assert!(!is_newer("0.3.3", "0.3.4"));
        assert!(!is_newer("garbage", "0.3.4"));
    }
}
