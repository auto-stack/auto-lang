# PLAN-784 T-04 迁出入口清单（重放基线 2026-10-15）

依据：777 边界门 ownership 表（27 项，`.autoos/module-boundaries.toml`）+
[调研报告 §4.6/§5.2](../../../reports/v06-modularization-research.md) +
[拆分路线图](../../../reports/v06-extraction-execution-roadmap.md)。
重放基线：auto-lang master `7d51845ff`（模块边界冻结基线同步锚定此提交，39 声明边）。
本表是**迁移入口登记**，不是已迁移声明；现有兼容边按当期基线记录，本计划不消除任何边。

## 1. 迁出候选路径 → 未来 owner → 资源/生成物/测试/消费者

| 迁出候选（auto-lang 内） | 未来 owner（exit 阶段） | 资源 / 生成物 / 测试 / 消费者 | 在途计划重叠 |
|---|---|---|---|
| `src/vm/native.rs`（编辑器/clipboard/scroll 直调 ui） | auto-ui（R2） | natives 测试在 test/vm；frame provider 已由 779 外置为 R2 首个探针 | **781**（新增 scroll 原生族+native 验收通道）、**785**（clipboard 2925/2926 真迁，触及 native_catalog.rs/host_capabilities.rs） |
| `src/vm/ffi/stdlib.rs`（MediaAssetKey 等 UI 类型） | runtime+auto-ui（R2） | 媒体资产类型消费面在 ui/gallery 样例 | 781（UI/VM 触面） |
| `src/vm/ffi/term_engine.rs` | runtime+auto-ui（R2） | term_engine_loader 装配；term 样例 | — |
| `src/a2r_std.rs`（frame_bench/code_editor 调用） | runtime（R2） | a2r-std 生成 runtime 消费方（auto-ai 外部抽查） | — |
| `src/lib.rs`（WidgetRegistry/DynamicComponent/ui_build 门面） | auto-ui（R4） | UI SDK 门面；examples/ui 381 受管文件按族归属 | 781（ui/design、ui/iced 触面） |
| `src/aura/validate.rs` | ui-core（R4） | AURA schema `docs/schema/aura.at` + `stdlib/aura`（62 文件） | — |
| `src/back_proxy.rs`（MediaIndex/PhotoEntry） | app-host（R4） | 桌面宿主样例（examples/desktop-host，auto-os/desktop.ps1 外部消费） | — |
| `src/database/mod.rs` | lang-core（R4） | — | — |
| `src/ast.rs`（UI 兼容 AST 声明） | auto-ui（R8，compatibility 退出） | 全语言语法对照（Core view 参数/task on/Store/Godot 样例必须保留） | — |
| `src/dialect.rs` / `src/dialect/ui.rs` / `src/parser.rs` | lang-core / auto-ui（R1） | 778 ParserBridge model pilot 已立缝；parser 测试族 | — |
| `src/ui/app_registry.rs` | project-model（R3） | 780 轻项目模型 + pac 对拍 fixture（test/fixtures/modularity/project-model/T04-OWNER-MAP） | **786**（display reader 收口，pac 标题/展示字段读取） |
| `build.rs`（UI 扫描构建） | lang-build（R6） | producer fingerprint/cache 身份（Design 37 §6） | — |
| `crates/auto/src/main.rs`（auto CLI 顶层） | auto-man（R5） | CLI 命令面（Run/Build/Ui/Bp/Tauri/Serve 路由） | — |
| `crates/auto-man/src/{vue,rust_ui}.rs`（19,243 行 UI 工程编排） | auto-ui tool plugin（R5） | auto-man/assets 376 文件中对应模板资产；gallery/wm/image_viewer | **786**（读取侧先收口） |
| `crates/auto-man/src/{pac.rs,builder/}` | man-core（R5） | pac.lock 契约；builder/{cargo,ninja} | 786（pac_scalar 对拍） |
| `crates/auto-man/src/{api_gen.rs,http_service.rs}` | lang-service-adapter（R5） | api_gen 8,043 行；776 HTTP 资源契约保留 | — |
| `crates/auto-gen/src/lib.rs` | man-ecosystem（R5，模板核心与 Auto 模板 adapter 分开） | asset::Templates（AutoInterpreter） | — |
| auto-cache（lang-infra 域） | 语言仓保留（R6+ 深化） | 缓存存储/生命周期 | **783**（storage.rs 产物契约修复中）、**787**（AC 语法增量 CST 草案） |
| auto-atom / auto-val（data 域） | 数据域只依赖更低层（独立仓名称待定） | auto-ai 外部消费 auto-atom/auto-val/a2r-std | — |

外部消费者（改 SDK/runtime 导出需迁移或兼容窗口，777 external-consumers 组）：
`../auto-ai`、`../auto-down`（jade-garden 绝对 path 依赖）、`../auto-os`（desktop.ps1）、
`../auto-shell`（关默认 feature 依赖）、`../auto-term`（at-gen → language runtime）。

## 2. 重叠判定（在途计划 vs 迁移窗口）

| 在途计划 | 状态（2026-10-15） | 与迁移面重叠 | 编排约束（路线图既定） |
|---|---|---|---|
| PLAN-781 reader-runtime | executing 18/26（lang-781 树） | scroll 原生族（→C 波必须包含）、ui/design、ui/iced、stdlib http-server、VM handler budget | UI 实际路径迁移以 781 最终 review+merge 为硬门 |
| PLAN-783 ac-artifact-probe | 修复中（lang-783 树，复审附件在主检出写入） | auto-cache/storage.rs、AC/ACC/HIR 实验链（J 波相邻） | 隔离实验，非 UI/Man 搬迁前置；不与生产替换混淆 |
| PLAN-785 clipboard-text | drafting | native.rs/clipboard 2925/2926 真迁 | 入场=784 完成 + 781 宿主热点释放 |
| PLAN-786 display-reader | drafting | pac 展示字段读取、project-model | 入场=784 完成；不依赖 785/781，无同文件写入时可独立 |
| PLAN-787 ac-incremental-cst | drafting（主检出草稿） | AC 实验语法链（J 波相邻） | 与搬仓线无生产路径冲突 |

## 3. 777 边界门刷新记录（本计划执行）

- 冻结基线 `.autoos/module-boundaries-baseline.json` 重锚 `7d51845ff`
  （旧锚为 rebase 前旧哈希，777 归档时已登记为常规运维待办）；声明边 36→39：
  新增 auto-man→auto-project-model、auto-project-model→{auto-atom,auto-val}
  （780 交付面）与 auto-lang→autodown-core（optional，根外 path 依赖显式入册）。
- 刷新后 strict **pass**（0 违例，2 grandfathered：auto-gen/auto-man→auto-lang
  巨包依赖，owner+exit_phase R5 在册）。
- 文本候选漂移记录（不判红）：core_to_ui 886→891、man_to_language 571→573
  （777 后 781/783 期合入的受管变更所致，人工确认项）。
- V04 反控：基线缺失 / 配置缺失 / 根缺失三种必需输入不可读场景均显式
  exit 3 失败，不吞为空（attachments/784/t04/v04-negative-controls.txt）。
