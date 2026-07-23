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
plan-report 07 文中的 plan 链接指向 `docs/plans/`，实际文件均已移至 `docs/plans/old/`，属报告链接腐化。
