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

## 2. AC 逐条对账（R5 终形，2026-10-14；基线 `088e47823` + R5 修复 `5bdca42e4`）

> 本表在 R2 终形表基础上按 R5 修复轮（F776-R5-01..04）改写；plan776 全族
> 现 **54/54**（49 项 R1-R3 复现 + R5 新增 5 项：SSE 传播三 wire + 限定名/
> 别名链双提取级）。

| AC | 证据（R5 终形） | 判定 |
|---|---|---|
| AC-01 真碰撞 | e2e 碰撞族 + 声明门/查表臂退役双反例（`declared_stream_plain_int_is_data`、`real_generator`）+ **R5 负控制 S4**（`let s = events(); return 0` → JSON 0——生产点包装不授予数值身份） | **过** |
| AC-02 逐值身份与兼容 | 传播探针 + 5 producer + setter owned-slot（9000 步跨窗）+ client egress 凭据 + bytes 快照 + **R5：SSE 身份跨全部传播边界**（`http_e2e_plan776_r5_sse_identity_propagates`：直接/局部/helper/闭包捕获/容器取出/Internal Future 尾六形态同 wire SSE+data 帧）；for-in 迭代消费凭据解包兼容（`credential_unwrapped_for_internal_iteration`）；6 legacy=plan326 34/34 | **过** |
| AC-03 sync/async 同形 | 单入口编组 `resource_claim_reply` 同形依据 + 代表例（async 资源终值 302/i64 全宽/record/SSE）；三 Future 形态；closure 面 plan442 17/17；**R5：stream 声明 fn 返回 Future 的立即体终值正确**（`return ~{ 42 }` → 42、`return ~{ events() }` → SSE——bp==0 直驱读位修复）；未逐格组合边界仍如实列（Upload/File async 终值未逐格） | **过** |
| AC-04 终态 | Failed→500、成功 Nil 200 null、stale/wrong-owner id 层拒绝、迟到/取消=plan705；R3 分诊负控制（P776-D3 归 705 轨）保持 | **过** |
| AC-05 错凭据 | owner 11 位段（2048 VM 受控拒绝+顺序 2047 中间 VM+live-A 交叉）、id 层校验先行、local 2^20 耗尽、Future 0xF0 位模式守卫 | **过** |
| AC-06 存活 | 1000 闭环回基线、常量重放、SSE 断连回收、setter 跨窗、容器级联（Box{response} 异常回基线）；729/730/707 scoped 绿 | **过** |
| AC-07 来源身份 | **R5：限定名先于用户名集**（`http.FileResponse` + 用户同名声明 → File——V5-T1 复现闭合）；**alias 目标链**（`Download=http.FileResponse`→File、Second→First→FileResponse→File、链尾用户声明→Json、循环 alias 保守 Json——V5-T2/T3 闭合）；提取级双测 `plan776_r5_qualified_name_and_alias_chain_identity`/`alias_chain_and_cycle_rejected` + back_proxy 同语义（th 守卫族绿）；755 两守卫、738 双指纹 75/75、751 三态 | **过** |
| AC-08 生成/拓扑面 | **已证面（R5 扩）**：生成服务**真 TCP oracle 建立**——「预存编译漂移归 742」实为 e2e harness Cargo.toml 缺 `hyper-util` 依赖（生产模板一直在案），一行修复后 **plan729 生成 File wire 矩阵**（GET/HEAD/range 206/416/etag 304/412/404/403/12MiB hash/async 文件）与 **plan730 生成 Upload 矩阵**全绿（真实 cargo build + 真 TCP wire）；api_gen 46+1、plan776 消费族、merged/IPC/back-proxy 矩阵（file-501/upload-501 守卫）、不泄漏裸 id。**边界如实列**：生成 JSON record handler 的 wire 面需要 route-A/B 业务体（无 db fixture 下 handler 桩 404）——生成级已验证（`pub async fn meta(...) -> Result<JsonResponse<FileMeta>, StatusCode>` 正确发射），wire 面归 742 业务体轨（R5 明言不要求 776 接管）；生成 bus SSE 保持生成级测试在案 | **partial→已证主体**（终判归 R6） |
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
