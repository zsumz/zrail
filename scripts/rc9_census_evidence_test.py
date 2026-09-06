"""Adversarial checks for fragment census artifact binding, separate from architecture analysis."""

import copy
import gzip
import hashlib
import json
from pathlib import Path
import runpy
import tempfile
import unittest


ROOT = Path(__file__).resolve().parent.parent
MODULE = runpy.run_path(ROOT / 'scripts/rc9-inventory-check')
VERIFY = MODULE['fragment_inputs']


class FragmentEvidence(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        data = ROOT / 'crates/zrail-testkit/tests/fixtures/rc9'
        cls.registry = json.loads((data / 'additional-rust-inputs.json').read_bytes())
        with gzip.open(data / 'census.json.gz', 'rb') as stream:
            cls.census = json.loads(stream.read(64 * 1024 * 1024 + 1))
        cls.summary = json.loads((data / 'census-summary.json').read_bytes())
        cls.files = {(row['repository'], row['path']): row for row in cls.census['files']}

    def test_all_reviewed_fragment_and_including_source_identities_match(self):
        VERIFY(self.census, self.summary, self.files)

    def test_rebound_registry_cannot_change_frozen_source_or_review_identity(self):
        for mutation in ['fragment-hash', 'commit', 'review-hash', 'duplicate', 'missing',
                         'syntax', 'missing-review', 'duplicate-review', 'reason', 'unknown']:
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                registry = copy.deepcopy(self.registry)
                row = registry['inputs'][0]
                if mutation == 'fragment-hash':
                    row['sha256'] = '0' * 64
                elif mutation == 'commit':
                    row['commit'] = '0' * 40
                elif mutation == 'review-hash':
                    row['review_sources'][0]['sha256'] = '0' * 64
                elif mutation == 'duplicate':
                    registry['inputs'].append(row)
                elif mutation == 'missing':
                    registry['inputs'].pop()
                elif mutation == 'syntax':
                    row['syntax'] = 'expressions'
                elif mutation == 'missing-review':
                    row['review_sources'] = []
                elif mutation == 'duplicate-review':
                    row['review_sources'].append(row['review_sources'][0])
                elif mutation == 'reason':
                    row['reason'] = ' '
                else:
                    row['ignore_unresolved'] = True
                payload = json.dumps(registry).encode()
                Path(temporary, 'additional-rust-inputs.json').write_bytes(payload)
                census = dict(self.census, additional_rust_inputs_sha256=hashlib.sha256(payload).hexdigest())
                summary = dict(self.summary, additional_rust_inputs_sha256=hashlib.sha256(payload).hexdigest())
                original = VERIFY.__globals__['DATA']
                try:
                    VERIFY.__globals__['DATA'] = Path(temporary)
                    with self.assertRaises(ValueError):
                        VERIFY(census, summary, self.files)
                finally:
                    VERIFY.__globals__['DATA'] = original


if __name__ == '__main__':
    unittest.main()
