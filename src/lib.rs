//! KSON language support for Zed.
//!
//! The extension registers the `kson-language-server` adapter: it
//! launches the KSON language server (a Node script) over stdio and
//! propagates initialization options and workspace configuration with
//! user overrides.
//!
//! Server script resolution prefers the user's
//! `lsp.kson-language-server.binary.path` override (WI-1 decision D2);
//! a pinned-distribution download/cache path exists for when upstream
//! publishes one. Node resolution prefers Zed's bundled runtime over
//! PATH and version-checks every candidate. See `distribution` and
//! `node` module docs and `docs/launch-matrix.md`.

mod config;
mod distribution;
mod node;

use zed_extension_api::{
    self as zed, serde_json, settings::LspSettings, LanguageServerId, Result,
};

const SERVER_ID: &str = "kson-language-server";

/// Transport selection flag for the server's Node entry.
///
/// The server bundles `vscode-languageserver/node` v10, whose
/// `createConnection()` selects stdio transport from argv when given
/// `--stdio` (upstream pin `928961c`, `docs/compatibility-inventory.md`
/// "Transport and CLI flags"; runtime verification:
/// `docs/launch-matrix.md`).
const STDIO_FLAG: &str = "--stdio";

struct KsonExtension;

impl zed::Extension for KsonExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let settings = LspSettings::for_worktree(SERVER_ID, worktree)?;
        let script = distribution::resolve_server_script(&settings)?;
        let node = node::resolve_node(worktree)?;

        let mut args = vec![script, STDIO_FLAG.to_string()];
        args.extend(
            settings
                .binary
                .as_ref()
                .and_then(|binary| binary.arguments.clone())
                .unwrap_or_default(),
        );
        let env = settings
            .binary
            .as_ref()
            .and_then(|binary| binary.env.clone())
            .unwrap_or_default()
            .into_iter()
            .collect();

        Ok(zed::Command {
            command: node.path,
            args,
            env,
        })
    }

    fn language_server_initialization_options(
        &mut self,
        _id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let user = LspSettings::for_worktree(SERVER_ID, worktree)?
            .initialization_options;
        Ok(Some(config::initialization_options(user)))
    }

    fn language_server_workspace_configuration(
        &mut self,
        _id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let settings = LspSettings::for_worktree(SERVER_ID, worktree)?.settings;
        Ok(Some(config::workspace_configuration(settings)))
    }
}

zed::register_extension!(KsonExtension);
