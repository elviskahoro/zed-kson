# Issue dispositions

Where each GitHub issue from the original backlog (#1–#41) landed, with the
evidence. Issues marked **open** need a person running Zed or a registry
submission; everything else is resolved in this repository.

## Compatibility discovery (#1)

| Issue | Disposition | Evidence |
| --- | --- | --- |
| #6 Pin upstream revisions, inventory VS Code behavior | Done | `docs/compatibility-inventory.md` (kson-org/kson `928961c`, `server/KSON_COMMIT`) |
| #7 Grammar ownership and release ownership | Done | Grammar lives in this repo (`tree-sitter-kson/`); releases owned by `elviskahoro/zed-kson`. Decision D1 in the inventory, `docs/grammar-registration.md`, `docs/zed-publication-guide.md` |
| #8 Server distribution, verify stdio | Done | `--stdio` verified end to end (`docs/launch-matrix.md`). This repo builds and hosts the server (`scripts/build-server.sh`, `.github/workflows/server-release.yml`); decision recorded in the inventory |
| #9 First-release support matrix | Done | README "Runtime requirements" and `docs/launch-matrix.md`. The server is plain JS, so any platform with Node.js 24+ is supported; no native artifacts |

## Syntax foundation (#2)

| Issue | Disposition | Evidence |
| --- | --- | --- |
| #10–#14 Grammar skeleton, scalars, structures, embeds, corpus | Done | `tree-sitter-kson/`, 58 corpus tests, `scripts/check-grammar.sh` |
| #15 Register pinned grammar | Done | `[grammars.kson]` in `extension.toml`; Zed's `extension_cli` builds it (RWX target `package`) |
| #16 highlights.scm, #17 indents/brackets, #18 injections | Done | `languages/kson/*.scm`, compiled and matched by `check-grammar.sh` |
| #19 Verify `.kson` recognition and manual selection in Zed | Done | Interactive pass on macOS arm64, `docs/dev-install-smoke-test.md` |

## LSP integration (#3)

| Issue | Disposition | Evidence |
| --- | --- | --- |
| #20 Stdio-runnable server artifact | Done | `server-v*` releases of this repo, built from the pinned upstream commit with its license |
| #21 Download, cache, version handling | Done | `PINNED_DISTRIBUTION` in `src/distribution.rs`, version-keyed cache, stale-version pruning, unit tests |
| #22 Local override and actionable failures | Done | `binary.path` / `binary.arguments`, Node resolution and errors in `src/node.rs`; the `process:exec` capability needed for the Node check is declared in `extension.toml` |
| #23 Settings propagation | Done | `src/config.rs`; settings go under the effective `distributionId` section, matching upstream `getConfiguration(distributionId)` |
| #24 Environment edge cases | Done | `docs/launch-matrix.md` rows 4–9 |
| #25 LSP integration tests | Done | `scripts/check-parity.sh` runs F1–F28 against the real server in RWX (target `parity`) and in the server release workflow: 31 passed, 0 failed |

## Feature parity (#4)

| Issue | Disposition | Evidence |
| --- | --- | --- |
| #26 Diagnostics | Done | F1–F7 |
| #27 Formatting | Done | F8–F16 |
| #28 Schema validation, hover, completion | Done | F17–F24 |
| #29 Schema association and reload | Done (adapted/deferred) | Association via `.kson-schema.kson`; hot reload deferred, server registers no file watchers (P12) |
| #30 Bundled schemas, `bundled://` navigation | Done (adapted) | F25–F28; `bundled://` URIs cannot be opened in Zed (P8) |
| #31 Schema picker, status bar, notifications, CodeLens | Done (classified) | Inventory P6 (no QuickPick API), P7 (no status bar API), P11 (CodeLens uses the server default indent) |
| #32 Snippets | Done | `snippets/kson.json`, S1–S3 |
| #33 Observability | Done | README "Troubleshooting" |

## Release readiness (#5)

| Issue | Disposition | Evidence |
| --- | --- | --- |
| #34 CI | Done | `.github/workflows/ci.yml`, `.rwx/zed-kson.yml` (`check`, `package`, `parity` all pass) |
| #35 Platform smoke tests | **Open** | macOS arm64 passed (`docs/dev-install-smoke-test.md`); formatting, snippets and Linux still to check |
| #36 README | Done | `README.md` |
| #37 License and publication prerequisites | Done | `docs/zed-publication-guide.md` checklist; Apache-2.0 here and upstream |
| #38 Submit to zed-industries/extensions | **Open** | After #35 passes |

#40 (the original starter prompt) is superseded by this work.

Parent and tracking issues are not used: the epics (#1–#5) and the
hand-off (#41) are closed, and the remaining work is tracked only by the
leaf issues #35 and #38. The section headings above keep the
original epic numbers for reference.
