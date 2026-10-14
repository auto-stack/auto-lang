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

## 2. AC 逐条对账（R2 终形重写，2026-10-14；基线 `088e47823` + 分诊测试 `73d4f0ff3`）

> 本表为 R3 修复轮（F776-R3-02）按 R2 实现终形改写的对账——T-08 初版表
> 与 R1/R2 实现失配处（AC-01 查表臂表述、AC-08 过判）已修正；plan776
> 全族现 **49/49**（48 项 R1+R2 反例复现 + 1 项 R3 分诊负控制）。

| AC | 证据（R2 终形） | 判定 |
|---|---|---|
| AC-01 真碰撞 | e2e 碰撞族：活跃资源实际 id → 同值普通 int wire 200 JSON 数字（`/api/plain/<id>`）→ 资源未被消费（`/api/replay` 仍 302）。iterator 族对照=**声明门/查表臂已退役**后的双反例：`declared_stream_plain_int_is_data`（`fn p() ~Iter { return 0 }` → JSON 0——声明不授予身份）+ `real_generator`/indirect（真实 generator → SSE；plan326 indirect 形态恢复 4/4） | **过** |
| AC-02 逐值身份与兼容 | 传播探针（globals/futures/captures）+ 5 producer 切换 + setter owned-slot（`setter_return_survives_grace_window` 9000 步跨窗 201）+ **client egress 凭据化**（`client_response_keeps_status_body`：本地上游 201+`upstream` body 保真转发，不再 JSON 裸编号）+ **bytes 快照**（`bytes_setter_roundtrip` body=AB）；6 legacy wire 用例=plan326 http_e2e 34/34（redirect/SSE 链/accessors/musk 常量内含，由真实生产点凭据保持） | **过** |
| AC-03 sync/async 同形 | **同形依据=单入口编组**：sync `marshal_handler_value` 与 async `final_value_reply`（`resolve_resource`/`resolve_resource_value`）汇于**同一** `resource_claim_reply` 分派（Response 快照 clone/Sse 校验+`sse_iterator_reply` 单源/File+Upload take 移交）——同形由构造保证，代表例验证：async 资源终值 302、i64 全宽 5000000000、record JSON（`"id": 7` 不降级）、SSE 不变 JSON；三 Future 形态（Ready 即完/External park/Internal body await——delay_async+park 驱动）；closure 面 plan442 axum Router 链 17/17。**未逐格展开的组合边界**（如实列）：四资源族 × 三 Future 形态 × named/closure 未做全笛卡尔逐格 wire——Upload/File 的 async 终值形态与 record 嵌套资源组合未逐格断言（单入口构造保证同形，代表例覆盖 Response/SSE/i64/record） | **过**（单入口构造 + 代表例） |
| AC-04 终态 | e2e Failed→500（`async_failure_reply`，不再 200 null）；成功 Nil 200 null 保持（plan705）；stale/wrong-owner id 层拒绝；迟到/取消=plan705 9/9 + 调用图门扩认。**R3 分诊（F776-R3-03）**：缓存 Internal Future 的 `.await` 组合（R2 future-cache 探针形态）= **705 域 sync-await 预存缺陷**（`execute_future_body` 帧复用 → body RET 弹 handler 帧 → handler RET bp==0 → Terminated；数据体 `~{42}` 同形失败、776 diff 零触该机制、失败在返回编组之前）——776 边界三不变量由负控制 `http_e2e_plan776_r3_cached_future_await_triage` 钉住：首回有界 500、重放普通 int 数据面（无身份泄漏）、半驱动凭据经任务终态清偿回基线。归 705 轨（KNOWN-DEBT P776-D3） | **过** |
| AC-05 错凭据 | **owner 11 位段**：`owner_segment_exhaustion_rejected`（顺序创建 2047 中间 VM + live-A 交叉，第 2049 个 VM 受控拒绝——7 位掩码回绕已修）；id 层 owner 校验先行（跨 VM 同号拒绝）；stale（回收后 miss）、非资源对象 downcast 拒识、local 2^20 耗尽分配前拒绝；Future 0xF0 位模式普通 int 守卫保留（plan705 int 反例） | **过** |
| AC-06 存活 | 1000 次构造-返回闭环 live 回基线（dying 8192 步宽限窗经 drain 跨过）；常量重放 3 请求全 302；SSE 断连凭据回收；setter 返回值跨宽限窗存活（9000 步 201）；**容器级联清理**（`nested_record_error_releases_credential`：Box{response}+1/0 异常 11 请求回基线——R2-04）；SSE/File/Upload 移交合同=729/730/707 scoped 绿 | **过** |
| AC-07 来源身份 | **alias 按 target 展开**（`type Download = FileResponse` → File；alias→用户名 → Json；`http.` 限定名不受名集遮蔽——R2-07）+ 用户同名（struct×2/enum/空声明）三层遮蔽（提取/VM 门/back_proxy 双集）；755 契约覆盖 4/4；738 双指纹 75/75；751 三态单点未动 | **过** |
| AC-08 生成/拓扑面 | **已证面**：auto-man api_gen 46+1 ignored（生成 fixture 无模板替代回归）+ plan776 消费族 1/1；unsupported=tauri HTTP 专属矩阵拒绝保持；merged/IPC/back-proxy=back_proxy 19 项 18 绿（file-501/upload-501 守卫恢复；corpora 预存在册，master 同红）；不泄漏裸 id（client egress 凭据化后上游保真；资源对象入 JSON 兜底→500 不冒充成功）。**未验证面（单列，F776-R3-02）**：生成 Rust **真 TCP 矩阵未验证**——预存编译漂移（生成物 E0433+hyper Builder 差异编译失败，master `5e21561d2` 同形 TIMEOUT，742 附件同类在案），非本 diff 引入，归 742 轨收口；不据此判 pass | **partial** |
| AC-09 门禁对账 | §1/§3 + R3 修复轮门禁：无新增未解释红/超时；SD/decision/AC 表已按 `088e47823` 终形重写（F776-R3-01/02，spec-delta R2 终形稿）；future-cache 已分诊归 705（F776-R3-03，P776-D3）——**终判归 R4 独立复审** | 待 R4 |

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
