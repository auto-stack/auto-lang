# T-04 轨D desktop 跨仓配对决策工件（PLAN-698 / 2026-09-23）

## T-04a 管道命名裁定（待澄清2 呈报，按授权默认案定稿）

**注册表发布 + pid 派生隔离；wellknown 兜底=桌面外 `-q` 轨不变。**

- 管道名：`autodesk-rqhost-desktop-<pid>`（RQHOST_PIPE 派生）——多桌面
  实例天然隔离；锁管道（`<name>-lock`）互斥面与 wellknown 实例无关。
- 发布：桌面宿主进程设 `AUTO_DESKTOP_ENDPOINT=<pipe>` env——子进程启动
  即继承，注册表 launch 面零改动；消费=a2r exe `--desktop-endpoint <pipe>`
  （P693 交付的 ClientTarget::Desktop 采纳臂，不孵化语义）。
- 兜底：桌面外启动的 exe 走既有 `-q`/wellknown rq 轨，语义零变化；
  desktop 模式缺端点 = 干净报错（P693 AC-03 已验）。

## T-04b 形态勘定：子进程（库形态被结构性否决）

**实机留痕**（scratch-w1/t04_desktop.log，2026-09-23）：桌面宿主进程内
专线程 `run_daemon` → winit 0.30.13 Windows 事件循环双杀：
①`event_loop.rs:190` "Initializing the event loop outside of the main
thread…use EventLoopBuilderExtWindows::any_thread"；②主线程桌面循环
`iced_winit lib.rs:81` "Create event loop: RecreationAttempt"——**winit
Windows 每进程单事件循环**，第二 iced 循环结构性不可行。按计划预授权
（待澄清3"不成立则子进程形态兜底，勘定后即定不阻塞"）落子进程形态：
`spawn_desktop_daemon(pipe)` → `auto rqhost --pipe <name>`（复用
`rqhost_auto_binary` 寻址 + NEXTEST_ env 清洗，spawn_rqhost_default
同型）；就绪探测预算 5s（桌面 boot 不被孵化面长阻塞；未就绪放行，
客户端 adopt 自带干净报错/重试）。生命周期：daemon 子进程按末窗自退
语义存活（零窗常驻与 `-q` 轨一致）；宿主强杀不级联（Windows Job 对象
级联挂接为后续债，不在本计划面）。

启用面：`DesktopOptions.rqhost_endpoint: Option<String>`（显式名）∨ env
`AUTO_DESKTOP_RQHOST=1`（pid 派生名）；两者缺席=不孵化（行为零变化）。
auto-os 侧 `scripts/desktop.ps1` 设 `AUTO_DESKTOP_RQHOST=1`（可显式置 0
停用）——vue/iced 两轨同过 run_session Desktop 门。

## T-04c 全环录证（003-converter exe，2026-09-23 实机）

载体：worktree 构建 `auto build -r rust` → converter.exe（70MB debug）。
宿主：ui_desktop（窗口模式）+ AUTO_DESKTOP_RQHOST=1。

观测行全环（t04-desktop-host.log.txt / t04-converter-client.log.txt）：

```
[ui-desktop] rqhost daemon spawned at autodesk-rqhost-desktop-6336 (AUTO_DESKTOP_ENDPOINT published)
[rqhost] serving on autodesk-rqhost-desktop-6336
[render] render-mode: desktop (CLI)                     ← 客户端
[render] desktop 模式：采纳桌面合成器端点（不孵化）        ← 客户端
[rqhost] adopt converter -> autodesk-rqhost-desktop-6336-app-1
[rqhost] window opened for `converter` (480x320)
[rqhost] first frame `converter`
[rqhost] window `converter` resized 480x320 ×2
[rqhost] perf `converter` fps=0.2 frame_bytes=412 ops=9 texts=6   ← v2 帧在流
[rqhost] client `converter` 断连（EOF）——窗回收            ← 关闭回收
[rqhost] 末窗关闭——daemon 退出
```

- 拉起=adopt+开窗+首帧；交互=存活期 cursor sweep 后 perf 帧行在流
  （帧管线+输入驱动的重排可见）；回收=EOF 窗回收+末窗自退。
- 双进程存活位：host alive=True × client alive=True（10s 窗口）；
  截图 t04-both-windows.png（屏级，双窗同框）。

## 与 SD-04 的对齐

desktop 端点契约（ui/overview.md）：auto-os 桌面=真实桌面端点（子进程
daemon + env 发布 + 003 互连口径）；管道命名如上；输入路由深集成
（虚拟窗内嵌/键面）仍归 P694-D2 线（本批非目标，计划已明确）。
