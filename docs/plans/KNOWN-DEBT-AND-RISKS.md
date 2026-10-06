## PLAN-741 复审批（2026-10-04，plan-741-dev @ 4dc4da636 同基线对照裁定——零新增债务，仅归档对照证据）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P741-R1 | info | 日常档红对照 | **PLAN-741 复审 cargo t 26 红裁定为非本回归**——worktree 基线 951b6c70f 上 `cargo t --no-fail-fast` = 4945 绿/26 红/1533 skip；同基线 scoped 对照（主检出 0d7c7f024，crates 与基线一致；PLAN-741 零 crates diff）：14 例同红（musk p053×3/p054×2、plan707 sse、plan502_m3、plan498_area、plan484_024、ffi_dual_017、dep_parity_017、projector_counter、plan606 gallery_029——**全部落入 THR-D3 既有清偿族**）+ 12 例 scoped 串行绿（并行负载 flake，与 THR-D4 同模式）+ 4 例 filter 未命中（schema_drift/docs_gen fence、ash_leak_probe 环境族）。无新增挂账项；本行为跨仓跟进类红的又一同基线实证样点 | PLAN-741 §9 复审记录；docs/reports/741-ac-hir-native/verification.md §4.1（入库后可解析）；THR-D3/THR-D4 |

## 执行吞吐治理第二批挂账（2026-10-02，PLAN-726 立项分批裁定：第一批四项见 726，余量在此）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| THR-D1 | medium | 构建面 | **crate 拆分前置测量**——单 crate 76.6 万行/1,650 文件（2026-10-02 实测），Rust crate 粒度增量下任何改动触发整 crate 重编+测试二进制重链；先测量冷 worktree check/链接时间占比（样点：sccache 79% 命中下 `cargo check -p auto-lang` 57s、lang-724 target 17G、non-cacheable 801 次），数据支撑再决定 workspace member 拆分立项——不测不做 | .cargo/config.toml jobs=12；sccache --show-stats；2026-10-02 会话实测样点 |
| THR-D2 | medium | 构建面 | **共享 target/链接缓存评估**——REG-3 已实证共享 target 候选撞名红（auto-edit.exe 撞名翻转策展测试），评估带产物命名空间隔离的跨 worktree 共享方案；未评估不实施 | 本文件 REG-3；clean-autolang-targets.cmd |
| THR-D3 | medium | 验证信号 | **master 14 红族清偿**（musk p053×4/p054×2/plan606/projector/plan707/e4→PLAN-726 承接/app_registry×2/desktop_protocol/iced renderer/a2vue 金样/plan484 streaming）——跨仓跨域，逐族立项；目标=验证信号零噪音、复审零定案税 | 711 §10② 红册；PLAN-715 批收据基线 |
| THR-D4 | low | flake 族 | **ffi_dual_019 / plan502_m3 并行负载序敏感 flake**——批量档观察名单积累复现率，频率上升再立项查并发交互 | 本文件 REG-2；PLAN-715 登记 |
| THR-D5 | low | release 档 | **release 档红册治理**（~254 环境红+tests/ 编译 rot 残余）——非维护门但偶发消费（P712-D4 陈旧产物观察） | P702-D2；release_full3.log 分诊 |
| THR-D6 | low | 走查基建 | **实机走查脚本沉淀**——主检出 `.tmp-vm-*` 一次性脚本族（20+ 件未跟踪）收编入 `.agents/skills/autoui-verifier/scripts/` 标准件（AGENTS.md 既有要求），消除每次走查重写成本 | 主检出 .tmp-vm-*；autoui-verifier 技能 |
| THR-D7 | low | 术语残件 | **725 计划落定后其文档内残存 10 处「谳」→「案」清理**（2026-10-02 出清批遗留——当时该文件由并行起草会话持有未动） | docs/plans/725-typed-frame-incremental-pipeline.md |

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

### P706 r2 收口批（2026-09-30，work 阶段定案的非本回归缺陷另立记账）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| ~~P707-R1~~ | high | async http 直驱臂（e4） | **已清偿（2026-10-02，PLAN-724 T-02）**——真因勘定=**测试侧违反 managed 提交协议**（非派发缺陷、非 detached 桥分离）：e4 三臂直接 `spawn_async_http_handle`/`spawn_async_http` 提交 managed job（999_001/999_101/999_102）却未先 `register_live_op`，707 `submit_client_job` 的 cancel-before-install 闭合（still-live 复查）在 spawn→登记窗口内发现令牌缺席 → 立即 abort 刚提交的任务 → 请求恒不到线 → 30s `recv_timeout` 恒红。生产 shim 全部先 register（stdlib.rs 5367/5390/5413/5434 等在案）。修复=测试改走生产协议：register→submit→wire 断言→消费配对（plain 臂增验 managed 完成落表 `Structured{status:200}`）+`cancel_live_op` 幂等清理，并钉住未登记反例（abort 句柄 submit 返回前回收 + 无令牌永无可消费结果）。取消守卫零改动；plan712（2/2）与 plan707_cancel（5/5）复证绿；修复后单跑 0.46s。公开入口覆盖保持：直驱两点即公开 native shim 的唯一 send 汇聚处，默认头/query/body wire 断言原样 | stdlib.rs e4 模块（default_headers 三臂+未登记反例臂）；`cargo t default_headers_reach_wire_on_plain_get` 绿日志（work 阶段在案） |
| P706-D1 | medium | back-bridge call（merged 直调臂） | **probe_mtime merged 臂 release 载体 `done` 恒 false（挂起）为独立预存缺陷**——探针 widget 零 computed/memo，Init 弧 `probe_cases` back-bridge call 不完成；A/B 定案：old-carrier c8f86ef92（pre-delta）release **同形红**（`done: false` 超时），修复 exe debug 全案绿 → release 特异的 back-bridge 调用面，非 706/plan047/705 delta 引入。清偿需 release 档 CALL/段驱动时序定位（debug canary 先爆掩蔽同窗） | jade-edit tests/probe_mtime.mjs；`p706-probe-mtime-rel.log` vs `p706-probe-mtime-oldrel.log` vs `p706-probe-mtime-dbg.log`（组目录 .wt/lang-706/） |
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
| P721-R1 | high | T-19 读侧机理（暂停回弹根因末环） | 下行 epoch 单调门只挡「世代回退」子类；桌面实测回弹写来自 **OnTime 触发的全新构建烤回 paused=false**（handler 两次写均 false、同一 binding 相邻构建读值不稳定）——读侧世界分裂/陈旧 memo 回放机理待定案。复现即现形：`AUTO_SCHED_DIAG=1` 桌面跑 030 点暂停看 `video_build` 行。 | evidence/721/apptick-verdict.md 结论二；desktop-721g.log |
| P721-R2 | medium | T-11 播控条布局塌缩 | col_right 438↔658 进程级翻转非确定性源未定案（712-r2 登记态原样移交）。定案入口=进程内双构建比对 + builder 侧 HashMap 遍历审计（style 层 Vec 面已初查非嫌疑）。 | 归档 712 §10①；layout_tests.rs p712 探针族 |
| P721-R3 | medium | 030 seek 派发被 arity 守卫跳过 | 用户实机取证（030-standalone4.log 07:15）：进度条点击/拖拽 → `[VM-ARITY] Controls.SeekFraction declares 1 param(s) but dispatch supplies 2 — skipping` ×6——seek 完全失效。疑拖拽事件路径多带一参（712 T-12 `$0` 剥离同域）。 | 030 controls.at SeekFraction 绑定；plan-576 D4 守卫 |
| P721-R4 | medium | 验收通道注入静默丢 | desktop-721i 实例 handler inject 排队后零派发（g/h 同代码正常）——registry_id 窗口反查在窗重用/多实例下疑 miss；inject 臂 `let _` 吞错无日志面。验收通道可靠性债。 | renderer.rs apply_desktop_injects Handler 臂 |
| P721-R5 | low | 独立窗 ok=true 自退（seek 后） | 030 独立轨在用户 seek 交互后 `vm interpreter returned ok=true` 优雅退出（无 wgpu 告警，非 712 T-07 press 死因；用户主动关窗不可排除）。复现则按 T-07 同窗定案。 | 030-standalone4.log 尾段 |
| P721-R6 | medium | T-19 桌面活体 ×10 复验 | epoch 门落地后桌面实机暂停 ×10 无回弹未走查（实例 I 用户占用+R4 注入丢+712 T-07 三重阻塞）。移交用户日常观察；若仍回弹=R1 机理现形入口。 | evidence/721/apptick-verdict.md 未竟段 |

### PLAN-712 r3 批（2026-10-01，归档收据 eed6acf25——未竟/登记项集中挂账）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P712-D1 | high | 桌面轨更新路径家族（PLAN-721 承接中） | **推进中（2026-10-01，PLAN-721，worktree plan-721-dev @088b32aa6）**：①**T-16 结案**——r2「桌面内嵌轨 0 trace/Loading 卡死」定案为伪象或已被 712-r3 治愈（sched_diag 四轴 trace 全绿：sub_parked/到达/恢复 50.9ms/置脏回填，018 内嵌实机复验）；②**T-19 帧级序列捕获**：暂停写全链落盘（mpv 真停+上行回灌）后 ~100-185ms，OnTime 触发的**全新视图构建烤回 paused=false**（同一 binding 相邻构建读值不稳定=读侧世界分裂/陈旧 memo 回放，机理开放；video_build trace 已就位）；③**已落部分修复**=下行世代单调门（VideoContractDown.epoch + contract apply 回退整包拒绝），mpv_contract 13/13 绿（712-r3 双锁零扰动），独立轨真实按钮暂停钉住 7s+ 零回弹；④**T-11 未动**（col_right 438↔658 定案入口不变）。定案文档 docs/plans/evidence/721/apptick-verdict.md（worktree 088b32aa6）。另：验收通道 handler inject 在 desktop-721i 实例静默丢（排队零派发，g/h 正常）——验收通道可靠性债，随 PLAN-721 待澄清②复查。 | 归档 plan docs/plans/archive/712-vm-desktop-defects.md §8.5.3/§9/§10①；PLAN-721 plan 文档 |
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

## PLAN-741 交付后质量复审（2026-10-04）

复审基线c1ac219e7，结论needs_fix。~~均未清偿~~ **已全部清偿（2026-10-05，PLAN-741 r2 Phase 2）**：
用户裁定不另开合同，激活741以plan_revision 2追加Phase 2修复；独立代理复审pass
（44/44测试、六反例全翻转），交付合入v0.6-dev@52c67d3df。清偿证据：
[741归档](741-ac-hir-native-core.md) §9 Phase 2记录；
原始报告：[复审报告](../reports/741-quality-review-20261004/REVIEW.md)。

| id | 级别 | 问题与实际证据 | 修复方向 |
|---|---|---|---|
| P741-QA-01 | P1 | bool icmp结果i8与local/ABI i32冲突；check绿而build panic101或后端拒绝 | 统一bool表示并补绑定/参数/返回原生测试 |；**已清偿**（r2 T-09 icmp_bool 唯一出口uextend归一+bool原生正例（bool-if exit3/bool-abi exit5））
| P741-QA-02 | P1 | 按eval_args索引校验而非bindings映射；异类型合法调用被拒、非法调用获Checked | 按映射检查类型，保留求值顺序 |；**已清偿**（r2 T-10 bindings映射制类型核对+双射校验（binding-swap原生exit9/swap类型不配check拒绝））
| P741-QA-03 | P1 | 两函数可共享异签名body且check绿；native参数索引panic101 | function/body双向owner及唯一归属检查 |；**已清偿**（r2 T-11 function↔body双向归属（shared-body-owner/same-sig/stolen全verify.owner-mismatch拒绝））
| P741-QA-04 | P1 | receipt写失败exit1，exe已更新、obj仍旧 | 制品准备/发布事务与失败回滚，补发布失败测试 |；**已清偿**（r2 T-12 暂存-备份-发布-回滚事务（发布失败exe/obj哈希不变+旧exe实跑；回滚测试还实抓并修复同stem备份名碰撞））
| P741-QA-05 | P2 | README多传ac-probe且示例无d_entry；真实exit2/entry.not-found，归档链接陈旧 | 统一命令/命名、可运行fixture和链接，复制运行验证 |；**已清偿**（r2 T-13 README真实bin名/带入口fixture/归档链接/schema笔误（三命令逐字复制运行全过））
| P741-QA-06 | P2 | rustc/reg发现直接output无deadline；执行器先等退出再排pipe | 有截止的统一执行器与并发输出读取 |；**已清偿**（r2 T-14 全子进程硬截止+并发管道读（1s截止杀挂起helper、>126KB输出不阻塞单测））
| P741-QA-07 | P3 | tests/common/mod.rs descriptor未使用warning | 清理并覆盖all-targets健康检查 |；**已清偿**（r2 T-15 未用导入清除+verify脚本all-targets零warning门）

## PLAN-741 r2 合入后复核（2026-10-05）

基线965b368a2，needs_fix；旧四项P1常规反例已确认修复，44/44原型测试通过。
~~以下为新增/扩展边界，未清偿~~ **已全部清偿（2026-10-05，PLAN-741 r3 Phase 3）**：
独立代理复审pass（AC-16..20全过、原型52项+60s证据测试、watchdog 60.05s准时拒绝、
cargo t双侧同基线归因零新增），交付合入v0.6-dev@a0b3750c4。清偿证据：
[741计划](741-ac-hir-native-core.md) §9 Phase 3记录；
原报告：[r2复核报告](../reports/741-quality-review-20261005/REVIEW.md)。

| id | 级别 | 观察 | 修复方向 |
|---|---|---|---|
| P741-R2-QA-01 | P1 | 入口块自环check栈溢出0xC00000FD；断开双块循环check0 | 先做完整块图无环/可达性验证，失败后阻止递归走查 |；**已清偿**（r3 T-19 块包含图结构门（显式栈DFS全可达+入边不变量、结构失败阻止走查；复审代理自构3块环/else回边反例全拒、两报告输入定位拒绝））
| P741-R2-QA-02 | P2 | 子进程退出后reader join不受截止约束；持pipe后代反例114.963s仍未返回 | 进程与输出收集共享deadline，补继承pipe反例 |；**已清偿**（r3 T-20 单一单调截止覆盖等待+输出收集（Job Object KILL_ON_JOB_CLOSE 管控子树、reader全出口回收；watchdog实测60.05s准时拒绝 vs r2实测114.96s挂起；read_to_end+lossy修截断成功））
| P741-R2-QA-03 | P2 | 收据暂存写失败遗留已链接.tmp exe；旧exe/obj保持不变 | stage所有权/统一清理覆盖准备/备份/发布失败 |；**已清偿**（r3 T-21 publish暂存所有权全失败出口回收（含已链接exe）+link.restore恢复诊断+占位目录保护；复审helper实测staged_exe_left=false）
| P741-R2-QA-04 | P3 | archive内frontmatter仍reviewed，与merge receipt的archived不一致 | 修正簿记，保留历史与revision |；**已清偿**（r3 T-22 状态/指针簿记（复审方激活+本轮核对一致；最终归档由本merge完成））

2026-10-05 用户明确再激活：[PLAN-741 Phase 3](741-ac-hir-native-core.md)，revision 3、executing。
上述 P741-R2-QA-01..04 由 T-19..T-24 / AC-16..20 跟踪；本次仅合同修订，
未经修复及独立复审不销账；旧 P741-QA-01..07 已修证据与历史记录保留。

## PLAN-743 合入后独立复核（2026-10-05）

基线 fa3abe476，needs_fix；用户授权激活 [PLAN-743 Phase 2](743-acc-bootstrap-hir-contract.md)，
r2/executing。18/18 既有测试通过、旧 F-01/F-03 已修，以下新问题待 T-08..14 / AC-09..14 闭环。
证据：[独立报告](../reports/743-quality-review-20261005/REVIEW.md)，本次不预销账。

| id | 级别 | 观察 | 修复方向 |
|---|---|---|---|
| P743-QA-01 | P2 | 跨行字符串里的 pretend() 被扫为调用；MD-506 认可换行恢复 | 字面量整体屏蔽/行号回归，复核观察和结论 |
| P743-QA-02 | P2 | Meter.len/new、真实 CG.new/Ar.new 被标 native-runtime | 接收者/owner 证据优先，未知保守分类 |
| P743-QA-03 | P2 | 无绑定/缺证据/空决定被 CHECK-OK | provided schema/证据绑定校验+完成态严格门 |
| P743-QA-04 | P2 | 必经路径常量溢出允许变编译期错误，observable/运行 trap 改变 | 保留 runtime trap，澄清编译期语境和 effect 术语 |
| P743-QA-05 | P2 | A2 源码误称 existing，候选①要 A8、候选②缺 A9 前置 | 分 HIR/源码状态，语料与无环能力前置对齐 |
| P743-QA-06 | P2（新鲜度） | 当前8决定/9绑定 stale，漂移拒绝正确 | 逐条语义重核后校准，不 blind hash update/exempt |
| P743-QA-07 | P3 | T-05/T-06 未勾与6/6/已修收据矛盾 | 历史勾选校正，新 Phase 待办/最终归档一致 |


## PLAN-743 r2 合入后复审 / Phase 3（2026-10-05）

基线 5983f8aeedeae2dc5769f9a0820eec4b722d4c6d，needs_fix；[报告](../reports/743-r2-quality-review-20261005/REVIEW.md)。
上节 r1 反例修复已核验（41/41、真实47决定新鲜）；该历史待办描述由本节取代，剩余保证由以下新 ID 跟踪，不把新增缺口抹成已闭环。
用户授权激活 [743 Phase 3](743-acc-bootstrap-hir-contract.md)，r3/executing，T-15..21/AC-15..20；仅合同，本轮不销账。

| ID | 级别 | 观察 | 修复 |
|---|---|---|---|
| P743-R2-QA-01 | P2 | 本地同名 IO/print 仍 native，跨 owner/无 owner 方法升级为 self | 当前 owner/冲突证据优先，unknown 保守 |
| P743-R2-QA-02 | P2 | evidence 未在同条绑定，改变内容严格门仍绿（当前真实决定无此缺口） | 同条证据 hash 覆盖和变化拒绝 |
| P743-R2-QA-03 | P2 | resolved family 删除 decision 仍闭环绿 | 适用决定引用与 open 去向真实校验 |
| P743-R2-QA-04 | P2 | kind/conclusion/bound/evidence 错误类型抛 traceback | 字段/元素类型先验、定位受控拒绝 |
| P743-R2-QA-05 | P2 | canonical R3 旧摘要与新正文冲突，报告 CLI 仍 ac-probe | 新 SD-07 同步摘要/正文，SD-06 工具政策，真实命令对账 |
| P743-R2-QA-06 | P3 | archived 13/14/T-14未勾、模块executing/r2、归档两链接失效/ledger旧指针 | 历史据实勾选，新Phase独立待办，merge核对终态/索引/链接 |


## PLAN-743 r3合入后复审 / Phase4（2026-10-05）

基线8e8f6f19b96516fdbc9d4c893297fa1757a7aad4，needs_fix；[新报告](../reports/743-r3-quality-review-20261005/REVIEW.md)。
上一轮未绑定证据、本地遮蔽/跨owner与canonical摘要/链接等反例已修；63/63和当前真实47决定绿，剩余保证由以下新ID跟踪。
用户授权激活[743 Phase4](743-acc-bootstrap-hir-contract.md)，r4/executing，T-22..27/AC-21..24；本轮仅合同，不修工具，不预销账。

| ID | 级别 | 观察 | 修复 |
|---|---|---|---|
| P743-R3-QA-01 | P2 | 被类型检查淘汰的决定仍入引用索引，evidence=17在族consumer处TypeError | 只消费验证记录/提前受控拒绝，引用组合负例 |
| P743-R3-QA-02 | P2 | subject/hash覆盖可代替evidence覆盖，严格门假绿（真实七族证据完整） | evidence相关性独立且必需，移除OR旁路 |
| P743-R3-QA-03 | P2 | 导入IO/List/File/process和Meter参数IO被标native | qualified宿主准入包含导入/局部绑定冲突，未知保守 |
| P743-R3-QA-04 | P3 | archived20/21、T-21未标完成、P743-3活动指针失效 | 历史补据实完成，新Phase另列；final所有指针/计数统一验收 |

## PLAN-741 r3合入后复审 / Phase4（2026-10-05）

本节新增发现不撤销前两轮已修的具体证据，也不沿用r3“全出口已回收”覆盖新反例。
[报告](../reports/741-r3-quality-review-20261005/REVIEW.md)基线f6a945db8，needs_fix；
依用户授权激活[741 Phase4](archive/741-ac-hir-native-core.md)，r4/executing，T-25..30 / AC-21..24。
~~本轮只复审/合同，不实施、不预销账~~ **已全部清偿（2026-10-05，PLAN-741 r4 Phase 4）**：独立代理复审pass（AC-21..24全过、death-watch反例5/5零泄漏、locked-cleanup全要素、60s生产路径准时拒绝、worktree红集⊂基线零新增），交付合入v0.6-dev@8fdae8c11；canonical SD-07/08已沉淀、live ledger P741-6已投影。清偿证据：[741计划](741-ac-hir-native-core.md) §9 Phase 4记录；原报告：[r3复核报告](../reports/741-r3-quality-review-20261005/REVIEW.md)。

| ID | 级别 | 观察 | 修复/跟踪 |
|---|---|---|---|
| P741-R3-QA-01 | P2 | 公开run_exe约60.106s返回link.deadline后自有后代仍活；时序相关，另次正确回收。先spawn再约束、约束失败静默None/detach有缺口 | 受控启动及失败安全收口/reader回收，T-26，AC-17/21 |；**已清偿**（r4 T-26 受控启动（CREATE_SUSPENDED→入 Job→ToolHelp 恢复，约束前窗口消除）+约束失败受控拒绝（安全收口零泄漏）+reader 全出口回收；death-watch 反例 5/5 零泄漏、public-60s 60.05s 准时拒绝且后代回收）
| P741-R3-QA-02 | P2 | 真实共享锁使owned暂存exe不能remove；收据准备失败只报link.receipt，未列该残留/清理错误，旧三件套不变 | 清理错误/自有路径/准确事务状态汇总，T-27，AC-18/22 |；**已清偿**（r4 T-27 清理错误全量汇总（link.cleanup：原失败+提交状态+残留自有路径+实际 OS 错误；目录=用户占位不删不计）；复审 locked-cleanup 实测 staged_exe_left=true+全保留标志、库内共享锁测试锁定）
| P741-R3-QA-03 | P3 | archived23/24、T-08漏勾与r1/r3收据矛盾 | 根据收据补T-08，重开T-20..24验收，新六任务；最终完成ID计数/链接断言，T-28，AC-23 |；**已清偿**（r4 T-28/merge T-08 据收据补勾、重开 T-20..24 依 r4 任务关闭、最终计数 30/30 与归档断言一致（本轮 merge 收口））

root门禁观察（非新增AC原型缺陷）：daily4971全跑4935通过/35失败/1超时；
超时ffi_dual_018单独2.532s通过。旧THR-D3/D4及阶段报告包含对应族，但本轮未逐个完成35红对照，
不得写成required daily gate绿或“新基线零新增已证实”；正式失败清单已入报告，T-29负责修复后复验/归因。


## PLAN-741 r4合入后复审 / r5 Phase5（2026-10-05，未修复）

绑定HEAD7b9c6948c；[独立报告](../reports/741-r4-quality-review-20261005/REVIEW.md)、[同ID修复合同](741-ac-hir-native-core.md)。r1..r4实施/原债项销账为历史，不删除；本轮54测试/一键13步/公开60s回收通过。

- P741-R4-QA-01 / P2：link_object_staged非零及执行器错误出口吞暂存exe清理失败；真实共享锁OS error32、残留不入诊断，旧成功三件套不变。T32/AC25，SD09兑现SD08，不降为informational。
- P741-R4-QA-02 / P3：archive内4个历史报告链接用active相对路径而失效；T33/AC26，必须最终真实归档门。

尚未实施产品修复。r5 executing/22-of-35，只写证据/合同/导航；canonical/live ledger留pass后merge。P741-3..6旧archive指针激活后暂缺，最终必须全部恢复；不占新号、不碰其它计划工作树。


## PLAN-743 r4合入后复审 → r5 Phase5（复审2026-10-05；2026-10-06补齐落地，未修复）

绑定HEAD c865adf66a75099df07113b053eb9d19e92986a3；[独立报告](../reports/743-r4-quality-review-20261005/REVIEW.md)、[执行合同](743-acc-bootstrap-hir-contract.md)。

- P743-R4-QA-01 / P2：mut/方法及裸调用参数遮蔽漏判，误升级native；T30/AC26。
- P743-R4-QA-02 / P2：合法JSON manifest容器形状错误traceback；T31/AC27。
- P743-R4-QA-03 / P2：非法语义/stale/重复先插入记录仍进入消费者；CLI正确拒绝，无假绿；T29/AC25。
- P743-R4-QA-04 / P3：归档6个报告链接失效，r4断言未检查；T32..33/AC28。

仅复审/合同激活，产品未修复，不宣称销账；83测试、fresh47、旧27/27/ledger真实通过。SD09/canonical/derived ledger留r5独立pass后的merge；激活导致P743-3..6旧archive指针暂缺，最终归档必须全部恢复。

2026-10-06补齐激活文档，当前22/33；原83测试/47fresh为复审基线历史证据，主线另有5条AutoAC Spec过期绑定待T28/T30逐条重审。现有741 r5合同/债项原字节保留，未实施743修复。b'\r\n'  - **已清偿**（r5 T-32 两出口经共享 discard_own_staged + with_staged_cleanup 汇总（link.cleanup：原失败+残留完整路径+OS 错误+NOT committed+恢复办法）；复审 nonzero/timeout 实测 path/cleanup_in_diagnostic=true、os error 32、旧三件套不变、released_cleanup_ok=true）b'\r\n'  - **已清偿**（r5 T-33 计划 7 处链接改 ../../reports（归档终态可解析）+ final_assertions.py 三态门；复审逐条验证 5 目标自 archive 父目录可解析（覆盖原 4 断链超集）；本轮 merge 归档后以 archive 模式复跑 35/35 全勾门收口）


## PLAN-741 r5合入后复审 → r6 Phase6（2026-10-06，待修）

基线6baed9bba84016dd8610221a9a0358195444385e；needs_fix仅P3验收/证据，r5核心P2修复已通过真实当前库nonzero/60s共享锁及58测试/一键13；不重新挂旧核心缺陷。
- P741-R5-QA-01/P3：archive current34/total35但35唯一任务已完成，最终断言不检查metadata，错误/缺数字仍通过。T36/AC28。
- P741-R5-QA-02/P3：active只检查模拟archive，实际父目录7断链仍通过；当前真实archive链接已正确。T36/AC29。
- P741-R5-QA-03/P3：旧反例任意取第一个缓存rlib可选旧库；r5当前摘要54/lib14过期（实际58/lib15）。T37/AC30。明确当前Cargo artifact重放已证实产品修好，旧库结果不得冒充回归。
详见 ../reports/741-r5-quality-review-20261006/REVIEW.md。同741激活r6 current26/total38，未实施修复/未销账；仅最终断言/文档证据范围。canonical/live ledger保持原状态，P741-3..7暂态archive指针留最终归档恢复。


## PLAN-743 r5合入后独立复审 → r6 Phase6（2026-10-06，未修复）

基线9111c21e53ccd9a81554bfd4764c540fdfdcad21；needs_fix，旧110测试/旧四类反例/47fresh/确定性/归档33门实际通过。
- P743-R5-QA-01/P2：额外绑定文件删除后全局stale但记录仍入可信索引；存在/删除/恢复三段式CLI0/1/0，中间族消费者仍21。T35/AC29。
- P743-R5-QA-02/P2：字段类型错误continue绕过去重，同ID坏记录前后两种顺序均保留合法同ID记录供族消费21；字段合法duplicate对照为0。T35/AC29。
- P743-R5-QA-03/P2：已观察普通参数与local type同名，bare仍type-construction、qualified仍type-qualified，违反SD09绑定在类型升级前优先。T36/AC30。
前两CLI受控非零/no traceback，不误报假CHECK-OK；当前真实人工层未因合成反例被宣称失效。全部复现/修复/SD10见../reports/743-r5-quality-review-20261006/REVIEW.md。同ID激活r6 current26/total38，未实施/未销账/未改canonical或live ledger；P743-3..7 archive暂态留最终merge恢复。741/742/ABI及所有历史pass/merge保留。b'\r\n'  - **已清偿**（r6 T-36 final_assertions v2（frontmatter 必需字段/数值解析、current_step==完成唯一数、total_steps==唯一总数合同派生无硬编码、archived=全完成+archive 位置、同 ID 冲突失败）；上线即抓到执行侧 current_step=35≠34 计数错；受控 fixture 8/8 真实退出码）b'\r\n'  - **已清偿**（r6 T-36 active 模式双链接验证（真实父目录 active 形 + 显式转换模拟归档副本，两行输出、任一失败即 exit 1，模拟不冒充实际 active）；复审确认 fixture 复刻缺陷形态正确非零；本轮归档后 archive 模式复跑收口）b'\r\n'  - **已清偿**（r6 T-37 采纳复审 exact-artifact 脚本（Cargo JSON 按 package/target/profile 唯一绑定库+CLI，无 rlib glob，Cargo 输出与哈希落盘）+ 全量重放；计数更正 54→58（lib15+43）带 provenance 入 r5 verification.md 与归档计划）


## PLAN-741 r6合入后复审 → r7 Phase7（2026-10-06，未修复）

基线d1adc03be39d2897f5dfd7090e0909095cf24465；[报告](../reports/741-r6-quality-review-20261006/REVIEW.md)、[同ID合同](741-ac-hir-native-core.md)。核心同源无新故障，17/17独立validator控制通过，不重新打开原型/Rust任务。

| ID | 级别 | 真实缺口 | 修复与验收 |
|---|---|---|---|
| P741-R6-QA-01 | P3 | fixture固定active文件，最终归档后FileNotFoundError；只修输入后T35冲突mutation为零次替换，反例exit0 | T39/AC31，active/archive来源、动态实际翻转、预期诊断与mutation核对 |
| P741-R6-QA-02 | P3 | 两模块741导航仍executing且指向缺失active文件；final_assertions只验Plan/ledger而遗漏README/导航，仍ALL-PASS | T40/41、AC32，最终门明确覆盖README/两模块/全部指针，实际归档后遗漏导航更新负控制非零 |

同741激活r7，current36/total41；未实施/未销账。只重开T36/T38，保留历史pass/merge；不改canonical行为/live ledger，P741-3..8暂态留最终归档恢复。
