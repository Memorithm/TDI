"""Read-only HTML projections of actual campaign and search catalogue records."""
from __future__ import annotations

from collections import Counter
import html
import urllib.parse

import tdi_artifact_contract as artifacts
import tdi_engine_runtime as runtime
import tdi_experiment_supervisor as durable
from tdi_engine_store import identity


def esc(value):
    return html.escape(str(value), quote=True)


def page(content):
    style = "body{font:16px system-ui;margin:40px auto;padding:0 24px;max-width:1200px;color:#172b3a;background:#fafbfd}h1{font-size:30px}table{border-collapse:collapse;width:100%}td,th{padding:10px;border-bottom:1px solid #ced7de;text-align:left;overflow-wrap:anywhere}pre{white-space:pre-wrap;overflow-wrap:anywhere;background:#edf2f5;padding:16px}a{color:#126782}nav,details,form{margin:20px 0}.muted{color:#566575}svg{max-width:100%;height:auto}input,select,button{font:inherit;padding:6px}"
    return ('<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>TDI research catalogue</title><style>' + style + '</style><body><nav><a href="/">Campaigns</a> · <a href="/searches">Searches</a> · <a href="/reports">Reports</a> · <a href="/compare">Compare</a></nav><p class="muted">Read-only catalogue · technical completion does not imply scientific benefit.</p>' + content + '</body></html>').encode()


def table(columns, rows):
    return '<table><thead><tr>' + ''.join('<th>' + esc(x) + '</th>' for x in columns) + '</tr></thead><tbody>' + ''.join('<tr>' + ''.join('<td>' + ('Unavailable' if x is None else esc(x)) + '</td>' for x in row) + '</tr>' for row in rows) + '</tbody></table>'


def details(title, value):
    return '<details><summary>' + esc(title) + '</summary><pre>' + esc(durable.canonical(value)) + '</pre></details>'


def checked_campaign(store, campaign):
    record = store.get(campaign)
    spec = runtime.canonical_campaign(record['spec'])
    if identity('tdi-operational-campaign/v1', spec) != campaign:
        raise durable.ContractError('campaign identity mismatch')
    return record


def checked_results(store, record, *, after=0, limit=50):
    rows = store.results(record['id'], after=after, limit=limit)
    for row in rows:
        evidence = row['evidence']; descriptor = artifacts.canonical_artifact(evidence['descriptor'])
        if (row['output'] not in record['spec']['outputs'].get(row['step'], {})
                or descriptor['access_class'] != record['spec']['domain'].lower()
                or artifacts.artifact_identity(descriptor) != evidence['artifact_identity']
                or artifacts.provenance_identity(evidence['provenance']) != evidence['provenance_identity']):
            raise durable.ContractError('output evidence integrity or access mismatch')
    return rows


def dependencies(step):
    return sorted(set(step['after']) | {v['step'] for v in step['inputs'].values() if v['kind'] == 'step'})


def dag_svg(steps):
    """Draw actual explicit/data dependencies in topological levels, at most 32 nodes."""
    if len(steps) > 32:
        return '<p>Dependency diagram omitted above 32 steps; use the paginated step table and complete plan.</p>'
    pending = {s['key']: dependencies(s) for s in steps}; levels = {}
    while pending:
        ready = [k for k, parents in pending.items() if all(p in levels for p in parents)]
        if not ready: raise durable.ContractError('invalid dependency graph')
        for k in ready:
            levels[k] = 1 + max((levels[p] for p in pending[k]), default=-1)
            del pending[k]
    grouped = {}; positions = {}
    for k, level in levels.items(): grouped.setdefault(level, []).append(k)
    # At most four nodes horizontally; additional peers wrap within the level.
    y = 30
    for level in sorted(grouped):
        for i, k in enumerate(sorted(grouped[level])): positions[k] = (20 + (i % 4) * 235, y + (i // 4) * 80)
        y += ((len(grouped[level]) + 3) // 4) * 80 + 30
    svg = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 960 ' + str(y) + '" role="img" aria-label="Declared workflow dependencies"><defs><marker id="arrow" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto"><path d="M0,0 L8,4 L0,8" fill="#647b8c"/></marker></defs>'
    for step in steps:
        x, dy = positions[step['key']]
        for parent in dependencies(step):
            px, py = positions[parent]
            svg += f'<path d="M{px + 105},{py + 45} L{x + 105},{dy}" stroke="#647b8c" fill="none" marker-end="url(#arrow)"/>'
    for k, (x, dy) in positions.items():
        svg += f'<g><title>{esc(k)}</title><rect x="{x}" y="{dy}" width="210" height="45" rx="6" fill="#e4f0f3" stroke="#24778b"/><text x="{x + 10}" y="{dy + 28}" fill="#172b3a" font-size="13">{esc(k[:26])}</text></g>'
    return svg + '</svg>'


def render_catalogue(store, campaign=None, *, after=0, phase=None, domain=None, step_after=0):
    store._page(after, 50); store._page(step_after, 50)
    if campaign:
        record = checked_campaign(store, campaign); spec = record['spec']; steps = spec['graph']['steps']
        snapshot = record['snapshot']
        observed = runtime.validated_snapshot(spec, snapshot)[1] if snapshot is not None else {}
        counts = Counter(s.get('state', 'unknown') for s in observed.values())
        counts['not observed'] = len(steps) - len(observed)
        content = '<h1>' + esc(spec['graph']['name']) + '</h1><p>' + esc(spec['domain']) + ' · ' + esc(record['phase']) + ' · ' + esc(campaign) + '</p>'
        content += '<h2>Observed progress</h2>' + table(['Step state', 'Count'], sorted(counts.items()))
        content += '<p>Counts come from the last stored Hub snapshot; this page does not poll or dispatch workers.</p><h2>Dependencies</h2>' + dag_svg(steps)
        rows = []
        for step in steps[step_after:step_after + 50]:
            actual = observed.get(step['key'], {})
            attempts = actual.get('attempts', [])
            rows.append([step['key'], ', '.join(dependencies(step)) or 'root', actual.get('state', 'not observed'), len(attempts)])
        content += table(['Step', 'Depends on', 'Observed state', 'Recorded attempts'], rows)
        if step_after + 50 < len(steps):
            content += '<p><a href="/?campaign=' + esc(campaign) + '&amp;step_after=' + str(step_after + 50) + '">Next steps</a></p>'
        for step in steps[step_after:step_after + 50]:
            content += details('Attempts: ' + step['key'], observed.get(step['key'], {}).get('attempts', []))
        content += details('Complete plan and latest execution snapshot', record)
        rows = checked_results(store, record, after=after)
        content += '<h2>Verified outputs</h2>'
        if not rows: content += '<p>No verified outputs in this page. The campaign may be incomplete.</p>'
        for row in rows:
            evidence = row['evidence']
            content += details(row['step'] + ' / ' + row['output'], evidence['json'])
            content += '<p>Artifact ' + esc(evidence['artifact_identity']) + '</p>' + details('Provenance', evidence['provenance'])
        if len(rows) == 50: content += '<p><a href="/?campaign=' + esc(campaign) + '&amp;after=' + str(after + 50) + '">Next outputs</a></p>'
        return page(content)
    rows = store.list(after=after, limit=50, phase=phase, domain=domain)
    content = '<h1>Research campaigns</h1><form><label>State <input name="phase" value="' + esc(phase or '') + '"></label> <label>Domain <select name="domain"><option value="">All non-final records</option>'
    content += ''.join('<option' + (' selected' if d == domain else '') + '>' + d + '</option>' for d in ('Development', 'Validation')) + '</select></label> <button>Filter</button></form>'
    content += '<table><thead><tr><th>Campaign</th><th>State</th><th>Hub workflow</th></tr></thead><tbody>'
    for row in rows:
        content += '<tr><td><a href="/?campaign=' + esc(row['id']) + '">' + esc(row['id'][:20]) + '</a></td><td>' + esc(row['phase']) + '</td><td>' + esc(row['workflow'] or 'Not submitted') + '</td></tr>'
    content += '</tbody></table>'
    if not rows: content += '<p>No campaigns match this page.</p>'
    if len(rows) == 50: content += '<p><a href="/?' + esc(urllib.parse.urlencode({'after': after + 50, 'phase': phase or '', 'domain': domain or ''})) + '">Next campaigns</a></p>'
    return page(content)


def render_compare(store, left=None, right=None):
    content = '<h1>Compare recorded campaigns</h1><form><label>Left campaign <input name="left"></label> <label>Right campaign <input name="right"></label> <button>Compare</button></form>'
    if left is None and right is None: return page(content)
    if left is None or right is None: raise durable.ContractError('two explicit campaigns required')
    a, b = checked_campaign(store, left), checked_campaign(store, right)
    same = a['spec']['graph']['root_plan_id'] == b['spec']['graph']['root_plan_id']
    content += '<p>Same declared root protocol: ' + str(same).lower() + '. This administrative comparison does not establish statistical comparability.</p>'
    content += table(['Field', 'Left', 'Right'], [[key, a[key], b[key]] for key in ('id', 'phase', 'workflow')])
    content += table(['Field', 'Left', 'Right'], [[key, a['spec'][key], b['spec'][key]] for key in ('domain', 'purpose')])
    content += '<p>Effect sizes and uncertainty require a separately declared paired analysis and its complete unit inventory.</p>'
    content += details('Left plan', a['spec']) + details('Right plan', b['spec'])
    return page(content)


def render_searches(store, search=None, *, after=0):
    store._page(after, 50)
    if search is None:
        rows = store.list_searches(after=after, limit=50)
        content = '<h1>Scientific searches</h1><ul>' + ''.join('<li><a href="/searches?search=' + esc(r['id']) + '">' + esc(r['id'][:20]) + '</a> · ' + esc(r['phase']) + '</li>' for r in rows) + '</ul>'
        if not rows: content += '<p>No recorded searches in this page.</p>'
        if len(rows) == 50: content += '<a href="/searches?after=' + str(after + 50) + '">Next searches</a>'
        return page(content)
    record = store.get_search(search); snapshot = record['response']['snapshot']
    content = '<h1>Search ' + esc(search[:20]) + '</h1><p>' + esc(record['phase']) + ' · scientific verdict: ' + esc(snapshot['scientific_verdict']) + '</p>'
    content += '<p>Stored Forge projection. Opening this page does not replay or qualify a search.</p>'
    content += table(['Budget field', 'Recorded value'], [[k, snapshot[k]] for k in ('attempts', 'charged_ms', 'baseline_qualified')])
    content += '<p>charged_ms is non-refundable reserved stage budget in milliseconds. Observed wall cost and missing attempt costs are reported separately below.</p>'
    content += table(['Candidate', 'Parameters', 'State', 'Observed wall ms', 'Unmeasured attempts', 'Metrics'], [[c['proposal']['candidate_id'], durable.canonical(c['proposal']['parameters']), c['status'], c['observed_wall_ms'], c['unmeasured_attempts'], durable.canonical(c['metrics']) if c['metrics'] is not None else None] for c in snapshot['candidates'][after:after + 50]])
    if after + 50 < len(snapshot['candidates']): content += '<a href="/searches?search=' + esc(search) + '&amp;after=' + str(after + 50) + '">Next candidates</a>'
    content += details('Pareto candidate identifiers', snapshot['pareto_candidate_ids'])
    content += details('Active attempt and cancellation/recovery diagnostics', {'active_attempt': snapshot['active_attempt'], 'last_transition': record['last_transition']})
    content += '<h2>Recorded stage workflows</h2><ul>' + ''.join('<li><a href="/?campaign=' + esc(r['campaign']) + '">' + esc(r['attempt'][:20]) + '</a> · ' + esc(r['phase']) + '</li>' for r in record['stages']) + '</ul>'
    content += details('Complete search contract, checkpoint and source declarations', record)
    return page(content)
