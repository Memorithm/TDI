import copy
import hashlib
import unittest

import tdi_execution_graph as graph
import tdi_hub_admission_contract as admission
import tdi_hub_edge_contract as hub

SHA = lambda c: c * 64
WORKFLOW = "123e4567-e89b-42d3-a456-426614174100"
ATTEMPT = "123e4567-e89b-42d3-a456-426614174200"
ARTIFACT = "123e4567-e89b-42d3-a456-426614174000"
COMPONENT = "11111111-1111-1111-1111-111111111111"
INPUT_ARTIFACT = "22222222-2222-2222-2222-222222222222"


def fixture_graph():
    return {
        "schema": 1,
        "semantic_version": "tdi-graph/1.0.0",
        "name": "admission-fixture",
        "root_plan_id": SHA("a"),
        "hub_contract": {
            "repository": "Memorithm/scirust-hub",
            "source_commit": graph.HUB_SOURCE_COMMIT,
            "workflow_schema_version": 1,
            "workflow_model_version": "1.2.0",
        },
        "max_concurrency": 2,
        "steps": [
            {
                "key": "prepare",
                "component_alias": "tdi-runner",
                "component_id": COMPONENT,
                "component_version": "1.2.3",
                "component_manifest_digest": SHA("9"),
                "capability": "tdi.prepare",
                "capability_contract_version": "1.0.0",
                "parameters": {"mode": "fixture"},
                "inputs": {"spec": {"kind": "artifact", "sha256": SHA("1")}},
                "outputs": ["file:prepared"],
                "after": [],
                "timeout_milliseconds": 1000,
                "checkpoint": {"mode": "none", "input": None, "output": None},
            },
            {
                "key": "evaluate",
                "component_alias": "tdi-runner",
                "component_id": COMPONENT,
                "component_version": "1.2.3",
                "component_manifest_digest": SHA("9"),
                "capability": "tdi.evaluate",
                "capability_contract_version": "1.1.0",
                "parameters": {"mode": "fixture"},
                "inputs": {
                    "prepared": {"kind": "step", "step": "prepare", "output": "file:prepared"}
                },
                "outputs": ["file:result"],
                "after": ["prepare"],
                "timeout_milliseconds": 2000,
                "checkpoint": {"mode": "none", "input": None, "output": None},
            },
        ],
    }


def _portable_root_binding(digest=SHA("1"), artifact_id=INPUT_ARTIFACT):
    descriptor = {
        "schema": 1,
        "name": "root-spec",
        "raw_sha256": digest,
        "size_bytes": 123,
        "media_type": "application/octet-stream",
        "access_class": "development",
    }
    response = {
        "id": artifact_id,
        "hub_digest": "c" * 64,
        "raw_sha256": digest,
        "size": 123,
    }
    return hub.bind_portable_artifact(descriptor, response)


def root_artifact_bindings():
    return {SHA("1"): _portable_root_binding()}


def artifact_ids():
    return {SHA("1"): INPUT_ARTIFACT}


def workflow_response(value=None):
    value = value or fixture_graph()
    preview = graph.compile_hub_workflow_preview(value, artifact_bindings=artifact_ids())
    canonical = graph.canonical_graph(value)
    pins = {
        step["key"]: {
            "component_version": step["component_version"],
            "manifest_digest": step["component_manifest_digest"],
            "capability_contract_version": step["capability_contract_version"],
        }
        for step in canonical["steps"]
    }
    return {
        "id": WORKFLOW,
        "name": canonical["name"],
        "spec": preview["workflow"],
        "state": "created",
        "admission": {"schema_version": 1, "steps": pins},
        "model_version": "1.2.0",
        "created_at": 1,
    }


def bind_fixture():
    return admission.bind_exact_workflow_admission(
        fixture_graph(),
        workflow_response(),
        root_artifact_bindings=root_artifact_bindings(),
    )


def authoritative_artifact_binding(*, workflow=WORKFLOW, step_key="evaluate"):
    payload = b"result"
    descriptor = {
        "schema": 1,
        "name": "result",
        "raw_sha256": hashlib.sha256(payload).hexdigest(),
        "size_bytes": len(payload),
        "media_type": "application/octet-stream",
        "access_class": "development",
    }
    response = {
        "id": ARTIFACT,
        "hub_digest": "b" * 64,
        "raw_sha256": descriptor["raw_sha256"],
        "size": descriptor["size_bytes"],
    }
    portable = hub.bind_portable_artifact(descriptor, response)
    publication = {
        "schema_version": 1,
        "workflow": workflow,
        "step_key": step_key,
        "attempt": ATTEMPT,
        "generation": 1,
        "outputs": {"file:result": ARTIFACT},
    }
    return hub.bind_authoritative_publication(
        portable,
        publication,
        expected_workflow=workflow,
        expected_step_key=step_key,
        expected_output="file:result",
    )


class HubAdmissionContractTests(unittest.TestCase):
    def test_exact_graph_and_hub_admission_are_execution_authorized_only(self):
        value = fixture_graph()
        binding = bind_fixture()
        self.assertEqual(admission.PINNED_HUB_EXACT_ADMISSION_SOURCE, binding["hub_source_sha"])
        self.assertEqual(graph.graph_identity(value), binding["graph_identity"])
        self.assertEqual(["evaluate", "prepare"], binding["admitted_steps"])
        self.assertEqual(INPUT_ARTIFACT, binding["root_artifact_bindings"][SHA("1")]["hub_artifact_id"])
        self.assertTrue(binding["execution_authorized"])
        self.assertFalse(binding["scientific_stage_authorized"])
        self.assertEqual(binding, admission.canonical_workflow_admission_binding(binding))
        admission.workflow_admission_binding_identity(binding)

    def test_root_artifact_binding_proof_is_required_and_digest_bound(self):
        with self.assertRaisesRegex(admission.HubAdmissionContractError, "coverage mismatch"):
            admission.bind_exact_workflow_admission(
                fixture_graph(), workflow_response(), root_artifact_bindings={}
            )

        wrong = {SHA("1"): _portable_root_binding(digest=SHA("2"))}
        with self.assertRaisesRegex(admission.HubAdmissionContractError, "does not match Graph/v1"):
            admission.bind_exact_workflow_admission(
                fixture_graph(), workflow_response(), root_artifact_bindings=wrong
            )

    def test_full_workflow_spec_must_equal_tdi_compilation_with_json_type_fidelity(self):
        for mutate in (
            lambda response: response["spec"]["steps"][0].__setitem__(
                "component", "33333333-3333-3333-3333-333333333333"
            ),
            lambda response: response["spec"]["steps"][0].__setitem__("capability", "tdi.other"),
            lambda response: response["spec"]["steps"][0]["parameters"].__setitem__("mode", "drift"),
            lambda response: response["spec"]["steps"][1].__setitem__("timeout_ms", 1999),
            lambda response: response["spec"].__setitem__("max_concurrency", 2.0),
            lambda response: response["spec"]["steps"][0].__setitem__("timeout_ms", True),
        ):
            response = workflow_response()
            mutate(response)
            with self.assertRaisesRegex(admission.HubAdmissionContractError, "spec does not exactly match"):
                admission.bind_exact_workflow_admission(
                    fixture_graph(), response, root_artifact_bindings=root_artifact_bindings()
                )

    def test_g3_identity_version_fields_reject_boolean_aliases(self):
        response = workflow_response()
        response["admission"]["schema_version"] = True
        with self.assertRaisesRegex(
            admission.HubAdmissionContractError, "workflow admission schema version"
        ):
            admission.bind_exact_workflow_admission(
                fixture_graph(), response, root_artifact_bindings=root_artifact_bindings()
            )

        binding = bind_fixture()
        binding["schema"] = True
        with self.assertRaisesRegex(
            admission.HubAdmissionContractError, "workflow admission binding schema"
        ):
            admission.canonical_workflow_admission_binding(binding)

        binding = bind_fixture()
        binding["admission_schema_version"] = True
        with self.assertRaisesRegex(
            admission.HubAdmissionContractError, "workflow admission schema version mismatch"
        ):
            admission.canonical_workflow_admission_binding(binding)

    def test_each_registry_pin_must_match_graph(self):
        fields = (
            ("component_version", "1.2.4"),
            ("manifest_digest", SHA("8")),
            ("capability_contract_version", "1.0.1"),
        )
        for field, changed in fields:
            response = workflow_response()
            response["admission"]["steps"]["prepare"][field] = changed
            with self.subTest(field=field), self.assertRaisesRegex(
                admission.HubAdmissionContractError, "does not match TDI Graph/v1"
            ):
                admission.bind_exact_workflow_admission(
                    fixture_graph(), response, root_artifact_bindings=root_artifact_bindings()
                )

    def test_admission_step_coverage_is_exact(self):
        response = workflow_response()
        del response["admission"]["steps"]["evaluate"]
        with self.assertRaisesRegex(admission.HubAdmissionContractError, "coverage mismatch"):
            admission.bind_exact_workflow_admission(
                fixture_graph(), response, root_artifact_bindings=root_artifact_bindings()
            )

        response = workflow_response()
        response["admission"]["steps"]["extra"] = copy.deepcopy(
            response["admission"]["steps"]["prepare"]
        )
        with self.assertRaisesRegex(admission.HubAdmissionContractError, "coverage mismatch"):
            admission.bind_exact_workflow_admission(
                fixture_graph(), response, root_artifact_bindings=root_artifact_bindings()
            )

    def test_legacy_or_unpinned_workflow_fails_closed(self):
        response = workflow_response()
        response["admission"] = None
        with self.assertRaisesRegex(admission.HubAdmissionContractError, "exact admission pins"):
            admission.bind_exact_workflow_admission(
                fixture_graph(), response, root_artifact_bindings=root_artifact_bindings()
            )

        response = workflow_response()
        del response["spec"]
        with self.assertRaisesRegex(admission.HubAdmissionContractError, "missing"):
            admission.bind_exact_workflow_admission(
                fixture_graph(), response, root_artifact_bindings=root_artifact_bindings()
            )

    def test_restored_binding_recomputes_embedded_evidence_identities(self):
        binding = bind_fixture()
        changed = copy.deepcopy(binding)
        changed["graph_identity"] = "0" * 64
        with self.assertRaisesRegex(admission.HubAdmissionContractError, "does not match embedded"):
            admission.canonical_workflow_admission_binding(changed)

        changed = copy.deepcopy(binding)
        changed["workflow_spec"]["max_concurrency"] = 3
        with self.assertRaisesRegex(admission.HubAdmissionContractError, "embedded Hub workflow spec"):
            admission.canonical_workflow_admission_binding(changed)

        changed = copy.deepcopy(binding)
        changed["admission"]["steps"]["prepare"]["manifest_digest"] = SHA("8")
        with self.assertRaisesRegex(admission.HubAdmissionContractError, "does not match TDI Graph/v1"):
            admission.canonical_workflow_admission_binding(changed)

    def test_binding_source_and_science_authority_are_fail_closed(self):
        binding = bind_fixture()
        changed = copy.deepcopy(binding)
        changed["hub_source_sha"] = "0" * 40
        with self.assertRaisesRegex(admission.HubAdmissionContractError, "not the qualified"):
            admission.canonical_workflow_admission_binding(changed)

        changed = copy.deepcopy(binding)
        changed["scientific_stage_authorized"] = True
        with self.assertRaisesRegex(admission.HubAdmissionContractError, "never authorizes"):
            admission.canonical_workflow_admission_binding(changed)

    def test_authoritative_artifact_can_be_combined_only_with_same_admitted_workflow_step(self):
        workflow_binding = bind_fixture()
        artifact_binding = authoritative_artifact_binding()
        combined = admission.bind_execution_authorized_artifact(artifact_binding, workflow_binding)
        self.assertEqual(3, combined["schema"])
        self.assertTrue(combined["execution_authorized"])
        self.assertTrue(combined["publication_authoritative"])
        self.assertFalse(combined["scientific_stage_authorized"])
        self.assertEqual(
            combined,
            admission.canonical_execution_authorized_artifact_binding(combined),
        )
        admission.execution_authorized_artifact_binding_identity(combined)

    def test_combined_binding_rejects_wrong_workflow_or_unadmitted_step(self):
        workflow_binding = bind_fixture()
        wrong_workflow = authoritative_artifact_binding(
            workflow="123e4567-e89b-42d3-a456-426614174999"
        )
        with self.assertRaisesRegex(admission.HubAdmissionContractError, "different workflows"):
            admission.bind_execution_authorized_artifact(wrong_workflow, workflow_binding)

        unadmitted = authoritative_artifact_binding(step_key="other")
        with self.assertRaisesRegex(admission.HubAdmissionContractError, "not covered"):
            admission.bind_execution_authorized_artifact(unadmitted, workflow_binding)


if __name__ == "__main__":
    unittest.main()
