import hashlib
import unittest

import tdi_artifact_contract as contract


H = lambda ch: ch * 64


class ArtifactContractTests(unittest.TestCase):
    def artifact(self, payload=b"payload", *, name="result", access="development"):
        return {
            "schema": 1,
            "name": name,
            "raw_sha256": hashlib.sha256(payload).hexdigest(),
            "size_bytes": len(payload),
            "media_type": "application/octet-stream",
            "access_class": access,
        }

    def provenance(self, artifact_identity, *, domain="Development"):
        return {
            "schema": 1,
            "artifact_identity": artifact_identity,
            "experiment_id": H("1"),
            "plan_id": H("2"),
            "trial_id": H("3"),
            "attempt_id": H("4"),
            "step_key": "fit",
            "step_identity": H("5"),
            "implementation_identity": "tdi:test@1",
            "domain": domain,
            "inputs": [
                {"name": "z", "identity": H("a")},
                {"name": "a", "identity": H("b")},
            ],
            "dependencies": [{"name": "scirust", "identity": H("c")}],
        }

    def cache_request(self, *, domain="Development", policy="exact-domain"):
        return {
            "schema": 1,
            "policy": policy,
            "domain": domain,
            "plan_id": H("2"),
            "step_identity": H("5"),
            "implementation_identity": "tdi:test@1",
            "backend_identity": "cpu:reference",
            "inputs": [
                {"name": "right", "identity": H("a")},
                {"name": "left", "identity": H("b")},
            ],
            "parameters_identity": H("d"),
        }

    def test_artifact_identity_and_bytes_are_exact(self):
        payload = b"payload"
        artifact = self.artifact(payload)
        first = contract.artifact_identity(artifact)
        reordered = dict(reversed(list(artifact.items())))
        self.assertEqual(first, contract.artifact_identity(reordered))
        contract.verify_artifact_bytes(artifact, payload)
        with self.assertRaisesRegex(contract.ArtifactContractError, "size mismatch"):
            contract.verify_artifact_bytes(artifact, payload + b"x")
        tampered = dict(artifact, raw_sha256=H("0"))
        with self.assertRaisesRegex(contract.ArtifactContractError, "SHA-256 mismatch"):
            contract.verify_artifact_bytes(tampered, payload)

    def test_portable_sha_is_not_hub_content_digest_semantics(self):
        artifact = self.artifact(b"same")
        raw = hashlib.sha256(b"same").hexdigest()
        self.assertEqual(raw, artifact["raw_sha256"])
        # Pinned Hub v1 uses a framed, domain-separated preimage. Reproducing
        # the prefix/domain here proves the wire values are intentionally not
        # interchangeable even though both are 64 lowercase hex characters.
        domain = b"scirust-hub:artifact-blob:v1"
        framed = b"scirust-hub-digest:v1\0" + len(domain).to_bytes(8, "little") + domain + b"same"
        self.assertNotEqual(raw, hashlib.sha256(framed).hexdigest())

    def test_provenance_is_order_canonical_and_domain_bound(self):
        artifact_id = contract.artifact_identity(self.artifact())
        provenance = self.provenance(artifact_id)
        normalized = contract.canonical_provenance(provenance)
        self.assertEqual(["a", "z"], [item["name"] for item in normalized["inputs"]])
        changed = dict(provenance, domain="Validation")
        self.assertNotEqual(
            contract.provenance_identity(provenance),
            contract.provenance_identity(changed),
        )
        duplicate = dict(provenance, inputs=[
            {"name": "same", "identity": H("a")},
            {"name": "same", "identity": H("b")},
        ])
        with self.assertRaisesRegex(contract.ArtifactContractError, "duplicate"):
            contract.canonical_provenance(duplicate)

    def test_export_verifies_complete_bytes_and_provenance_binding(self):
        payload = b"payload"
        artifact = self.artifact(payload, access="validation")
        artifact_id = contract.artifact_identity(artifact)
        provenance = self.provenance(artifact_id, domain="Validation")
        provenance_id = contract.provenance_identity(provenance)
        manifest = {
            "schema": 1,
            "export_id": "validation-report",
            "access_class": "validation",
            "members": [{
                "name": "result",
                "artifact_identity": artifact_id,
                "provenance_identity": provenance_id,
            }],
        }
        contract.verify_export(
            manifest,
            {"result": artifact},
            {"result": provenance},
            {"result": payload},
        )
        with self.assertRaisesRegex(contract.ArtifactContractError, "member set mismatch"):
            contract.verify_export(manifest, {"result": artifact}, {"result": provenance}, {})
        wrong = dict(provenance, artifact_identity=H("f"))
        with self.assertRaisesRegex(contract.ArtifactContractError, "provenance identity mismatch"):
            contract.verify_export(
                manifest,
                {"result": artifact},
                {"result": wrong},
                {"result": payload},
            )

    def test_export_cannot_weaken_artifact_access(self):
        payload = b"secret"
        artifact = self.artifact(payload, access="restricted-reference")
        artifact_id = contract.artifact_identity(artifact)
        provenance = self.provenance(artifact_id, domain="Validation")
        manifest = {
            "schema": 1,
            "export_id": "bad-public-export",
            "access_class": "public",
            "members": [{
                "name": "result",
                "artifact_identity": artifact_id,
                "provenance_identity": contract.provenance_identity(provenance),
            }],
        }
        with self.assertRaisesRegex(contract.ArtifactContractError, "weakens artifact access"):
            contract.verify_export(
                manifest,
                {"result": artifact},
                {"result": provenance},
                {"result": payload},
            )

    def test_export_may_make_public_artifact_more_restrictive(self):
        payload = b"public"
        artifact = self.artifact(payload, access="public")
        artifact_id = contract.artifact_identity(artifact)
        provenance = self.provenance(artifact_id, domain="Development")
        manifest = {
            "schema": 1,
            "export_id": "restricted-wrapper",
            "access_class": "development",
            "members": [{
                "name": "result",
                "artifact_identity": artifact_id,
                "provenance_identity": contract.provenance_identity(provenance),
            }],
        }
        contract.verify_export(
            manifest,
            {"result": artifact},
            {"result": provenance},
            {"result": payload},
        )

    def test_cache_key_is_exact_domain_and_input_order_canonical(self):
        request = self.cache_request()
        key = contract.cache_key(request)
        reordered = dict(request, inputs=list(reversed(request["inputs"])))
        self.assertEqual(key, contract.cache_key(reordered))
        self.assertNotEqual(key, contract.cache_key(self.cache_request(domain="Validation")))
        changed = dict(request, backend_identity="cpu:other")
        self.assertNotEqual(key, contract.cache_key(changed))

    def test_cache_disabled_and_unauthorized_fail_closed(self):
        with self.assertRaisesRegex(contract.ArtifactContractError, "disabled"):
            contract.cache_key(self.cache_request(policy="disabled"))
        request = self.cache_request()
        entry = {
            "schema": 1,
            "cache_key": contract.cache_key(request),
            "artifact_identity": H("e"),
            "provenance_identity": H("f"),
            "request": request,
        }
        with self.assertRaisesRegex(contract.ArtifactContractError, "lacks caller authorization"):
            contract.validate_cache_reuse(entry, request, cache_authorized=False)

    def test_cache_entry_must_bind_exact_request(self):
        request = self.cache_request()
        entry = {
            "schema": 1,
            "cache_key": contract.cache_key(request),
            "artifact_identity": H("e"),
            "provenance_identity": H("f"),
            "request": request,
        }
        accepted = contract.validate_cache_reuse(entry, request, cache_authorized=True)
        self.assertEqual(entry["cache_key"], accepted["cache_key"])
        drift = self.cache_request(domain="Validation")
        with self.assertRaisesRegex(contract.ArtifactContractError, "request binding mismatch"):
            contract.validate_cache_reuse(entry, drift, cache_authorized=True)

    def test_unknown_fields_and_oversized_lists_fail_closed(self):
        artifact = dict(self.artifact(), extra=True)
        with self.assertRaisesRegex(contract.ArtifactContractError, "unknown or missing"):
            contract.canonical_artifact(artifact)
        provenance = self.provenance(H("a"))
        provenance["inputs"] = [
            {"name": f"input-{i}", "identity": H("b")} for i in range(contract.MAX_RECORDS + 1)
        ]
        with self.assertRaisesRegex(contract.ArtifactContractError, "item bound"):
            contract.canonical_provenance(provenance)


if __name__ == "__main__":
    unittest.main()
