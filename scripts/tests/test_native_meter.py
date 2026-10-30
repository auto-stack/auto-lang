import os
import json
from pathlib import Path
import sys
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "test-governance"))
from native_meter import api, measure


@unittest.skipUnless(os.name == "nt", "native Windows process-tree meter")
class NativeMeter(unittest.TestCase):
    def test_exited_parent_with_live_child_is_diagnosed_and_reaped(self):
        code = ("import subprocess,sys; p=subprocess.Popen([sys.executable,'-c',"
                "'import time; time.sleep(30)'],stdin=subprocess.DEVNULL,"
                "stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,creationflags=0x08000000);"
                "print(p.pid,flush=True)")
        result = measure([sys.executable, '-c', code], timeout=10)
        self.assertEqual(result['exit'], 0)
        self.assertEqual(result['error'], 'descendants live after workload exit')
        pid = int(result['stdout'].strip())
        self.assertTrue(any(row['pid'] == pid for row in result['live_after_exit']['processes']))
        self.assertLess(result['wall_ms'], 3500)
        kernel = api()
        handle = kernel.OpenProcess(0x1000, False, pid)
        if handle:
            from ctypes import byref
            from ctypes.wintypes import DWORD
            try:
                status = DWORD(259)
                self.assertTrue(kernel.GetExitCodeProcess(handle, byref(status)))
                self.assertNotEqual(status.value, 259, 'reported descendant must actually be reaped')
            finally:
                kernel.CloseHandle(handle)

    def test_nested_job_cpu_remains_in_outer_tree_accounting(self):
        spin = "import time; end=time.process_time()+0.15\nwhile time.process_time()<end: pass"
        library = str(Path(__file__).resolve().parents[1] / "test-governance")
        code = ("import sys,json; sys.path.insert(0," + repr(library) + "); "
                "from native_meter import measure; r=measure([sys.executable,'-c'," + repr(spin) + "]); "
                "print(json.dumps({k:v for k,v in r.items() if k not in ('stdout','stderr')}))")
        outer = measure([sys.executable, "-c", code])
        self.assertIsNone(outer["error"])
        self.assertEqual(outer["exit"], 0, outer["stderr"])
        inner = json.loads(outer["stdout"])
        self.assertIsNone(inner["error"])
        self.assertGreaterEqual(outer["job_processes"], 3)
        self.assertGreaterEqual(outer["workload_cpu_ms"], inner["workload_cpu_ms"]-16)
        self.assertGreaterEqual(outer["workload_cpu_ms"], 140)

    def test_counts_child_cpu_not_only_parent_sleep(self):
        # Keep the signal above interpreter/helper startup variance. The
        # original 200ms signal produced a valid 109ms paired delta under
        # nested Jobs; 600ms strengthens both absolute and paired controls.
        spin = "import time; end=time.process_time()+0.60\nwhile time.process_time()<end: pass"
        parent = "import subprocess,sys; subprocess.run([sys.executable,'-c'," + repr(spin) + "],check=True,creationflags=0x08000000)"
        result = measure([sys.executable, "-c", parent])
        self.assertEqual(result["exit"], 0)
        self.assertIsNone(result["error"])
        # Windows may also account console/runtime helper processes. Preserve
        # the native total rather than pretending every process is a worker.
        self.assertGreaterEqual(result["job_processes"], 2)
        self.assertGreaterEqual(result["workload_cpu_ms"], 580)
        self.assertGreater(result["peak_job_bytes"], 0)
        self.assertGreaterEqual(result["total_cpu_ms"], result["workload_cpu_ms"])
        sleeping = parent.replace(repr(spin), repr("import time; time.sleep(0.60)"))
        control = measure([sys.executable, "-c", sleeping])
        self.assertIsNone(control["error"])
        self.assertGreater(result["workload_cpu_ms"]-control["workload_cpu_ms"], 420,
                           f"spin={result}; sleep={control}")

    def test_deadline_reaps_descendant_holding_pipe(self):
        code = "import subprocess,sys,time; child=subprocess.Popen([sys.executable,'-c','import time; time.sleep(20)'],creationflags=0x08000000); print(child.pid,flush=True); time.sleep(20)"
        result = measure([sys.executable, "-c", code], timeout=0.6)
        self.assertEqual(result["error"], "deadline")
        self.assertNotEqual(result["exit"], 0)
        self.assertGreaterEqual(result["job_processes"], 2)
        self.assertLess(result["wall_ms"], 3000)
        pid = int(result["stdout"].strip())
        kernel = api()
        handle = kernel.OpenProcess(0x1000, False, pid)
        if handle:
            from ctypes import byref
            from ctypes.wintypes import DWORD
            try:
                exit_code = DWORD(259)
                self.assertTrue(kernel.GetExitCodeProcess(handle, byref(exit_code)))
                self.assertNotEqual(exit_code.value, 259, "actual grandchild must be reaped")
            finally:
                kernel.CloseHandle(handle)

    def test_output_flood_fails_instead_of_truncating_success(self):
        result = measure([sys.executable, "-c", "import sys,time; sys.stdout.write('x'*200000); sys.stdout.flush(); time.sleep(5)"],
                         output_cap=1000, timeout=1)
        self.assertEqual(result["error"], "output cap")
        self.assertNotEqual(result["exit"], 0)
        self.assertLessEqual(len(result["stdout"]), 1000)


@unittest.skipUnless(os.name == 'nt', 'native Windows producer containment')
class ProcessCapture(unittest.TestCase):
    def test_preparation_exception_requires_exact_image_hash_and_completed_reap(self):
        import hashlib, tempfile
        from unittest.mock import patch
        from process_capture import capture, compiler_auxiliaries_reaped
        with tempfile.TemporaryDirectory() as directory:
            policy = Path(directory) / 'policy.json'
            pin = {'path': str(Path(sys.executable).resolve()),
                   'sha256': hashlib.sha256(Path(sys.executable).read_bytes()).hexdigest()}
            pins = [pin]
            # Windows can attach its own console host to this Python fixture.
            # Pin its exact bytes in the temporary fixture policy too; the real
            # compiler policy still contains only its reviewed vctip image.
            console = Path(os.environ['SystemRoot']) / 'System32/conhost.exe'
            pins.append({'path': str(console.resolve()),
                         'sha256': hashlib.sha256(console.read_bytes()).hexdigest()})
            policy.write_text(json.dumps({'schema': 'test-compiler-auxiliaries/1', 'images': pins}))
            with patch('process_capture.COMPILER_AUXILIARIES', policy):
                # Default workload policy remains strict even for a pinned image.
                strict = capture([sys.executable, '-c', self.leaky_helper()], timeout=10)
                self.assertNotEqual(strict.returncode, 0)
                accepted = capture([sys.executable, '-c', self.leaky_helper()], timeout=10,
                                   compiler_auxiliary_cleanup=True)
                self.assertEqual(accepted.returncode, 0, accepted.stderr)
                measured = accepted.measurement
                self.assertEqual(measured['error'], 'descendants live after workload exit')
                self.assertTrue(measured['live_after_exit']['cleanup_complete'])
                self.assertEqual(measured['live_after_exit']['remaining_active'], 0)
                self.assertTrue(measured['accepted_compiler_auxiliary_cleanup'])
                import copy
                for axis in ('unreaped', 'unknown', 'incomplete', 'image-error', 'nonzero'):
                    bad = copy.deepcopy(measured)
                    if axis == 'unreaped': bad['live_after_exit']['cleanup_complete'] = False
                    elif axis == 'unknown': bad['live_after_exit']['processes'][0]['image'] = str(policy)
                    elif axis == 'incomplete': bad['live_after_exit']['active'] += 1
                    elif axis == 'image-error': bad['live_after_exit']['processes'][0].pop('image')
                    else: bad['exit'] = 1
                    self.assertFalse(compiler_auxiliaries_reaped(bad), axis)
                pin['sha256'] = '0' * 64
                policy.write_text(json.dumps({'schema': 'test-compiler-auxiliaries/1', 'images': pins}))
                self.assertFalse(compiler_auxiliaries_reaped(measured))

    @staticmethod
    def leaky_helper():
        return ("import subprocess,sys; p=subprocess.Popen([sys.executable,'-c',"
                "'import time; time.sleep(30)'],stdin=subprocess.DEVNULL,"
                "stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,creationflags=0x08000000);"
                "print(p.pid,flush=True)")

    def test_zero_parent_exit_does_not_certify_a_leaking_producer(self):
        from process_capture import capture, checked_output
        from unittest.mock import patch
        import subprocess
        valid = capture([sys.executable, '-c', "print('ready')"], timeout=10)
        self.assertEqual(valid.returncode, 0)
        self.assertEqual(valid.stdout.strip(), b'ready')
        bad = capture([sys.executable, '-c', self.leaky_helper()], timeout=10)
        self.assertEqual(bad.measurement['exit'], 0)
        self.assertNotEqual(bad.returncode, 0)
        self.assertEqual(bad.measurement['error'], 'descendants live after workload exit')
        self.assertIn(b'[process-tree infrastructure]', bad.stderr)
        with patch('process_capture.capture', return_value=bad), self.assertRaises(subprocess.CalledProcessError):
            checked_output(['original-producer'])

    def test_preparation_rejects_leak_despite_existing_artifact_then_accepts_clean_build(self):
        import tempfile
        from unittest.mock import patch
        from process_capture import capture
        from prepare_tools import prepare
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            binary = root / 'tools/release/tool.exe'
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b'fixture artifact: existence alone is not successful preparation')
            document = {'schema': 'test-module-tools/1', 'tools': {'fixture': {
                'workspace': '.', 'target': 'tools', 'package': 'fixture', 'binaries': ['tool'],
                'profile': 'release', 'env': 'FIXTURE_WORKER'}}}
            prepared = set()
            def execute(program):
                def run(argv, **kwargs):
                    self.assertEqual(argv[:3], ['cargo', 'build', '--locked'])
                    return capture([sys.executable, '-c', program], **kwargs)
                return run
            with patch('prepare_tools.load', return_value=document):
                with patch('prepare_tools.capture', side_effect=execute(self.leaky_helper())):
                    with self.assertRaisesRegex(ValueError, 'preparation failed'):
                        prepare(root, ['fixture'], root / 'negative', prepared)
                self.assertFalse(prepared)
                failed = json.loads((root / 'negative/fixture-build.meter.json').read_text())
                self.assertEqual(failed['exit'], 0)
                self.assertEqual(failed['error'], 'descendants live after workload exit')
                with patch('prepare_tools.capture', side_effect=execute("print('built')")):
                    env, receipt = prepare(root, ['fixture'], root / 'positive', prepared)
                self.assertEqual(env['FIXTURE_WORKER'], str(binary))
                self.assertEqual(prepared, {'fixture'})
                self.assertTrue(any(row.get('meter', {}).get('error') is None for row in receipt if 'meter' in row))

    def test_output_overflow_is_not_truncated_success(self):
        from process_capture import capture
        result = capture([sys.executable, '-c', "print('ready'+'x'*30000)"], output_cap=16, timeout=10)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.measurement['error'], 'output cap')
        self.assertLessEqual(len(result.stdout), 16)


if __name__ == "__main__":
    unittest.main()
