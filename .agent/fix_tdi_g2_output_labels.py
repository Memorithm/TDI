from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count == 0 and new in text:
        return text
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


source = Path("scripts/tdi_hub_edge_contract.py")
s = source.read_text()
s = replace_once(
    s,
    "import uuid\n",
    "import uuid\nimport unicodedata\n",
    "unicodedata import",
)
s = replace_once(
    s,
    '''def _hub_name(value, name):\n    if not isinstance(value, str) or _HUB_NAME.fullmatch(value) is None:\n        raise HubEdgeContractError(f"{name} must match [a-z0-9][a-z0-9_-]{{0,63}}")\n    return value\n\n\n''',
    '''def _hub_name(value, name):\n    if not isinstance(value, str) or _HUB_NAME.fullmatch(value) is None:\n        raise HubEdgeContractError(f"{name} must match [a-z0-9][a-z0-9_-]{{0,63}}")\n    return value\n\n\ndef _hub_output_name(value, name):\n    if not isinstance(value, str):\n        raise HubEdgeContractError(f"{name} must be a string")\n    size = len(value.encode("utf-8"))\n    if size == 0 or size > 128:\n        raise HubEdgeContractError(f"{name} must be 1..=128 UTF-8 bytes")\n    if any(ch.isspace() or unicodedata.category(ch) == "Cc" for ch in value):\n        raise HubEdgeContractError(f"{name} must not contain whitespace/control characters")\n    return value\n\n\n''',
    "Hub output-name validator",
)
s = replace_once(
    s,
    '        label = _hub_name(name, "Hub publication output label")\n',
    '        label = _hub_output_name(name, "Hub publication output label")\n',
    "publication output validator",
)
s = replace_once(
    s,
    '    expected_output = _hub_name(expected_output, "expected Hub output label")\n',
    '    expected_output = _hub_output_name(expected_output, "expected Hub output label")\n',
    "expected output validator",
)
s = replace_once(
    s,
    '    output_label = _hub_name(binding["output_label"], "Hub publication output label")\n',
    '    output_label = _hub_output_name(binding["output_label"], "Hub publication output label")\n',
    "v2 output-label validator",
)
source.write_text(s)


tests = Path("scripts/test_tdi_hub_edge_contract.py")
t = tests.read_text()
t = replace_once(
    t,
    '        value = self.publication(outputs={"Bad.Label": self.ARTIFACT})\n        with self.assertRaisesRegex(hub.HubEdgeContractError, "must match"):\n            hub.canonical_authoritative_publication_response(value)\n',
    '        value = self.publication(outputs={"bad label": self.ARTIFACT})\n        with self.assertRaisesRegex(hub.HubEdgeContractError, "whitespace/control"):\n            hub.canonical_authoritative_publication_response(value)\n',
    "malformed output-label regression",
)
anchor = '''    def test_publication_lineage_mismatch_fails_closed(self):\n'''
new_test = '''    def test_publication_output_labels_follow_hub_graph_grammar(self):\n        portable = hub.bind_portable_artifact(self.descriptor(), self.response())\n        label = "file:result"\n        publication = self.publication(outputs={label: self.ARTIFACT})\n        canonical = hub.canonical_authoritative_publication_response(publication)\n        self.assertEqual({label: self.ARTIFACT}, canonical["outputs"])\n        binding = hub.bind_authoritative_publication(\n            portable,\n            publication,\n            expected_workflow=self.WORKFLOW,\n            expected_step_key="emit",\n            expected_output=label,\n        )\n        self.assertEqual(label, binding["output_label"])\n        self.assertEqual(binding, hub.canonical_authoritative_hub_artifact_binding(binding))\n\n        longest = "x" * 128\n        accepted = hub.canonical_authoritative_publication_response(\n            self.publication(outputs={longest: self.ARTIFACT})\n        )\n        self.assertIn(longest, accepted["outputs"])\n        with self.assertRaisesRegex(hub.HubEdgeContractError, "1..=128"):\n            hub.canonical_authoritative_publication_response(\n                self.publication(outputs={"x" * 129: self.ARTIFACT})\n            )\n\n'''
t = replace_once(t, anchor, new_test + anchor, "Hub output grammar regression")
tests.write_text(t)
