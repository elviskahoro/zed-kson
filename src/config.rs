//! Initialization options and workspace configuration for the KSON
//! language server, with user overrides taking precedence over defaults.
//!
//! The server accepts the initialization options documented in
//! `docs/compatibility-inventory.md` (upstream pin `928961c`):
//! `distributionId` (default `"kson"`, used as command-id prefix and
//! diagnostic source), `enableBundledSchemas`, `bundledSchemas`, and
//! `bundledMetaSchemas`.
//!
//! Bundled schemas are passed **inline**: the server only wires its
//! `BundledSchemaProvider` when the `bundledSchemas`/`bundledMetaSchemas`
//! arrays are non-empty (`startKsonServer.ts` at the pin), so the
//! extension ships the draft-07 metaschema (`schemas/` directory) and
//! passes its contents in the defaults. `enableBundledSchemas: false`
//! (a user override) disables the bundled provider at runtime.
//!
//! User-provided options are deep-merged over the defaults, so setting
//! one key (e.g. only `distributionId`) does not silently drop the
//! defaults of the others. Arrays replace the default entirely (they
//! are not merged element-wise).

use zed_extension_api::serde_json::{json, Value};

/// Schema id of the bundled draft-07 metaschema (matched against a
/// document's `$schema` content by the server).
const METASCHEMA_ID: &str = "http://json-schema.org/draft-07/schema#";

/// Display name for the bundled metaschema (surfaces in
/// `bundled://metaschema/{name}.schema.kson` URIs).
const METASCHEMA_NAME: &str = "draft-07";

/// The draft-07 metaschema contents, copied verbatim from the upstream
/// VS Code extension (`tooling/lsp-clients/vscode/schemas/` at the
/// pin). `.schema.kson` documents that declare this `$schema` are
/// validated against it without any workspace configuration.
const METASCHEMA_CONTENT: &str = include_str!("../schemas/metaschema.draft7.kson");

/// Default initialization options for the KSON server.
pub fn default_initialization_options() -> Value {
    json!({
        "distributionId": "kson",
        "enableBundledSchemas": true,
        "bundledSchemas": [],
        "bundledMetaSchemas": [{
            "schemaId": METASCHEMA_ID,
            "name": METASCHEMA_NAME,
            "schemaContent": METASCHEMA_CONTENT,
        }],
    })
}

/// Merges `user` over `defaults`.
///
/// Objects are merged recursively: keys present only in `user` are
/// added, keys present in both take the user's (recursively merged when
/// both sides are objects), and keys present only in `defaults` are
/// kept. Any non-object user value replaces the default entirely.
pub fn merge_defaults(defaults: Value, user: Value) -> Value {
    match (defaults, user) {
        (Value::Object(mut merged), Value::Object(user)) => {
            for (key, user_value) in user {
                let merged_value = match merged.remove(&key) {
                    Some(default_value) => merge_defaults(default_value, user_value),
                    None => user_value,
                };
                merged.insert(key, merged_value);
            }
            Value::Object(merged)
        }
        // A missing default (or a non-object user override) defers to
        // the user value.
        (_, user) => user,
    }
}

/// The initialization options to send: defaults with user overrides
/// deep-merged on top.
pub fn initialization_options(user: Option<Value>) -> Value {
    merge_defaults(
        default_initialization_options(),
        user.unwrap_or_else(|| Value::Object(Default::default())),
    )
}

/// The workspace configuration section for the server. The KSON server
/// reads its `didChangeConfiguration` payload from the `"kson"` section
/// (see `docs/compatibility-inventory.md`, "Workspace configuration").
pub fn workspace_configuration(settings: Option<Value>) -> Value {
    json!({
        "kson": settings.unwrap_or_else(|| Value::Object(Default::default())),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_user_options_yields_the_defaults() {
        let merged = initialization_options(None);
        assert_eq!(merged["distributionId"], json!("kson"));
        assert_eq!(merged["enableBundledSchemas"], json!(true));
        assert_eq!(merged["bundledSchemas"], json!([]));
        assert_eq!(merged["bundledMetaSchemas"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn defaults_pass_the_bundled_metaschema_inline() {
        // The server only enables its bundled provider when the arrays
        // are non-empty, so the metaschema contents must ride along in
        // the defaults (WI-4; see the module docs).
        let merged = initialization_options(None);
        let metaschema = &merged["bundledMetaSchemas"][0];
        assert_eq!(metaschema["schemaId"], json!(METASCHEMA_ID));
        assert_eq!(metaschema["name"], json!(METASCHEMA_NAME));
        assert_eq!(metaschema["schemaContent"], json!(METASCHEMA_CONTENT));
        assert!(
            metaschema["schemaContent"].as_str().unwrap().contains("draft-07"),
            "the shipped metaschema contents must actually be embedded"
        );
    }

    #[test]
    fn user_arrays_replace_the_default_bundled_schemas() {
        let merged = initialization_options(Some(json!({
            "bundledMetaSchemas": []
        })));
        assert_eq!(merged["bundledMetaSchemas"], json!([]),
            "an explicit user array replaces the default rather than merging");
        assert_eq!(merged["distributionId"], json!("kson"));
    }

    #[test]
    fn user_keys_override_defaults_key_by_key() {
        let merged = initialization_options(Some(json!({
            "distributionId": "custom"
        })));
        assert_eq!(merged["distributionId"], json!("custom"));
        assert_eq!(
            merged["enableBundledSchemas"],
            json!(true),
            "unset keys must keep their defaults"
        );
    }

    #[test]
    fn nested_objects_merge_recursively() {
        let merged = merge_defaults(
            json!({"a": {"x": 1, "y": 2}, "b": 0}),
            json!({"a": {"y": 3, "z": 4}}),
        );
        assert_eq!(merged, json!({"a": {"x": 1, "y": 3, "z": 4}, "b": 0}));
    }

    #[test]
    fn non_object_user_value_replaces_the_default_entirely() {
        let merged = merge_defaults(
            json!({"distributionId": "kson"}),
            json!([1, 2]),
        );
        assert_eq!(merged, json!([1, 2]));
    }

    #[test]
    fn explicit_user_null_wins_over_the_default() {
        let merged = merge_defaults(
            json!({"enableBundledSchemas": true}),
            json!({"enableBundledSchemas": null}),
        );
        assert_eq!(merged["enableBundledSchemas"], Value::Null);
    }

    #[test]
    fn workspace_configuration_wraps_settings_in_the_kson_section() {
        assert_eq!(workspace_configuration(None), json!({"kson": {}}));
        assert_eq!(
            workspace_configuration(Some(json!({"indent": 2}))),
            json!({"kson": {"indent": 2}})
        );
    }
}
