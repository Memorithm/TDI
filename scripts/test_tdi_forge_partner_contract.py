import copy
import unittest

import tdi_forge_partner_contract as forge
import tdi_partner_adapter_contract as partner
import test_tdi_hub_admission_contract as hub_fixture
import test_tdi_partner_adapter_contract as partner_fixture

SHA = lambda c: c * 64


def admitted_forge_step():
    descriptor = partner_fixture.descriptor("forge")
    descriptor["source_sha"] = forge.FORGE_SOURCE_SHA
    descriptor["protocol"] = {
        "name": "forge.scientific-external-domain",
        "version": 1,
        "schema_identity": forge.FORGE_PROTOCOL_SCHEMA_IDENTITY,
    }
    return partner.bind_admitted_partner_step(
        descriptor, hub_fixture.bind_fixture(), step_key="evaluate"
    )


def contract():
    return {
        "schema": 1,
        "partner_step": admitted_forge_step(),
        "forge": {
            "repository": forge.FORGE_REPOSITORY,
            "source_sha": forge.FORGE_SOURCE_SHA,
            "external_domain_schema_version": 1,
            "scientific_domain_schema_version": 1,
        },
        "domain_id": "tdi/boolean-policy-development-v1",
        "upstream": {
            "repository": "Memorithm/TDI",
            "commit_id": "c" * 40,
            "contract_sha256": SHA("a"),
        },
        "allowed_candidate_dimensions": [
            "predicate_schema",
            "policy_parameters",
        ],
        "data_boundary": {
            "generation_sources": ["development-v1"],
            "verification_sources": ["validation-v1"],
            "final_holdout_sources": ["confirmatory-v1"],
        },
        "verification": {
            "adapter_id": "tdi-reference-validator-v1",
            "adapter_sha256": SHA("b"),
        },
        "objectives": [
            {"name": "task_quality", "direction": "maximize"},
            {"name": "semantic_work", "direction": "minimize"},
        ],
        "environment": {
            "fingerprint_required": True,
            "isolation_required": False,
        },
        "permissions": {
            "forge_execution_qualified": False,
            "protected_holdout_access_authorized": False,
            "scientific_stage_authorized": False,
            "scientific_verdict_authorized": False,
            "runtime_actuation_authorized": False,
        },
    }


class ForgePartnerContractTests(unittest.TestCase):
    def test_valid_contract_compiles_exact_forge_v1_wire_shape(self):
        value = forge.canonical_forge_search_contract(contract())
        self.assertEqual("forge", value["partner_step"]["adapter"]["partner"])
        self.assertEqual(forge.FORGE_SOURCE_SHA, value["forge"]["source_sha"])
        self.assertFalse(any(value["permissions"].values()))

        manifest = forge.compile_forge_scientific_domain_manifest(value)
        self.assertEqual(
            {
                "schema_version": 1,
                "external_domain": {
                    "schema_version": 1,
                    "domain_id": "tdi/boolean-policy-development-v1",
                    "upstream": {
                        "repository": "Memorithm/TDI",
                        "commit_id": "c" * 40,
                        "contract_sha256": SHA("a"),
                    },
                    "allowed_candidate_dimensions": [
                        "predicate_schema",
                        "policy_parameters",
                    ],
                    "data_boundary": {
                        "generation_sources": ["development-v1"],
                        "verification_sources": ["validation-v1"],
                        "final_holdout_sources": ["confirmatory-v1"],
                    },
                    "verification": {
                        "adapter_id": "tdi-reference-validator-v1",
                        "adapter_sha256": SHA("b"),
                    },
                    "objectives": [
                        {"name": "task_quality", "direction": "maximize"},
                        {"name": "semantic_work", "direction": "minimize"},
                    ],
                    "environment": {
                        "fingerprint_required": True,
                        "isolation_required": False,
                    },
                },
            },
            manifest,
        )
        forge.forge_search_contract_identity(value)

    def test_contract_requires_exact_forge_partner_and_audited_source(self):
        value = contract()
        value["partner_step"]["adapter"]["partner"] = "elasticxxx"
        with self.assertRaises(forge.ForgePartnerContractError):
            forge.canonical_forge_search_contract(value)

        value = contract()
        value["partner_step"]["adapter"]["source_sha"] = "f" * 40
        value["partner_step"] = partner.bind_admitted_partner_step(
            value["partner_step"]["adapter"],
            hub_fixture.bind_fixture(),
            step_key="evaluate",
        )
        with self.assertRaisesRegex(
            forge.ForgePartnerContractError, "audited Forge source"
        ):
            forge.canonical_forge_search_contract(value)

        for field, changed in (
            ("name", "forge.other"),
            ("version", 2),
            ("schema_identity", SHA("e")),
        ):
            value = contract()
            value["partner_step"]["adapter"]["protocol"][field] = changed
            value["partner_step"] = partner.bind_admitted_partner_step(
                value["partner_step"]["adapter"],
                hub_fixture.bind_fixture(),
                step_key="evaluate",
            )
            with self.subTest(field=field), self.assertRaisesRegex(
                forge.ForgePartnerContractError, "protocol does not match"
            ):
                forge.canonical_forge_search_contract(value)

        value = contract()
        value["partner_step"] = partner.bind_admitted_partner_step(
            {
                **value["partner_step"]["adapter"],
                "hub_component": {
                    **value["partner_step"]["adapter"]["hub_component"],
                    "capability": "tdi.prepare",
                    "capability_contract_version": "1.0.0",
                },
            },
            hub_fixture.bind_fixture(),
            step_key="prepare",
        )
        with self.assertRaisesRegex(
            forge.ForgePartnerContractError, "Hub capability does not match"
        ):
            forge.canonical_forge_search_contract(value)

    def test_forge_source_and_schema_pins_fail_closed(self):
        value = contract()
        value["forge"]["source_sha"] = "f" * 40
        with self.assertRaisesRegex(
            forge.ForgePartnerContractError, "audited Forge source"
        ):
            forge.canonical_forge_search_contract(value)

        for field in (
            "external_domain_schema_version",
            "scientific_domain_schema_version",
        ):
            value = contract()
            value["forge"][field] = True
            with self.subTest(field=field), self.assertRaisesRegex(
                forge.ForgePartnerContractError, "must be the integer 1"
            ):
                forge.canonical_forge_search_contract(value)

    def test_upstream_repository_matches_forge_ascii_owner_name_grammar(self):
        value = contract()
        value["upstream"]["repository"] = "Mémorithm/TDI"
        with self.assertRaisesRegex(
            forge.ForgePartnerContractError, "owner/name syntax"
        ):
            forge.canonical_forge_search_contract(value)

    def test_development_and_validation_sources_must_be_nonempty_and_disjoint(self):
        for field in ("generation_sources", "verification_sources"):
            value = contract()
            value["data_boundary"][field] = []
            with self.subTest(field=field), self.assertRaisesRegex(
                forge.ForgePartnerContractError, "must not be empty"
            ):
                forge.canonical_forge_search_contract(value)

        value = contract()
        value["data_boundary"]["verification_sources"] = ["development-v1"]
        with self.assertRaisesRegex(
            forge.ForgePartnerContractError, "development/validation source overlap"
        ):
            forge.canonical_forge_search_contract(value)

    def test_final_holdout_cannot_leak_into_search_or_verification(self):
        for field in ("generation_sources", "verification_sources"):
            value = contract()
            value["data_boundary"][field].append("confirmatory-v1")
            with self.subTest(field=field), self.assertRaisesRegex(
                forge.ForgePartnerContractError, "final holdout source leaked"
            ):
                forge.canonical_forge_search_contract(value)

    def test_duplicates_and_invalid_objective_direction_are_rejected(self):
        value = contract()
        value["allowed_candidate_dimensions"].append("predicate_schema")
        with self.assertRaisesRegex(
            forge.ForgePartnerContractError, "duplicate allowed_candidate_dimensions"
        ):
            forge.canonical_forge_search_contract(value)

        value = contract()
        value["objectives"].append(copy.deepcopy(value["objectives"][0]))
        with self.assertRaisesRegex(
            forge.ForgePartnerContractError, "duplicate objective name"
        ):
            forge.canonical_forge_search_contract(value)

        value = contract()
        value["objectives"][0]["direction"] = "target"
        with self.assertRaisesRegex(
            forge.ForgePartnerContractError, "minimize or maximize"
        ):
            forge.canonical_forge_search_contract(value)

    def test_boolean_type_fidelity_and_authority_flags_are_strict(self):
        value = contract()
        value["schema"] = True
        with self.assertRaisesRegex(
            forge.ForgePartnerContractError, "must be the integer 1"
        ):
            forge.canonical_forge_search_contract(value)

        value = contract()
        value["environment"]["fingerprint_required"] = 1
        with self.assertRaisesRegex(
            forge.ForgePartnerContractError, "must be Boolean"
        ):
            forge.canonical_forge_search_contract(value)

        for field in value["permissions"]:
            changed = contract()
            changed["permissions"][field] = True
            with self.subTest(field=field), self.assertRaisesRegex(
                forge.ForgePartnerContractError, "grants no execution"
            ):
                forge.canonical_forge_search_contract(changed)

    def test_embedded_common_binding_is_revalidated(self):
        value = contract()
        value["partner_step"]["partner_adapter_identity"] = SHA("f")
        with self.assertRaisesRegex(
            forge.ForgePartnerContractError, "does not match embedded evidence"
        ):
            forge.canonical_forge_search_contract(value)

    def test_forge_manifest_is_output_only_and_does_not_promote_authority(self):
        value = contract()
        manifest = forge.compile_forge_scientific_domain_manifest(value)
        self.assertNotIn("permissions", manifest)
        canonical = forge.canonical_forge_search_contract(value)
        self.assertFalse(canonical["permissions"]["forge_execution_qualified"])
        self.assertFalse(canonical["partner_step"]["partner_execution_qualified"])


if __name__ == "__main__":
    unittest.main()
