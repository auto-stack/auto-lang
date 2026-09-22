# AutoUI 仓分离总方案（三件套 + 双插件缝）

> 状态：📝 方案稿，待评审（2026-09-20 立档，基于同日三轮调研与函数级勘定）
> 范围：auto-lang 仓结构 / parser 方言体系 / auto-man 工具链 / AutoUI 生态资产 / 跨仓消费方
> 前置文档：[dialect-extension-diagnosis.md](dialect-extension-diagnosis.md)（方言体系诊断——阶段 1 的直接实施依据）
> 关联文档：[20-autoui-separation-architecture.md](20-autoui-separation-architecture.md)（AutoUI 进程分离远期形态，
> 本方案的协议层远期锚点）、[08-ui-systems.md](08-ui-systems.md)（UI 域现状章）、
> auto-os `docs/design/01-stage-b-desktop-migration.md`（Stage B 迁移手法先例）
> 实施方式：L2 架构级任务——本文档评审通过后分解为 6–10 个独立 Plan 顺序执行
> 债册登记：DEBTS.md `拆仓-候选` 条目（下一版规划实施）

---

## 0. 背景与根因

**历史**：auto-lang、auto-man、auto-ui 最早就是三个独立项目。AutoUI 做到一定阶段后
`widget` / `view` / `model` 等成为语言关键字，三项目从此无法分开，逐渐融合为当前的
单一 auto-lang 仓（UI 相关代码约占 `crates/auto-lang/src` 的 ~52%——约 30 万 / 58 万行；
2026-09-22 cat 口径复核勘误，原稿"90%——27.2 万/30.3 万"分母量错，09-20 当日实测
55.7 万行——PLAN-691 SD-01）。

**融合根因（定性，2026-09-20）**：不是工程滑坡，而是两处"不可插拔"逼出来的——

1. **编译器语法不可插拔**：Auto 的 parser/lexer 无法把 `widget`/`view` 这类 AutoUI
   专属关键字做成编译器"插件"。语法一旦进核心，UI 代码就必须与编译器同仓。
2. **工具链不可插拔**：auto-man（cargo 位的项目模型/依赖解析/构建编排）没有
   cargo 式插件机制，UI 命令只能编进工具链本体，于是 auto-man 也被融合。

**本方案的本质**：把这两条缝补上（方言插件 + 工具链插件），拆回三件套。补缝本身
独立于拆仓成立价值——语法扩展点是语言能力，工具链插件是生态能力。

**关键事实（大幅降低风险）**：方言插件缝已在代码中**半建成**。
`crates/auto-lang/src/dialect.rs` 存在设计完整的 `Dialect` trait（keywords 表 +
`try_parse_stmt` / `try_parse_token_stmt` 双派发钩子，已接入 `parse_stmt_inner`），
`dialect/ui.rs` 的 `UiDialect` 已接管全部 UI 关键字派发。阶段 1 是"完成一次已经开始
的重构"，不是新架构。

---

## 1. 目标架构

```
auto-lang（语言仓）                          auto-ui（AutoUI 仓）
  lexer / token / ParserBridge                 UiDialect + UI 文法（dialect/ui/）
  基础 AST / infer / VM / trans                aura / ui / ui_gen / design_tokens（库 crate）
  auto/lib/*.at 标准库                         auto-ui.exe（插件二进制：渲染器后端 + UI 命令）
  auto-man（cargo 位）                         packages/@auto-ui/widgets、examples/ui、
    pac / resolver / builder / lock / git …    blueprints、schema/aura.at、stdlib/aura、
  auto.exe（统一入口，委派缝）                 skills、UI specs/docs、四条 CI
```

**依赖方向**（无环）：

- `auto-man → auto-lang`（仅语言库，**甩掉 `features=["ui-iced"]`**）；
- `auto-ui 插件 → auto-lang + 自有 UI crate`；
- `auto-man` **编译期零依赖 auto-ui**——UI 能力在运行期经子进程插件发现。

**双插件缝的对称结构**：

| | 编译器级插件 | 工具链级插件 |
|---|---|---|
| 形态 | 进程内 trait 对象（`Dialect`） | 子进程（`auto-ui.exe`） |
| 实现居所 | auto-ui 仓的 dialect crate | auto-ui 仓的插件二进制 |
| 注册/发现 | 消费方 `session.with_dialect()` 注入 | 发现序：auto.exe 同目录 → PATH → `AUTO_UI_ROOT` 解析族 |
| 未注册/未装 | 关键字回退普通标识符 | 定向报错"此工程使用 UI 方言，需安装 auto-ui" |

**边界裁定：拆仓只需进程内 trait 插件，不做 Swift 宏式跨进程编译器插件**。拆仓的
真实约束是依赖倒置（auto-lang 不得引用 auto-ui 类型），不是进程隔离。

### 1.1 工具链缝的设计要点（动词直通式委派）

cargo 的原生插件模型是"子命令名 = PATH 上的 `cargo-foo` 二进制"，但直接照搬会要求
用户改敲 `auto ui run`，破坏全部现有消费方（auto-os wrapper 脚本、`apps.manifest`、
CI 都指向 `auto run`）。故采用**动词直通式委派**：

- `auto` 保持唯一入口，外部接口零变化；
- `Run` / `Build` 解析 pac.at（auto-man 本就解析）后，若 `scene: ui` 或 render ∈
  {vue, rust, vm, jet, arkts, tauri}，以 env 契约 spawn `auto-ui <verb> <原样参数>`，
  透传 stdio 与退出码；插件是纯实现细节；
- **env 契约**（对标 cargo 给插件设 `CARGO_*`）：核心解析并注入
  `AUTO_PROJECT_DIR` / `AUTO_TARGET_DIR` / `AUTO_LANG_ROOT` / `AUTO_TOOLCHAIN_VERSION`，
  收敛当前散落四五处的"env → 兄弟目录 → 主检出"三级解析序为入口统一颁发；
- **版本握手**：`auto-ui --version` 对照核心兼容区间，失配告警（仓内开发靠 path dep
  锁步，仅发行态真正需要）；
- Windows 细节：文件名禁冒号，`test:ui` 做不成 PATH 插件名——定点委派天然绕开。

---

## 2. 现状盘点（2026-09-20 勘定）

### 2.1 代码足迹（crates/ 内）

| 位置 | 规模 | 内容 |
|---|---|---|
| `crates/auto-lang/src/ui/` | 168 文件 / 18.5 万行 | iced 桌面渲染器（renderer.rs 单文件 3.4 万行）、desktop_protocol、rqhost、vm_bridge + handler_codegen、MCP 服务器（:9247）、autodown 编辑器、代码编辑器、终端、gpui、Win32 互操作、libmpv |
| `crates/auto-lang/src/ui_gen/` | 43 文件 / 7.2 万行 | 多后端代码生成农场：vue.rs 2.96 万、rust.rs 1.07 万、jet/、ark/、bp/、ts_adapter、widget registry |
| `crates/auto-lang/src/aura/` | 8 文件 / 0.95 万行 | UI IR（AuraWidget）+ 元素目录（226 ElementDef），schema 自根 `schema/aura.at` 加载 |
| `crates/auto-lang/src/a2ui/` | 4 文件 / 0.2 万行 | A2UI 协议桥，**全仓零消费方**，可自由迁/删 |
| `crates/auto-lang/src/design_tokens/` | 4 文件 / 0.25 万行 | 语义 token 单源 + theme/recipe 解析 |
| `crates/auto-man/` | 3.5 万行 | 事实上的 AutoUI 编排器：vue.rs 1.16 万 + rust-embed 内嵌 376 个资产文件；`auto-lang { features=["ui-iced"] }` |
| `crates/autoui-skill/` | 非 Rust crate | AI 技能文档包，随迁即可 |
| parser.rs UI 文法 | ~4,700 行 | 详见附录 A（纯 UI ~3,900 直搬 + node 表达式族 545 深纠缠 + Godot 269 非 AutoUI） |
| AST UI 节点 | 1,921 行 | ast/ui.rs 1098 + store/on/route/tag/grid 五件 |

轻耦合消费方：`auto-lsp`（补全引 `aura::schema`）、`auto-cosmic`（`ui-headless`）、
`auto` CLI（Run/TestUi/Ui/vue 等命令，仅走 `auto_lang::ui*` 公共 API）。

### 2.2 生态资产（crates/ 外）

- **示例**：`examples/ui/`（36 编号 demo）+ 约 20 个兄弟 UI 目录（ui-gallery、
  vue-gallery、widgets-gallery、jet-*、bps-gallery、charts-gallery、quickstart、
  unified-demo、split-demo、multi-frontend-demo、desktop-host、remote、shared 等）。
- **前端包**：`packages/` 下 `@auto-ui/widgets`（发行组件库）、drawlist-renderer、
  playground-vue、forge-ui、lab-ui。
- **Schema/协议**：`schema/aura.at`（5,986 行，组件目录唯一声明源）+
  `projection-protocol-v1.md`。
- **内容资产**：`blueprints/`（65 文件）、`stdlib/aura/`（62 文件）+
  `stdlib/auto/{ui,widget,builder}.at`、`test/ui/`（43 fixture 工程）、
  `tests/screenshots/`。
- **工具/技能**：`.agents/skills/autoui-verifier/`、`crates/autoui-skill/`、
  scripts 四件（ui-layout-parity / style_palette_guard / gen-lucide-table /
  schema_drift_audit）、tools 三件（shadcn-snapshot / gen_tailwind_manifest /
  parity_shot_diff）。
- **文档**：`docs/specs/` 约 7 个纯 UI 模块；`docs/design/autoui/` 24 篇；
  `.autoos/specs.json` 616 条中约 163 条（26%）UI 相关；600 系计划约六七成。
- **CI**：build-ui-examples / build-vue-gallery / build-bps-gallery / deploy-website。
- **构建配置印记**：`.cargo/config.toml` 的 Windows 32MB 栈链接参数（Plan 408）与
  `RUST_MIN_STACK=16MB`（Plan 423）为 UI 专属。

### 2.3 外部消费方

- **auto-os**（最重）：apps/、ui-gallery、widgets-gallery、`desktop.ps1/sh` wrapper、
  `apps.manifest`、CI 跨仓 checkout——全部经 `auto` CLI + 三级解析约定。
- **auto-down**：jade-garden 为完整 AutoUI 应用，其后端以**绝对路径**依赖
  `auto-lang` crate 并内嵌 VM。
- **auto-term**：生成的 rust-workspace 路径烤死在已失效的 worktree
  （`compute_auto_lang_rel_path` 钉死生成时位置的实证）。
- **auto-shell / auto-ai**：只依赖非 UI crate（auto-atom/auto-val/a2r-std），不受影响。

---

## 3. 耦合分析摘要

### 3.1 三焊点

1. **parser UI 语法**（~4,700 行 + AST 1,921 行）——经方言插件迁出（本方案核心，
   见阶段 1 与附录 A）。
2. **VM 就进程托管**：`ui/vm_bridge.rs`（3,778 行）与 `ui/handler_codegen.rs`
   （2,891 行）直接调 `vm::Codegen` / `Linker` / `AutoVM`——VM 模式渲染即进程内
   VM 宿主。拆仓后变为 auto-ui 对 auto-lang 的库 API 依赖（现为 pub，需冻结契约）。
3. **CLI/工具链归属**——动词直通式委派解决（§1.1）。

### 3.2 反向引用（核心 → UI，全部可剪）

`aura/validate.rs → ui_gen::widget::AUTO_IMPORTED_WIDGETS`（一个常量，同迁后自然
消失）；`back_proxy.rs` 复制的 `MODULE_INIT_FN` 常量（约定耦合，提取共享）；
lib.rs 中引用均在 `#[cfg(test)]` 区。核心反向引用**无一是类型级**，这是拆仓可行的
根本依据。

### 3.3 先例与教训

| 先例 | 手法 | 教训 |
|---|---|---|
| Plan 325（auto-ai 拆出） | 新仓无历史 + path dep 重指 | 墓碑注释行习惯由此来 |
| Plan 330（auto-shell 拆出） | filter-repo 保历史 + 多 workspace | **多 workspace 漂移是后患**（REVIEW-2026-08-26 §A-2：lock 双版本、target 7GB 重复）；"隔离单位是 workspace 非 repo" |
| Stage B（auto-os Design 01） | 设计先行 → 先解焊路径三级解析 → pin 快照 + hash 锁 → INDEX 指针 → V1–V9 矩阵 | **最成熟手法，本方案主干采用**；"先做路径可解析，再搬文件" |

---

## 4. 分阶段方案

> 总原则：每阶段独立成 Plan、master 保持绿、行为零变化优先于结构搬家。
> 阶段 1–2 在 auto-lang 仓内完成（不拆仓已净赚）；阶段 3 起新仓。

### 阶段 1：方言迁移完成（仓内，行为零变化）

**前置**：[dialect-extension-diagnosis.md](dialect-extension-diagnosis.md) 评审通过，
按其三轴分析（轴 A 语法）执行。

**1A 纯 UI 文法搬迁**（预计 1 个 Plan，机械搬运为主）：
- 附录 A 所列纯 UI 函数族（A1 widget 声明族 807 行、A2 生命周期块族 412、
  A3 store 声明 102、A5 声明族 987、A6 view 族 1,182、A7 事件/handler 族 585、
  get_primary_prop 43——合计约 3,900 行）从 parser.rs 迁入
  `crates/auto-lang/src/dialect/ui/` 目录（同 crate，先不跨 crate）；
- 升格 **ParserBridge pub 面**（附录 A §6 清单：token 游标族、产产生族、
  `parse_body`（含 `is_node` 穿线处置）、lexer push_token save/restore 回溯、
  `lookup_meta` 符号查询、errors/warnings 收集）；
- 顺手修诊断文档 §2 四个隐藏不一致（含**顶层 `view {}` 在 UI 场景不可达的真 bug**）；
- 验收：`cargo tv` golden 零漂移；parser.rs 从 20,858 行降至约 16,600 行。

**1B 七点核心渗透改缝**（预计 1 个 Plan，附录 A §3）：
- `store` 并入 `UiDialect::keywords()`（消除 parser.rs:5244 硬编码——
  UiDialect 自有 in_on_body 判定同款逻辑）；
- `in_on_body` 状态位 → bridge 显式上下文 API（如 `set_handler_body_mode`）或
  方言注入回调（现核心读点 :7194 禁链合并）；
- `pub style` 前缀识别 → 方言声明"可 pub 关键字"数据（现 :4951–4962）；
- `TokenKind::On` 臂 / `TokenKind::View` 词法钉按附录 A §4 裁定留核（方言派发层
  已认领）；顺手清 `TokenKind::Grid` 遗留枚举（token.rs:414 已注释停用）；
- 验收：核心 crate `grep "UiDialect\|parse_widget_decl\|parse_view_node"` 除
  dialect 注册点外零命中；UI 测试照绿。

**1C node 表达式族裁定**（半个 Plan，主要是决策记录）：
- **方案 a（拆仓期采用）**：node 族 545 行（`parse_node`/`node_or_call_expr`/
  `parse_node_or_call_stmt`/`parse_nav_call`/`grid`）+ `Expr::Node`/`Expr::Grid`
  AST 变体**留核**——承认"UI 元素表达式化"为语言能力；vm/codegen 与
  trans/{rust,c,ts} 的既有消费不动；
- **方案 b（第二里程碑债）**：atom 层加 `try_dialect_atom` 钩子（镜像
  `try_dialect_stmt`），节点判别移入 UI 方言——atom 现有 `is_ui_scenario()` 分支
  （:3955）即插入点；同时服务 Godot `Ident{` 场景（:3968 注释），动它必须连
  godot 语义一起核；
- 验收：边界文档化 + 债条目登记（try_dialect_atom / Stmt::DialectItem 不透明形态）。

### 阶段 2：注册倒置与仓内 crate 化

**2a 注册倒置**（拆仓的钥匙，改动小）：
- 新增 `CompilerSession::with_dialect(Box<dyn Dialect>)` 外部注册 API；
- `Parser::build_dialects` 清空为纯扩展点（现硬编码 `if scenario == UI { push UiDialect }`）；
- UI 前端调用点（`ui_gen/api.rs`、auto-man 生成器入口）改为自注册；
- 验收：核心对 UI 的类型引用归零（为 2b 铺路）。

**2b UI crate 化**：
- `git mv`：`src/{aura,ui,ui_gen,design_tokens,a2ui}` + `src/dialect/ui/` →
  新 crate `crates/autoui`，反向 path dep `auto-lang`（ParserBridge/AST/VM pub 面
  冻结）；剪断两处小反向引用（§3.2）；
- `schema/aura.at` 加载路径走三级解析序（os_paths 模式，Stage B 先例）；
- 测试档位重切：65 个 `ui-iced` 门控根测试模块与 UI plan 测试随迁；
  `.cargo/config.toml` 别名与 nextest 三份配置同步调整；32MB 栈参数归
  UI crate 消费方；
- 验收：`cargo t` / `tf` / `tv` 全绿 + autoui-verifier 技能抽一个 demo 双端走查。

**2c auto-man 对半分**：
- UI 半边模块（`vue/vue_shadcn/jet/ark/rust_ui/tauri/tauri_backend/api_gen/
  gallery_assets/gallery_cache/wm_assets/image_viewer` + 376 个 rust-embed 资产）
  → `crates/autoui-cli`（bin 名 `auto-ui`）；
- auto-man 对 auto-lang 依赖**摘除 `features=["ui-iced"]`**；
- `crates/auto` 的 cmd_vue / cmd_ui / cmd_bp / cmd_tauri / cmd_watch 迁入插件。

**2d auto CLI 委派缝**（§1.1 设计落地）：
- Run/Build 加委派分支（scene/render 判定 → spawn `auto-ui` 原样参数）；
- env 契约注入；发现序三档；版本握手；插件缺失定向报错；
- 验收：`cargo tree -p auto-man` 零 iced/vue 依赖；`auto run` / `auto run -r vm`
  全链路照常（经委派）；`auto build --gen-only`（CI build-ui-examples）照常。

### 阶段 3：生态资产迁移（新仓 auto-ui 起建）

- 新建 `D:/autostack/auto-ui`；**git filter-repo 从 auto-lang 克隆中 carving 出
  UI 路径历史**（Plan 330 形态）；auto-lang 侧 plain `git rm`（其历史保留旧版本）；
- 迁移清单：§2.2 全部资产（examples UI 目录、packages、schema、blueprints、
  stdlib/aura、test/ui、scripts/tools、skills、四条 CI、UI 文档）；
- specs.json 约 163 条 UI 条目**条目级分账**（P-NNN-1/2/3 跨节镜像模式）；
  `docs/plans/INDEX.md` 留指针行；website UI 页面改外链或快照；
- 仓根垃圾不迁（`a/b/c/s*.json`、`remix*.json`、`donut_sfc.txt`、
  `vm-600-callout-box.png`、各类 log、scratch/tmp*）；
- 验收：新仓 git log 路径史完整；auto-lang 仓根无 UI 资产残留。

### 阶段 4：物理拆仓与消费方重指

- `crates/autoui` + `crates/autoui-cli` 移入新仓；依赖改相对路径
  （组内 worktree 布局 `D:/autostack/.wt/<组>/{auto-lang,auto-ui}` 保证
  `../auto-lang` 可解析——沿用现有约定 + wt-guard 红线）；
- 消费方重指：auto-os（wrapper 脚本、apps.manifest、CI 跨 checkout、解析族加
  `AUTO_UI_ROOT` 档）；auto-down jade-garden 绝对路径依赖改解析序；auto-term
  烤死路径重生成 + `compute_auto_lang_rel_path` 停止钉死生成时位置；
- auto-lang 侧 workspace members 删行 + 墓碑注释（Plan 325/330 先例）；
- 验收：新仓独立构建可跑 `auto-ui run`；auto-lang workspace 构建照常。

### 阶段 5：双向隔离验证与收尾

- 双向隔离（Plan 330 Phase 3 手法）：`cargo tree` 证明 auto-lang 默认构建零
  iced/vue 依赖；auto-ui 全量构建可跑双端；
- 插件专项：发现序三档各验一遍、版本失配告警、插件缺失可诊断报错；
- 跨仓回归：auto-os 画廊/应用走 wrapper 脚本；jade-garden；website 部署链
  （repository_dispatch）；
- 测试档位与资源表重测回填（各档测试数/耗时/内存权重）；
- 账本收尾：specs.json 分账落档、INDEX 再生（**P665-D6 坑：再生冲既有注记**）、
  KNOWN-DEBT 登记、两仓 AGENTS.md 工作范式更新。

---

## 5. 决策点记录

### 5.1 已裁定（2026-09-20，用户）

| # | 决策 | 依据 |
|---|---|---|
| D1 | auto-man 是独立项目（cargo 位），AutoUI 命令以 cargo-plugin 方式挂入 | 历史根因定性 |
| D2 | 动词直通式委派（非 PATH 扫描），`auto` 保持唯一入口 | 消费面零变化 |
| D3 | parser UI 语法经方言插件**迁出**（非留核） | 历史根因定性；推翻调研初版保守切线 |
| D4 | auto-man 留 auto-lang workspace，不单独拆仓 | Plan 330 教训（隔离单位是 workspace） |
| D5 | 拆仓只需进程内 trait 插件，不做 Swift 宏式进程插件 | 约束是依赖倒置非进程隔离 |
| D6 | AST UI 变体先留核作惰性数据（L1）；`Stmt::DialectItem` 不透明形态（L2）记债 | 搬变体=动所有下游 match 臂 |
| D7 | node 表达式族拆仓期留核（a），`try_dialect_atom`（b）第二里程碑 | 附录 A §2 |
| D8 | 历史保留 = filter-repo carving 新仓 + auto-lang plain git rm | Plan 330 形态 |

### 5.2 待裁定（实施期）

- Wizard 的 UI 模板（jet/capp）归属：暂留核心 + hash 锁同步（Stage B 手法）vs
  插件注册模板机制（正路，拆仓期先前者）；
- `run -r vm` 变两级进程后的退出码传播、Ctrl+C 信号传递、rqhost daemon 父子关系
  验收标准（rqhost 本身常驻应无碍，须实测末窗自退一次）；
- LSP 的 UI 补全（`aura::schema`）扩展点形态（读 schema 的扩展点 vs 直接依赖
  autoui crate）；
- 域章归属：design/20/29/30 倾向随迁 auto-ui，08 拆分，31（AutoScape）留语言仓
  待议；`docs/design/autoui/` 24 篇随迁；
- `auto-playground` / `auto-forge-ui` / `auto-lab-ui` 归属（UI 应用 vs 语言工具）；
- 未装插件时的报错形态落点（scenario 检测层定向错误）。

---

## 6. 风险与坑（先例提炼 + 勘定新发现）

1. **多 workspace 漂移**（auto-shell REVIEW A-2 实证：axum 0.7/0.8 双版本、
   windows 三版本并存、target 重复 7GB）→ 新仓控制为单 workspace，依赖尽量经
   auto-lang workspace.dependencies 传递。
2. **路径依赖 vs worktree**：相对路径 `../auto-lang` 在旧式 `.worktrees/` 下断裂；
   新仓从第一天纳入组内平铺布局 + `wt-guard.sh` 红线（禁 junction/symlink）。
3. **先做路径可解析，再搬文件**（Stage B 核心经验）：任何 include/加载器先改
   三级解析序（shell-pack 三回退字节等价是模板）。
4. **生成器烤死路径**：`compute_auto_lang_rel_path` 类生成物钉死生成时目录形态
   （auto-term 已踩坑）；拆仓前审计所有生成器输出。
5. **测试档位与资源账**：UI 测试占比高（日常档需 `--features ui-iced`），拆走后
   各档位测试数/耗时/内存权重表全部重测回填；heavy_gate/nextest 分组随迁。
6. **specs.json 分账精细活**：indent=1、条目级并集、P-NNN 镜像模式是既定手法；
   INDEX 再生冲既有注记（P665-D6 已四复发）。
7. **CI 跨仓缝合**：vm-files-ci 触发路径、deploy-website 跨 checkout、
   `repository_dispatch` 链重新对表。
8. **32MB 栈链接参数与 RUST_MIN_STACK 归属迁移**（.cargo/config.toml UI 专属印记）。
9. **名字撞车陷阱**（勘定新发现）：`parse_store_stmt`（**核心**变量语句解析器，
   store 是 Auto 的变量关键字，:8269）≠ `parse_store_decl`（UI store 块，:13888）；
   `.view` 后缀（DotView，**所有权系统**）≠ `view` 块（UI）——迁移时极易误伤。
10. **Event/OnEvents/Arrow 簇是语言级事件系统**（`Stmt::OnEvents` 消费方为
    dep/infer/indexer），非 UI 专属，不迁；`parse_task_on_block`/`parse_on_handler_body`
    属 async task 系统（核心），不迁。
11. **Godot scene 家族 269 行是另一方言**（任意场景生效），不随 AutoUI 迁，
    留作将来 dialect 迁移对象。
12. **Windows 冒号子命令名**：`test:ui` 做不成 PATH 插件名——定点委派已绕开。

---

## 7. 验证矩阵（对齐 Stage B V1–V9 风格）

| # | 验证项 | 手段 | 阶段 |
|---|---|---|---|
| V1 | golden 语料零漂移 | `cargo tv`（阶段 1 每步） | 1 |
| V2 | 核心零 UI 引用 | grep + `cargo tree`（auto-lang 默认构建零 iced/vue） | 1–2 |
| V3 | auto-man 零重依赖 | `cargo tree -p auto-man` 无 iced | 2 |
| V4 | 双端功能回归 | autoui-verifier 技能抽样 demo（Vue + VM 双臂） | 2 起 |
| V5 | 插件路径 | 发现序三档 / 版本失配告警 / 缺失定向报错 | 2d, 5 |
| V6 | 外部消费方回归 | auto-os wrapper、apps.manifest、jade-garden、website 部署链 | 4–5 |
| V7 | 测试档位 | 各档全绿 + 资源表重测回填 | 2, 5 |
| V8 | 历史保留 | 新仓 `git log` 路径史核验 | 3 |
| V9 | 账本收口 | specs.json 分账 / INDEX 再生 / DEBTS 销账 | 3, 5 |

---

## 附录 A：方言迁移工作量勘定（函数级分类账，2026-09-20）

> 行号均指 parser.rs（20,858 行 / 480 函数）。跨度格式：起始行–结束行。

### A.0 总账

| 类别 | 行数 | 处置 | 风险 |
|---|---|---|---|
| 纯 UI 直搬（A1/A2/A3/A5/A6/A7 + get_primary_prop） | ~3,900 | 机械迁入 `dialect/ui/` | 低 |
| node 表达式族 | 545 | 拆仓期留核（方案 a） | 中 |
| Godot scene 家族 | 269 | 不动（另案） | — |
| 核心反向渗透 7 点 | ~60（散布） | 改缝 | 中 |
| AST 层 | 1,921 | 留核惰性数据（D6） | 低 |
| 词法层 | ~0 | 仅清 Grid 遗留枚举 | 零 |

### A.1 主连续块 12669–17012（4,344 行 = AutoUI 4,075 + Godot 269 + 桥面候选 47）

| 族 | 跨度 | 行数 | 成员 |
|---|---|---|---|
| A1 widget 声明族 | 12669–13475 | 807 | parse_widget_decl(276)、mint_inline_event_handlers(64)、mint_view_inline(40)、mint_events_inline(48)、mint_bare_input_sync(68)、modal_dialog_tag_role(29)、hover_card_role(24)、mint_modal_dialog_toggle(_inner)(10+170)、direct_state_value_field(16)、wire_modal_close_recursive(62)。族内封闭（mint_* 仅被 parse_widget_decl:12883 与 parse_component_fn_decl:15425 调用） |
| A2 widget 生命周期块族 | 13476–13887 | 412 | use(158)、watch(132)、setup(55)、expose(43)、style_block_inner(24) |
| A3 store 声明 | 13888–13989 | 102 | parse_store_decl |
| A4 Godot scene（**非 AutoUI**） | 13990–14258 | 269 | looks_like_scene_decl(15)、parse_string_lit(26，**桥面候选**)、parse_scene_decl(64)、scene_node/instance/connection/signal/prop/value(45+10+22+29+13+10)、looks_like_subresource(16)、parse_scene_subresource(20) |
| A5 声明族 | 14259–15245 | 987 | widget_props(26)、msg(6+89)、model(6+49+69)、computed(94)、routes(58)、actions_decl(20)、style_recipe_decl(97)、timer(101)、actions 块(56+56+9+20+100)、menubar/toolbar/menu(19+40+9+63) |
| A6 view/view-node 族 | 15246–16427 | 1,182 | view_block(6)、or_fragment(15)、fragment_decl_body(16)、**parse_component_fn_decl(151)**、fragment_param_hint(14)、decl_body_tail(48)、view_block_inner(17)、view_root_nodes(27)、**parse_view_node(458，最大单函数)**、style_binding(48)、for/conditional/link(116+64+78)、condition_expr(124)、capture_condition_call_args(29) |
| A7 事件/handler 族 | 16428–17012 | 585 | event_modifiers(29)、event_value(28)、event_handler(42)、event_arg_list(27)、event_arg(164)、event_arg_object(49)、handler_params(23)、**parse_on_block(123)**、parse_bind_block(60)。in_on_body 置位点(:16938)在此族 |
| A8 尾部工具 | 17013–17139 | 106 | get_primary_prop(43，纯 UI)、extract_fstr_template_and_bindings(63，迁移时核调用方)、expect_ident(21，**桥面候选**) |

### A.2 表达式层深纠缠：node 族（11971–12545，545 行）

| 函数 | 行数 | 纠缠 |
|---|---|---|
| parse_node :11971 | 99 | 从 atom()/call 链调起（:3975/:4006/:12389/:12421/:12442），产出 `Expr::Node` |
| parse_node_or_call_stmt :12070 | 14 | 语句派发回退路径（Ident 兜底 :5251） |
| node_or_call_expr :12084 | **379** | 判别巨函数：节点 vs 调用 vs axum routing(:12334) vs builder 链 vs godot `Ident{`——五种语义共灶 |
| parse_nav_call :12493 | 30 | UI 导航调用，挂核心 call 链 |
| grid :12523 | 23 | `Expr::Grid`（TokenKind::Grid 已停用，走 Ident） |

`Expr::Node`/`Expr::Grid` 为核心 AST 表达式变体，被 vm/codegen.rs 与
trans/{rust,c,ts_stmt,ts_expr} 全消费——处置见 §4 阶段 1C。

### A.3 核心侧反向渗透点（7 处，不搬改缝）

1. 语句派发 Ident 分支 :5235–5265：`looks_like_scene_decl` 前置 → **store 硬编码**
   :5244（注释自认 "not yet in dialect table"；**UiDialect::keywords 现漏 store**）→
   方言缝 → parse_node_or_call_stmt 兜底；
2. `TokenKind::On` 臂 :5257：方言缝 + 核心回退 `Stmt::OnEvents`；
3. atom() 结构字面量拓宽 :3955–4010：`is_ui_scenario()` 门控 Type{...} 拓宽；
   **非 UI 路径也产出 Expr::Node**（godot 场景文件复用 parse_node 读体）；
4. `parse_body_inner` :7194：读 `in_on_body` 禁语句链合并（emit 链语义）；
5. `pub` 前缀分发 :4951–4962：核心识 `pub style`（+ lexer push_token 回退）；
6. parse_node 参数门 :12377：`is_ui_scenario()` + `is_ui_struct_construction`；
7. 状态字段 `in_on_body` :222：UI 写（:13726/:16938）核心读（:7194）。

### A.4 词法层

- `widget/msg/model/component/actions/style/store` 全走 Ident 上下文关键字——
  词法层零耦合；
- `view`：真表关键字（token.rs:364/:395）——**双重身份**兼 Core 参数模式
  `fn foo(view x int)`，留核；
- `on`：行首 header 通道（lexer.rs:895），always-on，留核；
- `grid`：TokenKind 已注释停用（token.rs:414），遗留可清；
- `.view` 后缀（DotView，lexer.rs:860）是**所有权系统**光学后缀，与 UI 无关，勿误伤。

### A.5 共享/核心（**勿搬**清单）

- `parse_store_stmt`(:8269–8607，339 行)：核心变量语句解析器（var/let/mut/const/
  shared 五 TokenKind 调用）——名字撞车陷阱；
- Event/OnEvents/Arrow 簇（12546–12668，123 行）：语言级 on 事件系统；
- `parse_task_on_block`(:6311)/`parse_on_handler_body`(:6398)：async task 系统；
- `is_ui_struct_construction`/`parse_braced_struct_args`(:7354–7394，43 行)：
  结构字面量共享机器；
- `parse_body(is_node)` / `parse_node_body`(:7120–7353)：is_node 穿线，parse_node_body
  仅被 parse_node 调用（可随 UI 走，parse_body 签名处置在桥面）。

### A.6 ParserBridge 公共面清单

token 游标族（next/peek/is_kind/expect_*）、产产生族（parse_expr 全家/parse_type/
parse_name/object()/parse_string_lit）、体解析（parse_body，含 is_node 形态处置）、
回溯（lexer.push_token + cur/prev save/restore）、符号查询（lookup_meta）、
错误收集（errors/warnings + span）。≈ Parser 已有 pub 方法超集，升格量小，
**需冻结契约**（golden 守护）。

---

## 附录 B：auto-man / auto 命令面归属审计

**留 auto-man（cargo 位核心）**：模块 `pac/resolver/builder/lock/git/index/cache/
scanner/target/exporter/pkg/asset/fs/group/port/pull/sidecar/dir/version` 等；
命令 `New/Init/Test/Add/Fetch/Deps/Clean/Info/Open/Device/Export/Env/Debug/Cffi/
Trans 转译族/OldRepl/Parse/Eval/Config`、语言侧 MCP。

**迁 auto-ui 插件**：模块 `vue/vue_shadcn/jet/ark/rust_ui/tauri/tauri_backend/
api_gen/gallery_assets/gallery_cache/wm_assets/image_viewer/vscode` + rust-embed
376 资产；命令 `Ui/Bp/Block(弃用别名)/test:ui/Watch/Gen/Tauri/vue/Rqhost`、
Run/Build 的 UI 委派目标；`Docs` 中 UI 相关部分。

**缝上（动词直通）**：`Run/Build` —— scene:ui 或 render∈{vue,rust,vm,jet,arkts,
tauri} 时委派 `auto-ui`（theme/accent/desktop/gallery 等 UI 旗标原样透传）。

---

*立档：2026-09-20（调研会话三轮：足迹/耦合勘定 → 插件缝架构裁定 → 方言迁移函数级
勘定）。实施启动时以 `/auto-plan:new` 分解为 Plan 序列，每阶段独立 worktree。*
