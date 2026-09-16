import unittest

import tdi_elastic_partner_contract as elastic
import tdi_hub_admission_contract as admission
import tdi_hub_edge_contract as hub
import tdi_partner_adapter_contract as partner
import test_tdi_hub_admission_contract as hub_fixture
import test_tdi_partner_adapter_contract as partner_fixture

SHA = lambda c: c * 64


def config_descriptor(*, access_class="development", raw_sha256=None, size_bytes=123):
    return {
        "schema": 1,
        "name": "elastic-operator-config-v1.json",
        "raw_sha256": raw_sha256 or SHA("1"),
        "size_bytes": size_bytes,
        "media_type": "application/json",
        "access_class": access_class,
    }


def admitted_elastic_step(config=None):
    config = config or config_descriptor()
    descriptor = partner_fixture.descriptor("elasticxxx")
    descriptor["source_sha"] = elastic.ELASTIC_SOURCE_SHA
    descriptor.update(elastic.elastic_adapter_surface())
    descriptor["protocol"] = {
        "name": elastic.ELASTIC_PROTOCOL_NAME,
        "version": elastic.ELASTIC_PROTOCOL_VERSION,
        "schema_identity": elastic.ELASTIC_PROTOCOL_SCHEMA_IDENTITY,
    }
    descriptor["hub_component"] = {
        **descriptor["hub_component"],
        "capability": elastic.ELASTIC_HUB_CAPABILITY,
        "capability_contract_version": elastic.ELASTIC_HUB_CAPABILITY_CONTRACT_VERSION,
    }
    portable = hub.bind_portable_artifact(
        config,
        {
            "id": hub_fixture.INPUT_ARTIFACT,
            "hub_digest": "c" * 64,
            "raw_sha256": config["raw_sha256"],
            "size": config["size_bytes"],
        },
    )
    admitted = admission.bind_exact_workflow_admission(
        hub_fixture.fixture_graph(),
        hub_fixture.workflow_response(),
        root_artifact_bindings={config["raw_sha256"]: portable},
    )
    return partner.bind_admitted_partner_step(descriptor, admitted, step_key="prepare")


def contract(*, domain="Development", access_class="development"):
    config = config_descriptor(access_class=access_class)
    return {
        "schema": 1,
        "partner_step": admitted_elastic_step(config),
        "elastic": {
            "repository": elastic.ELASTIC_REPOSITORY,
            "source_sha": elastic.ELASTIC_SOURCE_SHA,
            "hub_run_protocol_version": elastic.ELASTIC_PROTOCOL_SEMVER,
            "operator_config_schema_version": 1,
            "runtime_evidence_schema": elastic.ELASTIC_EVIDENCE_SCHEMA,
            "runtime_evidence_media_type": elastic.ELASTIC_EVIDENCE_MEDIA_TYPE,
            "runtime_evidence_source_command": elastic.ELASTIC_EVIDENCE_SOURCE_COMMAND,
        },
        "request": {
            "domain": domain,
            "config_artifact": config,
            "requested_mode": "dry-run",
            "resource_id": "ram-budget",
        },
        "expected_evidence": {
            "schema": elastic.ELASTIC_EVIDENCE_SCHEMA,
            "media_type": elastic.ELASTIC_EVIDENCE_MEDIA_TYPE,
            "source_command": elastic.ELASTIC_EVIDENCE_SOURCE_COMMAND,
        },
        "permissions": {
            "elastic_execution_qualified": False,
            "physical_actuation_authorized": False,
            "protected_holdout_access_authorized": False,
            "scientific_stage_authorized": False,
            "scientific_verdict_authorized": False,
            "runtime_actuation_authorized": False,
        },
    }


class ElasticPartnerContractTests(unittest.TestCase):
    def test_non_scalar_domain_and_mode_are_typed_contract_rejections(self):
        for field in ("domain", "requested_mode"):
            for invalid in ([], {}, None, True, 1):
                value = contract()
                value["request"][field] = invalid
                with self.subTest(field=field, invalid=invalid), self.assertRaises(elastic.ElasticPartnerContractError):
                    elastic.canonical_elastic_resource_contract(value)

    def test_exact_capabilities_and_io_schema_pins_are_required(self):
        mutations = [
            ("capabilities", []), ("capabilities", ["runtime.apply"]),
            ("capabilities", ["candidate.evaluate"]), ("inputs", []), ("outputs", []),
            ("inputs", [{"name": "operator-config", "schema": 1, "identity": SHA("f")}]),
            ("outputs", [{"name": "runtime-evidence", "schema": 2, "identity": elastic.ELASTIC_EVIDENCE_SCHEMA_IDENTITY}]),
        ]
        for field, replacement in mutations:
            value = contract()
            descriptor = value["partner_step"]["adapter"]
            descriptor[field] = replacement
            with self.subTest(field=field), self.assertRaises((elastic.ElasticPartnerContractError, partner.PartnerAdapterContractError)):
                value["partner_step"] = partner.bind_admitted_partner_step(
                    descriptor, value["partner_step"]["workflow_admission_binding"], step_key="prepare"
                )
                elastic.canonical_elastic_resource_contract(value)

    def test_valid_dry_run_contract_compiles_non_executing_interchange(self):
        value = elastic.canonical_elastic_resource_contract(contract())
        self.assertEqual("elasticxxx", value["partner_step"]["adapter"]["partner"])
        self.assertEqual(elastic.ELASTIC_SOURCE_SHA, value["elastic"]["source_sha"])
        self.assertFalse(any(value["permissions"].values()))
        request = elastic.compile_elastic_hub_run_request(value)
        self.assertEqual("elastic.hub.run", request["protocol"])
        self.assertEqual("1.0.0", request["protocol_version"])
        self.assertEqual("dry-run", request["operator_config"]["declared_mode"])
        self.assertEqual("ram-budget", request["resource_id"])
        self.assertTrue(request["independent_elastic_validation_required"])
        self.assertFalse(request["execution_qualified"])
        self.assertFalse(request["physical_actuation_authorized"])
        elastic.elastic_resource_contract_identity(value)

    def test_only_non_actuating_modes_are_accepted(self):
        for mode in ("observe-only", "plan-only", "dry-run"):
            value = contract()
            value["request"]["requested_mode"] = mode
            with self.subTest(mode=mode):
                self.assertEqual(mode, elastic.canonical_elastic_resource_contract(value)["request"]["requested_mode"])
        value = contract()
        value["request"]["requested_mode"] = "apply"
        with self.assertRaisesRegex(elastic.ElasticPartnerContractError, "apply is not authorized"):
            elastic.canonical_elastic_resource_contract(value)

    def test_contract_requires_exact_elastic_partner_source_protocol_and_prepare_capability(self):
        value = contract()
        value["partner_step"]["adapter"]["partner"] = "forge"
        with self.assertRaises(elastic.ElasticPartnerContractError):
            elastic.canonical_elastic_resource_contract(value)

        value = contract()
        value["partner_step"]["adapter"]["source_sha"] = "f" * 40
        value["partner_step"] = partner.bind_admitted_partner_step(
            value["partner_step"]["adapter"], value["partner_step"]["workflow_admission_binding"], step_key="prepare"
        )
        with self.assertRaisesRegex(elastic.ElasticPartnerContractError, "audited source"):
            elastic.canonical_elastic_resource_contract(value)

        for field, changed in (("name", "elastic.other"), ("version", 2), ("schema_identity", SHA("f"))):
            value = contract()
            value["partner_step"]["adapter"]["protocol"][field] = changed
            value["partner_step"] = partner.bind_admitted_partner_step(
                value["partner_step"]["adapter"], value["partner_step"]["workflow_admission_binding"], step_key="prepare"
            )
            with self.subTest(field=field), self.assertRaisesRegex(elastic.ElasticPartnerContractError, "protocol does not match"):
                elastic.canonical_elastic_resource_contract(value)

        value = contract()
        descriptor = value["partner_step"]["adapter"]
        descriptor["hub_component"] = {**descriptor["hub_component"], "capability": "tdi.evaluate", "capability_contract_version": "1.1.0"}
        value["partner_step"] = partner.bind_admitted_partner_step(
            descriptor, value["partner_step"]["workflow_admission_binding"], step_key="evaluate"
        )
        with self.assertRaisesRegex(elastic.ElasticPartnerContractError, "preparation boundary"):
            elastic.canonical_elastic_resource_contract(value)

    def test_elastic_source_and_wire_contract_pins_fail_closed(self):
        mutations = (
            ("repository", "Memorithm/Other", "does not match ElasticXxx"),
            ("source_sha", "f" * 40, "audited ElasticXxx source"),
            ("hub_run_protocol_version", "2.0.0", "protocol version"),
            ("runtime_evidence_schema", "other", "evidence schema drift"),
            ("runtime_evidence_media_type", "application/json", "media type drift"),
            ("runtime_evidence_source_command", "hub-run", "source command drift"),
        )
        for field, changed, message in mutations:
            value = contract()
            value["elastic"][field] = changed
            with self.subTest(field=field), self.assertRaisesRegex(elastic.ElasticPartnerContractError, message):
                elastic.canonical_elastic_resource_contract(value)
        value = contract()
        value["elastic"]["operator_config_schema_version"] = True
        with self.assertRaisesRegex(elastic.ElasticPartnerContractError, "must be the integer 1"):
            elastic.canonical_elastic_resource_contract(value)

    def test_config_artifact_is_exact_domain_json_and_schema_type_faithful(self):
        value = contract()
        value["request"]["config_artifact"]["schema"] = True
        with self.assertRaisesRegex(elastic.ElasticPartnerContractError, "artifact schema"):
            elastic.canonical_elastic_resource_contract(value)
        value = contract()
        value["request"]["config_artifact"]["media_type"] = "text/plain"
        with self.assertRaisesRegex(elastic.ElasticPartnerContractError, "media_type"):
            elastic.canonical_elastic_resource_contract(value)
        value = contract()
        value["request"]["domain"] = "Validation"
        with self.assertRaisesRegex(elastic.ElasticPartnerContractError, "access_class must match"):
            elastic.canonical_elastic_resource_contract(value)
        value = contract(domain="Validation", access_class="validation")
        canonical = elastic.canonical_elastic_resource_contract(value)
        self.assertEqual("Validation", canonical["request"]["domain"])

    def test_operator_config_must_be_exact_admitted_root_consumed_by_partner_step(self):
        value = contract()
        value["request"]["config_artifact"]["raw_sha256"] = SHA("a")
        with self.assertRaisesRegex(elastic.ElasticPartnerContractError, "direct root artifact"):
            elastic.canonical_elastic_resource_contract(value)
        value = contract()
        value["request"]["config_artifact"]["size_bytes"] += 1
        with self.assertRaisesRegex(elastic.ElasticPartnerContractError, "exactly match"):
            elastic.canonical_elastic_resource_contract(value)

    def test_expected_evidence_contract_is_exact(self):
        for field, changed in (("schema", "elastic-runtime-evidence-v2"), ("media_type", "application/json"), ("source_command", "apply")):
            value = contract()
            value["expected_evidence"][field] = changed
            with self.subTest(field=field), self.assertRaisesRegex(elastic.ElasticPartnerContractError, "drift"):
                elastic.canonical_elastic_resource_contract(value)

    def test_authority_flags_are_strict_booleans_and_cannot_be_promoted(self):
        value = contract()
        value["schema"] = True
        with self.assertRaisesRegex(elastic.ElasticPartnerContractError, "must be the integer 1"):
            elastic.canonical_elastic_resource_contract(value)
        for field in contract()["permissions"]:
            value = contract()
            value["permissions"][field] = True
            with self.subTest(field=field), self.assertRaisesRegex(elastic.ElasticPartnerContractError, "grants no execution"):
                elastic.canonical_elastic_resource_contract(value)
        value = contract()
        value["permissions"]["runtime_actuation_authorized"] = 0
        with self.assertRaisesRegex(elastic.ElasticPartnerContractError, "grants no execution"):
            elastic.canonical_elastic_resource_contract(value)

    def test_embedded_common_binding_is_revalidated(self):
        value = contract()
        value["partner_step"]["partner_adapter_identity"] = SHA("f")
        with self.assertRaisesRegex(elastic.ElasticPartnerContractError, "does not match embedded evidence"):
            elastic.canonical_elastic_resource_contract(value)

    def test_resource_id_is_optional_but_bounded_and_control_free(self):
        value = contract()
        value["request"]["resource_id"] = None
        self.assertIsNone(elastic.canonical_elastic_resource_contract(value)["request"]["resource_id"])
        value = contract()
        value["request"]["resource_id"] = "ram\nbudget"
        with self.assertRaisesRegex(elastic.ElasticPartnerContractError, "control character"):
            elastic.canonical_elastic_resource_contract(value)

    def test_compiled_request_carries_no_authority_or_execution_claim(self):
        request = elastic.compile_elastic_hub_run_request(contract())
        self.assertNotIn("permissions", request)
        self.assertFalse(request["execution_qualified"])
        self.assertFalse(request["physical_actuation_authorized"])
        self.assertTrue(request["independent_elastic_validation_required"])


if __name__ == "__main__":
    unittest.main()
