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
| 419-P1/P2 | RC canary 确定性触发（2026-08-23 外部报告） | ash-gui(ash-runner merged 模式,最大 VM 应用)下**确定性崩溃**:首条命令提交(type→submit→store.RunCommand)即 panic `[RC canary] use-after-free: heap object 4000000 was freed 0.0s ago`(rc.rs:378),×2 复现,master 4c4d6db1+合并。db8a4600(无 RC 代码)同负载数小时零崩溃。3124 单测不覆盖该路径。两种定性:①RC 计数实现过释放(误报);②真 UAF 一直存在 —— 若为②,即 auto-shell plan 060 静默退出债(第五/十五轮)的根因真身。复现:`cd ash-gui/ash-gui-auto && AUTOUI_MCP_PORT=9390 ../ash-server/target/debug/ash-runner.exe` + MCP echo 提交。**2026-08-23 二次复测:8b5426fa 后仍复现(id 4001245)**。
**三次复测(afe30bf8,RUST_BACKTRACE=full)定位数据**:
- UAF 访问点 = engine.rs:3936(GET_FIELD 类指令:值经 `i32>=4_000_000 → 堆id` 启发式解码后 get_heap_object)→ rc.rs:389 canary;
- 派发路径 = iced update → renderer.rs:6677(update 闭包)→ call_fn_by_name(即 on_with_input_for handler 派发)→ run_one_instruction;
- 堆 id 从 4_000_000 递增(engine.rs:471),故 id=4000111 即**第 111 号分配**(早期启动对象);三次运行 id 在 4000000~4001245 间浮动(分配序随启动路径微变);
- 机理二选一:①RC 过释放族(第 4 族,GET_FIELD 路径的持有份额缺口——已修三族未覆盖);②真陈旧 VmRef 跨 handler 存活。**注意 3936 的 i32≥4M 启发式本身也可疑**:合法大整数会被误当堆 id 探测(本次恰命中已释放 id 才炸;未命中则静默 None——建议排查时顺带审视该启发式的误判面)。
**✅ 已解决(2026-08-23,分支 419-uaf a76e9cbe)**:根因 = `json_to_vm_value` **外层** Array/Object 臂组装顶层容器时漏「插入即 retain」(内层 `_inner` 两臂 Plan 419 已落地,外层漏了同款)——顶层容器的直接子引用被 child_refs 声明却零持有,父对象死亡时级联子释放抵消他人真实 stake,子对象提前释放。定性 = 上述②**真 UAF**(RC 落地前同缺口静默堆损坏,即 auto-shell plan 060 静默退出债根因的强候选)。崩点修正:非 3936/GET_FIELD,实际在**首帧 view() 状态物化**(read_all_state_materialized → vmref_to_vec);触发不在 submit 主路径(init 期埋雷,首帧引爆)。修复后崩溃用例转绿、ash-gui 62 过、本仓 3125 测全过、canary 保持开启。详见 plan 419 §9.7(P419_UAF_TRACE 埋点随修复留在代码)。 | `vm/ffi/stdlib.rs json_to_vm_value`;plan 419 §9.7;auto-shell plan 060 §第十六轮 |

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
| 432 | SET_ELEM 栈序 quick-fix | 数组下标赋值编译为 rhs→arr→idx→`set.elem`（value 展栈底），源自 codegen.rs ~5954 的注释自述"quick fix"（原想 SWAP/ROTATE 未做）。操作数顺序反直觉且有 40 行困惑注释，理想为自然序编译或引入 ROTATE。v2 侧已镜像此序（divergences.md M4 扩语料节）。 | `vm/codegen.rs` SET_ELEM 发射段 |
| 432 | .len() 发射依赖运行时注册表 | `.len()` 走 ARRAY_LEN 还是 CALL_NAT 取决于 BIGVM_NATIVES 全局注册表内容（str.len 已注册 → CALL_NAT；未注册类型 → ARRAY_LEN 兜底，codegen.rs 7217-7229）。字节码发射应是 AST+类型表的纯函数，全局注册表状态使其不可静态复现（v2 侧只能按"接收者==Array"闸镜像主路径）。 | `vm/codegen.rs` 7177/7222 |
| 432 | 负 int/字符串哨兵池内别名（残留） | D30 已修越界哨兵（池界外回落裸 i32，native.rs push_tagged_value_rc），但**池界内**别名仍在：无类型 ListData<i32> 里存裸 -1 仍读回池字符串 pool[0]（实测 -1 → "len"）。.at 侧负 int 需偏置编码规避（engine.at ev_enc/ev_dec，−1e9）。根治需 ListData 值域带 tag 或 .at 侧类型化 List。 | `vm/native.rs push_tagged_value_rc` + divergences.md D30 |
| 432 | 参数个数不匹配静默毁帧 | 调用方/被调方 n_args 不一致时宿主无诊断：RET 按 fn 声明弹参，帧逐调用蚀一槽（432 执行期实测，D25 留档）。已在 242 tracker 挂账；建议 codegen 或引擎加一致性检查。 | `vm/engine.rs` RET + divergences.md D25 |
| 432 | bool 压无类型 List 别名 | bool 值经 decode_i32 成 i32::MIN，与字符串负哨兵编码别名（D29 留档，242 挂账）；v2 以 1/0 整型承载规避。 | `vm/native.rs` + divergences.md D29 |
| auto-shell-057 ✅ | vue codegen 五类缺陷阻塞下游构建（2026-08-24 外部报告）—— **已修复：Plan 444（合并 master de76581ea，feat 9ff6a38b9，2026-08-24）**。修复后下游复现 `auto gen → npx vue-tsc` 0 错 + `pnpm run build` 绿，无需任何手工补丁；修复明细与新约定（回调通道/emits 名册/emit 桥/__vmOnly 桩/any 通道）见 docs/plans/444-vue-codegen-ash-shell-057.md。原文存档： | 下游 auto-shell ash-gui 项目 `auto gen` 重生成后 vue-tsc 余 13 错 / 5 类，**Vue/浏览器渲染目标整体不可构建**（merged VM 目标不受影响）。443(defineModel 降级)/435 P0-P1 合入后复测构成不变。五类：① 子组件回调 props 生成 `on_delete`（snake_case 且必填）而父级绑定发射 `onDelete` —— 名字永不相配（043 R4 修过 PascalCase emit，`Delete` 形态漏网，3 错）；② 可空变体字段模板访问 `cell.Tagged.text` 在 v-if 守卫内仍报 TS18049，需生成 `?.`（2 错）；③ 多参 msg 的 emit 签名生成 0 参 —— `Sort(int,int)`/`Filter(str)` 调用点报 TS2554（043 B-1 只修单 payload，2 错）；④ VM-only stdlib 泄漏进 JS：widget handler 内 `fs.read_dir`/`File.is_dir`/裸 `await complete` 原样输出到 .vue script 产出坏 JS，无 JS shim 时应降级或显式报错（4 错）；⑤ str 模型字段的动态变体读 `__sse_status.Failed`（裸串或 {"Failed":msg} 二态）报 TS2339，需 any 通道或契约化（1 错）。另 gen 模板缺口：`auto gen` 重写 package.json 丢 `@vueuse/core`（shadcn ui 组件引用它）；无引用残留 CodeEditor.vue 不清理。**复现**：`cd auto-shell/ash-gui/ash-gui-auto && auto gen && cd gen/front/vue && pnpm add @vueuse/core && rm src/components/CodeEditor.vue && npx vue-tsc`。详见 auto-shell DEBTS.md「Vue 产物构建引擎侧阻塞」（2026-08-24，含逐类行号）。 | `ui_gen/vue.rs` prop_to_ts_type/sub_widget_event_to_vue（①③）、模板 emit（②）、ts_adapter handler 转译（④⑤）；auto-shell docs/plans/057 §Phase 5 |

---

## 🟢 已知限制（设计决策，非 bug）

| 计划 | 类别 | 描述 | 引用 |
|------|------|------|------|
| 451 | 条件串非 Expr AST | enabled_if/checked_if 两种拼写（引号串/裸表达式）在 AST 内同为规范条件串而非 Expr AST——单一表示贯通 vm 求值（eval_condition_with）/vue 转译（convert_condition）/auto-atom 文件层的**有意取舍**；条件仅有 token 文法级校验，无类型级编译期校验。需要时可升级 Expr AST + 序列化器保持三端兼容。 | `ast/ui.rs` ActionEntry + `parser.rs` parse_actions_cond_attr |
| 451 | use 拾取深度不对称 | actions 模块拾取：vm 走 import_stmts **传递闭包**（孙模块命中），vue 的 collect_use_module_actions 只扫**一级** use 模块——actions 放孙模块时 vue 不拾取；需要时补递归即可。 | `lib.rs` run_file_dynamic_ui_inner vs `ui_gen/api.rs` collect_use_module_actions |
| 451 | plain 模式占位不合成 | `menubar {}`/`toolbar {}` 占位标签的组件树合成仅在 shadcn 模式（依赖 shadcn Menubar/Button 组件族）；`shadcn: off` 的 plain 模式保持占位直通（vm 合成不挑模式）。keydown 回退层不挑模式，任何模式都随声明发射。 | `ui_gen/vue.rs` node_to_html 占位特判（is_shadcn 守卫） |
| 451 | 模块拆分热重载 | use 引入模块的 actions 声明改动需 touch 宿主 .at 才触发热重载（DSL 源 mtime 只 watch 宿主文件）。 | `ui/action_config.rs` reload_action_config |
| 444 | 变体断言启发式 | ts_adapter 对「PascalCase 字段访问」一律补非空断言（`cell.Tagged!.text`）——api.ts 惯例下 PascalCase 可选字段即变体 payload（else 分支不变量保证非空），但用户对象若有运行时可空的 PascalCase 字段，`!` 会把编译期检查换成运行时 TypeError。 | `ui_gen/ts_adapter.rs` transpile_expr Dot 臂 |
| 444 | emits 名册缺省回退 | 父级回调事件名优先按子 emits 名册解析（同文件自动并入 + auto-man 跨文件预扫描）；无 名册 的驱动路径（如 cmd_vue 遗留 `auto vue` 入口）回退 prop 派生命名，透传习惯（`on_delete`↔子 `DeleteBlock`）在该路径仍断并只发 R044/R045 警告。ash-gui 走 auto-man 路径不受影响。 | `ui_gen/vue.rs sub_widget_callback_event_to_vue` |
| 444 | VM-only 桩为运行时报错 | `fs.*`/`File.*` 在 Vue/JS 目标降级为 `__vmOnly` 抛错桩（gen 期 R 警告 + 运行时显式 Error），非编译期拒绝——按「显式报错优于静默坏代码」裁定，VM/merged 目标不受影响。 | `ui_gen/ts_adapter.rs` Call 臂 + `__vmOnly` 发射 |
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
| 427 | 已知限制 | 📋 | a2r 编译级防线目前仅覆盖单一 golden(008)且为 #[ignore] 按需运行——文本对比 golden 仍无法自动捕获编译级回归(本次回归潜伏数日的主因)。全量 per-golden cargo build 因成本与镜像抖动暂不做;后续可扩展冒烟集至三恢复库产物或接入 parity harness 构建路径 | `crates/auto-lang/src/tests/a2r_tests.rs` a2r_compile_smoke_str_param_borrow | 2026-08-23 |
| 417-followup | 已知限制 | 🟢 链接期字符串重映射表仍为 Vec<u16>:池索引操作数已 u32 化(8482021e),remap_string_indices/remap_obj_indices 的操作数读写已同步 u32,但 old→new 映射表本身仍是 u16——依赖模块字符串去重后若合并池规模越过 65535 且发生非平凡重排,映射值会截断。当前各库规模远未达到;触发前提是单个程序(主模块+依赖)去重后池 >65535 条目 | `lib.rs` remap_string_indices(表类型注释处) | 2026-08-22 |
| 417-E3-P4 | 延期/已知限制 | ❤✅ 已实施(2026-08-22 同日补齐):codegen 新增 fn_type_param_bounds + check_generic_call_bounds,调用点按参数声明类型映射到带 bound 类型参数,实参静态类型(User/GenericInstance) 可确定未实现 bound 时编译期拒绝;保守策略——非 Ident 实参/类型未知/调用者自身泛型参数透传/非 spec 约束/类型不可解析均放行(留给运行时 CALL_SPEC 报错)。trait_vm_tests +3(违规拒绝/合法通过/透传不误报) | `vm/codegen.rs` check_generic_call_bounds | 2026-08-22 |
| 410 | Expr::Dot 不查符号 | `x = a.b` 中 `a` 未定义今天仍通过（Expr::Dot 不经 check_symbol；Bina(Op::Dot) 分支源码不可达）。Phase 2 立项时须一并纳入。 | `parser.rs check_symbol` |
| 381 | v1 限制 | Node::deserialize 只处理 props（标量字段），不含 kids（命名子块）。嵌套块反序列化留给 v2（需 field-level resolver）。覆盖 role_config 等全部用例（字段全是标量/数组）。 | `auto-val/src/de.rs:79` |

---

| 425 | ~~view 可选化拼写~~ ✅ 已修(2026-08-23):Damerau-Levenshtein 距离 1(含相邻换位)且后随 `{` 的标识符发 W0009 告警(`SuspiciousBlockKeyword`,lexer save/restore 单 token 前看);合法元素零误报,单测锁定 | `parser.rs` parse_widget_decl `_ =>` 分支 |
| 425 | 根序语义 | 根组件 = 源序首个 widget（component fn 糖化后不再追加在 widgets 之后），App 应前置；真实工程 app.at 均如此。 | `ui_gen/api.rs` + scenario-dialect spec |
| 425 | 参数类型擦除 | component fn 参数的自定义类型仍擦除为 any（fragment hint 映射保留，保糖化字节等价）；widget 拼写走 parse_type 全类型。 | `parser.rs` fragment_param_hint_to_type |
| 426 | setup 解释器侧 | ~~`setup {}` 仅落 a2vue~~ **✅ Plan 436 已落地(2026-08-23)**:a2r 显式报错止血(ui_gen/rust.rs 决策 1-A——Rust 目标无每实例 setup 槽位,1-B 生成留待需要时立项;trans/rust.rs 逻辑路径同款守卫);解释器 L1 单实例语义(bridge UI 场景解析加载 widget 源 + setup 前导在独立 VM run 执行一次、绑定入 `WidgetState.fields`、先于首视图)。残留边界:解释器 setup 前导不延续程序级作用域(每次 run 新 VM)、`.Init`/`.Destroy` 事件路由仍未实现、a2r 真 setup 支持未做——详见 docs/syntax.md 三相位×后端矩阵。 | `ui/interpreter/bridge.rs run_setup_preambles` + `ui_gen/rust.rs generate_rust` 守卫 |
| 436 | feature 组合破损(既有,review 发现) | `ui-interpreter`/`ui-headless` **单独**启用不编译:aura_view_builder(:1796 非测试引用 `iced_adapter::window_width()`、:5578 测试 import)、class.rs 测试(:1506 `set_dark_mode`)、mcp_server(:2118 `ui::iced::encode_payload`)无条件引用 ui-iced 门控项——可编译组合实际只有含 ui-iced 的全集(ui-iced 传递包含 ui-interpreter)。另:bridge 定向测试须带 ui-iced 系 feature 才被收集,无 feature 的 `--lib` 运行静默不含它们(436 review 曾因此误报"定向全绿")。 | `ui/mod.rs:53-57` 门控 + `Cargo.toml:42-45` |
| 426 | async setup | setup 内 `await` 编译期拒绝（async setup 需 Suspense 边界），单测锁定；支持另立任务。 | `parser.rs` stmt_expr_contains_await |
| 426 | 模板 refs 解包 | setup 绑定的 refs 标注字段在模板中不自动解包（普通对象嵌套 ref 的 Vue 语义）；script 侧已注入 .value。继承 composable facade 机制。 | `ui_gen/ts_adapter.rs` facade_ref_fields |
| 428 | ~~阻塞式文件对话框冻结 UI~~ ✅ 已修(2026-08-23 set_parent 落地):`dialog_open/save` 经 Win32 EnumWindows 就地发现主窗口 HWND(本进程最大可见顶层窗口,OnceLock 缓存),rfd `set_parent` 挂属主(raw-window-handle 0.6 直依赖,与 rfd/iced 同实例)——对话框永远浮于应用窗口之上,异常路径的属主禁用态泄漏随之消除。E2E 实证:对话框 GW_OWNER==主窗口、WM_CLOSE 干净取消(handler 完整收尾)、关闭后真实键盘恢复。**未做(残留风险低)**:pick_file 仍同步阻塞 UI 线程(模态期主窗口输入死属正常模态语义,代理事件仍泵,VM/MCP 活);若未来要求模态期主窗口可交互,需异步 handler 框架。 | `vm/native.rs dialog_parent` + 428 计划 §7.5 |
| 428 | 折叠区滚动按原文行数推进 | 折叠状态下滚轮仍按原始行数滚动——滚过大型折叠区需要"空滚"隐藏行数(视觉无变化但滚动条动)。修法:wheel 路径按 fold-map 跳跃隐藏段(unfold_y 已有反投影基建)。日常编辑器尺度(块 ≤ 数十行)无感知,大文件深折叠才可察觉。 | `core/mod.rs` 滚动路径 + `core/fold.rs` FoldMap |

| 445 | .Tick 跨轨语义分歧 | vue 轨=setInterval 级 running 门控，VM 轨=Plan 402 handler 无条件派发自决——应用需在 handler 内自查 running 兼顾两轨（024 已如此），平台级统一待后续裁定。 | `ui/iced/renderer.rs:6650` / `ui_gen/vue.rs:3228` |
| 445 | svgdoc 流式性能样本有限 | v1 SVG vs v2 canvas 裁决数据仅 12 点窗口/400ms 实测（2.49/s 无积压）；更大窗口/更高频（16ms/百点级）未测，v2 触发条件留待真实负载。 | `examples/ui/024-charts/tests/golden/stream_perf_sample.txt` |

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
| ~~418~~ | ~~menubar 估位偏移~~ ✅ 已退役(2026-08-27 复核归档批) | Plan 422 P2 menubar Popover 迁移已落地(估位/2000px catch 删除,矩阵 29/29);原条目即预期"422 落地后退役",现随 414/422 归档批执行。 | `renderer.rs menubar 面板合成`(已删) |
| 445 | 几何重算三份内联 | Init/.Tick/.Reset 三处 ~90 行同式几何重算（模块级 fn 不进 vue SFC 的 §0.6.E-3 约束所迫）；435 组件化收口后应收敛为单一渲染函数来源。 | `examples/ui/024-charts/src/front/app.at` |
| 447-① | VM is 值语义 let 绑定位返回 0 | `let r = is x {...}` 绑定位返回 0（函数尾位值语义正常）；最小复现 `fn main() { let r = is "test" { "test" -> "pass" else -> "fail" } print(r) }` 输出 0。部分② Phase 4 语料设计需覆盖该形态。 | `vm/codegen.rs` Expr::Is 值位（2026-08-25 探针整备时发现） |
| 447-① | VM 函数内嵌套 fn 静默失效 | `fn main() { fn inner() {...} inner() }` 调用无输出无报错（top-level fn 正常）；99_idiom_probe 探针一律顶层 fn 规避。 | `parser.rs`/`vm/codegen.rs` 嵌套 fn 路径 |
| 447-① | `struct` 非关键字误用报 E0201 | 源码写 `struct Frame {...}`（Auto 具名结构体声明实为 `type`）被当表达式解析，报 "Variable 'Val' is not defined" 名字解析错而非语法错，误导排查方向。 | `parser.rs check_symbol` |
| 444 | master 3 个红 a2r golden | plan-444 改 `write_is_arm_body` 后 `02_types_004_pointer`/`12_specs_007_box_fn`/`19_ownership_003_loopvar_owned_field` 在纯 master（d86615620 detached worktree 实证）失败，plan-447 合并前已存在，非 447 引入。plan-444 收账。 | `trans/rust.rs` write_is_arm_body + `test/a2r/` 对应组 |
| 447-① | 全量并行下偶发测试 | `benchmark_downcast_performance`/`cookbook_vm_tests::cb_file_read_lines` 在全量并行负载下偶发失败（单跑均通过，性能阈值/文件 IO 受负载影响），非回归。 | `perf_benchmark_tests.rs` / `cookbook_vm_tests.rs` |
| 418 | 工具栏图标偶发变暗 | 观察项：最终构建 3 实例采样亮度一致（231/114）不复现，疑锁屏期 DWM 降级帧假象——复现再查，不主动处理。 | `041 toolbar svg 渲染` |
| 418 | VM int 推断显示坑 | `File.write_text` 返回值在 handler 内 let 绑定后 `.str()` 显示类型区间（"0-2147483647"）而非字节数——041 ActSave 曾绕过（改语句调用丢弃返回值）；根因（int 字面量区间推断/str 化路径）未查，§7.4 声称"另立债务"但一直未登记，2026-08-23 finish-plan 复审补登。 | `041 src/front/app.at` ActSave + VM 推断路径 |
| 451 复审 | master T10 热重载菜单开关竞态 | 041 desktop_mcp `T10 hot-reloaded menu item appears` 在 master（27124c4b8/2f0381c42 实测 48/1）确定性失败、独立复现则间歇——reload 响应正确（15 actions/5 menus）、新菜单进 menubar，但点击其 trigger 后面板未开（既有菜单正常，第二次点击可开）。plan-451 基线（895b7d413+451 改动）当日 50/0，嫌疑 = 895b7d413 之后触碰 renderer.rs/dynamic.rs 的 27124c4b8 或 plan-041a 合并（7d5f457b8），未定位到提交级。 | `examples/ui/041-auto-edit/tests/desktop_mcp.py` T10 §10.3 + `ui/iced/renderer.rs` __menubar_toggle |
| 451 复审 | ui_snapshots 双快照预存过期 | `snapshot_editor`/`snapshot_sidebar` 在 master（27124c4b8）即失败（stash plan-451 全部改动实测 EditorPanel 输出同为 6644B，非 451 引入）——015-notes 系 SFC 字节漂移未随改动刷新快照；待认领刷新（`cargo insta` accept）。 | `tests/snapshots/ui_snapshots__editor.snap` + `ui_snapshots__sidebar.snap` |
| 417-E2 | uninit var 收紧 | `var x T` 无初始化器：VM 合法（Nil 起始）但 a2r 发射 `let mut x: i64 = None;` 死形状（任何类型，非关联类型特有）。2026-08-22 调查：全仓 .at 仅 2 处使用（a2c 11_methods/003，同文件 2 行）；语言规范从未文档化该形态（spec 示例全带初始化器）；bare `var x` 全仓 0 处；auto-man 生成器无此模板。**方向（用户倾向）：parser 要求必须初始化**——代价远小于先前评估，待办：①REPL 单条声明是否豁免需定夺；②上层仓（auto-ai/rust-workspace）.at 源合并前复扫；③a2c 测试改 2 行；④a2r 删 `= None` 分支。 | `parser.rs var/let 声明` + `trans/rust.rs store()` |
| 417-E2 | has-Spec 关联类型绑定 | `has f Type for Spec` 子句只接受类型列表，无法携带 `Item=int` 命名绑定——spec 声明关联类型而实现者走 has 委托时 a2r 产物缺 `type Item = ...;` 编不过（VM 动态不受影响）。缺的是**语法设计决策**（绑定形态如何挂到 has 子句），非代码缺口；现无用例。语法拍板后实现量小（parser has 臂 peek 分支 + 委托转发处替换）。 | `parser.rs has 子句` + `docs/handoff-E2-followup-assoc-body-refs.md` F2 |
| 417-E2 | trait_checker 参数类型比对 | check_conformance 只比对参数个数不比对类型（代码内既有 TODO）。收紧=全量现存 spec 实现重新校准（str/String、i32/i64 映射边界的兼容规则需拿 stdlib/parity/上层项目实测定表），独立立项，勿混特性批。 | `trait_checker.rs check_conformance` |
| 417-E2 | 关联类型 bound 语法 | `AssociatedType.bound: Option<Type>` 字段已预留（Display/atom 已留位），parser 不接受 `type Item has Bound`。零用例需求（Rust `type Item: Bound` 亦低频），YAGNI，先有需求再设计语法。 | `ast/spec.rs AssociatedType` |
| 409 | CodeBlock/PreviewCard 混合模式 | widgets-gallery 的 CodeBlock/PreviewCard 仍为「Auto 声明壳 + codegen 硬编码 UI」（generate_codeblock_html/generate_previewcard_html），改纯 Auto widget 需先验证 codegen 转译浏览器 API 调用的能力，未立项。VM 侧识别已由 §10 组 E 补上。 | `ui_gen/vue.rs generate_codeblock_html` |
| 432-A4 | parser.rs 拆分 | parser.rs 17.7k 行单文件——core/ui 混杂，理想拆分 `parser/{core,ui}.rs`。 | `parser.rs`（431 A4 #1） |
| 432-A4 | 巨型 emit/run 分派外提 | codegen/engine 的巨型 emit/run 函数内嵌 UI 段——理想按 dispatch 表外提（v2 的指令 List 直译循环验证了该形态可行）。 | `vm/codegen.rs` / `vm/engine.rs`（431 A4 #2） |
| 432-A4 | u128 族访问器目录化 | `Duration`/`Instant` 等手写臂已由 plan-430 生成段接管，引擎侧残留 u128 族访问器待目录化。 | `vm/engine.rs`（431 A4 #3） |
| 432-A4 | native_catalog 分文件 | native_catalog 521 条单数组——核心/UI 分文件。 | `vm/native_catalog.rs`（431 A4 #4） |
| 432-A4 | SHL/SHR 语法缺口 | opcode 仅声明 6 条中 SHL/SHR 是实际语法缺口（429-B3），实现后 431 处置表需更新。 | `vm/opcode.rs`（431 A4 #5） |
| 433 | VM 枚举载荷跨函数传参丢标签 | VM 实证(probe20/22/23):enum 载荷值为运行期计算(拼接/算术)时,跨函数传参后 is-match 失败(NONE/空串/i32 哨兵泄漏);字面量构造或结构体载体不受影响。433 以 Val 判别器结构体绕过(divergences D34);根治需查 enum 值的参数编组(payload 池绑定)。 | `vm/native.rs`/编组路径;probe 复现件 tmp/p433/probe20-23.at(已失,可由 divergences D34 描述重建) |
| 433 | a2r is-绑定变量无类型跟踪 | is-pattern 绑定的载荷变量不进 local_var_types——后续 `.get`/字段解析失效(433 以带标注局部+辅助函数绕过)。 | `trans/rust.rs` local_var_types + enum_tuple_field_types 联动 |
| 433 | a2r 后处理归约 `.to_string().as_str()`→`.as_str()` | 17511 行的文本归约会吃掉 `.str()` 实参的正确借用链,数字接收者残留 `i64.as_str()` E0599(433 以 .at 侧提升局部绕过,D36-③);数字字面量剥离 regex 只兜 `(\d+)\.as_str` 形态。 | `trans/rust.rs:17511` + `fix_numeric_get_as_str` |
| ~~433~~ | ~~merge 模式 bootstrap 自测 main 遗留~~ ✅ 已清退(2026-08-24 plan-434) | 自测 main 追加块已移除(无测试消费其输出);v1 lib 仍由 lib-legacy 封存与 99_bootstrap #[ignore] 测试保留,其余 v1 路径(regex 族对 v2 无害直通)暂不动。 | `trans/rust.rs transpile_rust_project_merged` |
| ~~434~~ | ~~主 a2r 对 a2r.at/lexer.at 新模式的发射缺口~~ ✅ 已修(2026-08-24 plan-434 收官轮) | 242 #18 已修复:链式类型推断 + 赋值位 auto-clone 镜像 + str 型 to_string 家族;② 已回归整目录七文件,五方矩阵全绿,golden 零回归。 | 242-a2r-feature-gap-tracker.md #18(已修复) |
| 434 | AA2R golden 覆盖不完整 | S1/S2 余量:01/03 组大部分字节级一致;02/04/05 组部分;06+(is-match/闭包/spec/use/泛型声明)未移植;S2(use.rust 直通/dep/Cargo.toml/a2r_std_used 完整版)未做(仅 math 内建 max/min + a2r_std_used 头块)。差异定位 divergences.md D40。 | `auto/lib/a2r.at` Missing 节 |
| 434 | AA2R 输出与主 a2r 的格式残留差异 | ⑤ 产物与 ② 文本对比存在可解释级差异(mut-参数签名的绑定可变位、逐文件分隔注释、尾随空行等,行为等价;详见 divergences.md D40)。 | `auto/lib/a2r.at` DIVERGE 节 |
| ~~429-434~~ | ~~复审发现:aavm2 六道闸门全部挂 `test-vm-files` feature,六个 CI workflow 零覆盖~~ ✅ 已修(2026-08-25 vm-files-ci.yml) | 新增 .github/workflows/vm-files-ci.yml:闸门(M1-M5+里程碑+compile corpus,--include-ignored)/VM goldens+ffi_dual(含014)/conformance 三层必须绿,cookbook 非阻断收集信号;stable+nightly 双工具链(013 方法包管线真实执行);本地逐命令验证过(10/26/36 全绿)。触发面:crates/auto-lang|auto-lib|auto-cache|shim-metadata。 | `.github/workflows/vm-files-ci.yml` |
| ~~429-434~~ | ~~复审发现:19_rust_std VM goldens 全部 `#[ignore]`,430 已迁 std 臂无 VM 路径防回归网~~ ✅ 已修(2026-08-25 ffi_dual_014 补网) | ffi_dual_014_std_generated_segment 落地:Vec 14 臂行为断言+Duration u64 宽度回归(5e9 秒)+as_secs_f64+PathBuf.from;19_rust_std 10 个陈旧 ignore 一并解除(实测全过)。**残余**:uuid/semver/csv 端到端仍无自动化(建议 CI 定时 job);Vec.insert 源码参数序为(值,索引)的生成段 ABI 约定已注释留档。 | `test/ffi_dual/014_std_generated_segment/`、`vm_file_tests.rs` 19_rust_std 区块 |
| ~~ffi_dual_014 发现~~ | ~~跨 VM 运行状态毒化:String.from 返回值 tag 丢失~~ ✅ 已修(2026-08-25 fix/vm-string-route-pollution) | **根因**(与初判不同,非 tag 丢失而是编译路由翻转):codegen"已有 native 优先"启发式 + BIGVM_NATIVES 惰性注册(进程级全局,查询即注册)——同进程任何先前程序用过原生 String API(如 dstr 测试的 String.from)后,use.rust 的 String.from 被劫持到 auto.str.from native 路径,print 出裸堆 ID。**修法**:类型导入(首字母大写 key)恒走 dispatch 3000,劫持仅限 crate/模块导入(toml.parse/json.parse);ffi_dual_015 同测试内双跑(原生→use.rust)做确定性回归,014 加回 String.from 断言;全量 3293 过零新增。**遗留观察**:**✅ 半解耦已落地(2026-08-25 fix/native-registry-peek)**:新增 `peek_qualified` 纯读探测(查注册表+静态固定 ID 表,不写入),codegen 七处路由决策点(.len 检查/math/int/bitwise 三处/mono-dispatch 三处/is_native)全部转换——决策从此不改变注册表状态。**例外**:rust_native_map 的 has_existing 检查保留注册表-only 语义(auto.json.parse 在静态表但 CALL_NAT 编组与 dispatch 3000 不同,cb_encoding_json 实证 peek 会劫持出裸堆 ID;toml.parse 启动即注册不受影响)。emit/调用期解析点(要发 CALL_NAT)保留 resolve_qualified,注册副作用无害且预期。 | `vm/codegen.rs` rust_native_map 分支 + `vm/native_registry.rs` |
| ~~430~~ | ~~复审发现:compile_dep_methods 错误被静默吞掉(`.ok().flatten()` 无 else/日志)~~ ✅ 已修(2026-08-25 plan-430-fixes) | 调用点改 match 三分支:成功注册/nightly 降级 info/失败 warn 含完整错误串,降级走自由函数路径但不失语。 | `crates/auto-lang/src/compile.rs` compile_dep_methods 调用点 |
| ~~430~~ | ~~复审发现:方法包版本指纹用声明版本(`uuid = "1"` 兜底字面量)而非解析版本,缓存快路径不重解析~~ ✅ 已修(2026-08-25 plan-430-fixes) | `resolved_crate_version`(cargo metadata --locked)取真实版本入 PackMeta;缓存快路径核对 manifest 版本 vs 当前解析版本,不一致即重建;解析失败按缓存接受保持降级。 | `crates/auto-cache/src/methods_pack.rs` |
| ~~430~~ | ~~复审发现:rustc 剔环无轮次上限(430-f1 报告称"至多 4 轮"与代码不符)且肇事符号 `starts_with` 前缀匹配会误伤同名前缀方法~~ ✅ 已修(2026-08-25 plan-430-fixes) | `MAX_BUILD_ATTEMPTS=5`(首建+4 重试,与报告口径对齐);`plan_export_symbol` 提取为 emit_cdylib 公共函数,剔环按完整导出名精确匹配(单测覆盖 newest/new、set/set_label 不误伤)。 | `methods_pack.rs` + `emit_cdylib.rs plan_export_symbol` |
| ~~430~~ | ~~复审发现:泛型自由函数不做 generic 过滤,Ty::Generic 映射 RetPlan::Void → 假 'v' 签名进 manifest 被 resolve_signature 采信~~ ✅ 已修(2026-08-25 plan-430-fixes) | emit_pack_parts 对 free_fns 过滤 `generic`,manifest/指纹/signatures.json 三处一致排除;跳过项在 signatures.json skips 留痕 + log::warn(shim-metadata 补 log 依赖)。 | `emit_cdylib.rs` fingerprint_parts/emit_pack_parts |
| 430 | 复审发现:as_millis/as_micros/as_nanos 手写臂仍是 u128→i32 有损截断(与已修的 as_secs 族对照),未标可疑 | B3"可疑臂逐条裁决"遗留;使用方应知毫秒值 >i32::MAX 即溢出。 | `vm/ffi/stdlib.rs:6927-6929` |
| 432 | 复审发现:bool 哨兵防护不完整——shim_list_push 有 is_bool 规范化,set/insert 路径直接 decode_i32 | bool 元素经 set/insert 仍变 i32::MIN 别名(v2 侧以 1/0 承载规避不受影响);宿主侧待补。 | `vm/native.rs` set≈2165/insert≈2227 |
| 432 | 复审发现:engine.at ev_add 双 int 分支偏置不对称(直接 a.i+b.i 无 dec/enc,当前不可达);codegen.at hex4 编址 16 位静默截断(>0xffff dump 错无诊断) | 前者若 str.cat 发射条件放宽即触发错值,至少补警示注释;后者补溢出诊断。 | `engine.at:121-133` / `codegen.at:864-876` |
| 330 | VM 内省三件套缺位 | `auto debug --agent`(Plan 199)的 JSON state 只含 stack/call_stack/locals/registers,无 globals/heap-objects/symbols dump——排查全局变量污染/符号冲突/堆对象泄漏无工具(330 原 Phase 2 设计 vm/introspection.rs)。无当前消费者,330 已归档,设计沉淀 design/14。 | `vm/debugger.rs` AgentDebugState + `docs/design/14-developer-tools.md` |
| 330 | trace 无 CLI/env 暴露 | TraceCollector(Plan 199 P5,vm/trace.rs JSONL)仅引擎内集成,无 `--trace` CLI 开关或 AUTO_VM_TRACE 环境变量入口;330 的"handler 超步数/深度阈值自动告警递归"静态诊断也未做。 | `vm/trace.rs` + `crates/auto/src/main.rs` |
| 413 | code_editor 人工验收清单未跑 | 微软拼音 IME 实机输入、150% DPI 行号清晰度、Linux(X11/Wayland)复验、TESTING.md 交互行为(三击/Ctrl+词跳转/滚动条拖拽/vi 模式)——需实机人工;`@codemirror` 深化(主题/搜索 UI/多光标)属后续增强。 | code_editor TESTING.md + 041/gallery |
| 421 | vue code_editor natives 桥接 | `code_editor_*` VM natives 仅 iced 端有全局 registry,vue 端等价能力由 cursor payload 事件承担(natives 桥接属另计划);vitest 未搭(条件项,已降级手验清单);oncursor vue playground 实机验证未跑。 | `ui_gen/vue.rs` + scaffolded 工程 |
| 422 | popover 覆盖边界 | rust 模式 codegen 未覆盖 popover 标签(`ui_gen/rust.rs`)——VM/iced 语义完整,vue 走 shadcn 映射;子菜单 z 序嵌套(面板内再弹)远期不承诺;两项实机人工验收未跑(041 右键菜单落点/gallery popover 开合,MCP 无鼠标注入无法自动化)。 | `ui_gen/rust.rs` + 041 + gallery |
| 414 | auto-edit UX Phase B 族 | toolbar 右对齐被 VM Row 渲染器限制阻塞(§7.2,解除后一行启用);真折叠 Phase B(fill_raw 整缓冲绘制无法跳行,倾向 core 自绘,单独立项);action 声明块 Phase B(§6.1,parser/view/aura/a2r 四端改造)。menubar overlay 一项已由 Plan 422 P2 解除。 | `ui/iced/renderer.rs` + 041 |
| 423 | RC 安全挂账 ×2 | ARRAY_LEN 弹栈不释放引用操作数(每调用泄漏一个份额;持久列表无感,临时列表会长存);裸 i32 堆 id 编码与 TAG_OBJECT 双轨并存,pop 侧无法区分"带份额压栈"与"历史裸压",全量收口需一次性迁移(419 遗留议题)。 | `vm/engine.rs` ARRAY_LEN + 堆 id 编码 |
| 449 | vm 组件渲染三缺口 | 回调 props 退化/快照组件子树不可见/片段参数化条件不求值(§3.1-3.3)——修好后 041 tab 条/确认弹层可继续组件化;vue 侧 action 配置(vue codegen 全局 keydown+menubar/toolbar 合成)另立计划。 | `ui/render*` + 041 tab_bar/confirm_dialog 设计 |
| 449 | VM 字节码越界读 bug | store handler + `code_editor_set_text`(疑及同族 set 类内建)编译路径产出越界读,根因在 handler_codegen/vm codegen 对 store decl 的合成;041 目前以根 handler 规避,值得专项修复。 | `vm/handler_codegen.rs` + 041 |

---

## ⏸ 延期（finish-plan 登记的未竟项，Type=延期）

| 计划 | 类别 | 严重度 | 描述 | 根因/理由 | 引用 | 登记日 |
|------|------|--------|------|-----------|------|--------|
| 463 | 环境限制 | 🟡 | worktree 内 `cargo tb`/book_listing_tests 全数不可运行（61 项"失败"为环境假象） | book_listing_tests 经 `CARGO_MANIFEST_DIR/../../../book` 读仓库外兄弟目录 `D:\autostack\book`，该相对路径从 worktree 解析必然落空（结构性，非代码问题）。缓解：`.worktrees/book → D:\autostack\book` junction（复审时已建）；长期可考虑 book 路径支持 env 覆盖 | docs/plans/463-desktop-shell-auto-arrange.md §10.2 | 2026-08-28 |
| 398 | 下游任务 | 🟢 | ash-gui-native M0.5 测试骨架（conftest/desktop_mcp/test_smoke）+ M1 in-process 后端 | 属 auto-shell 仓的下游任务，本仓计划仅负责 VM 侧修复（已完成） | docs/plans/archive/398-*.md §14.3/§14.4 | 2026-08-20 |
| 408 | 功能缺口 | 🟢 | P5-4：纯 module fn 文件不被 codegen（ui_gen/api.rs:456 报错） | 低优先 + 既有 workaround（塞进 widget/store 文件）；根治需先设计 codegen 入口扩展 | docs/plans/archive/408-*.md §11 P5-4 | 2026-08-20 |
| 406 | 审计矩阵 | 🟢 | 全量 nanbox 生产者-消费者类型配对审计矩阵（docs/audit/vm-type-audit.md）未产出 | 立项驱动的 4 个目标 bug 已全部由审计批次 A4/B4 根治，矩阵价值让位 | docs/plans/archive/406-*.md Phase 1 | 2026-08-20 |

*最后更新：2026-08-27（复核归档批:413/414/421/422/423/449 六计划归档,遗留登记 7 条(413 人工验收/421 natives 桥接/422 popover 边界/414 Phase B 族/423 RC 安全×2/449 组件三缺口+越界读 bug),418 menubar 估位条目随 422 P2 落地退役;Plan 330 归档裁定：核心诉求被 199+MCP 工具族取代,剩余缺口登记 2 条——VM 内省三件套/trace 无 CLI 暴露;设计沉淀 design/14;Plan 332 同日改写聚焦 Serialize 方向;2026-08-25：plan-447 部分① 收尾登记 5 条：is 值语义 let 位返回 0/嵌套 fn 静默失效/struct 误用报 E0201/plan-444 3 红 golden/并行偶发测试；同日早前：vm-files-ci.yml 落地:六道闸门+goldens+conformance 接入 CI;ffi_dual_014 补 std 臂 VM 回归网+19_rust_std 10 ignore 解除;plan-430-fixes 清偿复审高危 4 条:compile_dep_methods 吞错/指纹声明版本/剔环上限+前缀误伤/泛型自由函数假签名——全部 ✅ 并补单测;aavm 系列 429-434 复审+归档:新增复审条目 9 条;Plan 434 AA2R 合并入库;Plan 444 修复 auto-shell-057;Plan 433 登记 4 条;2026-08-22 Plan 417-E3;2026-08-20 归档复审）*
