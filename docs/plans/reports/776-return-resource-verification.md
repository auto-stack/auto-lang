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

## 2. AC 逐条对账

| AC | 证据 | 判定 |
|---|---|---|
| AC-01 真碰撞 | e2e `http_e2e_plan776_collision_resource_vs_plain_int`：观察钩子取活跃资源实际 heap id → 同值普通 int wire 200 JSON 数字（`/api/plain/<id>`）→ 资源未被消费（`/api/replay` 仍 302+Location）。iterator 族对照由 legacy 查表臂保留（e2e_sse_chain 4/4 绿 + 探针 plain_numbers/legacy_registry） | **过** |
| AC-02 逐值身份与兼容 | 传播探针（globals/futures/captures）+ T-02 shim 接线（redirect/ok_response/file）+ 提取级条件分支由 e2e 碰撞族覆盖；6 legacy wire 用例=plan326 http_e2e 34 项全绿（redirect/SSE 链/accessors/musk 常量内含）；访问器面 engine CALL_SPEC 兜底不动（plan724 客户端矩阵绿） | **过** |
| AC-03 sync/async 同形 | e2e：async 资源终值 302、i64 全宽 5000000000、record 值域（`"id": 7` 序列化不降级）、Response 不变数字（碰撞族）/SSE 不变 JSON（sse_chain）；三 async 形态（Ready=External 延迟/Internal body await——delay_async+park 驱动）；named 面 e2e + closure 面 plan442 axum Router 链 17/17 | **过** |
| AC-04 终态 | e2e Failed→500（`async_failure_reply`，不再 200 null）；成功 Nil 200 null 保持（plan705 既有语义测试绿）；stale/wrong-owner 单测拒绝（resolve miss）；迟到/取消=plan705 9/9（park/cancel/shutdown 族零回归） | **过** |
| AC-05 错凭据 | 单测：wrong_owner（跨 VM 堆 miss）、stale（回收后 miss）、非资源对象 downcast 拒识、单调 id 无复用、耗尽防御（heap_id_gen 回绕分配前拒绝）；Future 0xF0 位模式普通 int 守卫保留（plan705 e2e int 反例绿） | **过** |
| AC-06 存活 | e2e：常量重放 3 请求全 302；**1000 次构造-返回闭环 live 计数回基线**（dying 8192 步宽限窗由工作循环 drain 跨过——`return 1` 式 drain 每请求 ~2 指令不足，量化在案）；SSE 断连凭据回收（cleanup 守卫+RC 归零）；rc 链路单测（push=1/release=0/reap_all 全收）；SSE/File/Upload 移交合同（take 单次/scope 收口）不动=729/730/707 scoped 绿 | **过** |
| AC-07 来源身份 | 三层冻结：提取级（ApiExtractor 两遍+`from_return_identity`——struct×2+enum 遮蔽反例单测）/VM 门（API_RETURN_SHADOWED 编译期冻结单测）/back_proxy（fn_meta 三段，primary 段分类+遮蔽）；755 契约覆盖 4/4+CFFI merge 守卫（cargo t 内）；738 双指纹 75/75；751 三态单点未动 | **过** |
| AC-08 生成/拓扑面 | auto-man plan776 消费族 1/1（遮蔽端点 JSON 数据面）+api_gen 46/46（生成 fixture 无模板替代回归）；unsupported=tauri HTTP 专属矩阵拒绝保持（api_gen 46 内含）；merged/IPC/back-proxy=back_proxy 19 项 18 绿（file-501/upload-501 守卫恢复，corpora 预存）；不泄漏裸 id（资源对象入 JSON 兜底→heap_object_to_json None→500，不冒充成功） | **过** |
| AC-09 门禁对账 | 本报告 §1/§3：无未解释红/超时；SD 草稿见 spec-delta 报告 | **过** |

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
4. **HTTP_RESPONSES 表**：仅剩 client 注入面 + `http.response()`+setter 死面
   （test/examples 零用户）；表仍 thread_local 无回收——client 面生命周期
   归属 FFI 资源范围（§1.4 非目标），server 构造路径已全部资源化不经表。

## 5. 交付物清单

- 代码：`vm/ffi/http_return_resource.rs`（新，凭据类型+解析+计数钩子）、
  `TypeTag::HttpReturnResource`、producer 5 族切换、marshal/final_value_reply
  资源臂与 SSE 单源、`value_to_nv_via_task` 完整值桥、dispatch 双点 rc_release、
  来源身份三层（contract/extractor/codegen/back_proxy）+ 消费面切换
  （targets/api_gen）、测试 25 项（10 单测+探针、6 e2e T-03/T-04、2 e2e T-05、
  4 e2e/单测 T-06 等）。
- 报告：本报告 + `776-return-resource-spec-delta.md`（SD 草稿）+
  `776-return-resource-decision.md`（T-01 决策包）。
