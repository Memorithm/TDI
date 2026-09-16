import copy
import hashlib
import unittest

import tdi_provenance_contract as provenance


HEX = lambda char: char * 64
ATTEMPT = "e" * 64
HUB_ARTIFACT_ID = "11111111-1111-1111-1111-111111111111"


def producer(*, attempt=ATTEMPT, result=HEX("5")):
    return {
        "experiment_id": HEX("a"),
        "plan_id": HEX("b"),
        "trial_id": HEX("c"),
        "attempt_id": attempt,
        "step_identity": HEX("d"),
        "scientific_result_id": result,
    }


def artifact(name, payload, *, role="result", access="development", producer_value=None):
    return {
        "schema": 1,
        "name": name,
        "role": role,
        "media_type": "application/octet-stream",
        "size_bytes": len(payload),
        "sha256": hashlib.sha256(payload).hexdigest(),
        "access_class": access,
        "producer": producer_value,
    }


def ref(descriptor):
    return {
        "artifact_identity": provenance.artifact_identity(descriptor),
        "sha256": descriptor["sha256"],
    }


def record(*, inputs, outputs, attempt=ATTEMPT, result=HEX("5")):
    return {
        "schema": 1,
        "experiment_id": HEX("a"),
        "plan_id": HEX("b"),
        "trial_id": HEX("c"),
        "attempt_id": attempt,
        "step_identity": HEX("d"),
        "checkpoint_id": None,
        "scientific_result_id": result,
        "adapter_identity": "fixture-adapter/v1",
        "backend_identity": "fixture-backend/v1",
        "environment_identity": "fixture-environment/v1",
        "inputs": inputs,
        "outputs": outputs,
        "dependencies": [
            {"name": "tdi", "identity": "source:fixture"},
            {"name": "runtime", "identity": "runtime:fixture"},
        ],
    }


def export_fixture(*, output_access="development"):
    input_payload = b"input-fixture"
    output_payload = b"result-fixture"
    input_artifact = artifact(
        "input.bin", input_payload, role="input", access="development", producer_value=None
    )
    output_artifact = artifact(
        "result.bin",
        output_payload,
        access=output_access,
        producer_value=producer(),
    )
    run = record(inputs=[ref(input_artifact)], outputs=[ref(output_artifact)])
    manifest = {
        "schema": 1,
        "kind": "tdi-portable-export",
        "experiment_id": HEX("a"),
        "plan_id": HEX("b"),
        "artifacts": [output_artifact, input_artifact],
        "provenance": [run],
        "roots": [provenance.provenance_identity(run)],
    }
    payloads = {
        provenance.artifact_identity(input_artifact): input_payload,
        provenance.artifact_identity(output_artifact): output_payload,
    }
    return manifest, payloads, input_artifact, output_artifact, run


class ProvenanceContractTests(unittest.TestCase):
    def test_artifact_identity_and_actual_bytes_are_bound(self):
        payload = b"evidence"
        descriptor = artifact("evidence.bin", payload, role="evidence", producer_value=producer())
        identity = provenance.artifact_identity(descriptor)
        self.assertEqual(identity, provenance.verify_artifact_bytes(descriptor, payload))
        changed = copy.deepcopy(descriptor)
        changed["name"] = "renamed.bin"
        self.assertNotEqual(identity, provenance.artifact_identity(changed))
        with self.assertRaises(provenance.ProvenanceContractError):
            provenance.verify_artifact_bytes(descriptor, b"tampered")

    def test_hub_storage_binding_keeps_digest_namespaces_separate(self):
        payload = b"stored"
        descriptor = artifact("stored.bin", payload, producer_value=None)
        portable = descriptor["sha256"]
        binding = {
            "schema": 1,
            "artifact_identity": provenance.artifact_identity(descriptor),
            "portable_sha256": portable,
            "provider": "scirust-hub",
            "provider_artifact_id": HUB_ARTIFACT_ID,
            "provider_digest_namespace": provenance.HUB_ARTIFACT_DIGEST_NAMESPACE,
            # Equal text is allowed: the namespaces remain semantically distinct.
            "provider_digest": portable,
        }
        value = provenance.canonical_storage_binding(binding, descriptor)
        self.assertEqual(value["portable_sha256"], value["provider_digest"])
        self.assertEqual(
            value["provider_digest_namespace"], "scirust-hub:artifact-blob:v1"
        )
        wrong = copy.deepcopy(binding)
        wrong["provider_digest_namespace"] = "raw-sha256"
        with self.assertRaises(provenance.ProvenanceContractError):
            provenance.canonical_storage_binding(wrong, descriptor)
        wrong = copy.deepcopy(binding)
        wrong["provider_artifact_id"] = "not-a-uuid"
        with self.assertRaises(provenance.ProvenanceContractError):
            provenance.canonical_storage_binding(wrong, descriptor)

    def test_provenance_identity_canonicalizes_sets_but_binds_attempt_and_environment(self):
        manifest, _, input_artifact, output_artifact, run = export_fixture()
        del manifest
        second_input = artifact("other.bin", b"other", role="input", producer_value=None)
        run["inputs"].append(ref(second_input))
        a = provenance.provenance_identity(run)
        reordered = copy.deepcopy(run)
        reordered["inputs"].reverse()
        reordered["dependencies"].reverse()
        self.assertEqual(a, provenance.provenance_identity(reordered))
        changed = copy.deepcopy(run)
        changed["attempt_id"] = "f" * 64
        self.assertNotEqual(a, provenance.provenance_identity(changed))
        changed = copy.deepcopy(run)
        changed["environment_identity"] = "fixture-environment/v2"
        self.assertNotEqual(a, provenance.provenance_identity(changed))
        self.assertNotEqual(ref(input_artifact), ref(output_artifact))

    def test_export_closure_is_order_independent_and_payload_verified(self):
        manifest, payloads, _, _, _ = export_fixture()
        allowed = {"development"}
        identity = provenance.export_identity(manifest, allowed_access_classes=allowed)
        reordered = copy.deepcopy(manifest)
        reordered["artifacts"].reverse()
        self.assertEqual(identity, provenance.export_identity(reordered, allowed_access_classes=allowed))
        self.assertEqual(
            identity,
            provenance.verify_export_payloads(
                manifest, payloads, allowed_access_classes=allowed
            ),
        )
        bad = dict(payloads)
        first = next(iter(bad))
        bad[first] = b"different"
        with self.assertRaises(provenance.ProvenanceContractError):
            provenance.verify_export_payloads(manifest, bad, allowed_access_classes=allowed)
        missing = dict(payloads)
        missing.pop(next(iter(missing)))
        with self.assertRaises(provenance.ProvenanceContractError):
            provenance.verify_export_payloads(manifest, missing, allowed_access_classes=allowed)

    def test_export_requires_explicit_access_authority(self):
        manifest, _, _, _, _ = export_fixture(output_access="validation")
        with self.assertRaises(provenance.ProvenanceContractError):
            provenance.canonical_export(manifest, allowed_access_classes={"development"})
        with self.assertRaises(provenance.ProvenanceContractError):
            provenance.canonical_export(
                manifest, allowed_access_classes={"development", "unknown"}
            )
        provenance.canonical_export(
            manifest, allowed_access_classes={"development", "validation"}
        )

    def test_export_rejects_missing_or_hash_mismatched_references(self):
        manifest, _, _, _, _ = export_fixture()
        broken = copy.deepcopy(manifest)
        broken["provenance"][0]["outputs"][0]["sha256"] = HEX("0")
        with self.assertRaises(provenance.ProvenanceContractError):
            provenance.canonical_export(broken, allowed_access_classes={"development"})
        broken = copy.deepcopy(manifest)
        broken["artifacts"].pop(0)
        with self.assertRaises(provenance.ProvenanceContractError):
            provenance.canonical_export(broken, allowed_access_classes={"development"})

    def test_artifact_producer_claim_must_close_to_included_provenance(self):
        manifest, _, _, output_artifact, _ = export_fixture()
        broken_artifact = copy.deepcopy(output_artifact)
        broken_artifact["producer"]["attempt_id"] = "f" * 64
        broken = copy.deepcopy(manifest)
        broken["artifacts"][0] = broken_artifact
        broken["provenance"][0]["outputs"][0] = ref(broken_artifact)
        broken["roots"] = [provenance.provenance_identity(broken["provenance"][0])]
        with self.assertRaises(provenance.ProvenanceContractError):
            provenance.canonical_export(broken, allowed_access_classes={"development"})

    def test_export_requires_provenance_and_at_least_one_root(self):
        manifest, _, _, _, _ = export_fixture()
        broken = copy.deepcopy(manifest)
        broken["roots"] = []
        with self.assertRaises(provenance.ProvenanceContractError):
            provenance.canonical_export(broken, allowed_access_classes={"development"})
        broken = copy.deepcopy(manifest)
        broken["provenance"] = []
        broken["roots"] = []
        with self.assertRaises(provenance.ProvenanceContractError):
            provenance.canonical_export(broken, allowed_access_classes={"development"})


if __name__ == "__main__":
    unittest.main()
