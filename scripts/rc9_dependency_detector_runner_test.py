"""Every required detector suite must execute exactly once, never merely appear in a listing."""

import sys
import unittest

sys.dont_write_bytecode = True

from rc9_dependency_detector_common import PREFIX, SUITES, execution


class DependencyDetectorRunner(unittest.TestCase):
    def test_complete_suite_scope(self):
        self.assertEqual(len(SUITES), 4)
        self.assertEqual([row[2] for row in SUITES.values()], [2, 0, 32, 32])
        self.assertEqual(sum(row[0] for row in SUITES.values()), 1)

    def test_exact_execution_rejects_skips_duplicates_and_wrong_counts(self):
        for short in SUITES:
            name = PREFIX + short
            output = (f"\nrunning 1 test\ntest {name} ... ok\n\n"
                      "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 633 filtered out; finished in 0.10s\n").encode()
            self.assertEqual(execution(output, 0, name, 634)["passed"], 1)
            for value in [output.replace(b"... ok", b"... ignored"), output.replace(name.encode(), b"unrelated"),
                          output.replace(b"1 passed", b"0 passed"), output.replace(b"0 failed", b"1 failed"),
                          output.replace(b"633 filtered", b"632 filtered"), output + output]:
                with self.assertRaises(ValueError):
                    execution(value, 0, name, 634)
            with self.assertRaises(ValueError):
                execution(output, 1, name, 634)


if __name__ == "__main__":
    unittest.main()
