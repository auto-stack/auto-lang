# App Presentation Host：声明式呈现区域与 Feature UI 装配

> 状态：Draft，2026-10-14。用户已提出方向，本文给出待验收的技术设计；没有实现新语法或修改运行时。
> 来源：用户要求 App 先声明灵活 viewport，Feature Pack 再绑定 UI 块；支持左/右/下区域、菜单、顶中悬浮框、通知和整页栏目，允许固定与部分动态加载。
> 关联：[Design 38](../38-autoui-features-and-system-services.md)、[Design 20](../20-autoui-separation-architecture.md)、[Design 31](../31-autoscape-autoweb.md)、[Design 37](../37-v06-modularization-and-repo-boundaries.md)、[嵌套生命周期](component-time-and-events.md)、[滚动体系](universal-scroll-architecture.md)。

## 1. 定案方向与建议术语

**App = 产品模型/流程 + Presentation Host 声明 + Feature/本地 UI 绑定。**

App 首先声明自己提供的呈现目标与组合规则，Feature Pack 导出独立 UI 出口；装配时选择哪些出口绑定哪些目标。框架管理几何、焦点、切换、生命周期与恢复。位置不再由功能包写死。

用户称 viewport 的抽象可保留为对外称呼；技术文档建议称 **Region（呈现区域）**，整个窗内组织机制称 **App Presentation Host（应用呈现宿主）**。现有 ScrollPane 和 AutoFrame 的 viewport 指可视坐标/滚动域，沿用同名会造成混淆。

新增的是框架内的**App 组合底座**，不增加 Widget/Blueprint/Feature/App 之间的第五个业务粒度。它也不是 OS WindowManager：OS/rqhost 管顶级窗/surface，Presentation Host 管 App 自己的内容组织。

建议关系：

```text
AppWindow
  └─ PresentationHost
       ├─ Region tree / Layout engine
       ├─ Section navigation / Scene state
       ├─ Overlay / Focus / Input coordinator
       ├─ Typed contribution targets
       └─ Surface bindings → Feature views / App local UI
```

所有 App 可以声明 Region，但简单工具也可以只声明一个 root content Region。没有文件项目的读书、下载、聊天 App 不需要制造 workspace、editor group、Activity Bar 或空白右栏。

## 2. 研究与本地依据

- VS Code 官方已支持移动 views、调整侧栏/panel 和主区布局。因此它的局限不能概括成“完全固定不能移动”；我们需要进一步解除的是编辑器/workbench 的默认语义与固定区域词汇。[Custom Layout](https://code.visualstudio.com/docs/configure/custom-layout)
- Zed 官方支持 Classic/Agentic 布局，并将多项目/threads 和 editor-oriented 组织分别呈现。可借鉴具名布局变化，不将 project/thread 身份作为所有 App 的基础模型。[Windows & Projects](https://zed.dev/docs/windows-and-projects)
- JupyterLab shell 提供 top/menu/left/right/main/down/bottom/header 等目标，widget 可加入不同区域。可借鉴“内容与目标分离”，AutoUI 则允许每个 App 自定义区域和容器树。[Common Extension Points](https://jupyterlab.readthedocs.io/en/stable/extension/extension_points.html#jupyter-front-end-shell)

2026-10-14 本地只读检查：`docs/schema/aura.at` 已声明 slot、popover、scroll-pane、toaster/notification/overlay 等；Blueprint 有布局 slot；`component-time-and-events` 管嵌套挂载订阅；`universal-scroll-architecture` 管滚动域；Design 38 已描述多出口 Feature。它们是基础资产，但不构成已有的 typed Region registry、布局持久化或完整 Surface relocation 能力。本轮未做运行验收。

不回改历史 Desktop Shell 设计中的 OS 桌面职责。本文归 AutoUI 框架域；具体 Musk/Jade/Reader 壳的实现设计分别进入产品仓。

## 3. 区分 Region、Section、Surface 和 Contribution

| 概念 | 表达什么 | owner |
|---|---|---|
| Region | 可承载内容的命名目标，拥有类型、容量和展示政策 | App 声明，Host 管理 |
| Layout Node | Region 如何排列：split/tabs/stack/grid/overlay 等组合 | App 默认布局＋受控用户布局 |
| Section（栏目） | 用户正在进入哪项产品活动，可切换主内容或整套内容布局 | App 导航/scene policy |
| Surface | Feature 输出的可挂载 UI 单元及其实例/会话绑定 | Feature 提供，Host 挂载 |
| Contribution | 菜单条目、命令、通知、状态项等结构化贡献 | Feature/App 提供，专用 target 合成 |

左侧栏是位置/Region，Sidebar 栏目入口是导航，两者独立。栏目可以从左侧导航进入，但占据中央或整个 App 内容区；也可以从顶栏、底部导航或命令进入。

菜单并非普通自由 View 容器，通知也不是 Feature 可任意操纵 z-order 的 Region。对外统一称“呈现目标”，内部需要类型：content、command-group、status-items、notifications 等。类型不匹配在装配时拒绝。

## 4. Region 不是写死的七个位置

允许 App 自定义 Region ID，如 `conversation.detail`、`library.catalogue`、`review.changes`、`reader.annotations`。left/right/bottom 只是常见布局位置，不是必需的唯一 ID。

Region 的最小语义建议：

| 字段 | 合同 |
|---|---|
| identity | 稳定 ID，属于 app/window/scene；不依赖屏幕坐标或产品文件名 |
| target_kind | 接收 view、command group、status items、notification event 等类型 |
| presentation | docked/inline/overlay/page 等允许形态；表达容器政策，不改变领域能力 |
| composition | single/tabs/stack/split/grid；决定多个 Surface 如何共存 |
| capacity | 是否允许多份内容、多实例；single 有占用时明确 replace/open-new/reject 政策 |
| constraints | min/max/preferred size、可折叠、可调整、允许迁移/分割；布局求解由 Host 负责 |
| scope | 哪些 scene/section 可见；global/window/section/overlay owner |
| empty_policy | 不显示、显示 App 空态、保留空间或显式占位 |
| lifecycle | hidden/suspended/unmounted 的区别；无效目标何时释放 |
| fallback | 小屏/不可用 Region 的有序合法替代目标；没有替代必须给明确错误 |

为普遍使用提供具名 Blueprint/preset：single-content、navigation-pages、reading、split-detail、studio。它们只是可展开的默认 Region/Layout 声明，App 可以重命名、增删和嵌套。不要用 preset enum 将整个框架锁成几套固定壳。

Region 可嵌套：栏目内部可有自己的 split/detail/overlay；不能为全窗口无限注册全局 ID。解析优先使用当前 owner scope，跨 scene/window 引用必须显式。

## 5. 两个组织轴：布局与栏目/Scene

### 5.1 Layout tree

一个通用布局树由有限原语组成：leaf（Region）、split（方向和尺寸）、tabs、stack/grid，以及不占常规流的 overlay layer。每个原语有独立几何和交互合同，避免给一个 AppShell 塞进几十个开关。

Layout tree 只承载几何与展示约束，不存 Feature 的文档、知识库或 job 数据。Layout Engine 接收窗口尺寸、Region constraints 与布局状态，给出可见区域和 bounds；Vue/VM 适配器消费相同语义，像素舍入差异容差明确。

不第一版实现任意拖拽 docking。首先完成声明式排列、resize/collapse、移动到声明目标和可恢复状态；拖拽使用相同 relocation 命令，后续追加输入方式，不另造布局真相。

### 5.2 Section 和 Scene

Section 表达导航身份；其 destination 可以是某 Region 的页面，也可以是一个 Scene。

Scene 是该栏目激活的**布局/贡献可见性配置**，仍在同一个 AppWindow 内，不是新 App 或 OS 桌面。简单栏目可只替换 main page；复杂栏目可以切换整个内容布局，而保留 App chrome、必要导航和通知入口。

例如 Musk：

```text
栏目 对话 → conversation scene：会话列表 + 对话主区 + 可选知识面板
栏目 知识 → knowledge scene：知识导航 + 笔记主区 + 反链/搜索
栏目 文件 → files scene：文件导航 + 编辑/diff + 可选工具面板
栏目 下载 → downloads scene：任务主页面
```

栏目不必须与 Feature Pack 一一对应。一个栏目可以组合多个包，一个包可以参与多个栏目。宿主编排绑定；包可以贡献 section 描述/默认装配建议，是否允许成为顶级栏目由 App 决定。

“独占整个 app 窗口”默认指**App 内容区域**。若要求连导航/chrome 一起隐藏，则提供 explicit immersive presentation 和明确返回入口，不能自行藏掉窗口控制或系统提示。

## 6. Feature 出口与装配

### 6.1 Pack 输出 SurfaceDescriptor

每个 UI 出口建议声明 surface_id、role、可用 presentations、所需 target capabilities、min size、session/实例关系、multi-view 支持、激活/可见时调度政策。包声明“可作为 inspector/page”；App 映射为 right/bottom/main。不声明“必须在全局右边 300px”。

View 形态须有真实支持：紧凑列表不是自动成为整页知识工作台。可复用同领域模型，提供 compact/full 两个 presenter；输出不兼容的目标要拒绝，不通过强行缩放拼成“支持”。

命令、菜单、通知等 contributions 继续引用 Design 38 的统一 command/权限链；不要把“编辑菜单”作为可以随意搬去 main Region 的 View。

### 6.2 装配流程与选择优先级

顺序为：声明 Host → resolve packs/profiles → 校验出口与目标 → 合成贡献 → 激活会话 → 按 scene 挂载。

默认绑定由 App 选择；Pack 可提供安装建议。用户可以在 App 允许的目标集内重新放置；UI 位置修改不扩大资源权限。有效位置由合法用户布局优先于 App 默认，受 App structural policy 约束；包 fallback 只补充宿主显式接受的策略。优先级不通过安装顺序决定。

必需出口缺目标、single Region 占用冲突、未知 surface/Region ID、目标类型不匹配，都返回带来源的诊断；可选出口可以不挂。动态装配先 prepare/validate，再原子发布有效贡献与绑定，失败保留原布局。

### 6.3 概念声明示例（JSON，非现有 pac.at/Auto DSL）

```json
{
  "host": {
    "regions": {
      "body": {"kind": "view", "composition": "single"},
      "tools": {"kind": "view", "composition": "tabs"},
      "prompt": {"kind": "view", "presentation": "overlay", "anchor": "top-center"},
      "edit-menu": {"kind": "command-group"},
      "status": {"kind": "status-items"},
      "alerts": {"kind": "notifications"}
    },
    "scenes": {
      "conversation": {"layout": {"split": "horizontal", "children": ["body", "tools"]}},
      "knowledge": {"layout": {"region": "body"}}
    },
    "sections": [
      {"id": "chat", "title": "对话", "scene": "conversation"},
      {"id": "wiki", "title": "知识", "scene": "knowledge"}
    ]
  },
  "bindings": [
    {"instance": "wiki.shared", "surface": "knowledge.compact", "region": "tools", "scene": "conversation"},
    {"instance": "wiki.shared", "surface": "knowledge.full", "region": "body", "scene": "knowledge"},
    {"instance": "wiki.shared", "contribution": "knowledge.commands", "target": "edit-menu"}
  ]
}
```

两种 presenter 可以共享同一知识会话但不同 view 状态；scene 单激活政策决定当前可见的一份。此例不是把同一个 View 同时挂两次，也不是承诺任意平台都能搬移原生 widget。presentation adapter 是否可迁移是显式能力。

Scene 中的 App 本地 UI 通过同样的 surface 合同装配，原有 widget/Blueprint 不被强制打包成 Feature 才能放入 Region。

## 7. 三种“全屏”行为必须区分

| 操作 | 改变什么 | 返回与状态 |
|---|---|---|
| Select section | 导航到产品栏目及其 page/scene | 保存原栏目布局、历史与会话；按 App 导航 policy 返回 |
| Maximize surface | 临时扩大当前 Surface，在当前栏目内占主内容区 | restore 恢复原 Region，栏目不变 |
| OS fullscreen | OS 顶级窗的显示模式 | 平台窗口合同，和上述两者正交 |

用户重新绑定位置、临时 maximize、自动小屏 fallback 应分别存储：显式用户选择形成 durable preference；临时展开与响应式位置不覆盖桌面原布局，恢复尺寸后可还原。

Feature 发出 placement/attention 意图，Host 作决定；不直接改其他 Region、scene 或全局 z-order。

## 8. 迁移、可见性与模型寿命

SurfaceHandle 对应可挂载 view，Document/WorkspaceSession 对应领域模型。两者独立；切栏目、收起右栏、移到下栏不等于关闭文档或卸载包。

合法迁移过程：校验新目标/能力 → 捕获 view state（光标/滚动/选区）→ detach presenter → attach/recreate presenter → 恢复 state/focus → 发布新 binding。底层组件若需销毁重建，其 buffer/undo 不得随 view 一起销毁；不支持保存 view state 的组件要显式声明限制。

多处同时显示同一文档必须由包声明 multi-view；创建独立 view_instance_id，共享 document_session。不能复制同一个 DOM/native widget 节点，也不能用相同 widget key 隐式实现多视图。

hidden、suspended、unmounted 与 disposed 分别定义：

- hidden：不显示，可以保留 presenter；必须停止不必要的帧/轮询。
- suspended：模型保留，重型 presenter/任务按合同休眠；恢复有状态水位。
- unmounted：释放 View owner 资源，会话仍存续。
- disposed：释放实例；dirty/在途保存等需经关闭合同。

业务 job 不因栏目切换自动取消。watch/timer/RPC 订阅由 owner 收口；文档更新/任务进度可持续，UI 渲染只对可见绑定调度。和现有嵌套挂载/TimeSource、帧预算机制衔接。

## 9. Overlay、通知、输入与可访问性

顶中悬浮框可以是 command palette、quick-open、局部搜索等，但每种描述是否 modal、anchor、owner、dismiss policy 和 return_focus。Host 统一 overlay layer 和 z-order；不让每个包各放一个任意全局浮层。

Modal trap focus、阻止底层点击；non-modal 不无条件抢焦点；IME composition 时 Escape 不提前关闭 App 搜索；dismiss 后恢复有效 owner 或合法替代焦点。嵌套 Escape 先给内部输入/弹层处理，再交当前 overlay/scene policy，不能总是直接跳回上一栏目。

菜单打开前捕获命令上下文，执行时仍校验目标/revision。Region relocation 后 scoped command context 同步更新，焦点不自动落到无关文档。隐藏 view 从键盘导航/命中树移除，tab order 与布局语义一致。

通知通过结构化事件汇入 App notification target：来源、owner、级别、去重 ID、可执行命令目标；toast、横幅、通知中心或整页历史由宿主选择。普通通知不夺焦点，紧急 modal 需要显式 policy。OS 级通知走 System Service/Shell，不能把 app-local 通知栏当 OS 通知中心。

## 10. Responsive、持久化与包更新

约束冲突时按 App 声明的优先级折叠/切为 drawer/page，不压缩主区直到不可用；未声明合法 fallback 的必需 view 给明确 unavailable 状态。小屏/触控条件以可用尺寸、交互能力表达，不根据 App 名称硬编码。

Layout snapshot 按 app/window/layout-profile 或用户定义的 scope 保存，包含 layout schema version、稳定 Region/Section/Surface identity、尺寸、排列、当前栏目和合法 view-state。文档正文、凭据、权限授予不存布局；知识库/文档恢复由 session restore 协议提供安全资源引用。

恢复时过滤已移除包/未知 ID，保留可恢复绑定并显示诊断；新包首次使用 App 默认。显示器/DPI/尺寸变化重新约束求解。migration 由 App/包提供 identity 映射，不用保存像素坐标当永久真相。动态移除当前栏目先选择合法替代、处理 dirty/订阅，再提交，拒绝时原栏目仍有效。

## 11. 实施顺序与验收调整

此方向成为 Design 38 阶段 A 的前置重点：**先设计并验证通用 App Presentation Host，再进行 Feature Pack 的完整抽取**。可在现有仓内框架缝做 pilot，与 Design 37 外部化路线衔接，不先建设产品专属 DockManager。

建议拆成适中 Plan，编号后续中央取号：

1. Region/Section/Surface identity 与类型化贡献合同；布局/命令状态实例作用域探针。
2. 第一版 Host：single/split/tabs、scene 栏目切换、尺寸/折叠、局部 UI 装配。
3. Overlay/菜单/状态/通知的专用目标、focus/context、有效贡献模型双端消费。
4. Surface placement/restore、view/model 分离、响应式与持久化。
5. editing/diff、知识功能包自消费和第二 App 消费；固定包先装配，动态已有 Surface 随后激活。
6. 动态新包安装/卸载、拖拽 docking、独立窗 detach 按真实需求追加；不混入前述首版验收。

首版先覆盖 editor、navigation-pages、reader/单内容三种明显不同产品布局，至少其中两种有双端行为证据；避免仅做 Musk/编辑器工作台一种而声称通用。

| 场景 | 判据 |
|---|---|
| 空壳与简单 App | 一个 content Region 可正常运行；不强制导航/文件树/workspace |
| 同包不同位置 | 知识 Surface 放右侧、下侧、整页，均正确；不兼容 presentation 显式拒绝 |
| 栏目切换 | 对话/知识/下载切换主区或 scene；恢复原草稿/光标/布局，不重建领域会话 |
| 临时最大化 | restore 回原 Region，section history 不改变；OS fullscreen 独立 |
| 多实例 | 两 editing instance 同时存在，保存命令目标正确；多 view 仅显式声明时共模型 |
| 异步与 hidden | 切栏目时 job/保存正确延续；隐藏 view 无多余帧/订阅泄漏，晚到事件不串 scope |
| Overlay/通知 | modal/non-modal、Escape、IME、焦点返回、命令 capture 在 Vue/VM 一致 |
| 动态装配失败 | single 冲突、错类型、missing required target 拒绝；原布局/贡献仍可用 |
| 尺寸变化 | 小屏按声明 fallback；再放大恢复 desktop user choice，不写坏 durable preference |
| 恢复/升级 | 包删除、ID 改名、layout schema/DPI 变化有解释；不恢复越权文档 |

当前仅 Category A 文档设计，不跑 cargo t/docs_gen；后续 Rust/UI codegen 修改按 t＋tu 等触面门禁，真实交互走 autoui-verifier 标准双端工具。此文不声称所有功能已实现。
