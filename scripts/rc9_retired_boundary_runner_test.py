"""No ignored, unrelated, partial or failed execution can qualify a boundary suite."""

from pathlib import Path
import sys
import unittest

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
from rc9_retired_boundary_common import execution, parse, SUITES


class Runner(unittest.TestCase):
    def test_exact_execution_and_closed_suite_counts(self):
        output = b"running 1 test\ntest exact::test ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.02s\n"
        self.assertEqual(execution(output, 0, "exact::test", 10)["passed"], 1)
        for bad in [output.replace(b"... ok", b"... ignored"), output.replace(b"exact::test", b"wrong::test"),
                    output.replace(b"1 passed", b"0 passed"), output.replace(b"0 ignored", b"1 ignored"),
                    output.replace(b"9 filtered", b"8 filtered"), output + output]:
            with self.assertRaises(ValueError):
                execution(bad, 0, "exact::test", 10)
        with self.assertRaises(ValueError):
            execution(output, 1, "exact::test", 10)
        self.assertEqual(sum(cases for _, scope, cases in SUITES.values() if scope in
                             {"physical", "native-only-cycle", "real-permissions"}), 147)
        self.assertEqual(sum(cases for _, scope, cases in SUITES.values() if scope.startswith("injected")), 84)

    def test_duplicate_json_fields_are_not_accepted(self):
        with self.assertRaises(ValueError):
            parse(b'{"verified":false,"verified":true}')


if __name__ == "__main__":
    unittest.main()
