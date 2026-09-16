#!/usr/bin/env python3
"""Public bounded softmax probes through actual pinned FLAT/optional NNIS APIs.

Only deterministic small numerical fixtures are dispatched by this Hub recipe.
It cannot select a model, dataset, scientific population or checkpoint. The
independent two-pass f64 oracle lives in the verification stage, after publication.
"""
from __future__ import annotations

import argparse
import hashlib
import ipaddress
import math
from pathlib import Path
import sys
import urllib.parse
import uuid

import tdi_execution_graph as graphs
import tdi_experiment_supervisor as durable
from tdi_engine_runtime import canonical_campaign
from tdi_engine_store import identity, atomic_json
from tdi_physical_telemetry import measured_process
from tdi_flat_partner_contract import FLAT_SOURCE_SHA
from tdi_nnis_partner_contract import NNIS_SOURCE_SHA

BACKENDS = ('flat-reference', 'nnis-cuda-fused')
MODULES = ('tdi_attention_fixture', 'tdi_experiment_supervisor', 'tdi_execution_graph', 'tdi_engine_store',
           'tdi_engine_runtime', 'tdi_physical_telemetry', 'tdi_flat_partner_contract', 'tdi_nnis_partner_contract',
           'tdi_partner_adapter_contract', 'tdi_experiment_contract', 'tdi_artifact_contract',
           'tdi_hub_client', 'tdi_hub_admission_contract', 'tdi_hub_edge_contract')


def pin(path, expected=None):
    """Hash an explicit immutable regular deployment file, refusing symlinks."""
    path = Path(path)
    if path.is_symlink() or not path.is_file(): raise durable.ContractError('regular attention deployment file required')
    path = path.resolve(strict=True); digest = durable.file_digest(path)
    if expected is not None and digest != expected: raise durable.ContractError('attention deployment changed')
    return {'path': str(path), 'sha256': digest}


def fixture_request(index, backend='flat-reference', domain='Development'):
    """Return one of six explicit public small tensor fixtures; no learned data.

    Binary-exact fractional values keep JSON transport distinct from precision
    conversion. NNIS case 3 uses its supported square causal API; FLAT case 3
    exercises an offset single-query MQA control. These are different contracts.
    """
    if type(index) is not int or not 0 <= index < 6 or backend not in BACKENDS or domain not in ('Development', 'Validation'):
        raise durable.ContractError('unknown non-final attention fixture')
    shapes = [(1, 1, 1, 4, 4, 4, 0, 'none'), (1, 1, 1, 4, 4, 8, 0, 'causal'),
              (2, 4, 2, 3, 5, 8, 0, 'none'), (1, 4, 1, 1, 5, 8, 4, 'causal'),
              (1, 2, 2, 7, 7, 16, 0, 'causal'), (1, 2, 1, 2, 9, 32, 0, 'none')]
    if backend == 'nnis-cuda-fused': shapes[3] = (1, 4, 1, 5, 5, 8, 0, 'causal')
    b, qh, kh, qn, kn, d, offset, mask = shapes[index]
    return {'schema': 1, 'purpose': 'public-kernel-probe', 'domain': domain, 'backend': backend,
            'mechanism': 'standard-softmax', 'dtype': 'f32', 'layout': 'BHND', 'mask': mask,
            'shape': dict(zip(('batch', 'q_heads', 'kv_heads', 'query_len', 'kv_len', 'head_dim', 'query_position_offset'),
                             (b, qh, kh, qn, kn, d, offset))), 'scale': 0.25,
            'q': [((i * 7 + index) % 17 - 8) / 8 for i in range(b * qh * qn * d)],
            'k': [((i * 11 + index) % 19 - 9) / 8 for i in range(b * kh * kn * d)],
            'v': [((i * 3 + index) % 13 - 6) / 16 for i in range(b * kh * kn * d)]}


def independent_oracle(request):
    """Two-pass f64 softmax oracle for the closed public fixture inventory.

    This verification-only path explicitly stores a small row of scores. It is
    neither FLAT's online update implementation nor an NNIS execution fallback.
    Returns output and log-sum-exp in BHND/BHN order, without a timing claim.
    """
    s = request['shape']; d = s['head_dim']; output, lse = [], []
    q, k, v = request['q'], request['k'], request['v']
    for b in range(s['batch']):
        for h in range(s['q_heads']):
            kh = h // (s['q_heads'] // s['kv_heads'])
            kb = (b * s['kv_heads'] + kh) * s['kv_len'] * d
            for row in range(s['query_len']):
                qb = ((b * s['q_heads'] + h) * s['query_len'] + row) * d
                n = min(s['kv_len'], s['query_position_offset'] + row + 1) if request['mask'] == 'causal' else s['kv_len']
                scores = [math.fsum(q[qb + j] * k[kb + x * d + j] for j in range(d)) * request['scale'] for x in range(n)]
                largest = max(scores); weights = [math.exp(x - largest) for x in scores]; total = math.fsum(weights)
                lse.append(largest + math.log(total))
                output += [math.fsum(weights[x] * v[kb + x * d + j] for x in range(n)) / total for j in range(d)]
    return output, lse


def check_observation(request, observed):
    """Validate source/semantics and independently compare each observed scalar.

    Predeclared software tolerances: FLAT abs/rel 3e-6; NNIS abs 1e-3 and rel
    1e-4. These fixture tolerances establish neither model quality nor general
    numerical equivalence. NNIS LSE is unavailable and must remain null.
    """
    expected_source = FLAT_SOURCE_SHA if request['backend'] == 'flat-reference' else NNIS_SOURCE_SHA
    contract = observed['contract']
    if (observed.get('schema') != 1 or observed.get('status') != 'observed'
            or observed.get('scientific_verdict') != 'not-assessed' or contract['source_commit'] != expected_source
            or any(contract[k] != request[k] for k in ('schema', 'domain', 'backend', 'mechanism', 'dtype', 'layout', 'shape', 'mask', 'scale'))
            or contract['model_execution'] is not False or contract['production_routing_authorized'] is not False
            or contract['performance_claim_authorized'] is not False):
        raise durable.ContractError('attention response source/contract mismatch')
    expected, expected_lse = independent_oracle(request)
    absolute, relative = (3e-6, 3e-6) if request['backend'] == 'flat-reference' else (1e-3, 1e-4)
    def compare(actual, wanted):
        if (not isinstance(actual, list) or len(actual) != len(wanted)
                or any(type(x) not in (int, float) or not math.isfinite(x) or not math.isclose(x, y, abs_tol=absolute, rel_tol=relative)
                       for x, y in zip(actual, wanted))):
            raise durable.ContractError('independent attention oracle rejected output')
        return max(abs(x - y) for x, y in zip(actual, wanted))
    error = compare(observed['output'], expected)
    if request['backend'] == 'flat-reference':
        lse_error = compare(observed['lse'], expected_lse)
        if observed['device'] is not None: raise durable.ContractError('CPU reference cannot claim a GPU')
    else:
        lse_error = None
        if observed['lse'] is not None or not isinstance(observed['device'], dict): raise durable.ContractError('NNIS output availability mismatch')
        if any(not observed['device'].get(k) for k in ('name', 'uuid', 'compute_capability', 'driver_version', 'nvrtc_version')):
            raise durable.ContractError('NNIS device identity incomplete')
    return {'oracle': 'public-small-softmax-two-pass-f64/v1', 'max_absolute_output_error': error,
            'max_absolute_lse_error': lse_error, 'absolute_tolerance': absolute, 'relative_tolerance': relative,
            'scientific_verdict': 'not-assessed', 'performance_qualified': False}


def call_probe(worker, request, *, run=False, allow_cuda=False):
    """Invoke a byte-pinned trusted process, retaining actual costs on rejection.

    The 30 s wall budget includes process setup/JIT if CUDA is explicitly selected.
    wait4 RSS is host process memory; it is never labeled VRAM. GPU timing/energy
    remain unavailable. A failed attempt is data for the independent verifier.
    """
    pin(worker['path'], worker['sha256'])
    command = [worker['path'], 'run' if run else 'validate']
    if allow_cuda: command.append('--allow-cuda')
    code, raw, diagnostic, costs = measured_process(command, input_bytes=durable.canonical(request).encode(), timeout=30, max_output=1024 * 1024)
    pin(worker['path'], worker['sha256'])
    response = None
    try: response = durable.strict_json(raw, max_bytes=1024 * 1024, max_items=100000)
    except ValueError: pass
    return {'return_code': code, 'response': response, 'cost_measurements': costs,
            'stdout_sha256': hashlib.sha256(raw).hexdigest(), 'stderr_sha256': hashlib.sha256(diagnostic).hexdigest()}


def prepare_attention_fixture(client, worker, *, backend='flat-reference', domain='Development', trials=6, allow_cuda=False):
    """Register run→verify stages over a closed public inventory, without running kernels.

    NNIS requires explicit CUDA opt-in in the immutable plan. Software-only
    callers can validate its requests using the standalone probe without a GPU.
    The local trusted Hub owns all execution and publication, as for other recipes.
    """
    if (backend not in BACKENDS or domain not in ('Development', 'Validation')
            or type(trials) is not int or not 1 <= trials <= 6 or type(allow_cuda) is not bool
            or allow_cuda != (backend == 'nnis-cuda-fused')):
        raise durable.ContractError('invalid public attention fixture/CUDA opt-in')
    if not ipaddress.ip_address(urllib.parse.urlsplit(client.endpoint).hostname).is_loopback:
        raise durable.ContractError('attention fixture requires a local trusted Hub')
    files = {name: pin(Path(__file__).with_name(name + '.py')) for name in MODULES}
    files['worker'] = pin(worker); files['python'] = pin(Path(sys.executable).resolve())
    config = {'schema': 1, 'purpose': 'development-software', 'backend': backend, 'domain': domain,
              'trials': trials, 'allow_cuda': allow_cuda, 'files': files}
    plan = identity('tdi-public-attention-fixture/v1', config)
    components = {}
    for mode in ('run', 'verify'):
        component_id = str(uuid.uuid5(uuid.NAMESPACE_URL, plan + mode))
        args = [files['tdi_attention_fixture']['path'], '--mode', mode, '--parameters', '{params}', '--output', '{output:result}']
        if mode == 'verify': args += ['--input', '{input:result}']
        manifest = {'id': component_id, 'name': 'tdi-public-attention-' + mode, 'version': '1.0.0', 'kind': 'tool',
                    'capabilities': [{'name': 'tdi.attention.' + mode, 'contract_version': '1.0.0',
                                      'inputs': [] if mode == 'run' else [{'name': 'result'}], 'outputs': [{'name': 'result'}]}],
                    'execution': {'type': 'process', 'program': files['python']['path'], 'args': args,
                                  'outputs': [{'name': 'result', 'path': 'result.json', 'media_type': 'application/json', 'required': True}]},
                    'metadata': {'tdi.plan': plan, 'tdi.scope': 'public-small-operator-software-probe'}}
        components[mode] = client.request('POST', '/api/v1/components', value={'schema_version': 1, 'manifest': manifest})['component']
        if components[mode]['id'] != component_id: raise durable.ContractError('attention component identity mismatch')
    steps, allowed, outputs = [], {}, {}
    for index in range(trials):
        for mode in ('run', 'verify'):
            component = components[mode]; key = f'{mode}-{index}'
            contract = {'component_id': component['id'], 'component_version': component['version'],
                        'component_manifest_digest': component['manifest_digest'], 'capability': 'tdi.attention.' + mode,
                        'capability_contract_version': '1.0.0'}
            steps.append(dict(contract, key=key, component_alias='attention-' + mode,
                parameters={'configuration': config, 'index': index, 'plan_id': plan},
                inputs={} if mode == 'run' else {'result': {'kind': 'step', 'step': f'run-{index}', 'output': 'file:result'}},
                outputs=['file:result'], after=[] if mode == 'run' else [f'run-{index}'], timeout_milliseconds=40000,
                checkpoint={'mode': 'none', 'input': None, 'output': None}))
            allowed[key] = contract
            outputs[key] = {'file:result': {'media_type': 'application/json', 'access_class': domain.lower(),
                'json_fields': {'status': 'Evaluated' if mode == 'run' else 'Verified', 'plan_id': plan,
                                'backend': backend, 'index': index}, 'cache': 'disabled'}}
    graph = {'schema': 1, 'semantic_version': 'tdi-graph/1.0.0', 'name': 'public-attention-' + plan[:24],
             'root_plan_id': plan, 'max_concurrency': 1, 'steps': steps,
             'hub_contract': {'repository': 'Memorithm/scirust-hub', 'source_commit': graphs.HUB_SOURCE_COMMIT,
                              'workflow_schema_version': 1, 'workflow_model_version': '1.2.0'}}
    return canonical_campaign({'schema': 1, 'purpose': 'development-software', 'domain': domain, 'graph': graph,
                               'policy': {'trust': 'trusted-software', 'allowed_steps': allowed}, 'outputs': outputs})


def parameters(raw):
    """Validate all immutable fixture fields and every deployed support-module hash."""
    p = durable.strict_json(raw, max_bytes=32768); c = p['configuration']
    if (set(p) != {'configuration', 'index', 'plan_id'} or set(c) != {'schema', 'purpose', 'backend', 'domain', 'trials', 'allow_cuda', 'files'}
            or type(c['schema']) is not int or c['schema'] != 1 or c['purpose'] != 'development-software'
            or type(c['trials']) is not int or not 1 <= c['trials'] <= 6 or type(p['index']) is not int or not 0 <= p['index'] < c['trials']
            or c['backend'] not in BACKENDS or c['domain'] not in ('Development', 'Validation')
            or type(c['allow_cuda']) is not bool or c['allow_cuda'] != (c['backend'] == 'nnis-cuda-fused')
            or set(c['files']) != {*MODULES, 'worker', 'python'}
            or p['plan_id'] != identity('tdi-public-attention-fixture/v1', c)):
        raise durable.ContractError('invalid attention fixture identity/configuration')
    for name, record in c['files'].items():
        if set(record) != {'path', 'sha256'}: raise durable.ContractError('invalid deployment pin')
        pin(record['path'], record['sha256'])
        if name in MODULES and Path(record['path']) != Path(__file__).with_name(name + '.py').resolve():
            raise durable.ContractError('attention support module location mismatch')
    return p, c


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--mode', choices=('run', 'verify'), required=True)
    parser.add_argument('--parameters', required=True); parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--input', type=Path); args = parser.parse_args()
    try:
        p, config = parameters(args.parameters)
        request = fixture_request(p['index'], config['backend'], config['domain'])
        base = {'schema': 1, 'plan_id': p['plan_id'], 'backend': config['backend'], 'index': p['index']}
        if args.mode == 'run':
            value = dict(base, status='Evaluated', request=request,
                         execution=call_probe(config['files']['worker'], request, run=True, allow_cuda=config['allow_cuda']))
        else:
            if args.input is None or args.input.is_symlink() or not args.input.is_file(): raise durable.ContractError('explicit attention result input required')
            with args.input.open('rb') as stream: observed = durable.strict_json(stream.read(1048577), max_bytes=1048576)
            if any(observed[k] != v for k, v in base.items()) or observed['request'] != request or observed['status'] != 'Evaluated':
                raise durable.ContractError('attention stage identity mismatch')
            execution = observed['execution']
            if execution['return_code'] != 0 or execution['cost_measurements']['technical_failure']:
                raise durable.ContractError('attention process failed; measured attempt retained in run output')
            value = dict(base, status='Verified', evidence=check_observation(request, execution['response']),
                         input_sha256=durable.file_digest(args.input))
        parameters(args.parameters)  # deployment must also remain unchanged after the work
        atomic_json(args.output, value); return 0
    except (ValueError, TypeError, KeyError, OSError) as error:
        print(durable.canonical({'schema': 1, 'status': 'attention-fixture-error', 'error': str(error)}), file=sys.stderr)
        return durable.EXIT_CONTRACT


if __name__ == '__main__': raise SystemExit(main())
