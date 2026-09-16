import copy
import unittest

import tdi_nnis_partner_contract as nnis
import tdi_partner_adapter_contract as partner
import test_tdi_hub_admission_contract as hub_fixture
import test_tdi_partner_adapter_contract as partner_fixture

SHA = lambda c: c * 64


def admitted_nnis_step():
    descriptor = partner_fixture.descriptor("nnis")
    descriptor["source_sha"] = nnis.NNIS_SOURCE_SHA
    descriptor.update(nnis.nnis_adapter_surface())
    descriptor["protocol"] = {
        "name": nnis.NNIS_PROTOCOL_NAME,
        "version": nnis.NNIS_PROTOCOL_VERSION,
        "schema_identity": nnis.NNIS_PROTOCOL_SCHEMA_IDENTITY,
    }
    descriptor["hub_component"] = {
        **descriptor["hub_component"],
        "capability": nnis.NNIS_HUB_CAPABILITY,
        "capability_contract_version": nnis.NNIS_HUB_CAPABILITY_CONTRACT_VERSION,
    }
    return partner.bind_admitted_partner_step(
        descriptor, hub_fixture.bind_fixture(), step_key="prepare"
    )


def contract():
    return {
        "schema": 1,
        "partner_step": admitted_nnis_step(),
        "nnis": {
            "repository": nnis.NNIS_REPOSITORY,
            "source_sha": nnis.NNIS_SOURCE_SHA,
            "rust_msrv": nnis.NNIS_RUST_MSRV,
            "manifest_path": nnis.NNIS_MANIFEST_PATH,
            "manifest_blob_sha": nnis.NNIS_MANIFEST_BLOB_SHA,
            "manifest_schema": nnis.NNIS_MANIFEST_SCHEMA,
            "manifest_status": nnis.NNIS_MANIFEST_STATUS,
            "frontend_contract": nnis.NNIS_FRONTEND_CONTRACT,
            "validator_path": nnis.NNIS_VALIDATOR_PATH,
            "validator_blob_sha": nnis.NNIS_VALIDATOR_BLOB_SHA,
            "evidence_schema": nnis.NNIS_EVIDENCE_SCHEMA,
        },
        "request": {
            "domain": "Development",
            "qualification_scope": "cuda-rust-simt-contract",
            "candidate_artifact": {
                "schema": 1,
                "name": "tdi-nnis-hardware-qualification-candidate-v1.json",
                "raw_sha256": SHA("a"),
                "size_bytes": 768,
                "media_type": nnis.NNIS_CANDIDATE_MEDIA_TYPE,
                "access_class": "development",
            },
        },
        "review": {
            "owner": nnis.NNIS_REPOSITORY,
            "independent_nnis_validation_required": True,
            "exact_nnis_revision_required": True,
            "exact_device_cuda_identity_required": True,
            "correction_before_performance_required": True,
            "real_device_claims_require_evidence": True,
        },
        "permissions": {
            "nnis_execution_qualified": False,
            "hardware_qualification_authorized": False,
            "device_performance_qualified": False,
            "performance_claim_authorized": False,
            "production_routing_authorized": False,
            "protected_holdout_access_authorized": False,
            "scientific_stage_authorized": False,
            "scientific_verdict_authorized": False,
            "runtime_actuation_authorized": False,
        },
    }


class NnisPartnerContractTests(unittest.TestCase):
    def test_valid_contract_compiles_unresolved_nonproduction_review_metadata(self):
        value = nnis.canonical_nnis_qualification_contract(contract())
        self.assertEqual("nnis", value["partner_step"]["adapter"]["partner"])
        self.assertEqual(nnis.NNIS_SOURCE_SHA, value["nnis"]["source_sha"])
        self.assertEqual("unresolved_blocking", value["nnis"]["manifest_status"])
        self.assertFalse(any(value["permissions"].values()))
        compiled = nnis.compile_nnis_qualification_request(value)
        self.assertEqual("cuda-rust-simt-contract", compiled["qualification_scope"])
        self.assertEqual("unresolved_blocking", compiled["upstream_manifest_status"])
        self.assertFalse(compiled["upstream_qualification_resolved"])
        self.assertFalse(compiled["hardware_qualification_authorized"])
        self.assertFalse(compiled["performance_claim_authorized"])
        self.assertFalse(compiled["production_routing_authorized"])
        nnis.nnis_qualification_contract_identity(value)

    def test_exact_partner_source_protocol_capability_and_surface_are_required(self):
        value = contract()
        value["partner_step"]["adapter"]["partner"] = "forge"
        with self.assertRaises(nnis.NnisPartnerContractError):
            nnis.canonical_nnis_qualification_contract(value)

        value = contract()
        descriptor = value["partner_step"]["adapter"]
        descriptor["source_sha"] = "f" * 40
        value["partner_step"] = partner.bind_admitted_partner_step(
            descriptor, hub_fixture.bind_fixture(), step_key="prepare"
        )
        with self.assertRaisesRegex(nnis.NnisPartnerContractError, "audited source"):
            nnis.canonical_nnis_qualification_contract(value)

        for field, changed in (
            ("name", "tdi.nnis.other"),
            ("version", 2),
            ("schema_identity", SHA("f")),
        ):
            value = contract()
            descriptor = value["partner_step"]["adapter"]
            descriptor["protocol"][field] = changed
            value["partner_step"] = partner.bind_admitted_partner_step(
                descriptor, hub_fixture.bind_fixture(), step_key="prepare"
            )
            with self.subTest(field=field), self.assertRaisesRegex(
                nnis.NnisPartnerContractError, "protocol does not match"
            ):
                nnis.canonical_nnis_qualification_contract(value)

        value = contract()
        descriptor = value["partner_step"]["adapter"]
        descriptor["hub_component"] = {
            **descriptor["hub_component"],
            "capability": "tdi.evaluate",
            "capability_contract_version": "1.1.0",
        }
        value["partner_step"] = partner.bind_admitted_partner_step(
            descriptor, hub_fixture.bind_fixture(), step_key="evaluate"
        )
        with self.assertRaisesRegex(nnis.NnisPartnerContractError, "preparation boundary"):
            nnis.canonical_nnis_qualification_contract(value)

        for field, replacement in (
            ("capabilities", []),
            ("capabilities", ["runtime.actuate"]),
            ("inputs", []),
            ("outputs", [{"name": "result", "schema": 1, "identity": SHA("f")}]),
        ):
            value = contract()
            descriptor = value["partner_step"]["adapter"]
            descriptor[field] = replacement
            value["partner_step"] = partner.bind_admitted_partner_step(
                descriptor, hub_fixture.bind_fixture(), step_key="prepare"
            )
            with self.subTest(field=field), self.assertRaisesRegex(
                nnis.NnisPartnerContractError, "audited source projection"
            ):
                nnis.canonical_nnis_qualification_contract(value)

    def test_exact_audited_nnis_metadata_is_required(self):
        mutations = (
            ("repository", "Memorithm/Other"),
            ("source_sha", "f" * 40),
            ("rust_msrv", "1.89"),
            ("manifest_path", "docs/other.json"),
            ("manifest_blob_sha", "f" * 40),
            ("manifest_schema", "other-v1"),
            ("manifest_status", "qualified_nonproduction"),
            ("frontend_contract", "OTHER"),
            ("validator_path", "scripts/other.py"),
            ("validator_blob_sha", "f" * 40),
            ("evidence_schema", "other-evidence-v1"),
        )
        for field, changed in mutations:
            value = contract()
            value["nnis"][field] = changed
            with self.subTest(field=field), self.assertRaisesRegex(
                nnis.NnisPartnerContractError, "audited NNIS source"
            ):
                nnis.canonical_nnis_qualification_contract(value)

        value = contract()
        value["schema"] = True
        with self.assertRaisesRegex(nnis.NnisPartnerContractError, "integer 1"):
            nnis.canonical_nnis_qualification_contract(value)

    def test_candidate_domain_scope_and_artifact_are_fail_closed(self):
        for invalid in ([], {}, None, True, 1):
            value = contract()
            value["request"]["domain"] = invalid
            with self.subTest(domain=invalid), self.assertRaises(nnis.NnisPartnerContractError):
                nnis.canonical_nnis_qualification_contract(value)

        for invalid in ([], {}, None, True, 1):
            value = contract()
            value["request"]["qualification_scope"] = invalid
            with self.subTest(scope=invalid), self.assertRaises(nnis.NnisPartnerContractError):
                nnis.canonical_nnis_qualification_contract(value)

        value = contract()
        value["request"]["domain"] = "Validation"
        with self.assertRaisesRegex(nnis.NnisPartnerContractError, "access_class"):
            nnis.canonical_nnis_qualification_contract(value)

        value = contract()
        value["request"]["domain"] = "Validation"
        value["request"]["candidate_artifact"]["access_class"] = "validation"
        self.assertEqual(
            "Validation",
            nnis.canonical_nnis_qualification_contract(value)["request"]["domain"],
        )

        for scope in ("cuda-rust-simt-contract", "cuda-rust-simt-evidence-review"):
            value = contract()
            value["request"]["qualification_scope"] = scope
            with self.subTest(scope=scope):
                self.assertEqual(
                    scope,
                    nnis.canonical_nnis_qualification_contract(value)["request"]["qualification_scope"],
                )

        value = contract()
        value["request"]["candidate_artifact"]["media_type"] = "application/json"
        with self.assertRaisesRegex(nnis.NnisPartnerContractError, "media_type"):
            nnis.canonical_nnis_qualification_contract(value)

    def test_review_gates_and_authority_flags_cannot_be_relaxed(self):
        value = contract()
        value["review"]["owner"] = "Memorithm/TDI"
        with self.assertRaisesRegex(nnis.NnisPartnerContractError, "qualification owner"):
            nnis.canonical_nnis_qualification_contract(value)

        for field in (
            "independent_nnis_validation_required",
            "exact_nnis_revision_required",
            "exact_device_cuda_identity_required",
            "correction_before_performance_required",
            "real_device_claims_require_evidence",
        ):
            value = contract()
            value["review"][field] = False
            with self.subTest(field=field), self.assertRaises(nnis.NnisPartnerContractError):
                nnis.canonical_nnis_qualification_contract(value)

        for field in contract()["permissions"]:
            value = contract()
            value["permissions"][field] = True
            with self.subTest(field=field), self.assertRaisesRegex(
                nnis.NnisPartnerContractError, "grants no execution"
            ):
                nnis.canonical_nnis_qualification_contract(value)

        value = contract()
        value["permissions"]["performance_claim_authorized"] = 0
        with self.assertRaisesRegex(nnis.NnisPartnerContractError, "grants no execution"):
            nnis.canonical_nnis_qualification_contract(value)

    def test_common_binding_identity_is_revalidated(self):
        value = contract()
        value["partner_step"]["partner_adapter_identity"] = SHA("f")
        with self.assertRaisesRegex(
            nnis.NnisPartnerContractError, "does not match embedded evidence"
        ):
            nnis.canonical_nnis_qualification_contract(value)

    def test_schema_identities_are_source_derived_and_stable(self):
        self.assertEqual(64, len(nnis.NNIS_MANIFEST_SCHEMA_IDENTITY))
        self.assertEqual(64, len(nnis.NNIS_EVIDENCE_SCHEMA_IDENTITY))
        self.assertEqual(64, len(nnis.NNIS_PROTOCOL_SCHEMA_IDENTITY))
        self.assertNotEqual(nnis.NNIS_MANIFEST_SCHEMA_IDENTITY, nnis.NNIS_EVIDENCE_SCHEMA_IDENTITY)
        self.assertNotEqual(nnis.NNIS_PROTOCOL_SCHEMA_IDENTITY, nnis.NNIS_EVIDENCE_SCHEMA_IDENTITY)


if __name__ == "__main__":
    unittest.main()
