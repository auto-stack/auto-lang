# KNOWN-DEBT-AND-RISKS — 已知技术债与风险登记簿

> **用途**：统一记录已归档计划中遗留的 workaround、一致性遗漏、架构风险和未来增强。
> 避免未来需要全扫归档计划才能找到这些隐患。
> **维护规则**：每次计划归档时的复审发现新遗留/风险，在此追加条目。
> **格式**：`[计划号] 严重度 | 类别 | 一句话描述 | 引用位置`

---

## 🔴 高风险（可能在特定场景导致 UB 或数据损坏）

| 计划 | 类别 | 描述 | 引用 |
|------|------|------|------|
| 385 | 逃逸风险 | 闭包 capture_slots 记录 creator_bp，若闭包逃逸（存入全局变量、在创建者函数返回后调用），creator_bp 指向已释放栈帧 → UB。当前无逃逸检测。常见用例（forEach 回调、直接调用）安全，因为创建者仍在栈上。 | `vm/engine.rs` Closure.capture_slots + `vm/codegen.rs:10971 compile_closure` |

---

## 🟡 一致性遗漏（功能正确但代码不干净）

| 计划 | 类别 | 描述 | 引用 |
|------|------|------|------|
| 377 | heap-aware 遗漏 | stdlib.rs 有 10 处 `push_i64(handle/server)` 未改用 `push_i64_vm`。值是 heap ID（< 2^48），实际安全，但不符合 Plan 377 的"所有 64 位值走 heap-aware"一致性目标。 | `vm/ffi/stdlib.rs:3092,3105,3115,3125,3135,3146,3478,3493,3511,3531` |
| 377 | TYPE_CAST_U64 | engine.rs:2690 的 TYPE_CAST_U64 用 `push_u64(v as u32 as u64)`，值 < 2^32 安全，但未走 heap-aware 路径。 | `vm/engine.rs:2690` |
| 340 | reduce init_val 类型 | shim_list_reduce 的 Value path 中 init_val 仍是 `pop_i32()`（而非 `pop_nv()`+`nv_to_value`）。若 reduce 初始值是 struct/str 会丢类型。常见用例（init=0/""）不受影响。 | `vm/native.rs shim_list_reduce` |
| 399 | api_gen 后处理兜底 | `post_process_db_rs` 保留 5 类后处理兜底（i32→i64 之外的：去 deref 正则、str→String、`id: *NEXTID as i64` 替换、use 路径映射、strip_collection_new），代码自注 "workarounds, not redundant"——a2r 根治回归面广，正则方案为永久设计。 | `auto-man/src/api_gen.rs:331` 起 |
| 396 | 一致性遗漏 | a2r-std/src/*.rs 是 stdlib/auto/*.rs.at 的手抄副本，无生成/校验环——time.rs 曾漂移到 i32（§2.6 教训：stdlib 声明 i64）。建议后续由 stdlib 生成或加 CI 签名比对。 | `crates/a2r-std/src/time.rs` vs `stdlib/auto/time.rs.at` |
| 396 | 绕道残留 | auto-ai 两处 sed 仍在（agent tier.rs `Some(m.clone())` 属 Plan 020、SOUL const `&str` 属 Plan 016 可选项）——396 §2 范围内 sed 已全部毕业，这两条按计划归属留在原计划。 | `auto-ai crates/auto-ai-agent/retranspile.sh:82` |
| 417-E2 | a2r 后处理盲重写 | `fix_vec_i32_index` Pattern 2 把任意 `xxx.get(i)`（参数名 ∈ int_like 名单：i/j/k/idx/n/...）正则重写为 `xxx[i as usize]`，不看 receiver 类型——用户类型若有 `.get(int)` 方法且局部变量名不在 hash_map_names 白名单，产物编坏（E0608）。E2 parity wrapper 以变量名 `data`（白名单内）规避；根治需让该启发式感知类型。 | `trans/rust.rs fix_vec_i32_index` Pattern 2 |

---

## 🟢 已知限制（设计决策，非 bug）

| 计划 | 类别 | 描述 | 引用 |
|------|------|------|------|
| 377 | BigInt 溢出 | virt_memory 的 push_i64/u64 在 >2^48 时 panic（快失败）。engine/native 层有 heap-aware 版本（push_i64_vm），但 virt_memory 层无 VM 访问无法堆装箱。设计如此。 | `vm/virt_memory.rs push_i64/push_u64` |
| 340 | forEach 闭包副作用 | forEach 的闭包副作用曾不生效（by-value 捕获）。Plan 385 的 capture_slots 已修复。但 forEach+Plan 385 的联动未单独测试。 | `vm/native.rs shim_list_for_each` + `vm/engine.rs capture_slots` |
| 385 | 保守 by-reference | 所有被闭包引用的外部变量都按 by-reference 处理（无 escape analysis 区分）。简单但可能有性能影响（大量变量走间接访问）。 | `vm/codegen.rs compile_closure` |
| 365 | W3 libcosmic lowering | `run_libcosmic` 的 VTree→libcosmic Element 真实 lowering 和 iced::Application 启动是 TODO（gated on libcosmic dep，Linux-only）。Windows 走 headless 委托。设计如此——按 COSMIC 组件复刻增量填充。 | `auto-cosmic/host-libcosmic/src/linux.rs` |
| 365 | W4 真实 D-Bus adapter | `LinuxNotificationsPort` 的 D-Bus signal handler 集成是 TODO（FreeDesktop Notifications 是 push API，需 COSMIC notification-daemon 组件驱动）。`LinuxPowerPort` 已实现 UPower 查询但未在 WSL2 验证。 | `auto-cosmic/ports-linux/src/linux.rs` |
| 365 | gpui Image/Grid placeholder | gpui 后端的 `View::Image` 渲染为 `[img: src]` 文本占位符，`View::Grid` 做行列分解但无原生 GPUI grid 支持。功能可用但不完整。 | `ui/gpui/auto_render.rs` + `ui/gpui/renderer.rs` |
| 365 | HostBackend Send bound | `HostBackend::run` 要求 `C::Msg: Send`（iced 的约束传播到方法级）。headless/gpui 本不需要 Send，但 GUI 消息类型按惯例都是 Send，实际无影响。 | `ui/host.rs` |
| 391 | trait impl 语法 | Auto 不支持 `impl Trait for Type` 语法（D6 仅提供清晰错误："Auto does not support trait impl syntax... Use a static fn/ext method"）。是语言设计决议，非缺陷——用 `ext Type for Spec` 表达外部 trait 实现。 | `parser.rs:4578-4585` |
| 346/317 | e2e 端口竞态 | test-http-e2e 串行套件：先行的 detached server 线程可能迟到 auto-start，读到进程级 AUTO_HTTP_PORT（彼时已属于后续测试）并用陈旧路由表抢占其端口（偶发 404/10048，受害者随负载轮转，e2e 单测均过）。缓解：受影响测试命名排序靠前 + CI --retries；根治需 per-server 传端口而非进程级 env。 | `vm/ffi/http_server.rs` http_e2e::start_server + AUTO_HTTP_PORT |
| 317 | serve_async 生命周期 | `serve_async` 无受控 shutdown（tokio::spawn_local 泄漏），高负载下 3 个 SSE e2e 测试偶发 flaky，靠 nextest `--retries 2` 缓解。已明确留作独立 follow-up（候选：serve_async 生命周期管理计划）。 | `vm/ffi/http_server.rs:1152 serve_async` |
| 411 | 绕道/已知限制 | 🟡 | vue codegen 三处 gap 属性分支(vue.rs 5277/7072/7119,含无 gap 默认 gap-4 兜底)与 VM view_builder 八处 gap 提取保留未拆——显式向后兼容决议;validator 属性白名单未加,防 AI 再写 gap 属性的防线缺失 | 拆除需按 411 §4 方案迁移存量 + 双端回归;当前渲染正确 | `ui_gen/vue.rs` + `ui/aura_view_builder.rs` | 2026-08-22 |
| 411 | 已知限制 | 🟢 | Inter 与 vue 并排字形截图人工核对未执行(服务端由 038/013 实机回归覆盖,emoji/中文回退正常) | 视觉确认项,无功能影响 | 411 P1-C | 2026-08-22 |
| 243 | 延期/已知限制 | 🟢 | VSCode 实机 F5 着色核验(semantic tokens 视觉效果与增量稳定性)——服务端逻辑已被单测+协议测试锁定(416 5-B),剩一次手动视觉确认 | 需桌面 VSCode 会话;约 10 分钟 | `crates/auto-lsp/src/semantic_tokens.rs` | 2026-08-22 |
| 243 | 一致性遗漏 | 🟡 | rename 实测为单文档作用域(416 6-B 协议测试实证):虚拟 URI 无 resolver 根时不跨文件;真实磁盘 workspace 是否跨文件未验证 | 跨文件 rename 需 resolver 根 + 磁盘文件,虚拟文档场景受限 | `crates/auto-lsp/src/workspace.rs` | 2026-08-22 |
| 417-E4 调查 | parity 断裂 | ~~未解~~ **已根治(同日 ifexpr 批)**:body() 尾位 Stmt::If(带 else)纳入 is_returnable + 值位旗标 value_if_tail(then/else/else-if 臂与嵌套 if 尾表达式免分号,语句上下文 Plan 393 E3 行为保留);golden 013_if_tail_value 锁定;**trait_advanced 三方 10/10 全绿** | `trans/rust.rs` if_stmt + body | 2026-08-22 |
| 417-E1 调查 | parity 断裂 | ~~已解~~(2026-08-22 同日由 417-D2 根治):string_utils a2r 构建断裂的根因是**单文件转译对 `use auto.<mod>:` 导入函数的签名盲区**(fn_ret_types/fn_str_param_indices 无导入条目)。417-D2 的 register_import_signatures 在 use_stmt 处理时解析可发现的模块源(./auto/<mod>.at)登记签名,配合 417-E1 的 a2r-std i64 对齐 + as_str 白名单扩展 + runner 镜像拼写,**string_utils 三方 22/22 恢复全绿**。 | `trans/rust.rs` register_import_signatures |
| DIV-A2R-STRPARAM-1 | 一致性遗漏 | ~~🟡~~ ✅ **已修复(2026-08-23, Plan 427)** | 引入提交实证为 3f6aa1be(396 §2.4):`is_str_slice_var` 补查 `local_var_types` 的 StrSlice 登记,误把显式 `let x str` 局部(产物 Rust 中是 owned String)当作已是 `&str`,抑制调用点 `.as_str()` 自动借用→E0308。修复:§2.4 的真实目标(&str 返回 scrutinee 的 `Some(x)` is-arm 绑定)改走专属集合 `str_slice_pattern_bindings`,`is_str_slice_var` 不再查 local_var_types(与 8108 处 Pass 7 注释同一陷阱的复引入)。golden 008_str_param_borrow(含独立 cargo 编译验证 + rustc 类型检查冒烟)锁定;三库恢复 serde_json 56/56、url 30/30、base64 33/33 | `trans/rust.rs` is_str_slice_var + str_slice_pattern_bindings | 2026-08-23 |
| 417-followup | 已知限制 | 🟢 链接期字符串重映射表仍为 Vec<u16>:池索引操作数已 u32 化(8482021e),remap_string_indices/remap_obj_indices 的操作数读写已同步 u32,但 old→new 映射表本身仍是 u16——依赖模块字符串去重后若合并池规模越过 65535 且发生非平凡重排,映射值会截断。当前各库规模远未达到;触发前提是单个程序(主模块+依赖)去重后池 >65535 条目 | `lib.rs` remap_string_indices(表类型注释处) | 2026-08-22 |
| 417-E3-P4 | 延期/已知限制 | ❤✅ 已实施(2026-08-22 同日补齐):codegen 新增 fn_type_param_bounds + check_generic_call_bounds,调用点按参数声明类型映射到带 bound 类型参数,实参静态类型(User/GenericInstance) 可确定未实现 bound 时编译期拒绝;保守策略——非 Ident 实参/类型未知/调用者自身泛型参数透传/非 spec 约束/类型不可解析均放行(留给运行时 CALL_SPEC 报错)。trait_vm_tests +3(违规拒绝/合法通过/透传不误报) | `vm/codegen.rs` check_generic_call_bounds | 2026-08-22 |
| 410 | Expr::Dot 不查符号 | `x = a.b` 中 `a` 未定义今天仍通过（Expr::Dot 不经 check_symbol；Bina(Op::Dot) 分支源码不可达）。Phase 2 立项时须一并纳入。 | `parser.rs check_symbol` |
| 381 | v1 限制 | Node::deserialize 只处理 props（标量字段），不含 kids（命名子块）。嵌套块反序列化留给 v2（需 field-level resolver）。覆盖 role_config 等全部用例（字段全是标量/数组）。 | `auto-val/src/de.rs:79` |

---

| 425 | view 可选化拼写 | widget 体内未知块关键字的拼写错误（如 `veiw {}`）静默解析为视图元素而非报错。spec 已注明取舍。 | `parser.rs` parse_widget_decl `_ =>` 分支 |
| 425 | 根序语义 | 根组件 = 源序首个 widget（component fn 糖化后不再追加在 widgets 之后），App 应前置；真实工程 app.at 均如此。 | `ui_gen/api.rs` + scenario-dialect spec |
| 425 | 参数类型擦除 | component fn 参数的自定义类型仍擦除为 any（fragment hint 映射保留，保糖化字节等价）；widget 拼写走 parse_type 全类型。 | `parser.rs` fragment_param_hint_to_type |
| 426 | setup 解释器侧 | `setup {}` 仅落 a2vue（script setup 顶层）；解释器（AutoUI 继承 AutoVM）每实例执行约定未实现，登记后续 auto-ui interpreter 联动。 | `parser.rs` parse_setup_block_inner + 归档计划 §5 |
| 426 | async setup | setup 内 `await` 编译期拒绝（async setup 需 Suspense 边界），单测锁定；支持另立任务。 | `parser.rs` stmt_expr_contains_await |
| 426 | 模板 refs 解包 | setup 绑定的 refs 标注字段在模板中不自动解包（普通对象嵌套 ref 的 Vue 语义）；script 侧已注入 .value。继承 composable facade 机制。 | `ui_gen/ts_adapter.rs` facade_ref_fields |

## 📋 未来增强（非风险，记录为后续优化方向）

| 计划 | 类别 | 描述 | 引用 |
|------|------|------|------|
| 377 | opcode 合并 | ADD/ADD_F/ADD_D/ADD_U64 等变体仍保留（都单槽但未合并为单一 ADD）。合并属 plan 389。 | `vm/opcode.rs` |
| 377 | typed print 未删 | NATIVE_PRINT_I32/F32/F64/U64 仍保留为显式入口（print 路由已统一到 PRINT_UNIFIED，但 native 本身未删）。 | `vm/native_catalog.rs` |
| 340 | remove/set/insert/sort | Plan 340 只做了 HOF 方法（map/filter/find/any/all/reduce/for_each）。remove/set/insert/sort 已由 Plan 335 支持了 ListData<Value>，但未经专项测试。 | `vm/native.rs` |
| 385 | escape analysis | 未来可加 escape analysis，让不可变捕获仍走 by-value（fast path），仅可变捕获走 by-reference。 | — |
| 393 | dead-code remap 清理 | `rust.rs:5186` 的 `Expr::Bina` dispatch 块里有旧的 `"append" => Some("push_str")` remap（无守卫），但该路径是 dead code（parser 实际走 `Expr::Dot` 的 :5127/:6447 路径）。不影响功能，可选清理。 | `trans/rust.rs:5186` |
| 395 | json.decode turbofish 迁移 | `json.decode[T](text)` 可迁移为 `json.decode<Type>(text)`（rust.rs:3569 特判改为读 `generic_args`）。当前 Index hack 仍工作，auto-ai 仅 1 处使用，非阻塞。 | `trans/rust.rs:3569` |
| 308 | a2gd documented gaps | 5 条 Godot demo 逆向翻译 sugar 差距（GDScript `$`/`&""`/三元 sugar、复杂 sub_resource、packed arrays、node metadata、is 工效）显式不实现，留档于归档计划附录。 | `docs/plans/archive/308-*.md` 附录 |
| 364 | Try/深递归 deferred | a2r 的 `Stmt::Try` 降级 deferred（try 是运行时 catch 模型，不映射 Result）；F4 深递归栈溢出根因未根治（perf 测试用 16MB 线程为 interim 缓解）。 | `trans/rust.rs` / `perf_benchmark_tests.rs:169` |
| 417-E2 | uninit var 收紧 | `var x T` 无初始化器：VM 合法（Nil 起始）但 a2r 发射 `let mut x: i64 = None;` 死形状（任何类型，非关联类型特有）。2026-08-22 调查：全仓 .at 仅 2 处使用（a2c 11_methods/003，同文件 2 行）；语言规范从未文档化该形态（spec 示例全带初始化器）；bare `var x` 全仓 0 处；auto-man 生成器无此模板。**方向（用户倾向）：parser 要求必须初始化**——代价远小于先前评估，待办：①REPL 单条声明是否豁免需定夺；②上层仓（auto-ai/rust-workspace）.at 源合并前复扫；③a2c 测试改 2 行；④a2r 删 `= None` 分支。 | `parser.rs var/let 声明` + `trans/rust.rs store()` |
| 417-E2 | has-Spec 关联类型绑定 | `has f Type for Spec` 子句只接受类型列表，无法携带 `Item=int` 命名绑定——spec 声明关联类型而实现者走 has 委托时 a2r 产物缺 `type Item = ...;` 编不过（VM 动态不受影响）。缺的是**语法设计决策**（绑定形态如何挂到 has 子句），非代码缺口；现无用例。语法拍板后实现量小（parser has 臂 peek 分支 + 委托转发处替换）。 | `parser.rs has 子句` + `docs/handoff-E2-followup-assoc-body-refs.md` F2 |
| 417-E2 | trait_checker 参数类型比对 | check_conformance 只比对参数个数不比对类型（代码内既有 TODO）。收紧=全量现存 spec 实现重新校准（str/String、i32/i64 映射边界的兼容规则需拿 stdlib/parity/上层项目实测定表），独立立项，勿混特性批。 | `trait_checker.rs check_conformance` |
| 417-E2 | 关联类型 bound 语法 | `AssociatedType.bound: Option<Type>` 字段已预留（Display/atom 已留位），parser 不接受 `type Item has Bound`。零用例需求（Rust `type Item: Bound` 亦低频），YAGNI，先有需求再设计语法。 | `ast/spec.rs AssociatedType` |
| 409 | CodeBlock/PreviewCard 混合模式 | widgets-gallery 的 CodeBlock/PreviewCard 仍为「Auto 声明壳 + codegen 硬编码 UI」（generate_codeblock_html/generate_previewcard_html），改纯 Auto widget 需先验证 codegen 转译浏览器 API 调用的能力，未立项。VM 侧识别已由 §10 组 E 补上。 | `ui_gen/vue.rs generate_codeblock_html` |

---

## ⏸ 延期（finish-plan 登记的未竟项，Type=延期）

| 计划 | 类别 | 严重度 | 描述 | 根因/理由 | 引用 | 登记日 |
|------|------|--------|------|-----------|------|--------|
| 398 | 下游任务 | 🟢 | ash-gui-native M0.5 测试骨架（conftest/desktop_mcp/test_smoke）+ M1 in-process 后端 | 属 auto-shell 仓的下游任务，本仓计划仅负责 VM 侧修复（已完成） | docs/plans/archive/398-*.md §14.3/§14.4 | 2026-08-20 |
| 408 | 功能缺口 | 🟢 | P5-4：纯 module fn 文件不被 codegen（ui_gen/api.rs:456 报错） | 低优先 + 既有 workaround（塞进 widget/store 文件）；根治需先设计 codegen 入口扩展 | docs/plans/archive/408-*.md §11 P5-4 | 2026-08-20 |
| 406 | 审计矩阵 | 🟢 | 全量 nanbox 生产者-消费者类型配对审计矩阵（docs/audit/vm-type-audit.md）未产出 | 立项驱动的 4 个目标 bug 已全部由审计批次 A4/B4 根治，矩阵价值让位 | docs/plans/archive/406-*.md Phase 1 | 2026-08-20 |

*最后更新：2026-08-22（Plan 417-E3 有界泛型落地:登记 417-E3-P4 调用点界校验延期；2026-08-20 Plan 308/317/364/399/404/407/409/410 归档复审后；398/406/408 finish-plan 归档）*
