import copy
import unittest

import tdi_execution_graph as graph

SHA = lambda c: c * 64
HUB = "4bf6186841e1ea70ed15cd84faf33de9b48429cd"


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
                "capability": "tdi.prepare",
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
                "capability": "tdi.evaluate",
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

    def test_hub_compiler_is_thin_and_exact(self):
        a = fixture()
        out = graph.compile_hub_workflow(
            a,
            component_bindings={"tdi-runner": "11111111-1111-1111-1111-111111111111"},
            artifact_bindings={SHA("1"): "22222222-2222-2222-2222-222222222222"},
        )
        self.assertEqual(out["schema_version"], 1)
        self.assertEqual(out["max_concurrency"], 2)
        self.assertNotIn("retry", out["steps"][0])
        self.assertEqual(
            out["steps"][0]["inputs"]["spec"],
            {"artifact": {"artifact": "22222222-2222-2222-2222-222222222222"}},
        )
        self.assertEqual(
            out["steps"][1]["inputs"]["prepared"],
            {"from_step": {"key": "prepare", "output": "file:prepared"}},
        )

    def test_hub_compiler_requires_explicit_bindings(self):
        a = fixture()
        with self.assertRaises(graph.ExecutionGraphError):
            graph.compile_hub_workflow(
                a,
                component_bindings={},
                artifact_bindings={SHA("1"): "22222222-2222-2222-2222-222222222222"},
            )
        with self.assertRaises(graph.ExecutionGraphError):
            graph.compile_hub_workflow(
                a,
                component_bindings={"tdi-runner": "11111111-1111-1111-1111-111111111111"},
                artifact_bindings={},
            )
        with self.assertRaises(graph.ExecutionGraphError):
            graph.compile_hub_workflow(
                a,
                component_bindings={"tdi-runner": "NOT-A-UUID"},
                artifact_bindings={SHA("1"): "22222222-2222-2222-2222-222222222222"},
            )


if __name__ == "__main__":
    unittest.main()
