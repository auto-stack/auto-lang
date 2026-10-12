"""Re-run the original R11 real generator/freshness probe on delivered 751."""
import importlib.util
from pathlib import Path

spec = importlib.util.spec_from_file_location(
    "r11_probe", Path(__file__).with_name("738-review-r11-probe.py")
)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
harness = module.harness
harness.ROOT = Path("D:/autostack/.wt/review-751-r2/auto-lang")
harness.SOURCE = harness.ROOT / "crates/auto-man/src/rust_ui.rs"
harness.LOG = harness.ROOT / ".review751-r2-lock.log"

if __name__ == "__main__":
    harness.main()
