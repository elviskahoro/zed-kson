//! Node.js runtime resolution for the KSON language server.
//!
//! The KSON server is a Node script, so launching it requires a usable
//! Node runtime. Resolution order:
//!
//! 1. Zed's bundled Node ([`zed::node_binary_path`]) — maintained by Zed,
//!    so it is the most reliable candidate and does not depend on the
//!    user's `PATH`.
//! 2. `node` on the worktree's `PATH` ([`Worktree::which`]).
//!
//! Every candidate is version-checked by running `node --version`; the
//! server declares `engines.node: "24.x"` (upstream pin
//! `928961c0669166b3852c5f67019ff0ceb57b7599`, see
//! `docs/compatibility-inventory.md`), so majors below 24 are rejected.
//! All failures are actionable: they name the prerequisite to install and
//! the setting to override.

use zed_extension_api::{self as zed, Worktree};

/// Minimum Node major version accepted for the KSON server.
///
/// The server's `package.json` declares `engines.node: "24.x"` at the
/// upstream pin. Newer majors are accepted (the runtime verification in
/// `docs/launch-matrix.md` exercised Node 26); older majors are not.
pub const MIN_NODE_MAJOR: u32 = 24;

/// A resolved, version-checked Node.js runtime.
pub struct NodeRuntime {
    /// Absolute path of the `node` executable.
    pub path: String,
}

/// Parses the output of `node --version` into `(major, minor, patch)`.
///
/// `node` prints e.g. `v24.1.2`; the patch component is optional and
/// defaults to 0. Returns `None` for anything that is not a `vX.Y` /
/// `vX.Y.Z` version string.
pub fn parse_node_version(output: &str) -> Option<(u32, u32, u32)> {
    let trimmed = output.trim();
    let rest = trimmed.strip_prefix('v')?;
    let mut parts = rest.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next().unwrap_or("0").parse().ok()?;
    Some((major, minor, patch))
}

/// Whether a parsed Node version is new enough for the KSON server.
pub fn is_supported_version(version: (u32, u32, u32)) -> bool {
    version.0 >= MIN_NODE_MAJOR
}

/// Resolves a usable Node runtime, or fails with an actionable error.
pub fn resolve_node(worktree: &Worktree) -> Result<NodeRuntime, String> {
    let mut candidates: Vec<String> = Vec::new();
    if let Ok(path) = zed::node_binary_path() {
        candidates.push(path);
    }
    if let Some(path) = worktree.which("node") {
        candidates.push(path);
    }
    if candidates.is_empty() {
        return Err(no_node_error());
    }

    let mut failures: Vec<String> = Vec::new();
    for path in candidates {
        match check_node_version(&path) {
            Ok(()) => return Ok(NodeRuntime { path }),
            Err(error) => failures.push(error),
        }
    }
    Err(all_candidates_failed_error(&failures))
}

/// Runs `node --version` for one candidate and applies the version gate.
fn check_node_version(path: &str) -> Result<(), String> {
    let output = zed::process::Command::new(path)
        .arg("--version")
        .output()
        .map_err(|error| node_unusable_error(path, &error))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let version = stdout.trim().to_string();
    match parse_node_version(&version) {
        Some(parsed) if is_supported_version(parsed) => Ok(()),
        Some(_) => Err(node_too_old_error(path, &version)),
        None => Err(node_unparsable_error(path, &stdout)),
    }
}

// Actionable error message builders. Kept as pure functions so their
// wording (which must name the prerequisite/setting to fix) is tested.

fn no_node_error() -> String {
    "kson-language-server: no Node.js runtime found. The KSON server \
     requires Node.js 24 or newer. Neither Zed's bundled Node nor `node` \
     on PATH was available. Install Node.js 24+ from https://nodejs.org \
     and make sure it is on PATH, or update Zed so its bundled Node is \
     included."
        .to_string()
}

fn node_unusable_error(path: &str, reason: &str) -> String {
    format!(
        "kson-language-server: Node.js runtime at '{path}' could not be \
         executed (`node --version` failed: {reason}). Fix the executable \
         or install a working Node.js 24+ on PATH and restart Zed."
    )
}

fn node_too_old_error(path: &str, version: &str) -> String {
    format!(
        "kson-language-server: Node.js at '{path}' is {version}, but the \
         KSON server requires Node.js {MIN_NODE_MAJOR} or newer \
         (upstream declares engines.node \"24.x\"). Install Node.js \
         {MIN_NODE_MAJOR}+ from https://nodejs.org on PATH, or update Zed \
         so its bundled Node is current."
    )
}

fn node_unparsable_error(path: &str, stdout: &str) -> String {
    format!(
        "kson-language-server: Node.js at '{path}' reported an \
         unrecognizable version ({stdout:?}). Ensure a stock Node.js \
         {MIN_NODE_MAJOR}+ install is on PATH."
    )
}

fn all_candidates_failed_error(failures: &[String]) -> String {
    format!(
        "kson-language-server: no usable Node.js runtime (each candidate \
         failed the Node {MIN_NODE_MAJOR}+ version check):\n  - {}",
        failures.join("\n  - ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_stock_version_output() {
        assert_eq!(
            parse_node_version("v24.1.2\n"),
            Some((24, 1, 2)),
            "stock `node --version` output with trailing newline"
        );
        assert_eq!(parse_node_version("v26.2.0"), Some((26, 2, 0)));
    }

    #[test]
    fn parses_two_component_versions() {
        assert_eq!(parse_node_version("v24.1\n"), Some((24, 1, 0)));
    }

    #[test]
    fn rejects_non_version_output() {
        assert_eq!(parse_node_version("not a version"), None);
        assert_eq!(parse_node_version(""), None);
        assert_eq!(parse_node_version("v24"), None, "major-only is rejected");
        assert_eq!(
            parse_node_version("vTwenty.one"),
            None,
            "non-numeric components are rejected"
        );
    }

    #[test]
    fn version_gate_matches_upstream_requirement() {
        assert!(is_supported_version((24, 0, 0)));
        assert!(is_supported_version((26, 2, 0)));
        assert!(!is_supported_version((23, 11, 0)));
    }

    #[test]
    fn errors_name_the_prerequisite_to_fix() {
        assert!(
            no_node_error().contains("Node.js 24"),
            "missing-node error must say what to install"
        );
        assert!(
            node_too_old_error("/usr/bin/node", "v18.0.0")
                .contains("https://nodejs.org"),
            "old-node error must say where to get a newer Node"
        );
        assert!(
            node_unusable_error("/opt/node", "permission denied")
                .contains("'/opt/node'"),
            "unusable-node error must name the failing path"
        );
        assert!(
            all_candidates_failed_error(&["a".to_string(), "b".to_string()])
                .contains("  - a\n  - b"),
            "combined error must list every candidate failure"
        );
    }
}
