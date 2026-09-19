---
plan_id: PLAN-658
status: executing
feature_name: uigallery-multi-backend-proxy
author: [agent]
created_at: 2026-09-19
updated_at: 2026-09-19
plan_revision: 1
current_step: 0
total_steps: 8
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-010]
affects: [auto-lang/vm, auto-lang/ui, auto-man, auto-os/ui-gallery]
---

# [PLAN-658] uigallery-multi-backend-proxy

## 0. 变更摘要

画廊内嵌 fullstack demo 的"后端半身"无宿主进程（PLAN-642 终审遗留
P642-D10/D13）：020-music-player 曲库空（`Http.get_json("/api/media/scan")`
无后端进程应答）、017-chat / 031-image-viewer 被 back 链 native-ns/stream
否决（"暂无内嵌形态"）。本计划建立**单进程多后端宿主（proxy）**：每个
app 的后端以独立 VM session 在同一进程内运行，按子 URL
（`/apps/<id>/api/*`）路由；前端 HTTP 调用经生成器 baseURL 子前缀适配。
**用户裁定范围（2026-09-19）**：不止解决 020，**017/031 的
native-ns/stream 后端为一等公民目标**——stream/WebSocket 转发与 binary
（图片）转发在立项范围内，不是风险排除项。

## 1. 目标

- **G-1 020 曲库内嵌可见**：画廊内嵌 020 经 `/apps/020-music-player/
  api/media/scan` 出真实曲库（E:/Music mp3 列表），P642-D10 核销。
- **G-2 stream 一等公民**：017-chat 的 `pub fn stream() ~Stream<ChatEvent>`
  （api.at:62 实证）经 proxy 转发——内嵌聊天可收发消息，事件流实时到达。
- **G-3 native-ns/binary 一等公民**：031-image-viewer 的 `use auto.image`
  后端（image_service.at 实证）在 session 内可执行，图片字节经 proxy
  转发——内嵌图库缩略图/大图出图。
- **G-4 端口收敛**：全部内嵌 demo 后端经单端口子 URL 前缀服务
  （3049..3050+N → 1）。
- **G-5 崩溃隔离**：单 app 后端 panic 被捕获（自动重启或错误面板），
  不带倒宿主与其他 app session。
- **非目标**：跨机/远程部署形态；子 domain 路由（远期选项）；独立运行
  形态（各 demo 自带 Axum 后端进程）的变更——必须零回归；web 臂
  （Vue）语义变更；proxy 热重载。
- **受影响仓库**：auto-lang（VM session 基建 / auto-man 生成器 baseURL /
  Http stdlib 前缀展开）+ auto-os（gallery 启动编排 + registry 注入）。
- **成功判据**：AC-01..06 全过；独立形态回归零红。

## 2. 架构方案

承 PLAN-642 §8.2 可行性四依据（归档在案）：

1. **会话隔离原型**：`auto serve` daemon（Plan 269）已是单进程多 stateful
   VM session（named pipe）——proxy 把 session 前端从 pipe 换成 HTTP 路由。
2. **back 链进程内执行已通**：PLAN-633 merged 模式（013/015 内嵌）证明
   back .at 可在宿主 VM 进程内装载执行——"每 app 一个 VM session"是同一
   机制的会话化扩展；**路由分发形态采用 session 内 `#[api]` fn 表动态
   分发（merged CALL 语义的 HTTP 化）**，Axum 生成器路由表为辅。
3. **前端适配面小**：生成器 api baseURL `:port` → 子 URL 前缀
   `/apps/<id>`；`Http.get_json` 相对路径按前缀展开——PLAN-617
   `AUTO_HTTP_BASE` 相对展开先例（get/post/put/delete/json 五臂）。
4. **推荐形态**：子 URL 路径前缀（零 DNS/hosts 成本）。

**风险面（用户裁定升级为一等公民目标）**：session 崩溃隔离（per-session
catch + 重启）；SSE/stream 转发（017）；binary 响应转发（031 图片字节）；
可观测性（每 session 日志通道）。

**进程形态**（T-00 决策产物定案，默认倾向）：复用 auto serve daemon 扩展
HTTP 前端 vs auto run 宿主内嵌 proxy 线程——按"gallery 一键体验
（`auto run -r vm` 即含全部后端）+ 启动编排复杂度"权衡。

## 3. 技术栈

- Rust：`crates/auto-lang/src/vm/`（session 基建——`auto serve` daemon
  Plan 269 形态参考）、session 内 `#[api]` 路由注册表（Plan 312
  `register_http_routes` 既有钩子）、axum（宿主路由层）。
- auto-man：`crates/auto-man/src/`（back→Axum 生成器路由清单复用 + api
  baseURL 子前缀适配）。
- Http stdlib：`crates/auto-lang/src/vm/ffi/stdlib.rs`（AUTO_HTTP_BASE
  相对展开，PLAN-617 五臂）。
- 语料/产物：examples/ui/{017-chat,020-music-player,031-image-viewer}
  back 链（语料不动）；auto-os ui-gallery 启动编排 + registry.at。

## 4. 需求分析与背景调查

### 4.1 授权记录

- 用户 2026-09-19（PLAN-642 收口裁定）："这个我同意独立立项，且不止解决
  020 一个问题，也要解决 017/031 的问题。先把它写进 DEBT 里，等 642 收尾
  后再单独立项。" → P642-D13 登记（KNOWN-DEBT-AND-RISKS.md 2026-09-19
  增补二），本计划即该裁定承接。
- 可行性调研授权承 PLAN-642 T-19 前置调研（§8.2，归档
  `docs/plans/archive/642-uigallery-vm-embed-repair.md`）。

### 4.2 关键实证（起草期核实）

| demo | back 形态 | 证据 |
|---|---|---|
| 020-music-player | `#[api]` 路由族（player/status 等；媒体索引"自动经 /api/media/scan 与 /api/media/stream/:id 服务"） | back/api.at:3/11 |
| 017-chat | `pub fn stream() ~Stream<ChatEvent>`（流签名后端） | back/api.at:62 |
| 031-image-viewer | `use auto.image`（原生命名空间图像管线） | back/api.at:7 / image_service.at:5 |

既有发射器否决点（auto-man vue.rs fullstack 级联的 unsupported 判定）：
`use auto.*` / `~Stream` / `~Promise` → back_ok=false → 静态面板——本计划
打通后按 demo 逐族解除。

### 4.3 背景

- PLAN-633：merged 模式 back 内嵌（013/015）——进程内 CALL reloc 语义。
- PLAN-617：AUTO_HTTP_BASE 相对 URL 展开（get/post/put/delete/json）。
- Plan 269：auto serve daemon named-pipe session 管理。
- Plan 312：`#[api]` 路由收集（register_http_routes）。

## 5. 详细设计

（T-00 决策产物回填：session 模型对比表、路由分发序列、stream 转发时序、
崩溃隔离边界。骨架方向：proxy 宿主持有 `HashMap<app_id, VmSession>`；
每 session 装载该 demo back 链（P633 级联产物）；HTTP 请求 → 子前缀解析
→ session 内路由表命中 → CALL 语义执行 → 响应（JSON 直返 / stream 逐
chunk 转发 / binary 字节转发）。）

### 规范增量

| delta_id | op | target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/vm/back-proxy.md（新建） | 无 → 多后端 proxy 契约：子 URL 路由、per-app VM session 模型、#[api] 动态分发语义、stream/binary 转发、崩溃隔离边界 | 单进程多后端宿主为 enduring 架构面 | AC-01..05 |
| SD-02 | add | docs/specs/auto-lang/ui/overview.md §ui-gallery | 无 → 内嵌 fullstack demo 后端经 proxy 子 URL 服务的画廊契约（registry/启动编排/回退页解除条件） | 画廊内嵌形态扩展 | AC-01..03 |
| SD-03 | modify | docs/specs/auto-lang/vm/plans.md | 沉淀行追加 PLAN-658 | 账目 | — |

## 6. 测试设计

- **单元**：session 注册/路由命中/前缀解析；stream 转发 chunk 序列；
  binary 转发字节保真；panic 注入隔离（单测可注入假 session）。
- **E2E（主验收）**：画廊 VM 实例 MCP 驱动——020 曲库列表出现 + 播放器
  状态路由；017 双向消息时序；031 缩略图网格 + 大图加载；panic 注入后
  其他 demo 存活。
- **回归**：独立形态 017/020/031 `auto run`（各自 Axum 后端）行为不变；
  既有内嵌矩阵（013/015/022..）零回归；`cargo t` 日常档 + 触发面档。

## 7. 验收标准

- **AC-01**：画廊内嵌 020 曲库非空（E:/Music mp3 列表可见，请求经
  `/apps/020-music-player/api/*` 前缀）；P642-D10 核销。（G-1）
- **AC-02**：内嵌 017 可发送消息并收到流式回复（~Stream 事件经 proxy
  到达前端，时序截图/快照证据）。（G-2）
- **AC-03**：内嵌 031-image-viewer 图库出图（缩略图网格 + 点开大图，
  二进制经 proxy 保真——像素级或字节级校验）。（G-3）
- **AC-04**：全部内嵌后端流量经单端口 `/apps/<id>/` 子前缀（端口数=1；
  抓包/日志证明无 3049+ 直连）。（G-4）
- **AC-05**：单 app 后端注入 panic → 该 app 降级错误态（自动重启或错误
  面板），宿主与其他 app session 存活。（G-5）
- **AC-06**：门禁零新增红；独立形态三 demo 回归通过；既有内嵌矩阵零回归。

## 8. 执行步骤

（原子任务；每步完成后追加 [✅] 证据行）

- [ ] **T-00 有界调研定案：session 模型与进程形态**
      决策产物回填 §5：①auto serve daemon 扩展 HTTP 前端 vs auto run 宿主
      内嵌 proxy 线程（一键体验/启动编排/生命周期对比表+裁定）；②017
      stream 的 wire 形态查证（~Stream 在既有 Axum 生成器的落线方式——
      SSE/长连接）；③031 auto.image 的进程内可用面（宿主 image pipeline
      PLAN-646 形态）。产出：设计决策记录 + 任务集校准（如需裂变在册）。
- [ ] **T-01 proxy 骨架 + 纯 JSON API 面**
      axum 单宿主 + `HashMap<app_id, VmSession>` 注册表 + 子 URL 前缀解析
      + session 内 `#[api]` fn 表动态分发（Plan 312 路由清单）+ JSON
      响应。冒烟：020 `/api/player/status` 经子前缀返回真实状态。
- [ ] **T-02 生成器 baseURL 子前缀适配**
      auto-man api baseURL `:port` → `/apps/<id>`（PLAN-617 AUTO_HTTP_BASE
      相对展开先例，五臂覆盖）；独立形态零变化（形态开关判定）。
- [ ] **T-03 gallery 集成 + 020 曲库端到端**
      auto-os 启动编排（proxy 随画廊拉起）+ registry/适配器注入子 URL +
      内嵌 020 出曲库（AC-01 全链）；P642-D10 核销。
- [ ] **T-04 stream 转发一等公民（017）**
      session 内 ~Stream 执行 + HTTP chunk/SSE 逐事件转发 + 前端事件流
      消费；内嵌 017 双向收发（AC-02）；017 否决解除（发射器 unsupported
      判定按 demo 通路放宽）。
- [ ] **T-05 native-ns/binary 转发（031）**
      `use auto.image` in-session 执行（宿主 image pipeline）+ 图片字节
      转发（Content-Type 保真）；内嵌 031 图库出图（AC-03）；031 否决解除。
- [ ] **T-06 崩溃隔离与可观测性**
      per-session panic catch + 重启策略（退避）+ 每 session 日志通道；
      panic 注入验收（AC-05）。
- [ ] **T-07 E2E 全矩阵 + 债核销 + 门禁收口**
      AC-01..06 全量复验；KNOWN-DEBT 核销/部分核销（D10 全销、D13 主体
      核销留远期项、031 家族债核）；spec 增量落稿；`cargo t` 日常档 +
      触发面档；独立形态回归三件套。

## 9. 复审记录

```yaml
stage: new
plan_id: PLAN-658
plan_revision: 1
outcome: pass        # 契约就绪,待用户确认后进入 work
code_commit: N/A
task_ids: []         # T-00..T-07 待执行
evidence: 起草期核实（§4.2 三 demo back 形态证据表）+ P642-D13 承接链
next: work（用户确认契约后：master commit 本计划 → 建 worktree 组
  D:/autostack/.wt/lang-658/{auto-lang,auto-os} → T-00 有界调研定案）
```

```yaml
stage: work
plan_id: PLAN-658
plan_revision: 1
outcome: pass        # 进入执行（2026-09-19 用户指令"实施它"=契约确认）
code_commit: N/A     # worktree 未建前主检出仅簿记
task_ids: []         # T-00 起
evidence: 用户会话指令确认契约；auto-lang 主检出 clean、auto-os 主检出
  WIP 定性为 widgets-gallery 生成漂移+会话产物（非 ui-gallery 面，已呈报不入本计划）
next: 建 worktree 组 D:/autostack/.wt/lang-658/{auto-lang,auto-os} → T-00
```

## 10. 待澄清事项

- T-00 决策产物"进程形态"若越出本计划授权面（如需改 auto serve 公共
  协议或引入新守护进程安装面），呈报用户裁定后再继续对应任务。
- 017 stream 若查证为非 HTTP wire 形态（纯 pipe/进程内），转发层设计
  相应调整并回填 §5——语义目标（内嵌可收发）不变。
