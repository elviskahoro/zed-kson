# Zed publication guide

How to submit this extension to
[zed-industries/extensions](https://github.com/zed-industries/extensions)
and how to ship later versions. Based on Zed's published docs
(<https://zed.dev/docs/extensions/publishing/overview>), which are
authoritative; re-read them before each submission. Submission is
reviewed and acceptance is not guaranteed.

**Status:** not submitted. Opening the PR is a manual, maintainer-run
step (it publishes to a third-party repository).

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
- [x] **`repository` field resolved.** The canonical repository is
  `https://github.com/elviskahoro/zed-kson` (the old value,
  `kson-org/zed-kson`, does not exist). The top-level `repository`,
  `[grammars.kson].repository` and the git remote now agree. If the
  extension later moves under kson-org, update all three together.
- [x] License review: this repository is Apache-2.0 (`LICENSE`). The
  server release asset redistributes kson-org/kson's `server.js`
  (Apache-2.0) with the upstream `LICENSE` and a `SOURCE` file naming
  the commit inside the archive, which satisfies Apache-2.0
  redistribution terms. The bundled draft-07 metaschema notice is in
  the README "License" section.
- [ ] `[grammars.kson]` `repository`/`rev`/`path` reference a **public,
  reachable** git repository whose `rev` contains the grammar sources
  under `path` (see
  [`docs/grammar-registration.md`](./grammar-registration.md)). Zed
  clones and builds `src/parser.c` from that pin at install time, so
  the pinned commit must already be pushed. Note: the currently pinned
  `rev` (`953c315`) must exist on the canonical remote — re-pin to the
  publication commit if the grammar moved.
- [x] `version` is `0.1.0` in `extension.toml` and `Cargo.toml`.

Content state:

- [ ] `README.md` accurate: setup, runtime requirements, configuration
  reference, feature coverage, parity gaps, troubleshooting.
- [x] LICENSE present (Apache-2.0) — required for the extensions
  listing.

## Registry rules that apply

- One extension per PR; at most three open PRs at once; respond to
  maintainer feedback within 3 weeks or the PR is closed.
- Extension ID `kson`: kebab-case, unique, no `zed`/`extension`.
- License must be on Zed's accepted list and sit at the extension root.
  Apache-2.0 (`LICENSE`) is accepted.
- Language server extensions must not bundle the server. This extension
  downloads it from this repository's GitHub releases at runtime
  (`src/distribution.rs`), which is the sanctioned pattern.
- Test manually in Zed at the exact commit being submitted.

## Submission steps

1. Make sure `main` is pushed, CI is green, and the submodule commit is
   on a branch (not detached) of the public HTTPS repository.
2. Fork `zed-industries/extensions` to the personal account (not an
   org), so Zed staff can push fixes to the PR. Clone it and run
   `git submodule init && git submodule update`.
3. Add this repository as a submodule over HTTPS:

   ```sh
   git submodule add https://github.com/elviskahoro/zed-kson.git extensions/kson
   git add extensions/kson
   ```

4. Add an entry to the top-level `extensions.toml`. `version` must equal
   `version` in `extension.toml` at the submodule commit:

   ```toml
   [kson]
   submodule = "extensions/kson"
   version = "0.1.0"
   ```

5. Run `pnpm sort-extensions` (sorts `extensions.toml` and
   `.gitmodules`).
6. Open a PR titled `Add kson extension` against
   `zed-industries/extensions`. After merge, Zed packages and publishes
   it. Then update README "Installation" to point at the Zed listing.

## Update procedure (future versions)

Two kinds of release exist and are independent:

- **Server bundle** (`server-v<version>` tags, built by
  `.github/workflows/server-release.yml`): see the README section on the
  server distribution. After publishing a new bundle, update
  `PINNED_DISTRIBUTION` in `src/distribution.rs`, which requires an
  extension release to reach users.
- **Extension** (`v<version>` tags): steps below.

1. Land changes with CI green; re-run the parity suite and the
   dev-install smoke test.
2. Bump `version` in `extension.toml` and `Cargo.toml`. If the grammar
   changed, bump `[grammars.kson].rev` to a pushed commit.
3. Merge to `main`, tag `v<version>`, push the tag.
4. In the fork of `zed-industries/extensions`, sync with upstream, then:

   ```sh
   git submodule update --remote extensions/kson
   ```

   and set the new `version` for `[kson]` in `extensions.toml`. Confirm
   it matches `extension.toml` at the new submodule commit.
5. Open a PR (one extension per PR). The merge publishes the update.

`huacnlee/zed-extension-action` can automate steps 4 and 5 if wanted.

## Known publication-relevant caveats

- The server is downloaded from this repository's releases, so the
  listing may describe server-backed features, but they depend on the
  `server-v*` release asset staying available. Do not delete it.
- The extension targets `zed_extension_api` 0.7.0; if the extensions
  repo requires a newer API at submission time, bump the dependency
  and re-run the whole gate set first.
