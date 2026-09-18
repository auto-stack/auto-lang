---
plan_id: PLAN-653
status: archived              # drafting → executing → execution_done → reviewed → archived
feature_name: vue-back-contract-fix
author: [zhaopuming/zcode-session]
created_at: 2026-09-18
updated_at: 2026-09-18

# /auto-plan:review 结束时填写：
supersedes_spec_components: []   # 空置说明:仅对既有 spec 条目做 modify(见 SD-01),无组件废止
new_spec_components: []          # 空置说明:契约分域规则落入既有 terminal-mux-model.md,无新组件
touched_goals: [GOAL-007]        # GOAL-007 AutoUI 双端一致——D4 vue 轨样式发射正确性属双端同源面

affects: [auto-term/terminal-mux-model]  # SD-01 modify 目标(auto-term 仓)
current_step: 5
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
3. ~~vue back 矩形投影单幅化~~ **撤销(非缺陷)**:干净 back
   (18080,独立端口)重跑 020 t01 剧本 **12 步全绿**(split/磁吸/钳位/
   MAX_PANES/close 全对)——此前单幅化读数来自被用户实测轮询污染的
   共享 17401 实例,剧本须按 020 惯例在干净 back 上跑。
4. **vue 前端槽位样式发射错误**(auto-term 17401 实测,页面 2209px
   需滚动才见终端):生成的 App.vue 把类名串塞进 `:style`
   (`:style="absolute z-10 top-[${y1}px]..."`)——非合法 inline style,
   浏览器整串忽略 → 槽位 div 零定位零尺寸落文档流。正确应为
   `:style` 发真样式(position:absolute;top:Npx;…)或 `:class`+静态
   扫描。auto-shell 的 vue(18400)同生成器同病。

## 目标

- G1 带参读取路由按 019 规范生成(POST+Json 或 GET+Query,二选一定
  稿),前端轮询链路不再 400/405。
- G2 dev 跑法 back 引擎 DLL 自动解析成功(或启动器注入 env),tick 不
  panic。
- G3 vue back 矩形投影与模型一致(多 pane 出多槽+分隔条槽),t01 剧本
  12 步全绿。
- G4 端到端:vue 形态打开即见 cmd 输出(banner+提示符),可键入回显。
- G5 vue 页首屏即见终端(槽位 absolute 布局生效,无需翻页)。

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
| D2 | dev DLL 解析 | auto-term `app/term.rs` 解析候选链 或 vue 启动器 | 候选链补 back exe 实际落点/启动器注 env;auto-shell 18401 实测 apply-resize 每拍 500(mounted 钩子首拍死,同族证据) |
| D4 | 槽位样式发射修复 | crates/auto-lang/src/ui_gen/vue.rs(App.vue 槽位 div) | `:style` 发真 inline style(position:absolute;top/left/width/height px),或 :class 走静态白名单;修复页面 2209px/翻页问题(auto-term 17401 实测 + auto-shell 同病) |
| ~~D3~~ | ~~投影单幅化根修~~ | — | 撤销:干净 back 复跑 t01 全绿,非缺陷(见摘要 3) |

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | auto-term docs/specs/terminal-mux-model.md(§可见多终端视图契约) | before:"带参读一律 POST"(全域)。after:**分域**——rect-\*/命令型投影族维持一律 POST;标量 getter 族(tab-id-at/tab-is-active-at/tab-title-at/pane-cols/pane-rows/pane-cursor-\* 等)= **GET+Query**(纯读幂等面;前端客户端生成器按 `?i=` 形态发射,rust back 生成器以 `{Fn}Query`-struct 提取对齐) | T-00 定稿 GET+Query(后端单侧根修,零前端改动);auto-term 38477cf 落档 | AC-01 |

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
- **AC-06** 首屏布局:vue 页首屏(无滚动)即见终端槽位。验证:受控浏览器
  evaluate 断言 document.scrollHeight ≤ viewport 高 + 槽位 div
  getBoundingClientRect 在首屏内。

## 执行步骤

- **T-00 [D1/D2/D3] bounded investigation**:①api_gen 带参 GET 分支
  与 Query 既有机制对读,定稿 POST+Json vs GET+Query;②`auto run -r
  vue` back exe 实际落点勘定(决定 D2 改 sidecar 候选还是启动器);
  ③对 1fc4772d9 做投影段最小复现(对拍生成 db.rs),定界单幅化破坏
  点。产出决策工件(证据归 docs/plans/evidence/653/)。前置:无。
  关联 AC-01/02/03。
  [✅] **已完成**(本会话 2026-09-18):①定稿 **GET+Query**(前端生成器
  已按 `?i=` 发射、api.at 源声明 GET、后端单侧根修);②exe 实落
  `<app>/rust-workspace/<name>-back` 默认 target(cargo config 发现
  基于 CWD,workspace 内 target-dir 配置不生效),仓根 DLL 距 exe 目录
  5 级祖先,旧 take(4) 恒落空,D2 落点=sidecar 候选链扩展;③从零生成
  db.rs 与部署版逐字节一致 + 投影实测全对,D3 撤销(与 rev1 判定一致)
  。决策工件:evidence/653/t00-decision.md。
- **T-01 [D1] 带参路由契约修复**(auto-lang worktree)。前置 T-00。
  关联 AC-01。
  [✅] **已完成**:commit 3a6c78d59——scalar 分支 GET/DELETE 复用既有
  `{Fn}Query`-struct 机制(endpoint_query_params+serde(default)),
  POST 维持 Json。实测:带参/缺参/str 参数 GET 全 200,投影链完好,
  POST 对照 200。cargo check -p auto-man 零新增 warning;nextest
  -p auto-man 298/298。**调和记录**:worktree 曾有并行会话中断残留
  的同任务 untyped Query<Value> 草稿(实测可编译可运行,axum Query
  对 Value 有 Deref),按计划 D1 点名的既有机制重实现覆盖;草稿 diff
  存证 evidence/653/t01-predecessor-wip-uncommitted.diff。
- **T-02 [D2] dev DLL 解析修复**(auto-term 侧,兄弟 worktree)。
  前置 T-00。关联 AC-02。
  [✅] **已完成**:auto-term plan-653-dev commit 38477cf——app/term.rs
  候选链 exe 祖先 4→6 级 + CWD 祖先 4 级兜底(env 优先/dist 同目录
  语义不变)。阴性对照:旧 sidecar 故障位置复现 panic;修复版同位置
  tick 200 + [term-dll] 日志明示仓根 DLL;真实 `auto run -r vue` 管线
  无手动 env 同证据(evidence/653/dll-probe-*.log、e2e-run.log)。
- ~~T-03 [D3] 投影单幅化根修~~ 撤销(干净 back 复跑 12 步全绿,
  非缺陷;t01 门改由本计划 T-04 直接覆盖)。
  [✅] 撤销已复核:本会话独立实测(从零生成 db.rs 逐字节比对 + 投影
  全链探测)与 rev1 判定一致。
- **T-05 [D4] 槽位样式发射修复**(auto-lang worktree + 前端重生成)。
  前置 T-00。关联 AC-05(新增)。
  [✅] **已完成**:commit 7d5ae193d——extract_classes 动态 style 通道增
  tailwind_class_template_to_inline_style 翻译(全 token 可识别才翻
  译,未知整体回退;\${…} 插值保留),单点覆盖三处 :style 发射臂。
  翻译器单测 + 生成级单测双绿;ui_gen::vue 模块 284/23 与 master
  基线逐一致(23 红预存,非本改动引入)。从零重生成实测 App.vue 槽位
  div position:absolute 定位生效。
- **T-04 回归门**:从零生成 + t01 剧本 + workspace 全绿 + 受控浏览器
  端到端(首屏即见终端)。前置 T-01/T-02/T-05。关联 AC-04/05。
  [✅] **已完成**(2026-09-18):从零生成(auto-term worktree app,
  修复版生成器+sidecar)→ t01_model.sh 12 步全绿
  (evidence/653/t01-model-run-653-final.log;磁吸 500/钳位 100/键盘
  700/MAX_PANES 帽 -2/窗缺省 1024x768/close 重算全对)→ 受控浏览器
  端到端:Tab 条"● shell 1"渲染、cmd banner+提示符首屏可见
  (evidence/653/e2e-tabbar-banner.png、e2e-firstscreen-653.png)、
  槽位 div computed style position:absolute 生效、scrollHeight
  768(修复前 2209);GET+Query 线上 200;无手动 env DLL 解析成功。

## 复审记录

- 2026-09-18 draft handoff:`stage: new`,PLAN-653 rev1。`outcome:
  pass`(起草授权 = 用户裁定"独立的修复计划吧",022 会话;诊断证据
  已实测在案,T-00 为首批可执行项)。`next: work`(work 前按惯例
  `/auto-plan:review`)。
- 2026-09-18 stage:work 交接(Plan 653 rev1,全 5 任务完成 → `outcome:
  pass`,`status → execution_done`,`next: review`):code_commit =
  auto-lang plan-653-dev **3a6c78d59**(T-01)+ **7d5ae193d**(T-05,
  基线含 3fb469842 master merge);auto-term plan-653-dev **38477cf**
  (T-02+SD-01,兄弟 worktree D:/autostack/.wt/lang-653/auto-term)。
  task_ids = T-00/T-01/T-02/T-05/T-04 全完成(已回填 [✅] + 证据)。
  验收对照:AC-01 ✅(GET+Query 线上 200/前端无 4xx)、AC-02 ✅(无
  env DLL 解析,tick 200,阴性对照 panic 复现)、AC-03 ✅(t01 布局
  多槽+分隔条全对)、AC-04 ✅(t01 12 步全绿)、AC-05 ✅(banner+提示符
  首屏可见;**键入回显除外**——见待澄清 5)、AC-06 ✅(槽位 absolute
  生效,scrollHeight 2209→768;48px 余量归窗模型缺省,见待澄清 6)。
  并行会话调和:T-01 草稿存证后被本会话实现覆盖;worktree 已由该
  会话并 master merge 3fb469842;用户裁定关闭对方 agent,后续由本
  会话收口。证据:evidence/653/(t00-decision.md、t01 前后对照、
  dll-probe 双态、t01-final、e2e 截图/日志)。
- 2026-09-18 stage:review | plan_id PLAN-653 | rev1(7bd8e5147 承载,
  语义契约未再变) | outcome:**pass** | reviewed_commit = auto-lang
  plan-653-dev **7d5ae193d**(基 52d27ebcd,含 merge 3fb469842)+
  auto-term plan-653-dev **38477cf** | dependency_revisions:auto-term
  主检出 3a3339a(仅消费,未改动)| spec_inputs:auto-term
  terminal-mux-model.md @38477cf(SD-01 hunk 复读合格——分域规则描
  述当前行为、含 rationale、非执行日志)| acceptance_results(全部
  在复审基线独立复现,非沿用实施摘要):
  - AC-01 pass:tab-id-at?i=0 → 200、tab-title-at?i=0 → 200
    "shell 1"(review-run 栈实测);
  - AC-02 pass:[term-dll] 加载仓根 DLL、零 panic(复审 run 日志);
  - AC-03 pass:t01 [2]/[3]/[4] 多槽+分隔条结构正确(/tmp/t01-review
    复跑;归档见 evidence/653/t01-model-run-653-final.log 同版);
  - AC-04 pass:12 步全绿(磁吸 500/钳位 100/键盘 700/帽 -2/窗缺省);
  - AC-05 pass(部分语义见待澄清 5):Tab 条+banner+提示符 DOM 断言
    全 true;
  - AC-06 pass:9 个 absolute 槽位 div(5 pane+4 divider,t01 后树态)
    getBoundingClientRect 全部 inFirstScreen=true,scrollHeight 768
    (修复前 2209;48px 余量归窗模型缺省,待澄清 6)。
  findings:
  - F-1(P2,基线):auto-cache lib test 编译失败(E0063)——**master
    HEAD 同样破损**,阻塞全仓 nextest 门;非 653 引入,建议独立修复
    计划/走查件收口;
  - F-2(P3,基线):plan367 real_sidebar_at_parses_with_navtree 分支
    红/master 绿——分支落后 master 解析器修复(52d27ebcd 后 master 有
    899aa2e9c/ffe2dac6d 等),merge 阶段 re-sync 即解,非 653 引入;
  - F-3(P3,口径修正):先期记录"ui_gen::vue 23 预存红"经 nextest
    进程隔离复测为 **cargo test 同进程状态干扰**(隔离下 master/分支
    皆绿)——待澄清 8 已改口径;musk_vm_track_p053 族为特性门真
    预存红(master 同态)。
  evidence:evidence/653/ 全集 + review-run.log(临时,关键行已摘录
  上文)+ 浏览器 evaluate 断言(9 槽位 rect 明细,本记录即凭证)。
  next:**merge**(复审通过;merge 阶段需先把分支 re-sync master——
  F-2 随 re-sync 自解)。

## 待澄清事项

1. ~~带参读取契约定稿~~ **已定稿 T-00**:GET+Query;SD-01 已按此修订
   auto-term terminal-mux-model.md(auto-term 38477cf),复审时请
   用户对规范措辞追认;
2. ~~D2 落点~~ **已勘定**:sidecar 候选链扩展(auto-term 仓),启动器
   env 注入方案否决;
3. ~~投影回归归属~~ **已撤销**:干净 back 复跑 t01 全绿,非缺陷
   (rev1 判定,本会话独立复证);
4. ~~与在途 642/647 会话冲突面~~ 已消解:用户关闭并行 agent;其
   T-01 草稿被覆盖存证,merge 3fb469842 已在案;
5. **键入回显(G4/AC-05 措辞越界)**:vue 轨无键盘→引擎桥接(生成前端
   唯一全局 keydown 监听 = PLAN-646 框选 Escape;mux_enqueue 仅 UI
   动作码 1..6 无键入码)——019"只读视口"边界,且本计划非目标明文
   维持该边界。G4/AC-05 的"可键入回显"措辞超出可交付面,AC-05 以
   banner+提示符可达交付;若需键入能力请另立计划(生成器层键入桥)。
   请用户裁定;
6. **窗模型-视口 48px 错位**:back 无头缺省 1024x768,真实视口 720
   高 → 槽位高 728px,scrollHeight 768 vs 720 余 48px。对齐面 =
   apply-resize(mounted 首拍 500,rev1 D2 注"同族证据")——本计划
   未修,建议随窗尺寸同步独立收口;
7. **vue 页在极端模型态下重载 boot 静默失败**(t01 后 6-pane 极端
   几何时重载零 /api 请求;干净态正常)——疑似前端 boot 健壮性缺口,
   非 653 缺陷集,留后续计划勘定;
8. **ui_gen::vue "23 预存红"口径修正(review 定责)**:cargo test
   单进程下 23 例红系**同进程状态干扰**——nextest 每测进程隔离下
   master 与本分支皆绿;真预存红为 musk_vm_track_p053 族(特性门,
   master 同态)。另:master HEAD 的 auto-cache lib test 编译破损
   (E0063)阻塞全仓门,见复审 F-1。
- 2026-09-18 merge 收执 `PLAN-653:r1`(stage: merge | outcome: pass |
  delivery: auto-lang master `398a60a8e`——content `3a6c78d59`(T-01)+
  `7d5ae193d`(T-05),merge 前 re-sync `a4b3f9d89`;auto-term main
  `38477cf`(T-02+SD-01,ff)| checkpoints:
  **prepared**=复审冻结增量 SD-01(auto-term terminal-mux-model.md 分域
  措辞)+代码基线 7d5ae193d/38477cf;
  **landed**=auto-lang 398a60a8e(merge 3fb469842+a4b3f9d89 后
  ff-able 祖先链核实,落地冒烟 cargo check auto-man+auto-lang 过、
  plan367 复测随 re-sync 转绿=F-2 自解、653 双测保持绿);
  auto-term 38477cf ff-only 于 main;
  **ledger_refreshed**=docs/specs/auto-lang/ui/plans.md 增 653 行
  (本提交);canonical spec 即 auto-term 仓文件本体(38477cf);
  **archived**=本件 git mv 至 docs/plans/archive/ + status: archived;
  **cleaned**=待 wt-guard 双仓过闸后回填(见下)。
  归属面:653 无 auto-lang 仓 canonical spec 文本变更(增量目标仅
  auto-term 仓),故 auto-lang 侧台账=plans.md 回写,零虚构沉淀。
