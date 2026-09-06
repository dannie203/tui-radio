use std::time::Duration;

pub const GITHUB_REPO: &str = "dannie203/tui-radio";

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub release_url: String,
}

/// Checks GitHub API for the latest published release.
/// Runs non-blocking in the background with a strict 4-second timeout.
pub async fn check_for_updates() -> Option<UpdateInfo> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(4))
        .user_agent(format!("boombox-rs/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .ok()?;

    let url = format!("https://api.github.com/repos/{}/releases/latest", GITHUB_REPO);
    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }

    let json: serde_json::Value = resp.json().await.ok()?;
    let tag = json.get("tag_name")?.as_str()?.trim_start_matches('v');
    let html_url = json.get("html_url")?.as_str()?.to_string();

    let current = env!("CARGO_PKG_VERSION");

    if is_newer_version(tag, current) {
        Some(UpdateInfo {
            current_version: current.to_string(),
            latest_version: tag.to_string(),
            release_url: html_url,
        })
    } else {
        None
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Version {
    numbers: Vec<u32>,
    pre: Option<String>,
}

fn parse_version(v: &str) -> Version {
    let clean = v.trim().trim_start_matches('v');
    let (num_part, pre_part) = match clean.find('-') {
        Some(idx) => (&clean[..idx], Some(clean[idx + 1..].to_lowercase())),
        None => (clean, None),
    };
    let numbers = num_part
        .split('.')
        .filter_map(|p| p.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse::<u32>().ok())
        .collect();
    Version {
        numbers,
        pre: pre_part,
    }
}

/// Robust SemVer version comparison helper supporting pre-release tags (-alpha, -beta, -rc)
pub fn is_newer_version(remote: &str, current: &str) -> bool {
    let rem = parse_version(remote);
    let cur = parse_version(current);

    // Rule: If app is on a Stable release (no pre-release suffix),
    // automatically skip all remote pre-releases (no noisy beta notifications).
    if cur.pre.is_none() && rem.pre.is_some() {
        return false;
    }

    // Compare core version numbers (major.minor.patch)
    let min_len = rem.numbers.len().min(cur.numbers.len());
    for i in 0..min_len {
        if rem.numbers[i] > cur.numbers[i] {
            return true;
        } else if rem.numbers[i] < cur.numbers[i] {
            return false;
        }
    }

    if rem.numbers.len() != cur.numbers.len() {
        return rem.numbers.len() > cur.numbers.len();
    }

    // If core versions match:
    // If current is pre-release (e.g. 3.8.6-beta) and remote is final stable (3.8.6), remote is newer!
    if cur.pre.is_some() && rem.pre.is_none() {
        return true;
    }

    // If both have pre-releases: e.g. "rc.1" > "beta"
    if let (Some(r_pre), Some(c_pre)) = (rem.pre, cur.pre) {
        return r_pre > c_pre;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("3.8.4", "3.8.3"));
        assert!(is_newer_version("3.9.0", "3.8.3"));
        assert!(is_newer_version("4.0.0", "3.8.3"));
        assert!(!is_newer_version("3.8.3", "3.8.3"));
        assert!(!is_newer_version("3.8.2", "3.8.3"));
    }

    #[test]
    fn test_prerelease_skipping() {
        // Stable current version should ignore remote beta/rc
        assert!(!is_newer_version("3.8.6-beta", "3.8.5"));
        assert!(!is_newer_version("3.9.0-rc.1", "3.8.5"));
        assert!(!is_newer_version("4.0.0-alpha", "3.8.5"));

        // Pre-release current version should upgrade to stable
        assert!(is_newer_version("3.8.6", "3.8.6-beta"));
        assert!(is_newer_version("3.8.6-rc", "3.8.6-beta"));
    }
}
