# PLAN-841 T-07/T-08 定向验收与测试 owner 映射

## 1. V-01 — Core 独立消费与零反向承载

- `ast-model-consumer`：3 项构造/身份测试 + Core/Scene golden 通过；修复前
  `auto_ast::ast::WidgetDecl` 不可解析是有效负控，证明旧 UI 字段已退出。
- `lex-consumer` 4 项、`dialect-api-consumer` 3 项、
  `language-sdk-consumer` 的 Core parse→compile→VM/trans 与裸 parse UI
  拒绝通过；`three-repository-consumer` 与其 verify-deps 通过。
- 上述 fixture 的 normal/build/dev `cargo tree` 均无
  auto-ui-runtime/compiler/tools/auto-man-runtime。777 strict 在干净
  lang/UI/Man 与固定 down 来源上显式重采样后 `violations=0`。
- 旧 36-case golden 未再生：Core/Scene 段留 auto-ast owner，9 个 UI 段由
  新 `ui-ast-model-consumer` 重放。`UseWeb` Display/ToAtom 差额先红后修，
  最终两段均逐字节一致。

## 2. V-02/V-03 — 文法、lowering、codec 与寿命

- auto-ui-dialect 91 单测 + 2 doctest；SDK `plan810` 21 passed/1 ignored；
  auto-lang-core `plan841` 与 scene 门测试通过。
- compiler `ui_ast::tests` 4 项：代表 lower→VM 与四生成器通过；compiler
  doctest 19/19 从旧 auto_lang 路径迁移后通过。
- `ui-grammar-consumer` 6 项、`ui-dialect-consumer` 全套串行通过；
  外部 model 改为 `DomainDialect<Output=UiStmt>`，非 UI ExternalDialect/
  Dialect、快照过期与跨 Parser 负控继续有效。
- V-03 既有 codec/缓存/Weak 证据保持；本次 model.rs 仅 Display/ToAtom
  投影修复后 dialect 全套复测通过。on-handler Err 现在同时恢复 on 旗标
  并退出参数 scope，同一 Parser 源尾 `arg` 未定义反例通过。

## 3. V-04 — 真实三仓与外部发行

`generated-ui-consumer/driver.py` 39/39：fresh resolve + locked 真构建、
A→B 重定位、metadata 本地 path/source 唯一、资源 hash、缺 patch/坏 pin/
外来目录/失败保留、dev 同源、随包工具 fresh/locked、真实协议消费与
失败归因全过。vue/rust/jet/ark/tauri/vscode 真实生成通过；Iced 主窗口
真实运行通过；VM Launched 端点探活、协议 cancel、端点关闭与宿主存续
通过。jet/ark 宿主构建按既有环境门控披露，不冒称工具链存在。

初轮收据绑定 lang `70f2be05b98e3d59e4080d0f444ebf6d8e67ed9b`、UI
`6623442189f208e4f806ddffe446c8196775b1d8`、down
`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。最终复验身份以同目录
`v08-792-receipt.json` 为准；投影修复不影响该初轮生成路径，但最终代码
仍以复验为交付证据。

## 4. V-05 — 测试 ID/owner 映射

| 原测试/资产 | 终态 owner 与选择面 |
|---|---|
| auto-ast `v02_projection_replay` | Core/Scene 段仍 `cargo test -p auto-ast --test v02_projection_replay`；UI 段移 `ui-ast-model-consumer/tests/v02_ui_golden.rs` |
| ast-model-consumer `WidgetDecl` 构造 | 删除 Core 残边；UI 类型构造/投影由 ui-ast-model-consumer 接续 |
| `plan810_full_ui_grammar` | 保持原测试目标；scene 门 `r04_scene_gate_prev_matches_old_two_push` 移 auto-lang-core parser tests |
| plan810 fn-store 与 Err scope 旧怪癖 | 改为新领域端口合法/平衡恢复 oracle，不保留已消除行为 |
| aura plan050 图标组件 | 仍 runtime 同 ID；改经 UI-owned 注册表，空注册表负控恢复 |
| compiler doctest | crate 路径改 `auto_ui_compiler`，19/19；7 个原 ignore 不变 |
| 外部 model/裸字段/混合方言 | `dialect-api-consumer` 与 `ui-dialect-consumer` 原 ID 继续可选；UI 用显式 Domain 组合，非 UI 旧方言独立保留 |
| compiler/ui_gen/runtime/Man | 定向入口分别为 ui_ast、plan050、真实 CLI plan816；统一 merge 门仍后移 |

## 5. 非验收红与边界

一次额外 `auto-ui-tools --lib` 宽面运行为 184 过/2 败：
`merged_api_client_crud_fallback_for_uncovered_endpoints`（实际走 api_impl
直调）与 `index_css_values_match_p1_baseline`（golden 含已不发射的
success/warning/info/error）。本计划未改这些生产路径或 golden，验收所需
迁移/生成定向例均绿；不借改写无关 golden 取得全绿，留独立基线债复核。
此前 runtime 的 3 个环境红沿用 T-05 基线归因，不记为迁移失败。

统一裸 `cargo t`、`tv/tt/tu` 与适用触面档仍为 review 后 merge-owned
pending；本文件不把它们提前标记通过。
