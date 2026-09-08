"""Traversal execution scope must include exact permission and frozen qualification runs."""

import sys
import unittest

sys.dont_write_bytecode = True

from rc9_traversal_common import PREFIX, ROOTS, SUITES, execution


class TraversalRunner(unittest.TestCase):
    def test_closed_nonoverlapping_roots_and_all_required_suites(self):
        self.assertEqual(len(ROOTS), len(set(ROOTS)))
        self.assertEqual(len(SUITES), 7)
        self.assertFalse(any(a != b and a.startswith(b + "/") for a in ROOTS for b in ROOTS))
        self.assertEqual([row[2] for row in SUITES.values()], [0, 42, 66, 19, 0, 96, 223])
        self.assertEqual(sum(row[0] for row in SUITES.values()), 2)

    def test_each_required_test_must_actually_pass_once(self):
        for short in SUITES:
            name = PREFIX + short
            output = (f"\nrunning 1 test\ntest {name} ... ok\n\n"
                      "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 628 filtered out; finished in 0.10s\n").encode()
            self.assertEqual(execution(output, 0, name, 629)["passed"], 1)
            for value in [output.replace(b"... ok", b"... ignored"), output.replace(name.encode(), b"unrelated"),
                          output.replace(b"1 passed", b"0 passed"), output.replace(b"0 failed", b"1 failed"),
                          output.replace(b"628 filtered", b"627 filtered"), output + output]:
                with self.assertRaises(ValueError):
                    execution(value, 0, name, 629)


if __name__ == "__main__":
    unittest.main()
