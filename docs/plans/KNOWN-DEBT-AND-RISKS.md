### PG-MEM（2026-10-09，playground 全量示例走查实测登记——走查副产品）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| ~~PG-MEM-1~~ 已销账（PLAN-746，2026-10-09 交付 95c093001：VM 协作式截止+timeout_secs 默认 10s，全量走查 +11MB/挂起清零） | high | auto-playground /api/run 执行管控 | **run 接口无超时/取消——挂起执行永不终止并持续吃内存**（2026-10-09 实测：1343 示例串行走查后服务端 16MB→1434MB，其中 ~740MB 来自 5 个挂起示例各跑满客户端 90s 超时仍在服务端继续；批量结束后挂起任务继续增长到 1.77GB+ 仍不收敛；早前 8 并发批 20GB+ 同因）。挂起源三类：①`print(i, end=" ")` kwargs 形态触发 VM 高速缓冲膨胀（~3MB/s，think-python ch07 块 01/02 两围栏 +210/+348MB）——独立 VM 缺陷；②书摘真死循环（`for true {}`）；③已 #[ignore] 的 plan231 无限循环语料。清偿方向：run_handler 加执行超时（返回错误+丢弃任务），根治需 VM 燃料/取消机制 | crates/auto-playground/src/routes/run.rs:26（spawn_blocking 无超时）；crates/auto-lang/src/lib.rs:471（join unwrap）；scratch/playground-check/results_mem.json（逐示例内存曲线） |
| ~~PG-MEM-2~~ 已销账（PLAN-746：内建族 kwargs/Bina(Asn) 形态编译期拒绝） | low | VM print kwargs 形态 | **`print(x, end=" ")` 不报错反而挂起并以 ~3MB/s 膨胀**（应为编译期拒绝或忽略 kwargs）——块 01 `for i in 0..3 { print(i, end=" ") }` 90s 内 +210MB；块 02 同形态 +348MB。与 PG-MEM-1 的取消机制正交，属 codegen/内建签名校验缺口 | scratch/playground-check/results_mem.json [903][904]；think-python ch07-iteration 块 01/02 |
| ~~PG-MEM-3~~ 已销账（PLAN-746 T-04 归因修正：基线缓涨系挂起旋转窗口误判，挂起源消除后净增 +11MB≈工作集涨落，非泄露） | low | auto-playground 单进程基线 | **逐运行基线缓涨 ~1MB/示例**（148 条 >1MB，多为快成功运行，如 ch14-classes-functions 块 03/04 各 +33/+26MB/0.02s）——Windows 工作集不归还 + 每运行全局状态（惰性 native 注册/类型缓存）叠加；相比 PG-MEM-1 次要，单独 profiling 再定责 | scratch/playground-check/results_mem.json |

### P726（2026-10-02，复审登记的预存发现——非本计划引入）


| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P726-R1 | medium | book_listing 再生成工作流 | **裸 `cargo tb` 经 `generate_book_expected` 无条件改写外部书仓全部 expected 文件**（fs::write 恒写，非仅漂移项）——①破坏性：书仓在途 WIP（如 PLAN-715 期 ch06 编辑）会被当前转译器输出覆盖；②竞态：与对拍族同池跨进程并行，expected.rs 读写互踩致红集非确定（同克隆实测 7 红/16 红两采样，2026-10-02）；③新交互：PLAN-726 T-03 脏态守卫落地后，generate 改写使书仓变 M 脏 → 后续对拍全 skip（消息可归因，非静默）。清偿方向：generate 收编为显式 env 门工作流（如 AUTO_BOOK_REGEN=1）或改仅写漂移项+先备份 | crates/auto-lang/src/tests/book_listing_tests.rs generate_book_expected（fs::write 无条件）；726 §8 T-03 回执预存发现① |
| P726-R2 | low | desktop_protocol FFI 卫生 | **stage3.rs 内同名 Win32 函数双声明签名不一致**——K32GetProcessMemoryInfo（mem_ffi，isize 句柄）与 EnumWindows（两处，签名不同）跨 extern 块重复声明且类型不匹配，编译器持续告警；同 crate 同符号两种签名理论 ABI 风险（现均为 8 字节句柄/x64 实际无害）。PLAN-726 已把自家 OpenProcess 声明对齐 isize 形态规避新增告警；存量两处清偿=统一收敛到单一 FFI 声明模块 | crates/auto-lang/src/ui/desktop_protocol/stage3.rs:146-152（mem_ffi）与 :5418（EnumWindows）；cargo check 告警实录（726 §8 T-05 预存发现②） |
| REG-4 | medium | kitchen-sink/schema 金样跨仓脱同步 | **schema_drift_fence / queue_coverage_drift_fence / docs_gen kitchen_sink_page_in_sync 三红族（2026-10-02 批量回归新红）**——根因=跨仓 auto-os commit `73d02b5`（Plan 717 "show all component examples"，2026-10-01 05:05）手改 widgets-gallery/src/front/pages/kitchen-sink.at 未走 `KITCHEN_SINK_UPDATE=1` 再生：时间线 0d5f5bf 重采样（09-30 14:21）→ 09-30 批量回执（10-01 00:20）三绿 → 73d02b5 手改 5h 后破红；auto-lang 窗口 c19b5351a..4614bd9df 零触 docs/components 与 schema 源；非 PLAN-726（stash 对照 pristine@17292c07e 同红，先于其 diff）。清偿=跨仓修复：auto-lang 侧 `KITCHEN_SINK_UPDATE=1 cargo test -p auto-lang --test docs_gen` 再生 + schema 围栏联动 + auto-os 侧提交重采样金样 | auto-os widgets-gallery/src/front/pages/kitchen-sink.at@73d02b5；crates/auto-lang/tests/docs_gen.rs:406；docs/plans/.last-batch-regression.json 2026-10-02 回执 |

### P723（2026-10-01，28应用图文介绍素材核查）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P723-D1 | medium | AutoTerm 当前本地 VM 拍摄 | 以现有auto.exe（SHA-256 d23bd5338d27259807eb31f5c3eea7457fbafdf7a0453bcfa692b82db22129de）运行auto-term app源码普通副本，run -r vm --merged，设置实际AUTOTERM_ENGINE_DLL后App.Tick仍报future_all/race: empty or invalid future list，窗口未产生有效会话，空图拒收。当前723使用2026-09-22已留存Rust原生验证图（素材commit6ff3bad0ff0f780e81e7c9c13af2f0b3f8902040）并公开说明日期/形态，AC-03真图成立；未声称当前VM功能通过。后续匹配工具链/真实App.Tick修复后重拍，不在网站任务改终端实现 | docs/reports/p723-capture-catalog.json auto-term；website/apps/demos/terminal/index.md；website/scripts/p723-capture-vm.py |

### P722（2026-10-01，网站 Apps 介绍复审）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P722-D1 | low | 网站应用开发画廊 | 网站已打包 registry 使用旧 ID（如012-stopwatch/025-dashboard），与现势源012-clock/025-sys-monitor不同；不以页面可挂载推断操作/后台覆盖。后续从源再生成画廊并逐项核验，禁止手改bundle。当前新介绍为静态分类且明确运行条件，未影响722必需验收；28候选后续素材/验证是批准设计，不计未完成实现 | Plan722；docs/specs/website/design/application-demos.md §4；docs/reports/p722-source-baseline.json（722 worktree/branch） |

### P716（2026-10-01，供③ 确认复核件降级挂账——plan_revision 2 用户裁定）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P716-D1 | low | auto-edit 矩阵 供③ 多跑复核 | **大文件卡死回归「确认复核件」未完成——多跑×3 环境阻断，018 登记保持**——auto-edit `desktop_mcp.py` 矩阵 6 次启动实录：MCP 端口 TOCTOU 死占×3（9247=ui_desktop.exe PID 2824 [auto-musk 域在跑 UI] LISTENING+孤儿 auto.exe 占 9248/9249；`pick_free_port` connect_ex 探测对 LISTENING 态误判空闲）+boot 瞬态×1+**app 楔死×2**（主进程存活但事件循环楔死：X9-ALIVE 心跳 ~90s 停+SYN_SENT→8018 挂起+MCP 监听套接字消失——18 PASS 后断连）。根因假说=712 r2 T-16 桌面轨 AppTick 泵断链残余族（形态吻合**未证实**——同二进制 01:5x 空闲时段健康推进 T1→T13 20min+，楔死与机器态相关非确定性）。供③ 语义=稳定性确认件（018 v2205 单跑 T17.2/17.3/17.8 已全绿），非新能力；本件（PLAN-716）三组主体交付与其零耦合。**清偿路径**：712 T-16..T-19 修复交付后或共栖负载空闲窗口期，重跑矩阵 ×3（AUTOUI_MCP_PORT 绕开 924x 带）——T17 全绿→018 注记销账；若楔死复现→供③ 转「复现，原诉求恢复」（m1-supply §17 语义）。018 登记位未回写（不假销账） | 主检出 docs/plans/716-m4-upstream-batch.md §10 Q-5+T-12 行实录；auto-edit specs/auto-edit/tests/desktop_mcp.py pick_free_port:47；018-m4-perf-unblock-and-budget-gate.md:341 供③ 定性 |

### P706 r2 收口批（2026-09-30，work 阶段定谳的非本回归缺陷另立记账）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P707-R1 | high | async http 直驱臂（e4） | **e4 `default_headers_reach_wire_on_plain_get` 确定性红（非环境红）**——`spawn_async_http_handle`/`spawn_async_http` 直驱三臂在独占测试进程内恒不到线：曾无限挂死整跑（2026-09-30 双 tf 现行实录，其一卡死 30+ 分钟），fix-test-stability 加 30s `recv_timeout` 定界后 30.0s 恒红实证。同机 plan705 e2e（`Http.post_json` 全链）与 plan707 SSE（真 TCP 分时帧）均绿 ⇒ 机器网络正常，**头号嫌疑=plan707 T-02 `submit_detached_client_job` 消息桥分离**（fire-and-forget 路径疑似不再派发直驱 job）。4f123a50e 曾把 `default_headers` 计入"预存红 4"以环境红定责——**误标**。修复归 plan707 域；修复前日常档带此红 + 30s 长尾在案（slow-timeout 120s 终止兜底，防挂死回归） | stdlib.rs e4 模块（default_headers 三臂）；4f123a50e 定责行；.config/nextest.toml slow-timeout |
| P706-D1 | medium | back-bridge call（merged 直调臂） | **probe_mtime merged 臂 release 载体 `done` 恒 false（挂起）为独立预存缺陷**——探针 widget 零 computed/memo，Init 弧 `probe_cases` back-bridge call 不完成；A/B 定谳：old-carrier c8f86ef92（pre-delta）release **同形红**（`done: false` 超时），修复 exe debug 全案绿 → release 特异的 back-bridge 调用面，非 706/plan047/705 delta 引入。清偿需 release 档 CALL/段驱动时序定位（debug canary 先爆掩蔽同窗） | jade-edit tests/probe_mtime.mjs；`p706-probe-mtime-rel.log` vs `p706-probe-mtime-oldrel.log` vs `p706-probe-mtime-dbg.log`（组目录 .wt/lang-706/） |
| P706-D2 | low | jade-edit e2e harness（消费方仓） | **vue 轨 Playwright e2e `快速打开` menubar 弹层点击预存 flaky**——元素反复 detach → click 重试至 240s 超时；双载体统计同 flaky（old c8f86ef92 2/4 pass / new 载体 1/4 pass，无区分度）⇒ 非本仓 delta 回归；gen/front/vue 为共享陈旧产物（前端非变量）。清偿归属 jade-edit harness 加固（force-click/动画等待/焦点稳定），不在 auto-lang | jade-edit e2e/matrix.spec.ts:632 waitButtonIn click；`e2e-old-run1..3.log`/`e2e-77d05-run*.log`（组目录） |
| P706-D3 | low | widgets-gallery VM 臂侧栏导航 | **sidebar 连按导航偶发失灵（双载体同）**——MCP press 后路由不切（old-carrier 2 连按后滞留 / new 载体同现），press 间页面重挂载期 MCP 响应可 >10s；四页收口经 press→fixture 双通道重试达成。机制未查（疑页面重建期 press 竞态），消费面仅探针脚本 idiomatic 影响向 | `.wt/lang-706/gallery-spot*.log`；widgets-gallery src/front/app.at routes |

| P706-D4 | medium | plan705 e2e 门禁（tf 并行池） | **plan705 e2e 探针裸 `#[test]` 入 tf 全量池→套件挂起风险**——`plan705_e2e_deadline_cancels_parked_and_reclaims` 绑定固定端口 18511/18512+全局 env 变更（AUTO_HTTP_REQUEST_TIMEOUT_MS），tf 并行 run 实测 >62min 挂起（隔离 <3min 绿）；其自家计划 §6 明示真 TCP e2e 应入 `test-http-e2e` 门/th 串行池。706 r2 复审按 `-E` 排除后收口（单测复证在案）。清偿=cfg 门收编 th 串行池（705 域治理，随 705 系后续档） | crates/auto-lang/src/tests/plan705_spike_tests.rs:711；.wt/lang-706/tf-review-full.log vs 隔离复跑记录 |

### P708（2026-09-29，实施方案审查：待核实风险）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P708-R1 | high | outlet memo 失效 | preview/nav 等宿主局部状态未明确纳入 outlet 产物的失效纪律；UI 标脏仍可能命中旧产物。本次仅静态检查，未实机复现。缺省扩大前须验证 memo 开/关行为一致；登记风险不表示验收通过 | aura_view_builder.rs:4387/5292；renderer.rs:17214；reports/708-design-review.md R708-05 |
| P708-R2 | high | UI 连续占用 | CPU handler/resume、computed、冷态模板构建与 layout/draw 仍可同步占用 UI；702 parked 与延迟 Init 不提供 CPU 时间片。10–30s 具体归因待 T-00 分段实测；不得以首帧截图代替加载期间交互验收 | vm/engine.rs:2331/2513；components/filetree.at:39；reports/708-design-review.md R708-01/02/07 |

### P683（2026-09-22，Plan 683 rq-remote-renderer 方案 2 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P683-D1 | low | remote wire v2 | **mesh/canvas 原语 v2 词汇面外**（G2 词汇表未列；tiny_skia `layer.primitives` 位图降级承接未实施）——试点三例无 canvas 面，首个 canvas 应用上 remote 时需 Path 序列化或位图降级（T-01 勘定登记） | headless.rs lower_layers_v2 观测臂；设计档 rq-remote-renderer §4 |
| P683-D2 | low | remote 降格 | **Transform 旋转/剪切矩阵元不可达**——iced `Transformation` 未公开矩阵元（仅 scale_factor/translation），v2 降格只产轴对齐 [s,0,0,s,tx,ty]；旋转内容（canvas 旋转动画等）remote 面暂缺 | headless.rs lower_text_v2 近似臂观测 |
| P683-D3 | low | remote 图像 | **svg 图像（tiny_skia Image::Vector）降格省略**——v1/v2 词汇 src 均为位图通道；svg 组件经 lucide 栅格化外的直引形态需 src 词汇扩展或宿主侧 svg 栅格化 | headless.rs lower_image_v2 Vector 臂观测 |
| P683-D4 | low | daemon 重放 | **shadow blur 无 canvas 原语**——v2 wire 携带 ShadowSpec 全参，daemon 重放为偏移半透明垫层（无 blur 扩散；alpha 按有无 blur 稀释）——观感近似，004 卡面 shadow-sm 实测可接受 | broker_surface.rs paint_shadow_scrim |
| P683-D5 | low | remote 环境注记 | **已销号（PLAN-690 T-07 自愈臂落地）**——daemon `WindowResized` 0x0 守卫：零尺寸（最小化路径）不向 app 转发、基线尺寸不动（恢复期假空白根除），真实尺寸恢复 resize 自然同步；管道环 p690 测试③断言绿。原走查恢复臂诉求由守卫替代 | PLAN-690 commit（rqhost.rs 守卫 + p690 测试） |
| P683-D6 | low | remote IME | **大部销号（PLAN-690 G1）**——中文 IME 组合/候选定位实机已验：组合串+候选窗在 daemon 窗内按 App 焦点框定位可见（录证 `docs/plans/reports/p690-ime-walkthrough/`），enable 下行/定位/上屏通道全链在档。**残面**：物理 IME「组合→上屏」连续实机段（commit 串经子类桥入值）在多会话争焦窗下未录成（前台锁拒合成注入；FG-FAIL 留痕），桥→上行→入值段由管道环 p690 测试覆盖（同路径构造性等价），留一键复核（安静窗内 003 remote 走查脚本即验）。**694 T-04 对账注记（PLAN-697 AC-04）**：一键复核在 694 走查期再次因环境竞夺未录成（⛔ 阻塞在案，非状态翻转）；694 复审裁定管道环=协议级等价，残面降级为可选复核——行状态维持"大部销号+残面在案"不变 | PLAN-690 win_ime.rs 子类桥 + 走查录证；plan 690 待澄清②；694 T-04/复审对账（PLAN-697 刷新） |
### P684（2026-09-22，Plan 684 gallery 示例修复批 work 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P684-D1 | low | 027 back API | **vue 轨写操作无后端端点**——rev7 只做了读面（fs_list/fs_home/fs_drives）；重命名/粘贴/新建/删除在 vue 轨统一守卫提示「写操作仅在 VM 轨可用」。后端加写端点（fs_rename/fs_mkdir/fs_delete/fs_copy 契约+impl）即可让 gallery 内嵌全功能，形态沿用 thin 契约纪律 | examples/ui/027-file-manager/src/back/api.at；src/front/app.at 写守卫四处 |
| P684-D2 | low | 027 媒体面 | **vue 轨图片缩略图缺失**——image.thumb 为 VM 轨媒体管线专属（256px rendition 渐进浮现），vue 轨条目 thumb_src 恒 ""，网格视图回落 FileIcon。补齐需后端缩略图端点或 media 路由跨轨化 | examples/ui/027-file-manager/src/front/app.at NavTo vue 分支 thumb_src:"" |
### P693（2026-09-23，Plan 693 native-exe-tri-mode work 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P693-D1 | low | a2r 生成物存量 | **rust-workspace 既有 member 的生成 Cargo.toml 仍含 `ui-gpui = ["auto-lang/ui-gpui"]` 行**（PLAN-691 移除 feature 前生成）——存量 member 重复 resolve 即败（fresh 生成已被本计划生成器根修）；各 member 下次 `auto build -r rust` 重生成自然收敛，禁手工清（生成物 never-track） | rust_ui.rs generate_cargo_toml（已修）；主检出 examples/rust-workspace/011-calculator 等存量 |
| ~~P693-D2~~ | medium | desktop 接入跨仓配对 | **已销号（PLAN-698 T-04，2026-09-23）**——形态勘定=子进程（库形态被 winit Windows 单事件循环结构性否决，RecreationAttempt 实锤）；`spawn_desktop_daemon` 孵化 `auto rqhost --pipe autodesk-rqhost-desktop-<pid>` + AUTO_DESKTOP_ENDPOINT env 注册表发布（命名裁定定稿）；003 converter.exe 全环录证（adopt→开窗→v2 帧→EOF 回收→末窗自退，reports/p698-batch/T04-desktop-pairing.md）。SD-04 落 ui/overview.md；WM 深度集成仍归 auto-os 侧后续 | renderer.rs run_session Desktop 臂；rqhost.rs spawn_desktop_daemon；desktop.ps1 |
| P693-D3 | low | a2r 测试漂移 | **`merged_api_client_crud_fallback_for_uncovered_endpoints` 预存红勘定新增**（不在既有已知红清单）——测试期望旧 CRUD 兜底形（`static API_DATA` + `-> Vec<Value>`），实发为 PLAN-681 route A `api_impl` 伴随模块形；base rust_ui.rs 外科对照同败（与本计划 diff 零交集）。修 = 测试期望随 route A 形更新，归 PLAN-681 线或独立 L0 | rust_ui.rs:4813 测试 vs generate_merged_api_client route A 臂 |

### PLAN-698（2026-09-23，Plan 698 desktop-substrate-hardening-batch work 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P698-D1 | low | submenu 浮动式残余 | **像素级确认被 D5 screenshot 通道阻塞**——任意 menubar 开态 autoui_screenshot 10s 超时（基线零改动同现=通道级预存，与浮动/内联形态无关，判位留痕 T03 工件）；浮动面板截图验证待 D5 清偿后补一轮。交互细节：子面板开态点击=项动作+菜单关闭并存（shadcn 语义可接受但不精细）；hover 换子菜单归 P695-D1 键盘/hover 导航批 | popover.rs Panel::overlay；reports/p698-batch/T03-submenu-floating.md |
| P698-D2 | low | desktop 端点生命周期/接线 | **daemon 子进程与宿主无级联**（Windows Job 对象挂接缺——宿主强杀后 daemon 按末窗语义存活，零窗常驻）；注册表 launch 面尚未消费 AUTO_DESKTOP_ENDPOINT（env 发布就绪，launcher 自动带 `--desktop-endpoint` 拉起 a2r exe 归 auto-os shell 线）；虚拟窗内嵌/键面深集成归 P694-D2 | renderer.rs run_session Desktop 臂；desktop.ps1 |

## PLAN-692 债务候选（2026-09-23，W-2/W-1 走查批）

1. ~~**ui-cache 键缺生成器版本（infra，高优先）**~~：**已销号（PLAN-698 T-01，2026-09-23）**——生成器指纹进缓存键（build.rs hash src/ui_gen/** → AUTO_UI_GEN_FINGERPRINT；UICache.generator_fingerprint 不匹配整缓存失效重签，VERSION 1→2 迁移弃旧）；692 W-1 场景实机回归绿（改生成器→自动失效重生成，免手工清）。入库评估=生成数据注记落 SD-01（runtime/overview.md），入库策略不阻塞。（锚点：ui_cache.rs；build.rs）
2. **popover 选中后不自动关闭**：Combobox 选中候选项后 popover 保持打开（shadcn 官方为选中即关）。需 popover open 状态绑定 + CommandItem @select 联动关闭。
3. **VM(iced) 臂 command 家族未实现**：八 command 元素 `iced: none` 维持——VM 臂渲染降级。接线（含 `size` prop 的 iced scrollable 宽度）另立。
4. **reka sizes 初始测量 18px 下限（观察项）**：本仓环境下 ScrollArea thumb 长度常命中 reka getThumbSize 的 18px 下限（A/B 证实与实现无关；拖拽数学内部自洽，仅视觉比例偏小）。
5. **脚手架 h1-h6 基础 margin 内联隐患**：基础样式给 h1-h6 的 mt/mb 对"内联进 flex 行"的标题同类泄漏（PLAN-692 W-3 实证一处）；其余位置待观察，出现即补 m-0。

### P694（2026-09-23，Plan 694 desktop-dogfood work 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| ~~P694-D1~~ | medium | 桌面宿主稳定性 | **已销号（PLAN-697，2026-09-23 外因结案）**——勘定归因档案 `docs/plans/reports/p697-stability/attribution.md`：机器级外因（内核 bugcheck 三日三蓝屏 0x9F/0x1A/0xA + GPU TDR 族；走查窗口 System Event 6008 = 09-23 10:59:12 非正常关机实锤），蓝屏瞬时吞进程 = 无 panic 无错误行无 WER 应用崩溃，与观测签名逐项吻合。嫌疑面 back-proxy lazy-start/media 供给/inproc panic 边界经 13 轮复现矩阵（基线窗口化×5+全屏×2+禁A×2+禁B×2+他App×2 全绿零死亡，嫌疑面点火实证）+ WER 零宿主崩溃记录三重排除。观测加固落地：ui_desktop 宿主 `[ui-desktop] start/alive/exit` 足迹桩 + panic 钩子（死亡判位表在档案；CLI 路径 X9 形制已在）；对照缝 AUTO_NO_BACK_PROXY/AUTO_NO_NATIVE_MEDIA（勘定器械，生产零影响）。残面=机器稳定性本身（GPU 驱动/内核，非本仓代码面） | 归因档案 p697-stability/；examples/ui_desktop.rs 足迹桩；back_provision.rs 对照缝 |
| P694-D2 | low | 桌面 shell 输入面（VM 轨） | **launcher 全量列表导航在 VM 轨不可用**——搜索框键入（物理字符派发为 `key_<char>` VM 事件，input widget 不收 CharTyped）、滚轮滚动、方向键（Named 键不路由至 bind 表）三路实测均不通（028-launcher overlay）；行击直点正常。本计划以 recent 存储夹具绕行完成发现面录证；shell 侧输入路由修复归 auto-os/shell 线（本计划非目标） | 028-launcher overlay；session.rs 键盘路由（key_<char> 派发臂）；walkthrough 走查留痕 docs/plans/reports/p694-dogfood/ |

### P695（2026-09-23，Plan 695 menubar-widgets-batch work 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P695-D1 | low | menubar 键盘面 | **方向键/Enter 键盘导航未实现**（T-08 时间盒裁定）——需 iced 焦点基建（逐项 focus 节点 + 键盘订阅 + 索引态进 MENUBAR_OPEN 注册表扩展），Esc 关闭已走 Popover 既有捕获（AC-04 最小集满足）。实施时对齐 shadcn：↑↓ 移项 / → 进 submenu / ← 回父级 / Enter 激活 | convert_menubar_component；menubar-snapshot.md PLAN-695 扩注 |
| ~~P695-D2~~ | medium | VM submenu 浮动式 | **已销号（PLAN-698 T-03，2026-09-23 翻案 T-11）**——真因勘定=iced 0.14 官方嵌套协议（overlay::Nested 递归 Overlay::overlay 钩子）未实现，嵌套 Popover overlay 永不注册；Panel::overlay 收集钩子落地（绝对坐标零平移）+ 视图臂浮动恢复（RightTop），内联降为 AUTO_MENU_SUBMENU_INLINE=1 降级路径。契约单测+快照+全程零死亡佐证；残余（像素截图确认被 D5 通道阻塞+交互细节）转 P698-D1 | popover.rs Panel::overlay；aura_view_builder 浮动臂；reports/p698-batch/T03-submenu-floating.md |
| P695-D3 | low | autoui-verifier × menubar | **MCP 合成指针事件 × hover-switch 干扰**——合成 press 的指针位移穿过被包裹 trigger 触发非预期 toggle，菜单开合状态在 MCP 驱动下呈非确定（真实鼠标语义不受影响）；autoui-verifier 对 menubar 的状态断言须容忍切换抖动或改坐标无关派发（menubar-snapshot.md 扩注已记） | aura_view_builder MouseArea 包裹臂；T-11 走查实录 |
| P695-D4 | low | gallery golden | **已销号（fold 门内完成，25cb53fa7）**——基线重基线落定（理由随提交：auto-os master 演进 + menubar.at 语料定稿），复跑对账绿（1108s exit 0） | crates/auto-lang/tests/fixtures/gallery_vue_golden.txt |
| P695-D5 | low | auto-edit 截图通道 | **auto-edit VM 臂截图与 1s 状态栏 Tick 争用（预存特征）**——无菜单态 screenshot 亦恒超时，AC-06 对照截图不可得；可读性由 token 断言链（bg-popover light=白/popover-foreground=墨）+ 快照结构验证 + probe_menu.py 承载。修复归 041/render 时序线 | tests/probe_menu.py（exit 0 两轮）；T-13 走查实录 |
| P695-D6 | low | gallery Vue 臂活体 | **widgets-gallery Vue 臂活体走查环境受阻**——worktree 无 front workspace/node_modules，`auto run`（vue）宿主窗不自动起 vite（3024 恒 000）。unblock：宿主窗交互启动 web 或主检出跑 692 同款全量再生成流程。SFC 发射面已由 p695 vue 单测 + gallery_properties schema 符合门 + fold 前 gallery_golden 确定性覆盖 | T-11 执行记录；.auto/ui-cache.json（合并后须再生） |

### P696（2026-09-23，Plan 696 HTTP runtime hardening 复审遗漏/延期扫描）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| ~~P696-D1~~ | medium | VM pubsub / publisher SSE | **已销号（PLAN-698 T-02，2026-09-23）**——bus.subscribe 转真运行面（案①进程内 iterator 注册表：EVENT_BUS broadcast::channel(256) 镜像生成 events.rs + 转发线程 + AsyncHttpStream 臂，696 回收语义沿用）；POST publisher 臂 publish_post_broadcast（Typing/New{Type} 对齐 api_gen，has_sse 门控经 record_api_return_type 新侧信道）；017 VM 臂全环录证（SSE 帧与 Axum 形态逐字节一致 + 带外 POST typing UI 渲染 typing 指示）。§8.1 落表+seam 注记划线 | stdlib.rs shim_bus_subscribe/EVENT_BUS；http_server.rs publish_post_broadcast；reports/p698-batch/T02-vm-bus-subscribe.md |
| P696-D2 | low | VM HTTP 生命周期 | **VM `#[api]` listener 尚无 graceful shutdown API**——本文 §7.3 只记录目标行为；本计划落实了 body/SSE producer 与断连连接任务的回收，不包含 Ctrl+C/SIGTERM 停止 accept、排空活跃连接的服务级接口。后续实现时需定义关闭信号、listener 停止和活跃连接排空契约。 | `docs/specs/stdlib/design/http-server.md` §7.3/§8.1；`crates/auto-lang/src/vm/ffi/http_server.rs` |

### P701（2026-09-25，Plan 701 供料包复审遗漏/延期扫描）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P701-D1 | medium | VM i64 算术面 | **.at 层 TAG_I64 操作数的算术解码错位**——`time.now_ms()/1000` 实测归 0（DIV 臂按 i32 解码 TAG_I64 nv；SUB 等同疑）；now_ms 全宽值在 nv 栈/unified print/桥面（nv_to_pub_value 已补臂）正确，唯 .at 内算术不可用。下游 bench「.at 内差值形」的前置清偿件（平台级，另件）；工作期探针 #[ignore] 留档 | tests/plan701_supply_probes.rs `vm_time_now_ms_agrees_with_now_sec_at_level`；specs/auto-lang/vm/design/time-natives.md 桥面节 |
| P701-D2 | low | trans JS 轨 | **JS 轨 print 模块内遮蔽（预存运行期边界）**——javascript.rs 维持裸 `console.log`（JS golden 字面断言，改道已回退 c9901f762）；store `console` 字段模块内在 JS 轨运行期同形遮蔽，无 TS2339 编译面。SD-03 已改口注记；如需清偿随 JS golden 重基线一并 | crates/auto-lang/src/trans/javascript.rs:427；specs/auto-lang/ui/design/vue-use-fn-emission.md strict-TS 节 |

### P702（2026-09-25，Plan 702 ui-handler-async work 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P702-D1 | low | busy 标志 .at 查询面 | **`__busy_handlers` 镜像仅宿主读路径可查**——根态运行期追加字段不在声明字段表，MCP read_all_state 不枚举；.at 编译期 GET_FIELD 按静态 field_idx 对运行期字段不可达。".at 原生查询"（如 `.store.__busy_handlers.len()`）需合成期注入保留字段，等真实查询需求立项（重入忽略行为本身不依赖该面） | vm_bridge.rs sync_busy_flag；plan §T-04 口径偏差注 |
| P702-D2 | low | 释放档全量红 | **`cargo test --release -p auto-lang` 非维护门**——base 上即有 2 处 tests/ 编译 rot（本计划顺带修复：headless `()` Renderer 运行期门→编译期门、osconfig_integration photo_root 漏改）+ ~254 档位环境红（全部同名测在 tf debug 全档绿）。release 档红册清点/维护归属后续档位治理 | release_full3.log 分诊；tf_full2.log 5700/5710（10 红全预存在册） |
| P702-D3 | low | CPU-bound handler | **段驱动无抢占**——CPU-bound handler 仍占满一次 update 的 10M 步预算（WARN 后段中止），UI 冻结面在；裁定建议=ui_lint+BudgetExhausted 段切两步走（决策工件 docs/design/34 §Q2，待用户裁定后另立计划） | drive_handler_segment budget 臂；Design 34 |
| P702-D4 | low | musk 人在环走查 | **E7 原始事故端到端（PickFolder 原生对话框滞留>60s 后选择）留人工走查**——机制面已由探针 032 同驱动路径全链证实（park/交互/resume/落账 + 零忙等超时）；musk 腿需 auth/daemon 栈 + 人在环对话框操作。复跑手册 docs/plans/evidence/plan702/README.md | evidence/plan702/README.md musk 节 |

### CLEANUP（2026-09-27，主检出清理：fix-music-slim 弃置裁定登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| CL-1 | low | media scan 协议瘦身（P041-D6 残件） | **fix-music-slim 分支整支弃置（复核否决，非合并）**——c60eeaf56（瘦身：base 顶层一发+stream 相对路径，退役 url/audio_url/video_url/name/rel_dir）文本可合但有双臂硬伤：① rust 臂发射器 `auto-man/src/api_gen.rs:1789` 无 base/stream（相对 url 旧形），合入后 020 默认形态（`api:"rust"`）`media_base+stream` 拼空串播放死；② rel_dir 退役破 030-video-player（playlist.at:92 按 rel_dir 分组；共享发射器 back_proxy 服务画廊嵌入臂，"前端零消费实证"只覆盖 020）。其动机（to_value 大体转换上限疑虑）已由 PLAN-042 e7efcaa28 幂等根修+ScanProbe 回归钉实质消解。**复活前置件**（=一个 L1 计划）：双臂发射器同步出 base+stream（rust 臂 base=""同源相对；proxy 臂 base=绝对 origin+apps 前缀）+030 rel_dir 保留或迁移+两臂测试面+spec SD-02 契约行同步（overview.md:867/871）。弃置提交 SHA（gc 前可恢复）：c60eeaf56（瘦身本体）/f0bcd6e42（wip 调试桩+本机 media_root 路径，不具合入价值） | crates/auto-man/src/api_gen.rs:1789；crates/auto-lang/src/back_proxy.rs media_scan；examples/ui/030-video-player/src/front/playlist.at:92；docs/specs/auto-lang/ui/overview.md §SD-02 |


## 首次批量回归（/auto-plan:regress）新发现预存红登记（2026-09-30，PLAN-710 merge 触发首跑）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| REG-1 | low | book listing 档 | **book_ch06_09 / book_ch09_02 预存红**——首次 `cargo tb` 批量腿暴露；基线检出（c80887ab7，710 落地前）单跑复现同红=预存非 710 回归（golden 漂移候选，修复归 book 域另档） | .tb-full.log 首跑；fix-regress-attr-p710 基线复跑 |
| REG-2 | low | ffi_dual 批量档 | **ffi_dual_019_dep_layout_invariants 批量负载 flake**——tf 批量（5921 并行）红、pre/post-710 双基线单跑均绿（3.27s/2.97s）→ 并行负载态 flake 非 710 回归；复现窗口=全档并行期，后续批量跑观察复现率 | .tf-full.log 首跑；双基线隔离复跑在案 |
| REG-3 | low | app_registry 档（scan/launch） | **共享 target 撞名环境态红**——`convention_native_exe` 候选面含仓根共享 `target/<build>/<name>.exe`，examples/ui/041-auto-edit（name=auto-edit）与他会话在主检出 target/debug 建的 a2r 工作区产物 auto-edit.exe（PLAN-021/r3 系 corpus/复验构建，76MB@2026-09-30T20:16）撞名→exe 背书判定翻转→按 PLAN-694 语义退出 C 档策展钉面（scan 19 vs 20 红+launch 改走 exe 臂超时红）。归因实证：artifact 改名隔离后 26/26 全绿、复位后复现；非窗口内任何计划代码回归（7fcf913eb..HEAD 零提交触 app_registry/examples）。脆弱面=测试设计（共享 target 候选过宽+应用/示例同名），修复候选=共享 target 候选剔除或策展测试对 exe 背书噪音隔离——归 ui 域另档； artifact 在场期间批量跑按本条豁免 | 改名隔离复跑收据（2026-09-30 p714-regress，26/26）；convention_native_exe 源码面 app_registry.rs:162-200 |
| REG-1b | low | book listing 档 | **REG-1 同族欠账注记**——ch02_05/ch03_08/ch06_05/ch06_06/ch06_08 五枚与 REG-1（ch06_09/ch09_02）同签名（`aborting due to N previous errors` parse 形）同族：输入 main.at 均为 book 仓已提交态（WIP 仅 expected.rs 再生面）、auto-lang 窗口 7fcf913eb..HEAD 零 parser/lexer/依赖变更（Cargo.lock 零 diff）→ 解析行为不可能新变=covered_commit 时已在案，上批收据 known_reds_seen 欠录 5 枚（tb 腿红清单未全录——本批补全）。修复归 book 域另档同 REG-1 | .tb-full.log 七枚同签名核验；book 仓 git status（WIP=expected.rs only）+Cargo.lock 窗口零 diff 实证 |

## PLAN-715 批量回归登记（2026-09-30，收据 a984834a2）

- **并行负载序敏感 flake ×2（新登记）**：`ffi_dual_019_dep_layout_invariants`（tf 档）、`plan502_m3_layout_geometry_e2e`（tt/tb 档）——批量全档并行跑偶发红，隔离单测与整族隔离均稳定绿（ffi_dual 23/23×2 轮）；与 p053 族已登记的"并行负载序敏感——隔离可绿"同类。无语义归因（窗口内测试文件与被测模块无对应改动）。处置：批量档观察名单，若频率上升再立项查并发交互。
- **book_listing 外部书仓敏感性（登记观察）**：`tests/book_listing_tests::*`（tb 档，ch02_05/ch03_08/ch06_05/ch06_06/ch06_08/ch06_09/ch09_02）输入源为外部兄弟仓 `D:/autostack/book/rust/listings` 的实时内容——该书仓在途编辑（d7a71a7 重生成金样 + ch06 金样未提交 WIP）期间这批测试确定性红，随书仓落定自然收敛。本仓窗口对该路径零改动；book 仓编辑会话应在落定后复跑 `cargo tb book_listing` 确认。
- 既有基线沿用：712 §10② master 14 红族（p053×4/p054×2/plan606/desktop_protocol/iced renderer/e4/plan707/app_registry×2）本轮复现一致（native_gate 本轮未现=成员漂移）；a2vue_desktop 金样=4f123a50e 在案；plan484 streaming=711 §10 在案。


### PLAN-721 批（2026-10-01，用户裁定「app 修改到此为止」余题登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P725-R1 | medium | 帧管线单帧单建的自动化回归守卫（PLAN-725 复审 P725-R1） | builds=1 断言与 MCP 快照等价对拍未自动化——现行证据=在仓双态谱 JSONL（56/56 typed 帧 builds=1）+可复跑脚本 ladder.py（AUTO_FRAME_BENCH 门控探针在库）；CI 时态守卫需 headless iced-session+MCP 测试基建（现仓无此 harness——musk/renderer 轨测试均不驱动 view() 全路径）。下游帧档（type_latency）为第二道语义守卫。 | docs/plans/evidence/725/T-00-survey.md §2；plan725 §9 复审记录 |
| P725-R2 | low | 键入载荷退役的计数断言面（PLAN-725 复审 P725-R2） | 1MB clone 计数断言未自动化——MCP 驱动走模型层直写不触发 on_change 闭包（实测约束，s1 探针全程 0）；谓词决策已单测（input_payload_consumed_contract）；真实键盘路径的诊断面=s1 分段探针（在库）。 | plan725 §9 复审记录；SD-01 §3 |
| P721-R1 | high | T-19 读侧机理（暂停回弹根因末环） | **PLAN-725 关联注记（2026-10-02）**：帧管线增量件 T-00④ 读侧一致性核查——MCP 驱动键入谱（[P725-FRAME] 分段，30 键×3 档）未现形相邻构建同 binding 读值漂移证据（编辑路径负载面）；机理定谳入口不变，仍在 T-19。增量缓存正确性纪律遵守=frozen⑤（本件零「跳过重解析」快道引入——T-03 memo 扩面裁定不扩）。 |
下行 epoch 单调门只挡「世代回退」子类；桌面实测回弹写来自 **OnTime 触发的全新构建烤回 paused=false**（handler 两次写均 false、同一 binding 相邻构建读值不稳定）——读侧世界分裂/陈旧 memo 回放机理待定谳。复现即现形：`AUTO_SCHED_DIAG=1` 桌面跑 030 点暂停看 `video_build` 行。 | evidence/721/apptick-verdict.md 结论二；desktop-721g.log |
| P721-R2 | medium | T-11 播控条布局塌缩 | col_right 438↔658 进程级翻转非确定性源未定谳（712-r2 登记态原样移交）。定谳入口=进程内双构建比对 + builder 侧 HashMap 遍历审计（style 层 Vec 面已初查非嫌疑）。 | 归档 712 §10①；layout_tests.rs p712 探针族 |
| P721-R3 | medium | 030 seek 派发被 arity 守卫跳过 | 用户实机取证（030-standalone4.log 07:15）：进度条点击/拖拽 → `[VM-ARITY] Controls.SeekFraction declares 1 param(s) but dispatch supplies 2 — skipping` ×6——seek 完全失效。疑拖拽事件路径多带一参（712 T-12 `$0` 剥离同域）。 | 030 controls.at SeekFraction 绑定；plan-576 D4 守卫 |
| P721-R4 | medium | 验收通道注入静默丢 | desktop-721i 实例 handler inject 排队后零派发（g/h 同代码正常）——registry_id 窗口反查在窗重用/多实例下疑 miss；inject 臂 `let _` 吞错无日志面。验收通道可靠性债。 | renderer.rs apply_desktop_injects Handler 臂 |
| P721-R5 | low | 独立窗 ok=true 自退（seek 后） | 030 独立轨在用户 seek 交互后 `vm interpreter returned ok=true` 优雅退出（无 wgpu 告警，非 712 T-07 press 死因；用户主动关窗不可排除）。复现则按 T-07 同窗定谳。 | 030-standalone4.log 尾段 |
| P721-R6 | medium | T-19 桌面活体 ×10 复验 | epoch 门落地后桌面实机暂停 ×10 无回弹未走查（实例 I 用户占用+R4 注入丢+712 T-07 三重阻塞）。移交用户日常观察；若仍回弹=R1 机理现形入口。 | evidence/721/apptick-verdict.md 未竟段 |

### PLAN-712 r3 批（2026-10-01，归档收据 eed6acf25——未竟/登记项集中挂账）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P712-D1 | high | 桌面轨更新路径家族（PLAN-721 承接中） | **PLAN-725 家族注记（2026-10-02，worktree plan-725-dev）**：编辑路径帧管线分段谱定谳（ui::frame_segments——五段 S1..S4+builds/dirty，AUTO_FRAME_BENCH 同门）：S2 VM 段非瓶颈（0.07-0.33ms/帧）；S3 恒定 ~8ms；S4 曾随文档尺寸线性爆炸（root=iced scrollable 构造期 size_hint→content_height→fresh_fold_map 逐行物化，1MB 95ms/帧——已修，无折叠快道 O(1)）；Element 缓存 put-then-take 死缓存勘定（快道结构性不可能命中，死写移除）。双态谱+勘定=docs/plans/evidence/725/（下游 auto-edit M4 帧两行 FAIL 的上游清偿面）。 |
**推进中（2026-10-01，PLAN-721，worktree plan-721-dev @088b32aa6）**：①**T-16 结案**——r2「桌面内嵌轨 0 trace/Loading 卡死」定谳为伪象或已被 712-r3 治愈（sched_diag 四轴 trace 全绿：sub_parked/到达/恢复 50.9ms/置脏回填，018 内嵌实机复验）；②**T-19 帧级序列捕获**：暂停写全链落盘（mpv 真停+上行回灌）后 ~100-185ms，OnTime 触发的**全新视图构建烤回 paused=false**（同一 binding 相邻构建读值不稳定=读侧世界分裂/陈旧 memo 回放，机理开放；video_build trace 已就位）；③**已落部分修复**=下行世代单调门（VideoContractDown.epoch + contract apply 回退整包拒绝），mpv_contract 13/13 绿（712-r3 双锁零扰动），独立轨真实按钮暂停钉住 7s+ 零回弹；④**T-11 未动**（col_right 438↔658 定谳入口不变）。定谳文档 docs/plans/evidence/721/apptick-verdict.md（worktree 088b32aa6）。另：验收通道 handler inject 在 desktop-721i 实例静默丢（排队零派发，g/h 正常）——验收通道可靠性债，随 PLAN-721 待澄清②复查。 | 归档 plan docs/plans/archive/712-vm-desktop-defects.md §8.5.3/§9/§10①；PLAN-721 plan 文档 |
| P712-D2 | ~~medium~~ | 018 阅读链点验未完成 | **已清偿（2026-10-01，PLAN-721 承接批前置点验）**：独立 VM 单跑走查全绿——书架 3 本 → 点第 2 卡开对书（route /book/2，712-r3 Init 代际修复实机复证）→ 详情页标题/作者/「3 entries」+ 章节目录 3 章（884bcfa67 app 层在位）→ 点 Chapter 2 → route /book/2/chapter/2 + **`router.param("ch").to_int()` 堆化形态实测=2**（712-r3 第二层 materialize_obj_to_heap 修复实机复证）→ reading 页渲染章节标题/正文/「Chapter 2 of 3」/66.0% → Next → 第 3 章 100.0%（路由参数变化 Init 重跑在位）。载具=lang-721 worktree 二进制（master+T-1 门控插桩，行为面零差异）。截图 docs/plans/evidence/721/d2-reading-ch2.png。 | 018 book_detail.at（OpenChapter → /book/:id/chapter/:n）；reading.at:146 |
| P712-D3 | medium | ui_desktop 壁纸克隆 OOM（env 缺陷） | 桌面启动数秒~数分钟死于 `memory allocation of 2660706 bytes failed`——`load_image_bytes`（renderer.rs:6590）**每次视图重建裸 clone 壁纸字节**（purple.png 2.55MB），提交压力尖峰下 OOM 金丝雀。master 构建同崩（A/B desktop-master-ab.log）= 预存非 712 引入。修法候选：壁纸字节/解码结果缓存一次（挂 config 或 handle 缓存，Plan 650 缓存族同源）。本日 4 份崩溃日志存 worktree 组目录（desktop-r3*.log，未入库）。 | 归档 plan §10⑧；renderer.rs:6590/15354 |
| P712-D4 | low | release 档产物陈旧 | release 二进制 + `gen/front/vue/dist` 未随 r3 合并重建（PLAN-092 先例观察项）；桌面实机消费面=worktree debug 二进制形态。 | 归档 plan 合并收据 cleaned 段 |
| P712-D5 | low | 018 vue 轨 json builtin TS 报错 | `auto build`（vue 轨）`useBooksStore.ts` TS2552 `Cannot find name 'json'`——生成器未把 `json.from_value` 等 builtin 映射到 TS 侧（book_store.at:23 遗留；VM 轨不受影响）。018 vue 轨构建失败为存量，非 r3 引入。 | 018 book_store.at:23；ui_gen/vue.rs builtin 映射面 |
| P712-D6 | low | worktree 清理 pending + 僵壳 | `D:/autostack/.wt/lang-712b/auto-lang`（分支 plan-712-dev，已全部 landed @68000a5bd）待删——**阻塞=验收桌面正从该 target 运行（用户复验载具）**；用户复验完成后：杀进程 → `bash D:/autostack/wt-guard.sh` clean → git worktree remove + branch -d。另 lang-712 旧残壳（.tmp-712-app.log + 空目录链被僵进程 PID 35340 锁死）待**重启后**删组目录。 | 归档 plan 合并收据 cleaned 段；§10⑤ |
| P712-D7 | low | 语料过时注释 | viewport.at 头注「`flex-1` 与 `shrink-0` 在 VM 不生效」已不成立（Plan 370 Issue 1 起 flex-1→width=Fill 在位）——误导排查，待语料清理批顺带更正。 | 030 viewport.at:10-12；iced_adapter.rs:1158 |
## PLAN-718 网站执行登记（2026-10-01，T-07 视觉复核发现）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P718-D1 | low | website /playground hero | **Playground 页 hero 大标题 "Playground" 字形顶部被水平裁切**——1440 桌面 dark/light × EN/ZH 四变体一致复现；已核对 715 已入库截图（docs/reports/p715-website-ui/final-playground-en-1440-dark.png）同缺陷=既存，非 718 引入（718 未触 /playground 样式）。疑似 hero 标题 line-height/overflow 裁剪，一处 CSS 可修；不扩围，留独立修复 | docs/reports/p718-website-ui/VISUAL-REVIEW.md；docs/reports/p715-website-ui/final-playground-en-1440-dark.png 对照 |
| P718-D2 | low | website playground i18n | **en 版 /playground 语料树/按钮等组件文本为中文**（搜索标题或标签…/书籍示例/在 IDE 中打开）——auto-playground-vue 包组件级 i18n 未覆盖既存缺口；包源码不在网站计划可编辑范围 | docs/reports/p718-website-ui/VISUAL-REVIEW.md 次要观察；packages/auto-playground-vue |
| P718-D3 | low | website zh docs 侧栏翻译完备度 | **zh docs 侧栏残留英文条目**（Components/Script to ship/Tour/Generics 等）——ZH_TITLE_MAP 未覆盖的目录/文件名回退，内容翻译范畴非视觉缺陷 | docs/reports/p718-website-ui/VISUAL-REVIEW.md 观察项 3；website/scripts/prepare-content.js ZH_TITLE_MAP |

## PLAN-727 复审登记（2026-10-02，复审 @plan-727-dev 1fa1a49a4）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P727-F1 | medium | book 仓 listing 语法脱同步 | book `rust/listings` 的 is 臂仍用旧语法 `0 =>`（book_ch03_08/ch06_05/ch06_08/ch09_02 四例），现行解析器只接受 `0 ->`（语料 test/vm/04_control_flow/006_is_stmt 即新语法）——任何全新 worktree 的 `cargo tb book_ch*` 必红；主检出单跑通过属其 target 增量构建本地假象（`cargo clean -p auto-lang` 后复测仍过，其增量状态未深究；判定依据=基线提交 b34852532 全新 worktree 复现同红，probe worktree 已 guard+清理）。修复归属 book 仓（listing 再生成或语法升级），auto-lang 侧不动他仓。复审实现会话独立性局限已声明（结论由工件重建：HEAD scoped 复测绿 + 大宗门禁红逐名对照零新增）。 | docs/plans/reports/727-transfer-verification.md §3；727 §9 复审记录 |

## PLAN-728 执行登记（2026-10-02，work @plan-728-dev 3bc929ca0/c19fec79f）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P728-D1 | medium | 分页 rope 行查询的页内换行重扫 | `line()`/`line_start_byte()` 每次调用对其所在页做 O(页) 换行扫描（64KB 页 ~千行级重扫/次）——`find_next` 全文档扫描在 50MB 档实测 ~30s（debug）/answer 线 3ms（release 暖缓存后）。优化方向=页级换行偏移索引（PageDesc 增量数组）或行游标缓存；触发条件=下游 1GB E2E 消费件裁定优先级（头窗口内打字/跳转不受影响）。 | rope.rs q_line_start_byte Chunk 臂；SD-01 帧域协同节 |
| P728-D2 | low | 703 对齐行走字节基修正的回归面 | 顺手修复：`range_content_equal`/`collect_prune_spans` 的 leaf 比较从 str 切片改字节切片（对齐切割在对侧内容上可落字符中间位——纯 ASCII 语料 703 期从未踩中，分块树形态分歧后确定性触发 panic）。语义不变（字节精确比较），但 703 既有测试族覆盖的多为 ASCII/对齐友好语料——非 ASCII 分歧对拍面靠本件 dual_track/paged_snapshots 补位。 | rope.rs LeafView::bytes 注记；file_backing::tests |
| P728-D3 | low | >50MB 档交互面窗口化边界 | 头窗口（2MB）外交互体验边界如实登记：find 匹配落点重物化窗口后 published 光标行=窗口相对坐标（模型绑定下游需知悉）；fold 面在 >50MB 档返回空图（折叠发现=全文档扫描，护栏早退）；`code_editor_text` 全量读出在 >50MB 档仍 O(n) 物化（Plan 413 事件面消费——下游适配件改走 doc_snapshot）。三项全部=auto-edit 消费件范围（供料档 §10 回执预告位在列）。 | SD-01 双轨形态节/回执节；auto-edit de6cb95 |

## PLAN-729 执行登记（2026-10-02，work @plan-729-dev，R1 复审修复后）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P729-D1 | low | shutdown drain 专项 wire 演练 | 文件 body 在优雅关停排水窗内的收尾未做专项 wire 测试（scope 收口臂与 SSE FrameStream 共用路径已由共用代码覆盖；drain 窗语义沿 serve_with 现状）。补测需以 test_trigger_shutdown 驱动 + 慢 body fixture。 | http_transport.rs serve_file_seed/serve_with drain；729 lifecycle 报告 §3 |
| P729-D2 | low | legacy stdnet 文件 500 诊断 wire 档 | serve_blocking_stdnet 的文件端点 500 诊断为代码面实现（fn_is_api_file_return 门 + 诊断写回）；legacy 为非默认调用图（plan705 spike 锁单调用点），无独立 wire 档。补测需手工 VM 构建管线。 | http_server.rs legacy 分支；SD-01 §6 矩阵注 |
| P729-D3 | low | 端口可重绑专项 | e2e 服务器线程为 detached（既有基建约束，与 http_e2e 全族一致）；"结束可重新绑定端口"未做专项断言（每测试固定新端口规避）。 | plan729 e2e 基建注 |
| P729-D4 | low | 排队取消的直接观测 | 文件排队项无外部取消 API：客户端 socket 关闭经 scope 级联、或 30s 准备期限到期出队（两者均 e2e 覆盖）；"取消排队即移除"以间接形态保证（lifecycle §3.2 如实记录）。 | http_file_service.rs 排队臂 |
| P729-D5 | low | 权限拒绝 wire 级 | PermissionDenied→403 映射已对齐计划 §5.3（R1 G-10 修正）；Windows ACL 类权限场景的 wire 测试环境不稳（只读文件对读打开不生效等），未设 wire 断言——映射由单元分类与 SD-01 行承载。 | http_file_service.rs OpenError::Escape(permission) |


### P733（2026-10-02，VM fn 调用语义计划复审登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P733-R1 | medium | 测试基础设施 | **plan502_m3_layout_geometry_e2e 并行负载 flake（双树同发预存）**——全量并行档下 `handler_FlowDiagram_Init` 被 CPU-slice Parked(CpuRunnable)（PLAN-711 片预算负载下耗尽→测试驱动轮次内未续完→几何断言落 x=108 旧值）；单跑恒绿（本树 4/4+master 3/3）、全量约 2/4~2/5 闪红（worktree 与 master 3053f1fdf 同发同错，非 PLAN-733 回归——其 diff 零触 diagram/layout/parking 面）。清偿方向：测试驱动轮次预算按 parked 续跑自适应或标记 load-flake 重试档 | crates/auto-lang/src/tests/plan502_diagram_tests.rs:284（td st/ck 中轴断言）；复审记录 F-1（master 全量对照 5 采样 2 红） |
| P733-R2 | low | VM 视图求值器 | **musk_vm_track p053_1/p053_4/p053_6/p054 预存红族=fn 调用语义同族深层腿**（master 3053f1fdf 基线即红，PLAN-733 三腿修复前后零行为变化/同错）——computed 链式 helper 断言面（musk chats 消息列表 computed 形态）残腿指向子件 override 态 `.store.X` 解析链深层；PLAN-733 边界登记于 design/vm-fn-call-semantics.md 已知边界③。**PLAN-749（2026-10-10）甄别：与条件通道异根**；**PLAN-748 T-02 首腿（2026-10-10，2e2164b5e 待合）分解定音**：六红三根——①p053_1×2+p053_6=`.store.X` 泛名展平空快照拒展平（三套 .store 口径不一致）**已翻绿**（musk_vm_track 97/6→100/3）；②p054×2 图标子件渲染面**归 748 T-02 续腿已修**（2026-10-10，3ec6a4914：PLAN-625 T-09(b) `tag.contains("icon")` 粗闸误判 PascalCase lucide 名→占位卡；判别器改 lucide 名集成员资格 `is_icon_component_tag`，AppViewport ∉ 名集保持占位卡意图；musk_vm_track 102/103）；③p053_4 merged `#[api]` CALL_NAT 发射**异根**（codegen 面，独立债务观察项） | crates/auto-lang/src/tests/musk_vm_track_tests.rs:563/628/675；docs/specs/auto-lang/vm/design/vm-fn-call-semantics.md 已知边界节；PLAN-749 §9 T-00 定音报告 |

### P735 批量回归登记（2026-10-03，merge 后到期档 tf/tt/tb@68a6ce919）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P735-R1 | medium | http 上传服务（PLAN-730 域） | **plan730_commit_target_matrix 确定性红（scoped 同红 3/3：tf 全档+tf fail-fast 首跑+单测 scoped）**——30.1s 恒定时长=内部 lease/deadline watchdog 到期形态；本批窗口（6988b1d7f..HEAD）代码面仅 PLAN-735 帧通道四文件（ui::frame_bench/frame_segments/iced::frame_probe+renderer 观测订阅——与 http_upload_service 零交集，无因果路径），同晨批量回执（covered 6988b1d7f 含 730 全码）报零新增确定性红→**环境/时序敏感测试（730 自己的 slow-upload/lease watchdog 族先例 38s 跨期限）在当前机时下翻红**，非 735 回归。清偿归属=PLAN-730 域（其 734 会话在途）：deadline 断言对机时负载的鲁棒化或超时预算放大 | http_upload_service.rs plan730_commit_target_matrix（30s 恒时长）；本仓 .tmp-regress.log 三腿实录（收据 JSON new_reds） |

## PLAN-737 执行登记（2026-10-03，work @lang-737，跨仓缺陷单 jade PLAN-037 T-02）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P737-D1 | medium | MCP HTTP current-thread runtime 结构弱点 | MCP 服务器=`new_current_thread()` 单 tokio 线程；737 已把工具执行下沉 `spawn_blocking`（handler 同步阻塞曾冻结 accept/IO 线程=探针连发 ECONNRESET 放大器），但 `note_activity` 等仍在 runtime 线程持锁；残余偶发连接复位（jade 侧旧 binary 亦两见=非 737 引入；737 修复后全量探针 4 跑 1 见）。疑点=多 app 顺序 runs 的 keep-alive/端口生命周期；需服务器侧 stderr 捕获 + AUTO_SCHED_DIAG 定向排查（复位时 runtime 线程是否存活）。 | mcp_server.rs run()/mcp_http_handler；p737 回执 §5 |
| P737-D2 | low | autodown 编辑壳键域双形态面 | storage key 归一形（`__autodown_editor_*`）与裸键在 MCP 载荷/装配注册两侧并存，靠 `normalize_payload_key` 幂等归一桥接（737 探针 MISS 实录后补）。后续新增 keyed 面应统一走该归一口，避免再分叉。 | autodown_editor/core.rs normalize_payload_key；renderer.rs ade_link_dispatch_message |

## PLAN-739 执行登记（2026-10-03，work @lang-739，跨仓缺陷单 jade PLAN-037 F-037-R1）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P739-F1 | medium | auto-cache 测试目标仓内腐坏 | `cargo check -p auto-cache --tests` E0063：methods_pack.rs:697 测试构造 `ShimMethod` 缺 `trait_name` 字段（PLAN-596 T-03 给 shim-metadata/types.rs 加字段未同步该测试）。**日常档门禁只建 `-p auto-lang` 目标（cargo t 别名 --lib + 三 integration test），auto-cache 自身 lib 测试从不参与编译，腐坏不可见**——master@9fcbdb5de 实证同破，非 739 引入。清偿=补 `trait_name: None`（或 Default 展开）+ 评估是否需要把 workspace 兄弟 crate 的 lib-test 编译纳入某档抽样（防再腐坏）。 | crates/auto-cache/src/methods_pack.rs:697；crates/shim-metadata/src/types.rs:109 |

## PLAN-740 执行登记（2026-10-08，work @lang-740）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P740-D1 | medium | VM app 启动健壮性 | `.am/state.at`（app 侧持久化旁路文件，gitignored）被杀进程写半截时残留 NUL 填充 → 下次 boot lexer panic（parser.rs:404 `lexer should produce first token`）**进程起不来**（2026-10-08 实录，删文件即恢复）。容错面（解析失败→忽略重建）属 app 健壮性域，非 740 范围 | 041 例 `.am/state.at` 23×NUL 实录 + handoff-v3 §3；复现=ladder/复判跑中途 kill 后再 boot |
| P740-D2 | low | 执行环境 | 本机 2026-10-08 实测 app 进程偶发静默退出（无 panic 痕迹，rc=0/1 混合，改前改后二进制均现，手工短复现不复现；同机他方会话重负载嫌疑）——谱采集协议已按「失败档重跑」消化；复判件需同款重试纪律 | handoff-v3 §3；740 执行实录 3/12 跑触发 |

## PLAN-736 复审登记（2026-10-08，review pass @6156c3de3）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P736-R1 | low | auto-man 服务进程托管 | `serve_rust` 父进程不 health 轮询子进程、不消费 ready 身份体做核对（代码注释自标"T-05 接管"未落地；T-05 执行记录"父进程核对同源"为过述）。AC-02 安全结论由 734 strict 门 + 占口 bind 失败非零 + 503-not-ready 探针承载，不受影响；父侧身份体核对列为可选增强（防子进程打印与监听者错位类的纵深防御） | crates/auto-man/src/http_service.rs:245-257；复审记录 R1 |
| P736-R2 | low | http_transport stderr 日志 | 736 新增 eprintln 臂（413/超时等，http_transport.rs:416-432/476/506/556-559）打原始 method+path（可含 query 串）到 stderr——在 JSONL sink 不脱敏合同之外，但与 plan736 §5.4 精神不一致；后续统一脱敏或改道 sink | crates/auto-lang/src/vm/ffi/http_transport.rs 上述行 |
| P736-R3 | low | 生成轨观测/响应头一致性 | 生成轨无显式 "startup" JSONL 事件（仅 state:ready，VM 轨有显式 startup）；生成轨 429 用 lowercase `retry-after`（合法但两轨风格不一）；main.rs:2842 重复 `#[cfg(test)]` 属性（无害） | api_gen.rs SERVICE_RUN_LOOP 模板；crates/auto/src/main.rs:2842 |
| P736-R4 | medium | auto-man 测试目标腐坏（预存，非 736 引入） | `cargo test -p auto-man` 全 targets 编译失败：bin auto-man E0601（空 main）+ tests/plan734_commands_fixture.rs E0255/E0425/E0433（未生成 api 夹具）——master bef73f52f **逐项同破**；日常档门禁不建这些目标故不可见（同 P739-F1 模式）。清偿=退役/修复 bin main + 夹具改可编译桩或移出默认 targets | crates/auto-man/tests/plan734_commands_fixture.rs；master 对照同错（复审实测） |
| P736-R5 | low | 执行环境 | 698 域 SSE 订阅会话首播后 ~1-2s 自然收口（帧竞速观察项，连续流不受影响）；VM resources RSS 单调缓升（59.6MiB，远低于 512 阈值，6/10 样本）——批量回归档复核 | reports/736-http-load.md 观察节；736-http-proxy.md 支持面边界③ |

## PLAN-738 执行登记（2026-10-08，work @lang-738，T-04 裁决遗留 + T-05/T-06 实勘）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P738-D1 | medium | 生产 native 注册面 ID 相撞真冲突（T-04 遗留 b 分诊裁决） | 扫描面动态 id 相撞 13 组（id1607 char×conv、id9930-9933 http×transfer 等）冻结基线（t04 `id_alias_conflict_detection`）。T-06 分诊裁决：`auto stdlib inspect --check` 按模块前缀把冲突组归入受影响模块**如实上报非零**（check_http_reports_frozen_id_conflicts 冻结），不静默、不弱化；重编号=ABI 变更，归 D3b/后续计划 | validate.rs id_alias_conflict_groups；crates/auto/src/cmd_stdlib.rs check_core；plan738 27/27 基线 |
| P738-D2 | medium | 核心模块 id 面分裂（T-04 遗留 c 同族扩展） | json 实勘（T-06 探针）：生产绑定面=catalog canonical 名（`auto.json.get`@1906，register_std_shims 实绑，生产 json 可用）；校验器问的公共扫描名面（`auto.json.json_get`@99xx）无 shim 可调——诚实判 Unverified（check_json_scan_face_unverified_nonzero 冻结）。与 http 扫描前缀名（裁决①）、io 第四绑定面（VmModule 方法表，遗留 a）同族：**公共符号 canonical 面全量重写**为统一清偿路径（下轮） | vm/native_catalog.rs 19xx 段；validate.rs canonical_native_name；cmd_stdlib.rs 实勘注记 |

## PLAN-738 补充复审 R5（2026-10-10，needs_fix @9c255993b）

本节为 revision 3 **未满足原验收的阻塞项**，不构成批准延期或 D3b 豁免。R4 pass 保留历史，本轮由实际反例回退 executing，详见 [738-review-r5.md](reports/738-review-r5.md)。

| id | 级别 | 领域 | 必要修复 | 锚点 |
|---|---|---|---|---|
| P738-R5-01 | high | 生成依赖失效 | 收据 lock=absent 无结束条件：首次 lock 物化和随后依赖版本变更均被实际新鲜度门接受。首次成功构建后绑定真实 lock/features，后续严格比较；缺失/读取失败闭合拒绝，补完整状态序列反例 | rust_ui.rs:4022-4042；api_gen.rs:1071-1089；R5 正式生成/门反例；AC-06/07，T-12 |
| P738-R5-02 | high | 六核心 receiver strict | 公开 JsonValue.len 的返回类型 int→str 漂移后，实际 CLI actual/check 仍 exit 0，proof 只有 parse。collector 不能对局部 receiver 直接跳过；补实际方法来源/producer/adapter 证明或拒绝，不能以 P738-D2 转 D3b | trans/rust.rs:5561-5586；json.at JsonValue.len；R5 CLI 反例；AC-03/07，T-11 |
| P738-R5-03 | medium | 真实消费者验收 | 三角测试把 CLI manifest clone 当生成消费者并丢弃 Rust 转译 Result，未覆盖普通/persistent VM 与 C。保留局部身份测试，另补真实 CLI/会话/产物/正式生成收据/ready 的共同身份与依赖变更对拍 | plan738_stdlib_assembly_tests.rs:1640-1702；AC-02/06/07，T-12 |

R6 复核（2026-10-10，@`68398d2d6`）：P738-R5-01/R5-03 **闭合确证**（一次绑定状态机+真实三目标三角/生成腿，[738-review-r6.md](reports/738-review-r6.md)）；P738-R5-02 **部分闭合**——len 形态闭合，但 receiver 公共方法面其余 11 方法（keys/is_null/as_int 等，含 is_null 可编译旁路）漂移仍 exit 0 无 proof，续期为 **P738-R6-01**（同锚点，修复要求见 R6 报告 §2.3）。

## PLAN-738 独立复审 R1（2026-10-09，needs_fix @e4425b673）

本节为本期**未满足验收的阻塞项**，不是批准延期。P738-D1/D2 的六核心部分由 R1/R2 收回，不能只登记为 D3b 后勾全。原计划已回 executing；详细修复要求、逐 AC、复现源码及门禁见 [独立复审报告](reports/738-independent-review.md)。

| id | 级别 | 领域 | 必要修复 | 锚点 |
|---|---|---|---|---|
| P738-R1 | high | 生产 strict/冲突 | 校验器未进入 compile/codegen/persistent/trans；双活 body resolve_uses 返回 Ok。生产闭包拒缺失/冲突/未核验/引用 unsupported，注册前防 ID 错绑；收紧 duplicate 测试的成功放行臂 | compile.rs:1245；plan738 测试:1466；review R1 |
| P738-R2 | high | 公共身份/独立签名 | tcp_listener_close 参数1改999仍 Supported+SignatureChecked。补六核心公共→真实callee映射及独立 producer 参数/返回/self/async/feature/stub 元数据；io第四面/json canonical 不得推到全库重写 | validate.rs:135/174；model.rs:128；review R2 |
| P738-R3 | high | epoch/来源缓存 | 同会话改层仍 old_layer；CWD B 同名模块仍用A缓存。先解析来源再查cache，按epoch/来源身份及递归依赖闭包失效 | compile.rs:1245/1284；module_cache.rs:348；review probe |
| P738-R4 | high | 装配指纹/ready | vm层改名rs层指纹不变；缺provider/host内容与环境features；receipt null+null 当fresh。绑定实际闭包与provider实现身份，依赖不可核验必须拒绝 | loader.rs:49；api_gen.rs:1039；rust_ui.rs:4015 |
| P738-R5 | high | 生产目标装配 | Rust/C实际trans无manifest，无共同AssemblyPlan；persistent仍单独根与后缀；Rust net发射不存在模块也Ok。真实消费者接线，共用来源与不可变manifest | lib.rs:5943/6020；autovm_persistent.rs:258；review R5 |
| P738-R6 | high | CLI严格语义 | rust net --check假pass；async公共parse失败假pass；jsonx actual检查无关六核心72违规。修target/env/实际闭包/parse/module归一及便携源ID | cmd_stdlib.rs:212/314；review CLI反例 |
| P738-R7 | medium | 全分母/环境矩阵 | 字段与读失败层缺条目、WalkDir错误被吞；VM矩阵用目标helper分母，Rust/C无两环境格，native绑定不能证明browser adapter | loader.rs:82/225；validate.rs:330；review R7 |
| P738-R8 | medium | 源错误定位 | 只有加目标文件名的Msg，没有source span/行列/双源位置；失败路径再次parse会写共享TypeStore | compile.rs:1650；review SOURCE_ERROR |
| P738-R9 | high / medium | 真入口证据/健康/SD | C支持样例实编与generated API→service→ready未执行，不能用C HTTP缺provider或普通Json.parse witness替代；最终SD未核验。新增model.rs rustfmt三处diff必修 | verification报告；model.rs:106/196/203；review R9 |

门禁完整重跑：t 14红均在已有批回执 known_reds；tt新增观测 back_provision红/dep_fields超时 scoped复跑均绿，归因未充分确认，暂不列本期确定性修复；th两back_proxy红见执行报告基线。本轮未新跑baseline worktree同命令，不过述“零新增红已双树证明”。

### R1 修复进展（2026-10-09，work needs_replan @4fbcfe968→4089b8418）

基础修复与剩余阻塞逐项对账见 [738-repair-round1.md](reports/738-repair-round1.md)；R1..R9 继续作为本期必要验收，**尚未整体清偿**。历史 P738-D1 的动态扫描 ID 冲突当前不再复现（分配避开 catalog 保留 ID，独立别名冲突检测为 0）；不同 callee 争同 ID 的实际生产门仍须补齐。P738-D2 的 json canonical/io VmModule 查询已接线，独立完整签名与最终活 callee snapshot 仍未闭合，不再采用“下轮公共面重写”作为当前延期许可。

本轮特定反例已修：双实现宽松放行、arity 数量自证、同会话源变更、clone 跨 root、深依赖缓存、层改名指纹不变、null/null fresh、Rust net CLI 假绿、actual jsonx 无关六核心假红、读失败丢层和二次 parse 猜源。生产 strict 门/真实 emitter/完整公共矩阵/C 与 generated-service 见证仍未完成。已证伪原 init 三件套等同最终 callee 的设计假设；TCP read 公开 buffer/单返回与 VM size/双输出的 adapter 合同待有界修订，不能伪造 SignatureChecked。计划保持 executing；具体次序和依据见修复报告，不降低验收。

另保留 HTTP 观察：full th 100/102；back_proxy routes 同 R1 断言，plan707 timed frames 在本轮两次定向复跑仍红（撤回新增 callee 排序/首个 producer 选择之后也红）。原 R1 log 该例通过；既有时序族 flake 记录不足以证明本次原因，未做新基线对照，不能宣称零新增红。最终 scoped 54/54 与三 crate check 通过；详细版本绑定见修复收据。

## P747-D1（2026-10-09，PLAN-747 复审登记）：无审计行的静默崩溃族未归因

WER 链中 09-12/09-22/09-29 的 AppCrash（c0000409/FastFail 0x7）当刻 `exit-audit.log`
无对应 panic 行——非 panic 路径，疑栈溢出（fastfail 且无消息）。PLAN-747 已消除
「caught panic → 打印失败 → panic-in-panic → abort」放大链（focus_pending 风暴根修 +
容错打印），但该静默族若无 panic 前导则不受此修覆盖。观察条件：再次出现 auto.exe
fastfail 崩溃且审计无行时，优先按栈溢出排查（重建符号 + RUST_MIN_STACK/主线程栈深，
嫌疑面：大树递归 Drop / vtree 递归转换）。证据锚：`AppCrash_auto.exe_*` 09-12/09-22×2/
09-29×2 五份 ReportArchive；PLAN-747 §需求分析的时间线对照。

## PLAN-748 执行登记（2026-10-10，work @lang-748）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P748-D1 | low | 测试基础设施 | **plan051_p2 模块级图标测试空洞绿**——`locate_corpus` 三候选路径（CARGO_MANIFEST_DIR 相对 / CWD 相对 / ../../ 相对）在 nextest 进程 CWD（crate 根）下全 miss `test/ui/plan051_p2_modules/pac.at`（语料实存仓根 test/ui/），`build_component_from_app` 返回 None → SKIPPED 分支静默 pass——该测试自落地起未真正跑过图标臂（PLAN-748 T-02 续腿定音插桩实证：全程零触 imported 分派臂）。清偿方向：locate_corpus 补仓根锚定（env!("CARGO_MANIFEST_DIR") 上溯两级）或断言 corpus 必达（miss 即 fail），防 SKIPPED 假绿 | crates/auto-lang/src/tests/plan051_p2_tests.rs:22（locate_corpus）/ :60（SKIPPED 分支） |
