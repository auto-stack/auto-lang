# PLAN-776 验证报告——HTTP 返回资源身份与同步/异步终值统一

> 阶段：T-08 门禁与对账｜plan_revision: 1
> 实施树：`D:/autostack/.wt/lang-776/auto-lang`（分支 `plan-776-dev`）
> 代码提交链（8 commits）：`a142756c9`（T-01 探针）→ `85eaddbe6`（T-02 资源与
> producer）→ `1c64d7bee`（T-03 编组统一）→ `0298f9c94`（T-04 值桥与终态）→
> `89560b46b`（T-05 存活收口）→ `8586a6409`（T-06 来源身份）→ `d2a51bed4`
> （T-07 矩阵+back_proxy 修复）→ `b4183e761`（格式化）。
> 依赖输入：auto-down 兄弟 worktree detached `c39ac7f`（组内 ../auto-down，
> 零跨仓改动）；基面 master `5e21561d2`。

## 1. 门禁执行记录（全部在实施树）

| 门禁 | 命令 | 结果 |
|---|---|---|
| 类型检查 | `cargo check -p auto-lang` / `-p auto-man` | 0 error |
| 日常全量 | 裸 `cargo t` | **5295/5295 绿**（89.3s；1 slow=plan738 预存长尾） |
| 语料档 | `cargo tv` | 162/162 绿 |
| HTTP e2e 完整 | `cargo th -j 1 --no-fail-fast` | **109/110 绿**；1 红=`back_proxy_real_routes_corpora_data_face` |
| auto-man 触面 | `cargo test -p auto-man --lib` | 350/357 绿，7 红全部与 master 基线等价（见 §3 分诊） |
| plan776 全族 | `cargo nextest run -p auto-lang --lib --features test-http-e2e plan776 --jobs 1` | 25/25 绿 |
| scoped | plan755 4/4、plan738 75/75、plan734 7/7、plan705 19/19、plan442 17/17 | 全绿 |
| 格式 | `rustfmt --check`（本计划触面 18 文件，格式化后） | 干净（全仓预存漂移不在本计划面） |

transpiler 零 diff（触面清单无 `trans/` 文件）——`cargo tt` 未触发（条件门禁不适用）。
aavm 零触发（`packages/aavm/**`、`test/vm/aavm2/**`、`test/parity/**` 无 diff）——
`cargo taa` 不适用。

## 2. AC 逐条对账（R6 终形，2026-10-14；基线 `088e47823` + R5 `5bdca42e4` + R6 修复轮）

> 本表在 R5 终形表基础上按 R6 修复轮（F776-R6-01..04）改写；plan776 全族
> 现 **56/56**（54 项 R1-R5 复现 + R6 新增：S7/S8+496 负控制 wire 两测 +
> T4/T5/环单测两测——见 R6 修复轮节明细）。

| AC | 证据（R5 终形） | 判定 |
|---|---|---|
| AC-01 真碰撞 | e2e 碰撞族 + 声明门/查表臂退役双反例（`declared_stream_plain_int_is_data`、`real_generator`）+ **R5 负控制 S4**（`let s = events(); return 0` → JSON 0——生产点包装不授予数值身份） | **过** |
| AC-02 逐值身份与兼容 | 传播探针 + 5 producer + setter owned-slot（9000 步跨窗）+ client egress 凭据 + bytes 快照 + **R5：SSE 身份跨全部传播边界**（直接/局部/helper/闭包捕获/容器取出/Internal Future 尾六形态同 wire）+ **R6：身份授予不依赖声明**（S7 旧 `int` 声明局部返回、S8 `int` helper 存 generator 再返回——同 SSE data 帧，Plan §0 旧 int 源码兼容）；for-in 迭代消费凭据解包兼容；6 legacy=plan326 34/34 | **过** |
| AC-03 sync/async 同形 | 单入口编组 `resource_claim_reply` 同形依据 + 代表例（async 资源终值 302/i64 全宽/record/SSE）；三 Future 形态；closure 面 plan442 17/17；**R5：stream 声明 fn 返回 Future 的立即体终值正确**（`return ~{ 42 }` → 42、`return ~{ events() }` → SSE——bp==0 直驱读位修复）；未逐格组合边界仍如实列（Upload/File async 终值未逐格） | **过** |
| AC-04 终态 | Failed→500、成功 Nil 200 null、stale/wrong-owner id 层拒绝、迟到/取消=plan705；R3 分诊负控制（P776-D3 归 705 轨）保持 | **过** |
| AC-05 错凭据 | owner 11 位段（2048 VM 受控拒绝+顺序 2047 中间 VM+live-A 交叉）、id 层校验先行、local 2^20 耗尽、**Future 0xF0 位模式普通数值守卫（R6-03 终形：F0/F1 双测——`~Iter` 或 `int` 声明 fn 构造活跃 Future 后返回字面量 496（=活跃 fid1 位模式）恒 JSON 496，门=return ~{} 血缘非声明/位模式）** | **过** |
| AC-06 存活 | 1000 闭环回基线、常量重放、SSE 断连回收、setter 跨窗、容器级联（Box{response} 异常回基线）；729/730/707 scoped 绿 | **过** |
| AC-07 来源身份 | **R5：限定名先于用户名集 + alias 目标链**；**R6：目标保结构**（`type D = Future<FileResponse>` → File，与直接 `Future<FileResponse>` 同规则——提取器 alias_of 携完整形态）+ **visited 集循环拒绝**（九层合法链 A8→…→A0→FileResponse → File，真环保守 Json——R5 深度 8 截断撤除）；单测 `plan776_r6_future_wrapped_and_deep_alias_chains`/`alias_cycle_visited_reject` + R5 双测保持 + back_proxy 同语义；755 两守卫、738 75/75、751 三态 | **过** |
| AC-08 生成/拓扑面 | **现行支持面四面真 TCP oracle 全绿（R6-04）**：plan729 生成 File wire 矩阵 + plan730 生成 Upload 矩阵（R5，hyper-util harness 修复）+ **plan776 生成 JSON record**（`http_e2e_plan776_generated_json_and_bus_sse_wire`：scaffold CRUD create——现行可编译运行的生成 JSON 形态，POST → `{"id":3,"text":"world"}` 真 wire）+ **plan776 生成 bus SSE**（events broadcast：订阅连接 → POST → NewMessage 帧真 TCP 到达）；api_gen 46+1、merged/IPC/back-proxy 矩阵、不泄漏裸 id。**边界如实列**：真实业务体 JSON 委托当前被 a2r 阻塞——db.at 跨模块引用 api 类型（`use api: X` 的 record ctor）独立转译报 undefined variable（点式同红；局部类型可转译但与 types.rs 撞型 E0308）——a2r 转译面（742 族）非 776 返回边界，证据矩阵见 R6 修复轮节 | **partial→四面已证**（真实体委托面归 742 族；终判归 R7） |
| AC-09 门禁对账 | §1/§3 + R5 修复轮门禁：裸 t **5306/5306 全绿**、tv 162/162、th 127/128（唯一红=corpora 在册）、plan776 54/54、auto-man lib 356+（3 红=P751-D2 在册预存）、rustfmt 触面干净、**776 diff 新增 warnings 逐条关闭**（死代码侧道/unused_doc_comments/失效注释清理）——终判归 R6 | 待 R6 |

## 3. 红项分诊（全部在册/预存，零本 diff 新增）

| 红项 | 归因 |
|---|---|
| `back_proxy_real_routes_corpora_data_face`（th 1/110） | **预存基线红**：master `5e21561d2` 同命令同红实证（参数绑定 `:id` 未替换 → 400，与返回编组无关的面）；755 收据"1 红（仅 back_proxy 在册基线红）"同项 |
| auto-man 7 红 | master 基线 8 红等价：P751-D2 登记预存 3（plan593 CSS 金样/shell_pack 新鲜度/merged CRUD fallback）+ 742 在途业务体形态 1（handler_body_module_calls——742 executing 不替验收）+ review738_r5/plan609 环境态 2 + desktop_extra_app_roots 族两树同形 1（master 与 worktree 隔离跑均 1 红 1 绿，同族环境态互换） |
| 全量 e2e 并行扫时偶发 sse_chain/plan724/031（单跑绿） | 已知固定端口+进程全局 AUTO_HTTP_PORT harness race（测试注释在案）；`th -j 1` 串行形态全绿 |

## 4. 已知边界与债务候选

1. **handler 任务栈残留对象的 RC**（T-05 修复仅覆盖返回值槽）：`remove_task`
   不清栈槽 rc——handler 局部堆对象在任务结束时残留引用（VM 既有模式，非本
   计划引入）。凭据面（最直接持有路径）已修；通用面属 RC 系统改造，超出
   §1 非目标（不动整体 RC）。候选登记 KNOWN-DEBT。
2. **点链 `http.response.redirect` CALL_SPEC 形态**：主线预存静默 None
   （plan352 在案）；真实 wire 契约是下划线 `http.response_redirect`。
3. **redirect 状态规范化**：shim 只产 301/302（307 输入归 302）——Plan 346
   既有行为保持，非本计划变更。
4. **HTTP_RESPONSES 表**：仅剩 client 死兼容面（int 查表；`push_handle_result`
   凭据化后 insert 退役，R2-05）；表仍 thread_local 无回收——client 面生命周期
   归属 FFI 资源范围（§1.4 非目标），server 构造路径已全部资源化不经表。
5. **缓存 Internal Future 的 `.await` 组合（R3 分诊，F776-R3-03）**：
   全局缓存 `~{...}` future 再 `.await`（含跨 handler 重放）触发 705 域
   sync-await 预存缺陷——`execute_future_body` 复用 handler 帧（只存取 ip
   不保 bp/sp），body RET 弹掉 handler 帧后 handler 自身 RET 落在 bp==0 →
   Terminated 有界 500；结果槽随帧错位，future 被写为 Ready(Int(0))
   （重放恒 0）。**归属 705 状态机**（Plan §5.2「Future 的展开/等待由
   705 状态机负责」）：数据体 `~{42}` 同形失败（无资源参与）、776 diff
   零触该机制（engine 侧仅加 rc_release，不在数据体失败路径）、失败在
   返回编组之前。776 边界三不变量（有界 500/重放数据面/凭据回收）由
   `http_e2e_plan776_r3_cached_future_await_triage` 钉住；KNOWN-DEBT
   **P776-D3** 归 705 轨。附带观察：sync handler `return ~{...}` 直接返回
   future 位模式数值走 JSON 数据面（如 1264）——sync 声明下数值即数据，
   与「声明不授予身份」一致，非泄漏。

## 5. 交付物清单

- 代码：`vm/ffi/http_return_resource.rs`（新，凭据类型+解析+计数钩子）、
  `TypeTag::HttpReturnResource`、producer 5 族切换、marshal/final_value_reply
  资源臂与 SSE 单源、`value_to_nv_via_task` 完整值桥、dispatch 双点 rc_release、
  来源身份三层（contract/extractor/codegen/back_proxy）+ 消费面切换
  （targets/api_gen）、测试 25 项（10 单测+探针、6 e2e T-03/T-04、2 e2e T-05、
  4 e2e/单测 T-06 等）。
- 报告：本报告 + `776-return-resource-spec-delta.md`（SD 草稿）+
  `776-return-resource-decision.md`（T-01 决策包）。
- **R3 修复轮新增**：分诊负控制测试 1 项
  （`http_e2e_plan776_r3_cached_future_await_triage`，提交 `73d4f0ff3`）；
  spec-delta R2 终形重写稿；decision §2/§3 R2 终形修订；本报告 §2 AC 表
  终形重写。

---

## R1 修复轮（2026-10-14，review-r1 needs_fix 后复修）

代码提交：`6494092de`（前链 a142756c9..b4183e761）。逐项闭合 F776-R1-01..09，
对账表见 Plan 文件"R1 修复轮交接"。门禁重跑：

| 门禁 | 结果 |
|---|---|
| 裸 `cargo t` | **5300/5300 全绿**（72.8s） |
| `cargo tv` | 162/162 |
| `cargo th -j 1 --no-fail-fast` | **117/118**；唯一红=back_proxy corpora（master 预存在册） |
| plan776 全族（含 R1 复现） | **38/38** |
| plan729/730/SSE/back_proxy 族 | 全绿（file-501 诊断门恢复；sse_chain 4/4；upload 40/40） |
| 生成真 TCP（auto-man test-http-e2e） | **预存编译漂移**：`http_e2e_plan729_generated_service_wire_matrix` 生成物 E0433+hyper Builder 差异编译失败——master 基线同形（TIMEOUT 120s / 同编译失败），742 附件同类在案；非本 diff 引入，归档待 742 轨收口 |

修复要点（相对 R1 审查基线的合同变化）：
- 普通数值经表取得资源的全部路径退役（iterator 命中制→声明门；Response 查表
  删除；upload/file take 门→纯诊断门）——P734-D7 查表猜测正式替代。
- 凭据 id 编入 owner 段（专用段 0x8000_0000|owner<<24|local），跨 VM 同号
  不可能；id 层校验先行。
- Internal future body/异常退出/取消的凭据终态清偿（body 栈槽释放 + remove_task
  扫栈）。
- 已知边界更新：owner 段 7 位（进程内 128 VM 后回绕——防御与通用段 2^48 回绕
  同级）；client 句柄返回=JSON 数据（查表退役后的行为变化，无 wire 用例依赖）。

---

## R2 修复轮（2026-10-14，review-r2 needs_fix 后复修）

代码提交：`088e47823`。逐项闭合 F776-R2-01..08，§5 诊断表与修复对账见 Plan
文件"R2 修复轮交接"。门禁重跑：

| 门禁 | 结果 |
|---|---|
| 裸 `cargo t` | **5304/5304 全绿**（91.7s；一次中间跑的 plan484/plan716 红单跑绿=日常间歇 flake） |
| `cargo tv` | 162/162 |
| `cargo th -j 1 --no-fail-fast` | **123/124**；唯一红=back_proxy corpora（master 预存在册） |
| plan776 全族（R1+R2 复现） | **48/48** |
| SSE 族（含 indirect generator） | 4/4（generator 身份在返回边界建立后恢复） |
| plan729/730/705/734/755 scoped | 全绿（plan705 调用图门扩认 `_http` 变体） |
| auto-man | api_gen 46+1 ignored；plan776 消费 1/1 |
| rustfmt --check 全触面（含 mod.rs） | 干净（mod.rs 注释改独立行解 rustfmt 行尾注释循环合并） |

修复要点（相对 R2 审查基线的合同收敛）：
- **iterator 身份只在返回边界建立**（R2-01 终形）：generator 生产点两个——
  `call_fn_by_name_http`（handler-是-generator 的 dispatch 臂）产 Sse 凭据；
  codegen 对 `~Iter/~Stream` handler 的 `return <generator fn 调用>` 静态插
  `WRAP_SSE_RESOURCE`(0xD5)。marshal int 臂不再授予 iterator 身份——声明/
  表命中/数值全部不构成来源（R2 反例 `fn p() ~Iter { return 0 }` → JSON 0）。
- owner 段 11 位无掩码（R2-02）：第 2049 个 VM 受控拒绝；id 层校验先行。
- setter 族（5 个含 bytes）owned-slot 协议（R2-03/06）：实参份额消费释放、
  返回槽 rc_push 影子重建——9000 步跨窗 wire oracle 锁 UAF 修复。
- 任务终态清偿扩到全部堆对象槽（R2-04）：容器父份额释放→dying 级联子
  stakes；逃逸/共享经宽限窗复活保护。
- client Response 进 egress 边界（R2-05）：`push_handle_result` 凭据化，
  上游 status/body 保真转发。
- alias 按 target 解析 + `http.` 限定名（R2-07）：`type Download =
  FileResponse` → File；alias→用户名 → Json；back_proxy 双集（直接声明 +
  alias→target 表）。
- 未验证面如实记录：生成真 TCP 矩阵=预存编译漂移（master 同形）；closure
  面 generator 分派无现实用例未包装（待察）。

---

## R3 修复轮（2026-10-14，review-r3 needs_fix 后复修）

代码提交：`73d4f0ff3`（仅新增分诊负控制测试，+101 行；实现链
a142756c9..088e47823 不动）。逐项闭合 F776-R3-01..03：

- **F776-R3-01（P1）**：spec-delta 全文重写为 **R2 终形稿**（SD-01..07
  after 规则按 `088e47823`：返回边界身份、owner 11 位段耗尽、setter
  owned-slot、容器级联清理、client egress 凭据、alias target 展开、bytes
  快照；R1 修订节废止——修订史与 r1/r2/r3 冻结副本链保留）；decision
  §2/§3 废止表述（「无显式 owner id / client 保持 int / 声明门保持 /
  查表保留 / setter 走表臂」）就地修订并标注 R2 终形（§2 总注、§3.1
  owner/代次、§3.3 client、§3.5 setter、§4 分派序）。
- **F776-R3-02（P2）**：本报告 §2 AC 表按 R2 终形重写——AC-01 iterator
  对照改双反例（查表臂退役）；AC-03 写明 `resource_claim_reply` 单入口
  编组同形依据+未逐格组合边界清单；AC-08 改 **partial**（生成真 TCP
  未验证面单列）；AC-09 改待 R4。
- **F776-R3-03（P2）**：future-cache 分诊完成——机制（`execute_future_body`
  帧复用 → body RET 弹 handler 帧 → handler RET bp==0 → Terminated 有界
  500；结果槽错位 → future 被写 Ready(Int(0))，重放恒 0）+ 三重归属证据
  （数据体 `~{42}` 同形失败=无资源参与；776 diff 零触该机制；失败在返回
  编组之前）→ **归属 705 状态机**（Plan §5.2；KNOWN-DEBT P776-D3）；
  负控制 `http_e2e_plan776_r3_cached_future_await_triage` 钉 776 边界三
  不变量（首回有界 500 / 重放普通 int 数据面 / 半驱动凭据经任务终态清偿
  回基线——R2-04 栈扫描的错误出口新路径）。

门禁重跑（实施树 `73d4f0ff3`）：

| 门禁 | 结果 |
|---|---|
| plan776 全族（test-http-e2e，`-j 1`） | **49/49**（48 项 R1+R2 复现 + R3 分诊 1 项） |
| `cargo check -p auto-lang` | 0 error（预存 warnings 不变） |
| 裸 `cargo t --no-fail-fast` | **5303/5304**；唯一红=plan484_024_charts（在册间歇 flake，**单跑 1/1 绿**——R2 轮同类实录） |
| `cargo th -j 1 --no-fail-fast` | **123/125**；红 2=back_proxy corpora（master 预存在册）+ e2e_sse_generator_handler（plan326 SSE 族已知偶发 harness race，T-07 在案，**单跑 1/1 绿**） |
| rustfmt --check（触面测试文件） | 干净 |

本轮实现 diff 仅测试文件（文档改动在主检出 docs/plans/**）；tt/taa/tu
零触发。cargo fmt 全仓因 examples/ui_container.rs 预存解析错误不可用
（仓库惯例=触面逐文件 rustfmt）。

---

## R5 修复轮（2026-10-14，review-r5 needs_fix 后复修）

代码提交：`5bdca42e4`（9 文件，+413/−137；实现链 a142756c9..088e47823 +
73d4f0ff3 保留）。逐项闭合 F776-R5-01..04：

- **F776-R5-01（P1）SSE 身份生产点凭据化**：codegen 对 `~Iter/~Stream`
  声明 fn 内 generator 调用（CREATE_GENERATOR 后）静态插
  `WRAP_SSE_RESOURCE`（幂等：已是凭据对象原样回推）；R2 的 return 位
  包装移除（被生产点包装完全覆盖）。凭据随 local/helper/容器/闭包捕获/
  Internal Future 尾**全载体**天然传播（VmRef/TAG_OBJECT 双桥）；非 HTTP
  消费兼容：`auto.iterator.next` 凭据感知解包。stream 声明 fn 可返回
  Future（marshal future 门扩「async 或 stream」）；dispatch 直驱立即体
  的 bp==0 读位修复（原 `sp > bp+1` 条件漏读 base 形态 → 终值恒 Int(0)；
  `fn p() ~int { return ~{ 42 } }` 一并修复 → 42——预存缺陷顺带清偿）。
  oracle：`http_e2e_r5` wire 三测（S0-S6 传播/S4 负控制+S42 立即体/
  iterstream 内部迭代解包）。
- **F776-R5-02（P2）限定名与别名链**：`from_return_identity` 重写——
  `http.X` 限定名**先于**用户名集判定（用户同名不遮蔽）；alias 目标
  **链**递归展开（每层限定目标先判 stdlib；链尾用户声明→Json；深度 8
  护栏循环保守 Json）。back_proxy fn_meta 同语义（初始限定名+链上限定
  目标均绕过用户名集）。oracle：提取级双测（V5-T1/T2/T3 + 正控制）。
- **F776-R5-03（P2）生成真 TCP oracle**：R1 起在册的「生成物编译漂移归
  742」实证为 **e2e harness 自写 Cargo.toml 缺 `hyper-util` 依赖**（生成
  器合法产 `hyper_util::…`，生产模板 api_gen.rs:1265 一直声明；两处
  harness 补齐后）——plan729 生成 File wire 矩阵（真实 cargo build +
  真 TCP：GET/HEAD/range 206/416/etag 304/412/404/403/12MiB hash/async
  文件）与 plan730 生成 Upload 矩阵**全绿**。生成 JSON record handler
  生成级验证（签名正确发射）；其 wire 面需 route-A/B 业务体（无 db
  fixture 下 handler 桩 404）——归 742 业务体轨（R5 明言不要求 776
  接管）。
- **F776-R5-04（P2）健康清理**：死代码侧道全链移除（API_GENERATOR_FNS/
  record_api_generator_fn/fn_is_generator/handler_is_generator/
  declared_return/expr_is_generator_call——`handler_declares_stream_return`
  复活为 stream future 门）；unused_doc_comments 关闭；资源模块头/
  marshal int 臂失效注释对齐终形。776 diff 触面新增 warnings 归零
  （cargo check 0 error，无 776 符号 dead_code/doc 告警）。

门禁（实施树 `5bdca42e4`）：

| 门禁 | 结果 |
|---|---|
| 裸 `cargo t --no-fail-fast` | **5306/5306 全绿**（116.8s；含 R5 新增 5 测试） |
| `cargo tv` | 162/162 |
| `cargo th -j 1 --no-fail-fast` | **127/128**；唯一红=back_proxy corpora（在册预存，R1..R5 同形） |
| plan776 全族（test-http-e2e，`-j 1`） | **54/54**（R1-R3 复现 49 + R5 新增 5；一次 e2e 偶发 30s 超时单跑/复跑全绿——在册 harness race 族） |
| 生成真 TCP | plan729 File 矩阵 1/1（30.8s）+ plan730 Upload 矩阵 1/1（14.7s） |
| `cargo test -p auto-man --lib`（test-http-e2e，串行） | 356 passed/3 红=P751-D2 在册预存（plan593 CSS 金样/shell_pack 新鲜度/merged CRUD fallback——master 等价）；api_gen 家族 46+1 |
| `cargo check -p auto-lang` | 0 error；776 触面新增 warnings 0 |
| scoped | generator 10/10、plan442 17/17、plan734 7/7、plan705 19/19、SSE 族 4/4 |
| rustfmt 触面 9 文件 | 干净 |

tt/tu/taa 零触发（无 trans/ui_gen/aavm 面 diff）。

---

## R6 修复轮（2026-10-14，review-r6 needs_fix 后复修）

代码提交：`5fda97ceb`（7 文件 +585/−97；实现链 a142756c9..088e47823 + 73d4f0ff3 +
5bdca42e4 保留）。逐项闭合 F776-R6-01..04：

- **F776-R6-01（P1）身份授予不依赖声明**：CREATE_GENERATOR 后包装**无条件**
  化（撤 R5 的 `fn_ret_declares_stream` 门）——旧 `int` 声明 handler 局部
  返回（S7）与 `int` helper 存 generator 再返回（S8）同保真实 SSE 来源
  （Plan §0 旧 int 源码兼容）；非 HTTP 消费由凭据感知 iterator natives
  吸收（scoped generator/tv/SSE/plan705/707/442/326 全绿）。
- **F776-R6-03（P1）future 门=血缘登记**：codegen 按 fn body 的
  `return ~{}` 返回位登记 API_ASYNC_RETURNS（声明无关），撤 R5 的
  「声明 stream 即进门」——`let f = ~{…}; return 496` 的普通数字（位模式
  撞活跃 fid）恒 JSON 496（F0=int 声明/F1=~Iter 声明双负控制）；S3
  （`return ~{ events() }`）经血缘进门保持 SSE。R5 门扩展引入的回归清偿。
- **F776-R6-02（P2）alias 目标保结构 + visited 集链**：提取器 alias_of
  携 target **完整形态**（type_to_string——`type D = Future<FileResponse>`
  实参不丢，分类与直接 `Future<FileResponse>` 同 File）；contract/
  back_proxy 链展开改 **visited 集循环拒绝**（九层合法链 → File；真环 →
  Json；R5 深度 8 截断撤除）。
- **F776-R6-04（P2）生成 JSON/bus SSE 真 TCP oracle**：
  `http_e2e_plan776_generated_json_and_bus_sse_wire`（plan776_gen_e2e
  harness，按生产管线组 db.rs 转译 + State<Db> CRUD scaffold + events
  broadcast bus）——POST /api/send → `{"id":3,"text":"world"}` 真 wire；
  订阅连接收到 NewMessage SSE 帧（真 TCP）。**真实体 JSON 委托的 a2r
  阻塞证据矩阵**：db.at 跨模块引用 api 类型——`use api: Message` /
  `use api.Message` 的 record ctor 独立转译均报 undefined variable；
  局部类型可转译但 db::Message 与 types::Message 撞型（E0277 无 serde
  derive / E0308 类型不符）——a2r 转译面（742 族）非 776 返回边界。

门禁（实施树 `5fda97ceb`）：

| 门禁 | 结果 |
|---|---|
| `cargo check -p auto-lang` | 0 error |
| 裸 `cargo t --no-fail-fast` | **5308/5308 全绿**（119.0s；含 R6 新增测试） |
| `cargo tv` | 162/162 |
| `cargo th -j 1 --no-fail-fast` | **129/130**；唯一红=back_proxy corpora（在册预存，R1..R6 同形） |
| plan776 全族（test-http-e2e，`-j 1`） | **58/58**（54 项 R1-R5 复现 + R6 新增 4；后台跑一次 e2e 偶发复跑全绿——在册 harness race 族） |
| 生成真 TCP 矩阵 | **3/3**：plan729 File（16.7s）+ plan730 Upload（14.1s）+ **plan776 JSON/bus SSE（13.8s）** |
| `cargo test -p auto-man --lib api_gen` | 47 passed, 1 ignored |
| scoped | generator 10/10、plan705 19/19、plan707 31/31、plan442 17/17、plan326 40/40、SSE 族 4/4 |
| rustfmt 触面 7 文件 | 干净 |

tt/tu/taa 零触发。死代码清理：`fn_ret_declares_stream`（codegen）、
`handler_declares_stream_return`（http_server——血缘门取代声明门后死亡）。

---

## R7 修复轮（2026-10-14，review-r7 needs_fix 后复修）

代码提交：`5d4d29ab0`（10 文件 +531/−225；实现链 a142756c9..5fda97ceb 保留）。
逐项闭合 F776-R7-01..04：

- **F776-R7-01（P1）Future 句柄 TAG_FUTURE 逐值载荷**：nano_value 新增
  tag 0xB（encode/is/decode_future）——全部生产点（CREATE_FUTURE +
  stdlib External creator ×4）产 `encode_future(fid)`；marshal 对
  TAG_FUTURE 值**无门驱动**（drive_future_reply 共享助手）。函数级登记
  两代（R5 声明门 / R6 血缘登记）全部退役。oracle：F0/F1（496 恒数据，
  含未执行 `return ~{}` 分支）+ F2（`future_helper()` 真实 Future → 42）。
  值经 `Value::Int` 载体往返（容器元素/闭包捕获）时标签降级为 legacy
  `(fid<<8)|0xF0` 位形态（decode_tagged_nv / native+interop nv_to_value /
  CREATE_ARRAY 三处映射）——AWAIT/POLL/SPAWN_GO/`future_all`-`race` 列表
  扫描经兼容臂保持功能（plan394 17/17）。
- **F776-R7-02（P1）iterator natives 凭据感知全族**：共享
  `pop_iterator_receiver`（TAG_OBJECT Sse 解包 + 裸 i32，**弹出槽清零**
  对齐 pop_arg_nv 纪律——残留 TAG_OBJECT 被 R2-04 任务栈扫描误读 → 双
  释放 → 底层 list 回收；**031 names 空列表回归由 th 揪出**，定位=collect
  pop 未清零，修复后 031 单跑/全档绿）适配 map/filter/enumerate/collect/
  reduce/find；**派生 iterator 身份**：凭据源派生 id 包装凭据
  （push_derived_iterator）。oracle：`plan776_r7_iterator_credential_native_
  family`（同一底层表项裸 id/凭据双载荷 enumerate 均有效 + 派生凭据经
  next 可迭代——镜 R7 探针 native 模式）。
- **F776-R7-03（P2）S3 有头无帧分诊**：判别实验复现并定性——rapid SSE
  **早断连 churn**（首帧即弃连 ×10）下 ~33% 触发**服务级死锁**（新连接
  accept 后零响应、普通路由探测 3s 无响应——观测形态两相：零字节或
  200+chunked 头后零 chunk）；**base `5e21561d2`（776 之前，老查表
  SSE）同款实验 2/6 复现同形**（判别 worktree：同端点 ×10 早断连）——
  **预存缺陷**，归属 SSE 服务/断连清理/调度域（plan707 轨），登记
  KNOWN-DEBT **P776-D4**。776 侧硬化：S3 oracle 改 **15s 有界**+失败时
  附探测诊断（区分服务级死锁 vs 泵死）；marshal future 驱动的 task
  try_lock 有界重试（100×2ms，防静默跳过驱动）。
- **F776-R7-04（P3）**：`bare_type_name` 死代码移除；无断言
  `plan776_dbg_db_transpile` 探针测试移除（证据在本文档 R6 节）；
  本轮新增触面零 dead_code/doc 告警。

门禁（实施树 `5d4d29ab0`）：

| 门禁 | 结果 |
|---|---|
| `cargo check -p auto-lang` | 0 error；776 触面新增 warnings 0 |
| 裸 `cargo t --no-fail-fast` | **5309/5309 全绿**（111.7s；一次 state_file concurrent_atomic 并行 flake 单跑绿——非本触面） |
| `cargo tv` | 162/162 |
| `cargo th -j 1 --no-fail-fast` | **131/132**；唯一红=back_proxy corpora（在册预存）；031 修复后全档绿 |
| plan776 全族（`-j 1`） | **61/61**（58 项 R1-R6 复现 + R7 新增 3：F0/F1/F2 wire、iterator 族单测、S3 有界 oracle；一次族跑命中预存 P776-D4 楔子单测红——复跑全绿，在册预存 flake 形态） |
| 生成真 TCP 矩阵 | **3/3**（File/Upload/JSON+busSSE） |
| scoped | plan394 17/17、plan705 19/19、plan707 31/31、generator 10/10、plan326 40/40 |
| rustfmt 触面 10 文件 | 干净 |

tt/tu/taa 零触发。判别实验的临时 base worktree（lang-776-base 组）已
清理（实验数据保留于本节文字）。

---

## R8 修复轮（2026-10-14，review-r8 needs_fix 后复修）

代码提交：`5b8bffdef`（7 文件 +246/−68；实现链保留）。逐项闭合：

- **F776-R8-01（P1）Value 载体保真 + legacy 臂退役**：TAG_FUTURE 句柄
  入 Value（四桥：decode_tagged_nv / native+interop nv_to_value /
  CREATE_ARRAY）保真映射**既有** `Value::Future(FutureData{id})`（句柄
  视图，state 占位 Pending；Plan 124 变体——非新表示）；还原路径
  （materialize_value / value_to_nv_via_task / **GET_ELEM**——原落 `_`
  臂 push_i32(0)）回 `encode_future(fid)`。**marshal legacy 位形态驱动
  臂整体退役**：`~int` 声明 fn 返回普通 496（位模式撞活跃 fid1）恒
  数据（V8-F2）；数组取出真实 Future → 42（V8-F3）；R7 形态（F0/F1/
  helper）保持。`future_all/race` 扫描接受 Value::Future 元素。
- **F776-R8-02（P1)iterator 消费所有权协议**：`pop_iterator_receiver`
  与 `shim_iterator_next` 凭据臂对**原始凭据 nv** 执行消费协议——
  StakeGuard::nv（保护读取）+ `rc_release`（清偿调用方栈份额；R7-02
  的 guard-解包-id 模式无法清偿高段 credential 的 owned 份额——R8
  探针实证源凭据消费后 RC=1 恒存活）+ 弹出槽清零（R7 交接声明确未
  落地于冻结提交，本轮一并修正）；错误早退同清偿；派生输出自带份额。
  oracle：`plan776_r8_iterator_consumer_ownership_protocol`（×3 重复
  消费净零——dying 宽限窗语义入账、早退非凭据对象、裸源控制）。

门禁（实施树 `5b8bffdef`）：

| 门禁 | 结果 |
|---|---|
| `cargo check -p auto-lang` | 0 error |
| 裸 `cargo t --no-fail-fast` | **5310/5310 全绿**（81.0s；一次 plan707 cancel-gap 并行 flake 单跑绿——Value 桥触面广故本轮仍跑整档；后置裁定下最终档归 merge） |
| `cargo tv` | 162/162 |
| plan776 全族（`-j 1`） | **63/63**（61 项 R1-R7 复现 + R8 新增 2：载体身份 wire、所有权协议单测） |
| 生成真 TCP 矩阵 | 3/3（File/Upload/JSON+busSSE） |
| scoped | plan394 17/17、plan705 19/19、plan707 31/31、generator 10/10、plan326 40/40 |
| rustfmt 触面 7 文件 | 干净 |

---

## R9 修复轮（2026-10-14，review-r9 needs_fix 后复修）

代码提交：`dd41d684d`（3 文件 +183/−11；实现链保留）。逐项闭合：

- **F776-R9-01（P1）消费恰一份**：StakeGuard::nv 只记 id（无 retain）、
  其 Drop 已执行 rc_release_id——R8 在 guard 之外叠加的显式
  `vm.rc_release` 是第二份释放（RC1 独占时第二次落在零计数未暴露；
  RC2 参数+全局共享时 2→0 销毁全局份额——V9-N1）。修复：撤两处显式
  release（pop_iterator_receiver / shim_iterator_next 凭据臂），guard
  Drop 即那一次消费；读取期间份额未动；早退同样恰一份。oracle：
  `plan776_r9_iterator_consumer_single_release_shared`——RC1 独占
  reap 回基线 + **RC2 共享**（消费后 2→1、跨 reap 存活、全局凭据仍
  可解析）。
- **F776-R9-02（P1）typed record Future 字段保真**：插桩实证真实路径
  ——`Box { f: ~{42} }` 走 **NEW_INSTANCE/CONSTRUCT_INSTANCE**（非
  CREATE_OBJ），其逐字段 tag 转换把 TAG_FUTURE 落 `Value::Int(fid)`
  （观测 body=1 的来源）；读取走 **GET_GENERIC_FIELD → push_value**
  （非 GET_FIELD），push_value 兜臂压 0。补 Future 臂：CONSTRUCT_
  INSTANCE 字段转换（保真入）、push_value（还原出）；CREATE_OBJ/
  GET_FIELD×2 臂保留（各自路径覆盖）。桥清点更新：nv↔Value 全入口
  =decode_tagged_nv/native+interop nv_to_value/CREATE_ARRAY/CREATE_OBJ/
  **CONSTRUCT_INSTANCE**；全出口=materialize_value/value_to_nv_via_task/
  GET_ELEM/GET_FIELD×2/**GET_GENERIC_FIELD(push_value)**；捕获存
  decode_tagged_nv/捕获读原槽 nv。oracle：`http_e2e_plan776_r9_record_
  field_future_identity`（record 字段→42、数组对照、普通数字负控制）。

门禁（实施树 `dd41d684d`）：

| 门禁 | 结果 |
|---|---|
| `cargo check -p auto-lang` | 0 error |
| 裸 `cargo t --no-fail-fast` | **5311/5311 全绿**（114.7s） |
| `cargo tv` | 162/162 |
| plan776 全族（`-j 1`） | **65/65**（63 项 R1-R8 复现 + R9 新增 2） |
| 生成真 TCP 矩阵 | 3/3 |
| scoped | plan394 17/17、plan705 19/19、plan707 31/31、generator 10/10、plan326 40/40 |
| rustfmt 触面 | 干净 |

---

## R10 修复轮（2026-10-14，review-r10 needs_fix 后复修）

代码提交：`38ad38aba`（2 文件 +188/−5；实现链保留）。闭合：

- **F776-R10-01（P1）五 setter 恰一份消费**：status/header/text/html/
  bytes 五 setter 与 R8 iterator 同型的双释放（StakeGuard::nv 只记 id、
  Drop 已 rc_release_id；R2-03 叠加的显式 `vm.rc_release` 是第二份）。
  共享 RC2（参数 + globals）经 setter 2→1，释放 rc_push 重建的返回槽
  后 0——全局合法 Response 被回收，读取触发 RC canary UAF（V10-R1）。
  修复：五点批量撤显式 release（guard Drop 即恰一次；读取期间份额未动；
  错误早退同样恰一份）；返回槽 rc_push 重建——setter 后共享形态
  RC2=返回槽+全局。oracle
  `plan776_r10_setter_single_release_shared`：RC2 共享矩阵（setter→2、
  释放返回槽→1、跨 reap 全局快照可读=201——镜 V10-R1 形态）+ RC1 独占
  净零 + ×5 重复 setter 链（全局份额恒在、终值 304）+ 错误早退恰一份
  （非凭据 receiver→Err、外部份额保留）。

门禁（实施树 `38ad38aba`）：check 0 err；裸 `cargo t` **5312/5312**
（79.6s）；`cargo tv` 162/162；plan776 全族 **66/66**（65 项复现 + R10
新增 1）；plan394 17/17、plan705 19/19；rustfmt 触面干净。
