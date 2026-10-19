# PLAN-810 独立 final review R2

2026-10-19；`stage: review | review_scope: final | outcome: needs_fix`。

## 冻结身份与结论

受审 `9226c8af9365389342bbc63d9c91e9ff0b6fee42`（修复链 `55ce34200`→`0f2995e5f`→`9226c8af9`），Plan revision **1**；base `ab08955c241f6a824091c53d3b02894ca17e193b`，前轮受审 `ccd024d7208caa14fb2f71d71b8d256d2ba74f82`。仍为唯一工作树 `D:/autostack/.wt/lang-810/auto-lang` / `codex/plan-810-dev`。入口与结论前代码 clean、HEAD 未漂移；auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` clean；Cargo.lock SHA256 `39820FDEF578729B42A0371ADAA22AA0DEAF2B62C3B702C6BF6868602F9544C7` 未变。

Reviewer 为本 Codex 复审上下文（也是 R1 reviewer，未参与实施；精确模型/session ID 未暴露）。未创建 agent/工作树、未修改实施代码。共享 Plan 从主检出读取。完整身份、canonical 输入及 proposal 指纹见 [baseline.json](baseline.json)；[冻结 SD](spec-delta.patch) SHA256 `88BBC1667DFDA4D05565CA5D247F8332CCC974E20A3A4377FF599BC93DA15CE0`。主检出在别的计划簿记中前进，本树未合流，未据此改变受审范围。

**主要行为修复已通过，最终验收仍不通过**：R1-01/02/04/06 CLOSED；R1-03/05 PARTIAL，剩余工作期 oracle、发行输入身份及 SD 仍不满足原合同。没有把本轮观察到的旧错误路径宣布为新回归，也不要求顺手修旧语法。整档回归继续归 merge。

## R1 finding 闭环

| R1 finding | 本轮核对 | 结论 |
|---|---|---|
| F810-R1-01 快照生命周期 | computed 成功臂 release、recipe 在承诺后先 release；目标凭据校验+尾部收缩正确。原始反例在新冻结库上重放：n=1/8/64 的公开 widget/object、默认 store/object、pub style 全为 **0**，body/派发对照也为0。新独立控制验证 release 不改游标/旗标/诊断、释放后 restore 拒绝、旧代次/外来 release 不破坏合法凭据、外层凭据仍可兑现 | **CLOSED** |
| F810-R1-02 777 | strict exit0 `pass violations=0 grandfathered=2`；基线绑定 `55ce34200`，审计没有放宽 forbidden；实际 self-test 正反控15/15 | **CLOSED** |
| F810-R1-03 oracle | 真 style None、Core/UI 两 session、store 对照、owned AST、跨ctx/重复凭据、三层 import 均已有实际断言，20新测全绿；VM/Vue/Rust/Ark/Jet 代表真实消费者重放通过。错误后同一 Parser 的 scope/旧怪癖证据仍缺，792 旧收据不能绑定当前源包；见 R2-01/02 | **PARTIAL** |
| F810-R1-04 prev | 两 push 重同步原语与 stream 手动恢复已区分；原反例 `scene_prev old=6 migrated=6`；scene 让行、subresource、stream 新测试通过；源码路径等价性检查通过 | **CLOSED** |
| F810-R1-05 SD | computed/完整文法迁出、inner wrapper 删除、parse_on_events 方法归属等大部纠正；parser-api 仍保留同段矛盾，错误边界仍未调和，见 R2-03 | **PARTIAL** |
| F810-R1-06 卫生 | 按两个 crate **实际 edition 2021** 检查 grammar7文件、桥、新测试、toy consumer，rustfmt exit0；新 core_parser warning 已消除；旧 events lint 明确保留，不冒充新增；diff --check exit0 | **CLOSED** |

R1 反例和期望未改：复制源为 [r1-replay.rs](r1-replay.rs)，原始 old=6 对照保留；当前 [probes.log](probes.log) 与 [重放脚本](replay-probes.ps1) 不覆盖 R1 的历史证据。R1 生命周期永久测试的 `props` 后半段实际已切换为 body 字符串，虽注释称 store/object，本轮用原始独立 object 反例另行验证了该真实入口，不能把该永久测试的注释当作 object 覆盖。

## 仍需 work 完成的 findings

### F810-R2-01 / P2 / R1-03 未闭合：错误后新建 Parser 掩盖原合同要求的状态证据

AC-02/03，T-06/T-07。`v03_shadow_and_error_paths` 的错误后正控是另建一个 Parser 解析 msg；这不能观察失败实例的 scope/临时绑定/旗标。新 credential 测试主要覆盖跨实例和重复兑现，不能把它与 shadow 正例的通过合并宣称全部失败/恢复边界已冻结。

独立 [extra-probes.rs](extra-probes.rs) / [extra-probes.log](extra-probes.log) 在**同一个** Parser 上的代表反例：

```text
合法: widget W { on { .Inc(arg) -> { let p = 1 } } view { col {} } }\narg
结果: declaration_ok=true, on_flag=false, tail_ok=false
错误: widget W { on { .Inc(arg) -> nope } view { col {} } }\narg
结果: declaration_ok=false, on_flag=false, tail_ok=true
```

错误后 flag 已恢复，但 handler 参数 `arg` 仍在失败 Parser 的作用域可见；新建 Parser 的正控完全观察不到这件事。原基面同样是 `enter_scope`→`body_res?`→`exit_scope` 的顺序，`?` 可先返回，见 [旧 on 出口源码](old-on-error-excerpt.txt)。这不是本轮新引入的正确性缺陷，要求是按计划 §5.2 将旧行为具名冻结、用同实例区分状态并说明恢复入口/边界，不能既保留旧泄漏又以别的实例的成功证明“错误恢复全部保持/已覆盖”。不要求改变旧行为。

此外，handoff §7 新承认未闭合 fragment 的 EOF 异常，却把实际基线隔离证明写成“需要时”。合同 §5.3/V-02 已要求已发现的无界EOF怪癖用隔离进程+外部截止证明基线。当前没有绑定原始输入/旧提交的实测收据；`full-grammar.md` 也未列这项具体旧支持边界。本轮对准确输入 `view fn F(x: int) { if x {` 作了**当前提交**隔离观察：[isolated-eof.py](isolated-eof.py) / [isolated-eof.json](isolated-eof.json)，子进程5秒未返回，已终止并回收。该结果只能证明本次5秒观察未收敛，不能证明旧提交结果或无条件复述 handoff 的 `0xc0000409`。

要求：提交实际同实例成功/失败/恢复对照和原基面适用证据；对发现的旧 scope/EOF 行为写精确支持边界，不扩大成修旧语法。补齐原 V-01..06 的证据映射（包括已有 AST/序列化/源位置 oracle 可复用的具体指向）；禁止用 fresh Parser、总测试数或 future 整档代替必需断言。永久测试中的 store/object 注释也须与实际 body 输入对齐或补回真实 object 输入。

### F810-R2-02 / P2 / R1-03/AC-05 未闭合：792 25/25 属于修复前的源包

AC-05，T-07/T-08。handoff 沿用 §4 初轮 `driver.py EXIT=0`，本轮修复后没有新的适用输入/提交收据。实际现存 `../scratch-consumer/driver-receipt.json` 确为25/25，但对应 consumer-b 的 `auto-ui-dialect/src/grammar/mod.rs` **不含**本轮新增的 `raw_cursor_release`、`advance_peek_kind_resync`：

| 输入 | SHA256 |
|---|---|
| 旧已构建包 mod.rs | `DE628E675F0C014D9C1A64D427807D5EBE6CA4E14E1707DD17879995493F3557` |
| 受审修复源 mod.rs | `82DE73A2FA615C002125CE42A989E15D9688262A0E5D7D061631A52497F0B02E` |

旧 [25/25 收据副本](old-792-driver-receipt.json)、[旧包源文件副本](old-792-packaged-grammar-mod.rs.txt)、[身份比较](792-source-identity.json) 均已保存；差异不只是时间戳。源码是发行验收输入的一部分，crate/依赖成员数没变不等于输入没变；旧包编译绿不能覆盖新端口及其上层接线。此项不是新增 merge-owned 整档门。

要求：原树 work 沿既有792 driver 对**最终修复源码**完成受影响 fresh、A→B locked、source 身份/缺源及反向依赖等既有控制，绑定 producer 提交和包内源码hash，再送审。不改发行器、不拿“无新依赖”作为源输入不变的替代。本轮确认强制 oracle 缺口后未再开一轮昂贵发行重构建，将已承认未完成的工作归还 work。

### F810-R2-03 / P2 / R1-05 未闭合：规范仍有冲突，旧错误边界未进入 SD

AC-05/06，T-08。proposal `docs/specs/auto-parser-api/project.md:74–79` 仍在同一个“边界与已知限制”条目写：

- “内建 UiDialect **仍经** pub(crate) parser() escape hatch 使用完整 Parser”；
- 紧接着又说“现行事实：PLAN-810 起该 escape hatch **已删除**”。

R1 对此的要求是改为单一现行事实、历史另述；本轮只是把括号里的“勘误”换成“现行事实”，前句仍未修。其他 SD 的主要 owner/迁出陈述已核实，不需重写已闭合部分。

`full-grammar.md` 关于错误/恢复的宽泛保持陈述还须与 R2-01 具名旧边界对账。handoff 已新增 fragment EOF 边界，而 proposal 的失败清单只笼统指向 msg/model；不能把新的已知事实只留在交接日志。`contract-baseline.md` §5 的三位点“共用旧臂形态”历史总结也未标明已被双原语修正，避免下一位维护者重复错误设计。

要求：完成 SD-01..06 的当前事实/旧边界调和，更新冻结 delta 和 handoff。修复不改变已批准语义合同，revision 不因簿记自动递增；如打算改变旧错误行为再按合同另作决策。

## 实际验证与 AC 对账

- `cargo test --offline -p auto-lang --lib plan810`：**20/20**；`cargo build --offline -p auto-lang --lib`：exit0。见 [plan810.log](plan810.log)、[lib-build.log](lib-build.log)。测试程序为本轮刚生成的 `auto_lang-e5c5783233e675be.exe`，原始 library 构建后才执行独立反例。
- 同程序、Cargo 包 cwd 下的针对性复验：plan806 **18/18**、plan778 **4/4**、plan408 **20/20**、plan051 **25/25**、native_css **7/7**；日志按 filter 同名存本目录。
- `inline_lambda` **9/9**：raw log [inline_lambda.log](inline_lambda.log) 明列真实 Parser→extract→Vue/Rust/Ark/Jet 产物与 VM handler dispatch；另 Rust simple_counter **1/1**。结合408真实 Vue SFC、051 VM/适配器和806/810一层/三层导入，这些代表消费者证据可对账；完整 tu/tv 仍 pending merge，不冒称本轮跑过整档。
- 独立轻 consumer `cargo test --offline --locked --manifest-path test/fixtures/modularity/ui-grammar-consumer/Cargo.toml` **7/7**，[consumer.log](consumer.log)。既有 ParseContext/ExternalDialect 未改方法面，UiParseContext 两新增能力及 toy 同步实现已检查。
- strict777 exit0、self-test15/15、触面fmt2021 exit0、diff --check exit0；[audit.log](audit.log)、[audit-controls.log](audit-controls.log)、[format.log](format.log)、[diff-check.log](diff-check.log)。保持 R1 的 edition2024 检查为历史，不把错误 edition 当作新卫生问题。
- [额外控制](run-extra.ps1) exit0：release 不恢复状态、已释放拒绝、旧代次/外来无操作、外层存活；style参数错误/raw computed fallback诊断出口无快照残留；同一Parser的scope观察如上。探针初稿把宽容语法 `let x = }` 误当必返回Err，已按实际诊断收集行为改用 `fn bad(`，没有将探针错误计为产品回归。
- EOF 只在独立子进程中受5秒外部截止观察，非进程内挂住测试；脚本在截止时终止并回收子进程。未跑裸 t/tv/tu/tt/tb/taa/tf、docs_gen 或全画廊；未改 canonical Spec/ledger。

| AC | 当前结果 | 限制/映射 |
|---|---|---|
| AC-01 | partial | 单源迁出/兼容入口继续成立；V01原有精确AST/序列化/位置及失败边界映射仍需与剩余oracle一起收口 |
| AC-02 | partial | 原快照/prev缺陷实测关闭；R2-01错误状态/基线边界证据未齐 |
| AC-03 | partial | 默认/Core/None/session/三层import已过；错误恢复的完整保持声明受R2-01限制 |
| AC-04 | pass（代表支持面） | VM与四生成器真消费者、错误对照相关旧测试已核实；全档回归仍归merge |
| AC-05 | fail | 轻包/777通过；R2-02当前源包792、R2-03 SD未闭 |
| AC-06 | fail | 独立R2及卫生检查已执行，仍有未完成work-owned项 |
| AC-07 | pending | merge-owned裸t/tv/tu、必要tt、master并集复核、规范/ledger/归档/guard清理未开始 |

## 路由及第二次失败后的诊断要求

这已是连续第二次最终验收非通过。下一次 handoff 前，work须先更新具名根因/覆盖调整表，不以增加几个测试数量代替解释：

| 剩余不变量 | 前次为何漏过 | 实际路径/状态 | 下一轮需调整的 oracle |
|---|---|---|---|
| 同实例错误状态及旧支持边界 | fresh Parser 无法观察失败实例；错误“怪癖”仅叙述 | handler缺body的scope/flag、fragment EOF及原基面 | 同实例成功/失败/恢复对照；隔离输入/截止/旧提交身份；明确保留的旧行为 |
| 发行输入与冻结代码相同 | 用成员数不变代替包内源hash不变，复用旧25/25 | 新两端口、上层接线、fresh/locked包 | 精确producer提交+包内hash+受影响driver实测 |
| SD单一当前事实 | 在旧错误句后插括号，没有替换原句 | parser-api限制、full-grammar失败边界、基线总结 | 逐条当前规则/旧边界对账后重冻delta |

状态回 `executing`；T-02 的端口修复现已通过，保留 T-01..05，共 **5/11**。T-06/07/08 未完成，T-09最终pass未达，T-10/11保持pending；修正入场frontmatter声称8而任务表仅4项打勾的不一致。`plan_revision: 1`不变。

下一步：**同一树 work 完成 R2-01..03，提交并绑定证据，再作受影响复审**。本次不授权合入、不修实施、不清理树。通过后 merge 才在最终冻结提交统一裸 `cargo t`+`cargo tv`+`cargo tu`（触trans再tt）并完成后续沉淀/归档。
