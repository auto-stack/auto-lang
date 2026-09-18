# PLAN-648 T-00 决策工件:T-10 dev 跑法 VM api 委托断供定因

日期:2026-09-18/19 · 调查人:PLAN-648 work 会话(lang-648 worktree)
复现载体:`auto-term/app`(e9f813d 后树)+ worktree 构建的 `auto.exe`
(仪器:AUTO_LANG_HTTP_TRACE=1,注入 simple_http_json——api 改写家族
get_json/post_json/put/delete/patch 的唯一 HTTP 汇聚点,覆盖全部
emit_api_http_call 发射的请求)。

## 一、断点层级定界(主定因,已根修)

**断点唯一且在生成面:split 模式的 api-over-HTTP 改写只匹配裸名调用。**

证据链(修复前,t00-repro1.log):

1. `auto run -r vm` 全程 60s,api 改写家族 HTTP 请求**仅 1 次**:
   `POST /api/mux/new-tab → 200 body~1`。Tick 每拍执行且
   `[VM_HANDLER_OK]`(handler 无错完成),但零 api HTTP 流量。
2. 唯一那次 HTTP 来自 db.at:885/914 boot 逻辑里的**裸名**
   `mux_new_tab()`(api.at:140 的 #[api] fn)——裸名命中
   `api_funcs`(仅按裸名注册)→ 改写生效。
3. 前台 app.at 的全部 api 调用是**限定名**(`api.get_lines()` 等,
   `use back.api` 惯例)。codegen 原门卫(codegen.rs Plan 340 块):
   `matches!(call.name, Expr::Ident(_))` 显式排除限定调用,注释原文
   "qualified calls (db.all_notes) skip rewriting"。
4. 限定调用回落常规解析 → 命中 back 链扁平编译体(PLAN-633:api.at
   实现体随 back 链编入前台模块)→ **宿主进程内执行 db.at** →
   `auto.term` shims → 引擎 DLL。term_engine.rs 解析序(env → exe
   同目录 → 祖先 target/)在 auto.exe 宿主下全部 miss →
   "dll 缺席优雅降级(spawn 返 0)" → 全空返回。
5. 由此前台 mux 表恒空:Tab 条无标签("mux 计数空")、面板空白;
   db boot 的裸名 mux_new_tab 却经 HTTP 在 **back** 建了孤儿 Tab
   (back 侧 body~1)——前后台状态分裂。
6. 双合成入口同病:`synthesize_widget_module` 与 `synthesize_from_decl`
   (iced VM 轨真实入口)各有一份裸名-only 注册副本,须同步修。

**根修(T-01)**:两注册点增注册限定名键(扁平化全名 + 别名末段.fn
两形态);codegen 门卫放行 `Expr::Dot(Ident, fn)` 形态,按点连
func_name 查表。db.* 等非 api 限定名不命中表,不受影响;merged 模式
路径零改动(保留 CALL reloc,回归测试断言锁定)。

**根修后实证**(t01-verify4.log,70s 零交互):api 流量 ~1063 次全
200,tick→tab-count(1)→tab-id-at?i=0(单次迭代正确 break)→
window-width/height 稳态轮询;`tab-title-at` 返回 "shell 1"、
`pane-lines` 返回真实 Windows banner 行。截图
t01-window-rendered.png:Tab 标签 **"● shell 1"** 渲染、面板为
Windows cmd banner + `C:\Users\zhaop>` 提示符——AC-02 判据达成。

## 二、021 证据链遗留问题的归因修正

1. **"3 条 keep-alive = 前台轮询"是误读**:前台 api 轮询根本不存在
   (修复前限定调用全部进程内消化);那 3 条连接来自非 api 通道
   (媒体/基建),与内容渲染无关。故不存在所谓"轮询与渲染之间的
   第三断点"——真相是零 api 数据到达,空态渲染正确。
2. **AUTO_FFI_TRACE=1 → 前台连接=0 的敏感性**:该 env 仅被
   auto-term 侧消费(autoterm-core ffi.rs / term.rs 侧车),auto-lang
   全仓零引用。修复前前台本就无 api 连接,该现象与委托断供无因果;
   属 auto-term 侧仪器行为,不再阻塞本计划(AUTO_FFI_TRACE 敏感性
   如实机仍复现,归 auto-term 侧后续处理)。

## 三、次生发现(既有缺陷,被打通的委托链暴露——非本计划回归)

**事件 handler 再入破坏 HTTP-yield 中的任务状态**(三次实证):

- verify4(70s 零交互):全程健康,i 恒 0(对照组)。
- verify5:runaway 于 SetForegroundWindow+ALT keybd_event 注入后
  起始(t01-verify5-runaway-trace.log:转折点 tick→tab-id-at?i=0
  之后同 tick 内 i=1,2,3... 无界爬升,i≥2 万)。
- final:纯 `SW_RESTORE`(零键盘/鼠标注入,仅窗口事件)同样触发
  (t01-final-trace.log 尾部 i→8764)。

机制:Tick handler 在 HTTP `Waiting("http")` yield 期间,iced 再投递
任意窗口事件(restore/resize/activate/键入)→ DynamicComponent 再入
调用 handler → 同任务帧/单槽 `waiting_http_request_id` 被覆写 → 恢复
后局部变量(循环上界 n)取错值 → `i >= n` 永假 → 无界循环打满
UI 线程(窗口"未响应")。修复前这些 handler 从不 yield(零 HTTP),
故再入窗口从未打开——**既有 VM 调度缺陷,非本根修引入**。
已按技能路由记 KNOWN-DEBT(候选:handler 执行期间事件排队/
Waiting 任务互斥),不并入本计划范围。

## 四、处置摘要

| 项 | 结论 |
|---|---|
| 主断点 | 生成面:裸名-only api 改写注册+门卫(两合成入口) |
| 根修 | 限定名键注册 + Dot(Ident,fn) 门卫放行(T-01) |
| AC-02 | 实机达成(截图+1063×200 轮询+真实内容流) |
| "第三断点" | 不存在(021 观测误读,见 §二.1) |
| AUTO_FFI_TRACE | auto-term 侧仪器行为,与本断供无因果(§二.2) |
| 再入缺陷 | 既有 VM 调度缺陷,独立立项(KNOWN-DEBT 在案) |

证据文件:同目录 t00-repro1.log(修复前)、t01-verify4.log(修复后
对照)、t01-verify5-*.log / t01-final-trace.log(再入触发)、
t01-window-rendered.png(渲染实证)。
