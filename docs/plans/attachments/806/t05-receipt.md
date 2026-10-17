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
  [delivery-driver-receipt.json](delivery-driver-receipt.json)、
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

## 微残差登记（T-01 D2，验收内）

named-payload lookahead 窗口内 lexer Err：旧码 `if let Ok(tok)` 静默吞（positional
None），新实现经 `advance` 提前把同内容 lexer 错误记入 errors（restore 不回滚诊断）。
该错误在旧臂后续 parse_type 推进时本会记录，差异仅时序/error_limit 提前触发；
计划 §5 明示 checkpoint+advance/restore 为认可等价形态。
