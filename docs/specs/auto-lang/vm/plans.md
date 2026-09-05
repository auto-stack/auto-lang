# vm 相关 Plan 索引

> 状态以 plan 文件自身为准；无显式状态行者已注明。归档列为文件当前位置
>（`plans/` = docs/plans/ 根，`old/`、`archive/` 为子目录）。
> 重编号注意：317/318/322 原编号为 327/336/338（2026-07-23 冲突改号，原号留给先创建者）；
> archive/355 与 plans/355-a2r-async-await-transpilation 同号不同 plan，勿混淆。

## 历史主线（old/）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 001 | vm-function-integration | ⏳ Planning | old/ | VM 早期函数集成设想，仅停留在规划 |
| 038 | fix-vm-borrowing | ✅ | old/ | 早期 VM 借用问题修复 |
| 039 | vm-tests-migration | 🔧 | old/ | vm_tests → autovm_tests 按复杂度分级迁移 |
| 068 | autovm-bigvm | ✅ | old/ | 9 阶段建成字节码引擎并成为默认后端（ADR-01/02） |
| 069 | autovm-global-vars | ✅（*） | old/ | REPL 全局变量持久化：任务复用 + 全局作用域 |
| 070 | bigvm-iterator | ✅ | old/ | List.iter()/next() 与 lazy map/filter 适配器 |
| 071 | bigvm-closures | ✅ | old/ | 直接捕获闭包模型，禁止借用捕获（ADR-03） |
| 073 | bigvm-migration-roadmap | ✅ | old/ | evaluator → AutoVM 全量迁移路线图 |
| 074 | use-statement-multi-dir-search | 🟡 | old/ | use 多目录查找，parser 侧完成、evaluator 侧后补 |
| 075 | config-template-modes | ✅ | old/ | CONFIG/TEMPLATE 独立 codegen，VM 模式无关（ADR-07） |
| 076 | bigvm-generic-type-support | ✅ | old/ | 泛型解析、单态化、List<T> 特化存储（ADR-04） |
| 077 | unified-object-registry | 🔧 自述 50%（代码已至 Phase 6，见分歧记录） | old/ | HeapObject 统一注册表（ADR-05） |
| 078 | automan-integration | ✅ | old/ | ModuleResolver trait 与 FilesystemResolver |
| 079 | automan-full-migration | ✅ | old/ | auto-man 包管理器迁入 monorepo |
| 080 | autovm-stack-frame-bug | ✅（*） | old/ | 入口压 dummy CONST_0，修 REPL 变量累积 |
| 081 | autovm-default-mode | ✅ | old/ | AutoVM 设为默认，pac.at 支持按依赖指定模式 |
| 087 | autovm-generics-type-erasure-specialization | ✅ 核心 90% | old/ | 用户泛型类型擦除存储 + 内置集合特化 |
| 088 | param-passing-modes | ✅ 核心 80% | old/ | 参数传递模式与 VmRef/VmMutRef 引用类型 |
| 092 | rust-ffi-sandbox | ✅ Phase 1-6 | old/ | Rust FFI 沙箱约定 |
| 094 | hybrid-ffi-bridge | ✅ Phase 1-5 | old/ | #[rust_fn] 宏与 43 个 shim |
| 117 | vm-type-coercion | ✅ | old/ | I32_TO_F32/I64_TO_F64 修混合算术位解释 bug |
| 118 | vm-test-failures-analysis | 🔧 183/197 | old/ | 系统性修复 VM 测试失败（u8 推断、越界、void 返回等） |
| 121 | task-msg-system | ✅ | old/ | Task/Msg actor 数据结构与语义 |
| 124 | async-future-await | ✅ Phase 2.1-2.3 | old/ | ~T 蓝图与 .await 基础 |
| 125 | phase3-polymorphic-routing | ✅ | old/ | on 块隐式 union、显式 ctx 路由 |
| 126 | phase4-micro-concurrency | ✅ | old/ | .go 微并发派发 |
| 127 | autovm-task-system-execution | ✅ Phase 1-3（4 deferred） | old/ | TASK_LOOP/HANDLE_MSG/REPLY 与 SPAWN_GO |
| 128 | scheduler-message-dispatch | ✅ Phase 1-8 | old/ | 调度器消息派发与 GlobalMeta |
| 177 | vm-file-test-framework | 无状态行（索引 ⏳，代码已落地，见分歧记录） | old/ | .expected.out/result/error 三断言文件测试框架 |
| 179 | migrate-vm-tests-to-file-based | 无状态行 | old/ | 内联测试向 test/vm/ 文件测试迁移 |
| 191 | assert-and-precise-linker-errors | ✅ | old/ | assert 内建与 linker 错误 span 精确化 |
| 192 | vm-enum-ext-codegen | ✅ | old/ | enum 声明、ext 方法、is-match 变体匹配 |
| 194 | monomorphic-dispatch | ✅ | old/ | 泛型集合 API 编译期单态派发 |
| 197 | vm-adt-generic-lists-pattern-debug | ✅ | old/ | enum data、List<UserType>、Option<T> 等 11 项运行时特性 |
| 198 | native-metadata-from-source | ✅ | old/ | native 元数据从 #[vm] 源声明派生 |
| 199 | vm-interactive-debugger | ✅ | old/ | SOURCE_LINE、调用栈、GDB/agent 双调试器 |
| 200 | vm-missing-features-examples-14-33 | ✅ | old/ | loop/continue/tuple/切片、map_err、fs 别名补全 |
| 201 | vm-four-pillars-enum-closure-result-spec | ✅ | old/ | 四支柱：多字段 enum、闭包 HOF、Result 堆对象、spec vtable |
| 203 | native-registry-namespace | ✅ Phase 1-5（5f deferred） | old/ | QualifiedName 命名空间，消除约 137 个别名（ADR-09） |
| 206 | closure-hof-call-closure-api | ✅ | old/ | call_closure 公共 API 与 List 高阶 shim |
| 207 | enum-multi-field-destruct-construction | ✅ | old/ | enum 多绑定解构与命名参数构造 |
| 208 | result-heap-object | ✅ | old/ | CREATE_OK/CREATE_ERR 堆对象与 ERROR_PROPAGATE |
| 212b | rust-ffi-e2e | ✅ Phase 1 MVP | old/212-rust-ffi-e2e.md | cdylib 构建 → VM 动态加载调用全链路 |
| 216 | cffi-bindgen | ✅ | old/ | auto-bindgen 接入 CLI 构建管线 |
| 221 | nanboxing-migration | ✅ | old/ | NanoValue 成为默认值表示（ADR-06） |
| 224 | vm-async-runtime | ✅ | old/ | TaskSystem.run 桥、AWAIT_FUTURE 重入、async shim |
| 226 | auto-byte-text-abt | ✅ Phase 1-3 | old/ | ABC↔ABT 汇编/反汇编与 Playground 集成 |
| 229a | vmtest-08-is-pattern-on-primitive | ✅ | old/229-vmtest-08-is-pattern-on-primitive.md | IS_VARIANT 对原始类型的兼容修复 |
| 230 | vmtest-17-f64-struct-literal | ✅ | old/ | 5 处 codegen 补 PROMOTE_F64 |
| 231 | nested-mut-fn-stack-corruption | ✅ | old/ | SET_GENERIC_FIELD Void 标记修嵌套 mut fn 栈损坏 |
| 249 | unified-native-registry | ✅ | old/ | 单一注册架构 + catalog 宏（ADR-09） |
| 265 | autovm-mcp-server | ✅ | old/ | 7 工具 JSON-RPC MCP 服务器 |
| 266 | vm-a2r-conformance | Phase 1 完成 | old/ | conformance 规范、对偶测试、差分引擎（ADR-10） |
| 269 | autovm-daemon-cli | ✅ | old/ | auto serve/req 命名管道守护进程 |

## 近期（plans/ 根目录）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 242 | a2r-feature-gap-tracker | living document | plans/ | a2r/VM 语义差距与 workaround 活文档 |
| 317 | vm-async-scheduling-investigation | Phase 1 已合并，Phase 2-4 待实施 | plans/ | 三套异步机制实测状态调研；actor handler 执行引擎落地（原编号 327） |
| 318 | list-struct-id-corruption | 无显式状态行（文内修复项均 ✅） | plans/ | List\<Struct\> 元素 ID 的 nanbox tagging 修复（原编号 336） |
| 322 | list-struct-runtime-diagnosis | 排查记录 | plans/ | List\<Struct\> runtime 根因定位与排查方法论（原编号 338） |
| 325 | autovm-enum-method-and-cross-module-bugs | 无显式状态行 | plans/ | enum 方法与跨模块调用 bug 修复 |
| 335 | list-struct-runtime-fix | 文内验证项 ✅ | plans/ | read_state_as_vec VmRef 解引用等 List\<T\> 运行时修复 |
| 340 | list-value-methods | 🔧 方法覆盖推进中 | plans/ | ListData\<Value\> 补齐 filter/map/remove 等全方法 |
| 341 | vm-debugging-methodology | 方法论文档 | plans/ | VM bug 排查最佳实践：先降级为纯 VM 脚本复现 |
| 348 | fix-parity-workaround-bugs | 🔧 部分完成 | plans/ | parity workaround 修复，含 SSE 流任务挂起机制 |

## 归档（archive/）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 298 | remove-non-nanbox | ✅ | archive/ | 删除全部非 nanbox 代码路径（2739/2746 通过） |
| 310 | auto-ownership-escape-analysis | ✅ | archive/ | 所有权/逃逸分析，为闭包借用捕获铺路 |
| 312 | autovm-api-routing-http-server | ✅ Phase 1-4 | archive/ | #[api] HTTP server 自动启动与路由 |
| 313 | autovm-tcp-flush-sse-server | ✅ Phase 1-2（3 留待） | archive/ | TCP flush 修复与 SSE server |
| 316 | auto-lang-fix-312-server-panic | ✅ | archive/ | 修 312 server 启动即 panic 的阻断 bug |
| 321 | generator-runtime-yield-iter-stream | ✅ | archive/ | yield/~Iter 生成器与 HTTP 流迭代器 |
| 326 | vm-runtime-struct-list-serialization | Phase 1-5 完成 | archive/ | struct list 序列化；顺手修 generator for-loop 重复值 |
| 355 | fix-persistent-session-fn-body-recursion | ✅ | archive/355-fix-persistent-session-fn-body-recursion.md | session.run 移至 8MB 栈独立线程，修解析栈溢出（与 plans/355 同号不同 plan） |

（*）plan-069/080 文件无显式状态行，状态取自 docs/plan-reports/07-vm-runtime.md。
plan-report 07 文中的 plan 链接指向 `docs/plans/`，实际文件均已移至 `docs/plans/archive/`，属报告链接腐化。

## 2026-08 增补（Plan 471）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 442 | cross-platform-closure（VM 侧） | ✅（reviewed→archived） | archive/ | 跨平台合龙 VM 侧全链：后端 .at VM 直跑（32 模块 31/32 clean，第 32=extern_sigs 旁车设计）+ axum serve 适配层（axum_adapter.rs：提取器编组/{x}→:x/__axum:N 即时安装）+ extern 响应构造器与 SSE 形态（musk_response_ctor.rs：响应族+Sse/KeepAlive+sse_frame_from_nv event:/data: 帧）+ host_bridge 转发（try_host_forward+RC 死区 retain 补偿）+ rust 形态桥三件套（env.var/.ok()/clone 直通）+ parser 四批修复（Rust 类型相容/args 链式 Bug A/format!/枚举 is→IS_VARIANT/# 注释容错）；复审日 P442-4 双层撞号修复：e2e 端口 ×3 去重+e2e_ports_unique 守卫、native ID 3129 撞号（musk_extern_dispatch×value_get_bool）dispatch 移段 3143+catalog 442 家族靶向钉；观察期 09-03 期满无回滚；债务 P442-1..5;P442-1..6 台账 |
| 446 | vm-backend-os-config-field-report | ✅ | archive/ | 实战 VM 渲染薄弱点清偿（A1 多 store 消歧/J1-J2 子树）；账本 P446-1..4；批五收口+下游结算完成（2026-08-29，reports/446-downstream-settlement.md） |
| 466 | test-speedup | ✅ | plans/ | sccache/cargo t ≤30s/全量门禁收敛 review；账本 P466-1..7 |
| 474 | vm-json-float-dot-read-fix | ✅ | archive/474-vm-json-float-dot-read-fix.md | CALL_SPEC 数学分发 nanbox 化石根除（一元 i32 位读/二元参数序倒置，plan011④）；三层回归载具 vm_json_float_read_tests；账本 P474-1..7 |
| 504 | calculator-fit-window-osconfig-stdlib（VM 侧） | ✅ | archive/ | stdlib 静态分发：Math.pow/Str.is_digit Rust shim（native_registry 自动扫描）+ 056/057 文件测试；fit 窗口 renderer/session 双路径（VM 独立窗首帧 shrink 测量 resize） |
| 510 | vm-pool-over-release | ✅ | archive/510-vm-pool-over-release.md | 字符串池 over-release 注入源清偿：19 处无计数池引用收口 add_string/intern_runtime_str/rc_push_str_idx 咽喉 + over-retain 家族（BUILD_FSTR/pop_tagged/StakeGuard）配平；PoolHealth 快照（underflow/phantom/live_shares）+ soak 双档门禁；P499-6/7 顺产清偿（Log ID 移段/kitchen-sink 关键字名）；账本 P510-1..6 |
| 525 | aavm-oop-batch | ✅（reviewed→archived） | archive/525-aavm-oop-batch.md | aavm 目标语言高阶能力六波全交付:VBool 载体(P474-旁支核销)/方法族(type fn+ext+static+接收者简写,独立 fn `Type.method` 重整)/is-struct 解构(宿主 VM panic 洞顺修)/容器族 List<T>(CallNat 通道,塔顶前置达成)/闭包(MakeClo/CallClo+VClo)/嵌套 fn/pub type 跨模块(tys 播种)/May 最小面(?T+Some/None);复审修复位置参构造(13-methods 本体逐行一致);四路 30/30+tf 绿(2 红归属非 525);账本 P525-1..5(specs P525-1..6);.git 丢失事故经远程+重建快照恢复全程存活 |
| 539 | pytorch-ffi-phased-support | ✅（reviewed→archived） | archive/539-pytorch-ffi-phased-support.md | PyTorch 训练/推理脚本级支持四波：W0 六条 DIV-PY 债清偿（kwargs 冒号语法 py_call_kw 452/list TAG_OBJECT 封送/float f64 直通+TYPE_TO tag 分派/py_call_may 453 May 通道/py_iter 454+py_next 455+GIL 臂/importlib 句柄裁定）；W1 12 opcode dunder 路由（反射协议+逐元素语义）+matmul/getitem/setitem/slice/call0/with 六内建+py_torch_infer 16 例；W2 py_item_kw 464 项导入 kwargs+py_float 465 标量语义（张量留柄保 backward）+混型 EQ+py_torch_train 10 例 seed 化收敛；W3 回调桥 py_callable 466（thread-local 桥窗口）；py_subclass 延期 P539-D4；栈帧纪律三次实证（特形拦截抽助手）；账本 P539-1..6；债务 P539-D1..D5 |
| 550 | null-family-audit | ✅（reviewed→archived） | archive/550-null-family-audit.md | null 家族静默垃圾全族翻转为可捕获 Python 风格 TypeError（脚本模式 W0 地基）：算术族共享弹栈助手 pop_arith_pair/operand_non_null+_F/_D/_U64/MOD peek 守卫；拼接病灶=STR_CAT 臂；GET_ELEM null→not subscriptable+越界（ListData 四型）→IndexError（负索引保留，tv 存量零撞击）；迭代病灶=ARRAY_LEN 静默 0 臂（顺带翻 null.len()）；CALL_CLOSURE 动态 callee→not callable（正常模式编译期 E0401 先拦，单测钉 P550-D4）；TYPE_TO_I32/F64 null 臂翻案；TYPE_TO_STR/print→"None"（a2py 三方对齐，py_torch_infer 扩例 17/17）；守卫边界=TAG_NULL only（三拼写 PUSH_NIL 归一，i32 哨兵不可守卫 P550-D3）；tests_null_guards 13 单测；门禁 tv 3585+tt 3772+py 五套件 64/64；顺手修 master ui-iced 档编译断裂（plan051 遗留，P550-D7）；账本 P550-1..6；债务 P550-D1..D7 |
| 555 | script-mode-w1-dispatch-foundation | ✅（reviewed→archived） | archive/555-script-mode-w1-dispatch-foundation.md | 脚本模式 W1 动态分派地基：ForeignObject 协议（HeapObject::as_foreign_object 默认钩子+PyObjectHandle 六操作臂首实现，send/contains 预留位）+分发组合子六件 1860-1865（vm/interop.rs，运行期 tag 分派 py 桥/Auto 原生方法表——str/list/map 索引按名读写/ARRAY_LEN 语义/array 通道迭代回推/守卫对标 550）+py 三桥 py_setattr 467/py_len 468/py_type_name 469（539 桥型）+CALL_PY 计数传输形态发射（P555-D5 命名债）；门禁 tf 3428/3429（唯一红=master 既有 charts 甄别）+tv 3588/tt 3775+py 五套件 64/64 零回归；账本 P555-1..6；债务 P555-D1..D5；P550-D6 销号/P550-D4 期望面更新 |
