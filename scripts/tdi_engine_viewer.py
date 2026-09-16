"""Read-only loopback viewer and paginated JSON API for the TDI catalogue.

No service is needed to execute the CLI's offline queries. This optional viewer
does not load artifact URLs, perform mutations or accept a Hub credential.
All producer-controlled strings are HTML-escaped; no scripts are embedded.
"""
from __future__ import annotations

from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path
import sqlite3
import urllib.parse

from tdi_engine_store import EngineStore
from tdi_research_views import render_catalogue, render_compare, render_searches, checked_campaign, checked_results, esc, page
import tdi_research_reporting as reporting
import tdi_experiment_supervisor as durable

MAX_RESPONSE_BYTES = 8 * 1024 * 1024


def make_handler(catalogue, *, reports=(), figures=False):
    """Create read-only handlers with explicit routes, host checks and response bounds."""
    if len(reports) > 8:
        raise durable.ContractError("viewer accepts at most eight explicit reports")
    selected = {}
    images = {}
    image_bytes = 0
    size = 0
    for path in reports:
        report = reporting.read_report(path)
        size += len(durable.canonical(report).encode())
        if size > 16 * 1024 * 1024:
            raise durable.ContractError("viewer report inventory exceeds 16 MiB")
        if report['identity'] in selected:
            raise durable.ContractError("duplicate viewer report")
        selected[report['identity']] = report
        if figures:
            for name, raw, _ in reporting.figure_bytes(report):
                if not name.endswith('.png'): continue
                image_bytes += len(raw)
                if image_bytes > 32 * 1024 * 1024:
                    raise durable.ContractError("viewer figure inventory exceeds 32 MiB")
                images[(report['identity'], name)] = raw

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
                query = urllib.parse.parse_qs(parsed.query, strict_parsing=True, keep_blank_values=True, max_num_fields=6)
                routes = {"/": {"campaign", "after", "phase", "domain", "step_after"},
                          "/api/campaigns": {"campaign", "after", "phase", "domain"},
                          "/compare": {"left", "right"}, "/searches": {"search", "after"},
                          "/api/searches": {"search", "after"}, "/reports": {"report"},
                          "/api/report": {"report"}, "/report.csv": {"report"}, "/figure": {"report", "name"}}
                if parsed.path not in routes:
                    self.failure(404, "unknown route"); return
                if parsed.scheme or parsed.netloc or set(query) - routes[parsed.path] or any(len(v) != 1 for v in query.values()):
                    raise durable.ContractError("invalid query")
                value = lambda key: query.get(key, [None])[0] or None
                after = int(value("after") or "0")
                campaign, phase, domain = value("campaign"), value("phase"), value("domain")
                media = "text/html; charset=utf-8"
                with EngineStore(catalogue, readonly=True) as store:
                    if parsed.path == "/":
                        data = render_catalogue(store, campaign, after=after, phase=phase, domain=domain, step_after=int(value("step_after") or "0"))
                    elif parsed.path == "/api/campaigns":
                        record = checked_campaign(store, campaign) if campaign else None
                        result = {"campaign": record, "results": checked_results(store, record, after=after, limit=10)} if record else store.list(after=after, phase=phase, domain=domain)
                        data = durable.canonical({"schema": 1, "result": result}).encode()
                        media = "application/json"
                    elif parsed.path == "/compare":
                        data = render_compare(store, value("left"), value("right"))
                    elif parsed.path == "/searches":
                        data = render_searches(store, value("search"), after=after)
                    elif parsed.path == "/api/searches":
                        result = store.get_search(value("search")) if value("search") else store.list_searches(after=after)
                        data = durable.canonical({"schema": 1, "result": result}).encode()
                        media = "application/json"
                    else:
                        key = value("report")
                        if key is None and parsed.path == "/reports":
                            content = '<h1>Selected research reports</h1><ul>' + ''.join('<li><a href="/reports?report=' + esc(k) + '">' + esc(r['kind']) + ' · ' + esc(k[:20]) + '</a></li>' for k, r in selected.items()) + '</ul>'
                            if not selected: content += '<p>No report selected at viewer startup. Use view --report with an explicit report file.</p>'
                            data = page(content)
                        elif key not in selected:
                            self.failure(404, "unknown selected report"); return
                        elif parsed.path == "/api/report":
                            data = durable.canonical(selected[key]).encode(); media = "application/json"
                        elif parsed.path == "/report.csv":
                            data = reporting.csv_bytes(selected[key]); media = "text/csv; charset=utf-8"
                        elif parsed.path == "/figure":
                            if (key, value("name")) not in images:
                                self.failure(404, "unknown selected figure"); return
                            data = images[(key, value("name"))]; media = "image/png"
                        else:
                            plots = ''.join('<img style="max-width:100%" alt="Recorded research values" src="/figure?report=' + key + '&amp;name=' + name + '">' for report_id, name in images if report_id == key)
                            data = page(reporting.report_html(selected[key], standalone=False) + plots + '<p><a href="/api/report?report=' + key + '">Original JSON</a> · <a href="/report.csv?report=' + key + '">Complete CSV</a></p>')
                if len(data) > MAX_RESPONSE_BYTES:
                    self.failure(413, "response exceeds viewer budget; use the CLI for this record")
                    return
                self.send_response(200)
                self.send_header("Content-Type", media)
                self.send_header("Content-Length", str(len(data)))
                self.send_header("Cache-Control", "no-store")
                self.send_header("X-Content-Type-Options", "nosniff")
                self.send_header("Content-Security-Policy", "default-src 'none'; img-src 'self'; style-src 'unsafe-inline'; form-action 'self'; frame-ancestors 'none'; base-uri 'none'")
                self.end_headers(); self.wfile.write(data)
            except (ValueError, KeyError, TypeError, OSError, sqlite3.Error, durable.StorageError):
                self.failure(400, "invalid query or unavailable catalogue")
    return Handler


def serve(catalogue, port=8765, *, reports=(), figures=False):
    """Serve only on 127.0.0.1; Ctrl-C stops the viewer without affecting workers."""
    if type(port) is not int or not 1 <= port <= 65535:
        raise durable.ContractError("invalid viewer port")
    with EngineStore(Path(catalogue), readonly=True):
        pass
    with HTTPServer(("127.0.0.1", port), make_handler(catalogue, reports=reports, figures=figures)) as server:
        server.timeout = 1
        try:
            server.serve_forever(poll_interval=0.25)
        except KeyboardInterrupt:
            pass
