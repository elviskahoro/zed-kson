#!/usr/bin/env python3
"""LSP parity harness for the zed-kson extension (WI-4).

Speaks JSON-RPC over stdio to the KSON language server (built from the
upstream pin, see docs/launch-matrix.md "Local server setup") and
exercises the feature-parity scenarios documented in
fixtures/parity/README.md. Each fixture prints a `PASS`/`FAIL` line with
the observed result; the process exits non-zero if any fixture fails.

Static fixtures (snippet conversion checks) run without a server.

Usage:
    python3 scripts/parity_harness.py [--server PATH] [--node PATH]

`--server` defaults to $KSON_SERVER, `--node` to "node". When no server
bundle is available, the server-dependent fixtures print `DEFERRED`
(this is an explicit disposition, not a pass) and only the static
fixtures decide the exit code.
"""

from __future__ import annotations

import argparse
import json
import os
import queue
import shutil
import subprocess
import sys
import tempfile
import threading
import time

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FIXTURES = os.path.join(REPO_ROOT, "fixtures", "parity")
METASCHEMA = os.path.join(REPO_ROOT, "schemas", "metaschema.draft7.kson")
SNIPPETS = os.path.join(REPO_ROOT, "snippets", "kson.json")

# Initialization options the adapter sends after WI-4 (src/config.rs):
# the bundled metaschema rides along inline because the server only
# wires its bundled provider when the arrays are non-empty.
def adapter_initialization_options() -> dict:
    with open(METASCHEMA, encoding="utf-8") as f:
        content = f.read()
    return {
        "distributionId": "kson",
        "enableBundledSchemas": True,
        "bundledSchemas": [],
        "bundledMetaSchemas": [
            {
                "schemaId": "http://json-schema.org/draft-07/schema#",
                "name": "draft-07",
                "schemaContent": content,
            }
        ],
    }


class FixtureResult:
    def __init__(self) -> None:
        self.results: list[tuple[str, bool, str]] = []

    def record(self, fixture: str, ok: bool, detail: str) -> None:
        self.results.append((fixture, ok, detail))
        print(f"{'PASS' if ok else 'FAIL'} {fixture}: {detail}")

    def deferred(self, fixture: str, reason: str) -> None:
        self.results.append((fixture, None, reason))
        print(f"DEFERRED {fixture}: {reason}")

    def exit_code(self) -> int:
        return 1 if any(ok is False for _, ok, _ in self.results) else 0


class LspClient:
    """Minimal LSP client: launches the server, frames JSON-RPC over
    stdio, answers server->client requests, and exposes request/response
    helpers. Mirrors the WI-3 smoke harness (docs/launch-matrix.md)."""

    def __init__(self, root: str, server: str, node: str, settings: dict | None = None):
        self.root = root
        self.proc = subprocess.Popen(
            [node, server, "--stdio"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.q: queue.Queue = queue.Queue()
        self.requests: dict = {}
        self.notifications: list = []
        self.settings = settings or {}
        self._id = 0
        threading.Thread(target=self._reader, daemon=True).start()

    def _reader(self) -> None:
        headers = {}
        while True:
            line = self.proc.stdout.readline()
            if not line:
                self.q.put(None)
                return
            line = line.decode().strip()
            if not line:
                if "Content-Length" in headers:
                    body = self.proc.stdout.read(int(headers["Content-Length"]))
                    headers = {}
                    try:
                        self.q.put(json.loads(body.decode()))
                    except Exception as e:  # pragma: no cover
                        self.q.put({"_parse_error": str(e)})
            else:
                k, _, v = line.partition(":")
                headers[k.strip()] = v.strip()

    def _handle_server_message(self, msg) -> str | None:
        """Returns 'HANDLED' for server->client traffic, else the message."""
        if msg is None:
            return None
        if "id" in msg and "method" in msg:
            self.requests[msg["id"]] = msg
            self._answer(msg)
            return "HANDLED"
        if "method" in msg:
            self.notifications.append(msg)
            return "HANDLED"
        return msg

    def _answer(self, msg) -> None:
        method = msg.get("method")
        if method == "workspace/configuration":
            items = msg.get("params", {}).get("items", [])
            result = []
            for item in items:
                if item.get("section") == "kson":
                    result.append(self.settings)
                else:
                    result.append(None)
            self._respond(msg["id"], result)
        else:
            # workspace/applyEdit, client/registerCapability, anything else.
            self._respond(msg["id"], None)

    def _respond(self, rid, result) -> None:
        body = json.dumps({"jsonrpc": "2.0", "id": rid, "result": result}).encode()
        self.proc.stdin.write(f"Content-Length: {len(body)}\r\n\r\n".encode() + body)
        self.proc.stdin.flush()

    def send(self, msg) -> None:
        body = json.dumps(msg).encode()
        self.proc.stdin.write(f"Content-Length: {len(body)}\r\n\r\n".encode() + body)
        self.proc.stdin.flush()

    def request(self, method: str, params, timeout: float = 25):
        self._id += 1
        rid = self._id
        self.send({"jsonrpc": "2.0", "id": rid, "method": method, "params": params})
        end = time.time() + timeout
        while time.time() < end:
            try:
                msg = self.q.get(timeout=max(0.1, end - time.time()))
            except queue.Empty:
                break
            if msg is None:
                return None
            handled = self._handle_server_message(msg)
            if handled == "HANDLED":
                continue
            if msg.get("id") == rid:
                return msg
        return {"_timeout": True}

    def initialize(self, init_options: dict | None = None):
        res = self.request(
            "initialize",
            {
                "processId": None,
                "rootUri": "file://" + self.root,
                # The server reads its workspace root from
                # workspaceFolders[0].uri, not rootUri.
                "workspaceFolders": [{"uri": "file://" + self.root, "name": "parity"}],
                "capabilities": {
                    "workspace": {"configuration": True, "workspaceFolders": True},
                    "textDocument": {
                        "formatting": True,
                        "hover": True,
                        "codeLens": True,
                        "definition": True,
                        "completion": {"completionItem": {"snippetSupport": True}},
                    },
                },
                "initializationOptions": init_options or adapter_initialization_options(),
            },
        )
        self.send({"jsonrpc": "2.0", "method": "initialized", "params": {}})
        return res

    def uri(self, relpath: str) -> str:
        return "file://" + os.path.join(self.root, relpath)

    def did_open(self, relpath: str, text: str, language_id: str = "kson"):
        self.send(
            {
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": self.uri(relpath),
                        "languageId": language_id,
                        "version": 1,
                        "text": text,
                    }
                },
            }
        )

    def set_workspace_settings(self, settings: dict):
        """Server settings update: push didChangeConfiguration (the
        adapter wraps user settings under the "kson" section,
        src/config.rs) and answer the resulting configuration pull."""
        self.settings = settings
        self.send(
            {
                "jsonrpc": "2.0",
                "method": "workspace/didChangeConfiguration",
                "params": {"settings": {"kson": settings}},
            }
        )

    def drain(self, seconds: float = 0.5):
        end = time.time() + seconds
        while time.time() < end:
            try:
                msg = self.q.get(timeout=max(0.05, end - time.time()))
            except queue.Empty:
                break
            if msg is None:
                break
            self._handle_server_message(msg)

    def diagnostics(self, relpath: str):
        return self.request(
            "textDocument/diagnostic",
            {"textDocument": {"uri": self.uri(relpath)}},
        )

    def format(self, relpath: str, insert_spaces: bool, tab_size: int):
        return self.request(
            "textDocument/formatting",
            {
                "textDocument": {"uri": self.uri(relpath)},
                "options": {"insertSpaces": insert_spaces, "tabSize": tab_size},
            },
        )

    def shutdown_exit(self):
        self.send({"jsonrpc": "2.0", "id": 9000, "method": "shutdown", "params": None})
        self.drain(1.0)
        self.send({"jsonrpc": "2.0", "method": "exit", "params": None})
        try:
            return self.proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            self.proc.kill()
            return None


def read_fixture(relpath: str) -> str:
    with open(os.path.join(FIXTURES, relpath), encoding="utf-8") as f:
        return f.read()


def new_workspace(with_schema: bool) -> tuple[str, str | None]:
    """Temp workspace root mirroring fixtures/parity/workspace. Returns
    (root, cleanup_path)."""
    root = tempfile.mkdtemp(prefix="kson-parity-")
    if with_schema:
        for name in (".kson-schema.kson", "config.schema.kson"):
            shutil.copy(os.path.join(FIXTURES, "workspace", name), os.path.join(root, name))
    return root, root if with_schema else None


def cleanup(root: str) -> None:
    shutil.rmtree(root, ignore_errors=True)


def diag_items(response) -> list:
    if not response or "result" not in response:
        return [{"_no_response": True}]
    return response["result"].get("items", [])


# ---------------------------------------------------------------------------
# Static fixtures (no server required)
# ---------------------------------------------------------------------------

def fixture_snippets(result: FixtureResult) -> None:
    with open(SNIPPETS, encoding="utf-8") as f:
        snippets = json.load(f)

    expected = {
        "kv": "Key-Value Pair",
        "embed%": "Embed Block (%)",
        "embed$": "Embed Block ($)",
        "list": "Array (list)",
    }
    ok = set(expected) == {s.get("prefix") for s in snippets.values()} and set(
        expected.values()
    ) == set(snippets)
    result.record("S1 snippets/prefixes", ok, f"{sorted(snippets)} (from VS Code kson.code-snippets)")

    # Placeholder semantics: ${1:key}, ${2:value}, $0 must survive; a `$`
    # outside a placeholder must be escaped as `\$` (LSP snippet grammar,
    # which Zed implements — crates/snippet/src/snippet.rs).
    bodies = {name: "\n".join(s["body"]) for name, s in snippets.items()}
    embed = bodies["Embed Block ($)"]
    ok = (
        "\\$${1:lang}" in embed  # literal `$` then a tabstop
        and "\\$\\$" in embed  # literal `$$` end marker
        and "$0" in embed
    )
    result.record("S2 snippets/escaping", ok, "embed$ body keeps \\$ escapes for literal $ and $$")

    ok = (
        "${1:key}: ${2:value}" in bodies["Key-Value Pair"]
        and "$0" in bodies["Array (list)"]
        and "%%" in bodies["Embed Block (%)"]
    )
    result.record("S3 snippets/placeholders", ok, "tabstops ${1:...}/${2:...} and $0 present")


# ---------------------------------------------------------------------------
# Server fixtures
# ---------------------------------------------------------------------------

def run_diagnostics_fixtures(result: FixtureResult, server: str, node: str) -> None:
    root, _ = new_workspace(with_schema=False)
    c = LspClient(root, server, node)
    c.initialize()
    c.drain(0.5)
    try:
        cases = [
            ("F1 diagnostics/valid-plain", "valid_plain.kson", 0),
            ("F2 diagnostics/valid-json-compat", "valid_json_compat.kson", 0),
            ("F3 diagnostics/invalid-trailing", "invalid_trailing.kson", 1),
            ("F4 diagnostics/invalid-indent", "invalid_indent.kson", 2),
        ]
        for fixture, doc, expected_count in cases:
            c.did_open(doc, read_fixture(os.path.join("documents", doc)))
            c.drain(0.8)
            items = diag_items(c.diagnostics(doc))
            ok = len(items) == expected_count
            summary = "; ".join(f"[{i.get('severity')}] {i.get('message')}" for i in items) or "none"
            result.record(fixture, ok, f"expected {expected_count} diagnostic(s), got {len(items)}: {summary}")

        items = diag_items(c.diagnostics("invalid_trailing.kson"))
        ok = items and items[0].get("severity") == 1 and items[0].get("source") == "kson"
        result.record("F5 diagnostics/severity-source", bool(ok), "severity 1, source kson on syntax error")

        items = diag_items(c.diagnostics("invalid_indent.kson"))
        severities = sorted(i.get("severity") for i in items)
        ok = severities == [1, 2]
        result.record("F6 diagnostics/error-plus-warning", bool(ok), f"severities {severities} (error + misleading-indent warning)")
    finally:
        exit_code = c.shutdown_exit()
        cleanup(root)
    result.record("F7 diagnostics/clean-exit", exit_code == 0, f"server exit code {exit_code}")


def run_formatting_fixtures(result: FixtureResult, server: str, node: str) -> None:
    root, _ = new_workspace(with_schema=False)
    c = LspClient(root, server, node)
    c.initialize()
    c.drain(0.3)
    unformatted = read_fixture("documents/unformatted.kson")
    try:
        c.did_open("fmt.kson", unformatted)
        c.drain(0.5)

        expected = {
            ("F8 formatting/spaces-2", True, 2): "a: 1\nb:\n  c: 2",
            ("F9 formatting/spaces-8", True, 8): "a: 1\nb:\n        c: 2",
            ("F10 formatting/tabs", False, 4): "a: 1\nb:\n\tc: 2",
        }
        for (fixture, insert_spaces, tab_size), text in expected.items():
            r = c.format("fmt.kson", insert_spaces, tab_size)
            got = (r.get("result") or [{}])[0].get("newText")
            result.record(fixture, got == text, f"newText {got!r}")

        # Configuration change: plain (default) vs delimited style via
        # workspace/didChangeConfiguration, the path Zed users drive
        # through lsp.kson-language-server.settings (src/config.rs wraps
        # them under the "kson" section the server pulls).
        delimited = "{\n  a: 1\n  b: {\n    c: 2\n  }\n}"
        got_style = None
        for _ in range(10):  # the server re-pulls settings async (WI-3 note)
            r = c.format("fmt.kson", True, 2)
            got_style = (r.get("result") or [{}])[0].get("newText")
            if got_style == "a: 1\nb:\n  c: 2":
                break
            time.sleep(0.3)
        result.record("F11 formatting/style-plain-default", got_style == "a: 1\nb:\n  c: 2", f"newText {got_style!r}")

        c.set_workspace_settings({"format": {"formattingStyle": "delimited"}})
        got_style = None
        for _ in range(10):
            r = c.format("fmt.kson", True, 2)
            got_style = (r.get("result") or [{}])[0].get("newText")
            if got_style == delimited:
                break
            time.sleep(0.3)
        result.record("F12 formatting/style-delimited-config-change", got_style == delimited, f"newText {got_style!r}")

        # Idempotency: formatting already-formatted output is a no-op.
        c.did_open("delimited.kson", delimited + "\n")
        c.drain(0.5)
        r = c.format("delimited.kson", True, 2)
        edits = r.get("result") or []
        stable = not edits or (len(edits) == 1 and edits[0].get("newText") == delimited)
        result.record("F13 formatting/idempotent-delimited", bool(stable), f"edits {json.dumps(edits)[:160]}")

        # Code-lens format commands (OQ4): the lens carries no
        # indentation, so without client middleware the server's
        # two-space fallback applies — the behavior Zed users see.
        lens = c.request("textDocument/codeLens", {"textDocument": {"uri": c.uri("fmt.kson")}})
        lenses = lens.get("result") or []
        commands = {l["command"]["command"] for l in lenses if "command" in l}
        ok = {"kson.plainFormat", "kson.delimitedFormat"} <= commands
        result.record("F14 codelens/commands-offered", bool(ok), f"lens commands {sorted(commands)}")

        c.requests.clear()
        c.request(
            "workspace/executeCommand",
            {"command": "kson.plainFormat", "arguments": [{"documentUri": c.uri("fmt.kson"), "formattingStyle": "plain"}]},
        )
        c.drain(1.5)
        apply_edits = [m for m in c.requests.values() if m.get("method") == "workspace/applyEdit"]
        ok = False
        detail = "no workspace/applyEdit observed"
        if apply_edits:
            changes = apply_edits[-1]["params"]["edit"].get("changes", {})
            edits = list(changes.values())[0]
            got = edits[0].get("newText") if edits else None
            ok = got == "a: 1\nb:\n  c: 2"
            detail = f"applied newText {got!r} (server 2-space default, no middleware injection)"
        result.record("F15 codelens/plain-format-default-indent", ok, detail)
    finally:
        exit_code = c.shutdown_exit()
        cleanup(root)
    result.record("F16 formatting/clean-exit", exit_code == 0, f"server exit code {exit_code}")


def run_schema_fixtures(result: FixtureResult, server: str, node: str) -> None:
    root, _ = new_workspace(with_schema=True)
    c = LspClient(root, server, node)
    c.initialize()
    c.drain(0.8)
    try:
        c.did_open("schema_valid.config.kson", read_fixture("documents/schema_valid.config.kson"))
        c.did_open("schema_invalid.config.kson", read_fixture("documents/schema_invalid.config.kson"))
        c.drain(1.2)

        items = diag_items(c.diagnostics("schema_valid.config.kson"))
        result.record("F17 schema/valid-doc-clean", not items, f"diagnostics: {items}")

        items = diag_items(c.diagnostics("schema_invalid.config.kson"))
        ok = len(items) == 1 and "required" in items[0].get("message", "").lower()
        result.record("F18 schema/missing-required-diagnostic", bool(ok), f"{[i.get('message') for i in items]}")

        hover = c.request(
            "textDocument/hover",
            {"textDocument": {"uri": c.uri("schema_valid.config.kson")}, "position": {"line": 0, "character": 2}},
        )
        value = ((hover.get("result") or {}).get("contents") or {}).get("value", "")
        result.record("F19 schema/hover-doc-from-schema", "Application Name" in value, f"hover: {value!r}")

        comp = c.request(
            "textDocument/completion",
            {"textDocument": {"uri": c.uri("schema_valid.config.kson")}, "position": {"line": 2, "character": 0}},
        )
        labels = [i.get("label") for i in (comp.get("result") or {}).get("items", [])]
        result.record("F20 schema/property-completion", "features" in labels, f"labels {labels}")

        c.did_open("features.config.kson", read_fixture("documents/features.config.kson"))
        c.drain(1.0)
        comp = c.request(
            "textDocument/completion",
            {"textDocument": {"uri": c.uri("features.config.kson")}, "position": {"line": 2, "character": 12}},
        )
        labels = [i.get("label") for i in (comp.get("result") or {}).get("items", [])]
        ok = {"logging", "metrics", "tracing", "caching"} <= set(labels)
        result.record("F21 schema/enum-completion", bool(ok), f"labels {labels}")

        gds = c.request("kson/getDocumentSchema", {"uri": c.uri("schema_valid.config.kson")})
        res = gds.get("result") or {}
        ok = res.get("hasSchema") and res.get("schemaUri", "").endswith("config.schema.kson")
        result.record("F22 schema/getDocumentSchema-associated", bool(ok), f"{res}")

        # Schema reload: a client with file watching notifies
        # didChangeWatchedFiles when .kson-schema.kson changes (VS Code
        # does; Zed will not — OQ3 — see fixtures/parity/README.md).
        os.remove(os.path.join(root, ".kson-schema.kson"))
        c.send(
            {
                "jsonrpc": "2.0",
                "method": "workspace/didChangeWatchedFiles",
                "params": {"changes": [{"uri": c.uri(".kson-schema.kson"), "type": 3}]},
            }
        )
        c.drain(1.5)
        items = diag_items(c.diagnostics("schema_invalid.config.kson"))
        result.record("F23 schema/reload-on-watched-file-change", not items, f"diagnostics after reload: {items}")
    finally:
        exit_code = c.shutdown_exit()
        cleanup(root)
    result.record("F24 schema/clean-exit", exit_code == 0, f"server exit code {exit_code}")


def run_bundled_fixtures(result: FixtureResult, server: str, node: str) -> None:
    root, _ = new_workspace(with_schema=False)
    c = LspClient(root, server, node)
    # Initialization options exactly as the adapter now sends them
    # (src/config.rs defaults + inline metaschema contents).
    c.initialize(adapter_initialization_options())
    c.drain(0.5)
    try:
        c.did_open("metaschema_good.schema.kson", read_fixture("documents/metaschema_good.schema.kson"))
        c.did_open("metaschema_bad.schema.kson", read_fixture("documents/metaschema_bad.schema.kson"))
        c.drain(1.2)

        gds = c.request("kson/getDocumentSchema", {"uri": c.uri("metaschema_good.schema.kson")})
        res = gds.get("result") or {}
        ok = res.get("hasSchema") and res.get("schemaUri") == "bundled://metaschema/draft-07.schema.kson"
        result.record("F25 bundled/metaschema-attached", bool(ok), f"{res}")

        items = diag_items(c.diagnostics("metaschema_good.schema.kson"))
        result.record("F26 bundled/valid-metaschema-doc-clean", not items, f"diagnostics: {items}")

        items = diag_items(c.diagnostics("metaschema_bad.schema.kson"))
        messages = [i.get("message", "") for i in items]
        ok = bool(items) and any("sub-schema" in m or "Expected one of" in m for m in messages)
        result.record("F27 bundled/metaschema-violations-reported", bool(ok), f"{messages}")
    finally:
        exit_code = c.shutdown_exit()
        cleanup(root)
    result.record("F28 bundled/clean-exit", exit_code == 0, f"server exit code {exit_code}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--server", default=os.environ.get("KSON_SERVER", ""))
    parser.add_argument("--node", default="node")
    args = parser.parse_args()

    result = FixtureResult()
    print("== static fixtures ==")
    fixture_snippets(result)

    if args.server and os.path.isfile(args.server):
        print(f"\n== server fixtures ({args.server}) ==")
        run_diagnostics_fixtures(result, args.server, args.node)
        run_formatting_fixtures(result, args.server, args.node)
        run_schema_fixtures(result, args.server, args.node)
        run_bundled_fixtures(result, args.server, args.node)
    else:
        print("\n== server fixtures ==")
        for fixture in [
            "F1-F7 diagnostics", "F8-F16 formatting", "F17-F24 schema", "F25-F28 bundled"
        ]:
            result.deferred(fixture, "no server bundle: set --server or $KSON_SERVER (build per README 'Local server setup')")

    passed = sum(1 for _, ok, _ in result.results if ok is True)
    failed = sum(1 for _, ok, _ in result.results if ok is False)
    deferred = sum(1 for _, ok, _ in result.results if ok is None)
    print(f"\n{passed} passed, {failed} failed, {deferred} deferred")
    return result.exit_code()


if __name__ == "__main__":
    sys.exit(main())
