# 28 - Playground 与在线文档体验架构

> 状态：✅ Accepted（2026-09-07 用户设计讨论定稿）。
> 来源：2026-09-07 两套 playground 入口差异调查（§1）与双层设计讨论。
> 关联：[14-developer-tools.md](14-developer-tools.md)（Web Playground 条目归本章细化）、
> specs 现状：[docs/specs/playground-vue/project.md](../specs/playground-vue/project.md)、
> [docs/specs/auto-playground/project.md](../specs/auto-playground/project.md)；
> 历史计划：archive/202（playground 设计）、219（source map）、225（交互调试器）、246（组件包抽出）。
> 实施：分解为两个 Plan——[581 组件分层 + Notes manifest 管线](../plans/581-playground-notes-foundation.md)、
> [582 Notes Explorer + 宿主合一 + 电子书嵌入](../plans/582-playground-notes-explorer.md)。

## 1. 背景与问题

### 1.1 现状：一个组件库，两个入口，两副面孔

Playground 前端资产目前是**一个组件库、两个页面入口**，两者功能级别不同、URL 只差一个斜杠：

| 入口 | 组件 | 能力 | 依赖 |
|---|---|---|---|
| `/playground/`（静态 SPA；`crates/auto-playground/frontend` 构建产物，同时作为后端自服务 UI） | `AutoPlaygroundFull` + `PlaygroundLayout`（~1000 行） | 文件树、多文件项目、调试、回放、字节码、转译 | 强依赖后端 `/api` |
| `/playground`（VitePress 页 `website/playground.md`） | `AutoPlayground`（731 行，号称精简） | 单文件编辑 + Run/Debug/Live 转译/分享 | 同样依赖后端，且工具栏仍挂 ExampleSelector |

历史三步（git 考古）：

1. `68e6c14f2`：VitePress 静态站初建，`playground.md` 用 **iframe** 嵌全量 SPA；
2. `95089c156`：全量 SPA 构建产物经 `scripts/build-playground.mjs` 同步部署到 `website/public/playground/`（独立静态页）与后端 `frontend/dist`；
3. `ebc43e7ac`（Plan 246）：抽出可复用组件包 `packages/auto-playground-vue`，website 页 iframe 换成内联 `<AutoPlayground>`——但旧全量静态页**未退役**，并存至今。

这是演进残留而非设计裁定。组件库统一（Plan 246）只完成了"同源"，没有完成"同形"。

### 1.2 问题清单

1. **两副面孔**：`/playground` 与 `/playground/` 只差一个斜杠，界面与能力完全不同。
2. **精简组件不纯**：`AutoPlayground` 仍拉取 `/api/examples` 挂 ExampleSelector，无法当"单 snippet 拼图"嵌入（电子书、landing page 等场景）。
3. **示例面窄**：ExampleSelector 只吃 `examples/playground-demo/`（25 个单文件 + 4 个项目示例），而仓库语料库有数百条（§1.3）。
4. **无后端即死页**：website 是静态部署，后端不在线时两个入口都是空白/报错，连"看代码"都做不到。
5. **语料沉睡**：vm golden、aavm corpus、双语书的 `auto` 代码块等高质量素材没有展示通道。

### 1.3 语料盘点（2026-09-07 实测）

| 源 | 路径 | 规模 | 形态 | 亮点 |
|---|---|---|---|---|
| vm golden | `crates/auto-lang/test/vm/[0-9][0-9]_*/` | 42 个分组目录、322 个 `.at`+`.expected.out` 配对 case | 单文件 | **自带期望输出**，天然适合展示与对照 |
| aavm corpus | `crates/auto-lang/test/vm/aavm2/corpus_{m1,m2,m3,m4,use,a2r}/` | 158 个 `.at` | 单文件 | 自举语料，覆盖面广 |
| 电子书 | `website/books/`（8 本双语书） | ~1282 个 ` ```auto ` 围栏（`.cn.md` 重复计数，唯一块约半数） | 片段/完整程序 | 带章节上下文，是"文档化"素材的主体 |
| playground-demo | `examples/playground-demo/` | 25 单文件 + 4 项目目录 | 单文件/多文件 | 现有 ExampleSelector 唯一来源 |
| parity | `parity/libs/**/auto/*.at` | 53 个 | 完整程序（多依赖 C 库/环境） | 需选择性收录（§9 开放问题①） |

## 2. 目标与非目标

**目标**：

1. **三层组件**：拼图层（SnippetRunner）→ 卡片层（PlaygroundCard）→ IDE 层（现有 Full），一层拼图可嵌入任意宿主。
2. **静态优先**：代码与说明在构建期打入页面，无后端可完整浏览全部笔记；仅 Run/转译需要后端，离线时降级为引导。
3. **笔记站**：demo 页与 website playground 合一为 Notes Explorer——左侧分组导航 + 搜索，右侧说明 + PlaygroundCard，覆盖全部语料源。
4. **电子书嵌入**：书内 ` ```auto ` 代码块一键 Run，CodeBlock 原地切换为 SnippetRunner。
5. **单一 manifest**：构建期采集脚本生成 notes manifest，三个消费方（VitePress 页 / 独立 SPA / 后端 `/api/examples`）共用一份数据源。

**非目标**：

- 不做 AutoUI 运行形态（UI 型示例归画廊线，Plan 578；playground 只收 stdout 型语料）。
- 不扩展后端能力（run/trans/debug/notebook API 维持现状）。
- 不做用户账号/云保存（分享沿用 `?code=` 链接形态）。

## 3. 总体架构

```
                构建期（静态）                            运行期
┌────────────────────────────────┐
│ 语料源（仓库内）                 │
│  vm golden · aavm corpus ·      │
│  books · playground-demo ·      │
│  parity(选择性)                 │
└──────────────┬─────────────────┘
               │ scripts/build-playground-notes.mjs
               ▼
┌────────────────────────────────┐
│ notes manifest（单一事实源）      │
└──────┬──────────┬──────────┬───┘
       │          │          │
       ▼          ▼          ▼
  VitePress    独立 SPA    后端 /api/examples
  /playground  (后端自服务 UI)   (改读 manifest)
  (Notes       NotesExplorer
   Explorer)   ——同一组件族——
                                    ┌──────────────────┐
              点 Run/转译 ────────▶  │ auto-playground  │
              (无后端→降级引导)      │ axum 后端 /api   │
                                    └──────────────────┘
       电子书 books/*.md
          │ 主题层 code fence 后处理
          ▼
       □ Run → 原地展开 SnippetRunner
```

核心不变式：**同一组件族 + 同一数据源 + 多宿主**。cargo run 打开的页面、website 的 `/playground`、嵌入电子书的运行器，三者视觉与行为一致，差别只在宿主给的容器。

## 4. 组件分层设计

现有两级（AutoPlayground / AutoPlaygroundFull）收紧为三层。新增两层都是从现有 `AutoPlayground.vue` + `usePlayground.ts` 拆出，不重写编辑器/运行逻辑。

### 4.1 SnippetRunner（拼图层）

最纯的运行单元，电子书 CodeBlock 展开后的形态。

| Prop | 类型 | 默认 | 说明 |
|---|---|---|---|
| `code` | `string` | 必填 | 初始代码 |
| `apiBase` | `string` | `''`（同源 `/api`） | 后端地址 |
| `autorun` | `boolean` | `false` | 挂载即运行 |
| `target` | `'run' \| 'rust' \| 'c' \| ...` | `'run'` | 首选动作 |
| `height` | `string` | `'auto'`（按行数） | 容器高 |

行为约束：**无工具栏语义**（仅一个小 ▶/⤴ 动作位）、无 ExampleSelector、无文件树；输出区内联折叠展示。不可用时（无后端）▶ 转为引导态。

### 4.2 PlaygroundCard（卡片层）

demo/笔记站拼图的标准单元，= SnippetRunner + 可配置工具栏。

| Prop | 类型 | 默认 | 说明 |
|---|---|---|---|
| `code` / `noteId` | `string` | 二选一 | 直接代码或 manifest 笔记 id（带期望输出/说明） |
| `toolbar` | `{ transpile?, share?, debug?, live? }` | 全开 | 工具栏项开关 |
| `exampleSelector` | `boolean` | **`false`** | 旧默认行为显式选入 |
| `height` | `string` | `'480px'` | 容器高 |

### 4.3 Playground IDE（IDE 层）

现有 `AutoPlaygroundFull` + `PlaygroundLayout` 原样保留：文件树、多文件项目、调试、回放、字节码、source map 联动。宿主：独立 SPA（后端自服务 UI）。

### 4.4 与现有组件的映射

- `AutoPlayground.vue` → 拆解为 `PlaygroundCard`（壳）+ `SnippetRunner`（核），`usePlayground.ts` 复用；
- ExampleSelector 从工具栏常驻改为 `exampleSelector` prop（默认关）；IDE 层保持常驻；
- `AutoPlaygroundFull` 不动（仅数据源改吃 manifest）。
- 包导出（`index.ts`）新增 `SnippetRunner` / `PlaygroundCard`，`AutoPlayground` 保留为向后兼容别名（过渡期后移除）。

## 5. Notes 数据管线

### 5.1 语料源与采集规则

| 源 | 采集规则 | 分组 | expectedOutput |
|---|---|---|---|
| vm golden | case 目录内 `.at` 与同名 `.expected.out` 配对；无配对者跳过 | 目录即组（组名取目录号后语义段，如 `05_loops`→"循环"） | ✓（`.expected.out` 内容） |
| aavm corpus | `corpus_*` 下全部 `.at` | 每目录一组（m1 基础/m2 …/a2r） | ✗ |
| 电子书 | 提取 `ch*.md` 的 ` ```auto ` 围栏；`.cn.md` 不重复采集（英文版为源）；含 `import` 或跨文件依赖的块标记 `standalone: false` | 每本书一组、章为子级 | ✗ |
| playground-demo | 单文件直采；项目目录采全部文件 | 一组（保留现状） | ✗ |
| parity | 仅收 stdout 可独立运行者，范围 Plan A 勘察后裁定（§9-①） | 一组 | ✗ |

书内块的运行语义：Auto 支持顶层语句，绝大多数围栏可直接 Run；`standalone: false` 的块展示-only（Run 位显示"依赖多文件"提示），不做自动包装。

### 5.2 manifest schema（v1）

```jsonc
{
  "version": 1,
  "builtAt": "…",                       // 仅元信息，内容确定性输出（排序稳定）
  "groups": [{
    "id": "vm-loops",
    "title": "循环",
    "order": 5,
    "source": "crates/auto-lang/test/vm/05_loops",
    "notes": [{
      "id": "vm-loops/001_while",
      "title": "while 循环",
      "sourceType": "vm-golden",        // vm-golden | aavm-corpus | book | demo | parity
      "sourcePath": "crates/auto-lang/test/vm/05_loops/001_while/while.at",
      "kind": "single",                 // single | project | fence
      "standalone": true,
      "code": "…",                      // kind=project 时为 null，files 见下
      "files": null,                    // [{ path, content }]（项目型）
      "expectedOutput": "…\n",          // 仅 vm-golden
      "description": null,              // 可选说明（书块=章节上下文摘要）
      "tags": ["while", "loop"]
    }]
  }]
}
```

标题与说明：目录/case 名按现有 `display_name_from_stem` 风格生成；`description` 允许后续人工增补（采集脚本不覆盖 manifest 旁的 overrides 文件，机制 Plan A 定）。

### 5.3 采集脚本与构建集成

- 脚本：`scripts/build-playground-notes.mjs`（Node，与 build-playground 同技术栈）；
- 输出：`website/public/playground-data/notes.json`（单文件起步；若体积超阈值改按组分片——§9-③）；
- 集成：website 构建前置步骤 + CI 校验（重新生成 diff 为空，防语料与 manifest 漂移）；
- 后端 `/api/examples`：改读同一 manifest（消除目录扫描第二事实源），实施方式 Plan B 裁定（§9-④）。

## 6. Notes Explorer（笔记站）

### 6.1 布局

```
┌──────────────┬─────────────────────────────────────────────────────┐
│ ⌕ 搜索/过滤   │  Fibonacci                          ● vm-golden     │
│              │  crates/.../06_loops/012_fibonacci.at               │
│ ▾ 基础 (32)  │  ─────────────────────────────────────────────────── │
│   01 基础    │  递归版斐波那契，演示……（说明文档，可折叠）             │
│   02 位运算  │  ┌─────────────────────────────────────────────────┐ │
│   …          │  │ ▶ Run  →Rust ▾   分享   编辑器 (CodeMirror)     │ │
│ ▸ 语言特性    │  │                                                 │ │
│ ▸ AAVM 语料  │  ├─────────────────────────────────────────────────┤ │
│ ▸ 电子书      │  │ [输出|期望输出 ✓|Rust|C|Python|TS|ABT|字节码]    │ │
│ ▸ Demo 示例  │  │                                                 │ │
└──────────────┴──┴─────────────────────────────────────────────────┘ │
                    └──────────────── PlaygroundCard ────────────────┘ │
```

- 左栏：分组树（可折叠）+ 计数徽章 + 标题/标签全文搜索；
- 右栏上部：笔记头（标题、来源路径 chip——链接到 GitHub 对应文件、来源类型徽章）+ 说明（可折叠）；
- 右栏主体：PlaygroundCard；输出区多 tab：输出 / **期望输出**（vm-golden）/ Rust / C / Python / TypeScript / ABT / 字节码。

### 6.2 交互与视觉

- 深链：`/playground#/notes/<noteId>`；分享沿用 `?code=` base64；
- 键盘：↑/↓ 选笔记、Ctrl+Enter 运行；
- **期望输出对照**（能力亮点）：实际输出 vs `.expected.out` 的差异行级高亮（✓ 一致 / ✗ 差异），绿色对勾徽章；
- 视觉不另起炉灶：直接消费 VitePress 主题 CSS 变量（`--border` / `--muted` / `--accent`），暗色模式跟随站点；笔记列表用紧凑等宽文件名风格——"笔记站"气质靠信息密度与来源标注，不靠另做皮肤。

### 6.3 项目型（多文件）笔记

manifest `kind=project` 的笔记，PlaygroundCard 内以文件 tab 呈现（entry 锁定 `main.at`）；完整 IDE 体验（文件树/调试）通过"在 IDE 中打开"跳独立 SPA。细节 Plan B 落地。

## 7. 电子书嵌入

- 机制：VitePress 主题层对 ` ```auto ` 围栏做后处理渲染（不改书内容），右上角常驻小 ▶ Run；
- 点击：CodeBlock **原地展开**为 SnippetRunner（`autorun`），带"收起"还原为纯代码块；
- `standalone: false` 的块（含 `import`/跨文件依赖）显示锁形提示，不提供 Run；
- 可选容器语法 `::: runnable`（带标题/说明），供需要定制展示的示例；
- 双语书（`.cn.md`）同等生效，零内容改动。

## 8. 部署与宿主合一

- **website 唯一入口**：`/playground`（VitePress 页升级为 Notes Explorer 布局）；
- **旧静态页退役**：`website/public/playground/` 构建产物移除，替换为跳转页（`→ /playground`）；`build-playground.mjs` 的 website 同步分支删除；
- **后端自服务 UI 保留**：`crates/auto-playground/frontend` 继续作为 `cargo run -p auto-playground` 打开的页面，宿主组件从 `AutoPlaygroundFull` 换成与 website 同源的 Notes Explorer（IDE 层经"在 IDE 中打开"入口保留）；
- **无后端降级**：所有 Run 位统一降级为引导态（"启动后端：`cargo run -p auto-playground`"或部署说明链接）。

## 9. 取舍与裁定记录

1. **三层组件而非两层**：拼图/卡片/IDE 三档密度对应三类宿主（电子书、笔记站、独立 IDE），两层会把"无工具栏"和"可配置工具栏"耦在一起。
2. **静态优先**：代码构建期打入，Run 才走后端。website 静态部署形态下"能看"不依赖"能跑"。
3. **单一 manifest**：构建期确定性生成，不做运行时扫描；后端 examples API 改读 manifest。
4. **第一版只收 stdout 型语料**：AutoUI 示例归画廊（Plan 578 归属裁定），不进 playground。
5. **期望输出是一等公民**：vm golden 的 `.expected.out` 进 manifest，UI 对照高亮。
6. **website 单入口**：旧静态页退役重定向，双维护终止。
7. **书内片段不自动包装**：顶层语句天然可跑；依赖型块标记展示-only，不做模板包装（避免运行出与书意不符的结果）。

开放问题（交由 Plan 处理）：

- ① parity 收录范围（依赖环境的程序占比高，Plan A 勘察 stdout 可跑子集后定）；
- ② 项目型笔记在 Explorer 的文件 tab 形态与"在 IDE 中打开"跳转（Plan B）；
- ③ manifest 单文件 vs 按组分片（体积阈值，Plan A 实测后定）；
- ④ 后端 `/api/examples` 与 manifest 的统一方式（读同一 JSON vs 共享生成逻辑，Plan B）；
- ⑤ 人工增补 description 的 overrides 机制（Plan A 定，可先不做）。

## 10. 实施切分

| Plan | 范围 | 关键交付 | 依赖 |
|---|---|---|---|
| [Plan 581](../plans/581-playground-notes-foundation.md)：组件分层 + Notes manifest 管线 | §4 组件三层收紧、§5 采集脚本与 manifest、语料源映射（含 ① 勘察） | `SnippetRunner`/`PlaygroundCard` 组件 + `build-playground-notes.mjs` + notes.json 产物 + `--check` 校验 | 无 |
| [Plan 582](../plans/582-playground-notes-explorer.md)：Notes Explorer + 宿主合一 + 电子书嵌入 | §6 笔记站、§7 电子书 Run、§8 部署合一（含 ②④） | NotesExplorer 组件、website `/playground` 改造、旧静态页退役、后端 frontend 换宿主、主题层围栏后处理 | Plan 581 |

两个 Plan 串行（B 依赖 A 的组件与 manifest），各自可独立验收。

## 11. 未来演进

- **notebook 接入**：后端已有 notebook 单元格 API，笔记可演进为可编辑持久单元；
- **agent debug 嵌入**：`agent_debug` WS 会话经 SnippetRunner 暴露给文档读者；
- **UI 示例联动**：画廊（Plan 578 轨）成型后，UI 型示例经画廊展示、playground 仅链入；
- **期望输出批量校验前端化**：笔记站"运行全部 vm-golden"按钮，作为 CI golden 回归的可视化镜像；
- **多语言书深链**：书内块运行结果回链笔记站对应笔记（双向导航）。
