"""PLAN-1066 T-04/NC-08: scheduler document-carrier negative controls.

Each control copies the REAL group into a throwaway directory, mutates ONE
judged axis of the AutoDown document, and runs the REAL case consumer
(`auto-case run`) over the mirror. The mutated case must go red with that axis
named in the console mismatch detail, while an unmutated mirror of the same
directory stays green 11/11 — a restored or weakened judge cannot pass.

They run via the explicit ACE group `plan1066-s-doc-controls`, whose declared
tools (case-parent + av) build the auto-case CLI and the av worker source-bound
before execution. A missing prepared tool fails the group instead of silently
skipping coverage.
"""
import json
import os
import re
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GROUP = ROOT / 'crates/auto-lang/test/cases/ace-m2/scheduler'
CLI = ROOT / 'target/debug/auto-case.exe'
WORKER = Path(os.environ.get('AC_AV_CASE_WORKER',
                             ROOT / 'experimental/ace/target/release/av-worker.exe'))
SUITE = ('```test-suite\n'
         'suite { schema: "autolang.suite/2", id: "ace-m2-scheduler-968", include: [\n'
         '  "scheduler.document.case.ad",\n] }\n```\n')
CASE_IDS = 11


class SchedulerDocumentControls(unittest.TestCase):
    def setUp(self):
        if os.name != 'nt':
            self.skipTest('Windows-only corpus (av worker console recipes)')
        self.assertTrue(CLI.is_file(), 'auto-case CLI was not prepared (case-parent tool)')
        self.assertTrue(WORKER.is_file(), 'av worker was not prepared (av tool)')
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)

    # -- fixtures ---------------------------------------------------------
    def mirror(self, name, mutate=None):
        target = self.base / name
        if target.exists():
            shutil.rmtree(target)
        shutil.copytree(GROUP, target)
        (target / 'suite.ad').write_text(SUITE, encoding='utf-8', newline='\n')
        doc = target / 'scheduler.document.case.ad'
        if mutate is not None:
            text = doc.read_text(encoding='utf-8')
            mutated = mutate(text)
            self.assertNotEqual(mutated, text, 'control did not change the document')
            doc.write_text(mutated, encoding='utf-8', newline='\n')
        return target

    def run_suite(self, directory):
        out = self.base / 'out' / directory.name
        if out.exists():
            shutil.rmtree(out)
        out.parent.mkdir(parents=True, exist_ok=True)
        env = dict(os.environ, AC_AV_CASE_WORKER=str(WORKER))
        result = subprocess.run(
            [str(CLI), 'run', '--suite', str(directory / 'suite.ad'), '--out', str(out)],
            cwd=str(ROOT), env=env, capture_output=True, text=True, timeout=60)
        report = json.loads((out / 'report.json').read_text(encoding='utf-8'))
        console = (out / 'report.console.txt').read_text(encoding='utf-8')
        return report, console, result.returncode

    @staticmethod
    def console_block(console, case):
        lines = console.splitlines()
        for index, line in enumerate(lines):
            if case in line and re.match(r'^(PASS|FAIL|ERROR)\s', line):
                block = [line]
                for nxt in lines[index + 1:]:
                    if re.match(r'^(PASS|FAIL|ERROR)\s', nxt) or nxt.startswith('total '):
                        break
                    block.append(nxt)
                return '\n'.join(block)
        return ''

    def assert_red_axis(self, label, mutation, case, axis_marker):
        directory = self.mirror(label, mutation)
        report, console, _ = self.run_suite(directory)
        self.assertEqual(report['load_errors'], [], console)
        row = next(r for r in report['results'] if r['case'] == case)
        block = self.console_block(console, case)
        self.assertEqual(row['status'], 'FAIL', 'mutated case stayed green: ' + label)
        self.assertIn(axis_marker, block,
                      'red without the %r axis: %s\n%s' % (axis_marker, label, block))
        others = [r for r in report['results'] if r['case'] != case]
        self.assertEqual(len(others), CASE_IDS - 1, label)
        self.assertTrue(all(r['status'] == 'PASS' for r in others),
                        'mutation leaked past its case: ' + label)

    # -- controls ---------------------------------------------------------
    def test_clean_mirror_is_green_and_byte_identical(self):
        directory = self.mirror('clean')
        self.assertEqual((directory / 'scheduler.document.case.ad').read_bytes(),
                         (GROUP / 'scheduler.document.case.ad').read_bytes())
        report, console, code = self.run_suite(directory)
        self.assertEqual(code, 0, console)
        self.assertEqual(report['summary'], {'total': CASE_IDS, 'PASS': CASE_IDS}, console)
        self.assertEqual(report['load_errors'], [])
        self.assertEqual(report['coverage_gaps'], [])

    def test_stdout_final_lf_is_judged_byte_exactly(self):
        self.assert_red_axis(
            'stdout-lf',
            lambda text: text.replace(
                '#### stdout\n\n```text\ngot one\n```',
                '#### stdout\nfinal-newline: false\n\n```text\ngot one\n```', 1),
            'ace.m2.p968.message-handler', 'stdout[bytes] mismatch')

    def test_stderr_oracle_is_not_ignored(self):
        self.assert_red_axis(
            'stderr',
            lambda text: text.replace(
                '#### stderr\n\n```text\n```\n',
                '#### stderr\n\n```text\nnoise\n```\n', 1),
            'ace.m2.p968.message-handler', 'stderr[bytes] mismatch')

    def test_exit_oracle_is_judged(self):
        self.assert_red_axis(
            'exit',
            lambda text: text.replace('entry: "d_main"\nexit: 0',
                                      'entry: "d_main"\nexit: 1', 1),
            'ace.m2.p968.message-handler', 'exit[exit] mismatch')

    def test_resolve_diagnostic_code_is_judged(self):
        self.assert_red_axis(
            'code',
            lambda text: text.replace('code: "source.sched-task"',
                                      'code: "source.sched-taskX"', 1),
            'ace.m2.p968.neg-unknown-task', 'code[code] mismatch')

    def test_message_contains_oracle_is_judged(self):
        directory = self.mirror(
            'message',
            lambda text: text.replace('message-contains: "send on a closed task"',
                                      'message-contains: "definitely-not-present"', 1))
        report, console, _ = self.run_suite(directory)
        row = next(r for r in report['results']
                   if r['case'] == 'ace.m2.p968.neg-send-after-stop')
        self.assertEqual(row['status'], 'FAIL', console)
        block = self.console_block(console, 'ace.m2.p968.neg-send-after-stop')
        self.assertRegex(block, r'message\[[a-z-]+\] mismatch')


if __name__ == '__main__':
    unittest.main()
