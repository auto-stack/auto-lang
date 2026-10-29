# Editing Feature 嵌入：AutoUI 使用方式与语法草案

> 状态：Draft v0.2，2026-10-28，仅设计；全部新语法/API 未实施、未编译验证。
> v0.2：常见用法改为 `Editing.open` 聚合入口、绑定目标的 surface/action 引用和一次性菜单/keymap 安装；v0.1 的四对象装配保留为显式层，不再要求普通消费者逐一接线。
> 上位设计：[Design 47](../47-autoui-state-design-and-feature-architecture.md)、[Design 38](../38-autoui-features-and-system-services.md)、[App Presentation Host](app-presentation-host.md)。
> 目的：用两个真实可区分的编辑功能实例，展开实例隔离、文档/视图共享、分散 UI 出口与命令目标；供后续语法和语义评审。
> 边界：不改迁移中的 auto-ui/auto-lang，不改 auto-edit 应用，不创建执行 Plan，不把示例登记为可运行 fixture。

## 1. 使用场景与设计选择

假设一个 ReviewApp 同时包含：

- **files**：编辑项目文件，使用宿主的 workspace ResourcePort。
- **notes**：编辑业务记录中的笔记，使用宿主的 record ResourcePort。
- 两边都有编辑面板；大纲可以放在远处的侧栏，状态项可以放在底栏。
- 一个通用“保存/撤销”菜单与快捷键；另有“保存文件”“保存笔记”两个固定目标入口。

设计选择：**底层保留功能实例、文档会话、编辑视图和命令目标四个对象；常见组合通过一个聚合入口使用，按需展开控制。** 位置不参与目标猜测。两个编辑器不必是两个 App，也不需要复制两份 auto-edit 源码。

此稿优先展示直接组合 Widget 的写法；同样的 surface 可以装入 Region，但不要求简单嵌入者先声明一整套工作台。Blueprint 可以封装默认布局，不改变下面的 owner 和 target。

## 2. 四个对象，分别负责什么

| 对象 | 内容与寿命 | 显式层名字（v0.1） |
|---|---|---|
| Editing instance | 领域会话集合、provider、配置、命令接收者、资源清理 owner | `files`、`notes` |
| DocumentSession | 文本内核、dirty、保存基准、encoding/EOL、领域 undo/redo、revision | `file_doc`、`note_doc` |
| EditorView | 绑定一份 document；光标、选区、编辑器折叠、滚动等交互状态 | `file_view`、`note_view` |
| EditorTarget | 指向确定的 feature/document/view 的有类型目标；不复制其状态 | `file_target`、`note_target` |

Outline 的展开项、菜单开合等仍可属于它自己的 presenter 局部状态。EditorView 不是所有相关 UI 的状态大杂烩。

```text
ReviewApp owner
  ├─ files : Editing
  │    ├─ documents/main : DocumentSession
  │    └─ views/main     : EditorView ──引用──> documents/main
  └─ notes : Editing
       ├─ documents/main : DocumentSession
       └─ views/main     : EditorView ──引用──> documents/main

file_target ──引用──> files + files/documents/main + files/views/main
note_target ──引用──> notes + notes/documents/main + notes/views/main

呈现位置另行安排：editor / outline / status 都可以引用同一个 target。
```

注意：document 与 view 是兄弟 owner 节点，通过引用关联，不是 document 随着 view 被销毁。上图中的 `main` 在两个实例内可以重名，因为唯一身份还包含 app/feature/对象种类。

## 3. 与现有 AutoUI 的差异

基线参考 Design 47 的 v0.5.0 证据；补充只读查看了该标签的 `examples/ui/041-auto-edit/src/front/{app,status_bar,editor_store}.at`。该示例已用 actions/on 和独立状态栏，但状态栏直接导入共享 EditorStore，动作委托根 handler；这不是本稿的目标实例模型。

| 写法 | 本稿地位 |
|---|---|
| `widget`、`use`、`view`、`on`、`msg`、`row/col`、现有 style | 沿用形态 |
| `actions { action(id:, handler:, ...) }`、menubar/toolbar 组件 | 沿用结构，新增 typed 引用/目标等字段；不能因此声称整个示例当前可编译 |
| `feature` 声明 | 拟新增：声明可实例化的领域功能及公共合同 |
| `scope { name = expression }` | 拟新增：owner 管理的稳定对象装配，不是每帧执行的普通代码 |
| `surfaces` / `contributions` | 拟新增的 feature 内上下文块，导出有类型的 UI 出口与贡献模板 |
| `surface(from:, target:, key:)` | 拟新增库节点/内建 lowering：挂载一个公开出口 |
| `save = action(...)`、`command:`、`target:`、typed `enabled_if` | 拟扩展现有 actions 绑定合同；保留 handler 作为最终执行入口 |
| `focus.command_target(Editing)` | 拟新增框架 API：返回当前窗口中该功能类型的目标 selector |
| `Editing.open` / `Editing.bind` / `.split` | v0.2 拟议库工厂：创建常用组合、引用已有 view、增加共享文档的 view；不新增业务层或全局关键字 |
| `.files.editor()` 等 bound surface 调用 | v0.2 拟议有类型 UI 表达式，展开为原 `surface(from:, target:, key:)`；不是字符串模板或任意 JS 调用 |
| `Editing.menu()` / `Editing.keys()` | v0.2 拟议贡献模板入口，复用原贡献/焦点路由合同；不在每个实例中自动安装 |
| `ResourcePort`、DocumentSessions、EditorViews 等 | 目标库合同；示例调用名不代表仓库已存在对应 API |

`scope` 是通用 owner 装配块，不专门服务 Feature；Feature、session、view 和只读引用可以用同一机制组织。它不引入第二套业务 model：可变领域状态继续由 store/领域内核持有。

## 4. 推荐默认用法与显式展开

### 4.1 双编辑器的默认写法（v0.2）

普通使用者只声明两个编辑入口。`Editing.open` 负责准备 Feature、一个 DocumentSession、一个主 EditorView 和它们的 target；公开返回一个聚合引用 `EditorBinding`。

```auto
use auto.editing: Editing
use auto.ui: ResourcePort

widget ReviewApp(file_resources ResourcePort, note_resources ResourcePort) {
    scope {
        files = Editing.open("src/main.at", resources: .file_resources)
        notes = Editing.open("notes/42", resources: .note_resources)
    }

    actions {
        Editing.keys()
    }

    view {
        col {
            menubar { Editing.menu() }

            row {
                col(style: "w-56") {
                    .files.outline()
                    .notes.outline()
                }
                col(style: "flex-1") {
                    .files.editor()
                    .notes.editor()
                }
            }

            row {
                .files.status()
                .notes.status()
            }
        }
    }
}
```

这是推荐的消费者表面语法，仍是未实施提案。`EditorBinding` 是四个既有对象及其绑定出口的聚合引用，**不增加第五个状态 owner**。库作者维护 session/view/action 的真实合同，普通使用者不必把四个名字全部声明一遍。

`open/bind/split` 是 Editing 包提供的普通类型化工厂/方法；框架只提供 owner 装配与可类型化的 surface/action/contribution 协议，不硬编码“所有 Feature 都必须有文档”。下载、主题配置等 Feature 可以提供自己的常用组合入口，不需要伪造 DocumentSession。默认出口和动作是返回类型公开的成员，不靠反射猜测函数名。

两个入口均有自己的文档、历史、保存状态和主视图；它们的 editor/outline/status 自动引用各自的同一 target。设计上下文默认继承所属 App/子作用域，没有显式主题时使用框架默认值。ResourcePort 仍须清楚选择，本例保留两个不同 provider；不能为了省字猜测业务数据应保存到哪里。

`.files.editor()` 等是 typed bound surface factory，返回可在 view 中挂载的描述符。仅这类 UI 值可作为此处的视图表达式；不能把任意有副作用的方法调用放进 view 执行。创建文档/实例仍只发生在 scope activation，surface 调用仅投影既有组合，不再次 open。

### 4.2 默认值是确定规则，不是启发式查找

| 省略的内容 | 默认规则 | 显式覆盖 |
|---|---|---|
| Feature 实例 key | 所属 owner＋scope binding 身份；示例分别为 files、notes | `Editing.open(..., key: "project-editor")` |
| 文档会话 | 在该实例中创建 `main` 会话，绑定给定 resource | `.feature.document(key:, resource:)` |
| 编辑视图 | 创建 `main` view，绑定上述会话 | `.split(key:)` 或 `.feature.view(key:, document:)` |
| target | 从被绑定的 view 推导 feature/document/view | `.target` 直接取出，或原始 feature 的 `.target(view:)` |
| surface 的目标 | 聚合引用已经绑定的 target | 原 `surface(from:, target:, key:)` 或对另一 view 调用 `Editing.bind` |
| presenter key | 调用位的稳定 UI 身份，循环仍要求 item key | `.editor(key: "primary-panel")` 等 |
| design | 继承 owner 的有效 DesignContext | `Editing.open(..., design: .alternate_design)` |
| 标准按键目标 | 当前窗口/owner 合法焦点链中的 Editing target | 原 typed action 的 `target:` / selector |
| 菜单目标 | 打开前的 originating context；菜单会话内固定目标身份 | `.files.menu()` 或显式 `target:` |

隐式 binding/call-site 身份只承诺既定稳定性范围。重命名 scope binding、跨 owner 移动、列表重排或需要跨重构恢复时，应给显式 key；绝不按 URI、标题、裸数组 index 或类型名推断是否共享状态。相同显式 key 出现在同一 owner/对象种类内仍要诊断冲突。

`open` 可以接收带 provider 身份的 ResourceRef，或“资源路径＋resources 参数”。只有 owner 已显式提供唯一合法默认 ResourcePort 时才可同时省略 resources；缺失/歧义要报错，不依次尝试所有 provider。运行期资源装载是异步领域行为，不把 open 解释为 scope 中同步读取全文。

重复 render 不重建组合；给同一组合换 resource 也不默认替换原来的脏文档。切换、关闭、保存冲突和 HMR 仍使用后文的显式协议。

### 4.3 聚合引用可以完整展开

| 表达式 | 类型/含义 |
|---|---|
| `.files` | `EditorBinding`，常见组合的轻量聚合引用 |
| `.files.feature` | 原始 Editing 实例 handle |
| `.files.document` | 当前组合固定绑定的 DocumentSession |
| `.files.view` | 当前组合固定绑定的 EditorView |
| `.files.target` | 从该 view 推导的 EditorTarget |
| `.files.actions.save` | 绑定了具体 target 的 action 引用，可直接交给菜单/按钮 |
| `.files.editor` / `.files.outline` / `.files.status` | 绑定 target 的有类型 surface factory |

v0.1 的 `files` 是原始 FeatureHandle，v0.2 的 `files` 是 EditorBinding；**二者不是静默互换的同一种类型**。本节中 `.files.document` 是属性；显式创建会话的方法在 `.files.feature.document(...)` 上，避免同名属性/方法含混。

示例性等价展开：

```auto
// 推荐写法。
scope {
    files = Editing.open("src/main.at", resources: .file_resources,
                         key: "project-editor")
}
```

```auto
// 对照其语义：独立示例，不与上段同时实例化。
scope {
    feature = Editing(key: "project-editor", resources: .file_resources,
                      design: design.current())
    doc = feature.document(key: "main",
                           resource: .file_resources.resource("src/main.at"))
    main = feature.view(key: "main", document: doc)
    files = Editing.bind(view: main)
}
```

`design.current()` 是上述继承规则的概念表达。`Editing.bind(view:)` 只产生一份聚合引用，feature/document/target 从 view 的受验证归属关系推导，不 open 文档、不新建 Feature、不挂载 native widget。它也不跨 owner 扩大可访问权限。

默认 open 创建的底层 Feature 归调用 owner 持有；`bind` 不改变所有权。解除临时引用或卸载 presenter 不等于关闭文档；释放 owner 仍须协调 dirty/在途保存。返回聚合引用不是丢弃原关闭协议的理由。

```auto
// 两种写法投影同一份已有状态。
.files.editor(key: "main-panel")

// 显式写法：作为上一行的替代，不同时挂载。
surface(from: .files.feature.surfaces.editor,
        target: .files.target, key: "main-panel")
```

`.files.actions.save` 等价于“原实例 actions.save＋files.target”的已绑定动作；容器不能再次把它重路由到 notes。若又给了矛盾 target，要报错。需要更窄条件或不同显示名时，用原 action 绑定扩展，而不是复制 Save handler。

### 4.4 高级场景逐层展开

**同一文档分屏：只增加一个 view。**

```auto
scope {
    files = Editing.open("src/main.at", resources: .file_resources)
    right = files.split(key: "right")
}
view {
    row {
        .files.editor()
        .right.editor()
    }
    row {
        .files.status()
        .right.status()
    }
}
```

`split` 是库工厂，默认沿用 feature/document，创建一个新 EditorView 和 target，再返回 EditorBinding；不会自己决定左右布局。两边共享正文、dirty 和文档 undo，各自持有光标、选区、折叠与滚动。还可以从 `right.document/view/target` 继续显式操作。

**同 Feature 的第二份文档：公开原始 handle，再重新聚合。**

```auto
scope {
    files = Editing.open("src/main.at", resources: .file_resources)
    utility_doc = files.feature.document(
        key: "utility", resource: .file_resources.resource("src/utils.at"))
    utility_view = files.feature.view(key: "utility", document: utility_doc)
    utility = Editing.bind(view: utility_view)
}
```

这与另一次 `Editing.open(...)` 不同：这里复用 Feature 的服务/配置，但有独立 document/history；另一次 open 创建另一个 Feature 实例。共享关系只由显式操作决定。

**固定目标菜单/按钮：直接用已经绑定的出口。**

```auto
menubar {
    .files.menu(title: "文件编辑")
    .notes.menu(title: "笔记编辑")
}
toolbar {
    item(action: .files.actions.save, title: "保存文件")
    item(action: .notes.actions.save, title: "保存笔记")
}
```

这是通用 `Editing.menu()` 的产品选择，不要求两种菜单同时存在。`.files.menu()` 固定 files.target，即使焦点在 notes，也不改目标；它本身不安装第二份全局快捷键。

**自定义快捷键/非标准路由：继续使用显式层。**

```auto
actions {
    // 本片段是 Editing.keys() 的替代示例。
    save_focused = action(id: "review.editor.save",
        command: Editing.actions.save,
        target: focus.command_target(Editing), shortcut: "Primary+S")

    save_file = action(id: "review.file.save",
        command: .files.feature.actions.save,
        target: .files.target, shortcut: "Primary+Alt+S")
}
```

### 4.5 标准菜单与 keymap 的简写规则

`Editing.keys()` 在宿主 actions 中展开为原 `standard_keys` contribution＋默认 focus selector；`Editing.menu()` 在 menubar 中展开为带标题/trigger 的标准编辑菜单，内容复用原 `edit_menu` contribution＋同一目标规则。

- 安装入口是显式的：本例各一行。open、editor/outline/status 挂载均不自动注册全局按键。
- 菜单和 keymap 共享 action 定义；菜单显示有效绑定，不另存快捷键字符串。单独装菜单不会顺便装 keymap。
- 同 owner 重复安装同键映射仍按冲突规则诊断，不因它来自简写就静默去重或 first wins；owner 释放/配置更新按原 contribution 生命周期处理。
- 动态目标只查询当前 AppWindow 和允许的焦点链。没有 Editing target 时禁用；目标只读/已关闭时拒绝，不回落到其他实例，也不采用隐式“上次编辑器”。
- 菜单仍在打开前捕获目标身份、执行时重算状态和权限；进入菜单不丢目标。固定目标入口不受动态焦点替换。
- 生成 binding 身份由 owner、贡献模板和明确安装点决定；Inspect/诊断必须可以展开看到具体 keymap、receiver、target 与来源。

默认值可以少写，但这些路由/生命周期语义不能随简写一起被省掉。

### 4.6 完全显式的对照写法（保留 v0.1）

下列是 **v0.1 语法提案**，不是可运行源码。`auto.editing` 代表未来依赖包入口；依赖声明仍归 pac.at/现有包工具，此处不提出新的包解析器。

```auto
use auto.editing: Editing
use auto.ui: ResourcePort, DesignContext, focus

widget ReviewApp(file_resources ResourcePort,
                 note_resources ResourcePort,
                 design DesignContext) {
    scope {
        // 同一个定义，两份由 ReviewApp 持有的独立实例。
        files = Editing(key: "files",
                        resources: .file_resources, design: .design)
        notes = Editing(key: "notes",
                        resources: .note_resources, design: .design)

        // 故意都用 main：key 只要求在各自 owner/对象种类内唯一。
        file_doc = files.document(key: "main",
                                 resource: .file_resources.resource("src/main.at"))
        note_doc = notes.document(key: "main",
                                 resource: .note_resources.resource("notes/42"))

        file_view = files.view(key: "main", document: file_doc)
        note_view = notes.view(key: "main", document: note_doc)

        file_target = files.target(view: file_view)
        note_target = notes.target(view: note_view)
    }

    actions {
        // 通用快捷键只注册一次；target 在输入事件发生时解析。
        save_focused = action(id: "review.editor.save",
                              command: Editing.actions.save,
                              target: focus.command_target(Editing),
                              shortcut: "Primary+S")
        undo_focused = action(id: "review.editor.undo",
                              command: Editing.actions.undo,
                              target: focus.command_target(Editing),
                              shortcut: "Primary+Z")
        redo_focused = action(id: "review.editor.redo",
                              command: Editing.actions.redo,
                              target: focus.command_target(Editing),
                              shortcut: "Primary+Shift+Z")

        // 显式目标入口：即使焦点在另一边，也操作指定实例。
        save_file = action(id: "review.file.save", title: "保存文件",
                           command: .files.actions.save, target: .file_target)
        save_note = action(id: "review.note.save", title: "保存笔记",
                           command: .notes.actions.save, target: .note_target)
    }

    view {
        col(style: "h-full") {
            menubar {
                menubar-menu(value: "edit") {
                    menubar-trigger "编辑"
                    menubar-content {
                        menubar-item(action: .actions.save_focused)
                        menubar-item(action: .actions.undo_focused)
                        menubar-item(action: .actions.redo_focused)
                        menubar-separator
                        menubar-item(action: .actions.save_file)
                        menubar-item(action: .actions.save_note)
                    }
                }
            }

            row(style: "flex-1 overflow-hidden") {
                // 大纲的位置不必邻接编辑面板。
                col(style: "w-56") {
                    surface(from: .files.surfaces.outline,
                            target: .file_target, key: "file-outline")
                    surface(from: .notes.surfaces.outline,
                            target: .note_target, key: "note-outline")
                }
                col(style: "flex-1") {
                    surface(from: .files.surfaces.editor,
                            target: .file_target, key: "file-editor")
                    surface(from: .notes.surfaces.editor,
                            target: .note_target, key: "note-editor")
                }
            }

            // 离开上方布局子树，也仍然引用准确的 document/view。
            row {
                surface(from: .files.surfaces.status,
                        target: .file_target, key: "file-status")
                surface(from: .notes.surfaces.status,
                        target: .note_target, key: "note-status")
            }
        }
    }
}
```

这份宿主代码只完成：提供资源接口、创建实例/会话、选择布局与绑定动作。它没有复制正文、dirty/undo 状态，也没有重写 Save/Undo handler。

`Primary` 是本提案的可移植主修饰键名，按平台映射 Ctrl/Cmd；例中的 redo 是显式选定的绑定，不声称各平台默认键完全相同。产品也可以选择平台 keymap profile。

### 4.7 scope 的精确语义

- `scope` 的 binding 在块内按声明名引用，如 `files`；块外使用 owner 成员，如 `.files`。同一 owner 内与 props/model/setup 成员重名报错。
- `Editing(key:, ...)` 的 `key` 是统一装配身份参数，不混入 feature 的业务 props；定义身份由类型/包决定，实例身份由 owner＋key 决定。
- 一次 scope activation 解析一张有向无环装配图。重复 render 不执行 factory；HMR 依据稳定声明/显式 key 协调保留或迁移。循环、重复声明键、身份/类型冲突报错。
- `document(...)` 在所属实例内按 session key 获取/创建会话；同 key 绑定不同 resource 拒绝，不能把已有脏文档悄悄改成另一份资源。同 resource 用不同 key 创建独立 session 是显式选择，不自动按 URI 去重。
- `view(...)` 在实例内按 view key 创建独立 view，要求 document 属于该实例。v0.1 的 view/document 关系不原地改绑；切 tab 选择另一份 view/target，不能偷偷改变已有 target 的含义。
- `target(view:)` 是只读组合引用，document 从 view 推导；不让调用方重复传一份可能矛盾的 document。
- `resource(...)` 此处是 provider 的资源身份解析，不代表同步读取全文；实际加载由 document owner 调度，并暴露 loading/error。
- 临时折叠、移动、重建 presenter 不结束父 scope；scope 的释放由 owner/关闭合同决定。声明 scope 的 widget 若真正被销毁，必须先执行它拥有的 Feature 关闭协调，不能无条件丢弃脏状态。需要跨路由保留时将 scope 提升到 App owner。

配置/provider 的变化不是“重新 render 一次就重建实例”；workspace/ResourcePort 变化需要显式关闭/切换协议，含 dirty veto。本例将它们视为该 App session 内稳定的依赖。

## 5. 隔离和共享分别发生在哪里

本节的显式代码沿用 §4.6 的 v0.1 原始 handle 名字；v0.2 对应的原始入口为 `.files.feature`，文档/视图/目标可从聚合引用直接取出，语义一致。

### 5.1 两个实例默认隔离

| 数据/资源 | files 与 notes 的关系 |
|---|---|
| 文档集合、buffer、dirty、undo、保存队列 | 独立 |
| EditorView 光标、选区、滚动 | 独立 |
| 逻辑 action ID（如 auto.editing.save） | 同一个定义；运行期接收者由 instance target 区分 |
| provider | 可以不同；即便注入同一个 provider，也不因此共享未保存 buffer |
| design | 可以显式共享只读设计上下文；不意味着业务状态共享 |
| 标准菜单/keymap 模板 | 共享定义，不共享当前命令目标 |

因此两边都使用 `key: "main"` 不串线，命令也不通过 `EditorStore.active_key` 进程变量决定对象。native registry key 必须从 app/feature/view 身份派生，不能继续用裸 `"main"` 或标题作为全局 key。

两份独立 session 打开同一个实际文件，仍然是两个编辑缓冲区；外部保存必须使用 provider 的 expected resource revision/hash 检查并发冲突。实例隔离不等于能够忽略共同磁盘资源的冲突。

ResourcePort 的产品适配是明确的复用边界：workspace adapter 将 read/write 映射到文件，record adapter 将它们映射到记录 API，并分别返回自己的 resource revision、readonly、冲突和错误。Editing 统一提供脏状态、保存中的用户行为、历史和视图；宿主不通过复制 `.Save` 来适配“保存笔记”。如果服务只能提交草稿、不能提供所选 profile 所需的写入语义，应选择不同 profile/显式 action，不能让 adapter 假报保存成功。

### 5.2 一个实例内，三个出口共享一个目标

| 出口 | 读取的状态 | 写入/行为 |
|---|---|---|
| editor | `target.document` 的正文/dirty；`target.view` 的选区/视口 | 输入进入 document mutation；光标和滚动进入 view |
| outline | 同 document 的 outline projection | 点击条目向 target.view 发 Reveal/Select，不修改全局 active editor |
| status | document 的 encoding/EOL/dirty；view 的行列/选区 | 只读展示；可选状态动作仍带同一 target |

**状态栏的行列属于 view，encoding/dirty 属于 document。** 不能为了方便把它们全部复制进一个“状态栏 store”。

Outline 的解析结果要带对应 document revision；点击过期条目时先转换锚点或拒绝/重算，不用旧字符偏移盲跳。两个实例异步解析的结果通过各自 owner/generation 回填。

### 5.3 同一文档的两个编辑视图

若需求是分屏查看同一文档，增加 view，**不要再建一个 Feature 实例或 document**：

```auto
// 增补到同一个 scope 中。
file_split = files.view(key: "split", document: file_doc)
file_split_target = files.target(view: file_split)
```

```auto
// 两个 editor surface 各自绑定不同 view。
row {
    surface(from: .files.surfaces.editor,
            target: .file_target, key: "main-editor")
    surface(from: .files.surfaces.editor,
            target: .file_split_target, key: "split-editor")
}
```

结果：共享 buffer、dirty、保存基准和文档 undo；光标、选区、折叠、滚动各自独立。Undo 恢复文档变更后，所有 view 的 selection/anchor 由领域内核转换到新 revision。不能只更新原 view 而让另一份使用失效偏移。

一个 EditorView 同时只允许一个 editor presenter 占用其输入/视口；outline/status 是 observer 不占用此资格。把同一个 view 原样挂成两个编辑器应诊断 `ViewAlreadyMounted`，而不是产生两个争抢同一 native widget 的界面。

### 5.4 不同 Feature 实例能否共享一个 document？

v0.1 的原始层明确拒绝直接跨 owner 绑定：`notes.view(document: file_doc)` 不合法；v0.2 聚合层同样禁止 `notes.feature.view(document: files.document)`。这个例子既要实例隔离，又要显式共享，应通过“同一个实例下的多个 view”表达，边界最清楚。

若后续确需两个独立 Feature 实例共用未保存 buffer，应由上层显式拥有 DocumentSession/DocumentService，再授权两个实例消费同一会话。那会改变关闭、保存与历史 owner，不应通过相同 URI、相同 key 或导入全局 store 偷偷实现。此项不是 v0.1 已支持承诺。

## 6. Feature 包作者公开什么

宿主只需上一节的使用方式。包作者另外声明工厂、surfaces、typed actions，保持“Action 为声明绑定，Event 为执行”的现有分工。

以下是合同骨架，DocumentSessions/EditorViews 的方法都是拟议库 API；这里只描述调用关系，不实现编辑器算法：

```auto
feature Editing(resources ResourcePort, design DesignContext) {
    scope {
        documents = DocumentSessions(key: "documents", resources: .resources)
        views = EditorViews(key: "views", documents: documents)
    }

    pub fn document(key str, resource ResourceId) -> DocumentSession {
        return .documents.ensure(key: key, resource: resource)
    }
    pub fn view(key str, document DocumentSession) -> EditorView {
        return .views.ensure(key: key, document: document)
    }
    pub fn target(view EditorView) -> EditorTarget {
        return .views.target(view)
    }

    surfaces {
        editor = EditorPanel
        outline = OutlinePanel
        status = EditorStatus
    }

    msg { Save(EditorTarget), Undo(EditorTarget), Redo(EditorTarget) }

    actions {
        save = action(id: "auto.editing.save", title: "保存",
                      target: EditorTarget, handler: .Save,
                      enabled_if: target.document.writable &&
                                  target.document.dirty &&
                                  !target.document.saving)
        undo = action(id: "auto.editing.undo", title: "撤销",
                      target: EditorTarget, handler: .Undo,
                      enabled_if: target.document.writable &&
                                  target.document.history.can_undo)
        redo = action(id: "auto.editing.redo", title: "重做",
                      target: EditorTarget, handler: .Redo,
                      enabled_if: target.document.writable &&
                                  target.document.history.can_redo)
    }

    on {
        .Save(t) -> { return .documents.Save(t.document) }
        .Undo(t) -> { return .documents.Undo(t.document, origin: t.view) }
        .Redo(t) -> { return .documents.Redo(t.document, origin: t.view) }
    }
}
```

`feature` 内的 `scope` 不输出视图。`surfaces` 导出组件描述符，三个组件都声明 `target EditorTarget` 输入；挂载器按描述符校验，传入 feature/design 上下文和 target。普通 widget 的 `model` 只存它自己的 presenter 状态。

示例中的 Save 返回值代表可完成的操作结果；同步/异步结果的统一适配属于后续运行时合同。包内必须执行 provider 权限、resource revision、目标存活性等实际检查，不能以 `enabled_if` 代替。此段不承诺现有 on handler 已支持这种 typed async result。

`EditorTarget` 的公开身份至少包含 instance/document/view handle 及 lifetime generation。它不携带正文副本、裸 VM heap ID、native pointer 或权限提升能力。文档/view 的公开只读字段可订阅，变更通过领域 action；构造器验证 view 与 instance 的归属。

一个状态出口的作者视角可写成：

```auto
widget EditorStatus(target EditorTarget) {
    view {
        row(style: "gap-3 text-muted-foreground") {
            text .target.document.display_name
            text .target.document.encoding
            text .target.document.eol
            if .target.document.dirty { text "未保存" }
            text "${.target.view.caret.line}:${.target.view.caret.column}"
        }
    }
}
```

这份组件不 `use editor_store: EditorStore`，也不依赖宿主产品名。`caret`、outline、dirty 是同一权威数据的投影；订阅的 lifetime 归 presenter，数据 lifetime 仍归 document/view owner。

## 7. Action 的三层身份，避免两套执行系统

下表的 `.files` 沿用原始 FeatureHandle 命名。v0.2 的 `.files.feature.actions.save` 是该实例引用，而 `.files.actions.save` 已进一步绑定到聚合引用的具体 target；两者的类型差异不能丢失。

| 层 | 示例 | 意义 |
|---|---|---|
| 定义 | `Editing.actions.save`，逻辑 ID `auto.editing.save` | 类型化语义 action，声明 target schema、enablement、handler |
| 实例绑定 | `.files.actions.save` | 同一定义预绑定接收者为 files |
| 宿主触发绑定 | `.actions.save_focused`，ID `review.editor.save` | 给命令面板/快捷键/菜单一个 App 内可配置入口 |

`command:` 表示复用一个 action contract；`handler:` 表示该 action 的最终执行事件。二者不能在同一 action 中同时声明。宿主绑定不重新实现 Save，也不复制 enablement。

`save = action(...)` 的 `save` 是显式成员别名；不从字符串 ID 猜测 `.actions` 成员名。`title`、`enabled_if`、目标类型来自定义，宿主可以覆写显示名，额外收窄条件，但不能去掉定义的约束或覆盖 provider 权限。

`target: EditorTarget` 出现在**定义**里表示类型；`target: .file_target` 或 selector 出现在**调用绑定**里表示目标来源。这种位置区别需要 parser/LSP 清楚诊断，不能作为普通运行值混过去。

所有触发最终都变成向确定 feature receiver 派发 `.Save(t)` / `.Undo(t)`，不新增一套绕开 on 的执行器。运行注册表的键包含 app session/feature instance/逻辑 action ID；相同包两实例不互相覆写。

## 8. 焦点、菜单与快捷键如何命中正确实例

### 8.1 每个 surface 发布自己的命令上下文

`surface(from:, target:)` 除了渲染，还为其交互根发布该 target 的 command context。它不将内部键盘监听无条件注册为全局捕获。

- editor 获得焦点 → files 或 notes 的对应 target。
- 点击 file outline → 上下文仍是 file target；Reveal 发给 file_view，即使笔记面板仍可见。
- 点击 note status 的编码按钮 → 使用 note target；只读文字不抢焦点。
- 焦点进入与编辑无关的 App 表单 → 不存在 Editing target，不自动采用“最近编辑过的一个”。

宿主额外按钮区可以用拟议的库容器 `command-scope(target:)` 关联：

```auto
command-scope(target: .file_target) {
    toolbar {
        item(action: .actions.save_file)
    }
}
```

这只给该容器的子节点提供命令上下文，不移动 state owner，也不给无关代码任意写 document 的权限。

### 8.2 focus.command_target 返回 selector，不返回全局 active_editor

`focus.command_target(Editing)` 的类型是 `TargetSelector<EditorTarget>`。它受当前 AppWindow、modal 和可见/存活 surface 约束；保存的是选择规则，不是 scope 初始化时捕获的一份焦点值。

固定 `EditorTarget` 与 selector 都可以作为 action 的目标来源：前者不猜焦点；后者在每次 invocation 捕获。没有可用 target 时，action 显示禁用/不可执行；不会向所有实例广播。

### 8.3 菜单打开时捕获，执行时再验证

菜单的焦点会从 editor 移到 menu。为此，菜单容器在打开**之前**为这一菜单会话捕获 originating command context；所有动态目标菜单项使用该 context，打开子菜单沿用原捕获。

窗口菜单以打开前的焦点上下文为来源；右键菜单优先使用触发节点声明的 target，因此右键点击 files 时，不沿用尚在 notes 的键盘焦点。键盘打开上下文菜单使用当前焦点节点的 target。三种来源最终都冻结成同一种 EditorTarget，不能在执行时重新猜。

```text
在 notes 编辑 → 打开“编辑”菜单
  → menu session 捕获 note_target
  → 焦点进入菜单，不把 target 清成 None 或改成 files
  → 点击“保存”
  → 校验 notes / note_doc / note_view 的 identity + generation 仍有效
  → 重算 writable / dirty / saving 与 provider 条件
  → 向 notes 派发 Save(note_target)
```

捕获的是**目标身份**，不是打开菜单时的旧 dirty 值，也不是一份旧正文。执行时保存仍指向被捕获的文档，读取它的最新有效状态；如果文档已关闭、view 已换代或权限已撤销，则失败/禁用，不能跳到另一个文档。

菜单保持打开期间，若用户通过其他入口切换 tab，该菜单也不重新跟随；它仍指向原 target。界面应显示捕获目标的标题/不可用状态，不能悄悄改变“保存”的含义。菜单关闭后释放捕获。

显式的 `.actions.save_file` 从一开始就固定 file_target，不受菜单捕获的动态 Editing 目标替换。

### 8.4 键盘路由与拒绝规则

建议顺序：平台/IME 保留输入 → 当前 modal/局部 widget 明确消费 → focused command context → App 声明的适用绑定。普通输入框中的 Ctrl/Cmd+Z 先撤销该输入，不抢去撤销文档；editor 内核的文档 undo 与 action adapter 必须协作只提交一次，不能 native 与 global listener 各做一次。

选定 target 后，`Disabled`、`Readonly`、`StaleTarget`、执行错误都是该次 invocation 的终态，**不得再寻找另一个编辑器**。只有输入路由中的显式 `NotApplicable` 才可继续寻找更外层绑定，且不能把“目标文档只读”编码成 NotApplicable。

同一 scope、同优先级、相同按键且适用条件重叠时诊断冲突，不按安装顺序 first wins。绑定值动态变化造成歧义时，也应拒绝歧义执行并报告，而不是随机选实例。

### 8.5 一张行为表

| 用户操作 | 结果 |
|---|---|
| files editor 中 Primary+S | 保存 file_doc |
| notes editor 中 Primary+S | 保存 note_doc |
| 焦点在 notes，点击“保存文件” | 保存 file_doc |
| 点击 file outline 后 Primary+S | 保存 file_doc，不保存 notes |
| notes 打开菜单后菜单获得焦点，点击通用保存 | 保存打开菜单前的 note_doc |
| 菜单打开后 note_doc 被关闭 | 拒绝陈旧目标，不转存 file_doc |
| notes 只读，按 Primary+S | notes 动作禁用/拒绝，不回落到 files |
| 编辑器内部查找输入框中 Primary+Z | 输入框自身 undo 先消费；不重复撤销文档 |
| 焦点在不相关业务表单中 Primary+S | Editing 绑定不适用；只允许产品显式定义的业务表单动作 |
| file_view 与 file_split 共享文档，split 中 Undo | 更新同一文档历史；两个 view 收到一致增量，目标 view 单独处理光标 |

## 9. 功能包如何连菜单一起复用

§4.6 的完全显式示例刻意展开动作，方便看清路由。v0.2 推荐用 §4.1 的 menu/keys 入口；本节保留其底层模板合同，供包作者、自定义菜单和兼容实现使用。

拟议包侧合同：

```auto
// 放在 Editing 的定义中；这些是模板，不在每次实例化时自动全局安装。
contributions {
    edit_menu = menu {
        item(command: Editing.actions.save)
        sep
        item(command: Editing.actions.undo)
        item(command: Editing.actions.redo)
    }
    standard_keys = keymap {
        binding(key: "Primary+S", command: Editing.actions.save)
        binding(key: "Primary+Z", command: Editing.actions.undo)
        binding(key: "Primary+Shift+Z", command: Editing.actions.redo)
    }
}
```

宿主将通用菜单插入自己的菜单树：

```auto
menubar-content {
    contribute(from: Editing.contributions.edit_menu,
               target: focus.command_target(Editing))
}
```

固定 files 的右键菜单则可以：

```auto
context-menu-content {
    contribute(from: .files.contributions.edit_menu, target: .file_target)
}
```

快捷键可以在宿主 `actions` 中一次性安装：

```auto
actions {
    contribute(from: Editing.contributions.standard_keys,
               key: "editing-defaults",
               target: focus.command_target(Editing))
}
```

**这是主示例 `save_focused/undo_focused/redo_focused` 快捷键绑定的替代写法，不是同时追加一份。** 宿主仍可保留 save_file/save_note 等没有同键 shortcut 的固定入口。

这里 `target` 始终表示命令对象；贡献插入的位置由外层菜单/actions 容器决定，不能与 Presentation Host 的 Region/容器目标混用。menu/keymap 模板不包含窗口退出、产品工作区切换等 App 私有行为。

定义级 `Editing.contributions.*` 在解析 target 后选中实际实例；实例级 `.files.contributions.*` 只接受 files 的目标。模板展开保留来源，生成稳定且不冲突的 binding 身份；菜单键位显示引用有效 keymap，不能又手写一份字符串。

## 10. 放进 Region 与跨布局移动

核心示例把 surface 放在普通 row/col，已证明 UI 出口与长期会话分离。若 App 使用 Presentation Host，可以在 Region 填充体中放同一节点：

```auto
// Region/绑定块仍待 Presentation Host 单独细化；此处展示填充内容。
surface(from: .files.surfaces.editor,
        target: .file_target, key: "file-editor")
```

把这份挂载从 main 迁移到 bottom，保留 file_view/file_target，只 detach/attach 或重建 presenter。不可同时把同一 editor view 挂两次；要同时呈现请创建 split view。

Region 的生命周期、大小与 overlay policy 由宿主管理；Feature 不接管宿主全局布局。关闭大纲/状态栏仅取消这些 observer，不关闭 file_doc。关闭 editor presenter 也不等于关闭文档，文档关闭由显式 session action 协调。

## 11. 保存、关闭、热重载的必要语义

### 11.1 保存不是把 dirty 直接设 false

保存开始时，document owner 取得对应本地 revision 的内容快照及 provider 的 expected resource revision；文件或记录 API adapter 执行同一 ResourcePort 写入语义。

保存期间继续编辑时，成功结果只更新那份成功保存的基准，dirty 应按当前内容相对基准计算，不能无条件清空。provider conflict/error 归本实例/本会话；不能因保存失败尝试另一个 target。只读、重复保存、保存中关闭的规则由领域会话统一提供，宿主不重写。

本例不引入跨 files/notes 的原子 SaveAll。两种 provider 不能因为菜单叫“全部保存”就获得跨文件/跨服务事务保证。

### 11.2 关闭要区分四种动作

| 动作 | 影响 |
|---|---|
| 收起 outline/status | observer 休眠或卸载，会话仍活着 |
| 移动/卸载 editor surface | 保留长期 view/session；释放 presenter/native 挂载资源 |
| 关闭 EditorView | 使该 view/target generation 失效，不默认销毁仍被保留的 document |
| 关闭 Document / dispose Feature | 处理 dirty、保存中状态、订阅与引用；可 veto/等待/取消后释放 |

App 离开页面或退出时统一协调，Feature 不自己弹一个不可控的产品退出流程。确认 UI 通过宿主 DialogPort/关闭合同提供；用户拒绝时，不能先把 owner 销毁了再宣称 veto。

### 11.3 热重载保留什么

scope/feature/document/view 身份和兼容 schema 不变时，保留会话和视图状态；更新 presenter/handler 后重新建立观察与贡献。保存中的任务按安全点 drain/cancel/reject 策略处理；旧 epoch 回填不能写入新一代实例。

菜单会话和已有 invocation 捕获的旧 target 在生命周期换代后必须失效或经显式兼容重绑定；不能因为 key 字符串没变就假定旧 handle 一定可用。scope 中的 factory 也不因每次 HMR 重新执行一遍而重复开文件、建 SSE 或注册快捷键。

## 12. 编译期与运行期诊断

下列名字是候选诊断名，不是已经存在的错误码。

| 情况 | 期望诊断/行为 |
|---|---|
| files/notes 两个 Feature 在同 owner 使用同一实例 key | `DuplicateInstanceKey` |
| `files.view(document: note_doc)` | 静态可确定时编译错误，否则装配时 `ForeignDocumentOwner` |
| `.files.actions.save` 配 `note_target` | `ActionTargetOwnerMismatch`，不重路由到 notes |
| `.files.surfaces.status` 配 `note_target` | `SurfaceTargetOwnerMismatch` |
| 同一 view 同时挂两个 editor surface | `ViewAlreadyMounted`，提示新建 split view |
| 同 document key 却更换 resource | `SessionResourceMismatch`，需要关闭/新建协议 |
| 字符串拼错 surface 或 action 成员 | 编译期未知成员，不降级成空 UI |
| 全局两份适用的同键绑定 | `AmbiguousKeyBinding`，标明来源/优先级/条件 |
| 菜单捕获后 target 被关闭 | `StaleTarget`，不访问释放对象、不落到别处 |
| 当前没有 Editing target | 动态绑定不可执行，不使用上次全局编辑器 |

不同 Editing 实例共享同一个静态类型；不要求首期引入复杂的依赖类型。编译器能推断 owner 的地方提前检查，运行时同样验证每个 handle 的 app/instance/generation 与 provider 权限。

## 13. 本例的候选验收与尚待冻结内容

未来测试至少覆盖上述行为表，并增加：同 URI 双 session 保存冲突、两个 scope 都叫 main 的隔离、outline 陈旧 revision、分屏 selection 转换、页面关闭 veto、多轮 HMR 后注册表/订阅数量有界，以及 Vue/VM 相同操作 trace。

本轮只做文档自洽检查，不执行这些验收，也不把现在 VM 的根展平/回调限制写成已解除。

优先需要冻结的少数语法选择是：

1. `scope` 是否作为通用稳定装配块；与现有 setup 的职责区分及关闭 veto 如何落到 owner lifecycle。
2. 是否接受 action/surface 的有类型成员引用，以及 `command:` 复用定义、`handler:` 最终执行的二选一规则。
3. target 的不可变身份与 selector 的延迟解析，以及菜单会话捕获身份、执行端实时校验的默认行为。
4. v0.1 是否只支持同 Feature 内多 view 共享 document；本稿建议如此，跨实例共享另立显式合同。
5. 是否接受 v0.2 的 EditorBinding 聚合入口及 bound surface/action 表达式：默认 open 创建常见组合，bind 只引用，split 只增加 view；原始 handle 始终可取出。
6. 省略 key/design/target 时的默认值是否足够确定，是否能由开发工具展开解释；简写不能改变隔离、关闭、迁移与输入路由合同。

后续即使调整关键字拼写，也应保留本例的所有权、路由和失败行为。它们是这份语法草案真正要表达的合同。
