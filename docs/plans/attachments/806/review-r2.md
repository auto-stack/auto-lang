# PLAN-806 最终复审 R2：needs_fix

2026-10-18（Asia/Shanghai） / stage: review / review_scope: final。
独立检查会话，未参与本计划实施；无额外 agent、无虚构模型或会话身份。
上一轮 R1 仅做送审就绪核查；本轮为首次对实际实施提交的完整最终审查。

## 基线与结论

- Plan revision **1**，送审时 `execution_done 6/8`。
- 实施树 `D:/autostack/.wt/lang-806/auto-lang`，branch `codex/plan-806-dev`。
- reviewed_commit：`a20fcf6fe9885102d0059c04c9ac1e0551a3e3dc`；
  base_commit：`5d8834bcc9cb4bde66ca9b610259036b82642227`。
- 799/801 均已合入；兄弟 auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- 实施树入口 clean。配置/源码/依赖/库产物/规范内容哈希：
  [review-r2-baseline.json.txt](review-r2-baseline.json.txt)。
- 冻结实际四份 Spec 备稿：
  [review-r2-spec-delta.patch](review-r2-spec-delta.patch)，SHA-256
  `dcd77cffcd1d568c5c4b5b4471ff9018bea6955e3a775ba6c6bcf4d43164f820`。

**R1 的 F806-R1-01 已关闭**：真实实施树/提交/轻 crate/consumer 与收据现在齐备。
抽取边界、单源 model 策略、实际生产调用委托成立，但 msg 前瞻与旧行为不等价，
且规范增量与格式仍有缺口；本轮不授 pass。

## F806-R2-01（P2，AC-02/T-01/T-02/T-05/T-06）：前瞻改变文法并重复副作用

`crates/auto-ui-dialect/src/declarations.rs:87–97` 用 `checkpoint → advance → restore`
替代旧 `lexer.next → push_token`。桥面 advance 调用 `Parser::next`，后者
**跳过注释、收集 pending_docs、记录 lexer 错误**；restore 只恢复游标/五旗标，
不恢复 pending_docs/errors。这不是只有词法错误“记录时序提前”的微残差。

| case ID / 原始输入 | 冻结旧规则 / 旧臂观测 | 当前提交实跑 |
|---|---|---|
| comment-before-colon：`msg { Change(value /*x*/ : str) }` | 原始 lookahead 看见 CommentStart，按位置类型解析，最终在 `:` 报 expected RParen，span offset25/len1 | **成功**，payload_names=[Some("value")]，扩大了默认文法 |
| doc-before-rparen：`msg { Set(str /// payload-doc\n) Go }`（`\n` 为换行） | 失败位置保持，pending docs 只有一份 `payload-doc` | pending docs 变成 `payload-doc\npayload-doc`，失败恢复仍留下重复副作用 |
| lex-error-after-type：`msg { Set(str \) }`（单反斜线） | lookahead 错误后原算法继续原路径，最终一个 UnknownCharacter，offset14/len1 | **两个**同位置 UnknownCharacter；restore 后再次记录同一个错误，会影响错误数量/上限与恢复 |

复现源码：[review-r2-probes.rs](review-r2-probes.rs)，包含上述三例与九个对照，
调用公开 `Parser::parse_msg_decl/parse_model_block`，同时打印 result/errors/docs/cur。
当前输出：[review-r2-probes-new.log.txt](review-r2-probes-new.log.txt)；
无 UI feature 输出：[review-r2-no-ui-probes.log.txt](review-r2-no-ui-probes.log.txt)。
纯 named、纯 positional、注释在 `)` 前、model 四注解/var、默认裸字段拒绝、未知
注解原文+位置等对照保持。未执行已知会无界循环的缺 `}` 输入。

旧行为证据分两层：准确读取 `git show 5d8834bcc:crates/auto-lang/src/parser.rs`
的完整旧算法、同版本 `next/skip_comments` 与 lexer 注释/错误路径；另以主检出
已有 `libauto_lang-1e0c89b08e057be7.rlib` 实跑同一消费者，输出
[review-r2-probes-historical.log.txt](review-r2-probes-historical.log.txt)。
**后者是历史预构建辅助对照，不声称刚重编了 base SHA**；其 SHA-256 已冻结，
精确旧规则依源码审计建立。当前实现的运行证据绑定本树 fresh 定向测试所用
`libauto_lang-1420be846cb3eeab.rlib`，源码、库身份见 baseline。

违反的是原合同 §5“不得收紧/扩展文法”“错误文本/阶段/位置保持”以及 V02/03
lookahead/恢复要求。允许使用 checkpoint 这一实现形态，不等于批准它引入行为变化；
work 收据与 SD-01 中“旧臂后续本会记录，差异仅时序”的解释被上述反例推翻。

修复：在原树恢复原始 token 前瞻及错误/注释消费语义；若实际端口不足，依 T-01
约定具名补通用能力及其 Core/兼容测试，不可搬回 Parser 留假抽取。
收编三例及相邻 named/positional/comment/doc/error/恢复对照；同步修正收据和 SD，
不能将重复诊断或新接受文法登记为已获批准的债务。

## F806-R2-02（P2，AC-04/T-05/T-06）：权威交付闭包 Spec 漏新 crate

生产 `crates/auto-man/src/rust_ui.rs:3988` 已把 `DELIVERY_CRATES` 从 12 改为 13，
正式 driver 也实际生成 14 packages（13 受管 + 应用）。但
`docs/specs/repository-boundaries/generated-ui-distribution.md:42` 仍列
**12 crate**，只写到 PLAN-801 的 auto-parser-api，遗漏必需的 auto-ui-dialect。
本计划 impact 与 SD-01～04 也未列该受影响权威规范。

这不是交付代码漏包：独立正式 driver 已验证实际新包进入闭包；缺口在知识增量。
需补该目标的 12→13 及新依赖边说明，将路径纳入 supersedes_spec_components 和具名 SD，
按本版实际交付清单校对，避免后续消费者依旧规范复制出不完整闭包。
不要发布到主检出的 canonical Spec/ledger；在实施树备稿并提交送复审。

## F806-R2-03（P2，health/T-05/T-06）：三份新增测试文件未通过格式检查

针对本次新增/修改 Rust 文件执行 `rustfmt --check --edition 2021`，退出 **1**：

- `crates/auto-lang/src/tests/plan806_ui_declarations.rs:162`；
- `test/fixtures/modularity/ui-declarations-consumer/src/lib.rs:43/50`；
- `test/fixtures/modularity/ui-declarations-consumer/tests/v01_light_grammar.rs:56/85/92/132`。

[原始差异](review-r2-fmt.log.txt)。新文法 crate 两源文件与 pilot 委托文件的 scoped
format 通过。请仅格式化受影响文件并提交；不在 review 中直接修代码。
`cargo check -p auto-lang` 的 411 warnings 与 work/801 基线数量一致；新增轻包零警告。
未把全仓既有 warning 都归因本计划，也未宣称全仓零警告。

## 本轮独立验证

| 命令/入口 | 结果 | evidence |
|---|---|---|
| `cargo check -p auto-ui-dialect` | pass，零警告 | review-r2-check-light.log.txt |
| `cargo check -p auto-lang` | pass，411 既有警告 | review-r2-check-lang.log.txt |
| `cargo test --manifest-path test/fixtures/modularity/ui-declarations-consumer/Cargo.toml` | 9/9 | review-r2-light.log.txt |
| Git Bash `…/ui-declarations-consumer/verify-deps.sh` | pass，normal/build/dev 轻闭包无 auto-lang/Man/VM/renderer | review-r2-deps.log.txt |
| `cargo test --manifest-path test/fixtures/modularity/ui-dialect-consumer/Cargo.toml` | 48/48（6+12+8+12+4+3+3） | review-r2-driver.log.txt |
| `cargo t plan806` | 7/7 | review-r2-plan806.log.txt |
| `cargo t msg model dialect` | 45/45 | review-r2-scoped.log.txt |
| `cargo t --ignore-default-filter test_vmodel_contract_child_quoted_emit test_quoted_component_event_multi_arg_handler test_msg_multi_param_enum_output test_017_decorators` | 4/4 Vue/Rust/Ark 定向生成，未更新金样 | review-r2-codegen.log.txt |
| `cargo build -p auto-lang --lib --no-default-features --message-format=json` + 同探针 | 构建 pass，库 features=[]；无 renderer 时仍解析 msg/model，同样复现 F01 | review-r2-no-ui-build.jsonl.txt / review-r2-no-ui-probes.log.txt |
| `python scripts/modularity-audit.py --mode strict` | pass，0 violations / 2 grandfathered | review-r2-boundaries.log.txt |
| `rustfmt --check …` | fail（三文件） | review-r2-fmt.log.txt |
| 公开 API 的 12 个语法/诊断对照 | 发现 F01 三种差异，其余对照保持 | review-r2-probes*.log.txt / review-r2-probes.rs |
| 生产 `build_dynamic_component` 真实 use 文件装配及 handler | model Int(1)→Int(2)，变更导入文件未知注解→模块诊断含 Bad806 | review-r2-import.rs / review-r2-import.log.txt |
| 792 正式 driver fresh/locked/重定位/负控 | **25/25，exit0**，fresh123s、重定位locked113s；13受管+应用=14包，缺输入/坏hash/失败保旧/恢复/sidecar等正式正反控全部过 | review-r2-delivery.log.txt / review-r2-delivery-receipt.json.txt |

工作树源码检查：顶层 UiDialect 与 widget/store/component 的旧 callsite 仍到 inner，
inner 现在仅建桥委托；注解旧方法已删；pilot 只调用新 crate 的 ExternalSubset，
没有第二份字段算法。无逆向依赖/镜像 AST/Parser/Pratt/type parser。
迁位/净删与剩余 widget/view/on/actions/style/computed/escape hatch/组合宿主依赖边
已如实登记。未发现额外 workaround 或未披露的文法范围削减。

### import 与消费证据的边界

新增树内 `import_created_parser_uses_default_declarations` 只读取真实文件再直接
Parser::from，没有走 use 装载；本轮没有把该 7/7 的名字当成 import 链的证据。
独立 [review-r2-import.rs](review-r2-import.rs) 通过**生产公开入口**装配含真实
`use child: Imported` 的两文件项目，再调用导入组件的 Bump handler，断言 model 状态
从 1 改为 2；随后实改 child.at 中的注解，模块错误注册表含 Bad806，证明实际读取该文件。
该补验已完成本项检查，不把“原测试没有覆盖”另立阻断 finding；建议修复时收编到正式测试。

探针首版误要求 build_dynamic_component 对坏模块整体返回 Err，但实测 API 返回 Ok、
模块错误表已登记；本轮将 oracle 校准为“真实导入+模块诊断”，保留
[首次原始日志](review-r2-import-initial.log.txt)，未把这项超出本计划的额外
错误传播承诺强加给实现。VM 消费也由 45 测中的 plan051_emit_msg_variant_routes_to_parent
（带参 msg 到父 handler 后状态变更）及本次导入探针实际执行支持，未以未来 tv 代替。

### 测试时序

本轮没有运行裸 t/tv/tt/tb/tu/taa/tf，也没有 docs_gen 整档。work 收据宣称曾在
work 阶段运行整档 tu，这与 2026-10-14 的时序裁定不符；本轮仅选四个必需消费测试，
不再复跑整档，也不将旧 tu 收据视为未来冻结合入门已过。
一次 deps 的 `bash` 默认宿主返回 127，改用已安装 Git Bash 后通过；一次 driver
scratch 误置树内被既有守卫直接拒绝，改用组内树外目录完成正式运行。均为本轮工具
调用修正，不归因实现；无源码修复/额外 reviewer checkout/agent。

## AC / Spec delta / 路由

| AC | tasks | result | evidence / boundary |
|---|---|---|---|
| AC-01 | T01/02/05 | pass（独立实现/闭包部分） | 单源算法+轻 consumer9+闭包守卫；算法保真问题归 AC02 |
| AC-02 | T01/02/04/05 | fail | F01，三种副作用/文法差异；48 pilot及其裸字段/恢复正反控通过 |
| AC-03 | T01/03/04/05 | partial | 实际顶层/结构/import/无UI/VM/生成消费通过；共有算法的F01阻止完整保真通过 |
| AC-04 | T04/05/06 | partial | 单源/剩余边/777与交付实现通过，F02权威Spec闭包遗漏，F03health未过 |
| AC-05 | T07 | fail | 当前 needs_fix，非最终通过 |
| AC-06 | T08 | pending | merge-owned，未合入/归档/清树 |

SD-01 依赖与单源范围可核实，但前瞻等价/微残差说明须纠正；SD-02/03/04 的
调用接线与仍未迁 UI 面正确，全面“默认文法零漂移”结论须待 F01 修复。
新增包规范还将“缺 } 经 expect 报错”与已知无 EOF 守卫怪癖并列，需明确其适用
边界，不应把目前会无界循环的输入描述为已覆盖错误出口；本项并入 F01 的收据/SD 校准。
F02 要求补权威交付规范的路径/SD/impact。未在 review 发布 canonical Spec 或 live ledger。

保留已完成 **T-03/T-04**；重开 **T-01/T-02/T-05/T-06**，T-07/T-08 未完成；
Plan 改 `executing 2/8`。next: 原实施树 work 修复三项 finding、收编反例并提交，
然后受影响复审；不擅自改变 plan_revision 或缩减原合同。
冻结裸 `cargo t`+`cargo tv`、实际新增触面档（ui_gen/trans 源未改时无额外 tu/tt）、
Spec/ledger/归档/guard 与批量到期判断均 **merge-owned pending**。

落盘终核：HEAD 仍为 a20fcf6fe，全表输入 SHA-256 无漂移、实施树 clean；
复审文档差异 `git diff --check` 通过。独立交付 scratch 在保存正式25/25收据后，
核定绝对路径、wt-guard clean，再用原生文件操作删除；
[cleanup收据](review-r2-scratch-cleanup.log.txt)。保留806实施树和分支，不执行合入清理。
