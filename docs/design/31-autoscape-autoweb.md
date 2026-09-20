# 31 — AutoScape / AutoWeb：Auto 原生浏览器与统一应用寻址运行时

> **状态**：📝 Draft（2026-09-20 立档）
> **来源**：外来设计输入 `AutoScape_Design_v0.1.md`（ChatGPT 产出，2026-09-20，用户裁定融入本设计体系）。
> 本文档为**重排版**：保留原稿全部有效设计信息，按本仓体系重组——追加 §3 术语×本地实现对照、
> §12 落地差距分析、§13 分期路线（v0.6 建议范围）、§14 落地计划分解（施工图）；原稿独立成文的
> wire-level 边界（§16 文档分层）照单保留。
> **定位**：域级章——Auto 生态"原生浏览器 + 统一应用寻址/运行"域的伞形视图。
> **关联**：[desktop-protocol-v1](autoui/desktop-protocol-v1.md)（RenderQueue 五通道 = AppProvider 的
> 本地执行底座）、[desktop-shell-and-launcher](autoui/desktop-shell-and-launcher.md)（R8–R12）、
> [desktop-shell](autoui/desktop-shell.md)、[virtual-desktop](autoui/virtual-desktop.md)、
> [web-ecosystem-strategy](strategy/web-ecosystem-strategy.md)（边界澄清见 §9.1）、
> [16-app-generation](16-app-generation-and-ai-authoring.md)（App 分发战略上位）。

---

## 1. 核心定义与设计原则

**AutoScape** 是 AutoOS 面向本地、网络与未来远程应用的统一浏览器 Shell。它以 **AutoWeb Runtime**
为底座，原生加载 AutoUI / AutoDown（抛弃 HTML/CSS/JS），并通过 `app://`、Intent、AppResolver、
AppProvider 与 AutoFrame，把"网站、应用、嵌入式应用和远程应用"统一到同一套
AppRealm / AppSession 模型中。

> **设计原则**：URI 表示"是什么"；Intent 表示"要做什么"；Provider 表示"在哪里、如何运行"。
> **应用身份不随 VM、EXE、RenderQueue 或远程执行方式变化。**

> **定位裁定（用户 2026-09-20）**：AutoScape 表面是浏览器，内核是 **app 容器虚拟机**——每个
> AppRealm 拥有**目录级文件沙箱 + 沙箱内 sqlite**（§6.1），app 制品（.at 源 + 编译产物）**安装式
> 本地缓存**（不重复下载）。对标 WASI preopen / Android Runtime / JVM+安装器，**不复制**浏览器
> localStorage 式 quota 箱模型。

AutoUI 已可由 Auto VM 直接解释，也可经 a2r 将前端转译为 Vue、后端转译为 Rust 服务；AutoDown 又能
flip 为与 AutoUI 同构的 Auto 代码。因此 AutoUI/AutoDown 已具备类似 HTML+CSS+JS 的
"可网络分发 UI 描述"属性——这是"Auto 原生 Web"成立的物质基础。

本域解决的三个现状痛点（均在本仓有实证）：

| 现状 | 问题 | 目标替代 |
|---|---|---|
| UI Gallery 把 40+ Demo 前端**源码合并进同一个 VM**（`AppViewport.vm.at` 生成产物），另起统一后端 Proxy | Host 与 Demo 强耦合；构建慢（PLAN-662 冷启 22s 即合并形态代价）；后端路径重写；新增 Demo 要重新编译 | AutoFrame 动态加载独立 App；每个 App 保留自己的 AppRealm 与 ServiceBinding |
| Launcher / File Manager 用"扩展名/目录 → 默认 EXE"式注册（R10 pac.at 扫描 + apps.manifest） | 应用身份与可执行文件/目录绑定；动作语义弱；难统一 Web/Remote App | URI + Intent + AppResolver + Handler Capability |
| 远程桌面若走像素传输 | 带宽/延迟高；丢失 UI 语义 | RemoteProvider 传 VTree/DrawList 操作流，本地渲染（§9.2 已有微型先例）；视频/legacy surface 回退像素流 |

**目标**：AutoUI/AutoDown 像 Web 资源一样经 URL 动态分发加载；`app://auto.image-viewer/` 是稳定逻辑
身份而非某 EXE/源码别名；VM 解释与 Native EXE+RenderQueue 经统一 AppSession 对上层透明；AutoFrame
是真正独立的 Embedded BrowsingContext；前端经 logical service 调后端；AutoScape/Launcher/File
Manager 共用 Universal Resolver；为远程 AutoOS 的 semantic session 保留扩展点。

**非目标**：第一阶段不重造 HTML/CSS/JS 引擎（传统 Web 交系统 WebView，§9）；不发明替代 HTTPS 的传输
协议（AutoWeb 建立在标准 URL/TLS/HTTP 缓存/CDN 之上）；不把 vm/exe/RenderQueue 等执行细节编码进
URI scheme；不再通过 Host 合并子 App 源码/合并后端实现嵌入。

## 2. 命名体系

产品名与技术名分层：AutoScape 是用户看到的浏览器产品；AutoWeb 是网络应用体系；browse 是动作/API。

| 名称 | 角色 | 说明 |
|---|---|---|
| AutoScape | 浏览器产品 | AutoOS 的原生 AutoWeb Browser Shell（薄壳，§8） |
| AutoWeb | 网络生态/平台 | AutoUI/AutoDown 的分布式网络应用体系 |
| AutoWeb Runtime | 共享运行时 | URI、Origin/Realm、Sandbox、Storage、Network、ServiceBinding、BrowsingContext |
| AutoFrame | 嵌入式运行容器 | 类似 iframe，但承载独立 AppRealm/AppSession（§7） |
| AppResolver | 解析与路由 | 把 Intent/URI 解析到具体 Handler、AppProvider 与 Session |
| AppRealm | 安全与身份域 | 应用身份、权限、存储、服务绑定等隔离边界 |
| AppSession | 运行实例 | 一次实际运行的 App；对 VM/EXE/Remote 统一 |
| AppProvider | 实现提供者 | 决定应用在哪里、如何运行 |
| Intent | 动作请求 | launch / view / edit / share / navigate 等 |
| ResourceRef | 资源引用 | URI + media type + 可选 capability |
| ServiceBinding | 逻辑服务绑定 | logical service → physical backend endpoint |
| Dev Registry | 开发注册表 | 开发期发现、启动、热更新本地 App |

## 3. 术语 × 本地实现对照（融入版新增）

原稿按绿地设计书写；本仓实际已建成其中大半底层件。下表是每个抽象的本地锚点与缺口，
**差距分析详情见 §10**：

| 抽象 | 本地现状锚点 | 缺口 |
|---|---|---|
| AppProvider (VM) | 解释态 inproc 直挂 `build_dynamic_component`（459 多会话运行时）；`auto run -r vm -q` 经 RqProjector（协议 v1.14 单投影器统一） | 无（已是双形态） |
| AppProvider (Native) | a2r 编译 exe 一等客户端（v1.6 `desktop_exe:` 孵化）；rqhost rendezvous 采纳（v1.12） | 无 |
| RenderQueue IPC | **Desktop Protocol v1** 五通道（§1 版本史至 v1.15：命名管道+双槽 shm+位图段、L1/L2/L3 窗口迁移、WS transport） | 无——正是文档设想的 RenderQueue 通道 |
| AppSession | 459 AppId 分配/panic 隔离 + 462 VirtualWindow 会话 + 480 broker 多 App 驻留 | 缺"统一抽象层"（VM/EXE/Remote 三形同律的 Session 对象） |
| BrowsingContext (top-level) | rqhost 原生窗 / 462 虚拟窗 / vue 挂载点，三种宿主 | 缺统一 history/navigate 语义 |
| AutoFrame (embedded) | PLAN-663 视口边界=iframe 语义（execution_done）；**但嵌入的仍是父 VM 内子组件** | **主缺口**：独立 AppSession 的嵌入容器（§7） |
| AppResolver | R10 pac.at 注册表扫描（`app_registry.rs`）+ auto-os `apps.manifest`；Launcher=028（R8 特权 shell App） | 缺 scheme/URI 解析、Intent 动词、Handler Capability 匹配 |
| App Manifest | pac.at 已有 name/icon/category/render/desktop_exe/desktop_render 字段 | 缺 capabilities.handle 声明、providers 双声明、permissions |
| ServiceBinding | pac.at `front_port` + gallery 统一 proxy（路径重写） | 缺 logical service 层与 per-app 绑定 |
| Dev Registry | 运行时 pac 扫描（VM 端）+ vue 构建期 registry | 缺懒启动 backend、热更新通道、闲置回收 |
| RemoteProvider / 语义流 | v1.4 WS transport + `remote.rs` 镜像会话 + `drawlist-renderer`（TS/Canvas2D 浏览器渲染，PLAN-508） | 已是"语义而非像素"的微型先例；缺 VTree snapshot/patch 流与桌面化 |
| 安全/沙箱 | 无对应物（inproc 信任模型） | 整层缺失（§6），网络面启用前必须建成 |

## 4. URI、Intent 与 AppResolver

### 4.1 Scheme 分工

| Scheme | 语义 | 示例 |
|---|---|---|
| `https://` | Internet 网络资源 / AutoWeb Deployment | `https://demo.example/003-converter/` |
| `app://` | 稳定逻辑应用身份 | `app://auto.image-viewer/` |
| `auto://` | AutoOS 系统资源 | `auto://settings/display` |
| `file://` | 本地文件资源 | `file:///E:/Photos/a.jpg` |
| `resource://` | Capability-backed 临时资源 | `resource://session/8dd7…` |

官方 namespace 保留 `auto.*`（`app://auto.image-viewer/`、`app://auto.settings/`）；第三方可用
`vendor.app` 形式，唯一性最终结合签名发布者身份解决。`app://auto.settings/display` 中
`auto.settings` 是 App Identity，`/display` 是 App Route。

### 4.2 URI 是名词，Intent 是动词

```
Intent { action: view,  target: ResourceRef("file:///E:/Photos/a.png") }
Intent { action: launch, target: ResourceRef("app://auto.calculator/") }
Intent { action: edit,   target: ResourceRef("file:///E:/project/main.auto") }
```

不要把动作塞进 URI 查询参数。传统"默认程序"退化为 Resolver 的用户偏好数据库，而非系统核心启动
模型（对应本仓 R10/R11 的注册表语义升级方向）。

### 4.3 Handler Capability

```
capabilities {
  handle { action: view; media: "image/*"; schemes: ["file", "https", "resource"] }
}
```

"图片查看器"不再注册 .png/.jpg/.webp 扩展名，而是声明能 `view image/*`。Resolver 按
action、media type、scheme、安全策略与用户偏好匹配 Handler。

### 4.4 App Manifest 与四层身份

```
app {
  id: "auto.image-viewer"
  name: "图片查看器"
  capabilities { handle { action: view; media: "image/*" } }
  providers {
    vm { entry: "./main.auto" }
    native.windows { executable: "./image-viewer.exe"; presentation: render-queue }
  }
  permissions { filesystem: capability-only; network: none }
}
```

| 对象 | 职责 | 生命周期 |
|---|---|---|
| App Identity | `app://auto.image-viewer/` 稳定逻辑身份 | 长期稳定；不随实现变化 |
| AppRealm | 权限、存储、Origin/信任域、服务绑定、资源能力 | 随应用安全域/用户 profile 存在 |
| AppSession | 一次运行实例；统一 VM/Native/Remote | 从 launch 到 suspend/close |
| BrowsingContext | 可导航 UI 容器；Tab 或 AutoFrame | 可绑定/重绑定 Session 与 Route |

Manifest 是稳定元数据；网络部署另需 Deployment Descriptor（声明该次部署的前端 artifact、service
bindings、版本与缓存信息），其 wire format 归《AutoWeb Service & Protocol Specification》（§12）。

## 5. AppProvider：执行方式与身份解耦

VM 与 EXE 不进 URI。`app://auto.image-viewer/` 永远表示同一应用，Resolver 按环境选 Provider：

```
app://auto.image-viewer/ → AppResolver → ┬ AutoVMProvider  → main.auto        → direct VTree
                                          └ NativeProvider → image-viewer.exe → RenderQueue IPC
                                                        └────► AppSession
```

| Provider | 初期职责 | 典型场景 | 本地锚点 |
|---|---|---|---|
| AutoVMProvider | VM 直接解释运行 Auto artifact，直接生成 AutoUI VTree | 开发、热更新、跨平台 fallback、AutoWeb sandbox | inproc 直挂 / `-q` RqProjector |
| NativeProvider | 启动 EXE，经 RenderQueue 与 AutoOS 做 UI/窗口通信 | Release 性能、原生可执行 | v1.6 孵化 + v1.12 rqhost |
| RemoteProvider（未来） | 远端执行状态/逻辑，本地语义渲染 | 远程 AutoOS / Remote App | v1.4 WS 镜像会话（雏形） |

开发调试强制 VM 用 execution hint 表达（`auto run app://auto.image-viewer --runtime=vm`），
**不引入** `vm://`、`exe://`（避免伪造执行 scheme 绕过安全策略）。本地对应：现有
`--autodesk-render=` / `desktop_render:` 三态裁决链（auto|queue|independent）正是同型的
"提示不进身份"实现。

## 6. 安全与隔离模型（网络面硬前置）

AutoWeb 代码是网络输入，**不能因使用 Auto 语言就天然获得本地 App 权限**。capability-first、
deny-by-default：

- 网络 AutoVM 默认不能访问任意文件系统、进程、Shell、窗口管理器或 native API；
- AppRealm 绑定自身 Origin/Identity、Storage、Permissions、ServiceBindings 和 Cache；
- AutoFrame 跨 Realm 只允许显式 message channel，不能直接跨 VM 读写状态；
- `file://` 只表达资源身份不自动授权；sandbox App 优先获得 `resource://` capability handle：

```
resource://session/8dd7...
├── target: file:///E:/Photos/private.jpg
├── permission: read
├── granted_to: auto.image-viewer
└── lifetime: session
```

- NativeProvider 的 EXE 按安装来源、签名、声明权限与用户授权分信任等级。

### 6.1 存储沙箱裁定（用户 2026-09-20）

**裁定**：不做浏览器式抠搜存储（localStorage quota 箱），改为**目录即沙箱**：

1. **文件夹沙箱**：每个 AppRealm 一个沙箱目录，app 可像本地文件一样任意建立/读写，但**任何
   文件操作不得越出沙箱**（路径包含：canonicalize 后前缀校验，VM 文件面单点实现——
   `vm/io.rs` + `native.rs` fs.* 模块即咽喉点）。
2. **沙箱 sqlite**：每 Realm 附一个 sqlite 数据库文件（位于沙箱目录内），路径包含自动覆盖，
   无需独立数据库权限系统。`db.*` 内建命名空间，`db.open("app.db")` → `<sandbox>/app.db`。
   （补真实缺口：examples 现有 "db" 均为内存种子数据，`d013todo_db.at` 实证，持久化一直缺位。）
3. **存储分区键控**："区分 local 和 host"——`app://auto.x`（本地应用）与 `https://a.com`
   （网络主机）各得一份独立沙箱，per-identity 持久。
4. **安装式制品缓存**：app 本体（.at 源码 + 编译产物）本地落盘，**二次访问零下载零编译**——
   三层映射：.at 源（可读层）→ ABC 字节码（内容 hash 键控）→ a2r 编译 exe（native 提供者形
   态）。基建先例：AutoCache（Design 09）+ PLAN-662 制品缓存（5s 二次启动即其功劳；NTFS
   mtime 延迟落定的坑已在册）。

**随裁定登记的实现要点（S1–S6 已全部裁定/确认，用户 2026-09-20；落地计划须覆盖）**：

| # | 要点 | 说明 |
|---|---|---|
| S1 | 粒度：**per-app（终裁）** | 沙箱根按 **app 身份**键控（非 per-tab）——同 app 的所有 tab/会话共享同一沙箱；"区分 local 和 host" = `app://` 本地应用与 `https://` 远程主机各自独立沙箱。私有 tab（一次性根）降为浏览器壳层可选 UX 特性，不进键控模型。**并发注记**：同 app 多会话并发写同库/同目录是 per-app 的固有形态——db 层 WAL + busy_timeout 兜底，普通文件写同 localStorage 串行化口径，冲突消解归 app 逻辑 |
| S2 | 链接与共享缓存：**限定区 + 目标白名单（终裁）** | 软链接不做一刀切禁令——它是 cache **单例化**的主要工具（否则重蹈 npm 每 app 复制一份巨大 node_modules）。模型三律：①**限定目录**——沙箱内仅固定目录（如 `<sandbox>/deps/`）允许链接存在，他处创建 symlink/junction 一律拒收；②**目标白名单**——链接目标 canonicalize 后必须落在**托管共享缓存根**内（AutoUI 中间产物：依赖库/组件包/媒体/字体）；③**写权限单点**——共享缓存对 app 运行时 API **只读**，写入与建链仅经可信安装·依赖解析层（内容寻址、写后不可变）；app 代码创建链接恒拒收。**实现二选一**：A=真链接（Windows junction；pnpm 同款模型先例，仓内 pnpm junction 运维经验在册——工具链须 junction-aware）；B=虚拟挂载（文件 API 将 `deps/` 解析为只读虚拟根 `deps://` → 缓存条目，沙箱内零真实 NTFS 链接，包含由构造保证；native 提供者 spawn 需真文件处仍由可信层落真链）——**推荐 B 先行**。逃逸纪律不变：canonicalize 后前缀比对（含大小写归一）是一切路径检查的底座 |
| S3 | 文件夹沙箱 ≠ API 门禁（**确认**） | 路径包含只管文件系统；desktop bus / native catalog / shell_bridge / 任意网络访问仍需粗粒度关闸（网络来源关特权内建）。**两道门正交，缺一不可**；发行红线不变：关闸与 https 加载同船 |
| S4 | 网络访问策略 v1（**确认**） | 仅同 origin + ServiceBinding 显式声明的外部 endpoint 白名单（与 §9.3 logical service 天然衔接） |
| S5 | 配额与管理面（**确认**） | 沙箱要有配额上限（防填盘）+ 管理 UI（"该站占 X MB / 清除"）——归 Permission UI/Downloads 面，v0.6 最小做列出+删除 |
| S6 | 更新语义（**确认**） | 安装式缓存引出版本比对：v0.6 最小 = 启动时 hash/ETag 比对、有新版提示换装（§16 Cache/Update 节的首次消费方） |

**战略含义**：存储层跟上限位后，整个体系自洽——AppRealm/AppSession 本就是 app 语义而非文档
语义，AutoScape 不是"能跑 app 的浏览器"，是"**长得像浏览器的 app 运行时**"；"网站"经一次访问
即成为"免安装的本地 app"。

**分期注记（融入版）**：本地 dev-only 阶段（app:// + file:// + Dev Registry）可先以"inproc 信任
模型 + 无跨进程隔离"运行（与现状 459 panic 隔离同级）；**https AutoWeb 加载解锁前本层必须建成**
——这是网络面的长杆，见 §11 分期。

## 7. AutoFrame 与 BrowsingContext（MVP 最重要 primitive）

AutoFrame 看似 AutoUI 组件，底层必须创建**独立 BrowsingContext / AppRealm / AppSession**，
而不是把子 App AST 合并进父 VM（现状 gallery 的 `use.web component` 合并形态正是要淘汰的）：

```
Gallery AppSession
└── AutoFrame { src: "app://demo.003-converter/", viewport: { width: 1024, height: 720 } }
    └── Embedded BrowsingContext
        └── Demo AppRealm → Demo AppSession（independent VM / service）
frame.post({ type: "reset" }); frame.on_message { ... }
```

需求清单：独立 viewport、DPI/scale/theme、导航、reload、错误页、父子 message channel、权限边界、
资源/网络隔离、detach 为顶层 Tab / AutoOS Window（L1 换窗先例：v1.2 `detach_surface_to_os_window`）。

**本地工程注记**：最接近的既有机制是 462 VirtualWindow——它已是"独立 App 会话的嵌入表面"（一窗一
App、WM 持位、命中分区）。AutoFrame 可建模为**固定 rect 的轻量嵌入容器**（复用 VirtualWindow 的
会话挂载/事件分区机制 + PLAN-663 视口边界的 iframe 尺寸语义）。"在独立页面打开"第一阶段 = 新 Tab
重新 load 同一 URI；后续支持 Embedded BrowsingContext detach 保留输入/滚动/临时状态（挂
L2/L3 迁移语义）。

## 8. AutoScape 浏览器 Shell（薄壳）

AutoScape 本身是**普通 AutoUI 应用**（薄 Shell）；核心 App loading 能力来自 AutoWeb Runtime
（R8 "shell 组件 = 特权 AutoUI App" 的同型延伸）：

| 组件 | 职责 |
|---|---|
| Omnibar | 接受 `https://`/`app://`/`auto://`/`file://`、搜索词、App 名与命令；统一进 Resolver |
| Tab Strip | Top-Level BrowsingContext 容器 |
| Navigation | Back/Forward/Reload/Stop；与 BrowsingContext history 绑定 |
| Permission UI | AppRealm 权限、资源授权、跨 Realm 请求 |
| Downloads / Resources | 网络下载与 capability resource 管理 |
| DevTools | VTree、Realm、ServiceBinding、网络、VM、RenderQueue、消息通道与性能分析 |
| Legacy Web Handler | `text/html` 等交系统 WebView/Chromium/WebKit |

Omnibar 与 AutoOS Launcher 最终共享同一 Universal Resolver：Launcher 偏应用/命令，AutoScape 偏
导航/历史，底层 URI/Intent 语义一致。

## 9. AutoWeb 统一访问模型与兼容边界

### 9.1 与 web-ecosystem-strategy 的边界澄清（融入版裁定）

[web-ecosystem-strategy](strategy/web-ecosystem-strategy.md) 裁定⑤"自研浏览器运行时不做（VM→WASM
之类）"针对的是**在 HTML 浏览器里跑 VM 的发射臂**（失去宿主框架生态免费午餐）；AutoScape 是反方向
——**原生桌面客户端直接消费 AutoUI/AutoDown**，不实现 HTML 引擎。两者互补不冲突：

```
                 Auto 源码 (.at)
                ┌──────┴──────┐
        AutoScape 原生客户端          a2r 发射（Vue/未来 React）
        （AutoUI 第一等 Native）      （Chrome/Safari/Edge 照常访问）
```

契约独立原则（527 清单）在两臂同时成立：同一份 AutoUI 契约，AutoScape 臂原生解释，HTML 臂 a2r
转译。AutoScape 让 Auto 生态拥有"无需宿主框架"的直达入口，a2r 保证传统 Web 兼容——这正是
"动静结合第一优势"在客户端侧的体现。

### 9.2 一个 URL 首先定位 Application Deployment

`https://demo.example/003-converter/` 不应理解为"某前端文件路径"，而解析为一个 Application
Deployment：AutoWeb Runtime 取 Deployment Descriptor → frontend artifact、assets、logical
services、版本、权限要求。

### 9.3 前端只依赖 logical service

```
services {
  default { endpoint: "./api/" }
  account { endpoint: "https://account.example/" }
}
// Frontend
service.default.call("convert", value)
service.account.call("profile")
```

应用代码依赖 logical service 名而非 `localhost:29481`、`/gallery/003-converter/api` 等物理细节
（gallery 统一 proxy 路径重写正是此痛）。ServiceRouter 在 Realm 建立时完成 logical→physical
binding；dev/prod/远程同一份前端代码。

### 9.4 Typed Service（推荐高级接口）

```
service Converter { fn convert_temperature(value: Float, from: Unit, to: Unit) -> Float }
let result = Converter.convert_temperature(20, Celsius, Fahrenheit)
```

Typed RPC 由编译器自动生成序列化、版本校验、错误模型与 a2r 的 TS/Rust stub；低层 HTTP/fetch 仍在
但不应成为 Auto App 调自身后端的主路径。

### 9.5 兼容传统 HTML Web

按 Content-Type / handler 路由：Auto 内容进 AutoWeb Runtime，`text/html` 进系统 WebView。a2r 提供
反向兼容（同一 Auto 源生成传统 Vue/HTML 形态）。第一阶段不自研 HTML engine。

## 10. 远程 AutoOS 演进方向（语义远程桌面）

核心思想不是远程 framebuffer，而是"**远程状态/逻辑 + 本地 AutoUI 渲染**"：

```
Remote AutoOS / App → RemoteProvider → VTree snapshot + patch 流 / resources + events + RPC
                                    → Local AppSession / BrowsingContext → Local AutoUI Renderer
```

视频、游戏、CAD viewport、legacy Windows App 走 hybrid surface（AutoUI 部分语义流，视频/legacy 走
AV1/H.265 或像素流）。远程 App 与本地 App 保持同一 `app://` 身份，上层只见不同 Provider。

**本地先例锚定**：v1.4 WS transport 已把 composed DrawList（Commands 语义流，非像素）推给浏览器
Canvas2D 渲染并完成点击闭环（PLAN-508 Playwright e2e）——语义流的可行性已被 miniature 验证；
RemoteProvider = 该链路的桌面化 + VTree/patch 增量 + hybrid surface 分账。这是"基于浏览器的远程
桌面"（语义传输而非像素传输）愿景的技术路线。

## 11. 生命周期、Dev Registry 与导航

```
open(Intent) → Normalize URI/ResourceRef → AppResolver/ContentHandler
  ├─ existing AppSession? ── yes ──► activate + navigate(route)
  └─ no → choose AppProvider → create AppRealm/AppSession
          → bind services + permissions + storage → create BrowsingContext → render
```

调用方不应知道底层是 VM 的 navigate() 还是 EXE 的 RenderQueue/IPC 导航消息。

**Dev Registry**（本地开发专用动态发现与启动层）：per-app 记录 project path、frontend artifact、
backend status/process、hot reload channel；`resolve(app://demo.003-converter/)` → 按需启动
backend → bind services → load frontend → create AppSession。本地开发不再把前端都编入 Gallery，
也不要求所有 backend 预先启动；Demo 切换懒启动、闲置 suspend/回收。开发环境与生产环境使用相同
App/Service 语义，仅替换 Resolver/Deployment backend。

## 12. 落地差距分析（融入版新增）

按"已有/缺口/规模"三档盘点（规模为粗估：小=<1 周，中=1–3 周，大=>3 周，单人全职口径）：

| # | 能力 | 状态 | 说明 |
|---|---|---|---|
| 1 | VM Provider（解释直挂 / -q） | ✅ 已有 | 459 + v1.14；inproc 边际 ≈0.86 MiB/App（508 实测） |
| 2 | Native Provider（exe + RenderQueue） | ✅ 已有 | v1.6/v1.12；rqhost 多客户端 daemon、位图通道 v1.15 |
| 3 | 多 AppSession 并存宿主 | ✅ 已有 | 462 WM + 480 broker 驻留 + L1/L2/L3 迁移 |
| 4 | 视口 iframe 语义 | ✅ 已有 | PLAN-663 `SizeValue::Screen` 边界重锚定（execution_done） |
| 5 | 嵌入会话时间源 | ✅ 已有 | PLAN-652/654（画廊内嵌 Tick 缺口已闭） |
| 6 | 注册表/清单 | 🟡 部分 | pac.at 扫描在；缺 capabilities/providers/permissions 声明面（小-中） |
| 7 | URI/Intent/Resolver 薄层 | ❌ 缺 | scheme 解析 + Intent 动词 + route + 偏好库（中） |
| 8 | **AutoFrame 独立会话嵌入** | ❌ 缺 | 主缺口：父 app 视口内独立 VM 会话 + 生命周期/reload/错误页/消息通道（中-大） |
| 9 | Dev Registry + 后端懒启动 | ❌ 缺 | per-demo backend 懒启动 + ServiceBinding + 闲置回收（中） |
| 10 | AutoScape 薄壳（Tab/Omnibar/导航） | ❌ 缺 | 普通 AutoUI app，宿主复用 rqhost/桌面（中） |
| 11 | 统一 AppSession 抽象 | 🟡 部分 | 机制在、抽象层缺：VM/EXE/Remote 同律 Session 对象 + history 语义（中） |
| 12 | AutoWeb https 部署 + Deployment Descriptor | ❌ 缺 | 网络加载 + media type/内容协商（大） |
| 13 | 安全沙箱/AppRealm 权限 | ❌ 缺 | deny-by-default + capability + storage 隔离（**大，网络面硬前置**） |
| 14 | Typed RPC / ServiceRouter | ❌ 缺 | IDL + 版本/错误模型（大，可后置） |
| 15 | Legacy HTML handler | ❌ 缺 | 系统 WebView 桥接（中，可后置） |
| 16 | RemoteProvider 语义流 | 🟡 雏形 | 508 WS DrawList 流已验证；VTree patch/hybrid surface 桌面化（大，远期） |

## 13. 分期路线（融入版重排；v0.6 建议范围 = M1 + M2-lite，计划分解见 §14）

原稿 MVP 八步保留为骨架，按本地差距重排为三里程碑：

**M1 — 本地 AutoScape（v0.6 主打候选，~3 个月单轨可达）**
1. 固定 URI/Intent/AppId/AppResolver 最小数据结构与 `app://auto.*` 命名规则（差距 #7）；
2. AppSession 抽象统一 VM/Native 两 Provider；Launcher 迁移到统一 `launch(Intent)`（第二 dogfood，#11）；
3. **AutoFrame + Embedded BrowsingContext**：独立 VM 会话、viewport、reload、message channel（#8）；
4. **Dev Registry + UI Gallery 重构**：彻底删除 Demo 前端 merge、统一 VM、backend proxy 与 URL
   rewrite——Gallery 只存 Demo 逻辑 App URI（#9，第一 dogfood）；
5. AutoScape 薄壳：Omnibar/Tab/history（本地 app://、file:// 导航即可发布）（#10）。

**M2-lite — 网络最小面（v0.6 建议纳入，2026-09-20 复核；范围待用户终裁）**
6. https 加载：Deployment Descriptor（JSON 清单）+ 制品下载进**安装式缓存**（§6.1 ④）+ 装载；
   服务端 = Auto HTTP Server 标准库自托管 demo 站（最强 dogfood，reqwest 已是依赖）；
7. 存储沙箱 §6.1 全套（文件夹沙箱 + sqlite + per-app 分区 + 安装缓存 + S2 共享缓存单例化）+
   粗粒度 API 关闸（S3 第二道门）；
8. **发行红线（不可让步）**：关闸与 https 加载**同船发行**——不存在"先能加载、后补安全"；
   网络来源一律无特权内建。

**M2-full — 完整 AutoWeb（v0.7+）**
9. 细粒度 capability 模型与 resource:// 句柄、签名/发布者身份；Permission UI/Downloads/DevTools
   扩展；Typed RPC；内容协商/a2r representation fallback；离线/缓存/更新完整语义。

**M3 — 兼容与远程（v1.0 线）**
10. Legacy HTML handler / WebView 兼容（#15）；
11. RemoteProvider / Semantic Stream 桌面化（#16；技术预览候选 = v1.4 WS DrawList 镜像 + token
    鉴权包成"远程会话"Tab，Playwright 点击闭环已验证，不打产品承诺）。

**M1 验收标准（沿原稿）**：Gallery 能仅通过 app:// / URL 动态发现并加载任意 Demo；每个 Demo 保持
独立 AppSession 和 backend binding；可一键在 AutoScape 新 Tab 打开；Host 不再预先整合 Demo 源码或
后端。**M2-lite 追加验收**：自托管 demo 站经 https:// 一次访问即安装缓存，二次访问零下载零编译、
数据落在该站沙箱且重启可见；网络来源 app 调特权内建被关闸拒收留痕。
**明确不做**（v0.6）：细粒度 capability/签名、Typed RPC、内容协商 fallback、legacy HTML、
RemoteProvider 产品化（技术预览除外）。

## 14. 落地计划分解（v0.6 施工图——下次启动照此拆 plan，无需回翻会话）

> **本节性质**（用户 2026-09-20 裁定：先行记录、暂不立项）：v0.6 = M1 + M2-lite 共 **七个计划
> P-A..P-G** 的依赖图、各自步骤、本地锚点与验收要点。下次启动时逐个走 `/auto-plan:new`，
> 以本节为 needs-analysis 种子。**动工前锚点按当时 master 复核**——本图基于 2026-09-20
> master（b4e2dddee）实勘。

### 14.0 依赖图与调度

```
P-A URI/Intent/Resolver（地基，先行）
 ├──→ P-B AppSession/Provider 统一 ──→ P-C AutoFrame 嵌入 ──→ P-D Gallery 重构（跨仓）
 ├──→ P-E 存储沙箱+安装缓存 ──→ P-F https 加载+关闸+自托管站（M2-lite 收口）
 └──→ P-G AutoScape 薄壳（依赖 P-B；可与 P-D 并行）
```

关键路径两条：**A→B→C→D**（gallery dogfood）与 **A→E→F**（网络面）；P-B 与 P-E 可并行，
P-G 与 P-D 可并行。体量粗估（单人全职周级）：A≈1–2 / B≈2–3 / C≈3–5 / D≈2–4（跨仓）/
E≈2–4 / F≈2–3 / G≈2–3；合计 ~14–24 周，双轨并行三个月窗口成立（与 §13 结论一致）。

### 14.1 P-A：URI / Intent / AppResolver 薄层

- **范围**：四 scheme（app/auto/file/resource）解析与 normalize；Route 拆分（identity + route）；
  Intent{action, ResourceRef} 数据结构；Resolver 匹配链（action/media/scheme → handler + 用户
  偏好最小库）；pac.at 扩展字段（capabilities.handle / providers{vm,native} / permissions，
  缺省=现状行为零变化）；execution hint CLI（`auto run app://... --runtime=vm`，不进 URI）。
- **步骤**：①URI/Intent 解析器（纯 Rust + golden 单测）②pac.rs 扩展解析 ③Resolver 匹配链
  ④CLI 入口 ⑤Launcher/desktop_registry 迁移 launch(Intent)（第二 dogfood）。
- **锚点**：`crates/auto-man/src/pac.rs`（R10 字段）、`crates/auto-lang/src/ui/app_registry.rs`、
  auto-os `apps.manifest`、`apps/028-launcher`。
- **验收**：scheme 矩阵 golden；launcher e2e 启动 examples/ui 代表集；pac 向后兼容（旧 pac 全量通过）。

### 14.2 P-B：AppSession 抽象统一 + Provider 接缝

- **范围**：VM/EXE 两 Provider 同律 Session 对象（launch/activate/navigate/close/suspend）；
  BrowsingContext history 最小语义（navigate/back/reload）；不做 RemoteProvider（M3）。
- **锚点**：459 多会话运行时（`build_dynamic_component`/allocate_app/panic 隔离）、480 broker
  孵化链、rqhost（协议 v1.12）。
- **验收**：同一段调用代码驱动 VM 与 EXE 两形态无分支差异；session 生命周期状态机单测；
  `auto run` 零回归（I2 纪律延续）。

### 14.3 P-C：AutoFrame = 嵌入式 BrowsingContext（主缺口）

- **范围**：父 app 视口内挂**独立 AppSession**（独立 VM）；viewport/reload/错误页/父子 message
  channel；detach 阶段一 = 新 Tab 重 load 同 URI。**实现裁定待计划内定**：VirtualWindow 特化
  vs 轻量 embed 容器（§15 待决第一新增问，本计划 T-01 裁定）。
- **锚点**：462 VirtualWindow/WmState（嵌入表面先例）、PLAN-663 视口边界 pre-pass（iframe 尺寸
  语义已落）、PLAN-652/654 嵌入会话时间源（Tick 缺口已闭）、459 panic 隔离。
- **验收**：嵌入 demo 代表集 e2e（自由尺寸 003 / 表单 005 / 全屏 020/027——663 塌缩回归集即用例）；
  子会话 panic 父壳不倒；父子消息往返；reload 状态重置。

### 14.4 P-D：Dev Registry + UI Gallery 重构（第一 dogfood，跨仓）

- **范围**：gallery 只存 demo 逻辑 App URI；Dev Registry（per-demo 发现、懒启动 backend、
  ServiceBinding 绑定、闲置回收）；**渐进替换**合并臂（先并存对照、后删除）。
- **仓位**：主体 auto-os（ui-gallery）+ 机制侧 auto-lang——跨仓计划，`.wt/<组>/{auto-lang,
  auto-os}` 组内双 worktree（Plan 529 布局）。
- **关键坑（必读）**：`AppViewport.vm.at` 头注"PLAN-625 T-10 生成产物(勿手改)——重新生成
  auto build / auto run(generate_gallery_host)"——**改生成器，不手改文件**；662 启动优化依赖
  制品缓存（NTFS mtime 坑在册）——重构后启动数据行必须对照 662 基线（二次 5s/冷启 22s）；
  vue 臂零改动（`AppViewport.vue` 即 web 臂 AutoFrame 对应物，天然对照）。
- **验收**（= §13 M1 验收标准）：41 demo 全量经 URI 动态加载；第 42 个 demo 零重编译入册；
  每 demo 独立 AppSession/backend binding；启动数据行不劣化超阈值。

### 14.5 P-E：存储沙箱 + 安装式缓存（§6.1 全套）

- **范围**：per-app 沙箱目录（路径包含**单点**：`vm/io.rs` + `native.rs` fs.* 咽喉层）；
  db.* 内建（沙箱 sqlite，WAL + busy_timeout）；共享缓存根 + `deps/` **虚拟挂载先行**（S2 方案
  B）；ABC/a2r 制品内容寻址缓存；配额 + 最小管理面（S5）；启动 hash 比对换装（S6）。
  **S1–S6 六要点即验收依据**。
- **锚点**：Design 09 AutoCache、PLAN-662 制品缓存经验、pnpm junction 运维先例（工具链
  junction-aware）。
- **验收**：路径逃逸测试矩阵（junction/symlink/`..`/大小写/UNC——S2 底座纪律）；同 app 双会话
  并发写库；缓存命中二次装载零编译数据行；配额超限行为；关闸不在本计划（归 P-F，S3 双门）。

### 14.6 P-F：https 加载 + 粗粒度关闸 + 自托管 demo 站（M2-lite 收口）

- **范围**：Deployment Descriptor（JSON 清单最小格式，完整版归《AutoWeb Service & Protocol
  Specification》）；制品下载进 P-E 安装缓存 + 装载；**网络来源特权内建关闸**（desktop bus/
  native catalog/shell_bridge 恒拒收 + 观测留痕）；同 origin + ServiceBinding 白名单（S4）；
  Auto HTTP Server 标准库自托管 demo 站（dogfood）。
- **红线（发行纪律，不可让步）**：**关闸与加载同船**——不存在"先能加载、后补安全"。
- **锚点**：reqwest 已是依赖（v1.9 http 图像抓取先例）、HTTP Server 标准库（Design 13）、
  native catalog 注册单点（29xx 撞号带教训：新内建 id 走高段）。
- **验收**（= §13 M2-lite 追加验收）：一次访问安装、二次零下载零编译、数据重启可见、
  特权内建调用拒收留痕；恶意 fixture 集（越权路径/外链 fetch/特权调用）。

### 14.7 P-G：AutoScape 薄壳（产品脸面）

- **范围**：Tabs + Omnibar + back/forward/reload/stop + 最小存储管理页（S5 列出+删除）；
  本体为**普通 AutoUI app**（R8 同型），宿主 = rqhost 或桌面。
- **锚点**：§8 组件表；宿主复用 rqhost（v1.12）/桌面（463）；双端一份源纪律（I5 同型）。
- **验收**：app:// 与 https://（P-F 后）双形态导航；tab 生命周期/崩溃恢复；vue/vm 双端 e2e。

### 14.8 启动 checklist（每计划通用）

- `/auto-plan:new` 以本节对应小节为种子起草 → 用户确认 → `D:/autostack/.wt/lang-<NNN>/`
  worktree（跨仓组目录，P-D 双仓）；**worktree 红线：禁 junction/symlink**。
- 分级测试门禁按 AGENTS §2（改 VM→`cargo tv`；触 aavm 才 `taa`；review 前裸 `cargo tf`）；
  文档/生成物改动勿跑 cargo。
- 执行裁定登记簿置顶；生成物勿手改（生成器侧改）；完成后走 `/auto-plan:review` → merge 范式。
- **明确不在 v0.6 批**：细粒度 capability/签名、Typed RPC、内容协商 fallback、legacy HTML、
  RemoteProvider 产品化（M2-full/M3，见 §13）。

## 15. 待决问题

原稿八问照录 + 融入版新增四问：

- 第三方 namespace 唯一性与签名规则（vendor.app 是否足够，还是 publisher identity 参与解析）；
- AppRealm/AppSession 复用规则（单例/多实例/per-document 由 Manifest 如何声明）；
- 网络 AutoUI artifact 发布格式：源码、AST/bytecode、VTree template，还是多 representation 协商；
- AutoWeb Origin 定义：沿用 scheme+host+port，还是叠加 logical app identity；
- ServiceBinding/Typed RPC 的版本协商、错误模型、authentication context；
- NativeProvider 的 RenderQueue 协议如何表达 route、Intent、window lifecycle 与 surface attachment
  （本地注：v1.11 表面 z 平面声明与 v1.12 采纳协议已是部分答案）；
- AutoFrame detach 时 Session 迁移还是重新绑定 Context；跨进程/远程下如何保持状态
  （本地注：L2/L3 迁移语义可复用）；
- Legacy HTML Tab 与 AutoWeb Tab 的统一 history/downloads/permissions/DevTools 体验。
- **（新增）AutoFrame 与 VirtualWindow 是否同一 primitive**：固定 rect 嵌入容器是否就是虚拟窗的
  特化形态，还是独立轻量容器？裁定影响 462 机制的复用面与 AutoFrame 实现成本。
- **（新增）合并单 VM 形态是否保留为 dev 快速路径**：PLAN-662 的快启动优化建立在合并编译产物缓存
  上；AutoFrame 独立会话后启动体验需复测（编译仅 ~3s/全量的量级，风险可控但须数据行）。
- **（新增）M1 的信任模型口径**：dev-only（inproc、无权限强制）是否作为 v0.6 发布口径明示，避免
  "AutoScape 已安全"的误读。
- **（新增）Launcher 迁移时序**：统一 launch(Intent) 是 M1 内完成还是 M2（影响 028-launcher 与
  desktop_registry 的改造排期）。

## 16. 文档分层（下一份文档建议）

AutoWeb Service / Protocol 应**尽早独立成文**——它已是独立平台层：AutoScape 只是客户端之一，
AutoFrame、AutoOS Launcher、CLI、AutoWiki、未来 RemoteProvider 都直接消费它。推荐命名
**《AutoWeb Service & Protocol Specification》**，至少包含：

| 章节 | 应定义内容 |
|---|---|
| Deployment Descriptor | URL→App Deployment 解析格式；frontend artifact、assets、services、version、runtime requirements |
| HTTP / Content Negotiation | AutoUI/AutoDown media type、Accept、HTML fallback、a2r representation |
| ServiceBinding / Router | logical service 名、物理 endpoint、same-app authentication、dev/prod mapping |
| Typed RPC | IDL、序列化、版本、错误、streaming、取消、超时 |
| Origin / Session / Auth | AppRealm 与网络 Origin 的关系；Cookie/token/session；跨 Realm 调用策略 |
| Cache / Offline / Update | artifact hash、ETag、content-addressed cache、版本切换、离线策略 |
| Dev Registry Protocol | 本地项目发现、懒启动 backend、hot reload、端口隐藏、错误反馈 |
| Security | sandbox、capability、service allowlist、签名、完整性校验 |
| Remote Extension（后续） | semantic session handshake、VTree snapshot/patch、resource stream、hybrid surface |

分层：AutoScape Design（本文）负责"客户端产品与其消费的抽象"；AutoWeb Service & Protocol 负责
"这些抽象如何通过网络和服务端兑现"；App URI/Intent/Resolver 若扩展到整个 AutoOS，可再抽第三份
《AutoOS Application Routing & Intent Specification》（当前保留在本文）。

## 17. 结论

AutoScape 最重要的价值不是"做一个能渲染 AutoUI 的浏览器"，而是把 AutoOS 的本地应用、网络应用、
嵌入式应用与远程应用统一到同一寻址和运行模型：URI 定位身份/资源，Intent 表达动作，AppResolver
决定 Handler，AppProvider 决定执行方式，AppSession 抹平 VM 与 EXE，AutoFrame/Tab 只是
BrowsingContext 的不同宿主。这套模型先在 Launcher 和 UI Gallery 跑通（M1），再向 HTTPS AutoWeb
扩展（M2），AutoScape 无需以"重造浏览器"为起点，自然长成 AutoOS 的网络应用入口。
