"""Independent controls for the actual p781 Driver.expect_reply implementation."""
import importlib.util
import pathlib
import sys

source = pathlib.Path(sys.argv[1]).resolve()
spec = importlib.util.spec_from_file_location("review_p781", source)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
module.POLL = 0.001

class Query:
    def __init__(self, presses, replies):
        self.presses = iter(presses)
        self.replies = iter(replies)
        self.press_count = self.reply_count = 0
    def press(self, label):
        assert label == "QC50"
        self.press_count += 1
        return next(self.presses, True)
    def reply(self):
        self.reply_count += 1
        return next(self.replies, "pending")

cases = [
    ("failed_press_cached_success", [False], ["applied"], False, 1, 0),
    ("pending_pending_applied", [True, True, True], ["pending", "pending", "applied"], True, 3, 3),
    ("failure_after_pending", [True, False], ["pending", "applied"], False, 2, 1),
    ("immediate_applied", [True], ["applied"], True, 1, 1),
]
for name, presses, replies, expected, press_count, reply_count in cases:
    query = Query(presses, replies)
    actual = module.Driver.expect_reply(query, "QC50", "applied", deadline=0.1)
    assert actual[0] is expected, (name, actual)
    assert (query.press_count, query.reply_count) == (press_count, reply_count), name
    print(f"PASS {name}: {actual!r}; presses={press_count}, reads={reply_count}")

query = Query([], [])
actual = module.Driver.expect_reply(query, "QC50", "applied", deadline=0.005)
assert actual[0] is False and query.press_count > 0
assert query.press_count == query.reply_count
print(f"PASS perpetual_pending_rejected: reads={query.reply_count}")
actual = module.Driver.oracle_negative_control(None)
assert actual[0] is True, actual
print(f"PASS permanent_negative_control: {actual!r}")
