import copy
import unittest

import tdi_partner_adapter_contract as partner
import test_tdi_hub_admission_contract as hub_fixture

SHA = lambda c: c * 64
SOURCE = "a" * 40

PROFILES = {
    "elasticxxx": ("Memorithm/ElasticXxx", "resource-control"),
    "forge": ("Memorithm/Forge", "candidate-search"),
    "scirust": ("Memorithm/scirust", "math-primitives"),
    "flat-attention": ("Memorithm/FLAT-ATTENTION", "attention-execution"),
    "nnis": ("Memorithm/NNIS", "hardware-qualification"),
}


def descriptor(kind="forge"):
    repository, role = PROFILES[kind]
    return {
        "schema": 1,
        "partner": kind,
        "repository": repository,
        "source_sha": SOURCE,
        "role": role,
        "protocol": {
            "name": "tdi.partner.fixture",
            "version": 1,
            "schema_identity": SHA("d"),
        },
        "hub_component": {
            "component_id": hub_fixture.COMPONENT,
            "component_version": "1.2.3",
            "manifest_digest": SHA("9"),
            "capability": "tdi.evaluate",
            "capability_contract_version": "1.1.0",
        },
        "capabilities": ["result.read", "candidate.evaluate"],
        "inputs": [
            {"name": "request", "schema": 1, "identity": SHA("1")},
            {"name": "context", "schema": 2, "identity": SHA("2")},
        ],
        "outputs": [
            {"name": "response", "schema": 1, "identity": SHA("3")},
        ],
        "permissions": {
            "read_protected_holdout": False,
            "authorize_scientific_stage": False,
            "publish_scientific_verdict": False,
            "actuate_runtime": False,
        },
    }


class PartnerAdapterContractTests(unittest.TestCase):
    def test_all_declared_partners_are_canonical_and_authority_free(self):
        for kind, (repository, role) in PROFILES.items():
            with self.subTest(partner=kind):
                value = partner.canonical_partner_adapter(descriptor(kind))
                self.assertEqual(repository, value["repository"])
                self.assertEqual(role, value["role"])
                self.assertEqual(["candidate.evaluate", "result.read"], value["capabilities"])
                self.assertEqual(["context", "request"], [item["name"] for item in value["inputs"]])
                self.assertFalse(any(value["permissions"].values()))
                partner.partner_adapter_identity(value)

    def test_partner_repository_role_and_source_are_fail_closed(self):
        cases = (
            ("repository", "Memorithm/Other", "repository"),
            ("role", "scheduler", "role"),
            ("source_sha", "A" * 40, "source_sha"),
        )
        for field, changed, message in cases:
            value = descriptor()
            value[field] = changed
            with self.subTest(field=field), self.assertRaisesRegex(
                partner.PartnerAdapterContractError, message
            ):
                partner.canonical_partner_adapter(value)

    def test_schema_versions_require_actual_integers(self):
        value = descriptor()
        value["schema"] = True
        with self.assertRaisesRegex(partner.PartnerAdapterContractError, "partner adapter schema"):
            partner.canonical_partner_adapter(value)

        binding = partner.bind_admitted_partner_step(
            descriptor(), hub_fixture.bind_fixture(), step_key="evaluate"
        )
        binding["schema"] = True
        with self.assertRaisesRegex(partner.PartnerAdapterContractError, "admitted partner step schema"):
            partner.canonical_admitted_partner_step(binding)

    def test_common_contract_cannot_grant_or_coerce_any_authority(self):
        for field in descriptor()["permissions"]:
            value = descriptor()
            value["permissions"][field] = True
            with self.subTest(field=field), self.assertRaisesRegex(
                partner.PartnerAdapterContractError, "grants no authority"
            ):
                partner.canonical_partner_adapter(value)

        value = descriptor()
        value["permissions"]["actuate_runtime"] = 0
        with self.assertRaisesRegex(partner.PartnerAdapterContractError, "grants no authority"):
            partner.canonical_partner_adapter(value)

    def test_capability_and_contract_sets_are_unique_and_bounded(self):
        value = descriptor()
        value["capabilities"].append("candidate.evaluate")
        with self.assertRaisesRegex(partner.PartnerAdapterContractError, "duplicate capability"):
            partner.canonical_partner_adapter(value)

        value = descriptor()
        value["inputs"].append(copy.deepcopy(value["inputs"][0]))
        with self.assertRaisesRegex(partner.PartnerAdapterContractError, "duplicate inputs name"):
            partner.canonical_partner_adapter(value)

        value = descriptor()
        value["protocol"]["version"] = True
        with self.assertRaisesRegex(partner.PartnerAdapterContractError, "positive JSON-safe integer"):
            partner.canonical_partner_adapter(value)

    def test_hub_component_uses_graph_v1_uuid_version_digest_and_capability_grammar(self):
        cases = (
            ("component_id", "not-a-uuid", "canonical UUID"),
            ("component_version", "not-a-version", "three numeric components"),
            ("manifest_digest", "A" * 64, "lowercase SHA-256"),
            ("capability", "tdi evaluate", "CapabilityName grammar"),
            ("capability_contract_version", "1", "three numeric components"),
        )
        for field, changed, message in cases:
            value = descriptor()
            value["hub_component"][field] = changed
            with self.subTest(field=field), self.assertRaisesRegex(
                partner.PartnerAdapterContractError, message
            ):
                partner.canonical_partner_adapter(value)

    def test_adapter_binds_only_to_exact_g3_admitted_step(self):
        admitted = hub_fixture.bind_fixture()
        binding = partner.bind_admitted_partner_step(
            descriptor(), admitted, step_key="evaluate"
        )
        self.assertEqual(hub_fixture.WORKFLOW, binding["workflow"])
        self.assertEqual(admitted["graph_identity"], binding["graph_identity"])
        self.assertTrue(binding["workflow_execution_admitted"])
        self.assertTrue(binding["partner_contract_bound"])
        self.assertFalse(binding["partner_execution_qualified"])
        self.assertFalse(binding["protected_holdout_access_authorized"])
        self.assertFalse(binding["scientific_stage_authorized"])
        self.assertFalse(binding["scientific_verdict_authorized"])
        self.assertFalse(binding["runtime_actuation_authorized"])
        self.assertEqual(binding, partner.canonical_admitted_partner_step(binding))
        partner.admitted_partner_step_identity(binding)

    def test_unadmitted_or_component_drift_is_rejected(self):
        admitted = hub_fixture.bind_fixture()
        with self.assertRaisesRegex(partner.PartnerAdapterContractError, "not present|not covered"):
            partner.bind_admitted_partner_step(descriptor(), admitted, step_key="missing")

        mutations = (
            ("component_id", "33333333-3333-3333-3333-333333333333"),
            ("component_version", "1.2.4"),
            ("manifest_digest", SHA("8")),
            ("capability", "tdi.other"),
            ("capability_contract_version", "1.1.1"),
        )
        for field, changed in mutations:
            value = descriptor()
            value["hub_component"][field] = changed
            with self.subTest(field=field), self.assertRaisesRegex(
                partner.PartnerAdapterContractError,
                "does not match admitted Graph/v1 step",
            ):
                partner.bind_admitted_partner_step(value, admitted, step_key="evaluate")

    def test_restored_binding_recomputes_embedded_identities_and_flags(self):
        binding = partner.bind_admitted_partner_step(
            descriptor(), hub_fixture.bind_fixture(), step_key="evaluate"
        )
        for field in (
            "graph_identity",
            "workflow_admission_binding_identity",
            "partner_adapter_identity",
        ):
            changed = copy.deepcopy(binding)
            changed[field] = SHA("f")
            with self.subTest(field=field), self.assertRaisesRegex(
                partner.PartnerAdapterContractError, "does not match embedded evidence"
            ):
                partner.canonical_admitted_partner_step(changed)

        for field in (
            "partner_execution_qualified",
            "protected_holdout_access_authorized",
            "scientific_stage_authorized",
            "scientific_verdict_authorized",
            "runtime_actuation_authorized",
        ):
            changed = copy.deepcopy(binding)
            changed[field] = True
            with self.subTest(field=field), self.assertRaisesRegex(
                partner.PartnerAdapterContractError, "violates the common partner boundary"
            ):
                partner.canonical_admitted_partner_step(changed)

    def test_embedded_g3_version_fields_reject_boolean_aliases(self):
        mutations = (
            (("schema",), "workflow admission binding schema"),
            (("admission_schema_version",), "workflow admission schema version"),
            (("graph", "schema"), "embedded Graph/v1 schema"),
            (("graph", "hub_contract", "workflow_schema_version"), "embedded Graph/v1 Hub workflow schema"),
            (("workflow_spec", "schema_version"), "embedded Hub WorkflowSpec schema"),
            (("admission", "schema_version"), "embedded Hub admission schema"),
        )
        for path, message in mutations:
            admitted = copy.deepcopy(hub_fixture.bind_fixture())
            target = admitted
            for key in path[:-1]:
                target = target[key]
            target[path[-1]] = True
            with self.subTest(path=path), self.assertRaisesRegex(
                partner.PartnerAdapterContractError, message
            ):
                partner.bind_admitted_partner_step(descriptor(), admitted, step_key="evaluate")

    def test_tampered_g3_evidence_is_revalidated(self):
        admitted = hub_fixture.bind_fixture()
        admitted["graph_identity"] = SHA("f")
        with self.assertRaisesRegex(partner.PartnerAdapterContractError, "graph_identity"):
            partner.bind_admitted_partner_step(descriptor(), admitted, step_key="evaluate")


if __name__ == "__main__":
    unittest.main()
