import os
import json
from pathlib import Path
import sys
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "test-governance"))
from native_meter import api, measure


@unittest.skipUnless(os.name == "nt", "native Windows process-tree meter")
class NativeMeter(unittest.TestCase):
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


if __name__ == "__main__":
    unittest.main()
