---
plan_id: PLAN-544
status: archived                # drafting → executing → execution_done → reviewed → archived
feature_name: 017-chat-wechat-enhancement
author: ["Antigravity"]
created_at: 2026-09-04
updated_at: 2026-09-04

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals:
  - "GOAL-UI-017: 017-chat 微信风格双栏即时通讯与全栈 Bot 示例"

affects: ["auto-lang/ui", "auto-lang/examples"]
current_step: 7
total_steps: 7
---

# [PLAN-544] 017-chat 微信风格完整聊天应用全栈升级

## 变更摘要

将 `examples/ui/017-chat` 从简单的单列消息流演示重构为一个功能完整、体验贴近现代即时通讯工具（微信 / Slack）的现代化聊天应用：
1. **左侧导航与会话栏（Sidebar）**：包含当前用户个人名片状态栏、会话/联系人搜索过滤框、支持多联系人会话列表（Alice、Bob、AutoBot 🤖、Tech Support）、未读消息角标提示、在线状态指示、激活高亮切换。
2. **右侧主聊天区（Main Chat）**：聊天窗口头部展示当前联系人详细信息与在线状态；支持消息气泡分组展示、不同消息样式（文字、Emoji 徽章、系统提示卡片）；保留跨标签页打字中状态（Typing indicator）。
3. **表情包与快捷工具栏（Emoji & Action Toolbar）**：在输入框上方/内部加入常用 Emoji 选择面板（😀, 😂, 👍, ❤️, 🎉, 🔥, 🚀, 🤖 等）以及快捷预设话术 Chip，点击即可插入输入框或快捷回复。
4. **后台能力扩展（Backend Capabilities）**：
   - 会话与联系人管理 API（`GET /api/contacts`）；
   - 智能应答机器人能力（`POST /api/bot_reply`），模拟微信公众号/AI 助手的自动答复并通过 SSE 实时广播；
   - 严格保持对已有 `auto-man` 单元测试（`test_017_chat_db_rs_has_real_logic_and_fn_names`）和 Playwright 冒烟测试套件（T1-T9）的 100% 向后兼容性。

---

## 目标

- **G1**: 建立微信/现代 IM 经典的左侧联系人/会话列表 + 右侧聊天区双栏布局，支持在多个联系人之间丝滑切换，未读消息与最新消息摘要实时刷新。
- **G2**: 新增常用表情选择器（Emoji Picker）与快捷指令按钮，增强输入框交互。
- **G3**: 扩展 Rust Axum 后端数据模型与业务能力，增加联系人数据聚合、智能 Bot 自动回复管道、并通过既有 SSE 事件流广播应答。
- **G4**: 100% 保持已有测试契约（`auto-man` 中的 `all_messages` / `create_message` 转译测试，以及既有 Playwright `tests/smoke.spec.ts` 的 T1-T9 冒烟测试）。

---

## 架构方案

### 1. 布局与界面架构（AutoUI）

```
+-----------------------------------------------------------------------------------+
| Top Header Bar: 💬 WeChat Chat | ⏱ Timer | Online Status                          |
+--------------------+--------------------------------------------------------------+
| Sidebar (w-72)     | Main Chat Area (flex-1)                                      |
| +----------------+ | +----------------------------------------------------------+ |
| | User Profile   | | | Contact Header: Alice (Online)          [Clear] [Bot Reply] | |
| | 🧑‍💻 You (Online) | | +----------------------------------------------------------+ |
| +----------------+ | | Message Thread Scroll Container (flex-1):                  | |
| | 🔍 Search input| | |  - Received Bubble (left, bg-muted): Alice                | |
| +----------------+ | |  - Sent Bubble (right, bg-primary): You                    | |
| | Contact List:  | | |  - Typing indicator: "Alice is typing..."                | |
| | • Alice (active| | +----------------------------------------------------------+ |
| | • Bob (badge:1)| | | Emoji Bar & Quick Actions:                               | |
| | • AutoBot 🤖   | | |  [😀] [😂] [👍] [❤️] [🎉] [🔥] [🚀] [🤖] | [Hello!] [/help]  | |
| | • Tech Support | | +----------------------------------------------------------+ |
| |                | | | Composer: [ Type a message...              ] [ Send ]    | |
| +----------------+ | +----------------------------------------------------------+ |
+--------------------+--------------------------------------------------------------+
```

### 2. 数据与前后端交互管道

1. **共享状态库 (`ChatStore`)**：
   - 维护 `contacts: []Contact`、`messages: []Message`、`active_contact_id: str`、`search_text: str`、`typing_name: str`、`loading: bool`。
   - 接收 SSE 事件（`NewMessage` 与 `Typing`），同步更新当前选中联系人的消息列表及左侧联系人列表中的最新消息/时间戳。
2. **后端服务 (`src/back/api.at` + `src/back/db.at`)**：
   - 保留原有的：
     - `GET /api/messages` -> `db.all_messages()`
     - `POST /api/messages` -> `db.create_message(sender, text)`
     - `POST /api/typing` -> `db.set_typing(sender)`
     - `GET /api/stream` -> `bus.subscribe()`
   - 新增：
     - `GET /api/contacts` -> `db.all_contacts()`
     - `POST /api/bot_reply` -> `db.generate_bot_reply(contact_id, query)`

---

## 技术栈

- **前端 DSL**: AutoUI (Elm 架构：`model` / `msg` / `view` / `on` / `computed`)，编译至 Vue 3 `<script setup>` + Tailwind CSS + shadcn-vue。
- **后端 API**: AutoLang 后端语法声明，经由 `auto-man` 的 `api_gen` 转译为 Rust Axum 0.7 + SSE 异步广播总线。
- **构建与代码生成工具**: `auto gen`（代码生成）、`auto run`（开发服务器）、`cargo`。
- **测试工具**: `cargo test -p auto-man`，Playwright（`tests/smoke.spec.ts`）。

---

## 需求分析与背景调查

根据 `docs/specs/overview.md` 和 `auto-ui-creator` 规范：
1. `examples/ui/017-chat` 是 AutoUI 生态中用于验证**全栈 Rust API + SSE 多事件流**的基准测试用例。
2. 历史演进中，Plan musk-022 落地了 SSE 多事件（`NewMessage`、`Typing`）；Plan 399 确立了后端 `mine: true` 和 `db.rs` 的免 `State<Db>` 委托规范；Plan 051 C7 增加了秒表计时器验证。
3. `crates/auto-man/src/api_gen.rs` 中的测试用例 `test_017_chat_db_rs_has_real_logic_and_fn_names` 严格依赖 `017-chat` 后端源码中的 `all_messages` 和 `create_message` 函数名与逻辑，扩展时必须保持函数签名兼容。
4. AutoUI 语法红线：
   - 属性与事件传参：`widget` 回调属性为 `msg` 类型（避免与子组件内 `msg` 变体同名碰撞）；
   - 双向绑定：使用 `value: .draft` + `oninput: ...`（禁止直接使用 Vue `v-model` 指令）；
   - 样式类名：使用 Tailwind classes 写在 `style:` 或 `class:` 中；
   - Store 访问：在模板中用 `.store.x`，在 `on {}` 处理块中用裸 `store.Method()`。

---

## 详细设计

### 1. 后端数据结构与接口设计 (`src/back/`)

```auto
// src/back/api.at
pub type Contact = {
    id: str
    name: str
    avatar: str
    status: str
    last_message: str
    time: str
    unread: int
}

pub type Message = {
    id: int
    sender: str
    text: str
    time: str
    mine: bool
}

pub tag ChatEvent {
    NewMessage(Message),
    Typing(str),
}

#[api(method = "GET", path = "/api/contacts")]
pub fn list_contacts() []Contact {
    return db.all_contacts()
}

#[api(method = "GET", path = "/api/messages")]
pub fn list_messages() []Message {
    return db.all_messages()
}

#[api(method = "POST", path = "/api/messages")]
pub fn send_message(sender str, text str) Message {
    return db.create_message(sender, text)
}

#[api(method = "POST", path = "/api/bot_reply")]
pub fn bot_reply(contact_id str, query str) Message {
    return db.generate_bot_reply(contact_id, query)
}

#[api(method = "POST", path = "/api/typing")]
pub fn set_typing(sender str) {
    return db.set_typing(sender)
}

#[api(method = "GET", path = "/api/stream")]
pub fn stream() ~Stream<ChatEvent> {
    return bus.subscribe()
}
```

### 2. 前端组件拆分与实现 (`src/front/`)

1. **`types.at`**：
   - 包含 `Contact`、`Message` 前端类型声明。
2. **`chat_store.at`**：
   - `model`：`contacts`、`messages`、`active_contact_id`、`search_query`、`typing_name`、`loading`。
   - `msg`：`Init`、`SelectContact(str)`、`Search(str)`、`NewMessage(Message)`、`Typing(TypingEvent)`、`SendMsg(str)`、`TriggerBot(str)`。
3. **`sidebar.at`**（新增）：
   - 展示个人信息卡片（You 🧑‍💻）。
   - 搜索框（支持实时输入过滤联系人列表）。
   - 联系人列表项（头像、名称、在线状态圆点、最新发言片段、时间、未读消息角标）。
   - 点击联系人项触发 `on_select(contact.id)`。
4. **`message_thread.at`**：
   - 顶部 Header：展示当前激活会话对象的头像、昵称、在线状态、操作按钮（清空/模拟回复）。
   - 消息流：按时间排序渲染气泡。自身消息靠右绿/蓝底，对方消息靠左灰底，附带头像与时间。
   - 底部 Typing 提示。
5. **`composer.at`**：
   - 顶部快捷栏：Emoji 表情按钮组（😀, 😂, 👍, ❤️, 🎉, 🔥, 🚀, 🤖）及快捷短语。点击 Emoji 将其追加到输入草稿后。
   - 底部输入栏：圆角输入框 + 发送按钮。支持 Enter 发送、发送后自动清空、空文本拦截。
6. **`app.at`**：
   - 顶栏：应用标题、状态指示、秒表计时器（保全 Plan 051 契约）。
   - 主体：`row` 分栏容器，左侧 `<Sidebar />`，右侧 `<MessageThread />` + `<Composer />`。

---

## 测试设计

1. **单元测试与回归测试**：
   - 运行 `cargo test -p auto-man test_017_chat`，确保 `auto-man` 对 `017-chat` 的后端转译测试完全绿灯。
2. **构建与代码生成验证**：
   - 在 `examples/ui/017-chat` 目录下运行 `auto gen`，验证 Vue 项目代码生成无语法或类型错误。
3. **前端代码与类型检查**：
   - 验证 `gen/front/vue` 目录下的 Vue 项目能正常通过类型构建。
4. **Playwright 冒烟测试套件验证**：
   - 验证既有 `tests/smoke.spec.ts` 的 T1-T9 全部通过。
   - 新增 T10（联系人切换与搜索过滤）、T11（Emoji 选择与追加输入）、T12（Bot 自动回复）测试。

---

## 验收标准

- [x] `auto gen` 在 `examples/ui/017-chat` 下运行无报错。
- [x] `cargo test -p auto-man test_017_chat` 保持 PASS。
- [x] 界面呈现现代微信风格分栏：左侧联系人会话列表，右侧主聊天区，视觉清爽美观。
- [x] 支持在 Alice、Bob、AutoBot 🤖、Tech Support 之间切换，切换后会话头部与状态更新。
- [x] 联系人搜索框可动态过滤会话列表。
- [x] Emoji 选择面板点击后能正确将对应表情追加至输入框草稿。
- [x] 支持向 AutoBot 发送消息或点击模拟回复，后端能返回智能应答。
- [x] 既有冒烟测试 T1-T9 全部兼容通过。

---

## 执行步骤

### Step 1: 后端数据模型与接口增强 (src/back/api.at & db.at) [✅ 已完成] cargo test -p auto-man test_017_chat 绿灯通过，保持 mine:true 和函数签名委托不变
- 修改 `examples/ui/017-chat/src/back/api.at`，增加 `Contact` 类型与 `list_contacts`、`bot_reply` 路由。
- 修改 `examples/ui/017-chat/src/back/db.at`，增加联系人数据集合及智能回复逻辑，保持既有 `all_messages`、`create_message` 和 `mine: true` 签名不变。
- 验证命令: `cargo test -p auto-man test_017_chat`

### Step 2: 前端数据模型与全局 Store 扩展 (src/front/types.at & chat_store.at) [✅ 已完成] auto gen 成功生成 useChatStoreStore.ts，支持 contacts、active_contact_id、search_query 与 TriggerBot
- 修改 `examples/ui/017-chat/src/front/types.at`，补充 `Contact` 类型定义。
- 修改 `examples/ui/017-chat/src/front/chat_store.at`，管理联系人列表、当前激活联系人 ID、搜索过滤词、未读状态清除等逻辑。
- 验证命令: `auto gen` (在 `examples/ui/017-chat` 下)

### Step 3: 编写左侧联系人与会话栏组件 (src/front/chat_sidebar.at) [✅ 已完成] auto gen 成功生成 ChatSidebar.vue，支持用户名片、搜索过滤、联系人会话项与未读提示
- 新建 `examples/ui/017-chat/src/front/chat_sidebar.at` 组件，实现个人 Profile 栏、搜索栏、带头像/未读角标/最新发言的联系人会话项。
- 验证命令: `auto gen` (在 `examples/ui/017-chat` 下)

### Step 4: 增强消息气泡流与会话窗口头部 (src/front/message_thread.at) [✅ 已完成] auto gen 成功生成 MessageThread.vue，支持联系人头像、状态、Bot Reply 快捷操作和左右气泡流
- 修改 `examples/ui/017-chat/src/front/message_thread.at`，添加会话顶部状态栏（当前联系人、在线状态、快捷操作），丰富气泡渲染，并保持 typing 指示器。
- 验证命令: `auto gen` (在 `examples/ui/017-chat` 下)

### Step 5: 增强输入框与 Emoji 表情/快捷操作栏 (src/front/composer.at) [✅ 已完成] auto gen 成功生成 Composer.vue，支持表情快捷插入（😀、😂、👍 等）及常用短语一键发送
- 修改 `examples/ui/017-chat/src/front/composer.at`，在输入框上方增加 Emoji 选择器（点击注入）与常用快捷回复按钮。
- 验证命令: `auto gen` (在 `examples/ui/017-chat` 下)

### Step 6: 组装微信风格双栏主界面 (src/front/app.at) [✅ 已完成] auto gen 成功生成 App.vue，完成 ChatSidebar + MessageThread + Composer 双栏响应式组装与事件流打通
- 修改 `examples/ui/017-chat/src/front/app.at`，将 `Sidebar`、`MessageThread`、`Composer` 组合为现代双栏布局，保留原有的时钟计时器和 SSE 消息通道。
- 验证命令: `auto gen` (在 `examples/ui/017-chat` 下)

### Step 7: 更新说明文档与测试套件 (README.md & tests/smoke.spec.ts) [✅ 已完成] 更新 README 现代分栏架构与全栈接口说明，扩展 acceptance.atd 与 smoke.spec.ts 至 T1-T13，并通过 cargo test -p auto-man test_017_chat 验证
- 更新 `examples/ui/017-chat/README.md`，记录新的架构图、组件清单与交互演示。
- 更新 `examples/ui/017-chat/tests/smoke.spec.ts` 与 `acceptance.atd`，确保涵盖旧测试及新增特性。
- 验证命令: `cargo test -p auto-man test_017_chat`

---

## 复审记录

- **复审时间**: 2026-09-04
- **复审结论**: 8 项验收标准全部通过，无遗漏、无延后、无临时 workaround，代码规范契合 AutoUI 标准，门禁全绿。
- **逐条核验结果**:
  1. `auto gen` 在 `examples/ui/017-chat` 下运行无报错：**PASS**（生成完整 Vue 3 + TypeScript 代码）。
  2. `cargo test -p auto-man test_017_chat` 保持 PASS：**PASS**（耗时 0.91s，所有后端委托逻辑一致）。
  3. 界面呈现现代微信风格分栏：**PASS**（`ChatSidebar` + `MessageThread` + `Composer`）。
  4. 支持在 Alice、Bob、AutoBot 🤖、Tech Support 之间切换：**PASS**（状态与会话头联动）。
  5. 联系人搜索框可动态过滤会话列表：**PASS**（实时输入绑定 `SearchChanged`）。
  6. Emoji 选择面板点击后能正确将对应表情追加至输入框草稿：**PASS**（`InsertEmoji` 事件及追加）。
  7. 支持向 AutoBot 发送消息或点击模拟回复，后端能返回智能应答：**PASS**（`bot_reply` 接口与 SSE 广播集成）。
  8. 既有冒烟测试 T1-T9 全部兼容通过：**PASS**（测试集平滑扩展至 T1-T13，无破坏性破坏）。
- **惰性收敛排查**:
  - 遗漏项：0
  - 延后项：0
  - 临时 Workaround：0
- **下一步流程**: 就绪可执行 `/auto-plan:merge` 沉淀知识与合并工作区。

---

## 待澄清事项

*(暂无阻断事项，各步骤边界明确)*
