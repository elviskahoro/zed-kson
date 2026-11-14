use zed_extension_api::{self as zed, serde_json, settings::LspSettings, LanguageServerId, Result};

const SERVER_ID: &str = "kson-language-server";

struct KsonExtension;

impl zed::Extension for KsonExtension {
    fn new() -> Self {
        Self
    }

    // Until upstream publishes a runnable server, the user points at a locally
    // built server.js via `lsp.kson-language-server.binary.path`.
    fn language_server_command(
        &mut self,
        _id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let settings = LspSettings::for_worktree(SERVER_ID, worktree)?;
        let binary = settings
            .binary
            .ok_or("set lsp.kson-language-server.binary.path to a built KSON server.js")?;
        let script = binary
            .path
            .ok_or("set lsp.kson-language-server.binary.path to a built KSON server.js")?;
        let node = worktree.which("node").ok_or("node not found on PATH")?;
        Ok(zed::Command {
            command: node,
            args: [
                vec![script, "--stdio".into()],
                binary.arguments.unwrap_or_default(),
            ]
            .concat(),
            env: binary.env.unwrap_or_default().into_iter().collect(),
        })
    }

    fn language_server_initialization_options(
        &mut self,
        _id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let user = LspSettings::for_worktree(SERVER_ID, worktree)?.initialization_options;
        Ok(user.or_else(|| {
            Some(serde_json::json!({
                "distributionId": "kson",
                "enableBundledSchemas": true
            }))
        }))
    }

    fn language_server_workspace_configuration(
        &mut self,
        _id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let settings = LspSettings::for_worktree(SERVER_ID, worktree)?.settings;
        Ok(Some(
            serde_json::json!({ "kson": settings.unwrap_or_else(|| serde_json::json!({})) }),
        ))
    }
}

zed::register_extension!(KsonExtension);
