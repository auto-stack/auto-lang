# PLAN-811 R1 修复轮报告（review-r1 needs_fix → 修复闭合）

2026-10-19 · stage: work · 针对 [review-r1](../review-r1/review.md) 四项
findings 的修复、oracle 补强与真实验收收据。修复树：同 worktree
`D:/autostack/.wt/lang-811/auto-lang`（分支 `codex/plan-811-dev`），基线
（送审冻结）`0f2ab56a9`，本轮修复提交：`c66ca5a6a`（F01/02/04 生命周期
+准入+格式）、`33fc23208`（F03 面：生成媒体路由语法+merged 成员依赖）。

## Finding closure 表

### F811-R1-01 · P1 · shutdown/drop 未管理生产者线程

- **invariant**：runtime 关闭必须统一拥有 scan/open/decode/resize 的退出
  与准入——关闭返回即意味着生产者已回收，registry 此后不可变更。
- **affected paths/states**：`scan_directory`/`open_session` 的 detached
  spawn（持 `Arc<RuntimeInner>`，无取消检查、无 join）；`shutdown()` 原
  序只关 worker→registry→sessions；Drop 同路。
- **fix**：`RuntimeInner` 增 lock-free `closing` 锁存 + `producers` 跟踪表；
  spawn 注册与 shutdown 排空共用同一 mutex（生产者要么被 join 要么根本
  未生——无注册竞态）；scan/populate 逐项检查锁存退出；shutdown 固定序：
  置锁存 → 锁外 join 生产者 → worker 泳道 → registry → 会话表；Drop 复用。
- **permanent cases**（`crates/auto-media-core/src/image_pipeline.rs` 测试）：
  `scan_shutdown_reaps_producer_and_registry_stays_quiescent`（600 文件
  扫描在途关闭：stats 双采样静止 + 关闭后立即删 temp 树=Windows 句柄释放
  证据）；`shutdown_with_real_decode_backlog_completes_within_external_deadline`
  （真 PNG 过 decode→resize 泳道 + scoped 看门狗外截止——旧版空字节在
  metadata 即死、同步断言看不见卡死）；`shutdown_interleaved_with_concurrent_
  submit_and_open`（关闭与提交/开会话交错后静止）。
- **current results**：复审 replay.py 原探针在本树三跑全绿
  （`producer_continued=false`×3，Drop 后媒体代次 9/12/99 逐静止——修复前
  同探针三跑 `producer_continued=true` 且代次持续前移）；core 34/34。
- **remaining gap**：无（会话打开线程与 scan 同走 tracked-producer 通路；
  未另做受控暂停实测——populate 逐项检查与 scan 同码路，由交错测试覆盖）。

### F811-R1-02 · P2 · 关闭未封住新票据发布与会话准入

- **invariant**：关闭锁存与 queue/会话/scan 准入、状态更新、发布共用一致
  同步规则；关闭后的运行时不再创建可发布状态或有效会话。
- **affected paths/states**：`MediaAssetRegistry::queue/transition/
  publish_ready/fail/update_metadata/retain` 均不检查 `shutting_down`；
  `open_session` 关闭后仍入表+派生线程；`scan_directory` 同。
- **fix**：registry 六个变更入口统一加 `shutting_down` 门（签名不变：
  queue 返回 DEAD ticket——新随机 id 不入表，后续操作一律 Missing/拒绝，
  不复活状态）；`open_session` 关闭态返回 `""`；`scan_directory` 关闭态
  no-op；重复 shutdown 幂等。
- **permanent cases**：`admission_after_close_is_refused_everywhere`
  （dead-ticket 全操作拒绝、会话/scan 准入、双关闭、新 runtime 与进程默认
  registry 正控——默认永不关闭）。`drop_releases_decoded_pixel_bytes`
  （Weak 释放证据：drop 后 upgrade 失败）。
- **current results**：replay ADMISSION 探针全绿（`fresh_pending=0
  transitions/publish/ready=false new_session=unknown_session`——修复前
  全 true 且新会话带活线程）；core 34/34。
- **remaining gap**：无。

### F811-R1-03 · P2 · 真实三形态/事件/生成媒体 HTTP 验收未完成

- **invariant**（AC-04）：VM/Iced、Vue split、Rust merged 真实媒体运行 +
  ImageSurface 渲染后事件 + 生成媒体 HTTP GET/HEAD/ETag/错误/非媒体路由
  与实际 Range 流输出，收据绑定提交/输入/原始结果；不得以 mock/字符串替
  代。复审许可：可用隔离真实生产 harness 避开无关基线编译红。
- **交付**（harness 在 [v05-harness/](../v05-harness/)，收据 JSON+截图在
  [v05-receipts/](../v05-receipts/)，均为本轮实跑原始结果）：
  - **VM/Iced**（`vm_media_run.py` → `vm_media_run.json`，9/9）：真
    `auto run -r vm` 窗口+MCP；探针应用（front 直调 auto.image+SettleTick
    80ms）经按钮步进：open→bind→**loaded 恰好 1 次**（两拍语义在真渲染拍
    后单发）→16 采样静止（无事件风暴）→**A→B→A 精确 URI 等值**（Next 走
    request_view、Prev 绑原票据）→**坏票 error 恰好 1 次**→重绑恢复 ready
    （计数 4/1 精确）。
  - **Vue split**（`vue_media_run.py` → `vue_media_run.json` + 
    `vue_ready.png`，7/7）：真 `auto run`（vite 前端+生成 axum 后端）+ 真
    Chromium；浏览器 `<img>` 绑定后端 `/api/__auto/media/...` URI 并**真实
    解码**（naturalWidth/Height=2/1=alpha-2x1.png 实尺寸）；onload→应用态
    `ready loaded=1 error=0` 恰好一次且稳定；浏览器 fetch 坏票 404。
  - **Rust merged**（`merged_media_run.py` → `merged_media_run.json`，6/6）：
    生产 `auto gen`（render:rust）生成 merged 应用：Init 经桥函数
    （`auto_media_core::open_media_session` 直调）开真会话、Bind 走真
    request_view+绑定不透明 URI、渲染帧后状态稳定、进程健康+10s 内净退。
  - **生成媒体 HTTP**（`gen_media_http.py` → `gen_media_http.json`，13/13）：
    生产 `auto gen` 生成 axum 后端→实编→真 TCP：非媒体用户路由存活
    （未被媒体拦截）、viewer 会话族、媒体 GET 200（content-type/
    cache-control immutable/ETag）、HEAD 无 body 同头、If-None-Match 304、
    坏票 404、`/api/media/scan` 条目、`/api/media/stream` Range **206 字节
    精确匹配**（bytes 100-299/4096 与源文件逐字节一致）+后缀 Range+416。
- **过程中修复的两个发射面缺陷**（本轮提交 e2ea15e79）：
  1. api_gen 媒体路由 `{id}/{revision}`（axum 0.7/matchit 0.7 下为字面量，
     plan736 起生成后端媒体 URI 全 404）→ 改 `:id/:revision`；
  2. rust_ui merged 成员清单缺 `auto-media-core`（fresh merged 媒体应用
     resolve 红）→ 增 `auto-media-core.workspace = true`。
  两者均被上述真 harness 钉住（修复前 13/13 中 8 项红/merged 编译红）。
- **如实登记的预存缺口（非本计划引入，不虚报完成）**：
  - **merged 模式 loaded/error 事件分发未接线**（PLAN-095 起）：notify
    sweep 只挂在 VM/desktop runner 的 `run_session`（renderer.rs:22854），
    静态组件 merged 应用的 `devtools_update` 循环无 sweep——merged 应用
    ImageSurface 渲染正常（resolve 走共享 registry）但事件从不派发。属
    新特性补全（PLAN-095 扩展），按计划 §10 交 R04 具名承接；merged
    收据已注明不据此断言。
  - **sweep 静止期绑定边界**：媒体全部静止后代次不变时，早退门（表空+
    pending 0+代次未变）会跳过新挂载 surface 的注册——031 实际用法
    （绑定伴随 request_view/.populate 活动）不触发；VM 探针已按真实用法
    覆盖，边界行为留 PLAN-095 owner（R04 一并）。
  - **031 全栈 `auto run`（vue/VM）在 master 基线即失败**：生成后端
    `mod types;` 双发（types.at companion 与固定发射冲突）+`crate::image`
    缺失+types 生存期错——基线 auto.exe（主检出新构建）在隔离 scratch
    生成**完全相同**的破损产物，与本计划 diff（媒体路径字符串）无关；
    属 681/736 时代发射面与 810 在途文法域。本轮以隔离探针项目绕开
    （复审许可），031 本体修复不属本计划。
  - Man `test-http-e2e` wire 档 master 基线编译红（generate_api_rs 签名
    漂移+plan734 tauri fixture，25 err）未代修；生成 TCP 证据由上述隔离
    真生产 harness 承担。
- **current results**：四收据全绿（9/9、7/7、6/6、13/13），harness 脚本
  随附件可重放。

### F811-R1-04 · P3 · 六文件格式

- **fix**：对复审点名的八文件跑 `rustfmt --edition 2021 --config
  skip_children=true`（六文件有 diff 全部落盘）。
- **current results**：同口径 `--check` exit 0；随后新增/修改文件
  （api_gen.rs/rust_ui.rs 修复轮）同样先行格式化。
- **remaining gap**：无。

## 复验入口

```text
# 生命周期/准入（含复审探针重放）
python docs/plans/attachments/811/review-r1/replay.py D:/autostack/.wt/lang-811/auto-lang
cargo test -p auto-media-core --lib          # 34/34
# 三形态真机（各 1-3 分钟；merged 需先 gen+build，见 harness 文档头）
python docs/plans/attachments/811/v05-harness/vm_media_run.py     <wt> <scratch> <out>
python docs/plans/attachments/811/v05-harness/vue_media_run.py    <wt> <scratch> <out>
python docs/plans/attachments/811/v05-harness/merged_media_run.py <wt> <scratch> <out>
python docs/plans/attachments/811/v05-harness/gen_media_http.py   <wt> <scratch> <out>
```

本轮未运行整档 t/tv/tf/docs_gen（merge-owned 时序裁定）；修复轮触面：
auto-media-core（生命周期/准入+测试）、api_gen/rust_ui 模板两行级修复
（已跑 api_gen 50/50、rust_ui 40 PASS、delivery 9/9）。
