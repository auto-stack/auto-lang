# Plan 446 — VM 渲染后端实战薄弱点（auto-os-config Plan 007 现场报告）

状态: drafting（上游修复提案，含复现载体与验收标准）
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
