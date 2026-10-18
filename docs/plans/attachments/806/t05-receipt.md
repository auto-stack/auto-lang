# PLAN-806 T-05/T-06 收据（work 阶段验证矩阵与收口）

2026-10-17 / worktree `D:/autostack/.wt/lang-806/auto-lang` / branch `codex/plan-806-dev` /
base `5d8834bcc`（master 分叉点） / 代码链 `970960b4a→557e0a176→8093301b8→8a86fe323→504ca0e0d→af3748db7`（+本收据提交）。
环境注记：本机多会话并行，sccache 端口冲突在案（801 复审同源），全部 cargo 命令以
`RUSTC_WRAPPER=""` 执行（不影响测试面）。

## V01 独立闭包（AC-01）

- `cargo check -p auto-ui-dialect`：clean，**零警告**（闭包链 auto-lex→auto-ast→
  auto-diagnostics→auto-parser-api→auto-ui-dialect）。
- `cargo test --manifest-path test/fixtures/modularity/ui-declarations-consumer/Cargo.toml`：
  **9/9 绿**（消费方 toy ParseContext 驱动真算法：canonical/legacy 等价、quoted/
  多 payload/named+positional、lookahead 不吞 token、注解 flags、默认裸字段拒绝、
  未知注解错误文本、子集 NotClaimed、轻 session 面）。
- `bash …/ui-declarations-consumer/verify-deps.sh`：**PASS**——normal/build/dev 闭包
  `={auto-lex, auto-parser-api, auto-ui-dialect}`+轻传递（auto-val 等），无
  auto-lang/auto-man/auto-vm/auto-ui；语言 driver 在上层（auto-lang 侧测试）。

## V02 msg 全族 / V03 model 全族（AC-02）

冻结旧臂 oracle（既有断言，换新臂重放全绿）：

- parser.rs 内联 12/12：`test_msg_decl_unnamed_and_legacy_name`、
  `test_msg_variant_named_payload`（named-payload lookahead 冻结）、
  `test_msg_multi_param_payload`、`test_aura_model_var_keyword`、
  `test_msg_full_shellstore_declaration`（store 结构路径 15 变体）、
  `test_widget_parsing_pipeline`/`test_widget_parses_via_dialect_in_ui_scenario`/
  `test_widget_block_keyword_near_miss_warning`/`test_widget_with_routes`/
  `test_widget_watch_block`/`test_widget_expose_block`/`test_store_use`。
- ui-dialect-consumer driver **48/48**：r2_bare_field_boundary **8/8**（默认无裸字段
  反控 + 显式 pilot 裸字段正控，四象限矩阵）、v03_dispatch 12、r1_delegation 6、
  r1_scope_expiry 12、v04_snapshot_lifetime 3、v01_v05 4、v05_legacy_api 3
  （AST 投影=旧臂 oracle、注册/递归/恢复/旧 API 面保持）。
- 滤面 `cargo t msg model dialect`：45/45。
- 轻侧字面量（V01 9 例内）：未知注解错误文本逐字（含合法清单原文）。
- 历史怪癖登记（不修，新旧同形态）：`msg {`+EOF 变体循环无 EOF 守卫。

## V04 场景/所有生产调用（AC-03）

`cargo t plan806`：**7/7**——顶层默认装配（无需显式注册）、widget 结构路径
（quoted/named/注解）、component fn+store 结构路径、**真实被导入文件**
（examples/ui/018-book-reader/src/front/components/treeview.at，经 import 管线同构
`Parser::from(..).with_session(ui)` 构造解析，产物同一 AST 类型）、Core fn/on 体
护栏（`msg` 为字段名/赋值不被拦截）、未知注解错误文本冻结、默认裸字段树内正控。
派发面由 v03_dispatch 12 + r1_delegation 6（Override/Append/递归/with_session）覆盖。

## V05 消费保持（AC-03）

`cargo tu`：**867/867 过 +2 ignored**（单进程 libtest，0.91s 测试段）——vue/jet/
ark/rust 四生成器全量 + a2ark 金样（020_msg_enum、017_decorators 含 model 注解
消费等），**零金样更新**。VM/编译器面语料归 merge 门 `cargo tv`；本计划未触
ui_gen/trans 源（不追加 tu/tt 合并门）。

## V06 单源/交付（AC-04）

- 单一实现审计（grep 收据）：`Unknown model field annotation`/`model field name
  expected identifier`/`fn parse_model_field_annotations` 生产代码仅存
  `crates/auto-ui-dialect/src/declarations.rs`；`MsgVariant {`/`ModelBlock { fields }`
  构造在 Parser/pilot 均无（auto-ast 定义与 AuraMsgVariant 另型不算）；pilot
  （dialect-api-consumer）不保第二份字段算法（净删 65 行，委托子集入口），
  轻测 3/3 + verify-deps PASS。
- 777：`module-boundaries.toml` 增 `auto-ui-dialect = "lang-core"`；基线重采样
  base_revision=8a86fe323、declared_edges=62，**增量恰 2 条合法声明边**
  （auto-lang→auto-ui-dialect、auto-ui-dialect→auto-parser-api）；
  `--mode strict` verdict=**pass violations=0** grandfathered=2（既有 man 系）。
- 792：DELIVERY_CRATES 12→13（auto-man/src/rust_ui.rs）；delivery 单测 **10/10**；
  交付 driver（generated-ui-consumer/driver.py）本版完整重跑 **25/25 PASS exit0**：
  V01 fresh 独立锁 5s+patch 单源+pin 全命中+真实 build 111s（crates=14=13 受管
  含 auto-ui-dialect+app）；V02 重定位 locked build 135s+iced_widget 单源@B 内+越界
  path=无；V03 十四态负控（等长篡改/坏 hash/缺资源/未登记/漏 patch exit101/
  不兼容 pin exit101/外来目录/换装保旧/恢复后成功/sidecar 外部+根相对/raw TOML/
  邻位保留/他应用存活）+恢复绿；V04 开发路由+模板同源。收据
  [delivery-driver-receipt.json.txt](delivery-driver-receipt.json.txt)、
  [delivery-driver.log.txt](delivery-driver.log.txt)。

## 净删与迁位清单

merge-base `5d8834bcc`…HEAD：`crates/auto-lang/src/parser.rs` **−213 行**（三算法
217 行原体→4 行薄委托+注释）；`dialect-api-consumer/src/lib.rs` −65 行（第二份
model 字段算法）；新增 auto-ui-dialect crate（Cargo.toml+lib.rs+declarations.rs）、
ui-declarations-consumer fixture（4 文件）、`plan806_ui_declarations.rs`（7 测）、
777 config+baseline、792 DELIVERY_CRATES、SD-01..04 备稿（docs/specs/，待 merge 沉淀）。

**未迁清单（另立计划）**：widget 壳与 view/on/actions/style recipe/computed/setup/
expose 块文法、store/component 壳文法、`use`/import 扫描、`ParserBridge::parser()`
pub(crate) escape hatch（UiDialect 其余分支仍经其取回完整 Parser）、旧 `Dialect`
trait + by-value `BridgeCheckpoint` 兼容层。组合宿主剩余边：auto-lang→auto-ui-dialect
path 依赖（777 已声明合法边），后续 Parser/组合宿主分层再清理。

## merge 门（pending，非 work 面）

最终审查通过后冻结提交上：裸 `cargo t`（parser.rs 触 VM/编译器面 + `cargo tv`）；
ui_gen/trans 零源触面不追加 tu/tt（tu 已 work 侧全量绿）；tf 主检出到期判定；
docs_gen 不触发。

## R2 修复轮更正（2026-10-18，取代下方原"微残差登记"叙事）

F806-R2-01 证实 checkpoint+advance/restore 前瞻**不等价**（注释跳过/docs 收集/
词法错误记录副作用扩散 + 注释分隔伪命名载荷文法扩大 + 词法错误翻倍），原
"差异仅时序"的解释作废。修复：端口新增具名通用能力 `ParseContext::
peek_next_kind`（原始单 token 前瞻，零副作用，auto-lang 侧 `port_peek_next_kind`
原语=旧 lexer.next+push_token 逐字同构）；三反例+相邻对照+peek 语义+真实
use-import 链（收编复审 review-r2-import.rs）已收编为 plan806 永久回归
（r2_lookahead_comment_before_colon_stays_positional / r2_lookahead_doc_
collected_once / r2_lookahead_lex_error_recorded_once / r2_lookahead_comment_
before_rparen_ok / r2_lookahead_fstr_start_stays_positional / r2_port_peek_
next_kind_is_side_effect_free / r2_real_use_import_chain_uses_default_
declarations），oracle=R2 冻结新旧探针日志。
另更正时序注记：work 阶段曾整档跑 tu 与 2026-10-14 测试时序裁定不符（越权），
该 867/867 结果不作为门禁证据（生成消费以 R2 复审四个定向金样为准），
冻结 tu/tt 归 merge 触面判定。
（历史登记保留于下，仅作过程记录。）

## 微残差登记（T-01 D2 原叙事——已被 R2 修复轮更正推翻，见上）

named-payload lookahead 窗口内 lexer Err：旧码 `if let Ok(tok)` 静默吞（positional
None），新实现经 `advance` 提前把同内容 lexer 错误记入 errors（restore 不回滚诊断）。
该错误在旧臂后续 parse_type 推进时本会记录，差异仅时序/error_limit 提前触发；
计划 §5 明示 checkpoint+advance/restore 为认可等价形态。

## R3 修复轮（2026-10-18，F806-R3-01/02）

### 诊断简表（按 R3 报告要求：剩余不变量/此前漏检原因/适用状态路径/设计与 oracle 调整/当前结果）

| 项 | 内容 |
|---|---|
| 剩余不变量 | 端口探测方法的公开合同与实现必须一致：成功路径零副作用/幂等/与首次 advance 一致；**失败路径**与旧回推前瞻失败消费协议逐字一致（是否推进输入由 Lexer 失败模式决定、错误丢弃不记录、不承诺幂等），且不得为"纯 peek"承诺改变旧 msg 错误语义 |
| 此前漏检原因 | R2 修复只测了成功路径（Colon/CommentStart/DocComment）与**恰巧不推进输入**的 UnknownCharacter（`\`），未枚举词法失败模式的空间——漏掉会推进输入的字符字面量失败（`'\q'` 非法转义：Lexer::char 消费失败前缀）；oracle 选择依赖了失败模式的偶然性质而非合同 |
| 适用状态路径 | probe × {成功 token（含注释/doc/Ident）、非推进型失败、推进型失败（非法转义/坏字符字面量）} × {单次、重复调用} × {probe 后 advance 的错误记录/游标/续读} |
| 设计/oracle 调整 | 端口方法更名 `peek_next_kind`→`probe_next_kind`（名实统一：探测非纯前瞻），合同改为双路径精确定义（成功=零副作用幂等；失败=旧回推前瞻失败消费协议，明示不承诺幂等）；oracle 补齐推进型失败的重复调用/后续消费断言（钉 R3 探针观测：`'\q'` 首 None 次 Some(Ident) 后 advance 得 Ident("q") 零错误；`'ab'` 次 Some(Ident) advance 得 "b"） |
| 当前结果 | plan806 18/18（新增 r3_probe_* 4 例永久回归）；无-UI 编译我方文件零错误（F3-02 门控后 plan806_ui_declarations 在 --no-default-features 下 0 错误，剩余红均为 plan712 等既有模块，复审已注不归因本计划）；重放 light/driver/内联/滤面见下 |

### 修复内容

- F806-R3-01：`auto-parser-api` `ParseContext` 方法更名 `probe_next_kind` 并按双路径精确定义
  （见 [auto-parser-api/project.md](../../../../../docs/specs/auto-parser-api/project.md) 端口与快照契约节）；
  auto-lang 原语同步更名 `port_probe_next_kind`（实现不变=旧 lexer.next+push_token 逐字同构）；
  declarations.rs/lib.rs/SD-01/auto-parser-api 规范同步校准（含 R3 指出的陈旧 pilot 位置/12 成员闭包文案）。
  **不改 msg 文法行为**——旧 12 例消费者输出与 R2 历史旧臂逐字一致（R3 复审已验，本轮未动算法主体）。
- F806-R3-02：`r2_real_use_import_chain_uses_default_declarations` 增 `#[cfg(feature = "ui-iced")]`
  （与 `build_dynamic_component` 同条件）；不依赖 UI 的基础文法测试保持无门控。

### R3 轮重放（worktree，RUSTC_WRAPPER="" 因机内 sccache 端口冲突在案）

`cargo t plan806` **18/18**；轻 consumer **10/10**+verify-deps PASS；driver **48/48**；
冻结内联 **12/12**；滤面 `msg model dialect` **45/45**；`cargo check -p auto-ui-dialect` 零警告、
`cargo check -p auto-lang` 411 既有警告零新增；触及文件 rustfmt --check 过。
（792 交付 25/25 仍为 R2 历史证据：API/Parser 源已变不冒称新 SHA 重编成功，最终冻结面
验收由 merge 门+终审确认影响后补/复用。）
