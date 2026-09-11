# vm 相关 Plan 索引

> 状态以 plan 文件自身为准；无显式状态行者已注明。归档列为文件当前位置
>（`plans/` = docs/plans/ 根，归档在 `archive/`；历史 `old/` 已并入）。
> 重编号注意：317/318/322 原编号为 327/336/338（2026-07-23 冲突改号，原号留给先创建者）；
> archive/355 与 plans/355-a2r-async-await-transpilation 同号不同 plan，勿混淆。

## 历史主线（archive/）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 001 | vm-function-integration | ⏳ Planning | archive/ | VM 早期函数集成设想，仅停留在规划 |
| 038 | fix-vm-borrowing | ✅ | archive/ | 早期 VM 借用问题修复 |
| 039 | vm-tests-migration | 🔧 | archive/ | vm_tests → autovm_tests 按复杂度分级迁移 |
| 068 | autovm-bigvm | ✅ | archive/ | 9 阶段建成字节码引擎并成为默认后端（ADR-01/02） |
| 069 | autovm-global-vars | ✅（*） | archive/ | REPL 全局变量持久化：任务复用 + 全局作用域 |
| 070 | bigvm-iterator | ✅ | archive/ | List.iter()/next() 与 lazy map/filter 适配器 |
| 071 | bigvm-closures | ✅ | archive/ | 直接捕获闭包模型，禁止借用捕获（ADR-03） |
| 073 | bigvm-migration-roadmap | ✅ | archive/ | evaluator → AutoVM 全量迁移路线图 |
| 074 | use-statement-multi-dir-search | 🟡 | archive/ | use 多目录查找，parser 侧完成、evaluator 侧后补 |
| 075 | config-template-modes | ✅ | archive/ | CONFIG/TEMPLATE 独立 codegen，VM 模式无关（ADR-07） |
| 076 | bigvm-generic-type-support | ✅ | archive/ | 泛型解析、单态化、List<T> 特化存储（ADR-04） |
| 077 | unified-object-registry | 🔧 自述 50%（代码已至 Phase 6，见分歧记录） | archive/ | HeapObject 统一注册表（ADR-05） |
| 078 | automan-integration | ✅ | archive/ | ModuleResolver trait 与 FilesystemResolver |
| 079 | automan-full-migration | ✅ | archive/ | auto-man 包管理器迁入 monorepo |
| 080 | autovm-stack-frame-bug | ✅（*） | archive/ | 入口压 dummy CONST_0，修 REPL 变量累积 |
| 081 | autovm-default-mode | ✅ | archive/ | AutoVM 设为默认，pac.at 支持按依赖指定模式 |
| 087 | autovm-generics-type-erasure-specialization | ✅ 核心 90% | archive/ | 用户泛型类型擦除存储 + 内置集合特化 |
| 088 | param-passing-modes | ✅ 核心 80% | archive/ | 参数传递模式与 VmRef/VmMutRef 引用类型 |
| 092 | rust-ffi-sandbox | ✅ Phase 1-6 | archive/ | Rust FFI 沙箱约定 |
| 094 | hybrid-ffi-bridge | ✅ Phase 1-5 | archive/ | #[rust_fn] 宏与 43 个 shim |
| 117 | vm-type-coercion | ✅ | archive/ | I32_TO_F32/I64_TO_F64 修混合算术位解释 bug |
| 118 | vm-test-failures-analysis | 🔧 183/197 | archive/ | 系统性修复 VM 测试失败（u8 推断、越界、void 返回等） |
| 121 | task-msg-system | ✅ | archive/ | Task/Msg actor 数据结构与语义 |
| 124 | async-future-await | ✅ Phase 2.1-2.3 | archive/ | ~T 蓝图与 .await 基础 |
| 125 | phase3-polymorphic-routing | ✅ | archive/ | on 块隐式 union、显式 ctx 路由 |
| 126 | phase4-micro-concurrency | ✅ | archive/ | .go 微并发派发 |
| 127 | autovm-task-system-execution | ✅ Phase 1-3（4 deferred） | archive/ | TASK_LOOP/HANDLE_MSG/REPLY 与 SPAWN_GO |
| 128 | scheduler-message-dispatch | ✅ Phase 1-8 | archive/ | 调度器消息派发与 GlobalMeta |
| 177 | vm-file-test-framework | 无状态行（索引 ⏳，代码已落地，见分歧记录） | archive/ | .expected.out/result/error 三断言文件测试框架 |
| 179 | migrate-vm-tests-to-file-based | 无状态行 | archive/ | 内联测试向 test/vm/ 文件测试迁移 |
| 191 | assert-and-precise-linker-errors | ✅ | archive/ | assert 内建与 linker 错误 span 精确化 |
| 192 | vm-enum-ext-codegen | ✅ | archive/ | enum 声明、ext 方法、is-match 变体匹配 |
| 194 | monomorphic-dispatch | ✅ | archive/ | 泛型集合 API 编译期单态派发 |
| 197 | vm-adt-generic-lists-pattern-debug | ✅ | archive/ | enum data、List<UserType>、Option<T> 等 11 项运行时特性 |
| 198 | native-metadata-from-source | ✅ | archive/ | native 元数据从 #[vm] 源声明派生 |
| 199 | vm-interactive-debugger | ✅ | archive/ | SOURCE_LINE、调用栈、GDB/agent 双调试器 |
| 200 | vm-missing-features-examples-14-33 | ✅ | archive/ | loop/continue/tuple/切片、map_err、fs 别名补全 |
| 201 | vm-four-pillars-enum-closure-result-spec | ✅ | archive/ | 四支柱：多字段 enum、闭包 HOF、Result 堆对象、spec vtable |
| 203 | native-registry-namespace | ✅ Phase 1-5（5f deferred） | archive/ | QualifiedName 命名空间，消除约 137 个别名（ADR-09） |
| 206 | closure-hof-call-closure-api | ✅ | archive/ | call_closure 公共 API 与 List 高阶 shim |
| 207 | enum-multi-field-destruct-construction | ✅ | archive/ | enum 多绑定解构与命名参数构造 |
| 208 | result-heap-object | ✅ | archive/ | CREATE_OK/CREATE_ERR 堆对象与 ERROR_PROPAGATE |
| 212b | rust-ffi-e2e | ✅ Phase 1 MVP | archive/212-rust-ffi-e2e.md | cdylib 构建 → VM 动态加载调用全链路 |
| 216 | cffi-bindgen | ✅ | archive/ | auto-bindgen 接入 CLI 构建管线 |
| 221 | nanboxing-migration | ✅ | archive/ | NanoValue 成为默认值表示（ADR-06） |
| 224 | vm-async-runtime | ✅ | archive/ | TaskSystem.run 桥、AWAIT_FUTURE 重入、async shim |
| 226 | auto-byte-text-abt | ✅ Phase 1-3 | archive/ | ABC↔ABT 汇编/反汇编与 Playground 集成 |
| 229a | vmtest-08-is-pattern-on-primitive | ✅ | archive/229-vmtest-08-is-pattern-on-primitive.md | IS_VARIANT 对原始类型的兼容修复 |
| 230 | vmtest-17-f64-struct-literal | ✅ | archive/ | 5 处 codegen 补 PROMOTE_F64 |
| 231 | nested-mut-fn-stack-corruption | ✅ | archive/ | SET_GENERIC_FIELD Void 标记修嵌套 mut fn 栈损坏 |
| 249 | unified-native-registry | ✅ | archive/ | 单一注册架构 + catalog 宏（ADR-09） |
| 265 | autovm-mcp-server | ✅ | archive/ | 7 工具 JSON-RPC MCP 服务器 |
| 266 | vm-a2r-conformance | Phase 1 完成 | archive/ | conformance 规范、对偶测试、差分引擎（ADR-10） |
| 269 | autovm-daemon-cli | ✅ | archive/ | auto serve/req 命名管道守护进程 |

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
| 567 | script-mode-w25-tail-w3-oracle | ✅（reviewed→archived） | archive/567-script-mode-w25-tail-w3-oracle.md | Err 值通道拦截（ERROR_PROPAGATE→catch_pc 绑 PyException 载荷）+主边界带错退出 exit 1+py 桥 476-480（getattr/getitem/kwargs may 变体·py_raise·py_int）+py_enter 返 __enter__ 值+PyException 前缀族+CALL_NAT 非 FFI 传播加固+ADD 先读后放池纪律（P053-8/got d 双根因）+namedtuple/structseq opaque 封送；p7 双红清偿 64/64·五相位 173 用例三方全绿；账本 P567-1..6；债 P567-R2 |
| 560 | script-mode-w2-sugar-batch | ✅（reviewed→archived） | archive/560-script-mode-w2-sugar-batch.md | py 桥 470-475 六件（contains/module/str/pow/truthy/is）+CALL_PY→CALL_NAT_COUNTED 改名（P555-D5 销号）+CALL_PY 错误出口 RuntimeError 统一+print shim GIL str() 臂+550 门控硬化 E5501（文件上下文限定·tv 带 path 通道·json_is_null 迁 .as）；tf 3435/tv 3595 零残留；账本 P560-1..6；债 P560-D1..D6 |
| 569 | py-ret-dynamic-dispatch | ✅（reviewed→archived） | archive/569-py-ret-dynamic-dispatch.md | P539-D2 根治：codegen py-类型侧表三件（last_expr_may_py/py_typed_vars/fn_may_py_returns 单层流型）+方法分派决策核插队（receiver_may_py 三源查表→.len() 发 obj_len 1863/其余发 obj_call 1862，str.* 静态路由优先权让位；谎言本体保留只覆盖路由）+shim_str_len PyObjectHandle 兜底 0→GIL len；Auto 臂逐字节零变化（.len() 语义同源 chars 计数）；py_list 去规避 8/8·p5-p9 五相位 20 套件零回归·四档门禁零新增红；账本 P569-1..6；债 P569-D1（rust 侧 StrFixed 同族）/P569-R1（.as lowering 括号重绑预存） |
| 575 | desktop-silent-exit-notification-toggle | ✅（reviewed→archived） | archive/575-desktop-silent-exit-notification-toggle.md | 526 桌面静默退出债归因收口：退出审计三挂点（`stdlib.rs` exit_audit/exit_audit_path/install_exit_audit_panic_hook——Process.exit shim site=vm_process_exit、全局 panic hook code=101+消息+位置链式保留既有 hook、`run_session` 正常返回 site=main_return 实机 shutdown 端到证；env `AUTO_DESKTOP_EXIT_LOG` 写失败静默=零行为变更）+ 复现驱动 notes_toggle_repro.mjs（bus 注入 notes_toggle×2 与真实铃铛同 DesktopCommand 路径，逐轮 stderr handler 证据）；N=20 独占运行 20/20 存活零退出、审计零记录→不可复现降档🟡（疑外部击杀 049 同族，审计机制常驻复现即启）；KNOWN-DEBT 526 行降档 + 535 D 销账；门禁 tf 3465/3466 + tv 3606/3607 唯一红=charts 预存；账本 P575-1..6 |
| 576 | vm-engine-semantics-batch | ✅（reviewed→archived） | archive/576-vm-engine-semantics-batch.md | DEBTS 043 引擎债合批清偿（G1-G5）：nanbox 位型保真——计划理论"encode 丢标签"实测证伪（Plan 437/474 已治愈往返，auto-val 对照集钉契约），存活根因=TAG_F32×TAG_I32 混算/比较时算术 else 臂与 LT/GT/LE/GE fallback 按位型 decode_i32（240.0+1→1131413505、50.0>100 恒真），修复=engine 消费臂 tag 驱动（nanbox_single_to_f32 混算臂+nv_pair_is_float_numeric 比较路由，DIV 含除零守卫）；tf 4658/4669 零红归因（11 红=layout 环境族 master 复现+osconfig 竞态+charts 预存，防误归因登记 KNOWN-DEBT）；账本 P576-1..7 |
| 588 | vm-inline-callarg-degradation | ✅（reviewed→archived） | archive/ | Stage B P-4/583 残留债①：CALL 内联实参形态跨迭代退化——静态模块白名单缺 file 致占位 receiver 每调用漏 +1 槽垫入表达式栈（ADD 吃 [0,结果] 而非 [a,结果]），白名单 += file 单点修（math 先例）；AUTO_VM_TRACE_OPS 定罪；corpus 99_p588debt 三形态金样；m16b 对账 319890==319890；观察债 P588-D1（其余 stdlib 模块白名单差集）；P588-1/2 台账 |
| 592 | dep-rust-parity-matrix | ✅（reviewed→archived） | archive/592-dep-rust-parity-matrix.md | 430 dep 管线 P0 特征化网：016 marshalling 全矩阵/017 生命周期语料 + 三轨对拍 runner（VM/a2r/oracle 同 path fixture 精确相等，a2r 腿 env 门控）+ 两处静默 0 收口（GET_FIELD dep 对象路由合成 getter/报错、未覆盖自由函数签名 VMError）；执行期抓出并修复 8 个管线真 bug（i8/i16 漏 cast、i32 返回零扩展、>2^48 弹参、自由函数 wrapper 装载链断裂、&str/String 参数折叠、path 依赖 syn 扫描、ffi header 条件发射、a2r rust 型绑定 let mut）；DIV-DEP-1..7 + 债 P592-D1..D5；账本 P592-1..6 |
| 597 | cffi-engine-face | ✅（reviewed→archived） | archive/597-cffi-engine-face.md | C 通道驱动 autoterm_core.dll 12 符号面（auto-term 004 §5③）：a2c 全量真机（MSVC 拒 DLL 直接输入 LNK1107→链 cdylib 副产 .dll.lib import lib+同目录分发；CFACE_OK/exit 0）+VM 标量子集（use.c 点式扫描缺口修复[Plan 216 存量]+load_manifest_file JSON 加载面+三分派臂+engine_face.json 8 符号，VFACE_OK）；缓冲出参 4 符号=VM 封送不可达边界实证归 004⑥；engine 胶水三径裁定（独立工具→a2c 链入/宿主内→a2r+侧车/VM→标量子集）；附带 VM 轨两枚存量缺陷实测定位（if 条件位内联 C-FFI+循环挂起/循环计数器嵌套 if 赋值断流，语料头注规避）；账本 P597-1..6 |
| 604 | vm-rc-lifecycle-fix | ✅（reviewed→archived） | archive/604-vm-rc-lifecycle-fix.md | KD-VM1~4 泄漏族根治（025-sys-monitor 55-147MB/min 复盘）：CONSTRUCT_INSTANCE 取 stake 改 **sp-1** 槽（pop field_count 后实例恰在栈顶；原实现取末字段槽 0 影子，NEW_INSTANCE 份额失明，struct 字面量经 List.push 每实例永久滞留 +100 obj/拍）；shim_list_push 四出口元素槽 stake transfer 配平（retain 后释放——CALL_SPEC→resolve 路径无 CALL_NAT 死区结算的缺口）+ ARRAY_LEN raw pop 补 DROP 同款收尾（for-in dup+arr.len 每拍孤儿一份列表拷贝）；GET_FIELD 噪音臂门控+结算转正（KD-VM2）；**勘误双落档**——B12 编码损坏=坏探针伪证（语料缺 LitPushTick msg/timer 声明致 handler 空跑）、KD-VM4「Number()」表述与码不符（实际 Cast 三处 emit 无降级，补整型族 Math.trunc 对齐 VM 截断）；协议固化 spec §RC 生命周期协议/§B12 编码不变量；验收 probe 5/5 硬断言（StructTick 4000→0、lenSeen=100/sum=4950）+ sys-monitor 实机暖机基线 10min +19.0MB≤20MB + stderr 0 GET_FIELD + tv 3634/3635、tf 3490/3491 唯红 charts 预存；账本 P604-1..6 |
| 596 | dep-rust-v2-trait-generic-callback | ✅（reviewed→archived） | archive/596-dep-rust-v2-trait-generic-callback.md | 591 V2 行为等价面：trait 白名单转发（Display/ToString/Clone/Engine，编译期定死分发）+泛型调用点单态化（词法 mono 提示+替换引擎+`__<label>` 条目/dispatch 剥后缀兜底）+反向回调 adapter 原型（注入式跳板+线程局部回调帧，experimental）；DIV-DEP-8 双半边收口（a2r Display 发射+VM url::Url 臂）与 D13 常量接收者点调用；base64 全红样本翻绿解禁（p10 13/13+p11 3/3，594 回填 0%→50%）；GENERATOR v1.4+manifest mono 段快路径比对；020/021/022 语料三件（回调拆分+A2R_SKIP 收窄+mono 重建对抗，独立 fixture 防 v3 键毒化——P592-D1 显形修正）；tf 3506/3506、ffi_dual+dep_parity 28/28、A2R 档 5/5；债 P596-D1..D6 |
| 608 | vm-dispatch-settlement-pool | ✅（reviewed→archived） | archive/608-vm-dispatch-settlement-pool.md | 604 直接后继·CALL_SPEC 分发路径结算收口（KD-VM5/6）：resolve→shim 臂补 CALL_NAT 同款死区（`rc_release_slot_range(sp_after, sp_before)` 单点——**shim 内池释放被 CALL_NAT 死区双下溢实证否证**（underflow +N/调用），池份额归属死区单点、shim 不参与）+ 内联 str/List 消费型臂 23 站点弹毕压结果前弹窗结算（堆按影子/池按内容，KD-VM6② 接收者暂存份额每调用孤儿 +1 修复）；T-01 定罪=静态差集（registry canonical 覆盖面盘点）+AUTO_DEBUG_DISPATCH tv 全量 trace——内联 push 臂**静态不可达**（resolve 恒命中 shim，复活即 UAF 面→`plan608_dispatch_golden_tests` 路由守护钉死+注释与实现脱节修正）；AC-03 索引接收者形态实证 50 CALL_SPEC、AC-03/04 专项 red→green 双向（修复前各 50 孤儿）；pool_soak 增 push 相位+收尾 reap_all（dying 宽限队列静止点收割，终态断言不失真）；规范 SD-01（执行点清单增 CALL_SPEC 分发区①②③）/SD-02（池份额口径）落 vm/overview §RC 生命周期协议；KD-VM5（死码移除候选·用户裁定）/KD-VM6（销账）+opaque_native 域外观察入 KNOWN-DEBT；tv 3654/3654、tf 3510/3510；账本 P608-1..3 |
| 602 | py-subclass-class-factory | ✅（reviewed→archived） | archive/602-py-subclass-class-factory.md | P539-D4 交付·Python 类派生工厂：**py 桥 481** `py_subclass(name,base,methods)`（exec 类模板+Str 方法缩进归一内联+Closure 方法真 `def` 包装器挂载——PyCFunction 非 descriptor 不绑 self 的实证修正；`_auto_cb_{i}` 单下划线避名称改写）+`run_closure_bridged` **n 参回调 ABI**（tuple 顺序封送/arity 不匹配 TypeError 单测钉死/0 参直呼/self 首参句柄）；a2py `_auto_subclass` helper 同构（lambda setattr）；语料 py_torch_subclass 三轨 4/4（p12：Module forward 钉值/Dataset n 参协议/训练环收敛/重入≥3 探针）；窗口/GIL/重入/生存期契约成文 roadmap §7.3；P539-D4 拆面销账（多线程泵归长期）；执行期登记存量怪癖 D/E（closure 内 py_*_may count 字节/py_call 单参 handle 面，语料绕开未修）；tv 3659/3659、tf 3516/3516、tt 3876 唯 3 预存红；账本 P602-1..6 |
