# PLAN-841 r1 / independent final review R1

2026-10-26，outcome: **needs_fix**。reviewer: 当前独立审查会话 Codex `/root`；本会话未实施该变更，未委派其他 agent，不虚构模型/session ID。审查对象为既有同组提交，未修产品、未合入、未发布 canonical Specs/ledger。

## 冻结基线

- lang `acd8d99d2eb180f4861fb235d783f19dba307a38`，完整 diff base `100172980efb0343fe7016a05eb40f18d0014d74`。
- UI `f7eeb82d0170f69a6a53cf0e4d625caea7eddab9`，base `aee499e87093ff727f68ac8467c506e9ecf6ecdc`。
- Man `ebfbb37903d24bcea3cbd81c6a0a5602da605982`，base `1b7a9ce0de9055f7953129d78a25777662508258`。
- 既有树 `D:/autostack/.wt/lang-841/{auto-lang,auto-ui,auto-man}`，分支均 `codex/plan-841-dev`；进入及退出审查三树均 clean、HEAD 未漂移。down 固定 clean clone `auto-down-clean` 为 `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；未纳入原 down 兄弟的其他 WIP。
- 主计划从语言主检出读取；主检出已有其他会话文档 WIP，不作变更/提交。G841 的 831/832 终态及来源交接有主计划与归档证据，未发现越门依据。
- [baseline.json.txt](baseline.json.txt) 冻结主计划输入 hash、各 commit、当前主检出 Spec 与候选 Spec 的逐文件 hash。候选规范差量完整冻结在 [spec-delta-reviewed.diff](spec-delta-reviewed.diff)，SHA256 `9a15abe00455fd72e9aab9b11380db42702b8ee2e54ab1045f42ec502191f2a6`。此副本用于复审，不代表批准沉淀。

## 发现与可复现关闭条件

### F841-R1-01 / P1 / OPEN — 生产入口绕过领域校验/lowering

合同内缺口；AC-02/04/05，T-05/T-08/T-09。计划 §1/2、T-05 及 SD-03/05 要求真实生成器、VM/renderer 只消费已校验/lowering 的领域表示，未准入数据在执行/生成前拒绝。

实际公开入口 `auto-ui-compiler/src/ui_gen/api.rs:39..80` 的 Rust/Vue 两路均 `parse_ui -> domain_stmts -> extract_widget_from_decl -> generator.generate`，无 `lower_ui`；`parse_ui` 自身只解析。UI 全仓 `lower_ui` 调用调查除测试外仅命中解释器 bridge；runtime `dynamic_build.rs:919` 和 SDK `ui_build*` 也直接解析/消费 UiCode，不能把 helper 的成功当作全入口准入完成。

可复现输入：

```auto
widget App { view { text "ok" {} } }
widget App { view { text "ok" {} } }
```

入口：同目录 `review841_probe.rs::review841_public_generators_enforce_domain_validation`。`parse_ui` 成功，`lower_ui` 明确报 `UI domain validation failed: duplicate widget declaration App`；但 `transpile_aura`、`transpile_vue_aura` 都返回 Ok 并生成。合法单 widget 两路均成功。见 [review841-confirmed.log.txt](review841-confirmed.log.txt)：`public Rust accepted=true, public Vue accepted=true`。

已证原因是上述两个公开入口缺失已存在的 validator 调用；runtime/SDK 的相同结构为源码确认的相邻风险，未声称本轮已运行其重复声明反例。不能仅修改测试或把重复声明标记为历史行为来关闭既定准入合同。

关闭 oracle：所有声明的生产消费入口及跨文件模块边完成受检接线；重复 widget、store/model/message 无效数据和 codec 解码后的无效数据在生成/执行及文件输出前具名拒绝；单 widget、合法五族、导入及四生成器/VM 对照仍通过。修复时按实际共用机制选择代表邻接测试；不要求每条 finding 重跑 792 全矩阵或整档 cargo。若需改变准入合同应走明确修订，不能由审查降低 AC。

### F841-R1-02 / P2 / OPEN — codec 不能解码自身产生的合法嵌套视图

合同内缺口；AC-03/05，T-02/T-06/T-08/T-09。计划 V-03 和 T-01 contract-baseline §4 明确合法嵌套/序列化往返，沿既有 parser 支持边界，不新增任意限额。

输入构造：`widget App { view { ` + `col { ` 重复 48 次 + `text "leaf" {}` + ` }` 重复 48 次 + ` } }`。在同一 32MiB 测试线程内真实 `parse_ui`、`lower_ui`、`encode_ui_code` 全部成功，编码仅 **4253 bytes**；`decode_ui_code` 返回 `UI AST schema/tag validation failed: recursion limit exceeded at line 1 column 2505`。深度 4/16/32 合法对照均成功，非大输入压力承诺。见 `review841_codec_roundtrips_supported_nested_views` 与 [确认日志](review841-confirmed.log.txt)。

原因位置 `auto-ui-dialect/src/codec.rs:24..26`：直接 `serde_json::from_slice` 的默认递归限制成为新解码边界，encoder/parser/lowering 没有对应拒绝，版本相同的自身产物无法 roundtrip。测试线程栈只防探针线程栈成为混淆因素，不修改生产预算。

关闭 oracle：保留本输入、较浅对照、字段/span/source_lines 与坏 domain/schema/tag 控制；在安全有界的解码实现下合法嵌套应可往返。不得靠减少合法嵌套支持、重写 schema 期望或无保护地移除全部边界取得绿。递归类型/组合树若共用相同 codec 机制，应补有代表性的邻接控制，非要求穷举所有深度。

### F841-R1-03 / P2 / OPEN — 触面格式门未通过

AC-05/06，T-09（相关 T-05 文件）。主仓 AGENTS §3 的独立复审健康要求为 clean formatting。按三仓完整差量筛选存在的 Rust 文件，执行 `rustfmt --check --edition 2021 --config skip_children=true <file>`，**30 文件失败**（lang 2、UI 28、Man 0），见 [fmt.log.txt](fmt.log.txt)。

包括全新 fixture `test/fixtures/modularity/ui-ast-model-consumer/tests/v02_ui_golden.rs:34`、本次新增的 recipe 签名空行、`dynamic_build.rs` 的 parse_domain 链和循环缩进、runtime 测试的 UiCode imports。新文件及本次新增行足以排除“只有旧 lib.rs 预存问题”的归因；不要求顺手格式化整个仓。

关闭 oracle：只对本次触面修格式后同命令通过；确有预存残项需逐项给基线差分。格式修复后按实际影响复核；无需因格式修改触发整个测试档。

## AC → task → artifact/evidence

| AC | 结果 | 实际审查与证据 |
|---|---|---|
| AC-01 | pass（本基线范围） | Core UI enum/模型/门面退出；SceneDecl 与中立 domain ports 保留；777 strict PASS/0 violations；SDK 5 个真实 parse/compile/VM/trans/拒绝面测试与 Core plan841 3 个测试通过。源码依赖闭包与既有独立 consumer 证据复核。 |
| AC-02 | fail | 既有 ui_ast 4 测包含五族、四生成器、VM 与跨文件模块，全部通过；公开入口仍接受 validator 已拒绝的数据，F841-R1-01。 |
| AC-03 | fail | 原 codec 5 测、持久 cache schema 1 测全部通过；Weak 最后 owner、字段/span/source_lines 与跨文件 owner 实现核对。独立合法嵌套反例失败，F841-R1-02。 |
| AC-04 | partial/fail | Man 真实 CLI 7 测通过，四生成器/VM helper 对照通过，SDK 与真实源身份已核。792 收据 39/39 可按未变代码范围参考；F841-R1-01 阻断全入口结论。本轮外部方言 fixture 编译因磁盘满中止，不算绿或产品回归。 |
| AC-05 | fail | 旧 Core UI 残边退出、测试 owner 映射与 UI golden 原字节 1 测通过；版本信封/cache 证据有效，但 F841-R1-01/02/03 与 SD 事实声明不符。 |
| AC-06 | fail | 独立最终审查执行完成，不能 pass；T-10 未勾选。 |
| AC-07 | pending | merge-owned closeout，未运行/未合入。 |

定向结果：Core plan841 **3**、SDK surface **5**、codec **5**、ui_ast **4**、cache schema **1**、UI golden **1**、Man CLI **7** 通过；独立确认 probe **1 pass / 2 fail**（另一个探索性 Core parity 测试过滤，见下）。这是各具名入口结果，不把测试数量当完整验收。

源码清理/健康：`git diff --check` 两实施差量无 whitespace error；不等于 rustfmt 已通过。未发现需单列的新增 debug print。编译仍有大量既存 warnings，工作记录已有归因；dialect 两个未使用 helper 在上游基线也仅有定义，**不将其计为本计划新增警告 finding**。先前观察“两条警告”的 commentary 不代表已经证明其为新增回归。

## SD 与遗漏审查

- SD-01 的 Core/UI 分类与旧模型删除基本吻合。SD-02/03/04/05 及新 ui-domain-model 关于“所有生成/执行只走受检路径”的断言被 F841-R1-01 否证；SD-02/05 的合法嵌套 codec/寿命表述受 F841-R1-02 阻断，不能沉淀为已实现事实。
- metadata 已有精确路径与 GOAL-003/007；候选还修改 `docs/specs/auto-lang/project.md`、`docs/specs/auto-ui-dialect/design/full-grammar.md`，work 应将这两目标与现有 SD-02/04 明确对齐，merge 时不得漏沉淀。此为收口建议，未新增合同功能。
- 842 的上层 re-export/别名/源码形适配清单已检查，未将其视为批准延期生产准入/codec 的理由。无通过新增债务条目豁免 AC；三项均为当前 Plan 阻断。

## 证据适用与环境限制

- 792 最终收据绑定 lang `338c95021`，到当前 `acd8d99d2` 仅多出该收据文件，UI/Man 代码均未再改；故生成与发行证据未因最终 receipt 提交自动失效。未重跑成本高的完整发行矩阵，也不以旧 39/39 覆盖新反例。
- 外部 `ui-dialect-consumer` 本轮编译停于 `rustc-LLVM ERROR: IO failure on output stream: no space on device`，原始 [external log](review841-external.log.txt) 保留。没有改 Cargo feature/锁/预算绕过；初始没有的三个 fixture Cargo.lock 已撤去。
- 探索性 Core on 输入在 Core/domain 两路均不合法，属于 probe setup error，**不是产品 finding**。可用的 Core parity 表达式/fn/scene 已过；task 的简单形式在探索中成功，更复杂未定义事件/错误语句分隔未计入验收。修正后的 probe 源保留，最终只复跑与产品两缺陷直接相关且函数内容未变的三个 case（跳过该探索性 parity case），确认 1 pass/2 fail；无从失败控制得出 Core 支持退化结论。
- 为重放临时将 probe 置于 UI compiler 的 tests 目录，之后已移出；最终三仓 clean。部分失败编译缓存清理被自动审批拒绝（`blocked by policy`，未给更细理由），未继续尝试替代删除；不删除任何已有成功产物/工作树。

## 重放与下一角色

1. 将 `review841_probe.rs` 复制到**原 UI 实施树** `crates/auto-ui-compiler/tests/review841_probe.rs`（测试临时接入，不建新树），RUSTC_WRAPPER 置空。
2. UI 树运行 `cargo test -p auto-ui-compiler --test review841_probe review841_public_generators_enforce_domain_validation -- --nocapture --test-threads=1`，以及同入口 filter `review841_codec_roundtrips_supported_nested_views`。当前两项均红；基础 codec 与合法单 widget 对照绿。
3. work 同树修复本轮全部 findings、补永久反例/相邻合法控制、处理触面格式并提交；刷新受影响来源/SD/ready receipt，再交受影响复审。未变证据可按 hash/配置/输入一致性复用。

本次重开 T-02/T-05/T-06/T-08/T-09，保留 T-01/T-03/T-04/T-07 的具体已完成产出，主计划 **executing 4/12**。统一裸 `cargo t` 与适用 `tv/tt/tu`、三仓统一交付门仍 pending，由修复后的 final pass 之后的 merge 角色在冻结输入执行；tf 只按批量到期规则在主检出串行。当前不得合入、归档或放行 U841。


存证说明（merge）：Git 将可读差量规范化为 LF；若核对原复审 CRLF 字节的 SHA256，解码 [原字节 base64](spec-delta-reviewed.crlf.base64.txt)。行尾映射与 hash 见 [存证映射](../merge-r1/evidence-line-endings.json.txt)，原 verdict 与规范内容未改。
