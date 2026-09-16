import hashlib
import unittest

import tdi_artifact_contract as artifact
import tdi_experiment_contract as experiment
import tdi_hub_edge_contract as hub


class HubEdgeContractTests(unittest.TestCase):
    WORKFLOW = "123e4567-e89b-42d3-a456-426614174100"
    ATTEMPT = "123e4567-e89b-42d3-a456-426614174200"
    ARTIFACT = "123e4567-e89b-42d3-a456-426614174000"

    def descriptor(self, payload=b"payload"):
        return {
            "schema": 1,
            "name": "result",
            "raw_sha256": hashlib.sha256(payload).hexdigest(),
            "size_bytes": len(payload),
            "media_type": "application/octet-stream",
            "access_class": "development",
        }

    def response(self, payload=b"payload"):
        return {
            "id": self.ARTIFACT,
            "hub_digest": "a" * 64,
            "raw_sha256": hashlib.sha256(payload).hexdigest(),
            "size": len(payload),
        }

    def publication(self, *, generation=1, attempt=None, outputs=None):
        return {
            "schema_version": 1,
            "workflow": self.WORKFLOW,
            "step_key": "emit",
            "attempt": attempt or self.ATTEMPT,
            "generation": generation,
            "outputs": outputs if outputs is not None else {"result": self.ARTIFACT},
        }

    def authoritative_binding(self, **publication_overrides):
        portable = hub.bind_portable_artifact(self.descriptor(), self.response())
        return hub.bind_authoritative_publication(
            portable,
            self.publication(**publication_overrides),
            expected_workflow=self.WORKFLOW,
            expected_step_key="emit",
            expected_output="result",
        )

    def test_portable_response_binds_exact_tdi_descriptor(self):
        descriptor = self.descriptor()
        binding = hub.bind_portable_artifact(descriptor, self.response())
        self.assertEqual(artifact.artifact_identity(descriptor), binding["artifact_identity"])
        self.assertEqual(hub.PINNED_HUB_PORTABLE_DIGEST_SOURCE, binding["hub_source_sha"])
        self.assertFalse(binding["execution_authorized"])
        self.assertFalse(binding["publication_authoritative"])
        self.assertEqual(binding, hub.canonical_hub_artifact_binding(binding))

    def test_raw_digest_and_size_mismatch_fail_closed(self):
        with self.assertRaisesRegex(hub.HubEdgeContractError, "raw SHA-256"):
            hub.bind_portable_artifact(self.descriptor(), self.response(b"other"))

        response = self.response()
        response["size"] += 1
        with self.assertRaisesRegex(hub.HubEdgeContractError, "size"):
            hub.bind_portable_artifact(self.descriptor(), response)

    def test_malformed_hub_identity_fields_fail_closed(self):
        response = self.response()
        response["id"] = "not-a-uuid"
        with self.assertRaisesRegex(hub.HubEdgeContractError, "canonical UUID"):
            hub.canonical_portable_digest_response(response)

        response = self.response()
        response["hub_digest"] = "A" * 64
        with self.assertRaisesRegex(hub.HubEdgeContractError, "hexadecimal"):
            hub.canonical_portable_digest_response(response)

        response = self.response()
        response["extra"] = True
        with self.assertRaisesRegex(hub.HubEdgeContractError, "unknown or missing"):
            hub.canonical_portable_digest_response(response)

    def test_qualified_hub_source_is_pinned(self):
        binding = hub.bind_portable_artifact(self.descriptor(), self.response())
        binding["hub_source_sha"] = "0" * 40
        with self.assertRaisesRegex(hub.HubEdgeContractError, "not the qualified"):
            hub.canonical_hub_artifact_binding(binding)

        binding = hub.bind_portable_artifact(self.descriptor(), self.response())
        binding["hub_repository"] = "example/other"
        with self.assertRaisesRegex(hub.HubEdgeContractError, "repository pin"):
            hub.canonical_hub_artifact_binding(binding)

    def test_v1_authority_cannot_be_promoted_by_caller(self):
        for field in ("execution_authorized", "publication_authoritative"):
            binding = hub.bind_portable_artifact(self.descriptor(), self.response())
            binding[field] = True
            with self.assertRaises(hub.HubEdgeContractError):
                hub.canonical_hub_artifact_binding(binding)

    def test_v1_binding_identity_changes_with_hub_lineage(self):
        first = hub.bind_portable_artifact(self.descriptor(), self.response())
        second_response = self.response()
        second_response["id"] = "123e4567-e89b-42d3-a456-426614174001"
        second = hub.bind_portable_artifact(self.descriptor(), second_response)
        self.assertNotEqual(
            hub.hub_artifact_binding_identity(first),
            hub.hub_artifact_binding_identity(second),
        )

        third_response = self.response()
        third_response["hub_digest"] = "b" * 64
        third = hub.bind_portable_artifact(self.descriptor(), third_response)
        self.assertNotEqual(
            hub.hub_artifact_binding_identity(first),
            hub.hub_artifact_binding_identity(third),
        )

    def test_authoritative_publication_binds_exact_lineage_without_execution_authority(self):
        binding = self.authoritative_binding()
        self.assertEqual(2, binding["schema"])
        self.assertEqual(hub.PINNED_HUB_AUTHORITATIVE_PUBLICATION_SOURCE, binding["hub_publication_source_sha"])
        self.assertEqual(self.WORKFLOW, binding["workflow"])
        self.assertEqual(self.ATTEMPT, binding["attempt"])
        self.assertEqual("1", binding["generation"])
        self.assertEqual("result", binding["output_label"])
        self.assertEqual(self.ARTIFACT, binding["hub_artifact_id"])
        self.assertTrue(binding["publication_authoritative"])
        self.assertFalse(binding["execution_authorized"])
        self.assertEqual(binding, hub.canonical_authoritative_hub_artifact_binding(binding))

    def test_publication_response_rejects_schema_shape_uuid_name_and_generation_drift(self):
        cases = []

        value = self.publication()
        value["extra"] = True
        cases.append((value, "unknown or missing"))

        value = self.publication()
        value["schema_version"] = 2
        cases.append((value, "publication fence schema"))

        value = self.publication()
        value["workflow"] = "not-a-uuid"
        cases.append((value, "canonical UUID"))

        value = self.publication()
        value["step_key"] = "Emit"
        cases.append((value, "must match"))

        value = self.publication(generation=0)
        cases.append((value, "positive unsigned 64-bit"))

        value = self.publication(generation=(1 << 64))
        cases.append((value, "positive unsigned 64-bit"))

        value = self.publication(generation=True)
        cases.append((value, "positive unsigned 64-bit"))

        for value, message in cases:
            with self.subTest(message=message), self.assertRaisesRegex(hub.HubEdgeContractError, message):
                hub.canonical_authoritative_publication_response(value)

    def test_publication_response_rejects_malformed_or_excess_outputs(self):
        value = self.publication(outputs=[])
        with self.assertRaisesRegex(hub.HubEdgeContractError, "outputs must be an object"):
            hub.canonical_authoritative_publication_response(value)

        value = self.publication(outputs={"Bad.Label": self.ARTIFACT})
        with self.assertRaisesRegex(hub.HubEdgeContractError, "must match"):
            hub.canonical_authoritative_publication_response(value)

        outputs = {
            f"out_{index}": f"123e4567-e89b-42d3-a456-{index:012d}"
            for index in range(hub.HUB_MAX_AUTHORITATIVE_OUTPUTS + 1)
        }
        value = self.publication(outputs=outputs)
        with self.assertRaisesRegex(hub.HubEdgeContractError, "exceed pinned bound"):
            hub.canonical_authoritative_publication_response(value)

    def test_publication_lineage_mismatch_fails_closed(self):
        portable = hub.bind_portable_artifact(self.descriptor(), self.response())

        with self.assertRaisesRegex(hub.HubEdgeContractError, "workflow does not match"):
            hub.bind_authoritative_publication(
                portable,
                self.publication(),
                expected_workflow="123e4567-e89b-42d3-a456-426614174999",
                expected_step_key="emit",
                expected_output="result",
            )

        with self.assertRaisesRegex(hub.HubEdgeContractError, "step does not match"):
            hub.bind_authoritative_publication(
                portable,
                self.publication(),
                expected_workflow=self.WORKFLOW,
                expected_step_key="other",
                expected_output="result",
            )

        with self.assertRaisesRegex(hub.HubEdgeContractError, "does not contain"):
            hub.bind_authoritative_publication(
                portable,
                self.publication(),
                expected_workflow=self.WORKFLOW,
                expected_step_key="emit",
                expected_output="missing",
            )

        mismatched = self.publication(
            outputs={"result": "123e4567-e89b-42d3-a456-426614174001"}
        )
        with self.assertRaisesRegex(hub.HubEdgeContractError, "does not match the bound"):
            hub.bind_authoritative_publication(
                portable,
                mismatched,
                expected_workflow=self.WORKFLOW,
                expected_step_key="emit",
                expected_output="result",
            )

    def test_full_u64_generation_is_normalized_before_tdi_canonical_identity(self):
        generation = experiment.JSON_SAFE_INTEGER + 1
        binding = self.authoritative_binding(generation=generation)
        self.assertEqual(str(generation), binding["generation"])
        # The v2 binding remains canonicalizable because the Hub u64 is encoded
        # as a validated decimal string rather than an unsafe JSON number.
        experiment.canonical(binding)
        hub.authoritative_hub_artifact_binding_identity(binding)

    def test_v2_source_and_authority_flags_are_fail_closed(self):
        binding = self.authoritative_binding()
        binding["hub_publication_source_sha"] = "0" * 40
        with self.assertRaisesRegex(hub.HubEdgeContractError, "not the qualified authoritative"):
            hub.canonical_authoritative_hub_artifact_binding(binding)

        binding = self.authoritative_binding()
        binding["execution_authorized"] = True
        with self.assertRaisesRegex(hub.HubEdgeContractError, "execution remains unauthorized"):
            hub.canonical_authoritative_hub_artifact_binding(binding)

        binding = self.authoritative_binding()
        binding["publication_authoritative"] = False
        with self.assertRaisesRegex(hub.HubEdgeContractError, "requires qualified authoritative"):
            hub.canonical_authoritative_hub_artifact_binding(binding)

    def test_v2_identity_changes_with_attempt_and_generation_lineage(self):
        first = self.authoritative_binding()
        second = self.authoritative_binding(
            attempt="123e4567-e89b-42d3-a456-426614174201"
        )
        third = self.authoritative_binding(generation=2)
        self.assertNotEqual(
            hub.authoritative_hub_artifact_binding_identity(first),
            hub.authoritative_hub_artifact_binding_identity(second),
        )
        self.assertNotEqual(
            hub.authoritative_hub_artifact_binding_identity(first),
            hub.authoritative_hub_artifact_binding_identity(third),
        )


if __name__ == "__main__":
    unittest.main()
