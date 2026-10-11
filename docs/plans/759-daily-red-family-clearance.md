---
plan_id: PLAN-759
status: executing              # drafting → executing → execution_done → reviewed → archived
feature_name: daily-red-family-clearance
plan_revision: 2               # r2: 按 2026-10-11 草案审核 F-01~F-08 修订
author: []
created_at: 2026-10-11
updated_at: 2026-10-11

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # goals.md 无逐红族治理条目，本计划不映射 GOAL-NNN（理由见规范增量节）

affects: [auto-lang/ui, auto-lang/vm]   # schema/aura.at 经 include_str! 编入产品且被 ui_gen 消费；p053_4 边界在 vm fn-call 语义 spec
current_step: 0
total_steps: 9
---

# [PLAN-759] 日常档 12 红族清偿（r2）

## 变更摘要

`cargo t --no-fail-fast`（@3cdafcd40，2026-10-11 实测 5231 例）12 红。**定性（F-07 修订口径）**：
这 12 项在当日批量回归收据（3cdafcd40）中均已登记在册；逐族定位显示已查明的成因为
陈旧断言（产品语义有据变更后测试未跟）、测试基建缺口、机器态隔离缺口与跨仓构建/再生
依赖欠账，**不是最近实现窗口的新增产品回归**——该收据的 `new_reds: []` 只证明「相对登记
清单无新增」，不单独证明 746-758 窗口零回归（其中 748 双臂即「加入时已红」的新增红
测试，非产品行为回归）。本计划逐族清偿至日常档 0 失败，跨仓项实际落地（非移交），
并只对已验证成员销账。

**范围声明（F-01/F-08 修订，撤回 r1「产品代码 diff=0」笼统承诺）**：本计划不改任何
产品**逻辑**，但会改动**编入产品的数据输入与库内围栏源码**——`schema/aura.at`（经
`aura/schema_loader.rs:838` `include_str!` 编入产品、`ui_gen/widget/registry.rs:72` 消费）
与 `aura/element_coverage.rs`（非 test-only 库源码）。承诺口径改为：既有条目（aliases/
props/tier/backends/sub_widgets）零变更、新增仅为「数据追平生产代码已接受的标签」，
既有用户语义保持；消费端验证见测试设计（schema loader 覆盖 + `cargo tu`）。

## 目标

1. `cargo t --no-fail-fast` 日常档 0 failed：12 红族全部实际清偿；ash 探针以**可见
   `#[ignore]` + 明确集成门禁命令**形态从日常档分离（本轮实际构建 DLL 并真跑绿，见
   T-07），不用静默 return 冒充通过。
2. 每族修复附 scoped 实际执行证据；plan502 鲁棒化后附全档运行证据（保留原几何断言，
   不以重试成功掩盖失败）。
3. KNOWN-DEBT 只对已验证成员销账/更新措辞（保留历史收据原文，不整族误销）；未清偿
   项如实登记（预期仅 R5-② 硬编码壁纸目录门控，本计划明确不做、登记为独立债务）。
4. 跨仓项闭环=实际动作+收据：auto-os 金样再生在其独立 worktree 完成并提交（哈希回填）、
   auto-term 在专用 worktree 构建 DLL 并真跑集成探针。

## 架构方案

无架构变更。修复分四类手段：
- **A 陈旧断言重锁**（R1/R4）：产品语义变更有据（用户裁定/计划裁定），测试按现行契约
  重录，且两臂契约（有实现直调/缺实现报错；显式 variant 描边/裸按钮 chromeless）均必测。
- **B 测试隔离/基建补齐**（R2/R3/R5）：整链截止时间的 in-proc 泵、真实 photo stub、
  存活至测试结束的空临时目录壁纸隔离——断言保持精确，不设宽松放行。
- **C 再生流程补课**（R7）：四维度分开核对（canonical 声明/别名同步/覆盖登记/金样同步），
  跨仓写入一律在独立 worktree + 显式 env 锚定下进行。
- **D 门禁分层与环境依赖**（R6/R8）：跨仓二进制依赖的探针显式分日常档（可见 ignore）与
  集成门禁（强制 DLL 在场、真跑）；负载 flake 以带截止时间的任务完成态驱动修复。

## 需求分析与背景调查

（诊断于 2026-10-11 master@3cdafcd40 完成；r2 按 F-01/F-05/F-07 修订 R7/R2/定性口径）

### 根因矩阵（12 红 → 8 根因簇）

| # | 测试 | 根因簇 | 引入点 | 类型 |
|---|---|---|---|---|
| 1 | `musk_vm_track_p053_4_merged_api_warning::merged_mode_api_call_emits_warn_opcode` | R1 | `4d5f8de13`（PLAN-633 T-04，09-15） | 陈旧测试 |
| 2 | `plan502_diagram_tests::plan606_gallery_tests…plan502_m3_layout_geometry_e2e` | R8 | P733-R1 在册（双树同发） | 负载 flake |
| 3/4 | `plan748_breakpoint_probes::p748_t00_handler_{static,dynamic}_arm` | R2 | `e9d953ec4`（PLAN-748 T-00，10-10）加入时即红 | 测试基建缺口 |
| 5 | `plan606_gallery_tests::test_029_photo_gallery_thumbnails_render_with_resolved_src_and_cover_fit` | R3 | `231cca2fd`（PLAN-043 Part1，09-24） | 陈旧测试+缺 stub |
| 6 | `ui::desktop_protocol::client_runtime::tests::projector_counter_layout_and_hits` | R4 | `b4c2dc38b`（PLAN-080 T-03，09-20） | 陈旧测试 |
| 7 | `ui::iced::renderer::tests::desktop_bus_inbox_drains_through_real_arms` | R5 | `74fbc8085`+`d990427ba`（712 T-13/T-14，09-30） | 测试隔离缺口（机器相关） |
| 8 | `ui::iced::renderer::tests::desktop_surface_merge_dedupe_and_injection` | R5 | 同上 | 同上 |
| 9 | `vm::ffi::term_engine::ash_leak_probe::ash_stream_leak_probe` | R6 | 测试硬编码主检出 DLL 路径 + auto-term target 未构建 | 跨仓环境依赖 |
| 10 | `schema_drift::schema_drift_fence` | R7 | Plan 717 别名/登记/生成物同步欠账 | 再生流程欠账 |
| 11 | `schema_drift::queue_coverage_drift_fence` | R7 | carousel 五项覆盖登记缺失 | 同上 |
| 12 | `docs_gen::kitchen_sink_page_in_sync` | R7 | `5bad66e0f` + auto-os `73d02b5`（REG-4 在册） | 跨仓金样脱同步 |

### 各根因簇机理（证据链）

**R1 p053_4（CALL_NAT 3142）**：merged 模式（`api_over_http=false`）下 `#[api]` 裸调用现为
三分支（`crates/auto-lang/src/vm/codegen.rs:9426-9456`）：① host 桥已注册→`auto.host.call`；
② **api 实现体已随 imports 编入模块（`exports` 含该函数）→ 不发桩、直落常规直调 CALL**
（PLAN-633 守卫 `!self.exports.contains_key(...)`，`codegen.rs:9445-9453`）；③ 实现体缺席→
才发 `CALL_NAT 3142`。r1 fixture 把 `#[api] fn chats_list_sessions` 放进 import_stmts →
声明期即注册导出（`codegen.rs:1560`；imports 先于 handler 编译，
`ui/handler_codegen.rs:1576-1685`）→ 命中分支②，字节码无 3142 → 断言失败。发射代码未丢，
是语义有据变更后的陈旧测试（KNOWN-DEBT P733-R2③ 所谓「codegen 面异根」即此守卫）。
另：PLAN-734 D7（`beec5b94f`）后 3142 原生语义已从「warn no-op」演进为带指引的
RuntimeError，测试名 `merged_mode_api_call_emits_warn_opcode` 双重过时。

**R2 p748_t00 双臂**：**加入时即红**（748 归档 §9
`docs/plans/archive/748-vm-cond-eval-and-field-write.md:615-618` 登记为「comp harness
in-proc 不泵 Http 续体」的 harness 边界；绿的 W:5 证据来自 E2E 腿，T-01「三臂全绿」指
fn 段探针 `p748_t00_resume_static_write` 等，今仍绿）。机理：测试
`p748_store_handler_write_face`（`crates/auto-lang/src/tests/plan748_breakpoint_probes.rs:322-393`）
两次 `drive_scheduler_to_quiescence` 后 `server.join()`，**此后无任何泵调用**——生产中续体
由 iced 事件循环 `__parked_resume_tick` 订阅→`poll_parked_resumes`（`ui/dynamic.rs:1699`）
驱动，in-proc harness 无该循环；`drive_scheduler_to_quiescence` 第二轮探
`async_http_result_ready` 未就绪即 `progressed=false` 早退（`ui/dynamic.rs:1572-1576`）。
**F-05 补充**：`p748_spawn_server` 在 `listener.accept()` / Read 上无截止时间阻塞，r1 方案
的 deadline 在 `server.join()` 之后才建立，保护不到服务器等待段。落前后历次收据均记
在册红；嫌疑提交（be51414da/7d51954d4）已排除。

**R3 plan606 test_029**：`231cca2fd`（PLAN-043）把 029-photo-gallery 从本地种子数据改为
Init 经 `Http.get_json("/api/photos/scan?dir=")` 拉后端 photo_service。单测进程无后端、
`AUTO_HTTP_BASE` 未设 → 相对 URL 传输失败 → 按 PLAN-712 T-17 语义抛可捕获异常 →
catch 臂置 `.photos=[]`（`examples/ui/029-photo-gallery/src/front/app.at:466-486`）→
空态视图零 image 节点 → 泵谓词永假 → 10s deadline（`plan370_test_support.rs:441-450`），
10.2s 时长即此。**即便起了后端现断言仍红**：thumb src 现为绝对 URL
`{base}/apps/{app_id}/api/photos/thumb/{id}`（`back_proxy.rs:1041-1042`）；`OpenPhoto`
签名已改 str，测试仍发 Int(1)。be51414da 与此红无因果（红早它 15 天，且本测试不跑
iced 渲染）。

**R4 projector_counter**：断言「按钮=底盒+四边描边」`quads.len()==5`
（`ui/desktop_protocol/client_runtime.rs:1058`）。裸按钮 border 类来源链：PLAN-571 缺省
`""`→UA 基线（含 border）；**`b4c2dc38b`（PLAN-080 T-03「裸 button 缺省 chromeless」）把
`ui/style/variants.rs:30` 的 `"" => ""` 改空** → `style.border=None` → 四条描边 Quad 消失
（`native_projector.rs:1927-1929` 仅 border Some 时 push）。产品行为=用户裁定，测试未跟。
时间线：`98b778f44`（09-19）重录 5-Quad（彼时 PLAN-571 基线成立，绿）→ b4c2dc38b（09-20）
→ 首红证 `e0e2d6b52`（09-21）→ 09-30 首次批量收据在册。

**R5 壁纸双测**：712 r2 T-13（`74fbc8085`）重排 `load_desktop_wallpaper`
（`ui/iced/renderer.rs:16972`）优先级——#hex 从「直传短路」降为「仅无目录机器」档④，
档②为新插入的分主题默认图（T-14 `d990427ba`，深色 `purple.png`）；目录解析链末端是
**硬编码机器探测 `D:\Down\stella-os\wallpapers`**（`renderer.rs:17031-17033`，PLAN-526 T19）。
测试会话用 `DesktopConfig::default()`（`wallpapers_dir:""`）且不设
`AUTO_DESKTOP_WALLPAPERS_DIR` → 本机目录存在 → 档②命中 →
`desktop_wallpaper="D:\Down\stella-os\wallpapers\purple.png"` ≠ `"#101014"`。
- `desktop_bus_inbox_drains_through_real_arms`（`renderer.rs:35143`）：SetWallpaper("#101014")
  后断言 `desktop_wallpaper=="#101014"` 落空。
- `desktop_surface_merge_dedupe_and_injection`（`renderer.rs:38013`）：快照构建里
  `starts_with('#')` 才生成 `bg-[#243b55]` 片段（`renderer.rs:17555-17559`），壁纸被图片
  路径覆盖 → `__desktop_bg==""` ≠ `"bg-[#243b55]"`。
**机器相关红**：无 stella 目录的机器（CI）至今绿。产品语义=T-13 用户裁定；旧测试未同步。
**R5-②（硬编码目录挪 env 门控）本计划不做**，登记独立债务（见待澄清/规范增量）。

**R6 ash 探针**：`ash_stream_leak_probe`（`vm/ffi/term_engine.rs:1222`）硬编码
`AUTOTERM_ENGINE_DLL=D:\autostack\auto-term\target\debug\autoterm_core.dll`（:1224-1227），
该 DLL 现不存在 → `engine_spawn` 返回 0 → 断言红。双重问题：① 跨仓构建依赖缺失；
② 测试把路径绑死在 auto-term**主检出** target 上（F-02：应 env 定位、专用 worktree 构建）。

**R7 schema 三围栏（F-01 修订口径）**：Plan 717（auto-lang `5bad66e0f` + auto-os `73d02b5`）
**并非「三道流程全未走」**——`5bad66e0f` 已新增 carousel 全族 canonical 声明
（`schema/aura.at:3759` 起）、calendar（`:3418`）、command 全族（`:3820` 起）及 command
后端元数据。真实缺口按四维度分开：
- **① canonical 声明**：已在场（执行时核对无缺失即可）；
- **② 别名/拼写同步**：生产 `ui/aura_view_builder.rs:2874-2990` 接受 `Calendar`（Pascal）
  与 kebab 写法（carousel-content 等），schema 用 calendar/下划线 canonical 名——
  `schema_drift_fence`（`tests/schema_drift.rs:1946`）报的 12 个 tag 即别名同步缺口，
  **预期修法=既有 canonical 元素补 aliases，禁止为每个拼写生成重复组件**；
- **③ 覆盖登记**：`queue_coverage_drift_fence`（`tests/schema_drift.rs:2282`）缺 carousel
  五项 `element_coverage.rs` 登记（登记档位须有真实理由，不能任意选以过门禁）；
- **④ 金样同步**：kitchen-sink.at（auto-os 仓）与 schema 脱同步（`tests/docs_gen.rs:406`）
  = REG-4 在册（auto-os `73d02b5` 手改未走再生）。
**跨仓写入落点（F-02）**：`docs_gen.rs:31` 经 `resolve_os_top_dir` 解析 gallery（env →
组内 auto-os → `D:/autostack/auto-os` 兜底），`:394-395` 的 UPDATE 直接 `fs::write` 已解析
路径——**不建 auto-os 兄弟 worktree 时会写进 auto-os 主检出**，必须显式 `AUTO_OS_ROOT`
锚定独立 worktree 并在写入前核验解析路径位于其中。UPDATE 测试写完**故意 panic**，属生成
信号，不能当验收绿；生成后须清 env 重编译重验。

**R8 plan502_m3**：P733-R1 在册负载 flake——全量并行档下 `handler_FlowDiagram_Init` 被
CPU-slice Parked(CpuRunnable)，片预算耗尽，测试驱动轮次内未续完 → 几何断言落旧值
（x=108）；单跑恒绿（本轮 scoped 亦绿）。**F-03 修订**：清偿标准=以带截止时间的任务
完成态驱动的修复+全档证据，保留原几何断言，不以重试成功掩盖。

## 详细设计

（各族互相独立；T-01 前置跨仓 worktree 布置，见执行步骤 T-00 环境布置）

### T-01 R7 四维度清偿（含跨仓安全落点）
1. **别名/声明再生**：`SCHEMA_DRIFT_GENERATE_AT=1 cargo test -p auto-lang --test schema_drift`
   （故意 panic=生成信号）。**diff 复核清单**：既有元素 aliases/props/tier/backends/
   sub_widgets 零变更；新增仅为既有 canonical 元素的别名（Calendar→calendar 等）；
   **零重复组件**（每个拼写不得新增独立 element）；再生后生产 tag ∪ aliases 与
   aura.at 双向对账。实际 diff 与新增数量以再生结果为准，不预设「13 个声明」。
2. **覆盖登记**：`crates/auto-lang/src/aura/element_coverage.rs` 补 carousel 五项；每项
   档位（NotYet/NotConsumed）给出真实理由（依 Plan 717 组件形态判断），不任意二选一。
3. **金样再生（跨仓）**：见 T-00 的 auto-os worktree 布置与 AUTO_OS_ROOT 锚定；
   `KITCHEN_SINK_UPDATE=1 cargo test -p auto-lang --test docs_gen`（故意 panic=生成信号）
   → 两仓 diff 复核 → auto-os 侧走其仓规构建/金样门禁 → auto-os worktree 分支提交。
4. **验证**：清 env 后三围栏重跑绿；`cargo tu`（`ui_gen/widget/registry.rs:72` 消费
   schema 的生成端）；日常档自带的 schema loader / component_registry 覆盖随全档复核。

### T-02 R2 p748_t00 整链截止时间 + 泵补齐
1. `p748_spawn_server` 有界化：accept/read/write 全部设超时（socket timeout 或非阻塞+
   截止轮询），server 线程整体存活不超过测试预算。
2. **deadline 从派发前建立，覆盖 派发 → server 应答 → join → 泵 全链**（F-05：不再只在
   join 后建）；任何一段超时即失败并输出诊断（parked/CPU/marker 现场状态）。
3. join 后泵循环与生产 `__parked_resume_tick` 同机制（先例
   `plan370_test_support.rs:433-443`），**泵阶段仍验证精确 W:5 / W:6**。
4. 顺手项（F-05）：noawait 臂不启动无请求 server；TempDir 以作用域持有替代
   `mem::forget`，测试结束正常释放。

### T-03 R1 p053_4 双臂必测（F-04：两臂都是必须项，无可选）
1. **臂 A（有实现→直调且真实落账）**：fixture 保持 `#[api]` 进 import_stmts；断言
   ① `module.exports` 含该函数；② handler 对该导出发出模块内直调 CALL；③ **运行模块，
   断言调用结果/副作用真实落账**（返回值到达 store/栈，非仅字节码形态）。host 桥状态
   显式隔离（断言分支①不参与：HOST_CALLS 空或显式复位）。
2. **臂 B（缺实现→显式报错）**：以**更低层 codegen fixture** 构造「api_funcs 注册而
   实现体未编译」（不走 `synthesize_widget_module`——api_funcs 与 exports 同源，表达不出
   该状态）：断言 ① 该形态下发射 `CALL_NAT 3142`（发射链证据）；② 执行该调用命中
   native 3142 的 RuntimeError 语义（文案含 "has no implementation"，对齐 PLAN-734 D7，
   `stdlib.rs:7971-7990`）。不能以「直接调用 native 函数」冒充发射链覆盖。
3. 测试名 `…emits_warn_opcode` 措辞修正（语义已是硬错误）；注释引 PLAN-633
   （`4d5f8de13`）与 PLAN-734 D7。销账：P733-R2③ 措辞更新。

### T-04 R4 projector 双形态分别验证（用户裁定：两案并做）
1. `COUNTER_SRC` 加 `variant: "default"` 臂：保留「底盒+四边描边」`quads.len()==5` 断言
   （PLAN-571 UA 基线路径的投影覆盖）。
2. 新增裸按钮锁测试：无 variant 时 `quads.len()==1`（PLAN-080 T-03 chromeless 契约），
   防两语义再漂。

### T-05 R5 壁纸双测隔离（精确断言，无宽松放行）
1. 两测试各自创建**空临时目录**并写入 `config.wallpapers_dir`，**目录存活至测试结束**
   （测试作用域持有 TempDir，结束才 drop）——挡掉 env/stella 两级探测，恢复 #hex 直传
   确定性。优先只改两测试自身的 config 构造点（`__test_session()` /
   `t3_session_with_shell()` 为共享 helper，若需动 helper 必须先审计全部调用面）。
2. 断言保持精确：`desktop_wallpaper == "#101014"`、`__desktop_bg == "bg-[#243b55]"`
   **逐字相等**；**删除 r1 的 `|| Path::is_file()` 宽松备选**（F-06：允许错误图片通过
   即削弱 SetWallpaper/背景注入断言）。
3. R5-②（硬编码 `D:\Down\stella-os\wallpapers` 探测挪 env 门控）：**本计划不做**，
   登记独立债务（语义增量草案见规范增量节），不作为消红前置条件。

### T-06 R3 plan606 test_029 真实 photo stub（主案唯一，F-03 删除 skip 备选）
1. **stub 服务**：tiny_http/TcpListener 绑 `127.0.0.1:0`（OS 分配端口）；路由：
   `/api/photos/scan?dir=…` 返回固定 24 条（条目字段对齐 photo_service 真实 schema——
   执行时从 `back_proxy.rs` / photo_service 源核对 `id/title/date/size_str/width/height/
   sort_key/thumb_url/full_url`）；`/apps/{app_id}/api/photos/thumb/{id}` 与 `/full/{id}`
   返回图片字节。server stop/join/请求处理全部有界。
2. **env 隔离**：`AUTO_HTTP_BASE` 指向 stub——执行时核对该 env 的既有消费面与仓内
   先例（plan349/back_proxy_tests）：nextest 一测一进程天然隔离；裸 libtest 并行形态下
   采用仓内既有 env 守卫机制（全局 serial 锁或恢复原值），并在测试设计节写明两形态。
3. **断言（精确映射，F-06）**：24 张缩略图；每张 src 精确等于
   `http://127.0.0.1:{port}/apps/{app_id}/api/photos/thumb/{id}`（id ∈ stub 条目集，
   非子串包含）；Cover fit；`OpenPhoto` 改发 `Value::str(id)`，全图 URL 与**所选同一 id**
   精确一致 + Contain；**断言 stub 侧收到 scan 请求**（请求日志断言）；**HTTP 续体完成
   后再读 View**（轮询 parked 清零/结果就绪，复用 T-02 同机制）。

### T-07 R6 ash 探针门禁分层 + 跨仓构建落点（F-02/F-03）
1. **测试改形**：删除硬编码 `set_var` 主检出路径 → 读 `AUTOTERM_ENGINE_DLL` env（未设
   且默认定位失败时以**静态 `#[ignore = "integration: 需 autoterm_core.dll（设 AUTOTERM_ENGINE_DLL）；集成门禁=…"]`**
   显式分层——日常档显示为 ignored（可见，非静默 pass），集成门禁命令写进 ignore 说明。
2. **集成门禁真跑**：auto-term 专用 worktree（组内 `.wt/lang-759/auto-term`，见 T-00）
   `cargo build -p autoterm-core`（独立 target，不碰主检出）→ 设
   `AUTOTERM_ENGINE_DLL` 指向该 worktree target 产物 → 显式跑探针（--ignored 形态）
   → 绿证据入执行步骤。缺 DLL 场景=「未执行」记录，**不记 AC-02 pass**。
3. 收据：auto-term worktree commit + 构建记录回填本计划。

### T-08 R8 plan502_m3 鲁棒化（必做，F-03 取消「可选」）
1. 驱动逻辑改为**带截止时间的任务完成态驱动**：循环驱动直到 `handler_FlowDiagram_Init`
   续体完成（或截止），预算耗尽即失败并输出 parked/CPU 现场——**保留原几何断言原值**，
   不放宽、不以重试掩盖。
2. 证据：scoped ≥2 轮绿 + 全档 `cargo t --no-fail-fast` ≥2 轮无该红（负载形态下验证）。

### T-09 收口与销账（只销已验证成员）
1. 裸 `cargo t --no-fail-fast` 全档终证据：0 failed（ash=可见 ignored）。
2. KNOWN-DEBT 更新（保留历史收据原文）：P733-R2③（措辞→已清偿）、REG-4（auto-os
   提交哈希回填后销）、712 §10② 基线红族**仅移除本轮已验证绿的成员**、735 收据 ash
   预存登记（→集成门禁分层说明）、P733-R1（T-08 落地后销）；R5-② 新增独立债务条目。
3. 跨仓收据：auto-os 金样提交哈希；auto-term worktree 构建+集成探针记录。
4. flake 证据：plan502（T-08）与 plan484（711 §10 在册）scoped 复跑绿记录。

## 规范增量（F-08：r2 新增；review 时冻结为最终 spec-impact）

- `docs/specs/auto-lang/vm/design/vm-fn-call-semantics.md` **已知边界节③**：R1 清偿后
  「唯一余留 p053_4」表述过时——更新为：merged 模式 `#[api]` 三分支派发契约（host 桥/
  实现体在模块内直调/缺实现 CALL_NAT 3142→RuntimeError）已由双臂测试锁定。
- `docs/specs/auto-lang/ui/overview.md`（候选，review 时定稿）：登记两条——①壁纸加载
  优先级（712 T-13/T-14）的测试隔离规约（测试态须显式 `wallpapers_dir` 隔离，机器态
  不得泄漏）；②硬编码 `D:\Down\stella-os\wallpapers` 探测为登记债务（R5-②，待独立
  裁定挪 env 门控）。
- `touched_goals` 留空理由：docs/specs/goals.md 无「日常档红族治理」类条目，本计划为
  测试/数据面清偿，不映射 GOAL-NNN。

## 测试设计

- **每族 scoped 验证**：`cargo t --no-fail-fast <族过滤串>` 实际执行绿（输出粘执行步骤）；
  plan502 额外全档 ≥2 轮。
- **再生类（T-01）验证**：清 env 重编译后三围栏绿（UPDATE 期 panic 不算绿）；`cargo tu`
  验证 ui_gen 消费端；schema loader / component_registry 覆盖随全档。
- **env 隔离双形态说明**：AUTO_HTTP_BASE（T-06）与 AUTOTERM_ENGINE_DLL（T-07）——
  nextest 一测一进程；裸 libtest 并行下依赖仓内既有 env 守卫先例（执行时核对并采用）。
- **触面档**：本计划零产品逻辑变更；schema 数据面追平 → 随全档 + `cargo tu`；如执行中
  意外触及 VM/编译器生产实现，按 AGENTS 增补 `cargo tv`。
- 复审门禁=裸 `cargo t`（全日常面）。

## 验收标准

- [AC-01] 日常档 `cargo t --no-fail-fast` **0 failed**；ash 探针=可见 `#[ignore]`+集成
  门禁命令（本轮已真跑绿）；12 族**无任何静默 return/skip 冒充通过**；plan502 保留原
  几何断言。
- [AC-02] 12 族逐族 scoped 实际执行绿证据；R1 双臂（直调落账+缺实现报错）齐备；
  photo stub 有 stub 侧请求收据与精确 URL 映射断言；plan502 有全档多轮证据。
- [AC-03] KNOWN-DEBT 仅更新已验证成员、保留历史收据原文；R5-② 如实登记为独立债务；
  无「未执行记 pass」。
- [AC-04] 跨仓闭环=实际动作：auto-os 金样在独立 worktree 再生+门禁+提交（哈希回填）；
  auto-term 专用 worktree 构建+集成探针真跑绿；UPDATE 期 panic 不计绿。
- [AC-05] 范围如实：零产品逻辑变更；显式列出**编入产品的数据输入改动**（schema/aura.at
  别名追平、element_coverage 登记），既有条目零变更、消费端验证齐（loader 覆盖+cargo tu）；
  R5-② 明确排除并登记。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [ ] **T-00** 跨仓 worktree 布置（F-02 前置）
  `git worktree add D:/autostack/.wt/lang-759/auto-lang -b plan-759-dev`（基 master）；
  auto-os 兄弟 worktree 入同组目录（`.wt/lang-759/auto-os`，基其 master，记录
  base/commit/dirty）；auto-term 同组 worktree（仅构建用）。三仓红线：组内禁
  junction/symlink；移除前跑 `wt-guard.sh`。
  验证：`git worktree list` + 三仓状态记录。
- [ ] **T-01** R7 四维度清偿（别名再生/覆盖登记/跨仓金样）
  文件：`schema/aura.at`、`crates/auto-lang/src/aura/element_coverage.rs`、auto-os
  kitchen-sink.at + auto-lang 金样（AUTO_OS_ROOT 锚定 worktree）。
  验证：`cargo t --no-fail-fast schema_drift_fence queue_coverage_drift_fence kitchen_sink`
  （清 env 后）+ `cargo tu`。
- [ ] **T-02** p748_t00 整链 deadline + 泵
  文件：`crates/auto-lang/src/tests/plan748_breakpoint_probes.rs`（spawn_server 有界化/
  全链 deadline/join 后泵/noawait 臂不启 server/TempDir 作用域化）。
  验证：`cargo t --no-fail-fast p748_t00`。
- [ ] **T-03** p053_4 双臂重锁
  文件：`crates/auto-lang/src/tests/musk_vm_track_tests.rs:700-742`（臂 A 重锁）+ 新增
  低层 codegen fixture 臂 B。
  验证：`cargo t --no-fail-fast musk_vm_track_p053_4` + 新臂过滤串。
- [ ] **T-04** projector 双形态
  文件：`crates/auto-lang/src/ui/desktop_protocol/client_runtime.rs:1000-1070`
  （variant:"default" 5-Quad + 裸按钮 1-Quad 锁）。
  验证：`cargo t --no-fail-fast projector_counter`。
- [ ] **T-05** 壁纸双测隔离
  文件：`crates/auto-lang/src/ui/iced/renderer.rs` 两测试构造点+断言（空 TempDir 存活至
  测试结束；删宽松放行）。
  验证：`cargo t --no-fail-fast desktop_bus_inbox_drains desktop_surface_merge_dedupe`。
- [ ] **T-06** plan606 photo stub
  文件：`crates/auto-lang/src/tests/plan606_gallery_tests.rs`（stub+精确断言+env 隔离）。
  验证：`cargo t --no-fail-fast plan606_gallery`（时长应降至秒级；无 skip 路径）。
- [ ] **T-07** ash 门禁分层 + auto-term worktree 构建
  文件：`crates/auto-lang/src/vm/ffi/term_engine.rs:1214-1233`（env 定位+#[ignore] 分层）；
  动作：`.wt/lang-759/auto-term` 构建 DLL → 集成门禁真跑。
  验证：日常档显示 ignored + `AUTOTERM_ENGINE_DLL=… <集成命令>` 真跑绿。
- [ ] **T-08** plan502 完成态驱动
  文件：`crates/auto-lang/src/tests/plan502_diagram_tests.rs`（截止时间任务完成态驱动，
  原断言不变）。
  验证：scoped ×2 + 全档 ×2 无该红。
- [ ] **T-09** 收口销账
  文件：`docs/plans/KNOWN-DEBT-AND-RISKS.md`、本文件复审记录。
  验证：`cargo t --no-fail-fast` 全档 0 failed 终证据 + 跨仓收据齐。

## 复审记录

### 2026-10-11 草案独立审核（实现前，r1）

- stage: review（draft design audit） | plan_id: PLAN-759 | plan_revision: 未设（r2 已补）
- outcome: **needs_replan**（草案保持 drafting；没有 implementation pass）
- reviewed_commit / base_commit: `d2c2c0a3352a0596958eed51a02c7a405bad7584`
- dependency_revisions: auto-os `ab6b2660623703e1f4669a1bcd2674dbc0a05512`；auto-term `7a0d0e8b718fb260641f0d09ada95a7346771cd3`
- acceptance_results: AC-01/AC-05 设计合同 fail；AC-02/AC-03/AC-04 partial；未执行测试。
- findings: F-01～F-08，重点为 R7 声明/别名诊断错误、跨仓 UPDATE 可写主检出、跳过/移交与清零验收冲突。
- spec_inputs / evidence / 原 Plan hash / 逐项纠正建议：[完整审核报告](reports/759-plan-review-20261011.md)。规范增量尚缺，未给予通过结论。
- next: 修订草案并确认合同，之后在 lang-759 专用 worktree 实现；本次未改产品代码、未销债或合并。

### 2026-10-11 r2 修订（对 F-01～F-08 逐项收敛）

- F-01：R7 改四维度口径（canonical 已在场/别名同步/覆盖登记/金样同步）；撤回「产品
  diff=0」承诺，改「零产品逻辑变更+显式列出数据面改动+消费端验证（cargo tu）」。
- F-02：新增 T-00 跨仓 worktree 布置；KITCHEN_SINK_UPDATE 经 AUTO_OS_ROOT 锚定
  auto-os worktree、写入前核验解析路径；auto-term 专用 worktree 构建+env 定位 DLL；
  UPDATE panic=生成信号不算绿。
- F-03：删除 photo skip 与 DLL 静默 return 备选；DLL 显式 ignored+集成门禁+本轮真跑；
  T-08 转必做；验收口径统一为 0 failed+如实记录。
- F-04：R1 双臂均为必做（直调真实落账+缺实现 RuntimeError），缺实现臂用低层 codegen
  fixture 证发射链，host 桥状态显式隔离。
- F-05：deadline 覆盖全链（server 有界化+派发前建）；noawait 臂不启 server；TempDir
  作用域化。
- F-06：photo stub 补运行边界（真实条目 schema/端口 OS 分配/stub 请求收据/续体完成后
  读 View/精确 URL 映射/env 双形态说明）；壁纸只做空 TempDir 主案、删宽松放行、不动
  共享 helper 除非审计调用面。
- F-07：定性改为「12 项均已在收据登记；已定位成因为旧断言/环境/基建」，区分产品行为
  回归与新增红测试。
- F-08：补 plan_revision/affects/规范增量节（vm-fn-call-semantics 边界③+ui overview
  候选）/touched_goals 留空理由；销账口径改「只销已验证成员」。
- 用户裁定（2026-10-11）：保留补泵/photo stub/壁纸隔离主案；T-04 双形态并做。

## 待澄清事项

1. **auto-os 合流归属**：金样再生在 `.wt/lang-759/auto-os` worktree 分支提交后，合回
   auto-os master 的操作由本会话执行还是移交用户（其仓规门禁为准）？默认：分支提交+
   收据后移交用户合流。
2. ~~R5-② 壁纸硬编码探测门控~~：已裁定**移出本计划**，登记独立债务（规范增量节候选
   条目）；不作为消红前置。
3. ~~T-04 二选一~~：已裁定双形态并做（variant 5-Quad + 裸按钮 1-Quad 锁）。
4. ~~T-06 形态~~：已裁定 photo stub 主案唯一，删除 skip 备选。
