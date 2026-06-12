//! Git URL normalization for the repo registry (METIS-I-0031 design D5).
//!
//! A repository is referenced by many URL spellings — `git@host:org/repo.git`,
//! `https://host/org/repo.git`, `ssh://git@host/org/repo` — that all denote the
//! same thing. [`normalize_git_url`] collapses them to a single canonical match
//! key so a client's `git remote` can be matched against the registry.

/// Normalize a git URL to a canonical match key.
///
/// The key is `host/path` with: scheme and userinfo removed, scp-style
/// (`user@host:path`) handled, host lowercased, a `.git` suffix and any
/// trailing slash stripped. Port, if present, is dropped (it rarely
/// distinguishes a repo and varies between access methods).
///
/// ```
/// use metis_core::repo::normalize_git_url;
/// assert_eq!(normalize_git_url("git@github.com:org/Repo.git"), "github.com/org/Repo");
/// assert_eq!(normalize_git_url("https://github.com/org/Repo.git"), "github.com/org/Repo");
/// assert_eq!(normalize_git_url("ssh://git@github.com/org/Repo/"), "github.com/org/Repo");
/// ```
pub fn normalize_git_url(url: &str) -> String {
    let s = url.trim();

    // Split into (authority, path). Handle the three shapes.
    let (authority, path) = if let Some(rest) = s
        .strip_prefix("https://")
        .or_else(|| s.strip_prefix("http://"))
        .or_else(|| s.strip_prefix("ssh://"))
        .or_else(|| s.strip_prefix("git://"))
    {
        // scheme://[user@]host[:port]/path
        match rest.split_once('/') {
            Some((auth, p)) => (auth, p),
            None => (rest, ""),
        }
    } else if let Some((auth, p)) = scp_split(s) {
        // scp-like: [user@]host:path
        (auth, p)
    } else {
        // Bare "host/path" or just a path; treat everything before the first
        // '/' as authority.
        match s.split_once('/') {
            Some((auth, p)) => (auth, p),
            None => (s, ""),
        }
    };

    // Drop userinfo and port from the authority; lowercase the host.
    let host = authority
        .rsplit_once('@')
        .map(|(_, h)| h)
        .unwrap_or(authority);
    let host = host.split_once(':').map(|(h, _)| h).unwrap_or(host);
    let host = host.to_ascii_lowercase();

    // Clean the path: trim slashes, strip a trailing ".git".
    let path = path.trim_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);

    if path.is_empty() {
        host
    } else {
        format!("{host}/{path}")
    }
}

/// Recognize scp-style `[user@]host:path` (no scheme): there is a `:` that comes
/// before any `/`, and the part after `:` is not a port (i.e. not all digits).
fn scp_split(s: &str) -> Option<(&str, &str)> {
    let colon = s.find(':')?;
    if let Some(slash) = s.find('/') {
        if slash < colon {
            return None; // a path-y URL, not scp
        }
    }
    let (auth, rest) = s.split_at(colon);
    let path = &rest[1..]; // drop ':'
                           // `host:1234` style (a port with no path) is not scp.
    if !path.is_empty() && path.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some((auth, path))
}
