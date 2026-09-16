import unittest

import tdi_flat_partner_contract as flat
import tdi_partner_adapter_contract as partner
import test_tdi_hub_admission_contract as hub_fixture
import test_tdi_partner_adapter_contract as partner_fixture

SHA = lambda c: c * 64


def admitted_flat_step():
    descriptor = partner_fixture.descriptor("flat-attention")
    descriptor["source_sha"] = flat.FLAT_SOURCE_SHA
    descriptor.update(flat.flat_adapter_surface())
    descriptor["protocol"] = {
        "name": flat.FLAT_PROTOCOL_NAME,
        "version": flat.FLAT_PROTOCOL_VERSION,
        "schema_identity": flat.FLAT_PROTOCOL_SCHEMA_IDENTITY,
    }
    descriptor["hub_component"] = {
        **descriptor["hub_component"],
        "capability": flat.FLAT_HUB_CAPABILITY,
        "capability_contract_version": flat.FLAT_HUB_CAPABILITY_CONTRACT_VERSION,
    }
    return partner.bind_admitted_partner_step(
        descriptor, hub_fixture.bind_fixture(), step_key="prepare"
    )


def contract():
    return {
        "schema": 1,
        "partner_step": admitted_flat_step(),
        "flat": {
            "repository": flat.FLAT_REPOSITORY,
            "source_sha": flat.FLAT_SOURCE_SHA,
            "api_module": flat.FLAT_API_MODULE,
            "api_blob_sha": flat.FLAT_API_BLOB_SHA,
            "api_version": flat.FLAT_API_VERSION,
            "boolean_mask_module": flat.FLAT_BOOLEAN_MASK_MODULE,
            "boolean_mask_blob_sha": flat.FLAT_BOOLEAN_MASK_BLOB_SHA,
            "boolean_mask_schema": flat.FLAT_BOOLEAN_MASK_SCHEMA,
            "boolean_signature_module": flat.FLAT_BOOLEAN_SIGNATURE_MODULE,
            "boolean_signature_blob_sha": flat.FLAT_BOOLEAN_SIGNATURE_BLOB_SHA,
            "boolean_signature_schema": flat.FLAT_BOOLEAN_SIGNATURE_SCHEMA,
        },
        "request": {
            "domain": "Development",
            "qualification_scope": "boolean-front-end-contract",
            "candidate_artifact": {
                "schema": 1,
                "name": "tdi-flat-qualification-candidate-v1.json",
                "raw_sha256": SHA("a"),
                "size_bytes": 512,
                "media_type": flat.FLAT_CANDIDATE_MEDIA_TYPE,
                "access_class": "development",
            },
        },
        "review": {
            "owner": flat.FLAT_REPOSITORY,
            "independent_flat_validation_required": True,
            "dense_reference_required": True,
            "real_device_claims_require_evidence": True,
        },
        "permissions": {
            "flat_execution_qualified": False,
            "real_device_performance_qualified": False,
            "boolean_front_end_speedup_qualified": False,
            "protected_holdout_access_authorized": False,
            "scientific_stage_authorized": False,
            "scientific_verdict_authorized": False,
            "runtime_actuation_authorized": False,
        },
    }


class FlatPartnerContractTests(unittest.TestCase):
    def test_valid_contract_compiles_nonexecuting_review_metadata(self):
        value = flat.canonical_flat_qualification_contract(contract())
        self.assertEqual("flat-attention", value["partner_step"]["adapter"]["partner"])
        self.assertEqual(flat.FLAT_SOURCE_SHA, value["flat"]["source_sha"])
        self.assertFalse(any(value["permissions"].values()))
        compiled = flat.compile_flat_qualification_request(value)
        self.assertEqual("boolean-front-end-contract", compiled["qualification_scope"])
        self.assertTrue(compiled["dense_reference_required"])
        self.assertTrue(compiled["real_device_claims_require_evidence"])
        self.assertFalse(compiled["flat_execution_qualified"])
        self.assertFalse(compiled["boolean_front_end_speedup_qualified"])
        flat.flat_qualification_contract_identity(value)

    def test_exact_partner_source_protocol_capability_and_surface_are_required(self):
        value = contract()
        value["partner_step"]["adapter"]["partner"] = "forge"
        with self.assertRaises(flat.FlatPartnerContractError):
            flat.canonical_flat_qualification_contract(value)

        value = contract()
        descriptor = value["partner_step"]["adapter"]
        descriptor["source_sha"] = "f" * 40
        value["partner_step"] = partner.bind_admitted_partner_step(
            descriptor, hub_fixture.bind_fixture(), step_key="prepare"
        )
        with self.assertRaisesRegex(flat.FlatPartnerContractError, "audited source"):
            flat.canonical_flat_qualification_contract(value)

        for field, changed in (
            ("name", "tdi.flat-attention.other"),
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
                flat.FlatPartnerContractError, "protocol does not match"
            ):
                flat.canonical_flat_qualification_contract(value)

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
        with self.assertRaisesRegex(flat.FlatPartnerContractError, "preparation boundary"):
            flat.canonical_flat_qualification_contract(value)

        for field, replacement in (
            ("capabilities", []),
            ("capabilities", ["flat.api.v1"]),
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
                flat.FlatPartnerContractError, "audited source projection"
            ):
                flat.canonical_flat_qualification_contract(value)

    def test_exact_audited_flat_metadata_and_integer_versions_are_required(self):
        mutations = (
            ("repository", "Memorithm/Other"),
            ("source_sha", "f" * 40),
            ("api_module", "src/other.rs"),
            ("api_blob_sha", "f" * 40),
            ("boolean_mask_module", "src/other-mask.rs"),
            ("boolean_mask_blob_sha", "f" * 40),
            ("boolean_signature_module", "src/other-signature.rs"),
            ("boolean_signature_blob_sha", "f" * 40),
        )
        for field, changed in mutations:
            value = contract()
            value["flat"][field] = changed
            with self.subTest(field=field), self.assertRaisesRegex(
                flat.FlatPartnerContractError, "audited FLAT source"
            ):
                flat.canonical_flat_qualification_contract(value)

        for field in ("api_version", "boolean_mask_schema", "boolean_signature_schema"):
            value = contract()
            value["flat"][field] = True
            with self.subTest(field=field), self.assertRaisesRegex(
                flat.FlatPartnerContractError, "integer 1"
            ):
                flat.canonical_flat_qualification_contract(value)

        value = contract()
        value["schema"] = True
        with self.assertRaisesRegex(flat.FlatPartnerContractError, "integer 1"):
            flat.canonical_flat_qualification_contract(value)

    def test_candidate_domain_scope_and_artifact_are_fail_closed(self):
        for invalid in ([], {}, None, True, 1):
            value = contract()
            value["request"]["domain"] = invalid
            with self.subTest(domain=invalid), self.assertRaises(flat.FlatPartnerContractError):
                flat.canonical_flat_qualification_contract(value)

        value = contract()
        value["request"]["domain"] = "Validation"
        with self.assertRaisesRegex(flat.FlatPartnerContractError, "access_class"):
            flat.canonical_flat_qualification_contract(value)

        value = contract()
        value["request"]["domain"] = "Validation"
        value["request"]["candidate_artifact"]["access_class"] = "validation"
        self.assertEqual(
            "Validation",
            flat.canonical_flat_qualification_contract(value)["request"]["domain"],
        )

        for scope in ("api-contract", "boolean-front-end-contract"):
            value = contract()
            value["request"]["qualification_scope"] = scope
            with self.subTest(scope=scope):
                self.assertEqual(
                    scope,
                    flat.canonical_flat_qualification_contract(value)["request"]["qualification_scope"],
                )

        value = contract()
        value["request"]["qualification_scope"] = "performance-win"
        with self.assertRaisesRegex(flat.FlatPartnerContractError, "not an admitted"):
            flat.canonical_flat_qualification_contract(value)

        value = contract()
        value["request"]["candidate_artifact"]["media_type"] = "application/json"
        with self.assertRaisesRegex(flat.FlatPartnerContractError, "media_type"):
            flat.canonical_flat_qualification_contract(value)

    def test_review_gates_and_authority_flags_cannot_be_relaxed(self):
        value = contract()
        value["review"]["owner"] = "Memorithm/TDI"
        with self.assertRaisesRegex(flat.FlatPartnerContractError, "qualification owner"):
            flat.canonical_flat_qualification_contract(value)

        for field in (
            "independent_flat_validation_required",
            "dense_reference_required",
            "real_device_claims_require_evidence",
        ):
            value = contract()
            value["review"][field] = False
            with self.subTest(field=field), self.assertRaises(flat.FlatPartnerContractError):
                flat.canonical_flat_qualification_contract(value)

        for field in contract()["permissions"]:
            value = contract()
            value["permissions"][field] = True
            with self.subTest(field=field), self.assertRaisesRegex(
                flat.FlatPartnerContractError, "grants no execution"
            ):
                flat.canonical_flat_qualification_contract(value)

        value = contract()
        value["permissions"]["flat_execution_qualified"] = 0
        with self.assertRaisesRegex(flat.FlatPartnerContractError, "grants no execution"):
            flat.canonical_flat_qualification_contract(value)

    def test_common_binding_identity_is_revalidated(self):
        value = contract()
        value["partner_step"]["partner_adapter_identity"] = SHA("f")
        with self.assertRaisesRegex(flat.FlatPartnerContractError, "does not match embedded evidence"):
            flat.canonical_flat_qualification_contract(value)


if __name__ == "__main__":
    unittest.main()
