# PLAN-639 T-00：Block→Blueprint 更名 manifest + PLAN-635 跨包机制裁定

> 状态：T-00 产物（2026-09-17，plan-639-dev worktree 实勘）。
> 用途：①T-03 更名执行清单 + T-08 grep 门的白名单/豁免依据；②T-04 跨包解析
> 的机制裁定（扩展 vs 平行）。

---

## 1. 术语语义裁定：三类 "block"，只更名其一

全仓 `block` 命中分三类，T-03 只改 **A 类**：

| 类 | 语义 | 代表位置 | 处置 |
| --- | --- | --- | --- |
| **A. UI 三层 Block**（Design 17 Skill-tier 包） | Widget/Block/App 中间层：spec.md + reference/*.at + gotchas 包 | `ui_gen/block/`、`cmd_block.rs`、`blocks/`、`blocks-gallery` | **更名 Blueprint/bp** |
| B. 语言 AST block | `Stmt::Block`/`Expr::Block`/lexer/token、语料测试（a2r 004_blocks 等） | parser/lexer/ast/trans/infer、`test/**` 语料、plan634/plan577 测试 | 豁免（语言语义不动） |
| C. autodown 文档块 | auto-down PBlock 家族、`autodown_blocks.rs`、jade `blocks_store.at` | `ui/autodown_blocks.rs`、auto-down 仓 | 豁免（另一概念，计划非目标） |
| D. 无关同形词 | CSS `display:block`/`@apply block`、`CodeBlockMenu`、fixture id | cmd_vue.rs:1925、autoui-skill reference、021-block-static | 豁免（grep 门按模式排除） |

历史归档（`docs/plans/archive/**`）整体豁免——保留历史语境（计划 §1 非目标）。

## 2. 更名 manifest（A 类全集）

### 2.1 代码（T-03）

| # | 现路径/标识 | 目标 | 备注 |
| --- | --- | --- | --- |
| C1 | `crates/auto/src/cmd_block.rs` | `cmd_bp.rs` | 文件 + `mod cmd_block` 声明（main.rs:10） |
| C2 | `main.rs` `BlockAction`（:313）、`Commands::Block`（:519/:1736） | `BpAction`/`Commands::Bp` | CLI 主名 `auto bp`；**`auto block` 保留为别名 + 弃用提示**（Q-2 裁定：保留至下一版） |
| C3 | `crates/auto-lang/src/ui_gen/block/{mod,registry,spec}.rs` | `ui_gen/bp/` | `BlockRegistry→BlueprintRegistry`、`BlockPackage→BlueprintPackage`、`BlockSpec→BlueprintSpec`；`ui_gen/mod.rs` `pub mod block`→`bp` |
| C4 | `registry.rs:103` `repo_root.join("blocks")` | `join("blueprints")` | 包库扫描根 |
| C5 | `main.rs:329` `--out` 默认 `src/front/blocks` | `src/front/bps` | L2 落地默认目录；cmd_bp.rs 文档同步 |
| C6 | 唯一外部 import `cmd_block.rs:16` `ui_gen::block::{BlockPackage, BlockRegistry}` | `ui_gen::bp::{BlueprintPackage, BlueprintRegistry}` | 已核实无其他调用点（grep 2026-09-17） |

### 2.2 包库与示例（T-03）

| # | 现路径 | 目标 | 备注 |
| --- | --- | --- | --- |
| P1 | `blocks/`（4 包 + README） | `blueprints/` | `git mv`；包内容术语（README/spec.md 正文）同步；kind 分类不变 |
| P2 | `examples/blocks-gallery` | `examples/bps-gallery` | 目录 + 内部 `src/blocks.ts`→`bps.ts`、`pages/BlockPage.vue`→`BpPage.vue`、router、README |
| P3 | `test-blocks-tab.js`（仓根，website dist 探针，硬编码绝对路径） | 删除 | 一次性调试脚本（指向 `.vitepress/dist` 构建产物），随 website 页更名一并退役 |

### 2.3 文档/spec（T-01/T-02/T-03）

| # | 现路径 | 目标 | 备注 |
| --- | --- | --- | --- |
| D1 | `docs/design/blocks/`（4 篇） | `docs/design/blueprints/` | git mv 保历史；`blocks-first-class.md`→`blueprints-first-class.md`、`block-package-format.md`→`blueprint-package-format.md`、另两篇文件名不变；Design 17 增补三通道节（T-02） |
| D2 | `docs/design/00-intro.md`（:123 归位索引） | 指向 `blueprints/` | T-02 |
| D3 | `docs/specs/blocks/` | `docs/specs/blueprint/` | project.md 更名改写 + **新增 contract.md**（T-01，SD-01/02）；INDEX.md 引用同步 |
| D4 | `docs/specs/auto-lang/ui/design/block-tier.md` | `blueprint-tier.md` | git mv + 内容术语；`ui/architecture.md`、`ui/plans.md` 引用同步（T-02 核对） |
| D5 | `docs/specs/goals.md` GOAL-011（:20） | Blueprint 表述 | T-01（SD-03） |
| D6 | `docs/specs/INDEX.md`（:28 ui_gen 模块清单、:133 cmd_block 行、:253 blocks 行、:325/:332-339 专题页与明细） | bp/blueprint 行 | T-08 前完成（随各任务顺带，T-08 收口核对） |
| D7 | `docs/specs/auto-cli/project.md`（cmd_block 行） | `auto bp` 行 | T-03 |
| D8 | ~~website 更名~~ **改判豁免（T-03 实勘）** | 维持现状 | 实勘裁定：`website/blocks.md` 与 `public/ui/blocks/` 描述的是 **ui-gallery 构建产物的 `#/blocks` 路由**（24 个 shadcn-vue 组合复刻演示页），非 Block 包层概念；SPA 构建产物 URL 与 ui-gallery 内部路由属消费侧产品面，移交 PLAN-070。grep 门按路径豁免 |
| D9 | `examples/ui/README.md`（:156 "Tier 2 · Blocks"） | "Tier 2 · Blueprint（组合层）" | 示例分层名与三层术语对齐 |
| D10 | `docs/specs/autoui-skill/project.md`（SD-04） | 补 vue 轨 actions 发射契约行 | 实勘：该 spec 无 Block 术语残留（SKILL.md 命中的 block 均为 D 类语义）→ SD-04 术语臂零改动，仅补契约行 |

### 2.4 豁免清单（grep 门白名单）

- **路径豁免**：`docs/plans/archive/**`、`docs/specs/_archive/**`、`target/**`、`website/node_modules/**`、`.git/**`
- **AST/语言语义**（B 类）：`crates/auto-lang/src/{ast.rs,parser.rs,lexer.rs,token.rs,indexer.rs,scope.rs,dep.rs,error.rs,implicit_union.rs}`、`src/ast/**`、`src/infer/**`、`src/trans/**`、`src/plan*_tests.rs`（block 语义词测试）、`crates/auto-lang/test/**`（语料目录 004_blocks 等）、`crates/auto-atom/**`、`crates/a2r-std/**`
- **autodown**（C 类）：`crates/auto-lang/src/ui/autodown_blocks.rs`
- **fixture id**：`examples/capability-tests/021-block-static/**`（EDGE-16 载体，`BlockStore` 为应用域命名）
- **词形模式豁免**（D 类）：`display: block`、`@apply block`、`inline-block`、`CodeBlock*`、`block_cursor` 等代码标识符中非 UI 层语义（grep 门按 `\bblock(s)?\b` 扫描**活跃 UI 层路径白名单**而非全仓，见 §2.5）
- **auto-down 仓**：本仓 grep 门不管辖（跨仓移交 PLAN-070）

### 2.5 grep 门口径（T-08 落地）

对**活跃 UI 层路径集**（`crates/auto/src/**`、`crates/auto-lang/src/ui_gen/**`、
`blueprints/**`、`examples/bps-gallery/**`、`docs/design/blueprints/**`、
`docs/specs/blueprint/**`、`website/**`（豁免 node_modules））执行
`grep -rniE '\bblocks?\b'`，命中仅允许出现于：`blueprint(s)` 词内（正则
`(?<!auto\ )` 前瞻不可行，用先剥 `blueprint` 词再查 `\bblock\b` 的两步法）、
豁免模式（§2.4 词形）、`auto block` 弃用提示字符串（仅 cmd_bp.rs 一处）。
脚本落 `scripts/blueprint_rename_guard.py`，CI/门禁随调。

## 3. PLAN-635 跨包机制调查与裁定（T-04 前置）

### 3.1 机制实勘（证据）

| 面 | 现状 | 位置 |
| --- | --- | --- |
| 依赖声明 | pac.at `dep "name" { path: ... }`（引号/裸名两形态）+ workspace `members: [...]`；**声明门控**（pnpm 式）：`deps/<name>` 物化但未声明 = 幽灵依赖，解析阻断 + 修复指引 | `lib.rs` `pac_declares_dep`（:2777）、`pac_workspace_member_dir`（:2795） |
| 解析序 | `resolve_module_path`：base_dir 直探 → 父目录 → 向上 4 级逐层探 `deps/<name>`（门控）/ workspace member / 本地 path dep；dotted module → 路径段；`{sub}.at`/`{sub}/mod.at` 通用候选 | `lib.rs:2768-2926` |
| 双轨共享 | VM 轨 `resolve_use_module`（+632-F2 item 命名文件探测）；vue 轨 `collect_module_imports`；两轨同走 `resolve_module_path`；style recipe 经 `prepare_style_recipe_imports`（宿主 parse 前预注册 pub 符号）+ `load_and_validate_style_recipes_with_imports` 单点注册 | `lib.rs:2929-`、`recipe.rs:187-300` |
| 缓存 | 模块解析**无缓存**（每次构建重解析，visited-set 防循环）；recipe 语料 cargo 产物按内容 hash 缓存（与 .at 包解析无关） | 实勘 |
| env/组内/主检出序 | **.at 运行时解析面无 env 覆盖与组内 sibling 步骤**——AGENTS.md 的 env→组内→主检出序是 Rust 构建期跨仓规则；.at 包用 pac.at path dep 指向相对路径即可达成等价（worktree 组内 `dep x { path: "../../../<repo>/blueprints" }`） | 实勘 |

### 3.2 裁定：**扩展 635 机制**（非平行）

理由：
1. `use` 导入的 items 天然类别无关（"may name widgets/fns/stores"——recipe 即先例）；
   bp 参考实现是 widget 形态 `.at`，跨包 widget 消费 635 已覆盖（"依赖包 widget
   已可跨包消费"），零新解析设施。
2. 通用候选 `{pkg_root}/{sub}.at` 已可达 bp 包文件：`dep "bps" { path: "../../blueprints" }`
   + `use bps.form.login.reference.default` → sub=`form.login.reference.default` →
   候选 `<blueprints>/form/login/reference/default.at` **逐字命中**。T-04 以 demo 实证。
3. 声明门控（幽灵依赖阻断）正是 L1"零副本消费"的账面——依赖关系留在 pac.at，
   无物化副本。

### 3.3 扩展点（T-04/T-06 落地项）

| # | 扩展 | 说明 |
| --- | --- | --- |
| E1 | L1 消费形态约定 | 应用 pac.at 声明 `dep "bps" { path: <blueprints 包库> }`；绑定工件生成 `use bps.<kind>.<name>.reference.<variant>: <WidgetSym>` + 布局装配 widget（GENERATED 头注）。**位置/命名裁定（Q-3，T-06 实勘修订）**：`src/front/bps/<name>_bind.at`（下划线 stem）——模块解析面 dots→路径段 映射无法命中含点文件名（`login.bind.at` 不可达），下划线 stem 使 `use bps.<name>_bind` 经 base_dir 直探命中；构建期 `bps/` 目录与 `components/` 同构扫描（vue.rs T-06 扩展），绑定 widget 获独立 SFC |
| E2 | BlueprintSpec 前向兼容字段 | spec.rs frontmatter 增可选 `props`/`actions`（六问契约 §4.1 的声明面）；既有 4 包无此字段=合法（缺省可省） |
| E3 | bind 一致性校验 | `auto bp add --bind`：bp 存在、variant 存在、bind 引用的 action id ⊆ spec `actions:` 声明（声明缺省时降级 warning——存量包未声明不阻断） |
| E4 | 双轨消费实证 | demo app（bps-gallery 扩展或独立 fixture）：VM `auto run -r vm` 渲染断言 + vue `auto build` 绿（AC-03） |

### 3.4 风险与边界

- bp reference `.at` 的场景启发式（back/ → core，其余 UI）沿用 recipe import 的
  session 判定；bp 包无 back/ 目录 → 恒 UI session，符合预期。
- 版本面 MVP=主检出单版本（Q-1 维持待澄清，PLAN-070 消费前裁定）。
- 多 bp 包同时 use 时符号撞名：沿用 635 撞名规则（后者注册报错）——bind 工件
  以 `<Name>` Pascal 化导入，包名前缀天然去重。

---

## 附：实勘命令记录（可复跑）

```
find . -type d -iname '*block*'（排除 target/.git）        # §2 目录面
grep -rln 'ui_gen::block|BlockRegistry|BlockPackage|BlockSpec' crates/   # C3/C6 唯一调用点
grep -rln 'blocks-gallery' .                                # P2 引用面（9 文件）
grep -rln 'auto block' docs/ .agents/ scripts/ examples/ blocks/ .github/ # D7/D 面确认
```
