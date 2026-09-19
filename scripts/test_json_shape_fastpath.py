"""Differential acceptance tests against the original iterative JSON shape walk."""
import itertools
import json
import random
import unittest

import tdi_experiment_supervisor as supervisor


def reference(value, max_depth, max_items, max_string_bytes):
    stack, seen = [(value, 0)], 0
    while stack:
        current, depth = stack.pop()
        if depth > max_depth:
            raise supervisor.ContractError("depth")
        seen += 1
        if seen > max_items:
            raise supervisor.ContractError("items")
        if isinstance(current, str):
            if len(current.encode("utf-8")) > max_string_bytes:
                raise supervisor.ContractError("string")
        elif isinstance(current, dict):
            stack.extend((key, depth + 1) for key in current)
            stack.extend((item, depth + 1) for item in current.values())
        elif isinstance(current, list):
            stack.extend((item, depth + 1) for item in current)


class ShapeTests(unittest.TestCase):
    def test_differential_limits(self):
        rng = random.Random(19092026)
        def value(depth):
            if depth == 0:
                return rng.choice([None, True, 3, 1.5, "", "é", "abc"])
            if rng.randrange(2):
                return {f"é{i}": value(depth-1) for i in range(rng.randrange(4))}
            return [value(depth-1) for _ in range(rng.randrange(4))]
        cases = [{}, [], {"": []}, {"é": {"abc": [True, "é"]}}]
        cases.extend(value(4) for _ in range(100))
        for case, depth, items, size in itertools.product(cases, (0, 1, 3, 9), (0, 1, 4, 20, 1000), (0, 1, 2, 8)):
            outcomes = []
            for fn in (reference, supervisor._validate_json_shape):
                try:
                    fn(case, depth, items, size)
                    outcomes.append(True)
                except supervisor.ContractError:
                    outcomes.append(False)
            self.assertEqual(outcomes[0], outcomes[1], (case, depth, items, size))

    def test_keys_count_and_unicode_bytes(self):
        for raw, limits in ((b'{"a":0}', {"max_items": 2}),
                            ('{"é":0}', {"max_string_bytes": 1}),
                            (b'{"a":{}}', {"max_depth": 0})):
            with self.assertRaises(supervisor.ContractError):
                supervisor.strict_json(raw, **limits)
        self.assertEqual(supervisor.strict_json('{}', max_depth=0, max_items=1), {})
        self.assertEqual(supervisor.strict_json('{"é":0}', max_string_bytes=2, max_items=3), {"é": 0})

    def test_decoder_guards_remain(self):
        for raw in ('{"a":1,"a":2}', 'NaN', '1e999', b'"\xff"'):
            with self.assertRaises(supervisor.ContractError):
                supervisor.strict_json(raw)
        value = {"a": [1, 2, {"b": "é"}]}
        self.assertEqual(supervisor.strict_json(json.dumps(value)), value)


if __name__ == "__main__":
    unittest.main()
