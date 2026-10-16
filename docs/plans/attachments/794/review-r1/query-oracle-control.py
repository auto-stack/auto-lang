import importlib.util
import pathlib
import sys

source = pathlib.Path(sys.argv[1]).resolve()
spec = importlib.util.spec_from_file_location("p781_review_driver", source)
driver = importlib.util.module_from_spec(spec)
spec.loader.exec_module(driver)

class FailedQuery:
    def press(self, label):
        return False
    def reply(self):
        return "applied"  # stale label from an earlier successful request

actual = driver.Driver.expect_reply(FailedQuery(), "QC50", "applied", deadline=0.01)
print(f"query action failed; expected rejection; actual={actual!r}")
assert actual[0] is False, "oracle incorrectly accepts a failed query using cached success"
