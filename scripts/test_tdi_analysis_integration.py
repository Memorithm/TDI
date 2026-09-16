"""A real Hub campaign feeds selected, verified outputs to the SciRust process."""
import copy
import json
import os
from pathlib import Path
import unittest

import tdi_engine_runtime as runtime
import tdi_experiment_supervisor as durable
from tdi_engine_store import EngineStore, atomic_json
from tdi_hub_fixture import prepare_fixture
from tdi_research_analysis import observations_from_catalogue
import test_tdi_engine_integration as hub_fixture
from test_tdi_research_analysis import fixture


class RealAnalysisIntegration(unittest.TestCase):
    def test_catalogue_selection_cli_report_and_reused_source_rejection(self):
        hub_fixture.OperationalIntegrationTests.setUpClass()
        hub = hub_fixture.OperationalIntegrationTests()
        self.addCleanup(hub.doCleanups)
        hub.setUp()
        protocol, _ = fixture()
        protocol["units"] = [{"id": "u0", "stratum": "small", "replicates": ["r0"]}, {"id": "u1", "stratum": "small", "replicates": ["r0"]}]
        protocol["missing"] = "reject-incomplete"
        spec = prepare_fixture(hub.client, hub.worker, trials=4)
        with EngineStore(hub.catalogue) as store:
            campaign = runtime.submit(hub.client, store, spec, {})
            runtime.execute(hub.client, store, campaign)
            selections = [{"unit": f"u{i // 2}", "replicate": "r0", "arm": "reference" if i % 2 == 0 else "candidate",
                           "metric": "score", "campaign": campaign, "step": f"run-{i}", "output": "file:result", "pointer": "/scores/0"} for i in range(4)]
            selected = observations_from_catalogue(store, protocol, selections)
            self.assertEqual([2, 2, 2, 2], [r["value"] for r in selected["rows"]])
            bad = copy.deepcopy(selections); bad[1]["step"] = bad[0]["step"]
            with self.assertRaises(durable.ContractError): observations_from_catalogue(store, protocol, bad)
            wrong = dict(protocol, domain="Validation")
            with self.assertRaises(durable.ContractError): observations_from_catalogue(store, wrong, selections)
        protocol_path, selections_path, report_path = (hub.root / x for x in ("protocol.json", "selections.json", "report.json"))
        atomic_json(protocol_path, protocol); atomic_json(selections_path, selections)
        binary = Path(os.environ["TDI_SCIRUST_STATS_BIN"]).resolve(strict=True)
        result = hub.cli("analyze", "--protocol", protocol_path, "--selections", selections_path, "--worker", binary,
                         "--worker-sha256", durable.file_digest(binary), "--source-commit", os.environ["TDI_SCIRUST_SOURCE_COMMIT"], "--output", report_path)
        self.assertEqual("not-assessed", result["scientific_verdict"])
        report = json.loads(report_path.read_text())
        self.assertEqual(0, report["results"][0]["interval"]["estimate"])
        self.assertEqual(campaign, report["observations"]["rows"][0]["source"]["campaign"])
        # Atomic publication refuses to overwrite the original report.
        hub.cli("analyze", "--protocol", protocol_path, "--selections", selections_path, "--worker", binary,
                "--worker-sha256", durable.file_digest(binary), "--source-commit", os.environ["TDI_SCIRUST_SOURCE_COMMIT"], "--output", report_path, expected_code=22)


if __name__ == "__main__": unittest.main()
