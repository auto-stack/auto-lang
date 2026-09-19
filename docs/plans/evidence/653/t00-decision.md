# PLAN-653 T-00 bounded investigation 决策工件(2026-09-18,work 会话)

## ① 带参读取契约定稿:**GET + Query**(D1 依据)

- 前端客户端生成器(auto-man,`@/lib/api`)对带参 GET 一律按
  `fetch(\`/api/mux/tab-id-at?i=${…}\`, {method:'GET'})` 发射(生成产物
  app/gen/front/vue/src/lib/api.ts:270 实测);api.at 源(`app/src/back/api.at:197`)
  亦声明 `GET`。
- api_gen.rs 主类型路径对 GET/DELETE 带参**既有** `{Fn}Query`-struct + Query
  提取机制(:1741/:1829 一带);仅 db-covered scalar 分支(PLAN-013 T2,
  :1570-1600)无 method 分流误发 Json。
- 结论:GET+Query = 后端单侧根修(本仓 api_gen.rs),零前端/跨仓改动;
  统一 POST+Json 则需同时改前端生成器 + api.at + 019 规范,爆炸半径大。
- SD-01 兑现:auto-term terminal-mux-model.md 带参读取契约分域措辞修订
  (rect-*/命令族维持一律 POST;标量 getter 族定为 GET+Query),
  auto-term worktree commit 38477cf。
- 前任会话残留草稿(untyped `Query<serde_json::Value>` + query_i64/bool/str
  容错助手路线,19:31 未提交)已实测可编译可运行(axum Query 对 Value 的
  Deref 使 `&query` 传参合法;scratch back 200 实证),但偏离计划 D1 点名的
  既有机制。按 D1 重实现为 typed struct 路线并提交 3a6c78d59;草稿 diff
  存证 t01-predecessor-wip-uncommitted.diff。
- 实测(修复后 scratch 从零生成 back,18080):
  `tab-id-at?i=0`→200 数值;缺参→200(serde default);`tab-title-at?i=0`→
  200 "shell 1";POST 对照 rect-kind→200;投影链 new-tab/split/layout 全对。

## ② back exe 实际落点勘定 + D2 落点:**sidecar 候选链扩展**(auto-term 仓)

- 实测:`auto run -r vue`(CWD=app 目录)经 `start_api_server` 以
  `cargo run --manifest-path <app>/rust-workspace/<name>-back/Cargo.toml`
  启动;cargo 的 `.cargo/config.toml` 发现基于 CWD,workspace 内的
  target-dir 配置不生效 → exe 落 workspace 默认 `rust-workspace/target/debug/`
  (真实产物 app/rust-workspace/target/debug/app-back.exe,Sep 18 18:43 佐证)。
- DLL 唯一常规落点 = `<repo>/target/debug/autoterm_core.dll`
  (auto-term/target/debug 实存;`app/rust-workspace/target/debug/`、
  `app/target/debug/` 实测无 DLL)——距 exe 目录 **5 级祖先**,旧 take(4)
  候选链恒落空 → 首 tick panic(term.rs:225)。与 022 §9 实测吻合。
- D2 落点 = auto-term `app/term.rs` sidecar 候选链扩展(exe 祖先 4→6 级
  覆盖仓根 + 新增 CWD 祖先 4 级兜底;env 优先/dist 同目录语义不变)。
  启动器 env 注入方案否决(仓特异性知识不应进 auto-lang 通用管线)。
- 阴性对照:旧 sidecar 于故障位置 exe → tick panic
  (dll-probe-oldsidecar-panic.log);修复版同位置 + CWD=app 目录 →
  tick 200 + `[term-dll] 加载引擎: …\auto-term\target/debug/autoterm_core.dll`
  (dll-probe-fixed-tick200.log)。真实 E2E 管线(无手动 env)同证据
  (e2e-run.log)。

## ③ 投影"单幅化"定界:**当前 master 生成面无缺陷**(T-03 关闭依据)

- 从零生成(scratch 副本,当前 master 二进制)的 db.rs 与部署版
  **逐字节一致**(api.rs/types.rs 亦同;main.rs 仅差 sidecar mod 行)。
- db.at `rects_recompute` 源(:340-)与生成 db.rs(:328-)逐行忠实对应,
  1fc4772d9(= plan-022 两提交合并,T-00 读明其真实身份)未破坏该面。
- 实测投影:新表→横分 50/50+divider→纵分深度 2(3 pane+2 divider)→
  t01 全 12 步(磁吸 480→500/钳位 50→100/键盘 700/MAX_PANES 帽 -2/
  close 重算)全对(t01-model-run-653.log)。"单满幅槽"形状仅与 zoom
  分支(zoomed!=0)吻合,疑似 022 观测会话残留 zoom 态或 GET-400 断链
  期间的第二手观察偏差;不构成 master 代码缺陷。
- 结论:T-03 无代码改动;若复现需带现场模型态重查(见计划待澄清)。

## ④ 附加发现(work 实测,移交复审)

- vue 轨**无键入桥接**:生成前端唯一全局 keydown 监听 = PLAN-646 框选
  Escape 处理;mux_enqueue 仅 UI 动作码(1..6),无键入码。G4/AC-05 的
  "可键入回显"措辞越出 019 只读视口边界(653 非目标明文维持)——AC-05
  以 banner+提示符可达为交付,键入回显留用户裁定。
- 页面在 back 模型处于极端几何(t01 后 6 pane)时重载 boot 静默失败
  (零 /api 请求);干净模型态 boot 正常。疑似前端 boot 对异常模型态
  的健壮性缺口,非 653 缺陷集,留待澄清。
- 并行会话曾在同一 worktree(19:31)落 T-01 草稿并以其构建产物跑过
  真机(其 app-back 进程因 DLL 缺口退出,反证 T-02 必要性);本会话
  已调和并提交,若该会话仍活跃需用户仲裁。
