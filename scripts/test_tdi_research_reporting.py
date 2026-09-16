"""Actual Hub/SciRust report exports and read-only consultation failure boundaries."""
import copy
import csv
from http.server import HTTPServer
import io
import json
import os
from pathlib import Path
import tempfile
import threading
import unittest
from unittest.mock import patch
import urllib.error
import urllib.request

import tdi_engine_runtime as runtime
from tdi_engine_store import EngineStore, identity, atomic_json
from tdi_engine_viewer import make_handler
from tdi_research_views import render_catalogue, render_compare, render_searches, dag_svg
import tdi_research_reporting as reporting
import tdi_experiment_supervisor as durable
from test_tdi_engine_unit import campaign_fixture
import test_tdi_engine_integration as operational
from test_tdi_research_analysis import fixture
from tdi_research_analysis import analyze
from tdi_scirust_client import SciRustStats


def series_fixture():
    report = {'schema': 1, 'kind': 'tdi-observed-series', 'domain': 'Development',
              'scientific_verdict': 'not-assessed', 'limitations': ['synthetic presentation fixture'],
              'series': [{'label': '=unsafe<script>', 'x_unit': 's', 'y_unit': 'points', 'x_semantics': 'physical-time',
                          'x': [0, 0.5, 3], 'y': [-2, None, 4],
                          'source': {'campaign': 'a' * 64, 'artifact_identity': 'b' * 64, 'provenance_identity': 'c' * 64}}]}
    report['identity'] = identity('tdi-observed-series/v1', report)
    return report


class PresentationBoundaryTests(unittest.TestCase):
    def test_csv_html_nulls_formula_escaping_hashes_and_failed_export(self):
        report = series_fixture()
        rows = list(csv.reader(io.StringIO(reporting.csv_bytes(report).decode())))
        self.assertEqual("'=unsafe<script>", rows[1][0])
        self.assertEqual('-2', rows[1][5]); self.assertEqual('', rows[2][5])
        self.assertNotIn('<script>', reporting.report_html(report))
        self.assertIn('Unavailable', reporting.report_html(report))
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'report'
            with patch('tdi_research_reporting.os.fsync', side_effect=OSError('injected disk failure')):
                with self.assertRaises(OSError): reporting.export_report(report, output)
            self.assertFalse((output / 'manifest.json').exists())
            output = Path(directory) / 'complete'
            manifest = reporting.export_report(report, output)
            for member in manifest['files']:
                self.assertEqual(member['sha256'], durable.file_digest(output / member['name']))
            self.assertEqual(report, reporting.read_report(output / 'report.json'))
            with self.assertRaises(durable.StorageError): reporting.export_report(report, output)
            bad = copy.deepcopy(report); bad['series'][0]['y'][0] = 500
            with self.assertRaises(durable.ContractError): reporting.validate_report(bad)
            bad['identity'] = identity('tdi-observed-series/v1', {k: v for k, v in bad.items() if k != 'identity'})
            bad['domain'] = 'Confirmation'
            bad['identity'] = identity('tdi-observed-series/v1', {k: v for k, v in bad.items() if k != 'identity'})
            with self.assertRaises(durable.ContractError): reporting.validate_report(bad)

    def test_http_selection_snapshot_no_mutation_host_bounds_and_error_envelope(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); catalogue = root / 'catalogue.sqlite'
            with EngineStore(catalogue) as store:
                campaign = store.create(campaign_fixture(), 'http://127.0.0.1:8477')
            report_path = root / 'selected.json'; report = series_fixture(); atomic_json(report_path, report)
            before = catalogue.read_bytes()
            figures = os.environ.get('TDI_REPORT_FIGURES') == '1'
            handler = make_handler(catalogue, reports=[report_path], figures=figures)
            # Report bytes were frozen at startup; later path contents cannot
            # inject another report into the HTTP allowlist.
            report_path.write_text('{}')
            with HTTPServer(('127.0.0.1', 0), handler) as server:
                thread = threading.Thread(target=server.serve_forever, daemon=True); thread.start()
                origin = f'http://127.0.0.1:{server.server_port}'
                try:
                    for path, expected in (('/', b'Research campaigns'), ('/?campaign=' + campaign, b'not observed'),
                            ('/searches', b'No recorded searches'), ('/compare?left=' + campaign + '&right=' + campaign, b'Same declared root protocol: true'),
                            ('/reports?report=' + report['identity'], b'Unavailable')):
                        with urllib.request.urlopen(origin + path) as response:
                            self.assertIn(expected, response.read()); self.assertIn("default-src 'none'", response.headers['Content-Security-Policy'])
                    with urllib.request.urlopen(origin + '/api/report?report=' + report['identity']) as response:
                        self.assertEqual(report, json.load(response))
                    if figures:
                        with urllib.request.urlopen(origin + '/figure?report=' + report['identity'] + '&name=figure-00.png') as response:
                            self.assertEqual('image/png', response.headers['Content-Type'])
                            self.assertTrue(response.read().startswith(b'\x89PNG'))
                    for path, code in (('/api/campaigns?domain=Confirmation', 400), ('/api/campaigns?after=-1', 400),
                                       ('/api/report?report=../../selected.json', 404), ('/api/report?report=a&report=b', 400)):
                        with self.assertRaises(urllib.error.HTTPError) as failure: urllib.request.urlopen(origin + path)
                        self.assertEqual(code, failure.exception.code); self.assertEqual('error', json.load(failure.exception)['status'])
                    request = urllib.request.Request(origin + '/', headers={'Host': 'attacker.example'})
                    with self.assertRaises(urllib.error.HTTPError) as failure: urllib.request.urlopen(request)
                    self.assertEqual(403, failure.exception.code)
                    with self.assertRaises(urllib.error.HTTPError) as failure: urllib.request.urlopen(origin + '/', data=b'{}')
                    self.assertEqual(501, failure.exception.code)
                finally:
                    server.shutdown(); thread.join(timeout=5)
            self.assertEqual(before, catalogue.read_bytes())

    def test_dag_data_dependencies_domain_filter_and_incompatible_comparison(self):
        spec = campaign_fixture()
        spec['graph']['steps'][1]['after'] = []  # Data dependency still draws an edge.
        svg = dag_svg(spec['graph']['steps']); self.assertIn('marker-end', svg)
        with tempfile.TemporaryDirectory() as directory, EngineStore(Path(directory) / 'catalogue.sqlite') as store:
            left = store.create(runtime.canonical_campaign(spec), 'http://127.0.0.1:8477')
            changed = copy.deepcopy(spec); changed['domain'] = 'Validation'
            changed['graph']['root_plan_id'] = 'b' * 64
            for outputs in changed['outputs'].values():
                for rule in outputs.values(): rule['access_class'] = 'validation'
            right = store.create(runtime.canonical_campaign(changed), 'http://127.0.0.1:8477')
            self.assertEqual([right], [r['id'] for r in store.list(domain='Validation')])
            self.assertIn(b'Same declared root protocol: false', render_compare(store, left, right))
            self.assertIn(b'Development', render_catalogue(store, left))


class ActualReportTests(unittest.TestCase):
    setUp = operational.OperationalIntegrationTests.setUp
    start_hub = operational.OperationalIntegrationTests.start_hub
    stop_hub = operational.OperationalIntegrationTests.stop_hub
    cli = operational.OperationalIntegrationTests.cli

    @classmethod
    def setUpClass(cls):
        cls.hubd = Path(os.environ['TDI_HUBD_BIN']).resolve(strict=True)
        cls.worker = Path(os.environ['TDI_DURABLE_WORKER']).resolve(strict=True)
        binary = Path(os.environ['TDI_SCIRUST_STATS_BIN']).resolve(strict=True)
        cls.stats = SciRustStats(binary, durable.file_digest(binary), os.environ['TDI_SCIRUST_SOURCE_COMMIT'])

    def test_actual_scores_selected_without_imputation_and_cli_portable_exports(self):
        spec_path = self.root / 'spec.json'
        self.cli('fixture-plan', '--worker', self.worker, '--trials', 1, '--output', spec_path)
        campaign = self.cli('submit', spec_path)['campaign']; self.cli('run', campaign)
        self.assertEqual('completed', self.cli('--format', 'pretty', 'status')[0]['phase'])
        selection = [{'campaign': campaign, 'step': 'run-0', 'output': 'file:result', 'label': 'Finite counter scores',
                      'x_pointer': None, 'y_pointer': '/scores', 'x_unit': 'step', 'y_unit': 'points', 'x_semantics': 'logical-index'}]
        path = self.root / 'selection.json'; atomic_json(path, selection)
        result = self.cli('series-export', '--selections', path, '--output', self.root / 'series')
        report = reporting.read_report(self.root / 'series/report.json')
        with EngineStore(self.catalogue, readonly=True) as store:
            actual = store.result(campaign, 'run-0', 'file:result')['json']['scores']
            self.assertEqual(actual, report['series'][0]['y'])
            self.assertEqual(list(range(len(actual))), report['series'][0]['x'])
            self.assertIn(b'Recorded attempts', render_catalogue(store, campaign))
            changed = copy.deepcopy(selection); changed[0]['x_semantics'] = 'physical-time'
            with self.assertRaises(durable.ContractError): reporting.series_from_catalogue(store, changed)
            changed = copy.deepcopy(selection); changed[0]['y_pointer'] = '/plan_id'
            with self.assertRaises(durable.ContractError): reporting.series_from_catalogue(store, changed)
        second = self.cli('report-export', self.root / 'series/report.json', self.root / 'second')
        self.assertEqual(result['report_identity'], second['report_identity'])

    def test_real_scirust_intervals_exclusions_and_optional_figure_formats(self):
        protocol, observations = fixture(); report = analyze(protocol, observations, self.stats)
        self.assertEqual(report, reporting.validate_report(report))
        columns, rows = reporting.table_rows(report)
        self.assertEqual([2, 1], rows[0][7:9])
        self.assertEqual(5, rows[0][3]); self.assertEqual('points', rows[0][2])
        incomplete = copy.deepcopy(observations)
        incomplete['rows'] = [r for r in incomplete['rows'] if r['unit'] != 'u1']
        empty = analyze(protocol, incomplete, self.stats)
        self.assertIn('Unavailable', reporting.report_html(empty))
        self.assertEqual('insufficient-units', reporting.table_rows(empty)[1][0][-1])
        bad = copy.deepcopy(report); bad['results'][0]['included_units'] += 1
        bad['identity'] = identity('tdi-paired-analysis/v1', {k: v for k, v in bad.items() if k != 'identity'})
        with self.assertRaises(durable.ContractError): reporting.validate_report(bad)
        if os.environ.get('TDI_REPORT_FIGURES') != '1': return
        # Real SciRust intervals; the public fixture observations are labeled
        # as caller assertions, never presented as measured performance.
        output = self.root / 'figures'; manifest = reporting.export_report(report, output, figures=True)
        members = {r['name']: r for r in manifest['files']}
        self.assertTrue((output / 'figure-00.png').read_bytes().startswith(b'\x89PNG'))
        self.assertTrue((output / 'figure-00.pdf').read_bytes().startswith(b'%PDF'))
        self.assertIn('points', (output / 'figure-00.svg').read_text())
        self.assertIn('figure-00.png', (output / 'index.html').read_text())
        for member in members.values(): self.assertEqual(member['sha256'], durable.file_digest(output / member['name']))
        reporting.export_report(empty, self.root / 'empty-figures', figures=True)
        reporting.export_report(series_fixture(), self.root / 'series-figures', figures=True)


if __name__ == '__main__': unittest.main()
