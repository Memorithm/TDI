"""Read-only research reports, explicit catalogue curves and portable figures.

No statistical estimate is computed here. Tables and figures use recorded
estimates/intervals or explicitly selected observed arrays. Missing values and
unavailable intervals remain visible. Export never changes a source report.
"""
from __future__ import annotations

import csv
import hashlib
import html
import importlib.metadata
import io
import os
from pathlib import Path
import re

import tdi_artifact_contract as artifacts
import tdi_engine_runtime as runtime
import tdi_execution_graph as graphs
import tdi_experiment_supervisor as durable
from tdi_engine_store import atomic_json, identity
from tdi_research_analysis import canonical_protocol, finite

MAX_REPORT_BYTES = 8 * 1024 * 1024


def _text(value):
    if not isinstance(value, str) or not 1 <= len(value) <= 256 or any(ord(c) < 32 for c in value):
        raise durable.ContractError("invalid report text")
    return value


def validate_report(report):
    """Validate a bounded non-final report and its self-contained content identity.

    Identity consistency is not external attestation or proof of a scientific
    claim. Supplied reports retain their own source/provenance declarations.
    """
    if (not isinstance(report, dict) or type(report.get('schema')) is not int or report['schema'] != 1
            or report.get('scientific_verdict') != 'not-assessed'):
        raise durable.ContractError('unsupported report envelope or verdict')
    kind = report.get('kind')
    domains = {'tdi-paired-analysis': 'tdi-paired-analysis/v1', 'tdi-sensitivity-analysis': 'tdi-sensitivity-analysis/v1',
               'tdi-observed-series': 'tdi-observed-series/v1'}
    if kind not in domains or len(durable.canonical(report).encode()) > MAX_REPORT_BYTES:
        raise durable.ContractError('unsupported or oversized report')
    if report.get('identity') != identity(domains[kind], {k: v for k, v in report.items() if k != 'identity'}):
        raise durable.ContractError('report identity mismatch')
    if kind == 'tdi-paired-analysis':
        p = canonical_protocol(report['protocol'])
        if report['protocol_identity'] != identity('tdi-analysis-protocol/v1', p):
            raise durable.ContractError('report protocol identity mismatch')
        results = report['results']
        if not isinstance(results, list) or not 1 <= len(results) <= 136:
            raise durable.ContractError('invalid report comparison inventory')
        expected = {(c['id'], s) for c in p['comparisons'] for s in [None, *p['strata']]}
        actual = [(r['comparison']['id'], r['stratum']) for r in results]
        if len(set(actual)) != len(actual) or set(actual) != expected or report['effect_convention'] != 'candidate-minus-reference':
            raise durable.ContractError('incomplete report family or effect convention')
        if report['observations_identity'] != identity('tdi-analysis-observations/v1', report['observations']):
            raise durable.ContractError('report observations identity mismatch')
        for row in results:
            if row['comparison'] not in p['comparisons'] or row['stratum'] not in [None, *p['strata']]:
                raise durable.ContractError('unplanned report comparison')
            for key in ('included_units', 'excluded_units', 'expected_units'):
                if type(row[key]) is not int or row[key] < 0: raise durable.ContractError('invalid report denominator')
            if row['included_units'] + row['excluded_units'] != row['expected_units']:
                raise durable.ContractError('report unit counts do not balance')
            if row['expected_units'] != sum(row['stratum'] is None or u['stratum'] == row['stratum'] for u in p['units']):
                raise durable.ContractError('report unit count differs from protocol')
            interval = row['interval']
            if (row['status'] != ('analyzed' if interval is not None else 'insufficient-units')
                    or (interval is None) != (row['included_units'] < 2)):
                raise durable.ContractError('report status differs from available units')
            if interval is not None:
                for key in ('estimate', 'lower', 'upper', 'confidence'): finite(interval[key])
                if interval['lower'] > interval['upper'] or not 0 < interval['confidence'] < 1:
                    raise durable.ContractError('invalid report interval')
    elif kind == 'tdi-sensitivity-analysis':
        plan = report['plan']; p = plan['protocol']
        if (p['domain'] not in ('Development', 'Validation') or p['purpose'] != 'exploratory-sensitivity'
                or p['method'] not in ('morris', 'sobol', 'ablation') or not 1 <= len(p['factors']) <= 8):
            raise durable.ContractError('invalid non-final sensitivity report')
        if plan['identity'] != identity('tdi-sensitivity-plan/v1', {k: v for k, v in plan.items() if k != 'identity'}):
            raise durable.ContractError('sensitivity plan identity mismatch')
        for f in p['factors']: _text(f['name']); _text(f['unit'])
        _text(p['output_unit'])
        keys = {'morris': ('mu', 'mu_star', 'sigma'), 'sobol': ('first', 'total'), 'ablation': ('effects',)}[p['method']]
        for key in keys:
            values = report['result'][key]
            if not isinstance(values, list) or len(values) != len(p['factors']): raise durable.ContractError('invalid sensitivity estimate shape')
            for value in values: finite(value)
    else:
        if report.get('domain') not in ('Development', 'Validation') or not isinstance(report['series'], list) or not 1 <= len(report['series']) <= 16:
            raise durable.ContractError('invalid observed-series inventory')
        for series in report['series']:
            for key in ('label', 'x_unit', 'y_unit'): _text(series[key])
            if series['x_semantics'] not in ('logical-index', 'physical-time', 'declared-coordinate'):
                raise durable.ContractError('explicit series coordinate semantics required')
            if not isinstance(series['x'], list) or not isinstance(series['y'], list) or not 1 <= len(series['x']) <= 4096 or len(series['x']) != len(series['y']):
                raise durable.ContractError('invalid observed-series coordinates')
            for value in series['x']: finite(value)
            for value in series['y']:
                if value is not None: finite(value)
            if any(a > b for a, b in zip(series['x'], series['x'][1:])):
                raise durable.ContractError('series coordinates must retain nondecreasing order')
            for key in ('campaign', 'artifact_identity', 'provenance_identity'):
                graphs._sha256(series['source'][key], key)
    return report


def read_report(path):
    """Read an explicitly selected regular report file with strict bounds."""
    path = Path(path)
    if path.is_symlink() or not path.is_file(): raise durable.ContractError('regular report file required')
    with path.open('rb') as stream:
        return validate_report(durable.strict_json(stream.read(MAX_REPORT_BYTES + 1), max_bytes=MAX_REPORT_BYTES, max_items=1000000))


def _array(value, pointer):
    if not isinstance(pointer, str) or not pointer.startswith('/') or len(pointer) > 256:
        raise durable.ContractError('explicit array JSON pointer required')
    for token in pointer[1:].split('/'):
        if re.search(r'~(?![01])', token): raise durable.ContractError('invalid pointer escape')
        token = token.replace('~1', '/').replace('~0', '~')
        if isinstance(value, dict) and token in value: value = value[token]
        elif isinstance(value, list) and re.fullmatch(r'0|[1-9][0-9]{0,5}', token) and int(token) < len(value): value = value[int(token)]
        else: raise durable.ContractError('selected series array is absent')
    if not isinstance(value, list) or not 1 <= len(value) <= 4096: raise durable.ContractError('selected array requires 1..4096 points')
    return value


def series_from_catalogue(store, selections):
    """Select actual result arrays; logical indices are labeled, never physical time.

    `x_pointer=null` is allowed only with logical-index semantics and unit step.
    Real null ordinates remain gaps. Sources must be successful declared outputs
    from terminal Development/Validation campaigns in one common domain.
    """
    if not isinstance(selections, list) or not 1 <= len(selections) <= 16:
        raise durable.ContractError('requires 1..16 explicit series selectors')
    series, domain = [], None
    for selection in selections:
        if not isinstance(selection, dict) or set(selection) != {'campaign', 'step', 'output', 'label', 'x_pointer', 'y_pointer', 'x_unit', 'y_unit', 'x_semantics'}:
            raise durable.ContractError('invalid series selector')
        for key in ('label', 'x_unit', 'y_unit'): _text(selection[key])
        record = store.get(selection['campaign']); spec = runtime.canonical_campaign(record['spec'])
        if domain is not None and domain != spec['domain']: raise durable.ContractError('series cannot mix execution domains')
        domain = spec['domain']
        if record['phase'] not in ('completed', 'failed', 'cancelled', 'imported'):
            raise durable.ContractError('series requires a terminal source campaign')
        state, steps = runtime.validated_snapshot(spec, record['snapshot'])
        step = steps.get(selection['step'])
        if state not in runtime.TERMINAL or step is None or step.get('state') != 'succeeded' or selection['output'] not in spec['outputs'].get(selection['step'], {}):
            raise durable.ContractError('series source is not a successful declared output')
        evidence = store.result(selection['campaign'], selection['step'], selection['output'])
        descriptor = artifacts.canonical_artifact(evidence['descriptor'])
        if (descriptor['access_class'] != domain.lower() or artifacts.artifact_identity(descriptor) != evidence['artifact_identity']
                or artifacts.provenance_identity(evidence['provenance']) != evidence['provenance_identity']):
            raise durable.ContractError('series evidence integrity/access mismatch')
        y = _array(evidence['json'], selection['y_pointer'])
        if selection['x_pointer'] is None:
            if selection['x_semantics'] != 'logical-index' or selection['x_unit'] != 'step':
                raise durable.ContractError('generated indices must be declared logical steps')
            x = list(range(len(y)))
        else:
            if selection['x_semantics'] not in ('physical-time', 'declared-coordinate'):
                raise durable.ContractError('observed x coordinates require explicit semantics')
            x = _array(evidence['json'], selection['x_pointer'])
        series.append({**{k: selection[k] for k in ('label', 'x_unit', 'y_unit', 'x_semantics')}, 'x': x, 'y': y,
                       'source': {**{k: selection[k] for k in ('campaign', 'step', 'output', 'x_pointer', 'y_pointer')},
                                  'artifact_identity': evidence['artifact_identity'], 'provenance_identity': evidence['provenance_identity']}})
    report = {'schema': 1, 'kind': 'tdi-observed-series', 'domain': domain, 'series': series,
              'scientific_verdict': 'not-assessed', 'limitations': ['selected values only; no interpolation or missing-value imputation',
                'coordinate semantics and units are explicitly declared by the selection', 'a trajectory plot does not establish causality']}
    report['identity'] = identity('tdi-observed-series/v1', report)
    return validate_report(report)


def table_rows(report):
    """Project report values into explicit columns, preserving absent intervals."""
    report = validate_report(report)
    if report['kind'] == 'tdi-paired-analysis':
        columns = ['comparison', 'stratum', 'unit', 'estimate', 'lower', 'upper', 'confidence', 'included_units', 'excluded_units', 'status']
        rows = []
        for r in report['results']:
            interval = r['interval'] or {}
            rows.append([r['comparison']['id'], r['stratum'] or 'all', r['comparison']['unit'],
                         *[interval.get(k) for k in ('estimate', 'lower', 'upper', 'confidence')], r['included_units'], r['excluded_units'], r['status']])
        return columns, rows
    if report['kind'] == 'tdi-sensitivity-analysis':
        p = report['plan']['protocol']; method = p['method']
        keys = {'morris': ['mu', 'mu_star', 'sigma'], 'sobol': ['first', 'total'], 'ablation': ['effects']}[method]
        unit = 'dimensionless' if method == 'sobol' else p['output_unit']
        return ['factor', 'unit', *keys], [[f['name'], unit, *[report['result'][k][i] for k in keys]] for i, f in enumerate(p['factors'])]
    columns = ['series', 'x_semantics', 'x_unit', 'y_unit', 'x', 'y', 'artifact_identity']
    return columns, [[s['label'], s['x_semantics'], s['x_unit'], s['y_unit'], x, y, s['source']['artifact_identity']]
                     for s in report['series'] for x, y in zip(s['x'], s['y'])]


def csv_bytes(report):
    """Export explicit selected columns; textual spreadsheet formulas are escaped."""
    columns, rows = table_rows(report)
    stream = io.StringIO(newline=''); writer = csv.writer(stream)
    writer.writerow(columns)
    for row in rows:
        writer.writerow([("'" + x if x and x[0] in '=+-@' else x) if isinstance(x, str) else '' if x is None else x for x in row])
    return stream.getvalue().encode('utf-8')


def report_html(report, *, standalone=True):
    """Render actual estimates, denominators, source references and missing states."""
    report = validate_report(report); columns, rows = table_rows(report)
    esc = lambda x: html.escape(str(x), quote=True)
    title = {'tdi-paired-analysis': 'Paired effects and uncertainty', 'tdi-sensitivity-analysis': 'Sensitivity and ablation', 'tdi-observed-series': 'Observed trajectories'}[report['kind']]
    text = '<h1>' + title + '</h1><p>Scientific verdict: not assessed.</p><p>Report ' + esc(report['identity']) + '</p>'
    text += '<p>Content identity verified. Source declarations remain those of the supplied report.</p><table><thead><tr>'
    text += ''.join('<th>' + esc(c) + '</th>' for c in columns) + '</tr></thead><tbody>'
    for row in rows[:500]:
        text += '<tr>' + ''.join('<td>' + ('Unavailable' if x is None else esc(x)) + '</td>' for x in row) + '</tr>'
    text += '</tbody></table>'
    if len(rows) > 500: text += '<p>First 500 points shown; the complete selected data remain in JSON/CSV exports.</p>'
    if report['kind'] == 'tdi-sensitivity-analysis': text += '<p>Point estimates only. Confidence intervals are unavailable.</p>'
    if report['kind'] == 'tdi-paired-analysis':
        text += '<p>Effect convention: ' + esc(report['effect_convention']) + '. Repeats are averaged within declared units before resampling.</p>'
        text += '<p>Cost status: ' + esc(report['cost_status']) + '</p>'
    costs = report.get('cost_measurements', report.get('observations', {}).get('cost_measurements'))
    text += '<details><summary>Costs and availability</summary><pre>' + esc(durable.canonical(costs) if costs is not None else 'Not measured or selected in this report') + '</pre></details>'
    text += '<details><summary>Scope, provenance and limitations</summary><pre>' + esc(durable.canonical(report)) + '</pre></details>'
    if not standalone: return text
    return '<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>TDI research report</title><style>body{font:16px system-ui;max-width:1200px;margin:40px auto;padding:0 20px;color:#172b3a}table{border-collapse:collapse;width:100%}td,th{padding:10px;border-bottom:1px solid #ccd5df;text-align:left}pre{white-space:pre-wrap;overflow-wrap:anywhere}details{margin:20px 0}</style><body>' + text + '</body></html>'


def figure_bytes(report):
    """Render optional SVG/PNG/PDF figures without modifying scientific values.

    Paired intervals are drawn independently of the point estimate: a percentile
    interval need not contain the empirical estimate. Different units have
    separate figures; null series values break the line without interpolation.
    """
    validate_report(report)
    try:
        if importlib.metadata.version('matplotlib') != '3.11.2' or importlib.metadata.version('numpy') != '2.5.3':
            raise durable.ContractError('figure dependencies differ from the pinned reporting profile')
    except importlib.metadata.PackageNotFoundError as error:
        raise durable.ContractError('install the optional requirements-reporting.txt profile') from error
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    figures = []
    if report['kind'] == 'tdi-paired-analysis':
        units = sorted({r['comparison']['unit'] for r in report['results']})
        for unit in units:
            rows = [r for r in report['results'] if r['comparison']['unit'] == unit]
            fig, ax = plt.subplots(figsize=(9, max(3, .32 * len(rows) + 1.5)))
            labels = []
            for i, r in enumerate(rows):
                v = r['interval']
                label = r['comparison']['id'] + ' / ' + (r['stratum'] or 'all') + ' · n=' + str(r['included_units'])
                if v is not None: label += ' · CI ' + format(100 * v['confidence'], '.3g') + '%'
                labels.append(label)
                if v is None: ax.text(.02, i, 'insufficient units', transform=ax.get_yaxis_transform(), va='center')
                else:
                    ax.hlines(i, v['lower'], v['upper'], color='#24778b', linewidth=2)
                    ax.scatter([v['estimate']], [i], color='#123f58', zorder=3)
            ax.set_yticks(range(len(rows)), labels); ax.set_ylim(len(rows) - .5, -.5); ax.axvline(0, color='#97a5b0', linewidth=1)
            ax.set_xlabel('Candidate minus reference [' + unit + ']'); ax.set_title('Recorded paired effects and percentile intervals')
            figures.append((fig, {'unit': unit, 'kind': 'paired-effects'}))
    elif report['kind'] == 'tdi-sensitivity-analysis':
        p = report['plan']['protocol']; method = p['method']; keys = {'morris': ['mu', 'mu_star', 'sigma'], 'sobol': ['first', 'total'], 'ablation': ['effects']}[method]
        fig, ax = plt.subplots(figsize=(9, 4))
        n = len(p['factors']); width = .75 / len(keys)
        for j, key in enumerate(keys):
            ax.bar([i + (j - (len(keys) - 1) / 2) * width for i in range(n)], report['result'][key], width, label=key)
        ax.set_xticks(range(n), [f['name'] for f in p['factors']]); ax.axhline(0, color='#97a5b0', linewidth=1)
        unit = 'dimensionless' if method == 'sobol' else p['output_unit']
        ax.set_ylabel(unit + (' per normalized factor range' if method == 'morris' else ''))
        ax.set_title(method.capitalize() + ' point estimates; intervals unavailable'); ax.legend()
        figures.append((fig, {'unit': unit, 'kind': method}))
    else:
        for s in report['series']:
            fig, ax = plt.subplots(figsize=(9, 4)); ax.plot(s['x'], s['y'], marker='o', markersize=3, color='#24778b')
            ax.set_xlabel(s['x_semantics'] + ' [' + s['x_unit'] + ']'); ax.set_ylabel(s['y_unit']); ax.set_title(s['label'])
            figures.append((fig, {'x_unit': s['x_unit'], 'y_unit': s['y_unit'], 'x_semantics': s['x_semantics'], 'kind': 'observed-series'}))
    outputs = []
    try:
        for index, (fig, metadata) in enumerate(figures):
            fig.text(.01, .01, 'TDI · exploratory, not assessed · report ' + report['identity'][:20], fontsize=8, color='#52616b')
            fig.tight_layout(rect=(0, .045, 1, 1))
            for extension in ('svg', 'png', 'pdf'):
                stream = io.BytesIO()
                info = {'Creator': 'TDI research reporting', 'Subject': report['identity'], 'CreationDate': None} if extension == 'pdf' else {'Creator': 'TDI research reporting', 'Description': report['identity']}
                fig.savefig(stream, format=extension, dpi=160, metadata=info)
                outputs.append((f'figure-{index:02d}.{extension}', stream.getvalue(), metadata))
    finally:
        for fig, _ in figures: plt.close(fig)
    return outputs


def export_report(report, output, *, figures=False):
    """Publish a new complete report directory with a manifest written last.

    Missing/failed rendering leaves no completion manifest. A pre-existing
    destination is refused. File hashes identify the generated presentation;
    cross-platform pixel-identical output is not promised.
    """
    validate_report(report)
    output = Path(output)
    if output.exists() or output.is_symlink(): raise durable.StorageError('report export directory already exists')
    rendered = figure_bytes(report) if figures else []
    output.mkdir(parents=False)
    page = report_html(report)
    links = '<p><a href="data.csv">Complete selected data (CSV)</a> · <a href="report.json">Original report (JSON)</a></p>'
    for name, _, _ in rendered:
        links += '<p><a href="' + name + '">' + name + '</a></p>'
        if name.endswith('.png'): links += '<img style="max-width:100%" alt="Recorded scientific values" src="' + name + '">'
    page = page.replace('</body>', links + '</body>')
    files = [('index.html', page.encode(), {'kind': 'html'}), ('data.csv', csv_bytes(report), {'kind': 'selected-table'}), *rendered]
    atomic_json(output / 'report.json', report)
    manifest = {'schema': 1, 'kind': 'tdi-report-export', 'report_identity': report['identity'], 'files': [],
                'renderer_sha256': durable.file_digest(Path(__file__)), 'figure_dependencies': {'matplotlib': '3.11.2', 'numpy': '2.5.3'} if figures else None}
    for name, raw, metadata in files:
        with (output / name).open('xb') as stream:
            stream.write(raw); stream.flush()
            os.fsync(stream.fileno())
        manifest['files'].append({'name': name, 'sha256': hashlib.sha256(raw).hexdigest(), 'bytes': len(raw), **metadata})
    raw = (output / 'report.json').read_bytes()
    manifest['files'].append({'name': 'report.json', 'sha256': hashlib.sha256(raw).hexdigest(), 'bytes': len(raw), 'kind': 'original-report'})
    manifest['identity'] = identity('tdi-report-export/v1', manifest)
    atomic_json(output / 'manifest.json', manifest)
    return manifest
