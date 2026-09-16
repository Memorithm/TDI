import copy
import unittest

import tdi_partner_adapter_contract as partner
import tdi_scirust_partner_contract as scirust
import test_tdi_hub_admission_contract as hub_fixture
import test_tdi_partner_adapter_contract as partner_fixture

SHA = lambda c: c * 64


def admitted_scirust_step():
    descriptor = partner_fixture.descriptor("scirust")
    descriptor["source_sha"] = scirust.SCIRUST_SOURCE_SHA
    descriptor["protocol"] = {
        "name": scirust.SCIRUST_PROTOCOL_NAME,
        "version": scirust.SCIRUST_PROTOCOL_VERSION,
        "schema_identity": scirust.SCIRUST_PROTOCOL_SCHEMA_IDENTITY,
    }
    descriptor["hub_component"] = {
        **descriptor["hub_component"],
        "capability": scirust.SCIRUST_HUB_CAPABILITY,
        "capability_contract_version": scirust.SCIRUST_HUB_CAPABILITY_CONTRACT_VERSION,
    }
    surface = scirust.scirust_adapter_surface()
    descriptor["capabilities"] = copy.deepcopy(surface["capabilities"])
    descriptor["inputs"] = copy.deepcopy(surface["inputs"])
    descriptor["outputs"] = copy.deepcopy(surface["outputs"])
    return partner.bind_admitted_partner_step(
        descriptor, hub_fixture.bind_fixture(), step_key="prepare"
    )


def contract():
    return {
        "schema": 1,
        "partner_step": admitted_scirust_step(),
        "scirust": {
            "repository": scirust.SCIRUST_REPOSITORY,
            "source_sha": scirust.SCIRUST_SOURCE_SHA,
            "tensor_ir_crate": scirust.SCIRUST_TENSOR_IR_CRATE,
            "representation_module": scirust.SCIRUST_REPRESENTATION_MODULE,
            "representation_blob_sha": scirust.SCIRUST_REPRESENTATION_BLOB_SHA,
            "public_surfaces": list(scirust.SCIRUST_PUBLIC_SURFACES),
        },
        "request": {
            "domain": "Development",
            "candidate_artifact": {
                "schema": 1,
                "name": "tdi-scirust-representation-candidate-v1.json",
                "raw_sha256": SHA("a"),
                "size_bytes": 321,
                "media_type": scirust.SCIRUST_CANDIDATE_MEDIA_TYPE,
                "access_class": "development",
            },
            "primitive_kind": "representation-primitive",
            "promotion_scope": "candidate-only",
        },
        "review": {
            "owner": scirust.SCIRUST_REPOSITORY,
            "independent_scirust_validation_required": True,
            "implementation_copy_forbidden": True,
        },
        "permissions": {
            "scirust_execution_qualified": False,
            "primitive_promotion_authorized": False,
            "protected_holdout_access_authorized": False,
            "scientific_stage_authorized": False,
            "scientific_verdict_authorized": False,
            "runtime_actuation_authorized": False,
        },
    }


class SciRustPartnerContractTests(unittest.TestCase):
    def test_valid_contract_compiles_authority_free_promotion_request(self):
        value = scirust.canonical_scirust_promotion_contract(contract())
        self.assertEqual("scirust", value["partner_step"]["adapter"]["partner"])
        self.assertEqual(scirust.SCIRUST_SOURCE_SHA, value["scirust"]["source_sha"])
        self.assertFalse(any(value["permissions"].values()))

        request = scirust.compile_scirust_promotion_request(value)
        self.assertEqual(scirust.SCIRUST_PROTOCOL_NAME, request["protocol"])
        self.assertEqual("representation-primitive", request["primitive_kind"])
        self.assertEqual(list(scirust.SCIRUST_PUBLIC_SURFACES), request["public_surfaces"])
        self.assertTrue(request["independent_scirust_validation_required"])
        self.assertFalse(request["scirust_execution_qualified"])
        self.assertFalse(request["primitive_promotion_authorized"])
        scirust.scirust_promotion_contract_identity(value)

    def test_contract_requires_exact_scirust_adapter_surface(self):
        mutations = (
            ("capabilities", ["runtime.actuate"]),
            ("inputs", []),
            ("outputs", [{"name": "response", "schema": 1, "identity": SHA("f")}]),
        )
        for field, changed in mutations:
            value = contract()
            descriptor = value["partner_step"]["adapter"]
            descriptor[field] = changed
            value["partner_step"] = partner.bind_admitted_partner_step(
                descriptor, hub_fixture.bind_fixture(), step_key="prepare"
            )
            with self.subTest(field=field), self.assertRaisesRegex(
                scirust.SciRustPartnerContractError, f"adapter {field}"
            ):
                scirust.canonical_scirust_promotion_contract(value)

    def test_non_string_request_selectors_fail_as_contract_errors(self):
        for field in ("domain", "primitive_kind"):
            value = contract()
            value["request"][field] = []
            with self.subTest(field=field), self.assertRaises(
                scirust.SciRustPartnerContractError
            ):
                scirust.canonical_scirust_promotion_contract(value)

    def test_contract_requires_exact_scirust_partner_source_protocol_and_prepare_capability(self):
        value = contract()
        value["partner_step"]["adapter"]["partner"] = "forge"
        with self.assertRaises(scirust.SciRustPartnerContractError):
            scirust.canonical_scirust_promotion_contract(value)

        value = contract()
        value["partner_step"]["adapter"]["source_sha"] = "f" * 40
        value["partner_step"] = partner.bind_admitted_partner_step(
            value["partner_step"]["adapter"], hub_fixture.bind_fixture(), step_key="prepare"
        )
        with self.assertRaisesRegex(scirust.SciRustPartnerContractError, "audited source"):
            scirust.canonical_scirust_promotion_contract(value)

        for field, changed in (
            ("name", "tdi.scirust.other"),
            ("version", 2),
            ("schema_identity", SHA("f")),
        ):
            value = contract()
            value["partner_step"]["adapter"]["protocol"][field] = changed
            value["partner_step"] = partner.bind_admitted_partner_step(
                value["partner_step"]["adapter"], hub_fixture.bind_fixture(), step_key="prepare"
            )
            with self.subTest(field=field), self.assertRaisesRegex(
                scirust.SciRustPartnerContractError, "protocol does not match"
            ):
                scirust.canonical_scirust_promotion_contract(value)

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
        with self.assertRaisesRegex(scirust.SciRustPartnerContractError, "preparation boundary"):
            scirust.canonical_scirust_promotion_contract(value)

    def test_audited_scirust_surface_is_exact_and_type_faithful(self):
        mutations = (
            ("repository", "Memorithm/Other", "does not match SciRust"),
            ("source_sha", "f" * 40, "audited SciRust source"),
            ("tensor_ir_crate", "other", "crate identity drift"),
            ("representation_module", "other.rs", "module identity drift"),
            ("representation_blob_sha", "f" * 40, "blob drift"),
        )
        for field, changed, message in mutations:
            value = contract()
            value["scirust"][field] = changed
            with self.subTest(field=field), self.assertRaisesRegex(
                scirust.SciRustPartnerContractError, message
            ):
                scirust.canonical_scirust_promotion_contract(value)

        value = contract()
        value["scirust"]["public_surfaces"] = list(reversed(scirust.SCIRUST_PUBLIC_SURFACES))
        with self.assertRaisesRegex(scirust.SciRustPartnerContractError, "public surface drift"):
            scirust.canonical_scirust_promotion_contract(value)

        value = contract()
        value["schema"] = True
        with self.assertRaisesRegex(scirust.SciRustPartnerContractError, "integer 1"):
            scirust.canonical_scirust_promotion_contract(value)

    def test_candidate_artifact_is_domain_bound_and_candidate_only(self):
        value = contract()
        value["request"]["domain"] = "Validation"
        with self.assertRaisesRegex(scirust.SciRustPartnerContractError, "access_class"):
            scirust.canonical_scirust_promotion_contract(value)

        value = contract()
        value["request"]["domain"] = "Validation"
        value["request"]["candidate_artifact"]["access_class"] = "validation"
        canonical = scirust.canonical_scirust_promotion_contract(value)
        self.assertEqual("Validation", canonical["request"]["domain"])

        value = contract()
        value["request"]["candidate_artifact"]["media_type"] = "application/json"
        with self.assertRaisesRegex(scirust.SciRustPartnerContractError, "media_type"):
            scirust.canonical_scirust_promotion_contract(value)

        value = contract()
        value["request"]["promotion_scope"] = "auto-merge"
        with self.assertRaisesRegex(scirust.SciRustPartnerContractError, "candidate-only"):
            scirust.canonical_scirust_promotion_contract(value)

    def test_only_declared_reusable_primitive_classes_are_admitted(self):
        for kind in ("representation-primitive", "storage-accounting-primitive"):
            value = contract()
            value["request"]["primitive_kind"] = kind
            with self.subTest(kind=kind):
                self.assertEqual(
                    kind,
                    scirust.canonical_scirust_promotion_contract(value)["request"]["primitive_kind"],
                )

        value = contract()
        value["request"]["primitive_kind"] = "tdi-scientific-state"
        with self.assertRaisesRegex(scirust.SciRustPartnerContractError, "not an allowed"):
            scirust.canonical_scirust_promotion_contract(value)

    def test_review_owner_and_authority_flags_cannot_be_relaxed(self):
        value = contract()
        value["review"]["owner"] = "Memorithm/TDI"
        with self.assertRaisesRegex(scirust.SciRustPartnerContractError, "review owner"):
            scirust.canonical_scirust_promotion_contract(value)

        for field in ("independent_scirust_validation_required", "implementation_copy_forbidden"):
            value = contract()
            value["review"][field] = False
            with self.subTest(field=field), self.assertRaises(scirust.SciRustPartnerContractError):
                scirust.canonical_scirust_promotion_contract(value)

        for field in contract()["permissions"]:
            value = contract()
            value["permissions"][field] = True
            with self.subTest(field=field), self.assertRaisesRegex(
                scirust.SciRustPartnerContractError, "grants no execution"
            ):
                scirust.canonical_scirust_promotion_contract(value)

        value = contract()
        value["permissions"]["primitive_promotion_authorized"] = 0
        with self.assertRaisesRegex(scirust.SciRustPartnerContractError, "grants no execution"):
            scirust.canonical_scirust_promotion_contract(value)

    def test_embedded_common_binding_is_revalidated(self):
        value = contract()
        value["partner_step"]["partner_adapter_identity"] = SHA("f")
        with self.assertRaisesRegex(
            scirust.SciRustPartnerContractError, "does not match embedded evidence"
        ):
            scirust.canonical_scirust_promotion_contract(value)

    def test_compiled_request_carries_no_execution_or_promotion_authority(self):
        request = scirust.compile_scirust_promotion_request(contract())
        self.assertNotIn("permissions", request)
        self.assertFalse(request["scirust_execution_qualified"])
        self.assertFalse(request["primitive_promotion_authorized"])
        self.assertTrue(request["independent_scirust_validation_required"])


if __name__ == "__main__":
    unittest.main()