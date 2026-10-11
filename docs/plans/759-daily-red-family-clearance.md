---
plan_id: PLAN-759
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: daily-red-family-clearance
author: []
created_at: 2026-10-11
updated_at: 2026-10-11

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: []                   # 本计划全部为测试侧修复（唯一可能触产品码的 R5-②/R6-② 已拆为待澄清项）
current_step: 0
total_steps: 9
---

# [PLAN-759] 日常档 12 红族清偿（cargo t 零红收口）

## 变更摘要

`cargo t --no-fail-fast`（@3cdafcd40，2026-10-11 实测 5231 例）12 红。**定性：零近期回归**——
当日批量回归收据（3cdafcd40，覆盖 746-758 窗口）`new_reds: []`，12 红全部在册。它们是
2026-09 以来多次「产品语义有据变更、测试/再生流程未同步」的累积旧账 + 两处测试基建缺口 +
一处跨仓构建依赖缺失。本计划逐族清偿至日常档 0 红（或唯一残留=登记保留项），并同步销
KNOWN-DEBT 对应条目。

除两个待澄清的可选产品侧子项（R5-② 壁纸硬编码探测门控、R6-② DLL 自守卫）外，**全部
修复落在测试侧**（测试代码/测试基建/schema 再生产物），零产品行为变更。

## 目标

1. `cargo t --no-fail-fast` 从 12 红降到 0 红（含跨仓协调项落地或有明确移交收据）。
2. 每族修复附带 scoped 验证证据（失败过滤串单跑绿）。
3. KNOWN-DEBT-AND-RISKS.md 对应条目销账：P733-R2③、REG-4、712 §10② 基线红族、
   735 收据 ash 预存登记、P733-R1（若 T-08 实施）。
4. 负载 flake（plan502_m3）至少有鲁棒化处置或维持登记并降频实证。

## 架构方案

无架构变更。修复分四类手段：
- **A 陈旧断言重锁**（R1/R4）：产品语义变更有据（用户裁定/计划裁定），测试按现行契约重录。
- **B 测试隔离/基建补齐**（R2/R3/R5）：补 in-proc 泵循环、stub 后端、隔离机器态目录。
- **C 再生流程补课**（R7）：走仓内既有 regen 通道（SCHEMA_DRIFT_GENERATE_AT /
  KITCHEN_SINK_UPDATE / element_coverage 登记），跨仓侧需 auto-os 配合提交。
- **D 环境依赖自守卫**（R6/R8）：DLL 缺席 skip、驱动预算自适应。

## 需求分析与背景调查

（诊断于 2026-10-11 master@3cdafcd40 完成；三路并行调查 + 收据/账本交叉核对）

### 根因矩阵（12 红 → 8 根因簇）

| # | 测试 | 根因簇 | 引入点 | 类型 |
|---|---|---|---|---|
| 1 | `musk_vm_track_p053_4_merged_api_warning::merged_mode_api_call_emits_warn_opcode` | R1 | `4d5f8de13`（PLAN-633 T-04，09-15） | 陈旧测试 |
| 2 | `plan502_diagram_tests::plan502_m3_layout_geometry_e2e` | R8 | P733-R1 在册（双树同发） | 负载 flake |
| 3/4 | `plan748_breakpoint_probes::p748_t00_handler_{static,dynamic}_arm` | R2 | `e9d953ec4`（PLAN-748 T-00，10-10）生而红 | 测试基建缺口 |
| 5 | `plan606_gallery_tests::test_029_photo_gallery_thumbnails_render_with_resolved_src_and_cover_fit` | R3 | `231cca2fd`（PLAN-043 Part1，09-24） | 陈旧测试+缺 stub |
| 6 | `ui::desktop_protocol::client_runtime::tests::projector_counter_layout_and_hits` | R4 | `b4c2dc38b`（PLAN-080 T-03，09-20） | 陈旧测试 |
| 7 | `ui::iced::renderer::tests::desktop_bus_inbox_drains_through_real_arms` | R5 | `74fbc8085`+`d990427ba`（712 T-13/T-14，09-30） | 测试隔离缺口（机器相关） |
| 8 | `ui::iced::renderer::tests::desktop_surface_merge_dedupe_and_injection` | R5 | 同上 | 同上 |
| 9 | `vm::ffi::term_engine::ash_leak_probe::ash_stream_leak_probe` | R6 | auto-term target 未构建（DLL 缺失） | 跨仓环境依赖 |
| 10 | `schema_drift::schema_drift_fence` | R7 | `5bad66e0f`（Plan 717，10-01）漏再生 | 再生流程欠账 |
| 11 | `schema_drift::queue_coverage_drift_fence` | R7 | 同上（漏登记） | 同上 |
| 12 | `docs_gen::kitchen_sink_page_in_sync` | R7 | `5bad66e0f` + auto-os `73d02b5`（REG-4 在册） | 跨仓金样脱同步 |

### 各根因簇机理（证据链）

**R1 p053_4（CALL_NAT 3142）**：merged 模式（`api_over_http=false`）下 `#[api]` 裸调用现为
三分支（`crates/auto-lang/src/vm/codegen.rs:9426-9456`）：① host 桥注册→`auto.host.call`；
② **api 实现体已随 imports 编入模块（`exports` 含该函数）→ 不发桩、直落常规直调 CALL**
（PLAN-633 守卫 `!self.exports.contains_key(...)`，`codegen.rs:9445-9453`）；③ 实现体缺席→
才发 `CALL_NAT 3142`。测试 fixture 把 `#[api] fn chats_list_sessions` 放进 import_stmts →
声明期即注册导出（`codegen.rs:1560`；imports 先于 handler 编译，
`ui/handler_codegen.rs:1576-1685`）→ 命中分支②，字节码无 3142 → 断言失败。发射代码未丢，
是语义有据变更后的陈旧测试（KNOWN-DEBT P733-R2③ 所谓「codegen 面异根」即此守卫）。
另：PLAN-734 D7（`beec5b94f`）后 3142 原生语义已从「warn no-op」演进为带指引的
RuntimeError，测试名 `merged_mode_api_call_emits_warn_opcode` 双重过时。

**R2 p748_t00 双臂**：生而红，非 master 回归。T-00 定音报告（748 归档 §9
`docs/plans/archive/748-vm-cond-eval-and-field-write.md:615-618`）明确登记「comp 根件
in-proc 不泵 Http 续体（harness 边界：server 已应答而 marker 恒 unset）」；绿的 W:5 证据
来自 E2E 腿（`auto run --render=vm` 真实管线），T-01「三臂全绿」指 fn 段探针
（`p748_t00_resume_static_write` 等，今仍绿）。机理：测试路径
`p748_store_handler_write_face`（`crates/auto-lang/src/tests/plan748_breakpoint_probes.rs:322-393`）
两次 `drive_scheduler_to_quiescence` 后 `server.join()`，**此后无任何泵调用**——生产中续体
由 iced 事件循环 `__parked_resume_tick` 订阅→`poll_parked_resumes`（`ui/dynamic.rs:1699`）
驱动，in-proc harness 无该循环；`drive_scheduler_to_quiescence` 第二轮探
`async_http_result_ready` 未就绪即 `progressed=false` 早退（`ui/dynamic.rs:1572-1576`）。
落前后历次收据（e8ba2d323、738-review-r9-baseline.json、755-external-audit-response.md）
均记在册红，从未有绿记录；嫌疑提交（be51414da/7d51954d4）已排除（diff 零功能变更/异域）。

**R3 plan606 test_029**：`231cca2fd`（PLAN-043）把 029-photo-gallery 从本地种子数据改为
Init 经 `Http.get_json("/api/photos/scan?dir=")` 拉后端 photo_service。单测进程无后端、
`AUTO_HTTP_BASE` 未设 → 相对 URL 传输失败 → 按 PLAN-712 T-17 语义抛可捕获异常 →
catch 臂置 `.photos=[]`（`examples/ui/029-photo-gallery/src/front/app.at:466-486`）→
空态视图零 image 节点 → 泵谓词永假 → 10s deadline（`plan370_test_support.rs:441-450`），
10.2s 时长即此。**即便起了后端现断言仍红**：thumb src 现为绝对 URL
`{base}/apps/{app_id}/api/photos/thumb/{id}`（`back_proxy.rs:1041-1042`），不含
"thumbnails"/不以 .jpg 结尾；`OpenPhoto` 签名已改 str，测试仍发 Int(1)。be51414da
（iced 渲染端 URL 展开）与此红无因果（红早它 15 天，且本测试不跑 iced 渲染）。

**R4 projector_counter**：断言「按钮=底盒+四边描边」`quads.len()==5`
（`ui/desktop_protocol/client_runtime.rs:1058`）。裸按钮 border 类来源链：PLAN-571 缺省
`""`→UA 基线（含 border）；**`b4c2dc38b`（PLAN-080 T-03「裸 button 缺省 chromeless」）把
`ui/style/variants.rs:30` 的 `"" => ""` 改空** → `style.border=None` → 四条描边 Quad 消失
（`native_projector.rs:1927-1929` 仅 border Some 时 push）。产品行为=用户裁定，测试未跟。
时间线：`98b778f44`（09-19，Plan 033 T-04）重录 5-Quad（彼时 PLAN-571 基线成立，绿）→
b4c2dc38b（09-20）→ 首红证 `e0e2d6b52`（09-21）→ 09-30 首次批量收据在册。

**R5 壁纸双测**：712 r2 T-13（`74fbc8085`）重排 `load_desktop_wallpaper`
（`ui/iced/renderer.rs:16972`）优先级——#hex 从「直传短路」降为「仅无目录机器」档④，
档②为新插入的分主题默认图（T-14 `d990427ba`，深色 `purple.png`）；而目录解析链末端是
**硬编码机器探测 `D:\Down\stella-os\wallpapers`**（`renderer.rs:17031-17033`，PLAN-526 T19
「机器便利默认」）。测试会话用 `DesktopConfig::default()`（`wallpapers_dir:""`）且不设
`AUTO_DESKTOP_WALLPAPERS_DIR` → 本机目录存在 → 档②命中 →
`desktop_wallpaper="D:\Down\stella-os\wallpapers\purple.png"` ≠ `"#101014"`。
- `desktop_bus_inbox_drains_through_real_arms`（`renderer.rs:35143`）：SetWallpaper("#101014")
  后断言 `desktop_wallpaper=="#101014"` 落空。
- `desktop_surface_merge_dedupe_and_injection`（`renderer.rs:38013`）：快照构建里
  `starts_with('#')` 才生成 `bg-[#243b55]` 片段（`renderer.rs:17555-17559`），壁纸被图片
  路径覆盖 → `__desktop_bg==""` ≠ `"bg-[#243b55]"`。
**机器相关红**：无 stella 目录的机器（CI）至今绿。产品语义=T-13 用户裁定（「任何时候打开
都加载默认壁纸」，T-13 自带新单测已双结果放行），旧测试未同步。

**R6 ash 探针**：`ash_stream_leak_probe`（`vm/ffi/term_engine.rs:1222`）硬编码
`AUTOTERM_ENGINE_DLL=D:\autostack\auto-term\target\debug\autoterm_core.dll`
（:1224-1227），该 DLL 现不存在（auto-term target 未构建/已清理）→
`engine_spawn` 返回 0 → 断言红。跨仓构建依赖，任何 auto-term 侧 clean 都会复发。

**R7 schema 三围栏**：Plan 717（auto-lang `5bad66e0f` 2026-10-01 + auto-os `73d02b5`）
落地 carousel/command/Calendar 组件族，**三道再生流程全未走**：
- `schema_drift_fence`：生产表新 tag（Calendar, carousel-content/item/next/previous,
  command-empty/group/input/item/list/separator/shortcut）未被 `schema/aura.at` 覆盖
  （`tests/schema_drift.rs:1946`）。修法=仓内标准通道
  `SCHEMA_DRIFT_GENERATE_AT=1 cargo test -p auto-lang --test schema_drift` 再生。
- `queue_coverage_drift_fence`：carousel 五元素未在 `src/aura/element_coverage.rs` 登记
  （`tests/schema_drift.rs:2282`；登记动作不可省，默认 not-yet 即过）。
- `kitchen_sink_page_in_sync`：kitchen-sink.at（auto-os 仓）与 schema 金样脱同步
  （`tests/docs_gen.rs:406`）= REG-4 在册（根因 auto-os 73d02b5 手改未走
  `KITCHEN_SINK_UPDATE=1` 再生）。清偿=REG-4 既定路径：auto-lang 侧再生 + auto-os 侧
  提交重采样金样。

**R8 plan502_m3**：P733-R1 在册负载 flake——全量并行档下 `handler_FlowDiagram_Init` 被
CPU-slice Parked(CpuRunnable)，片预算耗尽，测试驱动轮次内未续完 → 几何断言落旧值
（x=108）；单跑恒绿（本轮 scoped 亦绿，与 10-11 收据「scoped 2/2 绿」一致）。

## 详细设计

（按工作量从小到大排序；各族互相独立，可并行/任意序）

### T-01 R7 schema 三围栏清偿
1. `SCHEMA_DRIFT_GENERATE_AT=1 cargo test -p auto-lang --test schema_drift` 再生
   `schema/aura.at`；复核 diff——预期仅新增上述 13 tag 的 element 声明。
2. `crates/auto-lang/src/aura/element_coverage.rs` 登记
   `carousel, carousel_content, carousel_item, carousel_next, carousel_previous`
   （默认 not-yet/not-consumed 档，照抄邻近条目形态）。
3. `KITCHEN_SINK_UPDATE=1 cargo test -p auto-lang --test docs_gen` 再生 kitchen-sink
   金样；**跨仓协调**：auto-os 仓 widgets-gallery/src/front/pages/kitchen-sink.at 需按
   REG-4 既定路径配合提交（重采样后金样回写）。
4. 不带 env 重跑三测试确认绿。

### T-02 R2 p748_t00 泵循环补齐（保回归面，推荐案）
在 `p748_store_handler_write_face` 的 `server.join()` 之后加带 deadline 的泵循环（与生产
`__parked_resume_tick` 同机制，先例 `plan370_test_support.rs:433-443`、
`plan622_store_facade_gap_tests.rs:261`）：
```rust
server.join().expect("server thread");
let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
loop {
    comp.drive_scheduler_to_quiescence(1_000);
    if marker 已翻转（read_state("marker") 非 "unset"） { break; }
    assert!(std::time::Instant::now() < deadline, "parked 续体 5s 未恢复");
    std::thread::sleep(std::time::Duration::from_millis(5));
}
```
static/dynamic 双臂同修（共用该 harness 函数）。产品代码零改动。
（备选：`#[ignore]` 登记——弃，丢 Plan 748「探针钉死为回归面」意图。）

### T-03 R1 p053_4 断言重锁现行契约
1. 主断言改锁分支②：`module.exports` 含 `chats_list_sessions`，且 handler 对该导出发出
   模块内直调 CALL；注释改引 PLAN-633（`4d5f8de13`）。
2. （可选补覆盖）分支③ 3142 发射路径无法经 `synthesize_widget_module` 表达（api_funcs
   与 exports 同源于 import_stmts），改做行为测试：运行缺实现模块断言 RuntimeError 文案
   含 "has no implementation"（对齐 PLAN-734 D7，`stdlib.rs:7971-7990`），并顺手把测试名
   的 `warn_opcode` 措辞修正（语义已是硬错误）。
3. 销账：KNOWN-DEBT P733-R2 条目③ 措辞更新为已清偿。

### T-04 R4 projector 断言重录
按 PLAN-080 语义把「按钮=底盒+四边描边」`quads.len()==5` 重录为 `==1`（裸按钮
chromeless，恰回归 Plan 480 原始期望）；**或**改 `COUNTER_SRC` 为
`button "+" { variant: "default", onclick: ... }` 保留 5-Quad 描边覆盖。二选一（倾向
后者：variant 路径覆盖面更大，另加一条裸按钮 1-Quad 锁测试防再漂）。

### T-05 R5 壁纸双测隔离
1. （测试隔离，主修复）两测试会话构造处（`__test_session()` /
   `t3_session_with_shell()`，`renderer.rs:34115` 附近）设
   `config.wallpapers_dir = <空临时目录>`——一行挡掉 env/stella 两级探测，恢复 #hex
   直传确定性。或断言面改为 `config.wallpaper_path == "#101014"`（记忆面，确定性）+
   `__desktop_bg` 按 T-13「双结果放行」（`== "bg-[#243b55]" || Path::is_file()`）。
2. （根治项，待澄清#1）硬编码 `D:\Down\stella-os\wallpapers` 探测
   （`renderer.rs:17031-17033`）挪到 env/配置显式开启——dev 便利不该进库代码路径，
   PLAN-526 遗留机器绑定债；涉及产品行为，须用户裁定后另立子项。

### T-06 R3 plan606 test_029 重构（工作量最大，可独立成腿）
首选：测试内起 in-process photo stub（仓内先例 `back_proxy_tests.rs`、
`plan349/plan341_tests.rs` 的 tiny_http/TcpListener），serve `/api/photos/scan` 返回固定
24 条 + `/thumb/:id`；设 `AUTO_HTTP_BASE` 指向 stub；断言改现行契约：24 张、src 为
`api/photos/thumb` 形态、Cover fit；`OpenPhoto` 改发 `Value::str(id)`，全图断言
`api/photos/full` + Contain。（降级备选：fit→StyleClass::ObjectFit 回归收缩到最小 .at
fixture，029 整链移交 th 档；最低止损：沿用文件内既有 SKIP 模式探测后端不可达即
eprintln+return，先消 10s/日 红烧。）

### T-07 R6 ash 探针环境修复 + 自守卫
1. （环境动作，非提交物）`cd D:/autostack/auto-term && cargo build -p autoterm-core`
   产出 debug DLL（测试硬编码路径即指向它）。
2. （自守卫，待澄清#2）DLL 缺席时 skip+eprintln 而非红——跨仓二进制依赖的测试在任何
   auto-term 侧 clean 后必复发；守卫保有 auto-term 已构建机器上的覆盖。

### T-08 R8 plan502_m3 鲁棒化（可选，低优先）
驱动轮次预算按 parked 续跑自适应（P733-R1 既登记方向）；或维持观察名单（本轮与 10-11
收据均 scoped 绿，频率未上升）。不实施则本计划验收口径注明残留。

### T-09 收口与销账
1. 裸 `cargo t --no-fail-fast` 全档证据（预期 0 红或仅剩 T-08 残留登记项）。
2. KNOWN-DEBT 销账/更新：P733-R2③、REG-4、712 §10② 基线红族中对应成员、735 收据 ash
   预存登记、（如实施）P733-R1。
3. 跨仓收据：auto-os kitchen-sink 提交哈希回填（REG-4 清偿闭环）；auto-term DLL 构建
   记录。
4. flake 双例（plan502/plan484） scoped 复跑绿记录。

## 测试设计

- 每族 scoped 验证：`cargo t --no-fail-fast <族过滤串>` 单族绿后合入。
- 全档收口：`cargo t --no-fail-fast`（约 2 分钟）零新红且目标红清零。
- 再生类（T-01）验证：不带 `SCHEMA_DRIFT_GENERATE_AT`/`KITCHEN_SINK_UPDATE` 重跑三围栏绿。
- 本计划零产品行为变更（除待澄清项），无需跑 tv/tt/tb 触面档；T-01 触 schema/docs 生成
  产物，复审门禁按 Category C 跑 `--test docs_gen`/`--test schema_drift` 已含在 scoped 验证内。

## 验收标准

- [AC-01] `cargo t --no-fail-fast` 全档 0 红（若 T-08 不实施，唯一允许残留=
  plan502_m3 负载 flake 且 P733-R1 维持在册）。
- [AC-02] 12 红逐族有 scoped 绿证据（过滤串+结果粘进执行步骤）。
- [AC-03] KNOWN-DEBT 对应条目销账/措辞更新完成。
- [AC-04] 跨仓项有闭环：auto-os 金样提交回填、auto-term DLL 构建记录（或明确移交单）。
- [AC-05] 产品代码 diff = 0（除非待澄清#1/#2 获裁定后追加的子项，须单独标注）。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [ ] **T-01** schema 三围栏清偿（R7）
  文件：`schema/aura.at`（再生）、`crates/auto-lang/src/aura/element_coverage.rs`（登记）、
  kitchen-sink 金样（再生）+ auto-os 仓配合提交。
  验证：`cargo t --no-fail-fast schema_drift_fence queue_coverage_drift_fence kitchen_sink`。
- [ ] **T-02** p748_t00 泵循环（R2）
  文件：`crates/auto-lang/src/tests/plan748_breakpoint_probes.rs`（server.join 后泵循环）。
  验证：`cargo t --no-fail-fast p748_t00_handler`。
- [ ] **T-03** p053_4 断言重锁（R1）
  文件：`crates/auto-lang/src/tests/musk_vm_track_tests.rs:700-742`。
  验证：`cargo t --no-fail-fast musk_vm_track_p053_4`。
- [ ] **T-04** projector 断言重录（R4）
  文件：`crates/auto-lang/src/ui/desktop_protocol/client_runtime.rs:1000-1070`。
  验证：`cargo t --no-fail-fast projector_counter_layout_and_hits`。
- [ ] **T-05** 壁纸双测隔离（R5-①）
  文件：`crates/auto-lang/src/ui/iced/renderer.rs`（`__test_session`/`t3_session_with_shell`
  构造处 + 两断言）。
  验证：`cargo t --no-fail-fast desktop_bus_inbox_drains desktop_surface_merge_dedupe`。
- [ ] **T-06** plan606 stub 重构（R3）
  文件：`crates/auto-lang/src/tests/plan606_gallery_tests.rs`。
  验证：`cargo t --no-fail-fast plan606_gallery`（时长应从 ~10s 降到秒级）。
- [ ] **T-07** ash 环境修复+自守卫（R6）
  动作：auto-term 侧 build；文件：`crates/auto-lang/src/vm/ffi/term_engine.rs:1214-1233`。
  验证：`cargo t --no-fail-fast ash_stream_leak_probe`。
- [ ] **T-08** plan502 鲁棒化（R8，可选）
  文件：`crates/auto-lang/src/tests/plan502_diagram_tests.rs:284` 附近驱动预算。
  验证：全档两轮 + scoped 两轮。
- [ ] **T-09** 收口销账
  文件：`docs/plans/KNOWN-DEBT-AND-RISKS.md`、本文件复审记录。
  验证：`cargo t --no-fail-fast` 全档终证据。

## 复审记录

## 待澄清事项

1. **R5-② 壁纸硬编码探测门控**（`renderer.rs:17031-17033` 的
   `D:\Down\stella-os\wallpapers` 机器便利默认）：是否随本计划一并挪到 env/配置显式开启？
   触产品行为（无目录机器不受影响，本机行为改变），须用户裁定；不裁定则本计划只做
   T-05-① 测试隔离。
2. **R6-② ash 探针自守卫**（DLL 缺席 skip 而非红）：会弱化「忘记构建 auto-term」的可见
   性，换跨仓 clean 后不复发；倾向加守卫（eprintln 明示原因），请确认。
3. **T-04 二选一**：断言重录 1-Quad（最小）vs `variant:"default"` 保 5-Quad 覆盖+裸按钮
   锁（覆盖面大）；默认取后者。
4. **T-06 形态**：首选 in-process stub（保端到端契约，工作量中等）vs 降级离线 fixture /
   最低止损 SKIP；默认取首选。
