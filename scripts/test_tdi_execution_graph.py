import copy
import unittest

import tdi_execution_graph as graph

SHA = lambda c: c * 64
HUB = "4bf6186841e1ea70ed15cd84faf33de9b48429cd"
COMPONENT_ID = "11111111-1111-1111-1111-111111111111"
ARTIFACT_ID = "22222222-2222-2222-2222-222222222222"


def fixture():
    return {
        "schema": 1,
        "semantic_version": "tdi-graph/1.0.0",
        "name": "fixture-graph",
        "root_plan_id": SHA("a"),
        "hub_contract": {
            "repository": "Memorithm/scirust-hub",
            "source_commit": HUB,
            "workflow_schema_version": 1,
            "workflow_model_version": "1.2.0",
        },
        "max_concurrency": 2,
        "steps": [
            {
                "key": "prepare",
                "component_alias": "tdi-runner",
                "component_id": COMPONENT_ID,
                "component_version": "1.2.3",
                "component_manifest_digest": SHA("9"),
                "capability": "tdi.prepare",
                "capability_contract_version": "1.0.0",
                "parameters": {"mode": "fixture", "count": 2},
                "inputs": {"spec": {"kind": "artifact", "sha256": SHA("1")}},
                "outputs": ["file:prepared", "file:checkpoint"],
                "after": [],
                "timeout_milliseconds": 1000,
                "checkpoint": {"mode": "exact", "input": None, "output": "file:checkpoint"},
            },
            {
                "key": "evaluate",
                "component_alias": "tdi-runner",
                "component_id": COMPONENT_ID,
                "component_version": "1.2.3",
                "component_manifest_digest": SHA("9"),
                "capability": "tdi.evaluate",
                "capability_contract_version": "1.1.0",
                "parameters": {"mode": "fixture"},
                "inputs": {
                    "prepared": {"kind": "step", "step": "prepare", "output": "file:prepared"},
                    "checkpoint": {"kind": "step", "step": "prepare", "output": "file:checkpoint"},
                },
                "outputs": ["file:result"],
                "after": ["prepare"],
                "timeout_milliseconds": 2000,
                "checkpoint": {"mode": "none", "input": None, "output": None},
            },
        ],
    }


class Tests(unittest.TestCase):
    def test_graph_identity_and_step_identity_are_stable(self):
        a = fixture()
        b = copy.deepcopy(a)
        b["steps"][0]["outputs"].reverse()
        self.assertEqual(graph.graph_identity(a), graph.graph_identity(b))
        self.assertEqual(graph.step_identity(a, "prepare"), graph.step_identity(b, "prepare"))
        self.assertNotEqual(graph.step_identity(a, "prepare"), graph.step_identity(a, "evaluate"))

    def test_forward_dependency_and_unknown_output_fail_closed(self):
        a = fixture()
        a["steps"][0]["after"] = ["evaluate"]
        with self.assertRaises(graph.ExecutionGraphError):
            graph.canonical_graph(a)
        a = fixture()
        a["steps"][1]["inputs"]["prepared"]["output"] = "file:missing"
        with self.assertRaises(graph.ExecutionGraphError):
            graph.canonical_graph(a)

    def test_checkpoint_input_output_contract(self):
        a = fixture()
        a["steps"][0]["checkpoint"]["output"] = "file:missing"
        with self.assertRaises(graph.ExecutionGraphError):
            graph.canonical_graph(a)
        a = fixture()
        a["steps"][1]["checkpoint"] = {"mode": "exact", "input": "missing", "output": "file:result"}
        with self.assertRaises(graph.ExecutionGraphError):
            graph.canonical_graph(a)

    def test_parameters_reject_floats_and_unsafe_integers(self):
        a = fixture()
        a["steps"][0]["parameters"]["x"] = 0.5
        with self.assertRaises(graph.ExecutionGraphError):
            graph.canonical_graph(a)
        a = fixture()
        a["steps"][0]["parameters"]["x"] = 2**53
        with self.assertRaises(graph.ExecutionGraphError):
            graph.canonical_graph(a)

    def test_hub_source_contract_is_exactly_pinned(self):
        a = fixture()
        a["hub_contract"]["source_commit"] = "0" * 40
        with self.assertRaises(graph.ExecutionGraphError):
            graph.canonical_graph(a)

    def test_capability_must_match_hub_grammar(self):
        a = fixture()
        a["steps"][0]["capability"] = "tdi.prepare@1.0.0"
        with self.assertRaises(graph.ExecutionGraphError):
            graph.canonical_graph(a)

    def test_version_parser_matches_pinned_hub_prerelease_shape(self):
        # Pinned Hub Version::parse accepts any non-empty ASCII prerelease made
        # only of alphanumeric, dot and hyphen characters; it is deliberately
        # semver-shaped rather than a full SemVer parser.
        for version in ("1.2.3-.", "1.2.3-alpha..1", "1.2.3-01"):
            a = fixture()
            a["steps"][0]["component_version"] = version
            a["steps"][1]["component_version"] = version
            graph.canonical_graph(a)

    def test_component_pins_change_step_identity(self):
        a = fixture()
        base = graph.step_identity(a, "prepare")
        for field, value in (
            ("component_id", "33333333-3333-3333-3333-333333333333"),
            ("component_version", "1.2.4"),
            ("component_manifest_digest", SHA("8")),
            ("capability_contract_version", "1.0.1"),
        ):
            changed = copy.deepcopy(a)
            changed["steps"][0][field] = value
            if field in ("component_id", "component_version", "component_manifest_digest"):
                changed["steps"][1][field] = value
            self.assertNotEqual(base, graph.step_identity(changed, "prepare"), field)

    def test_one_alias_cannot_resolve_to_multiple_component_pins(self):
        a = fixture()
        a["steps"][1]["component_version"] = "1.2.4"
        with self.assertRaises(graph.ExecutionGraphError):
            graph.canonical_graph(a)

    def test_pinned_hub_run_limits_fail_closed(self):
        a = fixture()
        a["steps"][0]["timeout_milliseconds"] = graph.MAX_TIMEOUT_MS + 1
        with self.assertRaises(graph.ExecutionGraphError):
            graph.canonical_graph(a)

        a = fixture()
        a["steps"][0]["parameters"] = {"payload": "x" * graph.MAX_PARAMS_BYTES}
        with self.assertRaises(graph.ExecutionGraphError):
            graph.canonical_graph(a)

        a = fixture()
        a["steps"][0]["inputs"] = {
            f"i{i}": {"kind": "artifact", "sha256": SHA("1")}
            for i in range(graph.MAX_INPUTS + 1)
        }
        with self.assertRaises(graph.ExecutionGraphError):
            graph.canonical_graph(a)

        a = fixture()
        a["steps"][0]["inputs"] = {"bad.name": {"kind": "artifact", "sha256": SHA("1")}}
        with self.assertRaises(graph.ExecutionGraphError):
            graph.canonical_graph(a)

    def test_output_labels_reject_unicode_control_characters(self):
        a = fixture()
        a["steps"][0]["outputs"] = ["file:bad\u0080label"]
        a["steps"][0]["checkpoint"]["output"] = "file:bad\u0080label"
        with self.assertRaises(graph.ExecutionGraphError):
            graph.canonical_graph(a)

    def test_hub_compiler_is_structural_and_non_authoritative(self):
        a = fixture()
        out = graph.compile_hub_workflow_preview(
            a,
            artifact_bindings={SHA("1"): ARTIFACT_ID},
        )
        self.assertFalse(out["execution_authorized"])
        self.assertEqual(out["workflow"]["schema_version"], 1)
        self.assertEqual(out["workflow"]["max_concurrency"], 2)
        self.assertNotIn("retry", out["workflow"]["steps"][0])
        self.assertEqual(out["workflow"]["steps"][0]["component"], COMPONENT_ID)
        self.assertEqual(
            out["workflow"]["steps"][0]["inputs"]["spec"],
            {"artifact": {"artifact": ARTIFACT_ID}},
        )
        self.assertEqual(
            out["workflow"]["steps"][1]["inputs"]["prepared"],
            {"from_step": {"key": "prepare", "output": "file:prepared"}},
        )
        self.assertEqual(out["component_pins"][0]["component_version"], "1.2.3")
        self.assertEqual(out["capability_pins"][0]["contract_version"], "1.0.0")

    def test_hub_compiler_requires_artifact_bindings_and_canonical_ids(self):
        a = fixture()
        with self.assertRaises(graph.ExecutionGraphError):
            graph.compile_hub_workflow_preview(a, artifact_bindings={})
        a = fixture()
        a["steps"][0]["component_id"] = "not-a-uuid"
        with self.assertRaises(graph.ExecutionGraphError):
            graph.canonical_graph(a)


if __name__ == "__main__":
    unittest.main()
