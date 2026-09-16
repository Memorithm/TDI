"""Read-only loopback viewer and paginated JSON API for the TDI catalogue.

No service is needed to execute the CLI's offline queries. This optional viewer
does not load artifact URLs, perform mutations or accept a Hub credential.
All producer-controlled strings are HTML-escaped; no scripts are embedded.
"""
from __future__ import annotations

import html
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path
import sqlite3
import urllib.parse

from tdi_engine_store import EngineStore
import tdi_experiment_supervisor as durable

MAX_RESPONSE_BYTES = 8 * 1024 * 1024


def render_catalogue(store, campaign=None, *, after=0, phase=None):
    """Render actual catalogue records, bounded pages and honest empty states."""
    esc = lambda value: html.escape(str(value), quote=True)
    style = "body{font:16px system-ui;margin:40px;max-width:1200px;color:#172b3a;background:#fafbfd}h1{font-size:30px}table{border-collapse:collapse;width:100%}td,th{padding:12px;border-bottom:1px solid #ced7de;text-align:left}pre{white-space:pre-wrap;overflow-wrap:anywhere;background:#edf2f5;padding:16px}a{color:#126782}.muted{color:#566575}"
    head = '<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>TDI research campaigns</title><style>' + style + '</style><body><h1>TDI research campaigns</h1><p class="muted">Durable execution evidence · technical completion does not imply scientific benefit.</p>'
    if campaign:
        record = store.get(campaign)
        rows = store.results(campaign, after=after, limit=50)
        content = '<p><a href="/">All campaigns</a></p><h2>' + esc(record["phase"]) + '</h2>'
        content += '<details><summary>Plan, dependencies and execution attempts</summary><pre>' + esc(durable.canonical(record)) + '</pre></details>'
        content += '<h2>Verified outputs</h2>'
        if not rows:
            content += '<p>No verified outputs in this page. The campaign may be incomplete.</p>'
        for row in rows:
            evidence = row["evidence"]
            content += '<details><summary>' + esc(row["step"] + ' / ' + row["output"]) + '</summary><pre>' + esc(durable.canonical(evidence["json"])) + '</pre><p>Artifact ' + esc(evidence["artifact_identity"]) + '</p><details><summary>Provenance</summary><pre>' + esc(durable.canonical(evidence["provenance"])) + '</pre></details></details>'
        if len(rows) == 50:
            content += '<p><a href="/?campaign=' + esc(campaign) + '&amp;after=' + str(after + 50) + '">Next outputs</a></p>'
    else:
        rows = store.list(after=after, limit=50, phase=phase)
        content = '<form><label>State <input name="phase" value="' + esc(phase or '') + '"></label> <button>Filter</button></form>'
        content += '<table><thead><tr><th>Campaign</th><th>State</th><th>Hub workflow</th></tr></thead><tbody>'
        for row in rows:
            content += '<tr><td><a href="/?campaign=' + esc(row["id"]) + '">' + esc(row["id"][:20]) + '</a></td><td>' + esc(row["phase"]) + '</td><td>' + esc(row["workflow"] or 'Not submitted') + '</td></tr>'
        content += '</tbody></table>'
        if not rows:
            content += '<p>No campaigns match this page.</p>'
        if len(rows) == 50:
            content += '<p><a href="/?after=' + str(after + 50) + '&amp;phase=' + urllib.parse.quote(phase or '') + '">Next campaigns</a></p>'
    return (head + content + '</body></html>').encode()


def make_handler(catalogue):
    """Create read-only handlers with explicit routes, host checks and response bounds."""
    class Handler(BaseHTTPRequestHandler):
        def setup(self):
            super().setup()
            self.connection.settimeout(5)

        def log_message(self, *_):
            pass

        def failure(self, code, message):
            if self.path.startswith("/api/"):
                data = durable.canonical({"schema": 1, "status": "error", "code": code, "error": message}).encode()
                self.send_response(code)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(data)))
                self.send_header("Cache-Control", "no-store")
                self.end_headers()
                self.wfile.write(data)
            else:
                self.send_error(code, message)

        def do_GET(self):
            host = self.headers.get("Host", "")
            if host != f"127.0.0.1:{self.server.server_port}":
                self.failure(403, "loopback Host required")
                return
            try:
                parsed = urllib.parse.urlsplit(self.path)
                query = urllib.parse.parse_qs(parsed.query, strict_parsing=True, max_num_fields=5)
                if set(query) - {"campaign", "after", "phase"} or any(len(v) != 1 for v in query.values()):
                    raise durable.ContractError("invalid query")
                after = int(query.get("after", ["0"])[0])
                campaign = query.get("campaign", [None])[0]
                phase = query.get("phase", [None])[0]
                with EngineStore(catalogue, readonly=True) as store:
                    if parsed.path == "/":
                        data = render_catalogue(store, campaign, after=after, phase=phase)
                        media = "text/html; charset=utf-8"
                    elif parsed.path == "/api/campaigns":
                        result = {"campaign": store.get(campaign), "results": store.results(campaign, after=after, limit=10)} if campaign else store.list(after=after, phase=phase)
                        data = durable.canonical({"schema": 1, "result": result}).encode()
                        media = "application/json"
                    else:
                        self.failure(404, "unknown route"); return
                if len(data) > MAX_RESPONSE_BYTES:
                    self.failure(413, "response exceeds viewer budget; use the CLI for this record")
                    return
                self.send_response(200)
                self.send_header("Content-Type", media)
                self.send_header("Content-Length", str(len(data)))
                self.send_header("Cache-Control", "no-store")
                self.send_header("X-Content-Type-Options", "nosniff")
                self.send_header("Content-Security-Policy", "default-src 'none'; style-src 'unsafe-inline'; form-action 'self'; frame-ancestors 'none'; base-uri 'none'")
                self.end_headers(); self.wfile.write(data)
            except (ValueError, OSError, sqlite3.Error, durable.StorageError):
                self.failure(400, "invalid query or unavailable catalogue")
    return Handler


def serve(catalogue, port=8765):
    """Serve only on 127.0.0.1; Ctrl-C stops the viewer without affecting workers."""
    if type(port) is not int or not 1 <= port <= 65535:
        raise durable.ContractError("invalid viewer port")
    with EngineStore(Path(catalogue), readonly=True):
        pass
    with HTTPServer(("127.0.0.1", port), make_handler(catalogue)) as server:
        server.timeout = 1
        try:
            server.serve_forever(poll_interval=0.25)
        except KeyboardInterrupt:
            pass
