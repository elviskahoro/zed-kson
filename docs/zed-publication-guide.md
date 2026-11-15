# Zed publication guide (WI-5)

How to prepare this extension for submission to
[zed-industries/extensions](https://github.com/zed-industries/extensions),
and how to update the published entry for future versions.

**Scope:** preparing the submission (this guide and its checklist) is
in scope for this build. Actually opening or submitting the PR is
explicitly out of scope (requirements "Out Of Scope"; plan Non-Goals),
as is pushing releases or tags. Nothing in this document has been
submitted anywhere.

## Prerequisites (must all be true before submitting)

Repository state:

- [ ] CI green on the commit being published
  (`.github/workflows/ci.yml`: compile + grammar/query validity +
  fixture checks).
- [ ] `cargo check --target wasm32-wasip2` and `cargo test` pass
  locally on a clean checkout.
- [ ] Full parity suite re-run against a freshly built server bundle:
  `KSON_SERVER=<bundle> ./scripts/check-parity.sh` (all 31 checks
  pass; results recorded in `fixtures/parity/README.md`).
- [ ] Dev-install smoke test
  ([`docs/dev-install-smoke-test.md`](./dev-install-smoke-test.md))
  executed end-to-end in a clean Zed install, including the
  interactive steps that are deferred in this build's record.
- [ ] Every parity item in `docs/compatibility-inventory.md` (P1–P14)
  has a passing test or an explicit documented disposition — no silent
  gaps, no unverified claims.

Manifest (`extension.toml`) state:

- [ ] `id`, `name`, `version`, `schema_version`, `description`,
  `authors`, `repository` all present and correct.
- [ ] **Resolve the `repository` field.** It currently reads
  `https://github.com/kson-org/zed-kson`, but the git remote this
  build was developed against is
  `https://github.com/elviskahoro/zed-kson`, and the
  `[grammars.kson]` pin points there too. Decide before submission
  which public repository is canonical (an org repo under kson-org, or
  the personal one) and make the top-level `repository`,
  `[grammars.kson].repository` and this repo's remote all agree. This
  is the one deliberately unresolved decision in this build — recorded
  here rather than guessed.
- [ ] `[grammars.kson]` `repository`/`rev`/`path` reference a **public,
  reachable** git repository whose `rev` contains the grammar sources
  under `path` (see
  [`docs/grammar-registration.md`](./grammar-registration.md)). Zed
  clones and builds `src/parser.c` from that pin at install time, so
  the pinned commit must already be pushed. Note: the currently pinned
  `rev` (`953c315`) must exist on the canonical remote — re-pin to the
  publication commit if the grammar moved.
- [ ] `version` bumped from `0.0.1` for the first published release
  (and `Cargo.toml` kept in sync).

Content state:

- [ ] `README.md` accurate: setup, runtime requirements, configuration
  reference, feature coverage, parity gaps, troubleshooting.
- [ ] LICENSE present (Apache-2.0) — required for the extensions
  listing.

## Submission steps

Follow the process published in the
zed-industries/extensions repository README at submission time — it is
authoritative and may have changed since this guide was written. The
shape of the process as of this build:

1. Fork `zed-industries/extensions`.
2. Add a directory for this extension (`extensions/kson/`) containing
   the manifest stub that repository expects — the fields it requires
   (name and the source `repository` it should install from, plus the
   `rev` pin if it asks for one). Confirm the exact required fields
   against the repository's own documentation; do not invent them.
3. Open a PR to `zed-industries/extensions` adding the entry.
4. After merge, the extension becomes installable from Zed's bundled
   extension listing; update this repo's README "Installation" section
   to point at the listing instead of dev-install only.

## Update procedure (future versions)

1. Land the changes, keep CI green, re-run the parity suite and the
   dev-install smoke test.
2. Bump `version` in `extension.toml` (and `Cargo.toml`).
3. If the grammar changed, bump `[grammars.kson]` `rev` to the commit
   that carries the new grammar (a repo-maintainer push of that commit
   is a prerequisite — Zed installs the grammar from the pin, not from
   HEAD).
4. Push the release commit (and tag it, e.g. `v0.1.0`, if the project
   adopts tags — this build deliberately created none).
5. Update the `extensions/kson` entry in the zed-industries/extensions
   fork to the new `rev` and open a PR there; that update is what
   propagates the new version and grammar pin to users.

## Known publication-relevant caveats

- The server-distribution blocker (WI-1 decision D2) does not block
  publication: the extension is fully usable for syntax support
  without a server, and server-backed features work via the documented
  local build + `binary.path`. The README says so plainly; the
  extensions-listing description must not promise downloaded-server
  functionality.
- The extension targets `zed_extension_api` 0.7.0; if the extensions
  repo requires a newer API at submission time, bump the dependency
  and re-run the whole gate set first.
