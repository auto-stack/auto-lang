# 固定组周期回归（PLAN-896）

普通计划使用cargo t具名固定完整组及必要集成，不通过tm。无参数cargo t为日常范围，cargo t all显式全语言组。
周期触发和主检出单实例保持。语言侧直接一次固定组并集，不先发现旧language/tt/tb三档；UI/Man各保owner入口。
全仓迁移发现只在明确审计运行，不是每次局部运行前置。慢组/CPU/内存与构建成本如实分列，不按时间机械切叶。

以下保历史owner协议/冻结基线资料，其中旧tm/叶调用步骤由上述固定入口替换。

# 模块周期回归（PLAN-888）

计划合入门使用 cargo tm 的受影响完整模块及消费者/集成并集，不叠跑旧t/tv/tt。
周期触发条件和主检出单实例纪律保留，cargo tf --batch 消费显式periodic-module-schedule.json
清单，输出真实叶选择并只执行一次语言模块并集；UI/Man仍走原owner队列。
旧language/tt/tb列表只核历史守恒，不再逐档重复执行。tf/taa/ta/t3非batch为显式旧身份兼容导航，
滤串选完整叶。无参数cargo tm不会隐式运行全部；新模块用法见[test-governance/README.md](test-governance/README.md)。

work/review只做相关反例/模块与必要发现，不因提交运行周期全集。最终代码审查通过后由merge
执行冻结模块门；本次迁移横切影响必须真实证明义务守恒与执行。超60秒可拆叶继续拆，
不可拆的原采样/规模/生命周期逐ID列慢模块，原条件和限制保留。单实例、根目录/依赖身份、
故障停止、原报告保护/恢复、原UI/Man手工条件均继续生效。

## 850/867历史基线与owner协议（非当前语言执行顺序）

以下保留冻结身份、退役/迁移通道及历史来源；其中language/tt/tb执行顺序已由上面的模块并集替代。

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
