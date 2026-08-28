# Plan 446 — VM 渲染后端实战薄弱点（auto-os-config Plan 007 现场报告）

状态: executing（用户授权复活 2026-08-28：归档态解除，§P/U1-U7 转正为批五实施；此前四批已复审归档沉淀 P446-1..4；G2-G5 悬空引用经裁决删除）；§I 四批：批一诊断 §L/§K、批二可用性 §N、
批三语义统一 §O、批四打磨 §Q——待 /auto-plan:review；§P 新报与剩余
低优先项见待澄清；worktree .worktrees/plan-446-dev 留置待 merge）
创建: 2026-08-25
来源: auto-os-config Plan 007（前端 Auto 化第二步 — VM 桌面版，已合并 main）。
该仓库把完整 Vue 配置编辑器跑上了 `render: "vm"`（iced 桌面窗口 + MCP 驱动 e2e），
过程中实证了一批 VM 后端缺陷。本文是全部 auto-lang 相关问题的汇总上报，按严重度分级，
每条含证据、复现载体与修复建议。完整探针过程与结论见
`auto-os-config/docs/plans/007-frontend-vm-desktop.md`（Phase 1/2/4 + VG gotcha 清单）。

基线: auto-lang master `3d45fb10d`（auto-os-config 所用 auto.exe 构建基线）。

---

## 0. 总览

| # | 类别 | 严重度 | 一句话 |
|---|---|---|---|
| A1 | 多 store 消歧 | P0 | 撞名/未声明方法静默落到错误 store |
| B1 | 事件参数 | P0 | store 列表循环上字段访问实参 → MCP 通道死亡，零诊断 |
| C1 | popover | P0 | 特定形态 popover 使整个模块解析失败被静默丢弃 |
| D1 | 动态值 | P1 | json.parse 是占位 shim，JsonValue 方法链不可用 |
| D2 | 动态值 | P1 | handler 嵌套读取语义混乱（两跳数组空/局部中转失效/两语境不一致） |
| D3 | 动态值 | P1 | 数组跨 fn 边界作实参丢失 |
| E1 | http | P1 | res.status() 恒返回哨兵 -2147483647 |
| E2 | http | P1 | builder 链用过之后同 handler 第二次 http 调用崩溃 |
| F1 | 静默失效 | P1 | handler 崩溃静默回滚全部状态写入、无任何诊断 |
| F2 | 静默失效 | P1 | 模块 parse 失败仅 WARN、模块静默不渲染 |
| G1 | vue codegen | P1 | widget 直连 store 导入生成错误路径 |
| F3/D4/D5/D6/E3/G2-G5 | 混合 | P2 | 见各节 |
| J1 | 渲染器 | P0 | 嵌套条件+循环组合的子树构建静默失败（逐要素全过、组合即死） |
| J2 | 渲染器 | P0 | 循环内容器级 key（col/row/div）杀死子树；button/text key 无害 |
| J3 | 状态绑定 | P1 | 新增 store bool 字段视图绑定恒 false（state 池与视图不一致） |
| J4 | 稳定性 | P2 | boot/渲染线程崩溃零诊断（exit -1 无 stderr，含 MCP 轮询触发） |

复现载体：auto-os-config main 分支（`D:\autostack\auto-os-config`）——
`auto run -r vm`（cwd=auto/）+ `AUTOUI_MCP_PORT=9320` + `node scripts/e2e-vm.mjs`；
探针历史在 plan 007 文本内（tmp/vm-probes 已按惯例退役，结论均已在案）。

---

## A. 多 store 支持（P0）

### A1 方法名消歧错位（撞名 fallback / msg 未声明 fallback）

**机理**（`crates/auto-lang/src/ui/handler_codegen.rs:247-290`）：`store.Method()` 改写为
`handler_<Store>_Method` 时按 STORE_MSG_MAP 以方法名匹配；`matched.len() != 1`（撞名或未声明）
时**静默回退 alias 表**（`store` → 先注册的 store），生成对错误 store 的调用。

**现场证据**（auto-os-config，三 store 工程 Modules/Collection/Theme）：
1. `SetSidecar` 只在 Collection 定义但**漏列 msg Msg 声明**（vue 容忍）→ vm 消歧查不到 →
   回退 Modules → link 期 `Undefined symbol: handler_Modules_SetSidecar`。
2. `Init`/`Select` 在 Modules 与 Collection 撞名 → 子 widget 的 `store.Init(...)` 一律解析到
   Modules（把 "roles" 传给了无参的 Modules.Init，静默无效果）。我们被迫把 Collection 的
   方法全量改名为 Open/Pick/NewEntity/SaveEntity/DelEntity 规避（见 collection_store.at）。

**修复建议**：
1. msg 声明与 handler 定义不一致 → **编译期显式报错**（vue 后端同样应报）；
2. 消歧失败（撞名/未声明）→ **报错并列出候选**，禁止静默 alias 回退；
3. （增强）允许限定调用 `collection_store.Pick(...)` 消歧。

**验收**：三 store 工程中漏声明/撞名场景给出编译错误信息（含 store 名与方法名）；
auto-os-config 可把 Collection 方法名改回 Init/Select 并正常链接。

---

## B. 事件参数与 MCP 通道（P0）

### B1 store 列表循环上的字段访问实参 → MCP 通道死亡

**现场证据**（vm_collection.at，已绕但历史在案）：view 中
`for ent in .store.list { button (text: ent.name) { onclick: .Pick(ent.name) } }` ——
启动即 wedge：日志止于 `AutoUI MCP: first state sync in view()`，MCP HTTP server
不 bind 或失联，进程随后退出，**无任何错误输出**。
对照组：同样的字段访问实参在**本地/嵌套 map 循环**上正常（sidebar_vm 的
`for m in g.members { onclick: .Pick(m.id) }` 工作正常）→ 疑
view-builder 对 **store 来源列表的 vmref** 在 event-arg 求值路径 panic/死循环。

另一形态：map 循环变量整只作实参（`onclick: .Pick(ent)`）不崩溃但 handler 内
`ent.name` 读取**静默失效**（选不中任何实体）。

**我们目前的规避**：store 增平行 `names []str` 字符串数组 + `for i, e` 索引参数
（041-auto-edit 模式）。此模式要求每个列表维护影子数组，成本高且易漏。

**修复建议**：event-arg 求值统一走 vmref 物化（与 view 文本渲染同一路径）；
panic 时至少输出到 stderr。

**验收**：`for x in .store.list { button { onclick: .F(x.field) } }` 在 MCP 模式下
正常触发且 handler 收到正确字符串。

### B3（P2，已知但求改进）autoui_type 实参置换

`autoui_type` 把输入文本作为 handler 第一实参，置换循环变量/`$event` 实参
（041 desktop_mcp.py docstring 已注记）。下游被迫"输入框 handler 一律单参"。
建议：autoui_type 增加可选 `args` 透传，或 dispatch 时对多参 handler 报错提示。

---

## C. popover 解析毒药（P0）

**现场证据**：vm_collection.at 的 view 内（嵌套于 row/col 深层）放置
```
popover (open: .confirm_open, x: 400.0, y: 300.0, ondismiss: .ConfirmCancel, class: "w-72 border rounded p-3 gap-2") { col { … } }
```
→ `collect_module_imports: parse failed … aborting due to 20 previous errors`：
**无文件/行列定位**，模块被**静默丢弃**（该 widget 渲染为空，应用其余部分正常，
极难定位）。同日对照：041-auto-edit 的 popover（`x: .ctx_x` 变量绑定、顶层位置）可用。
未进一步二分差异点（我们以普通 if 块替代确认层规避）。

**修复建议**：
1. 定位并修复该形态的解析失败（疑似属性组合或嵌套深度相关）；
2. **通用诉求**：UI scene 的 parse 错误必须带 文件:行:列；
3. **通用诉求**：入口模块或任一被引用模块 parse 失败应升级为致命错误
   （当前 WARN + 静默空渲染，违背 musk 028 确立的"显式报错优于静默"哲学）。

**验收**：上述 popover 形态可渲染；人为制造语法错误时 run 输出含定位且非零退出。

---

## D. 动态值管线（P1）

### D1 `json.parse` 是占位 shim

`vm/ffi/stdlib.rs shim_json_parse` 注释自认 placeholder（原样返回字符串）。
实测（script 与 UI merged 双模式）：`json.parse(s)` 后 `body.provider.kind` = 0、
JsonValue 方法链（`body.keys()/get()/type()`）全返回 None/垃圾。
**book-reader 示例所依赖的 `json.parse(raw)` + 点访问模式在当前构建不可用**。
我们全链路改走 `json.keys/get/get_at/type_of/len` 文本工具链（可用且可靠）。

**修复建议**：接通 Plan 340 的 JSON↔VM 转换器，或删除占位并让 `json.parse`
显式报错（现状是最坏的"看似可用"）；顺带让 `json.keys` 文档写明裸 key 语义
（见 D6）。

### D2 handler 嵌套读取语义混乱

同一表达式在三个语境行为不同（全部实证，L/N 探针）：

| 表达式 | widget handler | store handler | fn 模块 |
|---|---|---|---|
| `r.ok` / `r.text`（单跳） | ✓ | ✓ | ✓ |
| `r.data.firstGroup`（两跳标量） | ✓ | **空** | ✓ |
| `r.data.modules`（两跳数组） | **空** | **空** | ✓ |
| `let d = r.data; d.x`（局部中转） | **失效** | 失效 | ✓（参数/循环变量读取） |
| model 数组循环变量 `m.id` | 失效（静默不匹配） | 失效 | ✓ |

fn 模块内一切正常（含 map 构造/循环读取/重建，L1-L3 实证）——说明是
handler 合成路径的值物化缺陷而非 VM 本体。**修复建议**：handler 上下文与
fn 模块统一语义；至少在文档钉死差异（我们已作为 VG12/VG4 规范登记）。

### D3 数组跨 fn 边界作实参丢失

`pub fn arr_len(a []any) int { return a.len() }`；调用 `arr_len(fetch_mods_flat())`
返回 **0**（同数据直接 bind 后 `.len()` 为 7）。数组作为 fn 实参整体丢失。
（VG13；我们把 API 面全部改为"文本 + 计数 + 逐项 getter"规避。）

### D4 `.find(闭包)` 在 handler 静默失效（P2）

`.modules.find(x => x.id == id)` 在 store handler 不匹配（同数据 for 循环 +
fn 模块 find 均正常）。与 D2 同源。

### D5 `json.get_at` 仅接受 JSON 文本（P2）

对 VM 数组（如 `json.keys()` 的返回）调用 get_at 返回空、无告警。建议：
类型不匹配时 panic 或返回 Option 语义文档化。

### D6 `json.keys` 字母序（P2）

serde_json 未开 `preserve_order` → keys/遍历为字母序，与 vue 轨
Object.entries 插入序不一致（跨后端 UI 字段顺序 parity 破坏，
auto-os-config 已登记已知偏差）。建议启用 preserve_order feature。

---

## E. http natives（P1）

运输配方实测收敛为"get_json 取文本 + put/post/delete 裸调 + 写后 GET 验证"，
以下三个 API 均不可用：

- **E1** `res.status()` 恒返回哨兵 `-2147483647`（活 daemon、死端口皆然）；
- **E2** builder 链 `http.request(...).header().body().send()` 本身可发出请求，
  但**同一 handler 内随后的任何 http 调用崩溃**（handler 原子回滚，无诊断）；
- **E3** `res.body()` 在错误响应上返回垃圾值（偶见巨大数字串）。

另：daemon 不可达时 `http.get` 返回非 nil、`get_json` 返回
`{"error":"error sending request for url …}` 文本——**可判别但契约未文档化**。
建议：修 status/builder；或官方提供 `get_json` 失败返回 nil 的约定 + 文档。

---

## F. 静默失效类（P1）

- **F1 handler 崩溃静默回滚**：handler 内任一步崩溃 → 该 handler 的**全部**状态写入
  回滚、无任何诊断输出（我们靠逐步插桩标记定位，成本极高）。建议：崩溃时 stderr
  输出 handler 名 + 崩溃点前后字节码/语句，并考虑不回滚已完成写入（或可配置）。
- **F2 模块 parse 失败静默丢弃**：见 C1-3。
- **F3 map 字面量求值陷阱（P2）**：字面量内**空数组字面量**（`{members: []}`）崩溃；
  字面量内的 `.len()` 等**方法调用求值为 0**。均无告警。建议：字面量内禁调用
  （编译期报错）+ 空数组给默认或报错。
- **F4 `substr(a,b)` 闭区间（P2，文档项）**：与常见半开区间直觉不符（我们 unquote
  差一 bug 即源于此）。请求文档显式标注。
- **F5 vm 实例偶发闲置死亡 / MCP 失联（P2，观察项）**：多次出现"进程存活但 MCP
  listener 消失"，全量杀进程后恢复；不排除环境/GPU 因素，列出供交叉验证。

---

## G. vue codegen（P1）

### G1 widget 内 store 直连导入生成错误路径

widget 里 `use modules_store: Modules`（015-notes 合法形态）在 vue codegen 生成
`import … from '@/stores/useModulesStore'`，而部署产物在 `src/stores/auto/` →
vue-tsc TS2307。006 惯例（ext composable facade）实为长期绕行。
auto-os-config 的 vm 专属组件当前以 regen 部署过滤规避
（`auto/gen/regen.sh` VM_ONLY 清单）。
**修复建议**：vue codegen 的 store 导入路径与实际生成位置对齐（或可配置）。

---

## H. 对下游的影响快照（供优先级权衡）

auto-os-config vm 轨为规避上述问题付出的常设成本：
- store 方法全量改名（Open/Pick/NewEntity/SaveEntity/DelEntity）+ vue 侧调用点同步；
- 每个列表维护平行字符串数组（`names`/`entry_keys`）+ 索引参数模式；
- API 面全文本化（约 55 pub fn 的文本管线，含 JSON 转义/重建手术）；
- 输入框一律"单参 + Apply 按钮"模式（B3）；
- 确认层用普通 if 块替代 popover（C1）。
这些 workaround 全部登记于 `auto-os-config/auto/README.md` 的 VG 清单——
上游逐项修复后对应条目可删。

## I. 建议的实施切分

1. **第一批（诊断性，收益最大）**：C1-2/C1-3（错误定位 + 模块丢弃致命化）+
   F1（崩溃诊断）——让后续所有问题可被下游自行定位；
2. **第二批（vm 可用性）**：A1（消歧报错 + msg 校验）、B1（event-arg vmref）、
   E1/E2（http）；
3. **第三批（语义统一）**：D1/D2/D3（动态值管线统一）、D6（preserve_order）、
   G1（vue store 路径）；
4. **第四批（打磨）**：F3/F4/B3/D4/D5/E3/G2-G5。

每批可独立验收；第一批落地后建议在 auto-os-config 复跑
`node scripts/e2e-vm.mjs` + `./scripts/e2e.sh` 双门禁做交叉回归。

---

## K. 批一首个落地（2026-08-26，auto-musk 045 现场根因修复）

**K0 handler 重写器不走查 try/catch 体（新发现，A/F 族根因之一）**：`rewrite_stmt`
的 match 无 `Stmt::Try` 臂——handler 体内 `try { .error = ... } catch { .x = ... }`
的状态引用漏成裸 `self`，VM 合成报 "Undefined variable: self"，handler 整体毒化。
auto-musk 七个 try/catch 形态 HTTP handler（AgentConfigs.Init + RelayStore 六件）
全体中招；此前被 let-重赋值错掩蔽，源清理后暴露。**已修**（分支
plan-446-try-rewrite，3429e3432）：补 Try 三体（body/catch/finally）走查镜像
Block + 回归测试 `rewrites_state_refs_inside_try_catch`。附带同款现场发现的
未登记缺口（musk 侧已规避、上游待议）：obj 接收者方法族零 VM native
（`obj.slice`/`obj.find` 链接死；`list.join` 经 `[]str` 类型注解可绕）、
普通 fn 直读 store state 不可链接（`ensureAssistantMsg` 类，需拆纯 fn）。

**K1 合成模块 >32KB 循环回跳 i16 回绕（新发现）**：全部 fn 可编译后 musk
synthesized App 模块字节码越 32767——range-for 回跳以 `as i16` 取绝对位相减，
回绕使 debug 构建减法溢出 panic 且随 fn 排列序 ~50% 波动。**已修**（分支
plan-446-try-rewrite）：九处回跳位点改 isize 域相减再收窄；顺带揭示
**32KB/模块 i16 偏移上限**为后续大应用的架构性约束（ widening 到 i32 或
分模块为长期项）。

## L. 批一实施记录（2026-08-26，auto-lang worktree plan-446-batch1）

按 §I 切分的第一批（诊断性）+ J1/J2 攻坚，全部落地：

**C1-2 parse 错误带定位**：`collect_module_imports` 的 parse 失败分支新增
`positioned_parse_errors`——从 miette `Diagnostic::labels()` 取 span 偏移对照
手边源码换算 `line:col`（MultipleErrors 递归展开，上限 20）。回归
`plan446_batch1_tests`（positioned_parse_errors_reports_line_col /
multi_error_cap）。

**C1-3 模块 parse 失败致命化**：全局登记表 `ui_module_parse_failures()` +
stderr 醒目块；`VmBridge::new_with_children` 构造时检查并升级为 Err（收集侧
保持非致命——投机解析路径安全）。效果：入口/被引用模块 parse 失败 = boot
失败非零退出，不再静默空渲染。

**F1 handler 崩溃诊断**：dynamic.rs 三处吞错位（dispatch 主路径/850 简路径/
fire_init）从 env 门控升级为无条件 stderr `[VM-HANDLER] Widget.Msg failed`；
vm_bridge 两处 call 位附加 `crash ip=0x.. in handler_X`（VMError 无位置信息，
task.ip 指向失败指令附近）。

**J1 根因修复（本批最大收获）**：J1/J4 时代的"空壳"三症状（子 widget 裸引用/
详情循环不建/旧工程退化）**全部是 MCP 快照层假象**——styled_vtree 仅在
`__bounds_collected` 回路后设置，而 bounds 仅在 view() 脏重建时请求；boot 后
无重建则 styled 永不落盘，快照首问必走源树回退（子 widget 裸标签、for 未展开
= 观感"空壳"）。视图构建与窗口渲染实际一直正常（headless 全展开实证）。
修复：dynamic_view 的每帧 MCP 同步块直接推送
`view_to_vtree_with_paths` 快照（bounds/计算样式注释仍由原回路后补）。
**验证**：os-config 两尖信号双绿——`vm-b3-check`（b955004 导出重建于
worktree tmp-corpus，roles/load/assistant true + inputs 7/Apply 4）与
`vm-detail-dump`（b3 与 HEAD 语料：6-7 inputs/4 Apply，详情区全渲染）。
e2e-vm 剩余败项属 446 在册的 harness 自身问题（按压错位/门禁待修）+ 首问
时序，归 os-config 侧。附带快照格式修复：input/textarea 的 placeholder 构成
节点 body（此前仅 class/events 开体，无样式 input 打裸行丢 placeholder——
e2e findInputByPlaceholder 断言面）。

**J2 验证解除**：`for` 内 keyed 容器（col/row/div 带 key）当前 master +
本批 = 子树正常渲染（j2check 探针：keyed-alpha/beta 双子树全出）。原"致死"
症状为快照回退时代观察。**下游绕行可撤**（os-config 侧 VG 清单相应条目 +
vue key 提升的配对验证由其自行安排）。

**回归**：lib+ui-iced+test-vm-files 全量——失败集 = cookbook_vm ×3（447 附录
在案基线红）+ benchmark_downcast（在案负载偶发）+ md_hidden_classes（master
同红）；charts_gallery 负载偶发复跑绿。musk 探针（plan442）1 passed。

## E4. 默认认证头/默认 query native（2026-08-26 立项即落地，plan-446-e4）

**来源**：auto-musk KNOWN-DEBT 442/045（setupAuthFetch VM 侧无法独立落地——
Http.get 仅收 url；builder 链踩 E2；post_bearer 仅 POST）。musk VM 前端的
鉴权 API 消费面（Bearer musk_jwt + workspace query）自此出现,需求升级。

**方案**：进程级默认项三 native——`Http.set_default_header(name, value)` /
`Http.set_default_query(name, value)` / `Http.clear_default_auth()`
（ID 3139-3141）。注入点 = 两个 send 汇聚处:plain handle 族
（spawn_async_http_handle,GET/POST/PUT/DELETE/get 族共用）与
http.request builder 路径（默认头先落、显式 header 后设覆盖；默认 query
追加 URL）。消费侧:musk `ports/platform.vm.at` platformSetupAuthFetch 实装
（读 localStorage musk_jwt/musk_workspace——442 native 桥同后端,双轨键互通）；
token 变更经 login 页 Submit 后 platformRefreshAuth 重注入（web adapter no-op
——拦截器逐请求读 storage,且避免二次 monkey-patch 的 query 重复追加）。

**验证**：单测端到端（真实 TcpListener 断言线上收到 `authorization: bearer …`
头与 `?workspace=…` query,含同名替换/已含 ? 分隔单元面）;musk 探针
（plan442）3/3 绿且 App.Init 零失败行——F1 诊断当场抓到首个实现版的
native ID 撞车（3132 与 value_get_array 冲突;catalog 存在多形态条目,
"最大 ID"扫描须含非 Void 返回型,教训入册）。

**已知边缘**：auth_store.Me 内部 Logout()（会话过期）无法跨层调端口,
VM 默认头滞留至下次 login/register/重启——UI 化 logout 时在派发点补调
（musk 侧注释在 platform.vm.at）。

## J. Plan 008 批 4 增补（2026-08-26，os-config 视图统一现场报告）

来源：auto-os-config Plan 008（vue/vm 视图统一）批 4 调试实证；
登记惯例同本计划主体（007 现场报告）。上游已顺带解决的对账项：
oninput/onchange 文本实参契约（原 wip(plan008) 两提交）已落库生效，
auto-os-config vue 轨三套 e2e + regen 在纯 master 下全绿。

### J1 嵌套条件+循环组合的子树构建静默失败（P0，当前 vm 门禁阻塞）

**现象**：统一后的 collection_browser 详情区不渲染——结构为
`if selected_name != nil → if loading == false → 富工具栏 + 确认行 +
if is_read_only == false → col → for e in store.entries { kind 分发 }`。
state 池一切正常（entries 4 vmref、selected_name 可读），子树就是不出现。

**实证过程**（逐要素二分，全部单独通过）：loading 门 ✓、富工具栏（含
prop + store 双重条件）✓、textarea ✓、单 kind 行 ✓、text-key ✓、
007 逐字结构（更浅嵌套 + `!= ""`）✓ 可渲染；**组合形态 ✗**。变形矩阵
（单/双变量循环 × wrapper/无 wrapper × key 位置）全部失败——指向
view-builder 构建期的路径相关缺陷，非单一要素。

**复现**：auto-os-config worktree `node scripts/e2e-vm.mjs`，失败固定在
`detail inputs/applies missing`（boot/导航/列表/实体选中已全过）。
**修复建议**：优先做 I.1 第一批的诊断性（构建失败显式化），再定位
aura_view_builder 对深嵌套 Conditional/ForLoop 的路径。

### J2 循环内容器级 key 杀死子树（P0）

**现象**：循环体内 col/row/div 带 `key:` → 整个子树在 vm 不渲染；
同一 key 挂在 button/text 上无害；双变量循环内的 keyed wrapper div
同样致死。
**下游绕行（已落库 master 8491c7a71）**：分支首 text 挂 key + vue
codegen 的 v-for wrapper key 提升（`find_loop_child_key`，含单测）——
vue R006 强制 keyed wrapper 与 vm 容器 key 致死形成的死锁由此解开。
**修复建议**：view-builder 对容器 key 的处理对齐 vue 语义（身份提示，
不是构建开关）；修复后可撤下游绕行。

### J3 新增 store bool 字段的视图绑定恒 false（P1）

**现象**：同一 widget 内，state 池（autoui_state）返回新增 bool 字段
true，视图绑定 `if .store.X` 恒 false；旧 bool 字段正常、新增 list
字段正常、与字段名无关（重命名复现）。
**下游绕行**：nil 比较（`selected_name != nil`，双端语义已验证一致）。
**修复建议**：审查 state 池与视图绑定的字段同步/注册机制对
"运行期前新增字段"的处理。

### J4 boot/渲染线程崩溃零诊断（P2）

**现象**：2026-08-26 晨 master（含在途 renderer.rs 构建）`auto run -r vm`
boot 即崩，exit 0xFFFFFFFF、无 stderr/panic，三连复现；同日午后新提交
后 5/9 步通过。与 os-config Plan 008 Phase 0 登记的 MCP 轮询硬崩溃
（空闲 app + 2Hz 轮询 ~30s 内 40-60% 概率）同类：**崩溃零诊断**。
**修复建议**：panic hook 落盘 + 最小崩溃现场（含 I.1 第一批）；下游
门禁已用自愈重试缓解，不作阻塞项。

### J 批增补（2026-08-26 午后，os-config Plan 009 接管会话——J1 视图侧二分定案）

环境：auto-lang master `86460a197` 干净重建（11:09；注：10:14 存量二进制
疑似含已 reset 的在途 renderer.rs 半成品状态，boot 即空壳/三连崩——重建
后恢复 J1 登记签名）；os-config 分支 `plan-008-view-unification`
（3d9c828：含条件扁平化 + 007 循环形态回归）。

**J1 二分矩阵（详情面板内 for 循环，全部形态均只出 1 个空 wrapper col、
子树零构建；同应用同时刻对照全建）**：

| 变量 | 试过的形态 | 结果 |
|---|---|---|
| 循环变量 | 单变量 `for e` / 双变量 `for i, e` | 均 ✗ |
| wrapper | 无 / `div (class:"contents")` + key | 均 ✗ |
| 子元素 | text / button / 8 类 kind-if 分支族 | 均 ✗ |
| 滚动祖先 | 无 / `scroll (style:"flex-1")` 直包 | 均 ✗ |
| 数组元素 | 对象（vmref）/ 纯字符串（entry_keys） | 均 ✗ |
| 条件覆盖 | 双层 if 下 / 面板顶层零条件首子位 | 均 ✗ |

对照组（同快照内全建）：侧栏 view_names/standalone/groups 循环
（container/overflow-auto 祖先）；列表区实体循环——且后者位于
`if names.len() > 0` 条件链之下，**"条件+循环组合"不足以刻画**：列表区
if 下循环可建、详情面板零条件顶层循环不可建，差异指向构建路径本身
（flex-1 第二子位 col？待 renderer 侧定位）。已排除：input 的
`"type"` 属性（移除无变化）。

**J1 之外的正面信号**：条件扁平化（`if A { if B }}` → `if A && B`）令
详情区从整区不建恢复到工具栏按钮组/sidecar textarea/fields 容器正常
构建——深嵌套条件确实是诱因之一，但循环体未被救回。

**旧工程整体退化（J1 家族扩大面）**：b955004（os-config 批 3 提交，当时
vm 门禁两连绿）整个 `auto/` 目录 git archive 导出后，在当前 master 二
进制上 = 空壳（root row 下 `Sidebar` 子 widget 裸引用不展开、主区空、
boot 零诊断）。同二进制下 os-config HEAD（批 4 后）应用侧栏正常。嫌疑
序列：plan-451 actions DSL vm 全链路（895b7d413，08-26 07:53）/
plan-450 iced 面板（715bc7dc5）/ plan-041a schema（86460a197）。

**J3 同根新症状**：工具栏 `text (text: .store.selected_name)`（`?str`，
运行期赋值）文本绑定不渲染（state 池可读、`!= nil` if 门正常）——疑与
J3 同根（state 池 vs 视图绑定同步），字符串字段亦有份。

**修复验证建议**：以 os-config 两探针为尖锐信号——
`tmp/vm-detail-dump.mjs`（详情循环，当前红）与 `tmp/vm-b3-check.mjs`
（批 3 导出回归壳，当前红）；两者转绿 + `node scripts/e2e-vm.mjs`
9 断言即 J1 族解除。下游门禁侧另登记：e2e-vm 按压对自注册模块
"Harness Roles"错位（active_id 期望 roles 实得 musk-harness-roles，
探针同场景正常）——门禁脚本自身待修，勿计入上游。


## M. auto-musk PLAN-046 obj 族基线交付(2026-08-27,供 plan454 实施消费)

auto-musk 侧为清偿其 VM 规避形态(KNOWN-DEBT 046-A/T4 清册 R1-R4),已把
动态接收者方法族的注册/路由/链接三层基础设施 + shim 实体合入本仓
master(合并点 0737c26f3,WIP 内容提交=c176c4533):

- shim(native.rs):shim_obj_keys/shim_obj_values(ObjectData 优先 +
  GenericInstance 兜底,键序确定性排序,不支持形态显式报错而非静默零)+
  shim_obj_find(委托通用 list-find)
- 注册四表:native_catalog 宏行 2090/2091/2092、NATIVE_ID_ENTRIES 白名单、
  TYPE_CANONICAL_MAP("obj"/"Object" -> "auto.obj")、stdlib 启动面
  register_shim_by_name 三件——注意 peek_qualified 没有 lazy 注册,
  只进宏表不进白名单时链接期才会炸
- codegen 模块路由表:("Object","keys"/"values") 显式映射两条

### plan454 需收口的三缺口(端到端最后一公里)

1. Option 返回的无类型路径传播:动态接收者 .find(...) 已达 shim 且正确
   返回,但结果经无类型 codegen 路径回传后语义丢失(实测观察到 0 值);
   需要确定 None/命中值在非 typed 栈上的表示约定。
2. 谓词闭包 x GET_FIELD 协作:obj 元素堆形态下闭包体内字段读取链。
3. 返回值静态型别标注:auto.obj.* 结果需标注为 List 型,使下游 .length/
   for-in 走 ARRAY_LEN/HOF 通道而非 GET_FIELD 字段读退化。

### 验收靶子(现成)

crates/auto-lang/src/plan046_obj_natives_tests.rs:两个 #[ignore] WIP 用例
(dynamic_find_with_predicate / object_values_returns_array)转绿即判语义层
收口;另有注册表数据断言 x2 守回归。

### 协调注记

- musk 侧 KNOWN-DEBT 046-A 行与本节互指;语义收口后 auto-musk 回归执行
  T4 清册 R1-R4 源回撤 + T13 终验(/auto-plan:work 续跑 PLAN-046)。
- plan-454 worktree 分支早于本基线创建,实施前先 git merge master
  (或 rebase);当前两改动文件面零重叠(auto/lib/*.at vs vm/*)。
- 勘误备案:c176c4533 的提交消息被跨仓会话 amend 误改(树内容与原
  提交 750b7d98e 全等,零损失;原消息已存该提交 git notes)。下次触碰
  454 计划文档时建议补一行更正。

### 收口回填(2026-08-27,plan-454 Phase E)

三缺口已全部闭合,两 #[ignore] 用例转绿(plan-454 分支提交 9f40be552):
缺口① = shim_obj_find 语言级 TAG_NULL 契约(弃 -1 哨兵);缺口② =
谓词闭包参数域捕获(0x8000 旗标 + CLOSURE 执行期绝对槽解析)与元素
Value 通道(i32 快路径 tag 位误读剔除);缺口③ = auto.obj.* 编译期路由
强制 + infer_native_return_type 型别标注(keys/values→Array,
find→NestedObject)+ for-in Array 源走索引循环通道。附带:harness 的
stdout 读取通道修复(此前 WIP 用例读 main 返回值恒空串)。

## N. 批二实施记录（2026-08-28，auto-lang worktree plan-446-dev，已合入 master）

按 §I 第二批（vm 可用性）实施，A1/E1 完成、E2 部分收敛；B1 未在本轮实施（见待澄清事项）。

**[✅ 已完成] A1 多 store 消歧显式报错**（handler_codegen.rs，提交 ce513361b）：
1. 消歧集合从"仅 Msg 变体"扩为 **Msg ∪ on-block 处理器 ∪ 生命周期名**——
   "handler 已定义但漏列 Msg 声明"（os-config SetSidecar 现场，§A1-1）自动
   解析到正确 store，不再回退错误目标；
2. 合格化调用 `Store.Method(...)`（alias 即 store 真名、多 store 工程）直接按
   alias 定位，不再被方法名匹配劫持（§A1 修复建议 3 的限定调用增强）；
3. 泛型接收 `store.X`：撞名（≥2 store 命中）→ 显式报错列出候选 store 名；
   未声明（0 命中）→ 显式报错列出在场 store；两者禁止静默 alias 回退，
   错误经 thread-local 收集、synthesis 收尾升级为 Err → boot 致命（与 C1-3
   同哲学）。单 store 工程保持静默 fallback（vue 轨兼容性）。
回归：plan446_a1_* 4 用例（qualified/ambiguous/undeclared/single-store-compat）
+ handler_codegen 13 用例 + vue store 相关 40 用例全绿。

**[✅ 已完成] E1 res.status() 哨兵修复**（engine.rs + codegen.rs，提交 551b6f9c7）：
根因链（实证）：
- `http.get`/builder `.send()` 返回的是 HTTP_RESPONSES 线程本地表里的**裸句柄**
  （非堆对象）；`res` 的静态类型在不同语境坍缩为 str/int/User(Response) 不等——
  `.status()` 由此被编译为 `str.status`/`Unknown_status`/按声明面
  `Response.status(self, code)`（服务端 setter！双参）等错误目标；
- 落入未定义调用的静默兜底 → 哨兵值/栈槽被吞/E2 崩溃（同一根因三症状）。
修复：
1. **运行期兜底**（engine.rs CALL_SPEC，判据收窄："小正整数句柄且不在堆上、
   但命中 HTTP_RESPONSES 表"）：`.status()/.status_code()/.body()/.header(k)/
   .header_get(k)` 直接路由到 shim 支撑的只读 native（NATIVE_RESPONSE_STATUS_CODE/
   BODY/HEADER_GET，2216-2218）；
2. **编译期 arity 分流**（codegen.rs，镜像 plan454 obj 路由先例）：零参
   `.status()` → `Response.status_code`；一参 `.header(k)` → `Response.header_get`。
验收：真实 TcpListener 断言——`res.status()` 返回线上真值 201
（plan446_batch2 e1_res_status_reports_wire_status_on_get_handle 绿，
旧代码红：stdout 为垃圾值）。

**[✅ 已完成] E2 builder 链后续调用崩溃——协议级根因修复**（engine.rs + stdlib.rs，提交 89adb9bdb）：
轨迹实证（CALL_NAT/CALL_SPEC 运行期日志）钉死根因：链式 `.send()` 经
CALL_SPEC 路径执行后置位 `task.waiting_http_request_id`，而 **CALL_SPEC 执行
路径没有 CALL_NAT 的挂起/重试协议**——标记无人消费；此后任何 CALL_NAT
（如 `.status()` 查询）撞上 stale 标记即无限 rewind+Yield（40 秒实测
826,488 次自旋；此前观察的"栈下溢崩溃/哨兵值"是同一失序在不同
发射形态下的另一表现，即现场"同一 handler 内随后的任何 http 调用崩溃"
的协议级根因）。
修复：engine CALL_SPEC 对"接收者堆对象 tag == RequestBuilder"的
`.header(2)/.body(1)/.timeout(1)/.json(1)/.send(0)` 直调 shim 支撑的 native
族（NATIVE_HTTP_REQUEST_BUILDER_* 2235-2239，目录中无实装的
RequestBuilder.*(2260-2264) 名面被绕开）；send 以**同步 drain 完成 Yield
协议**——轮询 ASYNC_RESULTS 就绪（30s 上限，超时报错）后触发 shim 重入
（其重入分支清位+推柄），waiting 标记零残留。新增非夺取式就绪探测
`async_http_result_ready`（stdlib.rs，与 take 语义的 check_* 并存）。
勘误备案：最初版拦截缺同步 drain（曾致挂死），已由带协议完成的版本取代。
验收：`chain(header/body/timeout/send) → built.status()==204 → http.get →
res.status()==200` 全链真实 TcpListener 断言绿；e2 探针解除 #[ignore]。
回归：plan446 全系 8 用例 + http 24 用例 + request_builder 全绿。

**[✅ 已完成] B1 store 循环字段访问实参——验证解除**（corpus+探针，2026-08-28）：
复现载体入库 `test/ui/plan446_b1_store_loop/`（现场同款：store 字段
Array<str> 标注装 obj 字面量、`for ent in .store.items` + `onclick:
.Pick(ent.name)`、`.Pick(name)` 带参 handler）。双探针在现 master 直接绿：
- 渲染面：build+首次视图渲染在 60s 限时线程内瞬时完成（无 wedge），
  两按钮渲染、onclick 实参物化为字段值 alpha/beta、无 VmRef 裸引用泄漏；
- handler 面：payload 编码→decode→`on_with_input_for` 分发后 `.picked`
  收到正确字符串。
判定：B1 的 wedge 已被批一 §L 的 J1 根因修复（styled_vtree 首问竞态/
视图同步路径重构）顺带治愈——原症状属快照回退时代观察，与 J2 同性质。
本批交付=corpus+探针锁行为；**下游绕行可撤**（os-config 的影子
names[]/索引参数模式，§H 清单对应条目可删）。

## O. 批三实施记录（2026-08-28，auto-lang worktree plan-446-dev，已合入 master）

按 §I 第三批（语义统一）实施：D1/D6/G1 修复落地，D2/D3 实证已愈、
探针锁定。开工前先逐项探针（基线距离现场报告已多批合入）：

**[✅ 已完成] D1 json.parse 接通 Plan 340 转换器 + 文档 native 双态化**：
1. `shim_json_parse_vm`（stdlib.rs）替换占位 rust_fn 版（engine
   AutoVM::new 的 inventory 覆盖块注册，与 str.contains 同款惯例）：
   JSON 文本 → `json_to_vm_value` 物化（对象→`__json_object`
   GenericInstanceData、数组→ListData<Value>）；GET_FIELD/for-in 运行期
   直读实例自身 field_names，无需注册模板。非法 JSON 显式报错
   （占位时代静默透传是"看似可用"最坏形态）。
2. **兼容面收口（关键）**：既有 corpus 的 `json.get(json.parse(x), k)`
   文本工具链会被 1 打破——六个文档类 native（get/get_at/has_key/len/
   keys/is_valid）双态化：首参是堆文档时经 `vm_value_to_json` 序列化回
   JSON 文本**原位替换**再走原 rust_fn 路径（片段切片/type_of 语义
   字节级不变，ffi_dual_003/008/009 全绿）。两种 idiom（点访问 /
   get 工具链）自此互通——正是"语义统一"的批目标。
验收：d1_json_parse_supports_dot_access / d1_json_parse_array_iteration
（点访问+标量字段+数组迭代元素字段读）绿；json 全系 26/26 绿。

**[✅ 已完成] D6 json.keys 插入序**：workspace `serde_json` 开
`preserve_order`。与 vue 轨 Object.entries 语义对齐（跨后端 UI 字段
顺序 parity）。验收：d6_json_keys_insertion_order（zebra,alpha,mid
插入序断言）绿。**全量回归零波及**（见下）——serde_json Map 换
IndexMap 后端的全局行为变更在 3752+3227 用例中无一处期望字母序。

**[✅ 已完成] G1 vue store 导入路径可配**：`ComponentGenOptions.
store_import_prefix`（默认 `@/stores` 不变）+ `VueGenerator::
with_store_import_prefix`，SFC import 行 `'{prefix}/use{X}Store'`。
部署管线把生成物迁至 `src/stores/auto/`（os-config regen 布局）时配
`@/stores/auto` 即对齐，vue-tsc TS2307 消除；006 惯例（ext composable
facade）与 regen VM_ONLY 过滤两套绕行可撤。验收：
test_g1_store_import_prefix_configurable（默认不变回归守卫 + 可配前缀
落 import 行）绿。

**[✅ 验证解除] D2 handler 循环变量字段读**：corpus
`test/ui/plan446_d2_handler_read/`（现场矩阵"model 数组循环变量 m.id
失效(静默不匹配)"行——App handler 内 `for m in .items { if m.name ==
"alpha" { .picked = m.name } }`）在现 master 直接绿。多批值通道工作
（plan454 E 元素 Value 通道 / 446 批二 B1 物化链）已顺带治愈；
本批交付=corpus+探针锁行为。

**[✅ 验证解除] D3 数组跨 fn 实参**：探针 `arr_len(make_list())==3`
（obj 字面量数组跨 fn 边界）直接绿——同上顺带治愈，探针入套件锁定。

**环境注记**：worktree 内路径依赖 `autodown-core`（`../../../auto-down/…`）
自 plan-019 合入后从 worktree 解析不到——`.worktrees/auto-down` 目录
联接（junction）指回主仓旁的 auto-down 检出解决；后续 plan worktree
沿用此约定。

**回归**：ui-iced 全量 3751/3752（唯一失败 = md_hidden_classes_parse，
批一 §L 在案 master 同红）+ 默认 feature 日常套件 3227/3227 全绿 +
plan446 全系 10/10 + json 26/26。

## P. Plan 010 增补（2026-08-28，os-config 双端一致性现场报告）

来源：auto-os-config Plan 010（vm 轨一致性对拍 + 清偿）。该仓把双轨像素
对拍做成了常驻门禁（`scripts/track-parity/`：vue=Playwright 1440x900@1x，
vm=每视图独立 `auto run -r vm` + autoui_screenshot，pixelmatch + 分区
metrics），过程中实证下列缺陷。全部条目已在该仓 workaround/登记，不阻塞
消费；下游终值（009 阶段一 css-era 基准 0.00% 零回归前提）：侧栏默认视图
diff% 0.97、最复杂编辑器视图 6.72。
观察基线：master `1487b5c5d`（含批二 A1/E1/E2）；报告时 master `f536c1ba1`
（批三 D1/D6/G1——语义管线修复，与本节渲染层条目无交集）。

| # | 类别 | 严重度 | 一句话 |
|---|---|---|---|
| P1 (=下游 U1) | 事件路由 | P0 | 集合页实体列表循环构建后，侧栏任意 press 被接受但 active_id 冻结（全局死导航） |
| P2 (=U2) | action_mapper | P1 | autoui_type 合成输入把内联 onchange 表达式粘连成 handler 名，文本不落盘 |
| P3 (=U3) | 截图通道 | P1 | 集合页详情态 autoui_screenshot 必超时（服务端 ~10s 放弃；同刻 state/snapshot 0.0s 存活） |
| P4 (=U4) | select 渲染 | P1 | select 控件 vm 端整体缺位（快照结构在、渲染丢） |
| P5 (=U5) | table 样式 | P2 | thead 内置暗色底白字；th 的 style 属性被忽略 |
| P6 (=U9/快照竞态) | 快照通道 | P2 | autoui_snapshot 偶发空壳树（105B），空壳期实测可 >8s |
| P7 (=U7) | 表达式求值 | P1 | **loop 字段 Dot 表达式在部分 prop 链求值失败**（见下，本节最大清偿项根因） |

### P7 loop 字段 Dot 表达式的链路求值不一致（P1）

同一循环变量字段表达式（如 `m.nav_class`），不同消费链结果不同：

| 消费链 | 结果 |
|---|---|
| 条件位 `if g.open` / `if .store.active_id == m.id` | ✓ 正常 |
| text prop 直取 `input (value: e.value)` | ✓ 正常 |
| **button 的 `class:`/`style:` prop** | ✗ 静默失败 → convert_button 兜底 primary preset（紫块/白字/h-10），叠加子树折叠为多行 label 被 h-10 裁剪，仅首行 icon 可见 |
| **text/label 元素的 children 折叠**（`label (…) { text (text: e.label) {} }`） | ✗ content 求值失败 → **整个 label 缺位** |

**修复建议**：把 children 折叠链与 button class: 链统一走 props 直取链
（`extract_string_with`）同款求值；「求值失败→静默 preset 兜底/空内容」
改为 BuildProbe 显式告警。修复后可撤下游 workaround：
侧栏 nav 条件展开双态静态串 + label 全量 text-prop 化
（auto-os-config commit 1610c21）。

### P3 补充证据

复现：roles 集合页 Load → 选中 assistant 进详情态 → `autoui_screenshot`
60s 客户端超时下 **10.0s 返回**"iced thread may not be responding"（服务端
内部上限）；同详情态 `autoui_state`/`autoui_snapshot` 0.0s 响应——iced 主
线程存活，仅截图路径与 detail 富子树的协作阻塞。7 个非详情视图截图全部
1.5s 正常。446 批一 J1 收口（"渲染一直正常"）与本条不矛盾：这是截图通道
特有问题，非渲染构建问题。

### P6 补充证据

独立实例 boot（autoui_state 轮询就绪 + sleep 2s）后首次 snapshot 返回
105B 空壳树；立即重试（t=1s）有时直接非空；持续重试 8 次全空的实例亦
实测到。疑似 styled_vtree 落盘与快照读取的窗口竞态（446 批一 J1 收口
"快照回退源树"家族的残余形态）。下游 capture/e2e 已普遍加非空重试缓冲。

## Q. 批四实施记录（2026-08-28，auto-lang worktree plan-446-dev，已合入 master）

按 §I 第四批（打磨）实施：F3/F4/B3/D4/D5/E3 六项收口（G2-G5 计划内无
登记细节，见待澄清）。开工逐项探针实证：

**[✅ 已完成] D4 `.find(闭包)` 根修**（变体矩阵定位）：
- 根因一：`get_list_i32_elements` 对 `ListData<Value>` 有"非 Int 元素
  coerce 为 0"的 fallback——对象/字符串元素的 find/map/filter/any/all
  全 HOF 走 i32 快路径时谓词接到恒 0 永不命中（plan454 已在 obj.find
  剔除同款 tag 位误读，list 族本批补齐）。删 fallback，Value 列表一律走
  各 shim 的 Value 路径。
- 根因二：list.find miss 推 -1 哨兵——`hit == None` 恒假（v5 探针实证）。
  对齐 auto.obj.find 的 Plan 454 E 语言契约：miss = TAG_NULL。
验收：fn 模块/局部字面量/`[]any`/obj 注解/str 元素五形态全绿（d4 探针
+ d2 corpus handler 语境探针 + scratch 矩阵）。

**[✅ 已完成] D5 json 文档探测覆盖键列**：`json.keys` 返回
`ListData<i32>` 负索引编码（字符串池引用）——批三双态探测未覆盖。
探测扩展 ListData<i32>，序列化走专用解码（全负 → key 字符串数组，
含非负 → 真 int 数组；不进共享 vm_value_to_json 以免负索引被当负数）。
验收：`json.get_at(json.keys(x), 0)` 返回首键（d5 探针）。

**[✅ 已完成] E3 res.body() 文本化**：此前推 Vec<i32> 字节——print/str.*
消费面只见堆 id（现场"巨大数字串"，官方示例 02_http_client 即
`str.find(res.body(),…)` 文本消费）。改推 UTF-8 lossy 文本。验收：500
响应 body 断言 "boom!" 且无数字串泄漏（e3 探针，真实 TcpListener）。

**[✅ 已完成] B3 多参输入分发显式化**：输入文本只能作第一实参，多参
handler 其余实参无从取得——此前静默置换错位运行（os-config 被迫
"输入框一律单参"）。dynamic.rs on_with_input_for 对
handler_param_count > 1 的输入分发：stderr 显式诊断（含 handler 名/
参数数/建议）+ 丢弃该次分发。单参/无参/未知元数行为不变。

**[✅ 验证解除] F3 map 字面量陷阱**：空数组字面量（{members: []}）与
字面量内方法调用（{count: a.len()}）双探针在现 master 直接绿——多批
值求值工作已顺带治愈，探针入套件锁定。

**[✅ 已完成] F4 substr 语义文档统一**：实证 VM 实际语义 = (start, LEN)
（str.substr/.sub 双探针）；三处文档面矛盾（str.at 注释+示例错标
end-exclusive 且示例结果错、string-library.md 错标、conformance 已对）
——统一为 (start, LEN) 显式标注。docs_gen 4/4 过（Category C 门禁）。

**预折叠门禁**（Plan 466 新规）：cargo ta 全量 + ui-iced 全量于 worktree。
非 book 面 worktree 失败集 ⊆ master 失败集（零新增；benchmark_downcast
在案偶发）；book 面 13 个 worktree 独有失败经孤立/成组复跑 13/13 全绿
——负载偶发家族非确定性回归。ui-iced 3754/3756（唯二 = 在案
md_hidden_classes 基线红 + benchmark 偶发）。**门禁通过**。

**批四回归**：plan446_batch4 6/6 + 全量门禁上述。

## R. 复审记录（2026-08-28，/auto-plan:review）

复审人：ZCode 会话（批二/三/四执行者复核，按"验证不轻信"重跑）。

### 逐项验收判定（证据 = worktree 内复跑）

| 验收项 | 判定 | 证据 |
|---|---|---|
| A1 消歧报错（多 store 撞名/漏声明含 store+方法名） | PASS | plan446_a1_* 4 用例（handler_codegen 内）复跑绿；单 store 兼容保留 |
| A1 下游半（os-config 方法名改回 Init/Select） | 委托下游 | 计划 §I 明文指派 os-config e2e 双门禁交叉回归，非本仓可验 |
| B1 store 循环字段实参（MCP 模式触发+正确字符串） | PASS（headless 等价） | plan446_b1 双探针（60s 限时渲染+payload 分发）绿；MCP 实机归 os-config e2e（§I 指派） |
| B3 多参输入分发 | PASS | 行为改动已文档化（诊断+丢弃），下游"单参+Apply"绕行可复议 |
| C1 popover 可渲染 + 错误定位/非零退出 | PASS | 定位/致命化 = plan446_batch1 2 用例；**渲染半原未验，本复审实证绿并转常驻锁**（plan446_c1_popover_tests + corpus） |
| D1 json.parse 物化 + 双态文档 native | PASS | batch3 双探针 + json 全系 26/26 |
| D2/D3/D4 handler 值语义 | PASS | d2 corpus 探针 + d4 五形态矩阵（本批根修 i32 强转 + None 契约） |
| D5 get_at 收键列 | PASS | d5 探针绿 |
| D6 插入序 | PASS | d6 探针绿；全量零波及 |
| E1/E2 http natives | PASS | batch2 真实 TcpListener 双探针绿（e2 已解除 ignore） |
| E3 错误响应 body | PASS | e3 探针断言 "boom!" 无数字串 |
| F1 handler 崩溃诊断 | PASS | 批一落地（dynamic.rs 三处无条件 stderr + vm_bridge crash ip），代码在位 |
| F3 字面量陷阱 | PASS（验证解除） | 双探针绿 |
| F4 substr 语义文档 | PASS | 实证 (start,LEN)；str.at/string-library.md 修正，docs_gen 4/4 |
| G1 store 导入前缀 | PASS | test_g1（默认不变守卫+可配落行）绿 |
| G2-G5 | N/A | 计划内无章节无证据（悬空引用，见下） |

### 全量门禁（复审运行）

- ui-iced 全量：3762/3763（唯一失败 md_hidden_classes_parse = 在案基线红）。
- cargo ta：非 book/非 book_listing 面**零新增失败**（失败集 ⊆ master 基线，
  benchmark_downcast 在案偶发）；tt/tv 组无新失败。
- book_listing：**环境性不可用**——依赖未跟踪的本地 book/ 语料，并行会话
  正在重建（同一用例分钟级内 fail→pass 抖动；失败模式=文件缺失非输出
  错配；本计划足迹不含 book 面）。记为环境排除，非计划债。

### 遗漏 / 延后 / workaround 清单

1. **C1 渲染半曾未验**（批一只落诊断半）——本复审补验转绿并上常驻锁，已闭合。
2. **G2-G5 悬空引用**——总览/§I 提及但报告从未有章节（无症状/证据/复现）。
   非被丢弃的任务（无范围可丢）；建议 merge 时从 §I 切分行删除，或要求
   os-config 补录后再立项。已登记待澄清（可见非静默）。
3. **§P（U1-U7）新范围**——执行中途并行会话登记，未纳入 §I 切分；去向
   （独立立项/并入续批）留用户裁决。已登记待澄清。
4. **E1 类型坍缩正本清源**——显式延后（运行期+arity 双层兜底已落地），
   待澄清在册。
5. 代码内 workaround 均已文档化：JSON_DOC_ORIGINALS 全局暂存（沿用
   NativeInterface 覆盖块惯例）、E2 同步 drain（含勘误备案）、
   .worktrees/auto-down junction（环境注记 §O）。计划自有 diff 零
   TODO/FIXME 残留。
6. tmp-corpus/b3check 入库为批一有据决策（§L + j1 repro 测试注释互指）。

### spec-impact 元数据

`specs/modules/` 台账在本仓不存在（specs/ 为并行会话本地瞬态内容，git 零
跟踪）——`supersedes_spec_components` / `new_spec_components` /
`touched_goals` 三字段**留空**（规则：宁空勿猜）。本计划的可沉淀面已
全部在本文档 §K-§Q 实施记录中。

### 结论

全部验收项 PASS 或按计划 §I 明文委托下游，无阻塞债 → **status: reviewed**。
残留开放项（§P 去向、G2-G5 清理、E1 正本清源、下游 e2e 对账）均已在册
可见。就绪待 /auto-plan:merge。

- **G2-G5 无登记细节**：§0 总览与 §I 切分提及 G2-G5（vue codegen P2 族），
  但报告主体从未写对应章节（无证据/症状/复现）。除非下游补充现场，
  视为悬空引用不作实施——建议复审时从 §I 切分行删除或要求 os-config
  补录。
- **§P（Plan 010 增补 U1-U7）去向待定**：并行会话在批三/四之间登记的
  渲染层新报（P0 事件冻结/P7 loop 字段 Dot 求值链为主）。属新范围，
  未纳入本计划 §I 切分——建议独立立项或并入 446 续批，由用户裁决。
- **E1 附带观察（低优先）**：`res` 的静态类型在不同语境坍缩为
  str/int/User(Response)（http.at 表面声明 + `#[vm]` decl 推断链所致），
  E1/E2 已在运行期与 arity 分流双层兜住；类型推断本身的正本清源
  （stdlib 表面 client accessors 正名）留待独立小批，见 §N E1 修复注记。
- **D6 下游对账**：preserve_order 后 vm 轨字段顺序=插入序，os-config
  已登记的"字母序已知偏差"条目可撤；vue/vm 双端 UI 字段顺序 parity
  的实机验证由其 e2e 双门禁覆盖。
- **基线红登记**（非本计划范围，供交叉参照）：md_hidden_classes_parse
  批一在案 master 同红；benchmark_downcast 与 book_listing 家族为负载
  偶发（本批 13 个孤立复跑全绿）。
