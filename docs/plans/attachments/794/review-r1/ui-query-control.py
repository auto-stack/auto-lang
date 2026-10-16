"""Replay the unmodified p781 driver, freshly executing state queries on retries.

The original expect_reply presses once, then polls a cached R=<state> label.
Only query buttons are re-pressed; enqueue and scroll actions remain unchanged.
"""
import importlib.util
import pathlib
import sys
import time

source = pathlib.Path(sys.argv[1]).resolve()
spec = importlib.util.spec_from_file_location("p781_review_driver", source)
driver = importlib.util.module_from_spec(spec)
spec.loader.exec_module(driver)

def fresh_query(self, label, want, deadline=12.0):
    start = time.monotonic()
    samples = []
    while time.monotonic() - start < deadline:
        pressed = self.press(label)
        got = self.reply() if pressed else None
        samples.append(got)
        if got == want:
            print(f"QUERY_CONTROL {label}: samples={samples!r}")
            return True, f"fresh_query={label} samples={samples!r}"
        time.sleep(driver.POLL)
    print(f"QUERY_CONTROL FAIL {label}: samples={samples!r} want={want}")
    return False, f"fresh_query={label} samples={samples!r} want={want}"

driver.Driver.expect_reply = fresh_query
sys.exit(driver.main())
