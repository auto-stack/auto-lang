# PLAN-815 I-TOOL / auto-ui-tool 交接文档

> T-08 交付物（T-05 R7/T-06..08 提前 R2 修复后版）。对象：816 CLI 客户端（M816-B 消费 B
> 能力）及后续消费者。本版与修复后实现（T-06/T-07 提前 R1 修复轮）逐项对账；
> 送审冻结身份见下表。

## 工具身份与冻结指纹

| 项 | 值 |
|---|---|
| 工具名 | `auto-ui-tool` |
| 源提交（冻结 SHA） | `e351cb2a73a78f353457e56ea20cb82f61c1ac88`（branch `codex/plan-815-dev`，基线 M814-B `93a25c924` 之后） |
| 二进制（debug 构建）SHA256 | `f21db18136403d1124ecbc1338f61c5603544401c434e7d12fad20a60c37dfb5`（`target/debug/auto-ui-tool.exe`，由上述源提交构建） |
| 二进制入口源 | `crates/auto-ui-tools/src/bin/auto_ui_tool.rs`（路径仅为导航；消费身份以上两行为准） |
| 协议 crate | `crates/auto-tool-protocol`（纯数据，无 AST/VM/renderer） |
| 协议版本 | `PROTOCOL_VERSION = 1` |
| 交付闭包 | `DELIVERY_CRATES` 26 项（`rust_ui.rs` 常量） |
| 777 域 | `auto-tool-protocol`→data、`auto-ui-tools`→ui |

### 支持指纹（能力×profile 矩阵，冻结于上述源提交）

| 操作 | served profiles | cancellable | interactive |
|---|---|---|---|
| `describe` | — | ✗ | ✗ |
| `validate_project` | —（I-PROJECT 元数据/路径/约束，不做语言解析/求值） | ✗ | ✗ |
| `generate_ui` | `vue` `rust` `jet` `ark` `tauri` `vscode`（多目标全生成；声明 `arkts` 规范化为 `ark`） | ✗ | ✗ |
| `build_ui` | `vue` `rust` `jet` `ark` `vscode`（vm 解释型无构建步=具名拒绝） | ✓ | ✗ |
| `run_ui` | `vue` `vm` `rust` `jet` `ark` `tauri` `vscode` | ✓ | ✓ |

### run 前置与执行语义（诚实边界）

- **所有 served run 都是真实执行链**（T-06 R1/F-815-T06-R1-02 后）：vue
  =dev-server（Launched URL）；vm=隔离子进程会话（Input q）；rust=cargo
  run；jet=gradle 构建+adb 安装/启动（设备探测=owned 捕获，R6 契约）；
  **ark=构建 HAP→hdc 安装→aa start 启动**（bundle/ability 读生成产物）；
  tauri=exec_local dev；vscode=code 启动。
- **前置缺失=具名 failed**：jet 缺 adb/设备、ark 缺 hvigorw/hdc/设备等
  均返回具名 error（含手动回退指引），不冒充运行完成；自然完成的空结果
  才是合法终态。取消=cancelled。
- **front 声明贯穿**（T-06 R1→R2 后全链）：请求 `app("front")` 节点形式
  （优先）或 `front:` prop 贯穿 VM entry/CWD/模块种子、**Rust 生成源/
  增量判定/交付/运行 CWD**、jet/ark/vscode 生成布局（`request_front_dir`，
  镜像 parse_workspace_path）；无声明=默认 `src/front` 正控保持；legacy
  CLI 直跑保持冻结探测链。
- **Ark 启动 ID**（R2 后）：bundleName/mainElement 按生成 JSON5 值语义
  读取（尾逗号/引号剥净），缺失/不可读=具名失败——不以默认 ID 猜值启动。

## 协议消息（v1 完整面）

```
ClientEnvelope{protocol_version:u32, id:u64, body}
  body = Describe
       | ValidateProject{request: WireRequest}
       | GenerateUi{request}
       | BuildUi{request}
       | RunUi{request}
       | Input{target_id:u64, data:Vec<u8>}     // 协议级控制消息
       | Cancel{target_id:u64}                   // 协议级控制消息

ServerEnvelope{id, body}
  body = DescribeOk{descriptor}
       | ValidateOk{report: ValidateProjectReport}
       | GenerateOk{report: GenerationReport}
       | BuildOk{report: BuildReport}
       | RunOk{report: RunReport}
       | Cancelled{target_id}                    // 取消回执（与工作终态分离）
       | InputAck{target_id, bytes}              // 输入投递回执
       | Event{event: Progress{message,done,total?} | Launched{url}}
       | Error{error: ToolError}
```

- `WireRequest` = `auto_project_model::GenerationRequest` 的 wire codec 编码
  （数据 owner 同源 codec，协议 crate 零复制）。
- 退出码：0=EOF 正常 / 1=内部（控制流写失败即 exit，不等 EOF） / 2=流级致命；
  vm 会话子进程另有内部臂专用 3=必需外部后端装载失败（不出现在客户端面）。

## 消费合同（816 客户端）

1. **只依赖** `auto-tool-protocol` + `auto-project-model`（不静态链接 auto-ui-tools）。
2. 启动 `auto-ui-tool` 子进程，stdin/stdout 帧化 stdio，stderr=日志。
3. `describe` 先行，按能力列表分派（A-only 客户端跳过 generate/build/run/input）。
4. 单飞工作操作：并发第二个工作操作=具名 busy 拒绝。
5. `Input` 字节到达前台子进程 stdin（VM 会话行 `q` 退出）；回执异步
   （`InputAck` 可能与 Progress 交错，按 id 匹配）。
6. `Cancel` 立即回 `Cancelled` 回执（回执≠终态）；被取消操作的终态：
   **run → `RunOk{outcome:"cancelled"}`，build → `BuildOk{outcome:"cancelled"}`**
   （两条终态通道都必须消费；取消后 owned 子树先真实终止再有终态，
   reader 收尾失败披露在 error 字段）。
7. build 的失败语义：构建命令非零退出、缺构建工具、或退出 0 但产物
   oracle 缺失（如 jet 无 APK/ark 无 HAP）都是 `outcome:"failed"` + 具名
   error——不得把 stderr 提示当作成功证据。
8. generate 的多目标语义：请求声明多少个 served frontend 就生成多少个
   （顺序保留）；`GenerateOk.frontends` 是实际生成集合，`output_root`
   是各 profile 真实输出根的最深公共祖先（tauri=`gen/`——前端
   `gen/front/vue` + 后端 `gen/back/rust` 双根聚合）。
9. 捕获探测的失败披露（R6）：内部捕获式探测（如 jet run 的
   `adb devices`）超时/截断/读错误/非零退出/未能启动都以
   `RunOk{outcome:"failed"}` + 具名 error（含 DrainEnd 分类与 owned
   树 teardown 收据）披露——不完整的探测不是"无设备"，客户端不得把
   completed 当探测成功的证据；自然 EOF+干净退出的空设备列表才是合法
   "无设备"。

## 运行架构（内部）

- 协议循环线程 + 工作者线程 + 专属输入写线程（128 深有界队列+可停写）。
- VM 会话进程隔离：`--internal-vm-exec` 内部执行臂（每 VM run 一个子
  进程，iced/winit 单事件循环约束的进程级解法；取消/EOF=终止 owned 树；
  stdin 接 Input 通道；stderr 管道化转发+URL 嗅探=Launched 准确端点；
  必需外部后端装载失败=具名 fatal 退出码 3，宿主映射 failed 不启动会话）。
- **每请求子进程上下文**（R4-01 根修）：VM 会话子进程只获得当前请求
  衍生的 env 条目（外部后端根/主题声明 JSON）；两个内部键先从继承环境
  移除（`env_remove`）再按请求追加——串行第二个请求（外部→本地、主题
  Some→None、失败后新会话）不沿用上一请求上下文；子进程不重读主 pac。
- **包管理器单源**（R5 / F-815-T05-R4-04 根修）：工具工作操作期间
  `pkg::resolve` 消费**当前请求**的 raw `pkg:` 字段（未建模字段经请求
  原样传递）；请求无该字段时跳过磁盘主 pac.at，只走外部全局配置
  （`~/.auto/auto-man/am.at`）→自动探测；legacy CLI 直跑保持"磁盘
  pac > 全局 > 自动探测"冻结链。每操作 guard 设置/清理，不跨操作缓存。
- **owned 树统一设施**（R4-05 收口，`auto-tool-execution`：Windows Job
  Object / Unix 进程组——取消/EOF 终止整树+有界收尸+reader join）：
  - vue：install/dev/build（pkg `run_command_live`/`run_script_live` 托管臂）；
  - vm：会话子进程本体（含其 stderr 管道）；
  - rust：cargo api-server/build/run（`cargo_status` 托管臂）；
  - jet：gradle wrapper 生成 / assembleDebug（build+run）/ installDebug /
    adb devices（`capture_stdout` 捕获排水，仍为 owned 树）；
  - ark：hvigorw assembleHap；
  - vscode：`code --extensionDevelopmentPath` 启动；
  - tauri：dev 链经 pkg `exec_local` 托管臂。
  上述均只在**托管形态**（RunIo 在装）走 owned 设施；legacy CLI 直跑保持
  冻结的直接 Command 行为（`run_external_status` 双形态）**与基线返回
  语义**（R5-01：构建失败/缺工具在旧公开入口打印提示后 Ok——严格具名
  Err 只属托管/协议形态）。真终态 oracle：reader 三分类
  NaturalEof/Stopped/IoError，非自然 EOF 具名披露。
- **捕获排水同约束**（R5 / F-815-T05-R4-05-P1）：caller-drained 捕获
  管道与 forwarder 同样应答本操作的取消/EOF（停止标志/取消标志每刻
  检查+硬界限+1MiB 收集上限）；终态分类 Eof/Stopped/Truncated/IoError/
  BoundOverrun——部分输出不冒充完整探测结果；被取消操作的终态恒为
  cancelled（`map_flow` 对取消后到达的 Ok 改判 Cancelled）。
- reader 可停读双平台同构：Windows PeekNamedPipe / Unix `poll(2)`（10ms
  轮询刻，停止标志恒可观察；非 owned 持管道者不能把 reader 钉死在阻塞读）；
  join 两阶段=短自然收尾窗（正常完成不被误分类为 teardown 停止）+强制
  停窗（有界，截止后不再无条件 join）。

## 来源清单

| 来源 | 位置 | 许可 |
|---|---|---|
| gallery 资产 | `crates/auto-ui-tools/assets/gallery/` | 本仓自著 |
| wm 资产 | `crates/auto-ui-tools/assets/wm/` + drawlist 输入 | 本仓自著 |
| shadcn-vue 快照 | `crates/auto-ui-tools/assets/shadcn-ui/`（345 文件） | MIT（上游 registry 快照 2026-08-27，SNAPSHOT.md 记录） |
| Inter 字体 | vue.rs 内嵌（三字重） | SIL OFL 1.1 |

## 规范增量（SD-01..07 正文草案，送 T-09 终审）

> before/after 规则正文；A 阶段已审定子集（review-r4/spec-delta-reviewed.md）
> 不变，此处为完整 B 面正文，随 T-09 审定后落 canonical。

### SD-01（add）`docs/specs/auto-ui-tools/project.md`

- **before**：UI 工程生成/构建/运行/专属资产（vue/rust_ui/jet/ark/tauri/
  vscode/shadcn/gallery/wm/…）由 `auto-man` 承载，管理大库与 UI 工具闭包
  互相缠绕；run 面经 `vue_run_compat`/`rust_ui_vm_compat` 兼容壳。
- **after**：`auto-ui-tools` 是 UI 工程工具唯一 owner——生成/构建/运行/资产
  全闭包脱 Man（normal/build/dev 无 auto-man/legacy auto-lang 大包依赖）；
  旧 Man 入口为薄转发门面（语义冻结）。生产入口全部消费已求值
  I-PROJECT 请求（`*_with_request` 家族），不得回读主 pac.at 形成第二
  真源；外部命令经统一 owned 子进程设施；构建失败/缺工具/产物缺失为
  具名失败；多目标请求完整生成并诚实聚合（输出根=真实布局）。
- 验收：AC-01..04（V-01/02/04）。

### SD-02（add）`docs/specs/tool-protocol/project.md`

- **before**：CLI 与 UI 工具间无统一轻交接协议，消费需静态链接实现。
- **after**：I-TOOL v1——版本/能力（五操作×cancellable/interactive 真实
  语义）/I-PROJECT wire 载荷/framing 预算/错误分类/日志与控制流分离/
  输入有界队列/取消与 EOF 的 owned 树终止/退出码合同。取消回执
  （`Cancelled`）与工作终态（`RunOk`/`BuildOk` outcome=cancelled）分离；
  单飞工作操作；能力拒绝先于任何副作用。非 Windows 清理不变量（reader
  可停/有界 join）为平台承诺，不得以 Windows 结果外推。
- 验收：AC-03（V-03/V-05）。

### SD-03（modify）`docs/specs/auto-man/project.md`

- **before**：Man 持有 UI 生产算法/资产/run 面；兼容壳散落。
- **after**：UI 生产算法退出 Man（T-04 迁移完成）；Man 门面纯转发（六兼容
  /媒体/请求缝定向验证）；非 UI adapter（api_gen 语言服务桥等）归属
  `auto-api-gen`（owner=lang-service-adapter）不随 UI 域；协议侧不得以
  旧 CLI 的提示性吞错语义为由掩盖构建失败（R4-06 教训入规）。
- 验收：AC-01/AC-04。

### SD-04（modify）`docs/specs/project-model/project.md`

- **before**：生成器各自读取工作区 pac.at（隐式第二真源）。
- **after**：所有生产入口消费 809 已交付的 I-PROJECT 单源请求（字段/
  源交接）；静态准入不扩大；`arkts`/`ark` 等同源名在工具边界规范化；
  主声明磁盘修改/移除不改变已求值请求快照字段（vscode/vue build 等
  字段来源表随测试锚定）。
- 验收：AC-02（V-02）。

### SD-05（modify）`docs/specs/auto-lang/ui/overview.md`

- **before**：工程工具/资产直接依赖语言私有面或 Man。
- **after**：工程工具/资产消费 814 公开包（auto-ui-compiler/
  auto-lang-core/auto-ui-runtime）；子进程所有权覆盖所有已声明
  cancellable/interactive 路径的实际外部命令（直接 `Command::status`
  绕过 owned 设施=违约）。
- 验收：AC-01/AC-04。

### SD-06（modify）`docs/specs/repository-boundaries/generated-ui-distribution.md`

- **before**：发行/producer/资源闭包按 Man 域描述；无 tool 二进制身份。
- **after**：新 tool/协议/source/embed/producer 闭包（DELIVERY_CRATES 26）
  与适用失败矩阵（构建失败具名/产物 oracle/取消树终止/中断恢复按
  profile 精确）；消费交接以**冻结源 SHA+二进制 SHA256+支持指纹**为身份
  （见本文首表），不以源码路径冒充二进制身份。
- 验收：AC-04。

### SD-07（modify）`docs/specs/auto-ui-runtime/project.md`

- **before**：runtime 无宿主退出请求/会话生命周期公开面。
- **after**：新增 `request_host_exit`/`host_exit_requested`（代计数器）/
  `host_exit_window_begin|end`（幂等窗口）公开面——CLI 直跑形态零行为
  变化；会话只响应窗口内到达的新代请求（初始化期请求不丢、跨会话陈旧
  请求不误杀）；实际可重入边界=每 VM 会话一个子进程（iced 单事件循环
  约束的进程级解法）。完整生命周期与失败边界正文随 T-09 冻结。
- 验收：AC-03/04。
