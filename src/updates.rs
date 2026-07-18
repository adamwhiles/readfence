//! Startup update check against the project's GitHub releases.
//!
//! The `releases/latest` web URL redirects to the tag of the newest release;
//! reading that redirect avoids both JSON parsing and API rate limits.

const RELEASES_LATEST_URL: &str = "https://github.com/adamwhiles/readfence/releases/latest";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateInfo {
    pub version: String,
    pub url: String,
}

pub async fn check_for_update() -> Option<UpdateInfo> {
    // Flatpak installs update through the store; pointing those users at
    // GitHub downloads would be wrong, so skip the check entirely.
    if std::path::Path::new("/.flatpak-info").exists() {
        return None;
    }

    tokio::task::spawn_blocking(fetch_latest_release)
        .await
        .ok()
        .flatten()
        .filter(|info| is_newer(&info.version, env!("CARGO_PKG_VERSION")))
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

    let location = response.header("Location")?;
    // Without any release yet, GitHub redirects to /releases instead of a tag.
    let (_, tag) = location.split_once("/tag/")?;
    let version = tag.trim_start_matches('v').trim();
    if version.is_empty() {
        return None;
    }

    Some(UpdateInfo {
        version: version.to_string(),
        url: location.to_string(),
    })
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
    let mut component = |part: Option<&str>| -> Option<u64> {
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
    use super::{is_newer, parse_version};

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
