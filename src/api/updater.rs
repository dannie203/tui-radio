use std::time::Duration;

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

    let url = "https://api.github.com/repos/dannie203/tui-radio/releases/latest";
    let resp = client.get(url).send().await.ok()?;
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

/// Robust SemVer version comparison helper (e.g. "3.8.4" > "3.8.3")
pub fn is_newer_version(remote: &str, current: &str) -> bool {
    let parse_parts = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|p| {
                p.chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse::<u32>()
                    .ok()
            })
            .collect()
    };

    let rem_parts = parse_parts(remote);
    let cur_parts = parse_parts(current);

    for (r, c) in rem_parts.iter().zip(cur_parts.iter()) {
        if r > c {
            return true;
        } else if r < c {
            return false;
        }
    }
    rem_parts.len() > cur_parts.len()
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
}
