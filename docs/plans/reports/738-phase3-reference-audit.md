# PLAN-738 Phase 3 / T-11 收据：实际 callee 与引用闭包完整性审计/修复

- 任务：T-11（实际 callee 与引用闭包完整性审计/修复）
- worktree：`plan-738-dev`，基线 `c1579ed71`（T-10 提交），本任务提交见 §5。
- 触面：`stdlib_assembly/host.rs`（verify_rust_import/public_symbol_exists/参数 facade 视图）、`trans/rust.rs`（call 统一收集器/use_stmt 导入台账/Bina 三段闭包/for-in 流反糖验证）、`auto-man/api_gen.rs`（back 模块证明接线）、`tests/plan738_stdlib_assembly_tests.rs`（t11_reference_closure 5 测）。

## 1. 发射点分类对账（R2「329 发射点」声明 → 本轮逐类核证）

`trans/rust.rs` 全部 `a2r_std::` 文本引用 334 处，其中六核心 FQN 引用 139 处，按发射臂分组归属如下（每组的调用名形状 → 收集器路径 → 状态）：

| # | 发射臂组（代表行号，rust.rs 当前树） | 调用名形状 | 收集路径 | 状态 |
|---|---|---|---|---|
| 1 | 类型名渲染（`a2r_std::http::FileResponse`/`{name}`，4 处） | 非调用（类型文本） | — | **面外**：资源类型按名渲染，调用点证明覆盖其面 |
| 2 | `fn expr` Await 臂（post_sync/post_bearer/post_bearer_sync 侧信道壳） | `await http.sym(...)` | T-10 已接线的独立钩子 | ✅ 收集+验证 |
| 3 | Dot 模块臂（`(obj_name, method)` 主分派，post_sync/get_sync/last_status/post_bearer 族/post/get/put/delete/get_stream/post_stream/request/transfer_*/upload_*/download/file_response/post_stream_with_headers） | `http.sym(...)` | call() P1（T-10 契约门） | ✅ |
| 4 | `Json.`/`json.` 别名臂（parse/parse_opt/get/get_str/as_*/len/is_valid/is_null/from_value） | `Json.sym(...)`/`json.sym(...)` | call() P2（Json→json 映射） | ✅ 公共面符号验证；**无公共声明的 legacy 别名**（json.get/as_int 等）按既有 P738-D2 json 面分裂债务记边界（Ok(None) 不获公共证明，不冒称） |
| 5 | 裸名流生产者/消费臂（get_stream/post_stream/post_stream_with_headers/stream_next/stream_is_done/stream_close + `_async` 变体） | `sym(...)`（use 导入形态） | call() 裸名 FQN 分支（本轮新增）+ stream 三契约（本轮新增） | ✅ |
| 6 | `http_post` legacy 平面符号（`a2r_std::http_post`，2 段） | 裸名 | — | **面外**：legacy 平面私有符号，无公共面（与 VM 私有 intrinsic 同类） |
| 7 | `auto.<core>.<method>` 三段 Bina 臂（io.read_line 等） | `auto.io.read_line()` | call() Bina 分支（本轮新增） | ✅（io 等无 Rust producer 的模块诚实拒绝，不再发幻影） |
| 8 | for-in 流反糖（`stream_next[_async](&recv)`） | 编译器反糖（非用户调用名） | 反糖点直接验证（本轮新增） | ✅ |
| 9 | json! 宏/from_value 复合（宏调用文本） | 宏 | — | **面外**：宏展开非函数调用；其载荷经 from_value 调用点证明 |
| 10 | `emit_http_verb_call`（动词族，同步/async 面） | Dot 臂内部 | 经 #3 路径 | ✅ |
| 11 | JsonValue 接收者方法臂（`d.get(...)`/`v.keys()` 等，变量接收者） | Dot(var, method) | **call() receiver 分支（R5-02/R6-01）** | ✅ 公共方法分母归属（12 方法）：模块限定/扁平 value_* helper/json 值绑定 def-use 直发三形态验证 + 未接管发射臂（原样保留实参丢失）诚实拒绝；序列面 Vec<T>→[]T；既有真漂移面（is_null/is_valid 公共 int vs producer bool）双拼写一致拒绝 |

**R2 声明对账结论**：「329 发射点均属胶水/面外」不成立——本轮把 #3/#5/#7/#8 四类（用户可达的核心调用形状）全部接入 strict 收集/验证；#1/#6/#9 确为面外；#4 的 legacy 别名与 #11 的接收者面按既有登记债务（P738-D2/json 面分裂；方法面第四绑定面）保留边界并在本报告显式列出，不以「胶水」一言蔽之。

## 2. 新增闭包（本轮修复）

| 闭包 | 旧行为 | 新行为 |
|---|---|---|
| 具名导入 `use auto.<core>: f` | 导入发射 `use a2r_std::<core>::{f}`，裸调用保持裸文本：适配面符号（post_sync 族）发射**无适配壳的破损产物**（tuple vs str 静默漂移）；无任何证明 | `verify_rust_import` 逐项验证绑定+公共签名；纯适配面符号拒绝并指明限定拼写（`http.post_sync(...)`）；台账记录供调用点闭包 |
| 裸名调用（具名导入后） | 裸文本不经任何验证 | call() 收集器重建限定文本（已限定不重复前缀）走 T-10 契约门（元数/async 证据齐） |
| wildcard 导入 + 裸名调用 | 裸文本静默 | 公共面**唯一归属**才验证；多模块同名=歧义拒绝；无归属（用户符号）放行；§5.3 wildcard 导入本身不因未引用 unsupported 拒绝 |
| 同名用户函数 | — | fn_ret_types/fn_param_types 遮蔽优先，不误拒 |
| 三段 `auto.<core>.<method>()` | io/net 等发射幻影 `a2r_std::io::read_line()`，无验证 | verify_rust_reference 诚实拒绝（无 producer）或验证（http 等有面模块） |
| for-in 流反糖 | `stream_next[_async]` 发射无证明 | 反糖点按 stream_next 契约验证（含 AsyncHTTPStream 参数 facade 视图——契约声明，非类型名猜测） |
| 生成服务 back 模块（db.at/伴生 *.at） | `transpile_back_module_to_rs` 用 **Standalone 默认**转译并**丢弃证明**（而产物经 qualify_a2r_std 实际链接内嵌镜像） | `transpile_back_module_to_rs_with_proofs`：Embedded 运行形态 + 证明返回；`generated_api_assembly` 将 db+伴生模块证明并入 manifest（endpoint 面+back 模块面共同构成生成装配引用闭包） |

## 3. 新增契约（ADAPTER_RULES 增补至 15 条；R3-02 勘误：初稿误记 17——84bec29ef 实测 15）

stream_next/stream_is_done/stream_close 三件套的 async 面（`*_async` callee、`&AsyncHTTPStream` 参数经 `param_facades` 契约视图映射公共 `HTTPStream`；is_done/close 的 `_async` producer 是同步函数——awaited=false/producer_async=false 如实声明）。

## 4. 测试与门禁

| 命令 | 结果 |
|---|---|
| `cargo t plan738` | 67/67（含 t11_reference_closure 5 测：具名导入同源 identity/适配面导入拒绝+限定拼写指引/wildcard+namespace 收集与不越界/同名用户函数遮蔽/漂移面跨拼写一致拒绝） |
| `cargo t plan724` | 5/5（裸名流族经新收集器+FQN 分支全绿） |
| `cargo test -p auto-man --lib api_gen -- --test-threads=1` | 43/43（back 模块证明接线后生成链全绿） |
| `cargo test -p auto-man --lib assembly_freshness -- --test-threads=1` | 1/1 |
| tt 全档（--no-fail-fast） | 15 红全分诊：11 文档化 master 预存 + 4 负载 flake（plan484_024/plan502/plan707_client/plan730_commit_target_matrix 两轮交替，均隔离复跑绿类）；**非基线红 = 0** |
| `cargo t use_semantics` / `native_registry` / `autovm_persistent` / `module_cache` | 7/7、13/13、20/20、16/16 |
| 三 crate check + 触面文件 rustfmt | 零错误/干净 |

VM 侧闭包（`verify_linked_native_closure` 字节码走查含 CALL_SPEC 动态分派/闭包/future/generator 入口；merge 失效陈旧证明；链接前业务指令零执行）沿用 R2 已闭合路径，测试在案（plan738 族 `final_interface_is_checked_before_any_business_instruction` 等）；本轮无 VM 触面改动。

## 5. 提交

- worktree `plan-738-dev`：`fix(stdlib): PLAN-738 T-11 引用闭包——导入/裸名/wildcard/三段限定/for-in 反糖/生成 back 模块全路径接入 strict 收集 (Plan 738)`（提交号见 git log；基线 `c1579ed71`）。
- 边界（不新增债务，均为既有登记）：#4 legacy 别名臂= P738-D2 json 面分裂（KNOWN-DEBT 在案）；#11 接收者分型面=方法面第四绑定面冻结（§9/P738-D2）；#1/#6/#9 面外。
- 映射：T-02/03/04/07、AC-01..05/07/08、SD-01..05/07。
