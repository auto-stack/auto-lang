# Frozen proposed Spec delta


| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-media-core/project.md | UI目录的中立媒体机制→独立单源类型/共享服务、资源/寿命/支持合同 | 可独立消费 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-lang/vm/overview.md | 17 native及HTTP UI直连→非UI服务/编解码、开关及stub冻结 | VM脱UI类型 | AC-01, AC-02, AC-04 |
| SD-03 | modify | docs/specs/auto-lang/ui/overview.md | 图像/媒体索引/Range机制迁core、renderer及剩余owner真实列明 | 不虚报全UI独立 | AC-01..05 |
| SD-04 | modify | docs/specs/auto-lang/ui/design/image-surface-events.md | 规则保持，registry/代次owner改core、事件与渲染拍留UI | 保护loaded/error | AC-04 |
| SD-05 | modify | docs/specs/auto-man/project.md | 媒体生成/HTTP接线消费新core；保留PLAN-809已落地的auto-man-core规则源与语言效果adapter分工，UI工具等剩余owner如实列明 | 对齐当前canonical的809/R04边界 | AC-04, AC-05 |
| SD-06 | modify | docs/specs/repository-boundaries/generated-ui-distribution.md | 新core及codec/source闭包进入受影响交付profile | 独立可分发 | AC-05 |


## Handoff SD draft
（merge 时沉淀 canonical；本轮不私改）

- **SD-01 add `docs/specs/auto-media-core/project.md`**：crate 使命/公开面
  （image_pipeline 全 API+MediaRuntime+media_service）/资源合同（预算表、
  1GiB/400MP、TTL 双值 30s 全局/2s 常量）/生命周期合同（shutdown 终态
  化、无复活、满队列有界收敛、锁序：无用户回调/无 registry 锁内等
  worker）/发行合同（image-codec feature、闭包断言）。
- **SD-02 modify `vm/overview.md`**：17 native+HTTP 快路径消费
  auto-media-core（直调，非 UI 门面）；ID/按名注册/feature 门/stub 语义
  冻结表；控制面不搬像素/路径。
- **SD-03 modify `ui/overview.md`**：image_pipeline/media_service 迁 core、
  门面身份同一；renderer/photo_service/mpv/i18n/frame 等 UI 剩余 owner
  真实列明（不虚报全 UI 独立）。
- **SD-04 modify `ui/design/image-surface-events.md`**：规则不变
  （Ready 后单发/形参投影/换源退订/A→B→A/50ms 代次）；registry/代次/
  pending 门 owner 改 core，事件与渲染拍留 UI。
- **SD-05 modify `auto-man/project.md`**：api_gen/rust_ui 媒体生成直调
  core；生成 manifest 携带 auto-media-core（workspace/path/交付闭包）；
  项目模型/UI 工具搬迁仍未发生（809/R04 边界）。
- **SD-06 modify `repository-boundaries/generated-ui-distribution.md`**：
  DELIVERY_CRATES +auto-media-core（image/exif/blake3/fastrand 走
  registry）；受影响 profile 的 fresh/locked 合同由 792 钉住。


## R1 effective supplement
（2026-10-19，review-r1 needs_fix 后）

修复细节与逐 finding 闭合表见 [fix-r1.md](fix-r1.md)；要点：

- 代码基线推进：`0f2ab56a9` → **`33fc23208`**（`c66ca5a6a` 生命周期/准入/
  格式 + `33fc23208` 生成面两修复）。
- §3 表修正：core 单测 30/30 → **34/34**（新增 5 个关闭 oracle，重写 1 个
  为真 PNG+外截止）；「V-05 pending」**撤销**——四形态真机收据全绿
  （VM/Iced 9/9、Vue split 7/7、Rust merged 6/6、生成媒体 HTTP 13/13，
  见 [v05-receipts/](v05-receipts/)）；777/792 收据绑定 `0f2ab56a9`，
  修复轮未触其判定面（模板两行级修复经 api_gen 50/50 + delivery 9/9 +
  rust_ui 40 PASS 复验；merge 前在冻结提交复核 strict 一次）。
- 「shutdown 无复活/满队列有界收敛」等 §5 SD 草案措辞在修复后为真实
  （dead-ticket 准入语义补入 SD-01/02 草案：关闭后 queue 返回死票据、
  open_session 返回空串——公开签名不变）。
- 新增 R04 具名交接：merged 模式 ImageSurface loaded/error 事件分发
  （sweep 未接 devtools_update，PLAN-095 扩展）；sweep 静止期绑定边界；
  031 生成后端 types.at companion/`crate::image` 发射破损（master 基线
  同款，归因收据在 fix-r1）；Man wire 档基线编译红。

## R2 修复轮增补（2026-10-19，review-r2 needs_fix 后）

修复细节见 [fix-r2.md](fix-r2.md)；代码基线推进至 **`4a1d8fcf3`**：

- R2-01（关闭竞态）：open_session 锁内复查 closing（与清表同 mutex）+
  spawn 结果消费；复审 R2 探针重放 `returned_session=""`/unknown/close=false。
- R2-02（oracle）：bounded_teardown（超时红不 join，负控 <800ms 红常驻）；
  resize 通道满以采样 running 峰值 ≥9 实证；scan 活动门 + 新 per-registry
  mutation_generation 静止断言。core 34→**38/38**。
- R1-03（事件合同）：sweep 早退门重排（静止期注册修复，VM 真机
  quiescent-Late 反例）；**merged 类型化 sweep 接线**（devtools_update，
  两拍/单发/随视图生死）；四 harness 全矩阵收据（VM 12/12 含形参投影/
  卸载，Vue 8/8 含真 `<img>` error 事件+恢复，merged 11/11 含静止期/
  卸载/恢复，生成 HTTP 13/13），全部嵌 provenance（commit/源码/二进制
  hash），探针源冻结于 v05-harness/probe-sources/。
- 792 driver 在修复树上重跑 **25/25**（收据 v05-receipts/
  driver-792-r2fix.json；一次 V02 红为 scratch 增量缓存损坏
  "cached cgu should have an object file"，清 scratch 后复绿——环境
  工件非代码回归）。

## R3 修复轮增补（2026-10-19，review-r3 needs_fix 后）

修复细节见 [fix-r3.md](fix-r3.md)；代码基线推进至 **`27d3387af`**：

- R3-01（订阅身份）：typed 键 = handler Debug 身份 + src（对齐动态轨）；
  复审 typed-consumer 四行重放全绿（same/swap/bad-same 修复前后对照见
  fix-r3）。
- R3-02（独立唤醒）：merged 门控 50ms 媒体心跳（pending/待派发门，
  空闲零消息）+ 全出口 sweep + 进程级表；无 timer 探针 5/5 证明。
- R2-02 余量：双负控（协作回收+真 owner swap 门回收）常驻；resize 占用
  仪表 backlog≥8=容量满；scan 活动门真落盘（R2 补丁锚点未命中事故已
  在 fix-r3 如实记录）。core **40/40**。
- 全矩阵收据重绑本提交：VM 12/12、Vue 10/10、merged 11/11、
  identity(no-timer) 5/5、生成 HTTP 13/13。

## R4 修复轮增补（2026-10-19，review-r4 needs_fix 后）

修复细节见 [fix-r4.md](attachments/fix-r4.md)；代码基线推进至 **`53f1111a2`**：

- R4-01（仪表下溢）：Work-only 预登记计数（Stop 不计、send 前预留、
  受控读序）——复审重放 `after_idle_shutdown_panicked=false value=Some(0)`。
- R4-02（关闭语义）：closing/closed 分离 + 完成等待（首调用独占、后续
  调用等同一结果；返回即完全关闭）——复审重放 `second_returned→完成/
  registry_shutdown=true/无新票`；真实 owner 负控改判完成合同。
- R3-02 余量：typed_media_wake 双入口（sync + run_app_with_task_devtools）
  + TYPED_RESWEEP 门（回调内换图后续拍）——复审 typed-consumer
  async-no-timer a=1、cascade a=1 b=1；本仓 async 探针 4/4、identity 6/6。
- core 42/42；六 harness 全部在**提交后**树实跑并重绑 `53f1111a2`
  （VM 12/12、Vue 10/10、merged 11/11、identity 6/6、async 4/4、
  生成 HTTP 13/13）；792 修复树重跑 25/25（driver-792-r4fix.json）。
- provenance 流程修正：先提交→重建→跑收据（R4 §6 指出的 WIP-hash 问题
  不再发生）。

## R5 修复轮增补（2026-10-19，review-r5 needs_fix 后）

修复细节见 [fix-r5.md](attachments/fix-r5.md)；代码基线推进至 **`2ffb87755`**：

- R4-01 余量（在途关闭 backlog 残留 1）：**关闭顺序重排**（先 join
  decode 发送方→再 Stop→再 join lane，堵死 Stop 后接受窗口）+ lane
  stopping 快路径结算；语义诚实化（accepted-after-send、阻塞 send 不计、
  读序方向修正）。复审 occupancy-r5 探针三试 `after_shutdown=0`。
- R5-01（P3）：unused import 清除，test cfg 零 warning。
- core **43/43**（新增在途关闭归零控制）；六 harness 重绑 `2ffb87755`
  全绿；792 修复树重跑 25/25（driver-792-r5fix.json）。


## R4 canonical reconciliation
SD-05 uses the current Plan row: preserve PLAN-809 auto-man-core rules and language/effect adapter split already deposited in docs/specs/auto-man/project.md. Earlier handoff statements that project rules have not migrated are historical and superseded by this refinement. Acceptance criteria and Plan revision remain unchanged.
