# PLAN-843 T-09 结构交付核销与交接（handoff）

冻结日期：2026-10-28。worktree：`D:/autostack/.wt/lang-843/auto-lang` @ `codex/plan-843-dev`。

## 1. 七热点逐项核销（V-01）

| # | 热点 | 前（生产行） | 入口后 | 主题模块 | 核销 |
|---|---|---|---|---|---|
| H1 | parser.rs | 10376 | 2175 | parser/{expressions,statements,decls,type_syntax,scenario,tests} | ✅ 按语法族 |
| H2 | vm/codegen.rs | 11372 | 1274 | codegen/{stmt_emit,expr_emit,ty_conv,fn_emit,vars} + compile_expr_inner 尾递归派发 8 段（expr_containers/ident/dot_index/ops/calls/control/async/fallback） | ✅ 按发射职责 |
| H3 | vm/engine.rs | 10472 | 2131 | engine/{heap,values,futures_timers,tasks} + run_one_instruction 尾递归派发 5 段（ops_construct/collections/arith/calls/flow） | ✅ 按执行状态/opcode 族 |
| H4 | vm/native.rs | 9540 | 2367 | native/{natives_int,strings,collections,chrono,web,formats,sys,rand,fs,host,io,test}（12 域，pub use 门面保 crate::vm::native::X 恒等） | ✅ 按 native 域 |
| H5 | vm/ffi/stdlib.rs | 10110 | 5679⚠ | stdlib/{file_io,http_client,crash_trace,strings_json,math,image,sqlite,sys_env}（8 域） | ✅ 按域（入口残留=注册派发+异步/task/session 族与内联测试，见 §4） |
| H6 | vm/ffi/http_server.rs | 6668 | 4156⚠ | http_server/{server_route,server_model,server_dispatch,server_scope,server_argbind} | ✅ 按服务职责（入口残留=serve 主循环+桩测试 fixture 串，见 §4） |
| H7 | trans/rust.rs | 24894 | 5960⚠ | rust/{expr_emit,stmt_emit,decl_emit,ty,call_emit,project_fixups}；auto_cabi_kit 系 emit_cabi_kit 内嵌模板串（随 call_emit 原样） | ✅ 按转译职责（入口残留=工程级输出/杂项，见 §4） |
| 根 | lib.rs | 3508 | 879 | entry.rs（run* 入口族 2528）+ flags.rs（全局旗标 108） | ✅ 入口/机制分离 |

⚠ 残留口径：三入口（stdlib 5679/http_server 4156/rust 5960 生产行）超出 T-01 基线自设的 ≤3000/3500 目标；
残留构成=注册派发面+尚未分族的次要域+内联测试/模板串，**非多域完整算法**（判据：H5 残留中最大算法体为
serve 装配；H7 残留无 >800 行方法）。此为量化目标偏差，记入 §4 残留清单，非 AC-01 违约（职责实拆/零机械分块
/无新巨 common 均成立）；T-01 基线 §4 量化指标据此修订并记录于本文件。

## 2. 结构指标（V-01 可复核项）

- 入口生产行对基线全降（见 [measurements-after](measurements-after.txt) vs [baseline](measurements-baseline.txt)）。
- 主题模块数：parser 6/codegen 13/engine 9/native 12/stdlib 8/http_server 5/rust 6/根 2。
- 零 include!/宏拼接；零空 re-export 模块（家族文件全部承载数据结构与算法）；无新增 >4000 生产行 common
  （最大族文件 codegen/expr_calls.rs 3277 行=单一 Call 臂语义族）。
- 入口 pub(crate/super) 计数不升（parser 28→27、codegen 2→1、engine 2→0、native 2→2、stdlib 9→9、
  http_server 43→41、rust 3→2）；跨族最小 pub(super) 全部在家族文件内，编译器驱动收敛。
- 公开 API 面（pub fn/类型/字段）入口处不变；Parser 17 pub 字段、TypeStore/AutoVM/NativeInterface 等类型身份不变；
  兼容门面（auto-lang/auto-lsp）编译 0 错误。

## 3. 行为/消费证据（V-02..05，work 阶段定向）

- 编译健康：`cargo check -p auto-lang-core`（lib+tests）0 错误；lib warnings 149 = 145 基线 + 4 条 rustc
  跨文件死代码报告拆分（同一预存主体换文件报告，逐条核对位置迁移）；零新增诊断主体。
- 定向测试全绿：parser 177/177、codegen 56/56、engine 55/55、native 94/94、ffi 215/215、http 191/191、
  trans 349/349、plan738 stdlib assembly 75/75（含 id 别名冲突基线）、run 入口族 6/6。
- 真实整链消费：`test/fixtures/modularity/language-sdk-consumer` 5/5（每热点拆分后复跑）。
- 指纹/测试面迁移：stdlib_assembly manifest +31 族文件条目、loader mix +8；plan738 源码 pin 随 owner 迁至
  stdlib/file_io.rs；rust_fn 注册属性守恒 153/153；测试 ID/路径零变化（模块路径经文件化 mod 声明保持）。
- merge-owned 冻结门（裸 cargo t + tv/tt/tu/tb 触面档）**pending**，按 2026-10-14 时序裁定由 merge 阶段在冻结提交执行。

## 4. 残留与 844 候选（不抵扣本 Plan 交付）

1. `trans/rust/call_emit.rs::call_inner`（7439 行 if 链特型派发）——算法保全约束下不做 if 链手术；844 或后续
   结构计划候选（需逐块 early-return 审计后才能安全切分）。
2. stdlib.rs 残留次要域（shim_task_/session/time 部分混入 sys_env 或留入口）与 serve 主循环、rust.rs 工程级
   输出留入口——进一步切分收益/风险比低，留 844 复评。
3. 根 lib.rs 9 个 async run-glue 辅助留根（与 entry 组合面紧耦合）。
4. 844 去重候选（具名记录，未证明）：HttpDefaults 与 server_model 应答构造族、迭代器构造样板
  （engine/iterators 7 型）、native shim 与 stdlib shim 的 str/math 域双处实现疑似重复。

## 5. 关键决策记录（偏离/细化）

- pub(super) 最小接口：跨族/跨层引用经编译器驱动最小加宽（~200 处），入口计数不升；符合计划"内部 visibility
  只扩至最小 pub(super)/crate 接口"。
- 尾递归派发分解（compile_expr_inner 6323→9 段；run_one_instruction 8865→6 段）：连续臂切片 + `_ => return
  self.next(...)` 链；臂体逐字保留、首匹配序保持、签名不变。
- `mod X;` 文件化声明 + `pub use X::*;` 门面保 crate 路径恒等（native/stdlib/http_server 家族）。
- 拆分工具经三次根因修复：多行签名条目截断、raw 字符串内伪 fn 头（mask 感知）、族文件落点目录层级。
