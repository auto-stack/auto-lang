# Cross-workspace full regression (PLAN-850, three-repository batch since PLAN-867)

Run `cargo tf --batch` on the main language checkout, alone. This runs language
full, UI full, language tt and language tb in that order — and since PLAN-867
the three-repository batch continues with the Man legs (man-t, man-tcore,
man-tc, man-ti, with the same te-child/auto-ui-tool bootstrap as
`scripts/cross-repo-tiers.py`) and the UI sub-tier legs (ui-tu single-process
libtest, ui-ti integration, ui-tn enumerated-but-manual). A failed child stops
the queue and leaves a failed receipt. `cargo tf` selects the two full owners;
`cargo taa`, `cargo ta` and `cargo t3` preserve their original language features
and full/milestone configuration. Explicit substring filters select a scoped
run; its receipt cannot certify full coverage. Existing `--no-fail-fast` callers
remain supported. Other nextest flags must be passed through a future explicitly
reviewed interface rather than silently changing a formal gate. PLAN-867 adds
one such interface: `--frozen-trees`, which permits an unfiltered batch against
a clean frozen three-repository worktree group (all three trees must have no
code/configuration WIP; HEADs and lock hashes are receipt-bound either way).
ui-tn never runs inside the batch unless `AUTO_UI_TN_RUN=1` — it stays a manual
native gate recorded as pending, never as passed.

The 850 semantics are unchanged: zero-selection legs fail the gate; every leg's
selection count is asserted against the frozen floors in
`scripts/tier-selection-minimums.json.txt` (the same data file
`scripts/check-tier-selection-minimums.py` uses for the standalone guard);
baseline validation for the legacy legs remains authoritative. Failed-ID lists
are extracted from the child logs (nextest `FAIL [time] binary name` lines /
libtest `test ... FAILED` lines) into the receipt.

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

PLAN-867 adds the evidence-cited retirement channel
`scripts/full-regression-retirements-844.json.txt`: the 850 baseline files stay
untouched acceptance inputs, and retired required IDs are removed through this
manifest instead (PLAN-844 T-04 retired zero-consumer legacy surfaces after the
last batch receipt had already been written; the retirement is adjudicated
there and reconciled as RETIRED in the PLAN-867 closure). Regenerating the
baseline files to accept a deletion remains forbidden.

The pre-split denominator and its reconciliation live in
`scripts/pre-split-universe/` (enumerated by
`scripts/enumerate-pre-split-universe.py` at commit 464e2316b = abd34a355^,
with last-compiling-witness runs for the faces PLAN-818 T-02 accidentally left
ungated — see its manifest) and `scripts/split-closure.py` /
`scripts/split-golden-ledger.py` produce the closure and golden-input ledger
committed as `docs/plans/attachments/867/{closure,golden-ledger}.md`.

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
