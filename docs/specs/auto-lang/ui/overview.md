# ui（AURA / UI 引擎）

> **Status**: partial（a2vue 主线与 VM 渲染在用；多后端与 token 系统未落地）

## 职责

Auto 的 UI 子系统，围绕 **AURA**（Auto UI Representation Abstract，UI-IR）组织：

- **前端解析**：UI 方言（`widget`/`msg`/`model`/`view`/`on` 仅在 `scenario: "ui"` 下生效，dialect 机制）。
- **AURA 提取与校验**：从 `WidgetDecl` AST 提取视图树/状态/事件三元 IR，按 `schema/aura.at` 校验。
- **代码生成**（`ui_gen/`）：a2vue（主力）、a2jet（Compose）、a2ark（ArkTS）、a2rust、ts/ark/kotlin adapter、block 层。
- **VM 运行时渲染**（`ui/`）：vnode、事件路由、VM 桥接、gpui/iced/headless 后端、MCP 调试服务。
- **a2ui 协议**：agent 驱动 UI 的 JSON 消息协议（surface update / component）。
- **前后端契约**：`#[api]` 注解生成 TypeScript/Tauri/Axum 通信层（`src/api/`，非 `ui_gen/`）。

## 现状

- a2vue 是事实上的主力后端：`VueGenerator` 生成 SFC + router + shadcn/tailwind 工程，`examples/ui/001–025` 教学序列以其为主；015-notes 已扩展为带后端的完整 app（plan-354/357/360 进行中）。
- VM 渲染路径（`ui/vm_bridge.rs` + `interpreter/` + iced/gpui/headless）可跑通 015-notes（plan-327/333），与 codegen 路径并存。
- AURA schema 校验已实现：`schema/aura.at`（Auto 自描述）经 `SchemaLoader` 加载为 `AuraSchema`。
- Block 层已落地包格式与注册表（`ui_gen/block/` + 顶层 `blocks/`），但 plan-342/343 自述"实施未开始"——代码已先于 plan 状态存在。
- a2jet/a2ark 代码在库（`ui_gen/jet/`、`ui_gen/ark/`），近期无活跃 plan；a2ui 协议随 composer/replica（plan-217/234）定型。
- 未实现：Design Token Compiler（`src/tokens/` 不存在）；router Phase 2-4（`app` 关键字、嵌套路由、守卫）。

## 关键入口

- `crates/auto-lang/src/dialect/ui.rs:UiDialect` — UI 场景关键字接管
- `crates/auto-lang/src/aura/extract.rs:extract_widget_from_decl` — WidgetDecl → AURA 提取
- `crates/auto-lang/src/aura/types.rs:AuraWidget` / `AuraRoute` / `LogicPayload` / `AuraApp` — IR 核心类型
- `crates/auto-lang/src/aura/schema_loader.rs:SchemaLoader` / `load_default_schema` — schema 加载
- `crates/auto-lang/src/aura/schema.rs:AuraSchema` — 元素/prop 校验表
- `crates/auto-lang/src/ui_gen/vue.rs:VueGenerator`（`generate_sfc` / `generate_router_file`）— a2vue
- `crates/auto-lang/src/ui_gen/block/registry.rs:BlockRegistry` / `spec.rs:BlockSpec` — block 层
- `crates/auto-lang/src/ui/widget_registry.rs:WidgetRegistry` — VM 侧 widget 注册
- `crates/auto-lang/src/ui/event_router.rs:EventRouter` — VM 事件路由
- `crates/auto-lang/src/a2ui/schema.rs:A2UIMessage` — a2ui 协议消息
- `crates/auto-lang/src/api/targets/typescript.rs:TypeScriptGenerator` — `#[api]` 前端契约生成
- `schema/aura.at` — AURA schema 定义（Auto 自举）；`stdlib/aura/widgets/` — 组件库；`examples/ui/001–025` — 教学/基准序列

## 使用示例

```auto
widget Counter {
    msg Msg { Inc, Dec }
    model { count int = 0 }
    view { col { button + { onclick: .Inc } h2 > Count: ${.count} } }
    on { .Inc => { .count += 1 } .Dec => { .count -= 1 } }
}
```

`pac.at` 设 `scenario: "ui"`；`auto vue source/front -o output/app` 生成 Vue 工程（含 router/main.ts/package.json）。

## 已知坑

- **文档与代码分歧**（详见 architecture.md 与报告）：docs/design/08 称 `#[api]`、AutoDown "未实现"，实际均已落地（`src/api/`、`src/autodown/`）；08 称 scenario 由 parser 直接检查 session，现已是 dialect 机制（`dialect/ui.rs`，见 docs/design/dialect-extension-diagnosis.md §6.1）。
- router 双语法并存：Plan 105（`"/" => HomePage {}`）与 Plan 106（`"/" => use index`，推荐），见 docs/router.md。
- plan-342/343 状态（"未开始"）落后于代码（`ui_gen/block/` 已存在），以代码为准。
- a2vue 工程庞大（`vue.rs` 近万行），曾发生 OOM/递归缺陷（plan-356、358、361）。

## 蒸馏来源（Phase 1）

- `docs/design/08-ui-systems.md`
- `docs/design/16-app-generation-and-ai-authoring.md`
- `docs/design/17-blocks-first-class.md`
- `docs/router.md`
- `docs/plan-indices/11-ui-generators.md`、`docs/plan-reports/`（UI 相关主题）
- `examples/ui/001–025` 教学序列
