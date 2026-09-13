# 015-notes — 清爽笔记应用（扁平双栏 + 始终可编辑）

一个「打开就能写」的笔记应用示例：左侧列表栏、右侧编辑栏，用发丝线分隔而不是
悬浮卡片；正文没有读/写模式切换，任何时候都能直接打字。

```
┌──────────────────────────────────────────────────────────────────────┐
│  ▤ Notes                                   ☀        [+ New note]     │  ← 顶栏 h-14
├──────────────────────┬───────────────────────────────────────────────┤
│  🔍 Search notes     │  Welcome                        Saved  Unpin 🗑 │
│  All Pinned 📁work … │  🕐 Just now                                  │
│  #intro #work …      │  (intro ×)  (+ Tag)                           │
│  ──────────────────  │ ─────────────────────────────────────────────  │
│  PINNED              │                                               │
│   ▎Welcome           │  This is your notes app. Click on any note…   │
│     Just now         │                                               │
│  NOTES               │                                               │
│    Quick Ideas       │                                               │
│    10 min ago        │                                               │
│  ──────────────────  │                                               │
│  ⚙ Settings  ☀ Theme │                                               │  ← 侧栏页脚
└──────────────────────┴───────────────────────────────────────────────┘
```

## 布局与视觉

- **扁平三区**：顶栏（`border-b`）+ 列表栏（`border-r`）+ 编辑栏，全域用语义 token
  （`bg-background` / `bg-card` / `text-muted-foreground` / `primary`），暗色与 5 色主题照常生效。
- **归一化排版**：标题 `text-2xl`、正文 `text-base leading-7`、元信息/筛选 `text-xs`；
  正文有阅读栏宽（`max-w-2xl`）而不是铺满整屏。
- **图标化**：全部用 lucide `icon` 元素（notebook/search/plus/check/x/trash-2/settings/
  sun/moon/folder/clock），不再用 emoji 当图标。置顶动作在 VM 字形表里没有 pin 字形，
  因此用文字标签 `Pin note` / `Unpin` 承载。

## 交互模型

| 动作 | 行为 |
|---|---|
| 直接编辑标题/正文 | 进入未保存态（状态文本变 `Unsaved changes`，出现 `Save`）；标题回车也可保存 |
| 切换笔记 / 切换筛选 / 新建 | **先自动落盘**上一条草稿，再切换（编辑中被切走不丢内容） |
| `New note` | 新建并**自动选中**空笔记（清空搜索，保证新笔记可见） |
| `Delete` | 两步确认：先出 `Delete`/`Cancel`，确认才真删 |
| `Pin note` / `Unpin` | 经后端 `toggle_pin` **落库**（不是只改内存） |
| 搜索框 | **真正过滤**（标题+正文，大小写不敏感，由后端 `search_notes` 提供）；无命中显示空态 |
| 筛选胶囊 | `All` / `Pinned` / 文件夹（📁 前缀）/ 标签（`#` 前缀）；重置选中到第一条可见笔记 |
| `Settings` → `Dark`/`Light` + 5 色板 | 主题与主题色；顶栏太阳/月亮图标按钮是快切 |

列表分两组：`Pinned`（置顶）与 `Notes`（其余）。文件夹与标签都从笔记数据派生，
不再在视图里硬编码。

## 数据与后端

- 后端 API 未改动：`src/back/api.at` 的 `list_notes / create_note / update_note /
  delete_note / toggle_pin / update_tags / search_notes`。
- 前端唯一状态源是 `src/front/notes_store.at`：
  - 工作集 `notes`（搜索时替换为命中集，因此列表下标是**当前工作集**的下标）
  - 可见索引表 `visible_pinned` / `visible_notes`（元素为 `.notes` 下标）
  - 草稿 `draft_id/title/body/time/folder/pinned/tags` + `dirty`
  - 词表 `all_tags` / `all_folders`
- 过滤**全部在 store 里**算成索引表，视图只做索引解引用。原因见下节。

## 已知限制（有意为之）

1. **筛选词表不回收已删除标签**：`all_tags` 在初始化时播种、新增时追加，但删除标签
   后不会从词表移除（清空重建需要「空数组字面量赋值」，该写法会触发 VM codegen panic，
   见 `tests/acceptance.atd` 的「示例侧 DSL/VM 使用约束」表）。
2. **正文是纯文本**：用原生 `textarea`，没有 markdown 富文本编辑（`autodown_editor`
   在 VM 端会降级为 textarea，本示例不做双端不一致的富文本）。
3. **文件夹与标签可能同名**：种子里同时存在文件夹 `work` 与标签 `work`。UI 用
   📁 / `#` 前缀区分，但两者过滤结果在种子数据下相同。
4. **设置面板是示例自带的扁平面板**：不再依赖跨仓共享 `SettingsPopover`
   （原 `deps/settings` 指向已删除的 `examples/ui/common/settings`）。主题能力仍走
   既有 store 链路（`dark_mode` / `accent_color` + 生成器注入的 `applyAccent`）。

## 源码结构

```
src/front/app.at          应用骨架：顶栏 + 双栏 + 消息转发（含主题镜像同步）
src/front/sidebar.at      NavTree：搜索行 / 筛选胶囊 / Pinned+Notes 分组 / 外观面板 / 页脚
src/front/editor.at       EditorPanel：无边框标题+正文、状态、Pin、两步删除、标签胶囊
src/front/notes_store.at  唯一状态源：过滤索引表、草稿、词表、主题
src/front/types.at        Note / Folder 类型
tests/acceptance.atd      验收契约（single source of truth）
tests/015-notes.autotest  MCP 场景（VM / Rust 可执行子集）
tests/smoke.spec.ts       Playwright（Vue）场景
```

## 如何运行

```bash
cd examples/ui/015-notes
auto gen              # 生成所有后端代码（vue / jet / ark / rust）
auto run              # Vue 开发服务器
auto run -r vm        # VM (iced) 后端
```

## 如何验证

```bash
# VM/Rust MCP 场景（19 条）：先起应用，再跑场景
cd examples/ui/015-notes && AUTOUI_MCP_PORT=9247 auto run -r vm     # 终端 A
cd examples/ui/015-notes/tests && python run_autotest.py 015-notes.autotest --mode vm   # 终端 B

# Vue 端
cd examples/ui/015-notes/tests && pnpm test

# 双端截图（autoui-verifier 技能脚本）
python ../../../.agents/skills/autoui-verifier/scripts/test_vm_mcp.py \
  --app-dir examples/ui/015-notes --initial-screenshot vm_initial
node ../../../.agents/skills/autoui-verifier/scripts/test_vue_playwright.mjs \
  http://localhost:3000 src/front/tests/screenshots/vue_initial.png
```
