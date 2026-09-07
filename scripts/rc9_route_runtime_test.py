"""The receipt producer rejects successful Cargo invocations that ran the wrong test."""

import copy
import json
import subprocess
import sys
import unittest

sys.dont_write_bytecode = True

from rc9_route_runtime_common import FULL_NAME, input_paths, parse, parse_execution, policy


class RuntimeProducer(unittest.TestCase):
    def test_full_frozen_tree_is_explicitly_bound(self):
        mirror = policy()
        self.assertEqual(len(input_paths(mirror)), 826)
        self.assertIn("Cargo.lock", mirror["inputs"])
        self.assertIn("src/request/bornera_test.rs", mirror["inputs"])

    def test_exact_execution_and_closed_json(self):
        artifact = {"reason": "compiler-artifact", "executable": "/unused/test-binary",
                    "target": {"name": "kafka_driver", "kind": ["lib"]}, "profile": {"test": True}}
        output = (json.dumps(artifact) + "\n" + f"test {FULL_NAME} ... ok\n"
                  + "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 468 filtered out; finished in 0.00s\n")
        result = subprocess.CompletedProcess([], 0, output.encode(), b"")
        self.assertEqual(parse_execution(result)[1]["passed"], 1)
        for payload in [output.replace(FULL_NAME, "unrelated"), output.replace("... ok", "... ignored"),
                        output.replace("1 passed", "0 passed"), output.replace("0 ignored", "1 ignored"),
                        output + f"test {FULL_NAME} ... ok\n", output + json.dumps(artifact) + "\n",
                        output.replace('"kafka_driver"', '"other"'), output.replace("... ok", "... FAILED")]:
            changed = copy.copy(result)
            changed.stdout = payload.encode()
            with self.assertRaises(ValueError):
                parse_execution(changed)
        result.returncode = 1
        with self.assertRaises(ValueError):
            parse_execution(result)
        with self.assertRaises(ValueError):
            parse('{"schema":1,"schema":1}')


if __name__ == "__main__":
    unittest.main()
