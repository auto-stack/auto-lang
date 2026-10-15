# PLAN-782 final review R1 — needs_fix

2026-10-15；stage=review；review_scope=final；plan_revision=1。
Reviewer：本次 Codex 独立检查会话 `/root`，未参与本树实施；未创建其他 agent，未指定/虚构模型标识。
技能：`C:/Users/zhaop/.codex/skills/auto-plan-review/SKILL.md` 及 new/references/verification-contract.md。

## 冻结基线与证据

- 主计划：主检出 `docs/plans/782-ac-text-record-owned.md`（入口 execution_done，7/9）。
- 实施树：`D:/autostack/.wt/lang-782/auto-lang`，`plan-782-dev`。
- reviewed_commit：`b025a85aa38197700763b944d1b7fec2ecbc4ad0`；base_commit：`4568e4ea795996a759b4747fc56b30a30c4308db`。
- 入口/出口 HEAD 相同、实施树 clean；未修改实现、canonical Specs 或 ledger；未合入。
- auto-down：`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；ac-core Cargo.lock SHA256 `296429E6DD6DB8E1AFFE9755F29C5FF94C902362361FA719B7BC1CD11EE7077E`；runtime/text Cargo.lock `36A992D94B5E272160E7558AA7338224187E07DC3CF7054F8D6557978B7E1AFC`。
- 当前 canonical 输入指纹：`spec-inputs.tsv`；规范增量冻结副本：`proposed-spec-delta.frozen.md`，SHA256 `BD9C6D4DE00D6EC5DA80DB96CF04B42E7F1548F007766B3C2DF9603CB12834C3`。
- 原有 review-r1 下未跟踪探针被本轮重新构建/执行，结果以本轮 `results.json` / `replay-output.txt` 为准；没有把旧探针文件存在视为历史 review pass。

本轮命令（cwd 均为实施树）：

1. `cargo build --manifest-path experimental/ac-core/Cargo.toml --locked`：通过、零警告，随后所有 PE 探针使用该二进制；SHA256 见 results.json。
2. `cargo test --manifest-path experimental/ac-core/Cargo.toml --locked --test text_record_hir --test text_record_source --test text_record_resources --test text_record_native`：13+19+27+12=71 passed，0 failed/ignored。
3. `cargo check --manifest-path experimental/ac-core/Cargo.toml --locked --all-targets`、`cargo fmt --manifest-path experimental/ac-core/Cargo.toml -- --check`、`git diff --check 4568e4ea7..HEAD`：通过。
4. `python D:/autostack/auto-lang/docs/plans/attachments/782/review-r1/replay.py`：14 个 source→verify→PE/前门探针，逐例原文、stdout/stderr/exit 与工具指纹落在本目录。runner 本身退出 0 只表示收集成功，不表示全部 oracle 通过。
5. `powershell -File D:/autostack/auto-lang/docs/plans/attachments/782/review-r1/api-replay.ps1` 的等价 rustc 编译/运行：真实 Bundle 9 控制，断言 mutation 发生；结果 `api-results.txt`。合法基线 CHECKED；call arg/param、ctor arg、FieldId、TypeId、自环、requires 各受控拒；Move.inner 越界 PANIC（catch_unwind 收集，非成功拒绝）。

未运行根 t/tv/tt/tb/tu/taa/tf 或全量旧 gate。最终代码审查未通过，不提前运行 merge-owned 回归；本计划无 crates/ 改动，根档本身不适用。旧 gate 的 historical 25/25 收据只证明其已覆盖案例，不抵消下列新反例。

## Findings（全部 blocking，原合同内缺口）

### F-782-R1-01 [P1] 嵌套资源字段隐式复制逃过拥有门，造成双重释放

AC-02/03，V-03/08，T-02/04/05。`verify.rs:995` 的 bare-transfer 门只匹配 ReadLocal；`text_resources.rs:1247` ctor 门同样只识别裸根。资源类型 FieldGet 被当作可新接收的 owner，native 对其位复制并给新根再安排释放。

- `field_copy.at`：`let b Inner = a.i`，应在前门按资源赋值规则拒绝（clone 兄弟合法）；实际 build=0，PE exit=6，末尾 `FREE id=1 / FREE-INVALID`。
- `field_copy_return.at`：view 参数 `a Outer` 的 `return a.i`，应拒 ownership-required；实际 build=0、PE exit=6、同一双 free。
- `field_ctor_copy.at`：`O { i: a.i }`，应拒隐式 owner 转移；实际 build=0、PE exit=6、同一双 free。
- `field_clone.at` 对照：`a.i.clone()` build=0、PE exit=6，id2/id1 各一次正确清理。

修复应按表达式的实际拥有来源覆盖 let/assign/return/ctor，不仅给三个字符串案例加特判；继续保留 copyable 子 record 的值复制和资源字段只读 view，不得隐式 clone 或放行部分 move。为上述三个入口及合法 clone/copyable 对照加永久回归。

### F-782-R1-02 [P1] 同语句 move-return 重建仍释放已转移 buffer

AC-03，V-05，T-04/05。`text_resources.rs:1676,1782` 以 RHS 之前的 st_snapshot.live 安排 replace_frees，未扣除 RHS 实际 move；native.rs:3219 按此计划先 free，再提交返回聚合。

`move_return.at`：`fn consume(move t T) T { return t.move }`，`a = consume(a.move)`；这是 §5.2/V-05 明文合法形。期望 PE exit=3，返回 buffer 在 a 最终出口只 free 一次。实际 build=0，`LIT id=1 / FREE id=1 / RT-FAIL code=-7`，PE exit=70。
`move_fresh.at` 相同 callee、独立 `let b = consume(a.move)` 对照 exit=3 且正确配对。

修复 RHS move 完成后的旧 owner 责任判定；保留 RHS 未读完前不覆盖 out-slot 的规则；永久断言同一目标与 fresh 目标的值、trace、分配数。

### F-782-R1-03 [P1] 动态短路的聚合临时没有路径存在标记，循环跳过侧重复释放

AC-04，V-09，T-05。native.rs:3787/3862/4137 将聚合临时放 pending_aggs；LogicalRhs (`4001..4065`) 只为 pending_temps 合流存在性，`release_stmt_temps:2852` 对聚合槽无条件 free。栈槽跨循环保留上轮地址。

`short_loop.at`：两轮循环，`cond(i) && check(a.clone())`，cond 仅第一轮 true。期望：a id1，第一轮 clone id2/free2，第二轮零 clone/free，最终 free1；PE exit=1。实际 build=0、exit=1，但 trace 为 `CLONE id=2 / FREE id=2 / FREE-INVALID / ... / FREE id=1`。exit 正确不能抵消无效释放。
单次 `dynamic_take.at` 与 `dynamic_skip.at` 对照运行通过；它们没有上轮残留，不能证明 loop 重入路径。

为 ctor/clone/拥有 call-return 聚合临时按实际执行路径保留到完整语句尾；覆盖 &&/||、嵌套/循环 take→skip、skip→take，且 skip 无 FREE（不靠零初始化栈槽偶然避免错误）。

### F-782-R1-04 [P2] 混合文本与聚合 owner 不遵守统一 LIFO

AC-04，§5.3，V-07/09，T-04/05。native.rs:2832 的 pending_temps 和 pending_aggs 分两轮 drain；2915 的 exit_frees 和 buffer_frees 也分两轮，类型类别优先覆盖了实际创建顺序。

- `reverse_temp_order.at`：先创建文本 id1，再创建 record id2；应 FREE2/FREE1，实际 FREE1/FREE2。`plain_temp_order.at` 相反创建序的对照恰正确。
- `mixed_local_order.at`：先 `let s Str` id1，后 `let a T` id2；应出口 FREE2/FREE1，实际 FREE1/FREE2。

两个 PE 均 exit=11、分配释放多重集相等；原 gate 仅配对计数不能检出顺序。应统一跨类型创建顺序的清理义务，根内仍按字段声明逆序；永久精确 trace 覆盖两个创建方向和临时/局部两层。

### F-782-R1-05 [P1] 不可信 Move 引用在结构界检之前被索引，verifier panic

AC-03/05，§5.4，V-08，T-02。verify.rs:830 先调用 check_text_record_rules，再 check_body；规则 `936` 直接索引 `body.exprs[inner.0]`。

`api_probe.rs` 从合法 source 生成、绑定真实 Bundle，先 clone 作对照，再实际修改 `ExprKind::Move.inner = ExprId(u32::MAX)`，断言与修改前不等，直接调用 verify_text_record。期望受控 bad-reference，无 Checked/panic；实际 `index out of bounds: len is 5 but index is 4294967295`。合法与其他七个 mutation 均有相应控制结果，见 api-results.txt。

在语义规则前完成全部结构/引用界检，或每个不可信索引安全读取并发诊断；查相邻新节点/不执行表达式/partial Bundle 路径，加真正程序化变异永久测试。收据称 text_record_hir 已覆盖 ctor/call 越界和删 requires，但该文件实际 13 测试未物化这些 mutation（本轮独立 probe 补查并不等于现有永久测试已存在），需纠正取证映射。

### F-782-R1-06 [P2] mandatory V-07 实际故障注入与部分输出 oracle 尚未交付

AC-04/06，V-07，T-04/05/06/07。verification.md §2 将 V-07 标 ✅，只引用三个清理计划形状测试；text_record_native 无 fail_nth_alloc/fail_ledger_nth/budget/status 回滚调用或断言。运行旧 runtime 的 20 个自身测试不能证明组合 lowering 在第二字段失败时的行为。

T-01 `probe/whole_path_probe.rs:408..439` 的 P6 是手写 Rust Ledger 模拟分配/回收，不调用真实 text runtime 或编译后组合代码。冻结 design.md:235..237 则声明“编译器发射的逐字段回滚”与 status/trap 双通道。实际 RecordClone 每字段调用 abic_call（native.rs:3850 附近），abic_call:2901 在任一负 len 上立即进入 trap，没有该 status-return 回滚路径的实证。

工作需在实际组合构造/clone/替换路径接入可观察的第 N 分配、账本、预算临界、零长控制，断言前驱 trace、旧目标/部分输出边界；分别说明实际存在哪个 status helper、哪条上层 fatal 路径，并执行对应 oracle。若实际只采用 fatal inline lowering，须先明确冻结设计与现有合同的关系，不能把模拟 P6 或 runtime 自身测试写成组合 status 回滚实现已通过。不得为通过删除 mandatory 子例。修正 verification/delta 中“全部实证”“零缺口”的声明。

## AC / SD 审核与路由

| AC | 结果 | 证据/边界 |
|---|---|---|
| AC-01 | pass（已检身份面） | 13 HIR 测试中的身份/roundtrip；本轮 build receipts 的新域一致；不授予整体 pass |
| AC-02 | fail | F01；clone/Token/256 层/名义等既有测试通过仍保留 |
| AC-03 | fail | F01/02/05；新 API arg/field/type/cycle/requires 控制多数通过；同异源 wrapper 27 测试内通过 |
| AC-04 | fail | F03/04/06；现有常量 skip、计数多重集绿不足 |
| AC-05 | partial/fail | 深 256/4096/零次原案例复验通过；F05 违反畸形入口不 panic；发布等旧凭证本轮未全重跑，不谎称新 pass |
| AC-06 | partial | Token PE/新旧两 profile 冒烟复验通过；mandatory 与取证声明仍缺口 |
| AC-07 | 本轮 review 完成、验收 non-pass | 六项 blocking；T-08 不勾选 |
| AC-08 | pending merge | 不合入、不归档、不清树 |

SD01..06 目标路径、组件元数据和 touched_goals=[] 的有限 profile 理由可追溯；但 SD01 支持全路径 move、SD02 语句尾/统一清理、SD04 “聚合临时语句尾 LIFO”以及附 A 零缺口声明被 F01..06 反证，不能批准发布。新增 source-text-record 仍仅提案；canonical 无修改。主目录相关 Spec 哈希已冻结，修复后重审受影响 delta。

健康：上述 check/fmt/diff 检查通过，未见新增调试打印混入产品路径；结论不依赖警告总数。主检出有其他计划未跟踪资料与既有 KNOWN-DEBT 修改，均保留。

next：返回同一实施树 work，先修完全部 findings，逐项及兄弟组合定向回归、提交冻结，再送 final review。重开 T-02/04/05/06/07，保留 T-01/03，current_step=2/9，status=executing；T-08/09 pending。根整档不适用，本计划适用的最终联合收口门禁仍由 merge 在 fresh final pass 后、冻结提交上执行，批量回归到期判定另归 regress。
