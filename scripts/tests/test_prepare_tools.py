#!/usr/bin/env python3
"""PLAN-1060 T-26: tool preparation reuses only a provable identity.

The receipt must MISS on every identity axis — source state, toolchain, tool
config, artifact bytes — and HIT only when all of them match, so a warm-tree
invocation stops repeating the same 12 no-op release builds while a real change
still rebuilds.
"""
import hashlib
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "test-governance"))
import prepare_tools as tools


class FakeResult:
    def __init__(self, returncode=0):
        self.returncode = returncode
        self.stdout = b"fake build\n"
        self.stderr = b""
        self.measurement = {"wall_ms": 1.0, "workload_cpu_ms": 1.0, "meter_cpu_ms": 0.1,
                            "peak_job_bytes": 1, "job_processes": 1}


class PrepareReceiptTests(unittest.TestCase):

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.binary = self.root / 'target' / 'fake' / 'debug' / ('fake-bin' + ('.exe' if os.name == 'nt' else ''))
        self.binary.parent.mkdir(parents=True)
        self.binary.write_bytes(b'v1')
        governance = self.root / 'scripts' / 'test-governance'
        governance.mkdir(parents=True)
        (governance / 'toolchains.json').write_text(json.dumps({
            'schema': 'test-module-tools/1',
            'tools': {'fake': {'package': 'fake', 'profile': 'debug', 'target': 'target/fake',
                               'binaries': ['fake-bin'], 'workspace': '.', 'env': 'FAKE_BIN'}},
        }), encoding='utf-8')
        self.calls = []
        self.original_capture = tools.capture
        self.original_marker = tools.toolchain_marker

        def fake_capture(argv, **kwargs):
            self.calls.append(argv)
            # a real build (re)writes the declared artifact
            self.binary.parent.mkdir(parents=True, exist_ok=True)
            self.binary.write_bytes(b'built')
            return FakeResult()

        tools.capture = fake_capture
        tools.toolchain_marker = lambda: 'rustc-marker-1'
        self.addCleanup(setattr, tools, 'capture', self.original_capture)
        self.addCleanup(setattr, tools, 'toolchain_marker', self.original_marker)

    def prepare(self, source_state='state-a'):
        env, receipts = tools.prepare(self.root, ['fake'], self.root / 'out', set(), source_state)
        return env, receipts

    def test_first_preparation_builds_and_writes_a_receipt(self):
        env, receipts = self.prepare()
        self.assertEqual(1, len(self.calls))
        self.assertEqual(str(self.binary), env['FAKE_BIN'])
        receipt = self.root / 'target' / 'fake' / tools.RECEIPT_DIR / 'fake.json'
        self.assertTrue(receipt.is_file())
        self.assertEqual('test-tool-receipt/1', json.loads(receipt.read_text())['schema'])

    def test_same_identity_is_reused_without_a_build(self):
        self.prepare()
        env, receipts = self.prepare()
        self.assertEqual(1, len(self.calls), 'second invocation must not rebuild')
        self.assertEqual(str(self.binary), env['FAKE_BIN'])
        self.assertTrue(any(row.get('reused') for row in receipts))

    def test_changed_source_state_rebuilds(self):
        self.prepare('state-a')
        self.prepare('state-b')
        self.assertEqual(2, len(self.calls))

    def test_changed_toolchain_marker_rebuilds(self):
        self.prepare()
        tools.toolchain_marker = lambda: 'rustc-marker-2'
        self.prepare()
        self.assertEqual(2, len(self.calls))

    def test_changed_tool_config_rebuilds(self):
        self.prepare()
        document = json.loads((self.root / 'scripts/test-governance/toolchains.json').read_text())
        document['tools']['fake']['features'] = 'extra'
        (self.root / 'scripts/test-governance/toolchains.json').write_text(json.dumps(document))
        self.prepare()
        self.assertEqual(2, len(self.calls))

    def test_tampered_artifact_rebuilds(self):
        self.prepare()
        self.binary.write_bytes(b'v2-tampered')
        self.prepare()
        self.assertEqual(2, len(self.calls))

    def test_missing_artifact_rebuilds(self):
        self.prepare()
        self.binary.unlink()
        _, receipts = self.prepare()
        self.assertEqual(2, len(self.calls))
        self.assertTrue(self.binary.is_file(), 'the fake build must restore the artifact')
        self.assertFalse(any(row.get('reused') for row in receipts))

    def test_artifact_hashes_are_recorded_for_reuse(self):
        _, receipts = self.prepare()
        artifact_rows = [row for row in receipts if 'sha256' in row and 'path' in row]
        self.assertTrue(artifact_rows)
        expected = hashlib.sha256(self.binary.read_bytes()).hexdigest()
        self.assertIn(expected, [row['sha256'] for row in artifact_rows])


if __name__ == '__main__':
    unittest.main()