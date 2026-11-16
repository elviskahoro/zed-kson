//! Resolution of the KSON language-server script to launch.
//!
//! Resolution order (user override always wins, per WI-1 decision D2 in
//! `docs/compatibility-inventory.md`):
//!
//! 1. `lsp.kson-language-server.binary.path` — user-supplied, locally
//!    built `server.js`. Validated for existence/type/readability with
//!    actionable errors.
//! 2. A pinned upstream server distribution, downloaded through the
//!    extension API into a content-addressed cache keyed on the pinned
//!    version, so repeat launches never redownload and version upgrades
//!    download exactly once.
//!
//! Upstream kson-org/kson publishes no runnable server, so this
//! repository builds `server.js` from the commit pinned in
//! `server/KSON_COMMIT` (`scripts/build-server.sh`) and publishes it as a
//! GitHub release asset (`.github/workflows/server-release.yml`, tag
//! `server-v<version>`). The bundle is plain JavaScript, so one asset
//! serves every platform. `kson-lib` native archives are Kotlin library
//! builds, not server executables, and are never downloaded.

use std::path::{Path, PathBuf};

use zed_extension_api::{self as zed, settings::LspSettings, DownloadedFileType};

/// A pinned upstream server distribution.
pub struct ServerDistribution {
    /// The version this extension pins.
    pub version: &'static str,
    /// Base URL of the release artifacts (tag-interpolated).
    pub base_url: &'static str,
    /// How the artifact is packaged.
    pub file_type: DownloadedFileType,
}

/// The pinned server distribution. `version` is the upstream package
/// version plus the short upstream commit; bump it together with
/// `server/KSON_COMMIT` and push the matching `server-v<version>` tag.
pub const PINNED_DISTRIBUTION: Option<ServerDistribution> = Some(ServerDistribution {
    version: "0.5.0-dev.0-928961c",
    base_url: "https://github.com/elviskahoro/zed-kson/releases/download",
    file_type: DownloadedFileType::GzipTar,
});

/// The release asset. The bundle is platform-independent JavaScript.
pub const ASSET: &str = "kson-language-server.tar.gz";

/// Asset name for a platform, or `None` when the platform has no
/// artifact. The bundle runs anywhere Node.js 24+ runs, so every
/// platform Zed supports gets the same asset.
pub fn asset_name(_os: zed::Os, _arch: zed::Architecture) -> Option<&'static str> {
    Some(ASSET)
}

/// The cache layout for one pinned version, rooted at the extension's
/// install directory.
pub struct CachePaths {
    /// Directory holding this version's extracted artifact.
    pub dir: PathBuf,
    /// Marker file recording the installed version (avoids redownloads).
    pub marker: PathBuf,
    /// The server script inside the extracted artifact.
    pub script: PathBuf,
}

/// Computes the cache layout for `version` under `root`.
pub fn cache_paths(root: &Path, version: &str) -> CachePaths {
    let dir = root.join("server").join(version);
    CachePaths {
        marker: dir.join("INSTALLED_VERSION"),
        script: dir.join("server.js"),
        dir,
    }
}

/// Whether a marker file's contents match the pinned version (exactly,
/// ignoring surrounding whitespace).
pub fn marker_matches(contents: std::io::Result<String>, version: &str) -> bool {
    match contents {
        Ok(text) => text.trim() == version,
        Err(_) => false,
    }
}

/// Full download URL for a pinned version on a platform.
pub fn download_url(base_url: &str, version: &str, asset: &str) -> String {
    format!("{base_url}/server-v{version}/{asset}")
}

/// Resolves the server script: user override first, then the pinned
/// distribution's cache, installing it if needed. Returns the path of
/// the script to launch (the override when set, the cached pinned
/// artifact otherwise).
pub fn resolve_server_script(settings: &LspSettings) -> Result<String, String> {
    if let Some(path) = settings.binary.as_ref().and_then(|b| b.path.as_ref()) {
        validate_override_path(path)?;
        return Ok(path.clone());
    }
    match PINNED_DISTRIBUTION {
        Some(distribution) => install_pinned(&distribution),
        None => Err(no_distribution_error()),
    }
}

/// Validates that the user-supplied `binary.path` is an existing,
/// readable file, with actionable errors otherwise.
fn validate_override_path(path: &str) -> Result<(), String> {
    let metadata = std::fs::metadata(path)
        .map_err(|_| missing_override_error(path))?;
    if !metadata.is_file() {
        return Err(not_a_file_override_error(path));
    }
    std::fs::File::open(path)
        .map(|_| ())
        .map_err(|error| unreadable_override_error(path, &error))
}

/// Ensures the pinned distribution is present in the cache and returns
/// its script path. Downloads only when the version-keyed cache entry
/// is missing, so repeat launches never redownload and upgrades touch
/// only the new version.
fn install_pinned(distribution: &ServerDistribution) -> Result<String, String> {
    let (os, arch) = zed::current_platform();
    let asset = asset_name(os, arch)
        .ok_or_else(|| unsupported_platform_error(os, arch))?;

    let root = std::env::current_dir()
        .map_err(|error| format!("kson-language-server: cannot determine the extension install directory: {error}"))?;
    let paths = cache_paths(&root, distribution.version);

    if marker_matches(std::fs::read_to_string(&paths.marker), distribution.version)
        && paths.script.exists()
    {
        return Ok(paths.script.to_string_lossy().into_owned());
    }

    let url = download_url(distribution.base_url, distribution.version, asset);
    std::fs::create_dir_all(&paths.dir)
        .map_err(|error| download_failed_error(&url, &error.to_string()))?;
    zed::download_file(&url, &paths.dir.to_string_lossy(), distribution.file_type)
        .map_err(|error| download_failed_error(&url, &error))?;
    std::fs::write(&paths.marker, distribution.version)
        .map_err(|error| download_failed_error(&url, &error.to_string()))?;

    // The artifact is a Node script, not a native binary, so it only
    // needs to be readable; the executable bit costs nothing to set and
    // keeps direct invocation working.
    let _ = zed::make_file_executable(&paths.script.to_string_lossy());

    // Drop stale versions so upgrades do not accumulate old artifacts.
    remove_stale_versions(&root, distribution.version);

    Ok(paths.script.to_string_lossy().into_owned())
}

/// Removes cached versions other than `keep` from the cache root.
fn remove_stale_versions(root: &Path, keep: &str) {
    let cache_root = root.join("server");
    let Ok(entries) = std::fs::read_dir(&cache_root) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_name().to_string_lossy() != keep {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
}

// Actionable error message builders. Kept as pure functions so their
// wording (which must name the setting or prerequisite to fix) is
// tested.

fn no_distribution_error() -> String {
    "kson-language-server: no server script configured. No runnable \
     KSON server distribution is published (as of the upstream pin \
     928961c, 2026-11-15; see docs/compatibility-inventory.md). Build \
     the server locally from kson-org/kson and set \
     lsp.kson-language-server.binary.path to the bundled server.js, as \
     described in README.md under \"Local server setup\"."
        .to_string()
}

fn missing_override_error(path: &str) -> String {
    format!(
        "kson-language-server: lsp.kson-language-server.binary.path is \
         set to '{path}', which does not exist. Point it at a locally \
         built KSON server.js (see README.md under \"Local server \
         setup\") or unset it."
    )
}

fn not_a_file_override_error(path: &str) -> String {
    format!(
        "kson-language-server: lsp.kson-language-server.binary.path is \
         set to '{path}', which is not a file. Point it at a server.js \
         file, not a directory."
    )
}

fn unreadable_override_error(path: &str, error: &std::io::Error) -> String {
    format!(
        "kson-language-server: lsp.kson-language-server.binary.path is \
         set to '{path}', which is not readable ({error}). Fix the \
         file's permissions (e.g. `chmod +r '{path}'`) or point the \
         setting at a readable server.js."
    )
}

fn unsupported_platform_error(os: zed::Os, arch: zed::Architecture) -> String {
    format!(
        "kson-language-server: no server distribution artifact for \
         {os:?}/{arch:?}. Build the server locally from kson-org/kson \
         and set lsp.kson-language-server.binary.path to the bundled \
         server.js instead."
    )
}

fn download_failed_error(url: &str, error: &str) -> String {
    format!(
        "kson-language-server: failed to download the server from \
         '{url}' ({error}). If you are offline or behind a restricted \
         network, build the server locally from kson-org/kson and set \
         lsp.kson-language-server.binary.path to the bundled server.js."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_platform_gets_the_platform_independent_asset() {
        for (os, arch) in [
            (zed::Os::Mac, zed::Architecture::Aarch64),
            (zed::Os::Linux, zed::Architecture::X8664),
            (zed::Os::Windows, zed::Architecture::X8664),
        ] {
            assert_eq!(asset_name(os, arch), Some(ASSET));
        }
    }

    #[test]
    fn pinned_distribution_points_at_this_repository_release() {
        let pinned = PINNED_DISTRIBUTION.expect("a server distribution is pinned");
        assert_eq!(
            download_url(pinned.base_url, pinned.version, ASSET),
            "https://github.com/elviskahoro/zed-kson/releases/download/server-v0.5.0-dev.0-928961c/kson-language-server.tar.gz"
        );
    }

    #[test]
    fn cache_layout_is_keyed_on_version() {
        let root = Path::new("/extensions/kson");
        let paths = cache_paths(root, "0.5.0");
        assert_eq!(paths.dir, Path::new("/extensions/kson/server/0.5.0"));
        assert_eq!(
            paths.marker,
            Path::new("/extensions/kson/server/0.5.0/INSTALLED_VERSION")
        );
        assert_eq!(
            paths.script,
            Path::new("/extensions/kson/server/0.5.0/server.js")
        );
    }

    #[test]
    fn marker_match_ignores_surrounding_whitespace() {
        assert!(marker_matches(Ok("0.5.0\n".to_string()), "0.5.0"));
        assert!(marker_matches(Ok("0.5.0".to_string()), "0.5.0"));
        assert!(!marker_matches(Ok("0.4.0".to_string()), "0.5.0"));
        assert!(!marker_matches(
            Err(std::io::Error::other("no such file")),
            "0.5.0"
        ));
    }

    #[test]
    fn download_url_interpolates_tag_and_asset() {
        assert_eq!(
            download_url("https://example.com/kson/download", "0.5.0", "server.tgz"),
            "https://example.com/kson/download/server-v0.5.0/server.tgz"
        );
    }

    #[test]
    fn no_distribution_error_points_at_the_override_setting() {
        let error = no_distribution_error();
        assert!(error.contains("lsp.kson-language-server.binary.path"));
        assert!(error.contains("README.md"));
    }

    #[test]
    fn override_errors_name_the_setting_and_the_path() {
        assert!(missing_override_error("/no/such/server.js")
            .contains("'/no/such/server.js'"));
        assert!(not_a_file_override_error("/some/dir")
            .contains("not a file"));
        let unreadable = unreadable_override_error(
            "/secret/server.js",
            &std::io::Error::other("permission denied"),
        );
        assert!(unreadable.contains("lsp.kson-language-server.binary.path"));
        assert!(unreadable.contains("chmod +r '/secret/server.js'"));
    }

    #[test]
    fn platform_and_download_errors_point_at_the_override_setting() {
        let unsupported =
            unsupported_platform_error(zed::Os::Windows, zed::Architecture::X8664);
        assert!(unsupported.contains("Windows"));
        assert!(unsupported.contains("lsp.kson-language-server.binary.path"));
        let offline = download_failed_error("https://example.com/s.tgz", "dns error");
        assert!(offline.contains("offline"));
        assert!(offline.contains("lsp.kson-language-server.binary.path"));
    }

    #[test]
    fn override_validation_accepts_a_readable_file() {
        let dir = std::env::temp_dir().join("zk3ew-test-override-ok");
        std::fs::create_dir_all(&dir).unwrap();
        let script = dir.join("server.js");
        std::fs::write(&script, "// stub").unwrap();
        assert!(validate_override_path(script.to_str().unwrap()).is_ok());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn override_validation_rejects_missing_and_directory_paths() {
        let missing = validate_override_path("/no/such/path/server.js")
            .unwrap_err();
        assert!(missing.contains("does not exist"));

        let dir = std::env::temp_dir().join("zk3ew-test-override-dir");
        std::fs::create_dir_all(&dir).unwrap();
        let is_dir = validate_override_path(dir.to_str().unwrap()).unwrap_err();
        assert!(is_dir.contains("not a file"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn stale_versions_are_removed_but_kept_version_survives() {
        let root = std::env::temp_dir().join("zk3ew-test-stale");
        let _ = std::fs::remove_dir_all(&root);
        for version in ["0.4.0", "0.5.0"] {
            let paths = cache_paths(&root, version);
            std::fs::create_dir_all(&paths.dir).unwrap();
            std::fs::write(&paths.marker, version).unwrap();
        }
        remove_stale_versions(&root, "0.5.0");
        assert!(!root.join("server").join("0.4.0").exists());
        assert!(root.join("server").join("0.5.0").exists());
        std::fs::remove_dir_all(&root).unwrap();
    }
}
