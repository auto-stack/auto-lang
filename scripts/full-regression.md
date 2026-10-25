# Cross-workspace full regression (PLAN-850)

Run `cargo tf --batch` on the main language checkout, alone. This runs language
full, UI full, language tt and language tb in that order. A failed child stops
the queue and leaves a failed receipt. `cargo tf` selects the two full owners;
`cargo taa`, `cargo ta` and `cargo t3` preserve their original language features
and full/milestone configuration. Explicit substring filters select a scoped
run; its receipt cannot certify full coverage. Existing `--no-fail-fast` callers
remain supported. Other nextest flags must be passed through a future explicitly
reviewed interface rather than silently changing a formal gate.

Work/review use `cargo tf --list-only` (optionally `--batch`): build and inspect
real nextest JSON, never run a whole test tier. The UI child uses the original
language full configuration, three UI lib packages, and compiler/ui, restoring
the feature unification formerly supplied by auto-lang/ui-iced. UI daily t
excludes ui_gen and therefore cannot substitute for UI full.

The `full-regression-*-ids.txt` files preserve suite + tab + test-name identity.
The UI source is PLAN-846's `merge-r2/auto_ui_*-pre831-root-binary-list.txt`:
3,100 raw lines contain nine duplicate IDs, yielding 3,091 unique IDs. The
language source is the unchanged original full selector with only the three
external packages routed separately: 3,718 IDs including 14 integration IDs.
No production/test Rust source changed between 847's batch baseline and 850's
base. Real owner listings match those sets exactly. `*-selected.txt` additionally
preserve required unignored/profile-selected IDs (language full 3,218, UI full
3,073); tt/tb use their own original daily profile. New IDs are reported.
Baseline files are acceptance inputs: do not regenerate them to accept deletion,
new ignore markers, feature loss or changed profile filters.

Each language feature tier (tt/tb/taa/ta/t3) has its own complete `ids` and
`selected` files. A feature-specific required ID is never intersected with the
tf base. Those snapshots come from the already-frozen real JSON listings;
explicit scoped runs may narrow selected tests but must still enumerate the
complete compiled identity set.

The wrapper retains the existing target directories and worker discovery. An
explicit invalid root fails; metadata checks that the selected UI owner is the
one the language workspace actually consumes. Receipt identities include both
HEADs, code/config diffs and lockfile hashes. A named machine mutex covers the
entire queue. Child deadlines do not alter nextest's per-test deadlines; timeout
and interruption terminate the child tree, with bounded cleanup. Existing
producer-only 738 matrix / 796 IPC / 818 turnaround / 827 inventory reports are protected and restored byte-for-byte while
generated reports are retained. Failure to protect/restore them fails the gate.

Focused controls: `python -m unittest discover -s scripts -p test_full_regression.py`.
The zero-dependency Rust driver only bridges Cargo aliases to Python 3; set
`PYTHON` when the platform's default Python command is unavailable.
