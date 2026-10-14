### Plan 773（2026-10-14，独立 final review R1 —— 已闭合）

历史实现已合入，当前独立验收 **needs_fix / executing 1/10**。以下是合同内阻断，
不是允许延期的完成项；正常 pilot 24/24 与协议测试 57/57 不抵消反例。
完整源码锚点、AC/task、原始输入/结果、修复方向见
[独立复审报告](attachments/773/review-r1/REVIEW.md)；修复树为 lang-773/plan-773-dev。

| id | 级别 | 缺口 |
|---|---|---|
| F-773-R1-01 | high / blocking | a2r 成功发射可满足 Error oracle，真实 CLI PASS/0 |
| F-773-R1-02 | high / blocking | 已接受的 adapter 观测和错误出口流断言静默忽略，IMPOSSIBLE 仍 PASS |
| F-773-R1-03 | high / blocking | snapshot FAIL 错误 BLOCKED actual 交接；成功路径消费者未消费实际 bytes |
| F-773-R1-04 | high / blocking | selector 全部执行后改 NOT_RUN；坏 adapter selector 未报合同 ERROR |
| F-773-R1-05 | high / blocking | out/json-out 覆盖 fixture/输入；输出域/父 reparse 链校验遗漏 |
| F-773-R1-06 | high / blocking | oracle sidecar bytes 漂移身份不变；compiler/parser/config/旧 report 核查未接线 |
| F-773-R1-07 | high / blocking | max_expanded_checks_per_run 仅定义，无加载/执行门；字节准入读取边界遗漏 |
| F-773-R1-08 | high / blocking | I/O 被归语言 run；嵌套 MultipleErrors 未完整递归，阶段/基础设施未独立观测 |
| F-773-R1-09 | medium / blocking | lint 缺 expected/ref resolve 仍 exit 0 |
| F-773-R1-10 | medium / blocking | pilot 聚合入口未在计划规定模块；tv filter 不包含 test_cases_tests |
| F-773-R1-11 | medium / blocking | legacy hash/context 对账、proposal 旧新 hash/diff/共享消费者/失败传播未齐 |

修复保留原 21 case/24 check 和既有 fence/strict schema 绿项；在同一树提供闭合表、
永久控制和新提交，再跑 §6 完整 final 门禁。未写 canonical Spec/ledger，未归档或清理。

**2026-10-14 闭合**：修复轮 commit 39e2ecafd（rebase 后 fed5c7842，git range-diff
等价）关闭 F-773-R1-01..11（闭合表见计划 §9）；R2 独立 final review pass，唯一
minor 发现转下表债务。canonical Spec/ledger 已沉淀，plan 归档。

| id | 级别 | 缺口 |
|---|---|---|
| F-773-R2-01 | low / 债务（fail-safe） | a2r project 模式两入口（transpile_rust_project Phase 2、transpile_rust_project_merged）仍把 parse 错误包成无名 AutoError::Msg → classify=Infrastructure→ERROR：方向安全（永不假 PASS）但 project-mode 强错误 oracle 不可满足（pilot 21 case 无 project-mode 负例，AC-04 不变量保持成立）。处置：Design 35 B 阶段或独立 fix；彻底修复需带模块路径上下文的类型化变体，非纯机械删压平。锚点 crates/auto-lang/src/trans/rust.rs；来源 R2 复审（计划 §9）。 |


### P762（2026-10-11，图表 Init 批量 flake 分诊——helper 判据遗留面）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| D-762-1 | medium | 测试载体驱动 helper 早退 | **`ui/dynamic.rs::drive_scheduler_to_quiescence` 的 `progressed` 判据不计"续跑片让出未完成"（`cpu.slices_run` 缺位）**——多片 Init 单调用确定性早退（761 tt 批量档 gallery 四路由 flake 在案；确定性复现钉子 plan437 `child_init_multi_slice_early_exit_and_completion_driving` 前半断言）。约 16 个调用点共享该暴露面（plan502 已自带外层完成态循环、plan437 本计划已修，其余裸调用点的多片 Init 在并行负载下理论可复现同类红）。清偿方向（须另行授权，PLAN-762 约定"改生产 helper 先修订确认"）：`progressed` 增计 `cpu.slices_run > 0` 与 `silent_done_transitions > 0`（对齐 `poll_frame_pump` 判据族），I/O 未就绪退出臂保持；修复时同步更新钉子前半断言（其注释已留指引）。生产帧泵不受影响（每帧重入天然推进让出片） | crates/auto-lang/src/ui/dynamic.rs:1556-1578；docs/plans/reports/762-chart-init-attribution.md（PLAN-762 worktree） |

### Plan 744（2026-10-13，fs.dir_names Rust 发射签名 parity）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P742-D9-SIG-01 | low / explicit | `fs.dir_names` 跨端映射 | VM 标准接口 `fs.dir_names` 在 Rust 发射中映射到 `a2r_std::fs::dir_names`。KNOWN-DEBT 396 的既有签名配对比较 `file.rs.at` 与 `a2r-std/fs.rs`，并非该命名空间函数的源码镜像，因此此实现以显式 allowlist 例外记录；AutoDown/Notes 双端应用门覆盖行为。若未来改为生成式 fs 镜像，应移除此例外并加入生成一致性验证。 | `stdlib/auto/fs.at`；`crates/auto-lang/src/trans/rust.rs`；`crates/auto-lang/src/vm/ffi/stdlib.rs`；`crates/a2r-std/src/fs.rs`；`crates/auto-lang/src/tests/a2r_std_signature_parity.rs` |

### P756（2026-10-10，网站文字修复复核：范围外 UI 观察）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P756-D1 | low | website SectionNav 手机目录 | 已有目录 toggle/list 在手机展开时呈 flex 横排，列表高且部分入口受限；本轮仅文字/待拍槽，未改目录。后续改为纵向展开，360/390 检查全部章节可达与标题遮挡 | website/.vitepress/theme/components/SectionNav.vue；reports/756-website-copy-review.md |
| P756-D2 | low | website v05 原图负载与阅读密度 | 已有桌面首图约5.2MB、发布页较长；本轮保留有效原图与内容。后续生成展示派生图并保留完整原图/来源，另复核手机滚动和章节密度 | website/public/desktop-showcase/02-desktop-dark.png；reports/756-website-copy-review.md |

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
| REG-4 | ~~medium~~ → 已清偿（待 auto-os 合流） | kitchen-sink/schema 金样跨仓脱同步 | **schema_drift_fence / queue_coverage_drift_fence / docs_gen kitchen_sink_page_in_sync 三红族（2026-10-02 批量回归新红）**——根因=跨仓 auto-os commit `73d02b5`（Plan 717 "show all component examples"，2026-10-01 05:05）手改 widgets-gallery/src/front/pages/kitchen-sink.at 未走 `KITCHEN_SINK_UPDATE=1` 再生：时间线 0d5f5bf 重采样（09-30 14:21）→ 09-30 批量回执（10-01 00:20）三绿 → 73d02b5 手改 5h 后破红；auto-lang 窗口 c19b5351a..4614bd9df 零触 docs/components 与 schema 源；非 PLAN-726（stash 对照 pristine@17292c07e 同红，先于其 diff）。**PLAN-759 T-01（2026-10-11）已清偿**：①根因深化——非仅金样手改，Plan 717 四表同步整体欠账（别名/覆盖登记/render 臂皆缺，schema_drift_fence 的 P1 红长期遮蔽 P0 累积漂移）；②aura.at 手工补 12 别名+render_support 补 carousel/command fallback 臂+element_coverage 登记（auto-lang 986cc9392）；③金样经 AUTO_OS_ROOT 锚定 auto-os 独立 worktree 再生并提交（auto-os plan-759-dev 分支 b5a2333）——**待合流**：合回 auto-os main 后主检出/CI 的 kitchen_sink 围栏方绿（worktree 侧兄弟解析已绿） | auto-os worktree .wt/lang-759/auto-os@b5a2333（待合流）；schema_drift_baseline 刷新说明见 PLAN-759 执行记录；docs/plans/.last-batch-regression.json 2026-10-02 回执 |

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
| P706-D1 | medium | back-bridge call（merged 直调臂） | **probe_mtime merged 臂 release 载体 `done` 恒 false（挂起）为独立预存缺陷**——探针 widget 零 computed/memo，Init 弧 `probe_cases` back-bridge call 不完成；A/B 定谳：old-carrier c8f86ef92（pre-delta）release **同形红**（`done: false` 超时），修复 exe debug 全案绿 → release 特异的 back-bridge 调用面，非 706/plan047/705 delta 引入。清偿需 release 档 CALL/段驱动时序定位（debug canary 先爆掩蔽同窗） | auto-jade tests/probe_mtime.mjs；`p706-probe-mtime-rel.log` vs `p706-probe-mtime-oldrel.log` vs `p706-probe-mtime-dbg.log`（组目录 .wt/lang-706/） |
| P706-D2 | low | auto-jade e2e harness（消费方仓） | **vue 轨 Playwright e2e `快速打开` menubar 弹层点击预存 flaky**——元素反复 detach → click 重试至 240s 超时；双载体统计同 flaky（old c8f86ef92 2/4 pass / new 载体 1/4 pass，无区分度）⇒ 非本仓 delta 回归；gen/front/vue 为共享陈旧产物（前端非变量）。清偿归属 auto-jade harness 加固（force-click/动画等待/焦点稳定），不在 auto-lang | auto-jade e2e/matrix.spec.ts:632 waitButtonIn click；`e2e-old-run1..3.log`/`e2e-77d05-run*.log`（组目录） |
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
| P733-R1 | ~~medium~~ → 已清偿 | 测试基础设施 | **plan502_m3_layout_geometry_e2e 并行负载 flake（双树同发预存）**——全量并行档下 `handler_FlowDiagram_Init` 被 CPU-slice Parked(CpuRunnable)（PLAN-711 片预算负载下耗尽→测试驱动轮次内未续完→几何断言落 x=108 旧值）；单跑恒绿（本树 4/4+master 3/3）、全量约 2/4~2/5 闪红（worktree 与 master 3053f1fdf 同发同错，非 PLAN-733 回归——其 diff 零触 diagram/layout/parking 面）。**PLAN-759 T-08（2026-10-11，d5a507db3）已清偿**：run_dir 单轮 drive 改带 10s 截止时间的完成态循环（Init demand/CPU 续体/I-O parked 收敛即出），几何断言原值不放宽；plan502 族 8/8 绿+m3 scoped 两轮绿 | crates/auto-lang/src/tests/plan502_diagram_tests.rs:284（td st/ck 中轴断言）；复审记录 F-1（master 全量对照 5 采样 2 红） |
| P733-R2 | ~~low~~ → 已清偿（③随 PLAN-759） | VM 视图求值器 | **musk_vm_track p053_1/p053_4/p053_6/p054 预存红族=fn 调用语义同族深层腿**（master 3053f1fdf 基线即红，PLAN-733 三腿修复前后零行为变化/同错）——computed 链式 helper 断言面（musk chats 消息列表 computed 形态）残腿指向子件 override 态 `.store.X` 解析链深层；PLAN-733 边界登记于 design/vm-fn-call-semantics.md 已知边界③。**PLAN-749（2026-10-10）甄别：与条件通道异根**；**PLAN-748 T-02 首腿（2026-10-10，2e2164b5e 待合）分解定音**：六红三根——①p053_1×2+p053_6=`.store.X` 泛名展平空快照拒展平（三套 .store 口径不一致）**已翻绿**（musk_vm_track 97/6→100/3）；②p054×2 图标子件渲染面**归 748 T-02 续腿已修**（2026-10-10，3ec6a4914：PLAN-625 T-09(b) `tag.contains("icon")` 粗闸误判 PascalCase lucide 名→占位卡；判别器改 lucide 名集成员资格 `is_icon_component_tag`，AppViewport ∉ 名集保持占位卡意图；musk_vm_track 102/103）；③p053_4 merged `#[api]` CALL_NAT 发射**异根**——**PLAN-759 T-03（2026-10-11，753733205）定谳并清偿**：PLAN-633 守卫语义（实现体在场=直调不发桩）为有意契约而非债务，旧断言陈旧；双臂重锁（直调落账+缺实现 3142→RuntimeError）绿 | crates/auto-lang/src/tests/musk_vm_track_tests.rs:563/628/675；docs/specs/auto-lang/vm/design/vm-fn-call-semantics.md 已知边界节；PLAN-749 §9 T-00 定音报告 |

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

## PLAN-738 归档后补充复查 R11（2026-10-10）

原计划归档终态保持。下项为实际反例确证的后续必修候选，须另走修复流程；登记不代表批准延期。R9 两项修复已确证闭合，不撤销其闭合记录。

2026-10-10 闭合更新：P738-R11-01 已由 PLAN-751 `226ebfbbb` 修复，归档后独立补充复验原 R11 探针 1/1（1.55s），空文件一次绑定/再生成身份均 `811c9dc5` 且 fresh，读错误与恢复矩阵通过；正式生成服务完整链 1/1（361.01s，测试夹具仅适配缓存路径）。原反例登记保留历史；后续 HTTP 能力缺口见 P751-D1 / [751-review-r2.md](reports/751-review-r2.md)。

| ID | 级别 | 问题 | 证据与修复要求 |
|---|---|---|---|
| P738-R11-01 | medium / P2 | 空 Cargo.lock 生产/消费身份不一致 | 落地 `9ff34ae0d`，api_gen.rs:1076 成功读取空字节写 `811c9dc5`，rust_ui.rs:4030 归 absent；实际生成成功后门判 stale，absent 首次空文件出现也未绑定。仅 NotFound=absent，成功读取统一实际内容身份；补正式生成/消费空文件、漂移/删除及读错误恢复矩阵。AC-06/07，SD-05；[R11](reports/738-review-r11.md)，探针 0/1（1.40s） |

验证尾项：R9 HTTP 全档 45/102、57 项未跑，移交 regress 尚无覆盖此次落地的收据；完成完整串行 th 及逐名分诊后补收据，不外推全档通过。

## PLAN-738 补充复审 R9（2026-10-10，needs_fix @19973b089）——**已全部闭合 @41b4d4be5（R10 pass 核证）**

（历史登记保留）本节曾是本期未满足验收的**必修**；主链 re-merge 回归已修复后以下两项阻塞收口，现均已闭合：R9-01=gate/生成两端 NotFound-only absent + 其它读错误 fail-closed；R9-02=三文件四格式点收敛。证据：[738-review-r9.md](reports/738-review-r9.md) 头部闭合注记 + [738-review-r10.md](reports/738-review-r10.md)。证据与修复合同见 [738-review-r9.md](reports/738-review-r9.md)。

| id | 级别 | 领域 | 必要修复 | 锚点 |
|---|---|---|---|---|
| P738-R9-01 | medium | lock 读取失败 | 未绑定收据 absent + Cargo.lock 读取 PermissionDenied 被实际门误判 fresh；生成/复用两端只允许 NotFound=absent，其他错误传播/判陈旧，补 unbound/bound 读取失败反例，保留一次绑定语义 | rust_ui.rs:4025 / api_gen.rs:1071；AC-06/07/08，T-12/13/14 |
| P738-R9-02 | medium | 格式健康门 | 最终 14 文件默认与 skip_children 检查均 exit1；只处理三文件剩余布局并提交，绑定实际通过的新提交收据 | lib.rs:119 / renderer.rs:24761 / engine.rs:9840、9852；AC-08，T-14 |

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

## PLAN-751 执行登记（2026-10-10，work @lang-751）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P751-D1 | high | HTTP e2e 挂死族 | **plan730 上传 e2e 整族（14 项）+ e2e_a_redirect_302 确定性挂死**——服务端 accept 后不响应（60s×2 read timeout 累计 120s 被 nextest TERMINATING）；隔离单跑确定性复现（vm_plain_endpoint_untouched / e2e_a_redirect_302_with_location 均 TIMEOUT 120.0s）；与负载无关（同条件 plan326 兄弟 39 项 0.6s 全绿）。红移窗口=PLAN-738 系列（plan734 时代 th 99/101 绿 `ae9b7a7e2` → 738 后转红；http_server.rs 最后触碰=R2 `bade1845c`，其后 `19973b089` 触及 http 族契约注册撤销语义；R2 收据自述 th 绿与本轮矛盾）。**根因 bisect + 修复走后续专项计划**；候选锚点 `dd9ffe35b`/`19973b089` | docs/plans/reports/751-th-full-receipt.md；crates/auto-lang/src/tests/plan730_http_upload_tests.rs:254（start_server）/ crates/auto-lang/src/vm/ffi/http_server.rs |
| P751-D2 | medium | auto-man 日常档盲区 | **auto-man lib 3 项预存红**（基面 stash 实证 0/3 同红，与 PLAN-751 diff 无关）：`vue::plan593_index_css_golden_tests::index_css_values_match_p1_baseline`（CSS 金样漂移）、`rust_ui::tests::test_shell_pack_lib_freshness`（shell-pack 入库物过期，panic 自述需重跑 regen_shell_pack#[ignore]+提交）、`rust_ui::tests::merged_api_client_crud_fallback_for_uncovered_endpoints`（merged API 客户端金样漂移）。根因：日常档 `cargo t` 只 `-p auto-lang`，auto-man 面长期脱离例行门禁——清偿方向：修复三红 + 将 auto-man scoped 面纳入复审触面清单 | crates/auto-man/src/rust_ui.rs:5743/6203；crates/auto-man/src/vue.rs（plan593 金样） |

### PLAN-751 R2 补充分诊更新（2026-10-10）

- **P751-R2-01（high/P1，P751-D1 的实证入口）**：落地树 `d79282eb6` 的原上传与重定向程序，通过实际 `crate::run` 在启动前即返回 `STDASSEMBLY.SIGNATURE_UNVERIFIED`，分别缺 `http.upload_receive #9937` 与 `http.response_redirect #3108` 的独立 producer/adapter 契约（探针 0/1、0.10s；shim 实际存在）。后续需审计最终公共 HTTP provider/ABI 并补真实合同，保留严格门，再验 wire/完整 th；不是 751 lock 修改引入。
- **P751-R2-02（medium/P2，纠正 D1 归因边界）**：D1 上述「accept 后不响应、60s×2」只能保留为执行者假设，尚非确证机制；重定向实际配置 5s read timeout；原两 start_server 均丢弃 run Err。历史表为 plan730 13 timeout + interop fail，另 redirect timeout，不能把 interop 也计为 120s 挂死。需有界保留启动 Err/panic/readiness，再接通原 120 秒等待位置。last-touch 非根因证据；代码引入校验锚点 `d82eb02eb` 是新候选，尚未 good/bad 对拍，不签署精确归因。
- 原 102 项计数保留历史；本轮没有再跑完整 th，不外推全绿。详情与新证据：[751-review-r2.md](reports/751-review-r2.md)。738/751 归档终态保持，修复走后续专用 worktree。
- 原未修改的 `http_e2e_plan730_vm_plain_endpoint_untouched` 本轮独立复跑仍 timeout（120.046s，exit 100）；其长等待与有界诊断快返契约错误的差异尚未接通，不能直接等同。

### PLAN-755 闭合更新（2026-10-10，work @lang-755；**同日 merge 销案定稿**）

- **P751-D1 已销案（delivered @`e586b9113`，R1/R2/R3 三轮复审后合并）**：PLAN-755 已按专项修复并全档复验——①根因实证：契约错误（`SIGNATURE_UNVERIFIED` #9937/#3108，`crate::run` 32ms 快返）被夹具 `let _ =` 吞掉 + 本机拒绝连接 ~2.15s/次 × ready-poll 50 次 + 客户端 60 次重试爬行 = nextest 120s TERMINATING；"accept 后不响应/60s×2 read timeout"旧机制证伪（read 相位从未到达），见 [755-startup-diagnosis.md](reports/755-startup-diagnosis.md)。②修复：HTTP 公共面 41 项缺契约补齐 + 5 项 inventory 契约漂移覆盖 + 走查/merge 存活常驻守卫（[755-contract-inventory.md](reports/755-contract-inventory.md)）；夹具启动诊断有界化。③复验：串行全档 th **102 项 101 过/1 红（仅 back_proxy 在册基线红）/0 超时，186.4s**——14 timeout 清零、interop 转绿、86 绿零回归（[755-th-full-receipt.md](reports/755-th-full-receipt.md)）。bisect 候选锚点（dd9ffe35b/19973b089）随根因实证销案；规范增量 SD-01（公共面契约覆盖完整性）落 assembly-manifest.md。
- P751-D2（auto-man 盲区）不在 PLAN-755 范围，保持开放。

### PLAN-755 独立复审 R2（2026-10-10，needs_fix）

| ID | 级别 | 影响验收 | 证据与修复要求 |
|---|---|---|---|
| P755-R2-01 | medium/P2 | AC-01、T-02 | 三启动助手的 `Err(panic)` 直接 Debug 格式化 boxed Any，父线程消息只得 `Any { .. }`。实际 plan730 助手注入标记 panic，负向验收 0/1（2.153s），标记只在默认子线程 hook，未转发至启动失败载荷；三处提取 &str/String payload 并冻结真实助手反例。 |
| P755-R2-02 | medium/P2 | AC-06、T-06 | clean `4ce645516` 的四改动 Rust 文件 rustfmt check exit 1；此前“fmt 干净”与提交工件不符。格式修复提交后重新绑定门禁证据。 |
| P755-R2-03 | medium/P2，簿记已纠正 | AC-07、T-07 | SD-01 modify 已存在的 assembly-manifest.md，不能因新增节登记为新组件；本复审已修正 spec-impact 分类，规范语义不变，修后重审定稿。 |

契约守卫及上传/重定向原反例独立 2/2 + 2/2 通过，主要 HTTP 修复事实保留；整项退回 executing，P751-D1 保留闭合候选，尚不销案。详细证据：[755-review-r2.md](reports/755-review-r2.md)。

2026-10-11 落地后独立闭合更新（覆盖上述 R2 的历史待修路由）：P755-R2-01/02 已由 `5672e8e2d`（原 `e121debf8`，range-diff 全等）修复，三助手共享字符串 panic 转发核心；本会话在 clean `cee541da7` 隔离树复跑守卫 4/4、四文件格式 exit 0、完整串行 HTTP 102/101/1 在册 back_proxy 红/0 超时（合并守卫共 106 run，265.241s）。P755-R2-03 元数据纠正保留，755 archived 不回退。blueprints 68 文件恢复后逐文件 Git 哈希与 HEAD 一致；恢复前额外日常红不同于原 R2 实现缺陷。P751-D1 已销案结论得到串行复验支持；批量回归仍 DUE（旧收据覆盖 740），另按 regress 补执行。证据：[755-landed-independent-check.md](reports/755-landed-independent-check.md)。

### P753（2026-10-10，README 计划知识投影前置阻塞）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
**P753-D1 注记（2026-10-11）**：753/756 的投影阻塞已经 README §5 手工回退路径清偿（P753-1/2/3、P756-1/2/3 入账，两计划归档/清理完成，收据见各计划 recovery receipt）；债务剩余部分收窄为——store 本体无法加载现有 `.autoos/specs.json`（缺 document metadata，GET /api/specs 500）且无注册写入器可达（758 r2 同期收据复证 blocked by policy）。后续计划投影可继续走回退路径，store 修复为独立事项，本条保留至 store 恢复或裁定退役。

| P753-D1 | medium | runtime Specs store 兼容性 | 既有 `.autoos/specs.json` 无法由当前 AutoMusk store 加载；任务自有 loopback 服务 `GET /api/specs` 返回 500，已停止。原账本 hash 保持不变；不通过手改 JSON 或删除重建修复。README/语言文档可独立复验，但 store-mediated 投影及随后的计划归档/清理必须等待受支持的恢复路径。未将此项视为已完成。 | [验证报告](reports/753-readme-verification.md)；PLAN-753 §9；auto-musk backend/crates/musk/src/specs.rs |

### PLAN-754（2026-10-11，reviewed；低危外观瑕疵）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P754-R1-01 | low | playground-vue 深链 | 无效 `?note=<id>`（manifest 中不存在）时参数残留在 URL——`deeplinkTried` 仅在命中或无参时置位，`history.replaceState` 只在命中时执行。欢迎态不崩已由 e2e ③ 覆盖；后续顺手修：未命中分支同样置位+清参。 | PLAN-754 §9 复审记录；packages/auto-playground-vue/src/components/PlaygroundIDEApp.vue |

### P758 补充（2026-10-11）

- **P758-D1 / publication blocked**：AutoJade 当前规范与网站可合入；derived ledger 的 `P732-1` / `P722-1` 名称和九份规范投影（含 r3 宣传数字/桌面平台口径）须通过 store writer 更新。当前未注册写入工具，检查的既有服务端口无监听；临时 AutoMusk 服务启动/只读探测被自动审批以 `blocked by policy` 拒绝，未给进一步原因。未手改账本。保留 reviewed Plan 与工作树，待 writer 可用后只续做投影/归档/清理；证据与描述见 [p758 复核](../reports/p758-autojade-verification.md)、[待投影描述](../reports/p758-autojade/ledger-projection-pending.json)。延续 P753-D1 的收尾限制，不抹去旧账本内容。
- **P758-OBS1 / asset provenance**：用户给的 P4 tags 文件实际是其他面板，未冒充标签图；SHOT-09 恢复仍依合同留空。六张合格深色原图有完整字节/hash，缺深色专属工具链 manifest，因此只标用户提供准备态。补拍要求见 [截图指南](../reports/website-v05-capture-guide.md)。
- **P758-OBS2 / baseline gate**：注释改名无非注释差异；最后 Rust 日常档 19 红，其中 18 项在原注释基面复现，余一动画项在基线及最终隔离复验均通过；tu 的既有金样红亦在基线复现。不虚报全绿或纳入本次无关修复，逐项日志见 p758 复核附件。

- **P758-OBS3 / concurrent baseline**：同步 PLAN-754 后，静态预览 /playground 主动探测同源 /api/examples 得到 404，fallback 127.0.0.1:3030 未运行产生连接拒绝；原控制台检查隔离复现。两个宿主组件与当前 master 等同，本计划未改变它们或放宽检查。证据：[基面核对](../reports/p758-autojade/concurrent-baseline-check.json)。最终当前网站 61/63，搜索隔离复验通过，余一既有此项；最终 t 5213/5231、18 红（16 已对照、2 flake 隔离绿），tu 857/2 红（原金样 + env 路径用例隔离绿），完整记录在 p758 复核。

### PLAN-757（2026-10-11，r3 用户裁定留空合入；产品缺陷待修）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P757-D1 | high（用户指示需修改） | AutoEdit VM/iced 比较视图布局 | 双文件比较视图正文上方大片空白编辑区、差异挤在下半部（并排与内联均观察到；`h-0 w-0` 隐藏 code_editor 为排查线索，未确证根因）；真实画面不适合宣传（问题图 `docs/reports/p757-capture/media/p757-autoedit-diff-layout.png`，+16/-1 一个 hunk 被压缩）。功能本身已核查可用（保存后自动重比 +17/-1 两个 hunk 在案）。**SHOT-03 宣传图因此留空**，布局修复后按拍摄合同重拍（指南行"待布局修复后重拍"）。 | PLAN-757 §9/§10；[诊断原图](../reports/media/p757-autoedit-diff-layout.png)；[捕获目录](../reports/p757-autoedit-capture-catalog.json)；auto-edit specs/auto-edit/src/front/app.at 比较隐藏编辑器 |
| P757-D2 | high | VM 查找输入与焦点路由 | 原生键入（聚焦查找输入后输入 ws_root）同时落入查找框与活动内置 untitled main.at 正文（样例缓冲区未保存，真实源码未受写入）；合格搜索图改用应用 MCP FindInput handler 规避。需专门复现焦点/键盘路由，不能把 MCP 成功当原生链无缺陷。 | [捕获目录](../reports/p757-autoedit-capture-catalog.json)；vm2.stderr.log；PLAN-757 r2 findings |
| P757-D3 | low | VM 搜索响应 | 跨文件搜索扫描完整工程时同步阻塞原生窗口，完成后真实返回 ws_root=22、fn ws_root=2；未计精确时延不外推通用性能，拍摄须等待完成 | [搜索状态](../reports/p757-autoedit-search-state.txt)；AutoEdit fsys.at search_files_json |
| P757-D4 | low | website 首次搜索索引加载 | VitePress local search 首次加载索引长尾，并行负载下 e2e 偶发超时（新端口单 worker 复跑 19.6s 过） | 同上 |

### PLAN-759 清偿登记（2026-10-11，日常档 12 红族收口）

清偿成员（证据链见 docs/plans/759-daily-red-family-clearance.md 执行步骤）：
musk p053_4（P733-R2③）/ p748_t00 双臂（生而红 harness 边界）/ plan606 test_029（10.2s→0.18s）/
projector_counter（双形态重锁）/ desktop_bus_inbox + desktop_surface_merge（壁纸机器态隔离）/
schema 三围栏（REG-4 四表清偿）/ ash_stream_leak_probe（门禁分层，集成真跑绿）/ plan502_m3
（P733-R1 完成态驱动）。

| 编号 | 优先级 | 域 | 描述 | 锚点 |
|---|---|---|---|---|
| P759-D1 | low | iced 渲染配置 | **硬编码机器壁纸目录探测**——`wallpapers_dir_or_default`（renderer.rs）解析链末端探测 `D:\Down\stella-os\wallpapers`（PLAN-526 T19 机器便利默认），dev 便利进库代码路径：①机器相关行为（测试须显式隔离，规约已入 ui/overview.md PLAN-759 注记）；②他人机器不生效。清偿=挪 env/配置显式开启；涉及产品行为须用户裁定 | crates/auto-lang/src/ui/iced/renderer.rs `wallpapers_dir_or_default`（~:17019）；docs/specs/auto-lang/ui/overview.md PLAN-759 注记 |
| P759-D2 | low | schema 漂移围栏 | **schema_drift 的 rs 扫描器仅认单行 `elements.insert("`**——schema.rs 现存 209 个多行 insert 形态不可见（仅 15 单行被扫到），四表对账长期靠千行级 baseline 白名单吸收。PLAN-759 刷新 baseline 时实证（+249/-301）。清偿=扫描器多行化+随后全维度 baseline 重审裁剪（牵动数百条目，独立计划） | crates/auto-lang/tests/schema_drift.rs:62 `scan_insert_tags`；crates/auto-lang/tests/fixtures/schema_drift_baseline.txt（1075+ 行） |

### PLAN-759 落地后独立核查候选（2026-10-11）

以下候选登记于[落地后审核](reports/759-post-landing-review-20261011.md)（原判定 needs_fix）；D3/D4 已由 `fd55d2d42`（fix-759-review-p2 轻量 worktree 路径）清偿，并于2026-10-11完成独立复审（`35be7f1e0`，两项修正 pass，日常5236/5236、tv162/162、裸libtest三线程4轮均绿；tu唯一余红为既有desktop金样，完整证据见同报告末节），不改变759归档终态。

| 编号 | 优先级 | 域 | 描述 | 锚点 |
|---|---|---|---|---|
| ~~P759-D3 / F-759-R1~~ 已清偿 `fd55d2d42` | ~~medium~~ | 生产渲染诊断 | **已清偿**：render_support 详情查询折叠 schema canonical（kebab/canonical 三围一致）；command/carousel 族按 view_builder 实际转换臂逐子件校准（command_input=partial 与 input 同口径、command_item=partial 仅 value 忽略、command_empty=fallback 恒渲染 Empty、command_group/shortcut=full、carousel_item/previous/next=full 等）；schema/aura.at 12 行同步；别名一致/ignored 不过报/实际 View 对照三回归测试入 render_support::tests | ui/render_support.rs（get_support+详情表+tests）；schema/aura.at backends.iced；报告F-759-R1 |
| ~~P759-D4 / F-759-R2~~ 已清偿 `fd55d2d42` | ~~medium~~ | 测试环境隔离 | **已清偿**：stdlib 新增 `HTTP_BASE_TEST_SERIALIZER`+`lock_http_base_for_test`（镜像 lock_storage_for_test 先例）与 `HttpBaseEnvGuard`（RAII panic-safe 恢复，含未设态）；plan606 photo 测试与 stdlib http_base_url_resolution 均改锁+守卫形态；panic 恢复回归测试入仓。全部原有断言（精确 URL/24 张恰一次/Cover/Contain/stub 收据）保留 | vm/ffi/stdlib.rs（锁+守卫+两测试）；tests/plan606_gallery_tests.rs；报告F-759-R2 |

### PLAN-761 根目录整理的范围外候选（2026-10-11）

| 编号 | 优先级 | 域 | 描述 | 锚点 |
|---|---|---|---|---|
| P761-D1 | medium | CLI test filter | `auto test --filter` 在 `test_file` 执行全部测试之后才过滤显示；a2ts runner 内部已有 FAILED 仍可显示 0 passed/0 failed 并返回0，VM runner即使使用不匹配过滤器也继续执行到超时。原有实现，本次仅迁移runner；不能把过滤后的退出0当套件验证通过。应将过滤条件传入执行层并保留实际失败退出码 | crates/auto/src/main.rs Test 分支（test_file→遍历reports→filter顺序）；reports/761-root-directory-review.md |
| P761-D2 | low | 历史 Plan446 repro | 原字节保留的 b3check 语料在当前引擎报 `Init` 同时匹配 Theme/Modules（multi-store disambiguation）；新路径确已加载且没有SKIP，显式ignored旧复现仍不兼容当前严格规则。保留原始历史导出，不在目录美化中改其业务语义；后续若恢复常规使用，另做版本化修复夹具 | test/fixtures/plan446/b3check/auto；tests/plan446_j1_repro_tests.rs；reports/761-root-directory-review.md |


### PLAN-761 收尾批量回归新观察（2026-10-11）

- P761-R1（未销案）：tt的plan437 gallery_chart_components_render_geometry缺/bar-chart h6且Init仍CpuRunnable；同提交tf/tb通过。当前归因未确定，独立分诊[PLAN-762](762-chart-init-batch-flake.md)，不得仅凭偶发绿降为环境豁免；详见[批量报告](reports/761-batch-regression.md)。原目录整理已交付，未改变上述实现。


### PLAN-763 迁移基线对照登记（2026-10-11，taa 全档 5 红分诊收口）

| 编号 | 优先级 | 域 | 描述 | 锚点 |
|---|---|---|---|---|
| P763-R1 | low | aavm compile 腿 + a2r goldens（master 预存） | **3 枚 AAVM 红为迁移前基线预存债，非 PLAN-763 目录归并致因**——基线 worktree（716ca62db=分支点）同滤串同红同错误形态实证：①`test_aavm2_compile_corpus`/`test_aavm2_compile_use_corpus` 双 E0433（生成 main.rs 引 `auto_lang::a2r_std::fs::read_text_range` 而临时工程 `[dependencies]` 空、harness 无内联 shim；血统=db4e5954b plan687 增补，2026-09-22）；②`test_aavm2_goldens_check` 参考执行 `STDASSEMBLY.INVALID_BYTECODE: 60`（字节码语义红，与路径无关）。清偿归属 aavm/a2r 域另档：compile 腿需给临时 Cargo.toml 接 auto-lang path 依赖或改发射面；goldens 需定位字节码 60 案。与 aavm 无实用面裁定一致（守护=CI+ta，本债不阻塞合入） | docs/reports/763-baseline-taa.{json,log}；763-final-taa.log；db4e5954b |
| P763-R2 | low | aavm 自举入口（master 预存） | **`auto <repo>/aavm.at` 单文件直跑 auto_link_E0401 `Undefined symbol: P.next in module typeinfo`**——master 主检出（迁移前源码+master 二进制）同命令逐字节同错实证=预存；与七删 CI 流最近五次全红的审计结论同向。迁移侧有效冒烟=命名空间 scratch（smoke-final aavm-namespace exit 0）；自举形态冒烟仍应走 aavm_build_smoke.sh（合并装载）而非单文件直跑 | docs/reports/763-smoke-recheck.json；763-ci-audit.md |

### PLAN-765 复审登记（2026-10-12）

| 编号 | 优先级 | 域 | 描述 | 锚点 |
|---|---|---|---|---|
| P765-D1 | low | VM 宿主内存 | idle 相位 ~1MB/min 线性缓漏未归因（1h 实测 +59.3MB/59.7min,AC-1 门 0.1MB/s 的 6× 余量内,线性不平台）。候选=每响应仍余 3 份全文瞬态(body/转义缓冲/VM Value+池)×2 拍/s 的堆记账漂移、消息批全窗重绘路径。后续杠杆:载荷池化复用/`unconditional-rendering` feature 退役评估(PLAN-708 基线启用项)。判定阈值:长寿实例(≥12h)私有内存超 ~1.2GB 再升级优先级 | docs/reports/765-t01-attribution-memos.md 后记⑥;soak tmp/soak-p765-1h.json(musk worktree,临时) |
| P765-D2 | info | 跨仓仪器 | ~~musk soak 仪器 slopeMBPerMin 单位 bug~~ **已修(musk plan-104-dev 9794aa1/0e06161,2026-10-12 核验)**;AC-01 已按真值重判 pass(idle 0.717MB/min 对门 8.4× 余量) | docs/reports/765-t01-attribution-memos.md 后记⑤;musk 104 evidence §T-03 |
| P765-D3 | info | 跨仓仪器 | bigpage-matrix.mjs 收尾击杀假象(try/finally 无 catch:click isError 抛错→finally taskkill→"VM EXITED code=1" 先于错误栈打印,被误读为进程自退;musk T-03 因此曾 blocked on 765-R2)。**765-R2 已定谳为假象非产品缺陷**(纪要后记⑦);修复归 musk 侧(click 补 value+收尾击杀标记,两行修,交接文案在案) | docs/reports/765-t01-attribution-memos.md 后记⑦ |

## v0.6-dev debt and review records

The following unique review and debt sections are carried forward from v0.6-dev. Their historical attachment paths and source commits remain unchanged; Plan 767 is now PLAN-772 in the current ledger.

## PLAN-772 独立复审（2026-10-12，revision 2，修复重审 pass）

以下为合同内阻塞缺陷与修复候选，必须在本计划修复后重新复审；登记不代表允许挂债收口或合入。受审提交 `033642a27d7404dee5797b195a9b7c680fe6ece5`，完整反例与任务路由见 [独立复审报告](attachments/767/review-codex-r2.md)。

**销号收据**：三项均已在 `4cc4645021f029459a3b870c34a23eb16bc4ce59` 修复。原 needs_fix 的独立复审者重放冻结 16 项、新增 48 项不变量检查及 scoped 门禁后确认 closed，无遗留阻塞。保留历史缺陷描述，当前结论见 [独立修复重审](attachments/767/review-codex-repair-4cc464502.md)。Plan 为 reviewed（8/9），T-09 尚未执行。

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P772-R1 | P1 / closed | HIR 身份 | Descriptor::domain 将未知三重身份降为 Core，程序化 bind/verify 与 writer 可接受未知 schema/revision/profile；三维分别复现，共六反例。精确注册身份随行并受控拒绝，不以 CLI registry 正确为豁免。 | experimental/ac-core/src/descriptor.rs:245、2111（受审树）；PLAN-772 T-02/06/07/08，AC-01/05；owner=767 work |
| P772-R2 | P1 / closed | HIR 类型可信门 | 签名 TypeUse 查表失败默认 I32，params/result TypeId(9999) 分别获 Checked 后 native 越界 panic；补所有签名引用验证和永久正反控制。旧默认逻辑不是本合同豁免理由。 | experimental/ac-core/src/verify.rs:226、255；native.rs:91（受审树）；PLAN-772 T-02/06/07/08，AC-04/05/06；owner=767 work |
| P772-R3 | P2 / closed | 设计收据 | design.md 的减法溢出公式写为 MIN-b/MAX-b，实际后端正确为 MIN+b/MAX+b；修收据并重绑证据，不照错误文档改正确实现。 | docs/reports/767-scalar-i32/design.md:112（受审树）；PLAN-772 T-01/07/08，AC-07；owner=767 work |
## PLAN-745 new 取号核查（2026-10-06）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P745-D1 | high | 中央取号 | scripts/new-plan.sh 的 `ls active-glob archive-glob \| grep -q .` 在 `set -o pipefail` 下，某目录无匹配导致 ls 非零，即使另一目录已有同号也漏检；本次 .next-id=744 与既有 744-vm-sqlite-module.md 冲突时误建了同号骨架。已撤回本次未跟踪骨架，再由中央脚本取得745，未覆盖旧744，计数现746。脚本还缺跨写者排他锁；修复应在独立worktree建立统一排他取号与active/archive重复控制，不在主检出顺手改代码。 | scripts/new-plan.sh:33；docs/plans/745-ac-source-checked-hir-native.md §4.3；owner=后续取号机制修复工作，未编号 |
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

### P722（2026-10-01，网站 Apps 介绍复审）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|

### P716（2026-10-01，供③ 确认复核件降级挂账——plan_revision 2 用户裁定）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|

### P706 r2 收口批（2026-09-30，work 阶段定案的非本回归缺陷另立记账）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| ~~P707-R1~~ | high | async http 直驱臂（e4） | **已清偿（2026-10-02，PLAN-724 T-02）**——真因勘定=**测试侧违反 managed 提交协议**（非派发缺陷、非 detached 桥分离）：e4 三臂直接 `spawn_async_http_handle`/`spawn_async_http` 提交 managed job（999_001/999_101/999_102）却未先 `register_live_op`，707 `submit_client_job` 的 cancel-before-install 闭合（still-live 复查）在 spawn→登记窗口内发现令牌缺席 → 立即 abort 刚提交的任务 → 请求恒不到线 → 30s `recv_timeout` 恒红。生产 shim 全部先 register（stdlib.rs 5367/5390/5413/5434 等在案）。修复=测试改走生产协议：register→submit→wire 断言→消费配对（plain 臂增验 managed 完成落表 `Structured{status:200}`）+`cancel_live_op` 幂等清理，并钉住未登记反例（abort 句柄 submit 返回前回收 + 无令牌永无可消费结果）。取消守卫零改动；plan712（2/2）与 plan707_cancel（5/5）复证绿；修复后单跑 0.46s。公开入口覆盖保持：直驱两点即公开 native shim 的唯一 send 汇聚处，默认头/query/body wire 断言原样 | stdlib.rs e4 模块（default_headers 三臂+未登记反例臂）；`cargo t default_headers_reach_wire_on_plain_get` 绿日志（work 阶段在案） |


### P708（2026-09-29，实施方案审查：待核实风险）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|

### P683（2026-09-22，Plan 683 rq-remote-renderer 方案 2 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
### P684（2026-09-22，Plan 684 gallery 示例修复批 work 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
### P693（2026-09-23，Plan 693 native-exe-tri-mode work 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| ~~P693-D2~~ | medium | desktop 接入跨仓配对 | **已销号（PLAN-698 T-04，2026-09-23）**——形态勘定=子进程（库形态被 winit Windows 单事件循环结构性否决，RecreationAttempt 实锤）；`spawn_desktop_daemon` 孵化 `auto rqhost --pipe autodesk-rqhost-desktop-<pid>` + AUTO_DESKTOP_ENDPOINT env 注册表发布（命名裁定定稿）；003 converter.exe 全环录证（adopt→开窗→v2 帧→EOF 回收→末窗自退，reports/p698-batch/T04-desktop-pairing.md）。SD-04 落 ui/overview.md；WM 深度集成仍归 auto-os 侧后续 | renderer.rs run_session Desktop 臂；rqhost.rs spawn_desktop_daemon；desktop.ps1 |

### PLAN-698（2026-09-23，Plan 698 desktop-substrate-hardening-batch work 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
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
## PLAN-743 r5合入后独立复审 → r6 Phase6（2026-10-06，激活时未修复；现已清偿，见后续复审）

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
## PLAN-743 r6合入后独立复审清偿记录（2026-10-06）

reviewed_commit=8b17e4de8851cdf37ccdce76a82f78422cf8bb15，outcome=pass；[独立报告](../reports/743-r6-quality-review-20261006/REVIEW.md)、[归档计划](archive/743-acc-bootstrap-hir-contract.md)。原r5反例/激活时待修文字为历史，本段为三个743债项的当前终态；不修改741或其它计划债项。

| ID | 当前状态 | 独立验证 |
|---|---|---|
| P743-R5-QA-01 | 已清偿（r6 T35） | 绑定文件存在/删除/恢复CLI0/1/0，consumer21/0/21，缺失文件不再入可信消费者 |
| P743-R5-QA-02 | 已清偿（r6 T35） | 错类型/合法duplicate双顺序及缺字段双顺序整组consumer0；错误ID容器类型受控拒绝无traceback |
| P743-R5-QA-03 | 已清偿（r6 T36） | 原参数/local type同名bare/qualified保持unknown；7组普通/mut方法、let/var/use控制通过，无冲突正例维持 |

128/128、真实strict47/146/7、确定性/manual原字节/三方一致及actual archive38/38/all P743指针通过。保持743 archived/r6，不新增修复phase；审查treeguard clean后已清理，Category A未跑Cargo/native/docs_gen。；**已清偿**（r7 T-39 fixture_tests.py 生命周期重写（计划源自动检测 active/git 历史重放归档态；冲突变换按真实行状态翻转恰一行+断言实际修改；状态变换通配 set_fm；全部变异经 assert_modified）；两矩阵 21 用例 + 全 PASS（含 archive 态 git 历史重放与全完成合成控制））；**已清偿**（r7 T-40 final_assertions v3 扩到 prototype README 741 链接与 auto-hir/auto-ac 741 导航行（状态生命周期一致+行号定位+--repo-root 沙盒负控制）；本轮归档后 archive 模式复跑收口）
## PLAN-741 r7合入后复审 → r8 Phase8（2026-10-06，未修复）

基线18beff64b4ceefa1e7f4176296f5e37f77dd6da1；[报告](../reports/741-r7-quality-review-20261006/REVIEW.md)、[同ID合同](741-ac-hir-native-core.md)。实际归档41/41与导航正确，核心同源；原缺文件/零操作已修，不重新挂旧缺陷。

| ID | 级别 | 当前缺口 | 修复与验收 |
|---|---|---|---|
| P741-R7-QA-01 | P3 | 两个导航控制未实际运行，总数固定+2；归档7报9、双源17报21且记录称已验证 | T42/AC33，纳入实际控制列表、失败传播、按执行记录计数，旧日志保留注明过期 |
| P741-R7-QA-02 | P3 | 必需README Plan引用缺失、README/模块/P741 review指向其它存在的归档Plan均仍ALL-PASS | T43/AC34，必需引用存在及规范唯一741目标严格比较，active暂态限定规范archive/741 |

同741激活r8 current38/total44，只重开T39..41，新增T42..44；未实施/未销账，不改Rust/行为Spec/live ledger，P741-3..9归档暂态由最终merge恢复。742/743/ABI及历史收据保留。；**已清偿**（r8 T-42 两矩阵导航控制以 add_external 进入统一 cases 真实执行（各带沙盒 repo root）+ 总数从执行记录派生（固定 +2 删除）；控制性确认 forgotten-nav-update 期望改 0 → fixture 整体非零（失败传播）；--source auto 实测 25 用例全 PASS）；**已清偿**（r8 T-43 final_assertions canonical 目标身份检查（README 必须存在 741 计划引用；README/两模块导航/全部 P741-* review 指针解析后必须恰等于 docs/plans[/archive]/741-ac-hir-native-core.md，错目标失败含 expected/got 与 entry ID；激活期 canonical_archive 尚不存在=显式列暂态）；4 个 identity 沙盒负例 + 反向验证旧版门假通过全部实证）
## PLAN-745 r1 最终独立复审（2026-10-06，待修）

reviewed_commit=b3cd8171966281a42f96187780342ce0ff51c5fb；needs_fix；[最终报告](attachments/745/review-r1/REVIEW.md)、[计划](archive/745-ac-source-checked-hir-native.md)。同一实现树保留，执行中1/9；本轮仅复审/证据/簿记，没有修复、合入或规范沉淀。原92绿/74例/两脚本通过为真实局部证据，不能覆盖本轮反例。

| ID | 级别 | 合同内缺口 | 修复/验收 |
|---|---|---|---|
| P745-R1-QA-01 | P2 | 非括号RHS/callee跨行仍check0 | T02/05、AC02；换行边界正反控制 |
| P745-R1-QA-02 | P2 | 用户ac_start/ExitProcess过Checked却native符号冲突 | T03/04/05、AC02/05/06；隔离native内部/用户名字空间，不静默保留源码名 |
| P745-R1-QA-03 | P2 | 作用域逃逸指return，双分支终结后语句指前面的if | T03/05、AC04；精准出错token/span，纠正N09宽oracle |
| P745-R1-QA-04 | P2 | Auto关键字当名字；!与转义str误分类 | T02/05、AC02/04；保留关键词和识别但排除的诊断 |
| P745-R1-QA-05 | P2 | source测试无截止output/先wait后read堵管道；合法20K trace控制93ms，错误顺序2s仍阻塞 | T05/07、AC05/06/08；有限截止、并发采集、后代/全部出口回收 |
| P745-R1-QA-06 | P2 | HIR trap结构断言、实际Bundle删let/错type/越域变异、source SENTINEL永久门缺失 | T03/05/07、AC01/03/05/06；真实变异/不变量oracle/失败传播；独立source三件保留已通过，不报告产品破坏 |
| P745-R1-QA-07 | P3 | inventory --output误作文件，误称manifest未跟踪而推merge重扫 | T06/07、AC07；正确strict47已过，纠正证据/策略，不自动重hash |
| P745-R1-QA-08 | P3 | source/HIR各复制Checked后构建管线，未兑现共用合同 | T04/05、AC06；同一构建入口，两模式回归 |

这些项未清偿；登记不等于批准延后。SD01/03及盘点策略待修后重新冻结，完整宽度/char保持open；canonical/live ledger保持当前状态。root门禁完整daily29红/tv4红另有原始清单，生产源/config零745 diff且均在历史红集并集；未称门禁绿色或严格同源逐项归因完成。
## PLAN-745 r1 第二轮最终独立复审（2026-10-06，待修）

reviewed_commit=91b37560cffda75cf07cef8620cf524f5d6a3c7c；[第二轮报告](attachments/745/review-r2/REVIEW.md)、[同ID计划](archive/745-ac-source-checked-hir-native.md)。needs_fix/executing/1-of-9；同树保留，未实施本轮修复、合入或规范沉淀。

R1-QA02（native符号）、QA04（关键词/排除）、QA07（盘点误归因）、QA08（共用构建）**已清偿**；QA01/03的具体原反例已修，但新增回归见下；QA05的大输出/普通父进程截止、QA06的trap结构/删let/错type/部分SENTINEL已修，剩余合同项按本轮ID追踪。旧历史段落保留，不撤销具体pass，也不以旧待修文字误称产品毫无修复。

| ID | 级别 | 当前缺口 | 修复/验收 |
|---|---|---|---|
| P745-R2-QA-01 | P2 | 合法多行call/括号闭合后的同行+表达式误拒 | T02/05、AC02/05；括号上下文和真实语句换行边界成对控制，保留C01/C03拒绝 |
| P745-R2-QA-02 | P2 | Rust助手500ms截止实际5016ms仍成功；PS1s实际5044ms；无效UTF8输出被吞 | T05/07、AC05/06/08；执行/采集同deadline，受控子树及全出口回收，字节排空/真实错误传播，两助手均修 |
| P745-R2-QA-03 | P2 | 两Stray分支被提升有效Div，后续死块遮住break/continue目标错误 | T03/05、AC02/04；组合保留Stray诊断可达性，固定实际冒犯token |
| P745-R2-QA-04 | P2 | Bundle第三变异只验异type；publish控制先删旧receipt后仅比较两文件 | T03/05/07、AC03/06；同type越域真实变异、保留三件旧文件的实际publish失败/恢复控制；独立verify.uninitialized-read已证实，不报告产品verifier失效 |
| P745-R2-QA-05 | P2 | 4122字节/4096未闭合括号使编译器栈溢出exit -1073741571 | T02/03/05、AC02/04；深畸形输入受控diagnostic/exit1/无panic，浅合法控制；不偷缩profile或只扩大栈 |

本轮99测试/两脚本/strict47均通过，但不覆盖这些反例；root日常33红/tv4红实录保留、hover隔离1/1通过，未声称严格同源逐项零新增或根门禁绿色。五项未清偿，登记不等于批准延后；SD01/03完整叙述待修后重新冻结，canonical/live ledger保持当前状态。
## PLAN-745 r1 第三轮最终独立复审（2026-10-06，待修）

reviewed_commit=7437eb137a789259f01446da7a170e74031ff7f2；needs_fix；[第三轮报告](attachments/745/review-r3/REVIEW.md)、[计划](archive/745-ac-source-checked-hir-native.md)。原树保留，executing 2/9；没有实施修复/合入/规范沉淀。

| ID | 级别 | 当前合同内缺口 | 修复/验收 |
|---|---|---|---|
| P745-R3-QA-01 | P2 | Rust 505ms 超时返回仍留后代和读线程；PS5.1 415ms 静默窗假成功，后代/输出未收完 | T05/07、AC05/06/08；受控启动子树、实际EOF、统一截止、所有出口回收/错误传播，持管道/延迟写/超时/约束失败控制 |
| P745-R3-QA-02 | P2 | NEST_LIMIT 把合法224字节99层括号当syntax错误；32794字节合法加法链仍栈溢出 | T02/03/05/06/07、AC02/03/07/08；覆盖实际树及后续处理递归，合法与不闭合边界控制；不得静默缩小r1支持集 |

R2-QA01 换行、QA03 Stray 组合、QA04 Bundle同类型越域/三真文件publish oracle已清偿。R2-QA02 字节读取/有界返回局部修复保留，整体转R3-QA01；R2-QA05原不闭合E02已受控，新健壮性/边界问题转R3-QA02。R1其它清偿保持。

本轮104 passed/1 ignored、741 13步/745 12步74例、独立74/74与strict47通过；根日常27失败+1超时，tv4失败未称绿色，新增dep018超时隔离1/1通过。初始PS5.1哈希失败是复审.NET驱动继承PS7模块路径的环境注入，已通过单变量正反控制定位并按计划原命令重跑成功，不记为实施缺陷。
## PLAN-745 r1 第四轮最终独立复审（2026-10-07，待修）

reviewed_commit=dc5e95be757dba647ef758aa283e7e2788015747；needs_fix；[第四轮报告](attachments/745/review-r4/REVIEW.md)、[计划](archive/745-ac-source-checked-hir-native.md)。同树保留，executing 3/9；本轮没有实施修复、合入或canonical/live ledger沉淀。

| ID | 级别 | 当前合同内缺口 | 修复/验收 |
|---|---|---|---|
| P745-R4-QA-01 | P2 | 新PS C#执行器约束失败只杀空Job，挂起子进程仍存活；真实ReadFile错误6被当EOF返回Code0/空输出 | T05/07、AC05/06/08；未入Job进程直接终止+wait，读取/等待等错误传播，有效及真实失败控制、无幸存/无截断成功 |
| P745-R4-QA-02 | P2 | NEST_LIMIT改语法计数仍未批准支持边界；200层完整合法括号426字节被syntax拒绝 | T02/05/06/07、AC02/07/08；保留r1支持或显式new修订边界/诊断/提案/正反验收，不以扩阈值替代合同义务 |

R3-QA01的Rust生命周期与PS静默窗口原反例已清偿；R3-QA02的合法16384链崩溃、99/100层括号误拒已清偿；未完成的边界子项和新PS错误出口按上表追踪。原C/D诊断、符号、换行、Stray和既有oracle没有退化。

本轮112 passed/1 ignored、741 13步/745 12步74例、独立74/74、strict47通过；新增native复杂求值24/baab、trap70空stderr、长链退出300通过。根日常35失败/tv4失败未称绿色；唯一新红名plan643隔离1/1通过。真实Win32失败控制保留原cleanup/result路径，不用全绿banner替代失败传播检查。
## PLAN-745 r1 第五轮最终独立复审（2026-10-07，待修）

reviewed_commit=15fd9263f6f4c400a6ba1d8ec5cb2cae44354d41；needs_fix；[第五轮报告](attachments/745/review-r5/REVIEW.md)、[计划](archive/745-ac-source-checked-hir-native.md)。原同树保留，executing3/9；没有实施修复、合入或canonical/live ledger沉淀。

| ID | 级别 | 当前合同内缺口 | 修复/验收 |
|---|---|---|---|
| P745-R5-QA-01 | P2 | 连续括号塌缩未覆盖1+(1+(…))；4096层完整16410字节与未闭合12314字节均stack overflow，5s仍未返回 | T02/05/07，AC02/04/08；实际非连续及组合路径受控，合法source→Checked通过、畸形诊断无panic，保留优先级/效果/span |
| P745-R5-QA-02 | P2 | 调用/块护栏1000仍未批准入r1/SD；函数体占一层，999可native/1000完整语法已syntax拒绝 | T02/05/06/07，AC02/07/08；保留支持或显式new修订计数/各入口/资源诊断/边界及正反验收，不仅扩大阈值 |

R4-QA01未入Job挂起子泄漏、ReadFile错误6假成功与wrapper传播已清偿（实际native失败控制无幸存）；R4-QA02原200层连续括号误拒已清偿，连续100000层通过，剩余边界转R5-QA02。历轮其它清偿保持；新发现不覆盖掉已经通过的控制。

本轮113 passed/1 ignored、741与745均13步、独立74/74、strict47通过；native300、24/baab、trap70空stderr、混合括号111/5、999call/if通过。root日常33失败/tv4失败未称绿色；唯一新增plan707_cancel隔离1/1通过。完整反例/原始stderr/边界两侧与冻结delta在第五轮报告附件，登记不代表批准延后。
## PLAN-745 r1 第六轮最终独立复审（2026-10-07，待修）

reviewed_commit=522b98a4c27d2064049e188d386f6958721b3a90；needs_fix；[第六轮报告](attachments/745/review-r6/REVIEW.md)、[计划](archive/745-ac-source-checked-hir-native.md)。原同树保留，executing3/9；没有产品修复、合入或canonical/live ledger沉淀。

| ID | 级别 | 当前合同内缺口 | 修复/验收 |
|---|---|---|---|
| P745-R6-QA-01 | P2 | 迭代表达式解析器逗号分支未先归约Bin：f(1+2,3*4)、named/异构bool/嵌套/trace均syntax误拒；同源H09由native111退化为拒绝 | T02/05/07，AC02/05/08；按Call/Paren边界归约当前实参，保留mark/命名映射/求值顺序，补正反永久回归 |
| P745-R6-QA-02 | P2 | 终止分支仅归约前检查顶帧，Bin遮住未闭合Call/Paren，非法源码check/build0；f(1+2缺右括号被丢调用而native3，真SENTINEL三件被覆盖 | T02/05/07，AC02/04/06/08；所有出口归约后检查未闭合帧，准确诊断，无Checked/发布，保留三件 |
| P745-R5-QA-02（块项） | P2 | 调用子项已清偿；块护栏1000仍未批准入r1/SD，999可native/1000完整if已拒，实施侧明确未完成 | T02/05/06/07，AC02/07/08；继续满足原合同，或显式new修订计数/入口/资源诊断/正反验收后获批 |

R5-QA01原4096层非连续括号崩溃清偿；R5-QA02调用边界清偿（20000调用check0、4096混合native4097、未闭合受控），剩余块子项沿用原ID。历轮执行器/来源/Bundle/旧事务清偿保持；额外实参括号只是定位正控制，不能视为批准workaround。

本轮113 passed/1 ignored、741与745均13步、独立74/74、strict47通过；Rust正常与PS9/9真实失败出口无幸存。root日常37失败全部历史共现、tv4失败未称绿色。完整输入/实际PE输出/SENTINEL前后指纹/冻结delta见第六轮附件，登记不代表批准延后。
## PLAN-745 r1 第七轮最终独立复审（2026-10-08，清偿）

reviewed_commit=bb8cb4b4330d505059ea20e7bfe561a93c1051f3；**pass**；[第七轮报告](attachments/745/review-r7/REVIEW.md)、[计划](archive/745-ac-source-checked-hir-native.md)。原同树保留，reviewed 8/9；没有合入、归档或canonical/live ledger沉淀。

| 原ID | 处置 | 独立证据 |
|---|---|---|
| P745-R6-CLOSE-01 | 已清偿 | positional/named二元实参42、嵌套192、异构bool5、trace23/ba、H09恢复111；括号内非法逗号仍拒 |
| P745-R6-CLOSE-02 | 已清偿 | return/let/if/while缺右括号均text/source.syntax准确拒绝；实际M21 check1/build1，exe/obj/receipt三件旧SHA逐件相同 |
| P745-R5-QA-02（剩余块项） | 已清偿，原finding整体关闭 | 无固定嵌套计数护栏；1000if/1500if-else/2048while真实PE、20000if到Checked、深未闭合受控；新增N01..24块流/作用域/循环/trace矩阵全过 |

历轮其它清偿保持；本轮无未清偿合同内finding。114 passed/1 ignored、741/745各13步、独立74/74、strict47新鲜；真实Bundle及执行器失败控制保持。root日常30失败全部历史共现、tv4历史semver失败，不称全绿或严格同源码main对照；生产路径无745 diff。冻结delta与终审HEAD/依赖/Specs/配置无漂移，证据见第七轮附件。T09真实closeout与批回归到期判断仍待执行，既有P745-D1中央取号脚本债务不在本计划修复范围。
## PLAN-745 合入批量回归观察（2026-10-08）

745 的独立源码 profile 合同已通过；以下属于根测试/外部输入的观察，未静默缩减验收或把失败报成全绿。完整原始日志与逐名归因见[合入报告](attachments/745/merge/MERGE.md)。根生产源、配置及依赖清单在落地前后无745变更。

| ID | 观察 | 后续处理边界 |
|---|---|---|
| P745-MERGE-ENV-01 | temp_plan009_t8_transpile_at_app 硬编码外仓 auto-term 输入路径不存在，tt失败 | 后续独立计划修测试环境合同；本次不创建或写外仓路径 |
| P745-MERGE-VERIFY-01 | tb 的55个book金样差异，且未ignore的generate_book_expected在同批成功写外仓金样；并发验证oracle会变化 | 后续独立计划隔离生成与只读验证；无外仓运行前快照，保留现存修改，不重生成/回滚来制造绿色，不等同旧七个parse失败 |

stage3_memory_baseline_n1_3_5 与 plan707_client_sse_poll_close_error_surface 批量失败后，单实例、单job各1/1通过；按负载时序观察记录。tf/tt/tb分别31/34/76失败均保留原始证据，新增未匹配名没有自动归为历史红。
## PLAN-774 状态核查（2026-10-13，规划交接漂移）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P774-D1 | medium / closed | 计划交接与取材 | 原共同契约769改为774后，活跃执行指令/意向工作树仍用769，标量前置链接仍指向不存在的旧767文件；770/771仍引用已不存在的769共同契约文件，消费者auto/lib路径亦已迁移。启动实施前校正活跃编号/依赖/取材，历史附件保留来源；这不是已完成实现的缺陷或豁免。774实际drafting 0/8，探针/报告/脚本/共同Spec均未产出。 | docs/plans/774-ac-data-resource-contract.md §4/8、770/771 §0；[状态核查](attachments/774/status-audit-20261013.md)；owner=774 plan handoff |

P774-D1销号（2026-10-13）：770/771确认未改号；774/770/771活跃依赖、执行指令、工作树入口、真实消费者路径及战略/模块导航已同步。历史附件保留来源，当前指纹已刷新；仅引用缺陷关闭，774仍drafting 0/8。证据见[引用更新收据](attachments/774/reference-update-20261013.md)。
