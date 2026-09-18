---
plan_id: PLAN-653
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: vue-back-contract-fix
author: [zhaopuming/zcode-session]
created_at: 2026-09-18
updated_at: 2026-09-18

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: []                   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 5
---

# [PLAN-653] vue-back-contract-fix

## 变更摘要

vue 轨(AutoUI vue 形态)打开后无 cmd 输出、基本不可用。诊断(auto-term
PLAN-022 §9/§10 + 受控浏览器/HTTP 实测,2026-09-18)定位三层缺陷,均属
生成器/运行时面,归本仓修复:

1. **back API 生成器把带参 GET 路由生成为 GET + `Json` 提取器**——
   axum 对无 body GET 一律 400(`Expected request with Content-Type:
   application/json`),前端每拍轮询 `tab-id-at?i=0` 即抛错,循环中断,
   Tab 标签/槽位/终端永不渲染(主因)。019 规范(auto-term
   terminal-mux-model.md §V1.8)明文"带参读一律 POST"(rect-* 面即
   合规),tab-* 带参 GET 面生成违反。
2. **dev 跑法(`auto run -r vue`)back 引擎 DLL 解析失败**——首 tick
   panic(app-back/src/term.rs:225 "autoterm_core.dll 未找到"),手动
   设 AUTOTERM_ENGINE_DLL 才活;020/021 可用走部署态配方掩盖了缺口。
3. **vue back 矩形投影单幅化**——visible=5 但投影仅出单满幅 pane,
   020 t01 剧本 12 步受阻;嫌疑收敛到 master `1fc4772d9`(rust lowering
   尾分号/括号修复)恰好改了发射投影的 rust_ui.rs,或 642/643 合并族。

## 目标

- G1 带参读取路由按 019 规范生成(POST+Json 或 GET+Query,二选一定
  稿),前端轮询链路不再 400/405。
- G2 dev 跑法 back 引擎 DLL 自动解析成功(或启动器注入 env),tick 不
  panic。
- G3 vue back 矩形投影与模型一致(多 pane 出多槽+分隔条槽),t01 剧本
  12 步全绿。
- G4 端到端:vue 形态打开即见 cmd 输出(banner+提示符),可键入回显。

### 非目标

- vue 终端只读视口能力扩展(auto-term PLAN-019 边界维持);
- rust 轨/VM 轨回归面(auto-term PLAN-022 已闭环);
- 桌面 launcher 装载形态。

## 架构方案

- 缺陷 1 落 `crates/auto-man/src/api_gen.rs`(db-covered scalar
  endpoints 分支,:1570-1600 一带:body_params 非空即发
  `Json(body): Json<serde_json::Value>`,未按 HTTP method 分流;仓库内
  已有 GET/DELETE Query-struct 机制 :1741/:1829/:2101 可复用)。修复
  =带参 GET 走 Query 提取,或路由与 handler 统一改 POST(与 rect-* 族
  对齐,019 规范合规),定稿于 T-00。
- 缺陷 2 落 auto-term `app/term.rs` sidecar(引擎 DLL 解析候选链:
  env → exe 目录 → 祖先 target/debug;dev 跑法 back exe 实际落点待
  T-00 勘定)或 vue 管线启动器 env 注入;跨仓改动走 auto-term 侧
  兄弟 worktree。
- 缺陷 3 落 `crates/auto-man/src/rust_ui.rs`(back 投影 rects_recompute
  发射);T-00 先对 1fc4772d9 diff 做最小复现(对拍生成 db.rs 投影段),
  定位 lowering 改动破坏点再修。

## 需求分析与背景调查

- **授权**:用户裁定 2026-09-18"独立的修复计划吧"(022 会话内,本人
  请求);预算/自动续跑未指定。
- **证据**(auto-term docs/plans/evidence/022/ + PLAN-022 §9/§10):
  - 受控浏览器实测:页面 3s 发 28 个请求,失败全为
    `tab-id-at?i=0 → 400`;curl 复现 GET+Json 400 文案;
  - vue back tick panic:app-back/src/term.rs:225;
  - t01 剧本:visible=5 但 layout 投影单满幅(slot1 满幅、无分隔条槽);
  - 载具侧一切服务链路(页面/代理/back)实测可达——排除网络/代理因。
- **代码事实**(读码 2026-09-18):
  - api_gen.rs:1588 Json 签名发射无 method 分流;:1741/:1829/:2101
    存在 GET Query 既有机制;
  - back 路由:tab-id-at/title-at/is-active-at = GET(main.rs:247-249),
    rect-* = POST(:263);
  - 投影发射:rust_ui.rs(rects_recompute 字样在案);back 产物
    app-back/src/db.rs:328;
  - DLL 解析候选链:auto-term `app/term.rs`(env → exe 目录 → 祖先
    target/debug);`auto run -r vue` 的 back exe 实际落点未勘定。

## 详细设计

| # | 改动 | 文件:符号(仓) | 说明 |
|---|---|---|---|
| D1 | 带参 GET 路由契约修复 | crates/auto-man/src/api_gen.rs(:1570-1600 分支;复用 :1741 Query 机制) | method 分流:GET+Query 或统一 POST+Json(019 规范),T-00 定稿 |
| D2 | dev DLL 解析 | auto-term `app/term.rs` 解析候选链 或 vue 启动器 | 候选链补 back exe 实际落点/启动器注 env |
| D3 | 投影单幅化根修 | crates/auto-man/src/rust_ui.rs(投影发射) | T-00 最小复现定界 1fc4772d9 破坏点后修复 |

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | (预计)无规范文本变更 | — | 019 规范"带参读一律 POST"已存在;本计划为**恢复合规**的缺陷修复。若 T-00 定稿走 GET+Query 路线,则修订 auto-term terminal-mux-model.md §V1.8 对应句 | 缺陷修复恢复既有契约;路线变更才动规范 | AC-01 |

(若定稿 GET+Query,SD-01 转为对 terminal-mux-model.md 的 modify 并
回填精确条目;纯 POST 路线则无规范增量,理由如上。)

## 测试设计

1. 生成物级:修复后从零生成 auto-term vue back,断言
   - 带参 GET 路由 curl(`tab-id-at?i=0`)返回 200 数值,非 400;
   - back tick 不 panic(DLL 自动解析);
   - split 后 layout 投影出多槽+分隔条槽(非单满幅)。
2. 剧本级:020 t01_model.sh 12 步全绿(clean back,端口按 020 惯例)。
3. 端到端:受控浏览器打开 vue 页(17400)——Tab 按钮"● shell N"渲染、
   终端出 banner+提示符、键入回显。
4. 回归:auto-lang 仓 scoped 门(cargo check -p auto-man;ui_gen 相关
   测试)+ auto-term workspace 全绿;rust 轨不受影响(auto-term
   PLAN-022 的 screen-split 实锤不回退)。

## 验收标准

- **AC-01** 带参读取路由按定稿契约生成,前端轮询无 4xx。验证:curl
  带参路由 200 + 受控浏览器无 failed request。
- **AC-02** dev 跑法(`auto run -r vue`,无手动 env)back tick 不
  panic,shell 输出可达。验证:启动日志无 panic + tick 返回 banner。
- **AC-03** 投影与模型一致:split 后 layout 出多槽+分隔条槽。验证:
  curl /api/mux/layout 结构断言。
- **AC-04** 020 t01 剧本 12 步全绿。验证:剧本日志。
- **AC-05** 端到端:vue 页可见 cmd 输出并可键入回显。验证:受控浏览器
  截图/断言。

## 执行步骤

- **T-00 [D1/D2/D3] bounded investigation**:①api_gen 带参 GET 分支
  与 Query 既有机制对读,定稿 POST+Json vs GET+Query;②`auto run -r
  vue` back exe 实际落点勘定(决定 D2 改 sidecar 候选还是启动器);
  ③对 1fc4772d9 做投影段最小复现(对拍生成 db.rs),定界单幅化破坏
  点。产出决策工件(证据归 docs/plans/evidence/653/)。前置:无。
  关联 AC-01/02/03。
- **T-01 [D1] 带参路由契约修复**(auto-lang worktree)。前置 T-00。
  关联 AC-01。
- **T-02 [D2] dev DLL 解析修复**(auto-term 侧,兄弟 worktree)。
  前置 T-00。关联 AC-02。
- **T-03 [D3] 投影单幅化根修**(auto-lang worktree)。前置 T-00。
  关联 AC-03。
- **T-04 回归门**:从零生成 + t01 剧本 + workspace 全绿 + 受控浏览器
  端到端。前置 T-01..T-03。关联 AC-04/05。

## 复审记录

- 2026-09-18 draft handoff:`stage: new`,PLAN-653 rev1。`outcome:
  pass`(起草授权 = 用户裁定"独立的修复计划吧",022 会话;诊断证据
  已实测在案,T-00 为首批可执行项)。`next: work`(work 前按惯例
  `/auto-plan:review`)。

## 待澄清事项

1. 带参读取契约定稿:POST+Json(019 规范字面)vs GET+Query(REST
   惯例)——T-00 给建议,方向涉及规范措辞时报用户裁定;
2. D2 落点:sidecar 候选链(auto-term 仓)vs vue 启动器 env 注入
   (auto-lang 仓)——T-00 按 back exe 实际落点定;
3. 投影回归若定界为 1fc4772d9(他人 master 修复引入):修复方式与
   该 fix 作者意图的兼容性——届时回报用户;
4. 与在途 642/647 会话的 auto-lang 冲突面:开工前复查其 worktree diff。
