# PLAN-810 独立 final review R1

2026-10-19，`stage: review | review_scope: final | outcome: needs_fix`。

## 基线与结论

- Plan revision **1**；受审提交 `ccd024d7208caa14fb2f71d71b8d256d2ba74f82`，base `ab08955c241f6a824091c53d3b02894ca17e193b`。
- 同一实施树 `D:/autostack/.wt/lang-810/auto-lang`，分支 `codex/plan-810-dev`；入口及结论前 HEAD 均一致，工作树 clean。兄弟 auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，clean；Cargo.lock SHA256 `39820FDEF578729B42A0371ADAA22AA0DEAF2B62C3B702C6BF6868602F9544C7` 未变。
- Reviewer：当前 Codex 独立复审上下文，未参与本会话之前的实施；精确模型/会话身份未暴露，不伪造。未新建 agent、工作树或修实施代码。
- 共享 Plan 从主检出读取；主检出期间由 `30c1b9a7e` 前进至 `fbd0d1671`（其他计划簿记），本树未同步/漂移。依赖、五份 canonical Spec 输入和 proposal 指纹、技能指纹见 [baseline.json](baseline.json)。
- 规范增量冻结于 [spec-delta.patch](spec-delta.patch)，SHA256 `D381EC49F98DD43549DAE0647EC6A0B2D1FC87174A52067C0392E9584A89FF23`；未发布 canonical Spec/ledger。

**不能给予最终 pass**：已复现新增快照保留/清理差异，777 严格门实红，前瞻状态不等价；若干承诺的工作期 oracle 未实现，SD 自相矛盾，触面 fmt/新增 warning 未净。回原树 work 一次性修完整 finding 集，再提交送复审；本登记不批准延期。

## Findings

### F810-R1-01 / P2 / 新回归：成功投机保留快照，直连入口无退出清理

涉及 AC-02/05，T-02/T-06/T-07。`grammar/widget.rs:1259–1269` 的 computed object 成功臂丢弃凭据但不释放其快照；`grammar/actions.rs:105–180` 的成功 style recipe 同样如此。`ParserBridge::raw_cursor_save` 复用 `Parser::port_checkpoint`，实际数据保存在 Parser 的表内，丢弃 ParseCheckpoint 不会释放槽。清理仅存在于方言派发器的 `port_release_scope`。

公开 `Parser::parse_widget_decl` 等薄门面没有派发作用域；**真实默认** store 直连（`parser.rs:5665`）和 `pub style` 直连（`parser.rs:5370`）也不经过该清理。于是成功解析后仍保留每份 LexerState/cur/prev，直到 Parser 销毁；下一次签发还会逐槽扫描所有存活项。旧臂局部 save_state 在返回时即释放，无这项表内累计。

独立 [probes.rs](probes.rs) / [replay-probes.ps1](replay-probes.ps1) / [probes.log](probes.log) 结果（slot 为下一次签发获得的槽号；成功 body 回退和正常 widget 派发作对照）：

| n | public widget/object | default widget/object | public widget/body | default store/object | default pub style |
|---|---:|---:|---:|---:|---:|
| 1 | 1 | 0 | 0 | 1 | 1 |
| 8 | 8 | 0 | 0 | 8 | 8 |
| 64 | 64 | 0 | 0 | 64 | 64 |

要求：为成功/失败出口提供有界的快照结束生命周期；覆盖公开门面、Core 直连、builtin 派发及嵌套重入，不能只依赖 builtin 退出。成功释放不得恢复游标/旗标或回滚诊断，也不能清掉外层仍存活的合法凭据。将上述输入和 body/派发对照收入永久测试，核查凭据存活/失效及作用域隔离。

### F810-R1-02 / P2 / 验收实红：777 基线未刷新，handoff 的通过结论不成立

涉及 AC-05，T-07/T-08。同冻结提交实际运行：

`python scripts/modularity-audit.py --mode strict --no-cargo-check`

退出 **2**，`verdict=fail violations=1 grandfathered=2`。唯一 violation 为 `stale_baseline`：`锚定输入自 base_revision 起已变更: ['Cargo.toml']；基线过期必须显式重采样，不能默默通行`。根 Cargo.toml:125–127 新增独立 consumer 的 exclude，未完成对应基线刷新。[audit.log](audit.log) 含 raw JSON 和受审 HEAD/clean 标识。这不是环境缺失；不能以“合法边无变化”绕过锚定输入校验。

要求：按既有 777 机制显式刷新基线并审计其精确输入/合法边；重跑 strict、原有相关负控，更新绑定修复提交的实际收据。不得放宽 forbidden 或屏蔽 freshness。

### F810-R1-03 / P2 / 原合同证据缺口：测试标题不等于状态/恢复 oracle

涉及 AC-01..05，T-06/T-07/T-08；属于 work-owned 未完成项，非 merge-owned 整档可替代。

- `plan810_full_ui_grammar.rs:235–242` 名为 `v02_style_recipe_none_fallback_keeps_cursor`，唯一输入是 `msg M { Go }`，没有 style、None 或回退后继续 Core。toy consumer 的 None 测试只断言返回 None/未注册，也未比较恢复后的 token/位置，不能替代真实 Parser。
- 同文件 `v04_two_sessions_isolated`（327–336）构造一个未使用的 Core parser，又丢弃另一个 Core parse 的 Result，未断言 Core 行为或两 session 隔离；`v04_store_contextual_keyword_guard`（341–344）只有正向顶层 store 声明，没有标题所述的 fn/on 体内同名引用对照。
- V-03 当前新增测试仅证明成功参数/循环 AST；缺少冻结合同要求的 shadow、失败/重入状态、raw checkpoint 跨上下文/过期拒绝、owned AST 源码释放等区分性断言。新 raw 凭据路径不能仅继承 801 普通 restore 的历史证据。
- 既有 806 真两文件 import+状态变更、051 VM adapter、408 Vue SFC 等**确实存在**，本轮相关复验也绿，不能称为完全没有真消费者。但 T-01/T-07 承诺冻结的 V-05 VM/Vue/Rust/Jet/Ark 代表输入→真实入口→现行支持/错误→原始产物矩阵仍未给出；17 个重叠名称 filter 的总数不足以核对该矩阵。普通一层 import 不证明 V-04 明列的导入再导入。完整 ui_gen 档延期至 merge 合规，所需工作期代表 oracle 不能一并延期。

要求：补实际输入/负控及状态断言，并将所需现有测试逐项映射到 V-01..06；允许复用已经存在且确实覆盖的测试，不必重写一套。明确命令、feature、路径、断言及支持边界；收据不能把上述空壳测试算作闭合。缺环境则如实 pending，并保持 executing。不要借本 finding 修正旧语法怪癖；旧/新状态需以基面可重放 oracle 区分。

### F810-R1-04 / P2 / 原语义不保持：scene 的两次推进被替成 stream 的一次推进

涉及 AC-02，T-02/T-06。旧 `looks_like_scene_decl` 与 `looks_like_subresource` 的算法是 next→回推 observed→回推 saved→再 next 重同步。新 scene.rs:18/233 均改用 `advance_peek_kind`；后者只 next 一次、回推 observed、手动恢复 cur（parser.rs:780–789）。该形态对应旧 stream 位点，**不对应** scene/subresource 的旧 prev 状态。

同 [probes.rs](probes.rs) 使用公开桥能力重放旧两 push 路径，并比较新生产 scene 门：`scene Player : Body2D {}` 的 `prev.pos` 为 **old=6, migrated=0**；cur 恢复不代表 prev 保持。冻结 baseline/新 full-grammar Spec 都把三位点写成等价，错误地把该差异当成“旧臂同构”。本轮证明端口/游标状态漂移，未把它夸大为已观察到的最终 UI 产物错误。

要求：区分旧两 push 重同步与旧手动恢复两种前瞻，并保持各自 prev、docs、错误消费及后继状态。补 scene 成功/让行、subresource 及 stream 的代表对照；冻结的是旧实际行为，不能只改文档批准状态漂移。

### F810-R1-05 / P2 / SD 未通过：新旧规范陈述互相冲突

涉及 AC-05/06，T-08，SD-01/02/03/04/05。

- proposal `frontend/overview.md:54–59` 前句称 computed 已迁，后句仍说 `computed 等结构文法仍在 Parser`；169–172 仍说 widget/on/store 完整外部化及生产接线未完成，与同页新完成陈述冲突。
- `auto-parser-api/project.md:74–78` 保留“UiDialect 仍经 parser() 使用完整 Parser”再插“勘误已删除”；应重写成单一现行事实，历史另述。
- `frontend/design/dialect-system.md` 仍把已删除的 inner wrapper 写成现行结构入口。
- `auto-ui-dialect/project.md:86` 将 `parse_on_events` 错标为 `ParseContext` 方法；实际定义在 `UiParseContext`。
- `full-grammar.md` 对三种 advance-peek/成功快照生命周期的等价性说明须随 F01/F04 调和。

要求：逐 SD 重写 before/after 后的现行规则，区分仍待 R10 的 AST/关键词与本 Plan 的完整文法迁出；基于修复后的真实状态重冻 delta/hash。impact metadata 五个 modify 路径、一个 add 路径和 GOAL-003/007 范围合理，无须把未验证行为提前沉淀。

### F810-R1-06 / P3 / 卫生门未完成：新增文件未格式化，新增测试 warning

涉及 AC-06，T-06/T-08。对 grammar 七文件、parser_bridge、新测试进行 `rustfmt --check --edition 2024 --config skip_children=true`，退出 **1**，见 [format.log](format.log)；大段迁出函数仍保 impl 内缩进，新增 parser 拼接还有 `}impl` 与大量空白残留。新测试 `plan810_full_ui_grammar.rs:333 core_parser` 在真实编译中产生 unused variable，[plan810.log](plan810.log) 有定位。

要求：清理触面格式和新增 warning/debug 残留，并复核修复后断言。区分已有 lint：widget.rs:566 的 unused events 是旧臂同款迁移，其他 auto-lang/依赖旧 warning 不冒充本轮新增，也不要求本 Plan 无关清零。

## AC / task / evidence 对账

| AC | 本轮结果 | 代码/任务及证据 |
|---|---|---|
| AC-01 | partial | 完整 grammar 六算法模块及 Parser/UiDialect 薄委托已存在；迁出旧区域、三自由 helper 和 806 单源复用核实。生产行为全集/状态证据受 F03/F04 限制；T-03/04/05 的迁出/接线成果保留 |
| AC-02 | fail | 伴随 trait 对象安全、无 Parser/TypeStore 引用逃逸，ParseContext 未改；computed 诊断事务、raw CSS 通过；F01/F04 为实测回归，V03 边界未齐 |
| AC-03 | partial | 默认 UI/Core 正控、scene 基本解析、806 真一层 import 状态变更通过；V04 的 None/session/store/嵌套链/错误状态证据欠缺见 F03 |
| AC-04 | partial | 408 Vue 真实 SFC、051 VM/计时器/适配器及 806 import handler 证据可复用；完整约定代表矩阵未冻结，不以 future tu/tv 代替 work oracle |
| AC-05 | fail | locked 轻 consumer 7/7；777 实红、SD 冲突；792 handoff 声称通过，本轮未重复其重构建/负控矩阵，既有收据未作最终 pass 的独立替代 |
| AC-06 | fail | 本独立审查已执行并绑定提交，存在 F01..06；fmt 和新增 warning 未净 |
| AC-07 | pending / merge-owned | 未合入、未沉淀/归档/清理；裸 t/tv/tu、必要 tt、批量回归到期判定均归后续合入阶段 |

## 本轮实际执行与可复现证据

- `cargo check --offline -p auto-ui-dialect -p auto-parser-api -p auto-lang`：exit 0（初次工具输出，含已有 warnings）；另 `cargo build --offline -p auto-lang --lib` exit 0，[lib-build.log](lib-build.log)。
- `cargo test --offline -p auto-lang --lib plan810`：**13/13**，[plan810.log](plan810.log)，实际生成 `auto_lang-e5c5783233e675be.exe`。
- 同一刚生成测试程序定向 `plan806`：**18/18**；`plan408`：**20/20**，[对应日志](plan806.log) / [408](plan408.log)。`plan051`：**25/25**，按 Cargo 的 `crates/auto-lang` cwd 重放见 [plan051-cargo-cwd.log](plan051-cargo-cwd.log)。首次直接在根 cwd 调用为 24/25，失败只因 corpus 相对路径找不到，保留 [初次日志](plan051.log)，不算产品回归。
- `cargo test --offline --locked --manifest-path test/fixtures/modularity/ui-grammar-consumer/Cargo.toml`：**7/7**，[consumer.log](consumer.log)；不是 fresh 全发行验证的替代。
- strict audit exit 2；触面 fmt exit 1；`git diff --check base HEAD` exit 0。**diff whitespace check 绿不代表 rustfmt 绿**。
- 独立 raw/state probes exit 0，观察结果为 F01/F04 的红例及对照值；这是观测器正常运行，不是验收 ALL PASS。首次独立 rustc 链接缺少 Windows import lib 路径，已用项目相同 rust-lld 与 registry native 路径补齐；未改产品构建配置。
- 没有执行整档 t/tv/tu/tt/tb/taa/tf 或 docs_gen；未为寻找更多问题重复全画廊/整档。确认上述阻塞及强制 oracle 缺口后结束本轮最终验收，不把未查矩阵标绿。

## 路由与状态

保留完成的 T-01（调用/能力调查成果）、T-03/04（单源迁出成果）、T-05（escape 删除与薄接线）；将缺陷修复和受影响证据归回 T-02/T-06/T-07/T-08，重开这四项。T-09 审查已发生但未通过，保持未完成；T-10/11 未开始。因此 **executing / current_step 4 / total_steps 11**，语义合同 revision 1 不因 findings/簿记改变。

下一步：同一树修 F810-R1-01..06，先定向反例/必要 check，提交并更新精确 AC/SD/handoff，再复审。final pass 后，merge 在新冻结提交执行裸 `cargo t` + `cargo tv` + `cargo tu`（trans 源如有改动再 tt），整档前仍须满足工作期验收。master 并集改变受审代码/依赖/配置/输入则重核影响面。未合入或清理工作树。
