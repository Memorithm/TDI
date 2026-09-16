import hashlib
import unittest

import tdi_artifact_contract as artifact
import tdi_hub_edge_contract as hub


class HubEdgeContractTests(unittest.TestCase):
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
            "id": "123e4567-e89b-42d3-a456-426614174000",
            "hub_digest": "a" * 64,
            "raw_sha256": hashlib.sha256(payload).hexdigest(),
            "size": len(payload),
        }

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

    def test_authority_cannot_be_promoted_by_caller(self):
        for field in ("execution_authorized", "publication_authoritative"):
            binding = hub.bind_portable_artifact(self.descriptor(), self.response())
            binding[field] = True
            with self.assertRaises(hub.HubEdgeContractError):
                hub.canonical_hub_artifact_binding(binding)

    def test_binding_identity_changes_with_hub_lineage(self):
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


if __name__ == "__main__":
    unittest.main()
