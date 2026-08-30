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
| 446 | 数据损坏（已定界 2026-08-28 晚:修在下游） | os-config roles 实体 soul sidecar 转义翻倍损坏:磁盘 `~/.config/autoos/roles/assistant.soul.md` = 655K 连续反斜杠(08-25 产生,2^k 增长形态=每轮保存翻倍);经 HTTP 文本管线进 UI 后快照侧再翻倍至 1.31M(线上转义形式未还原)。**翻倍点已定界**:下游 fetch 成语——os-config back/api.at:1052-1064 fetchEntityFlat 对原始响应文本调json.get(t,"sidecar"),我方 shim(stdlib.rs:2597,serde parse+to_string)对字符串值返回重转义字面量(字面量语义为文本工具链承载性设计,fragment 再喂 json.keys/len/get 依赖它,ffi_dual 字节级锁定,不可改),unquote 剥引号不解转义→内存 2N;保存侧 api.at:750 quote_json 忠实转义写盘→磁盘翻倍,2^k 循环。修在下游:字符串内容取点 parse-first(json.parse(json.get(t,...)));同成语 fm_name/fm_description/fm_body等为潜伏同款(含反斜杠/引号/换行内容均翻倍,非仅 soul.md);本仓 Json.get 语义不动。该损坏数据即 446-U3 渲染冻结的触发物(§S U3);renderer 侧已降级防冻(>64KB 只读预览),数据本体待下游恢复+管线修复。**✅ 已解决(2026-08-29 下游结算)**:修复落地(parse-first 四取点 sidecar/fm_name/fm_description/fm_body,下游提交 8a7b85f;`value` 字段以 literal 直插 PUT payload 属合法传输契约,有意不动)+数据本体重写(soul.md 639B 干净 markdown,原文无 .bak/无种子不可恢复);修后 save 循环磁盘尺寸恒定(639B,修前同循环 ×2),U3 详情截图 1587ms(<2s 门),e2e 双门禁绿。详见 docs/plans/reports/446-downstream-settlement.md §二 | docs/plans/archive/446-vm-backend-os-config-field-report.md §S U3 旁注;`~/.config/autoos/roles/assistant.soul.md` |
| 446 | 渲染回归（降级观察 2026-08-29:最新二进制未复现） | os-config collection 模块体像素不渲染:Roles/Skills 主区空白(仅页标题),AI Daemon 正常;AbstractView/快照树完整(实体按钮在、可 press)、截图通道活——**视图→iced 布局/绘制链路塌陷**,非快照假象。css-era 基线(tmp/parity/css/03-roles.png)有内容 = 区间回归;已排除:顶层类列表去除无效、sidecar textarea 移除无效。视觉取证与二分证据见 446 §S U3 残余③;独立于截图通道,阻塞 os-config vm 轨像素 parity。**降级观察(2026-08-29 下游结算)**:c83435764 二进制下 03/04 未复现(快照树完整+像素有内容;vue-vs-vm 全视图对拍 2.11-4.99% 与已知 L2/L3 量级一致);css-era 原口径基线 PNG 不在库,一次未复现不足以关闭 P0 候选——降级观察,复发再启立项。详见 docs/plans/reports/446-downstream-settlement.md §四 | docs/plans/archive/446-vm-backend-os-config-field-report.md §S U3 残余③;`auto/src/front/collection_browser.at` view 主体 |
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
| p1(plan011③) | 双同名 `ObjectData` 结构陷阱 | `vm/types.rs:148` 与 `vm/object_data.rs:9` 各有一个字段同构的 `ObjectData`，type_tag 同为 `TypeTag::ObjectData`；engine CREATE_OBJ 等全仓 25 处只分配 `types::ObjectData`，`object_data::ObjectData` **零分配点**（孤儿）。downcast/读路径认错即静默 miss 无诊断——实例：p1 canary [P011] 探针曾因此 10 轮「未知堆变体」脱靶（2026-08-29 定性，随 ba0416d15 修正）。根治：删除 `object_data.rs` 孤儿或统一 re-export，顺带清 `use` 面。 | `vm/types.rs:148` vs `vm/object_data.rs:9`；downcast 面 `grep -rn "ObjectData" crates/auto-lang/src --include=*.rs` |
| p1(plan011④) ✅ | `__json_object` 浮点字段消费误码（CALL_SPEC 数学分发 nanbox 化石）—— **已修复：Plan 474（plan-474-dev d55f98b0e + 85a0600b9，2026-08-29 复审通过待折叠）**。原症状：VM 轨 JSON 浮点字段消费出错乱值（os-config 现场 54.16 → -1073741824，`.floor()` 异常）。**根因（定性修正）**：不在写入/读取侧，而在 engine.rs CALL_SPEC 一元/二元数学分支的 i32 栈约定化石——`read_i32(receiver_pos)/push_i32/pop_i32` 把裸 f64（encode_f64=原始位）读成低 32 位（54.16 → 0xE147AE14 → -515396076）、TAG_F32 读成 payload 位型（54.16f32 → 0x4258A3D7 → 1113105367）；二元分支另有参数序倒置（rust_fn 宏逆序弹参，receiver 拷贝压顶致 powf(54.16,2.0) 算成 2.0^54.16）。本条目早前「-2.0f32 哨兵注入」假设已被证伪（值相关位错读）。修复=接收者/结果 NanoValue 透传 + 二元原地调用；handler 内 `.floor()` 编译为 CALL_SPEC 而脚本走 CALL_NAT，故纯脚本测试长期掩盖。回归：`tests/vm_json_float_read_tests.rs` 三层载具（脚本 13 用例/位级 GET_FIELD/widget handler 数学族）。**写入侧三级排除结论依然有效**（stdlib Number 分支/字段直存/GET_FIELD Double 臂均正确）。下游：os-config 撤 T12 整数化绕法恢复原始数值字段（其 plan011 待澄清#5④，修复折叠后执行）。 **同日后续清偿（474 待澄清#3/#4/#5，master 直修）**：BUILD_FSTR 编译期标签盲信（f-string `${x}` 插值裸 f64 按 i32 解码，④ 同族）/print TAG_BOOL 形态 1|0→true|false（三处哨兵特判摘除，真整数 i32::MIN 误显顺修）/decode_tagged_nv+GET_FIELD Nil 读写的 null 往返闭环。 | `engine.rs` CALL_SPEC 数学分支（修前 :5996-6026）；`convert.rs:104`（f64 pop）；回归 `tests/vm_json_float_read_tests.rs`；auto-os-config `docs/plans/011-daemon-auto-back.md` 待澄清#5④ |
| 474-旁支 | aavm bool 显示 parity（值模型级） | 宿主 print(bool) 已改 true/false（Plan 474 待澄清#3），但 aavm（auto/lib/engine.at 自举 VM）的 Val 枚举只有 VInt/VStr/VArr/VInst，bool 以 1/0 整型承载——裸 bool print 两侧形态分歧。根治需给 aavm 加 VBool 载体（值模型级改动，牵动全部 match 分派），非小修。过渡：corpus_m4 的 b13/b14/b19 已改写为「bool 求值+条件分支」形态保语义覆盖（m5 闸门绿）。 | `auto/lib/engine.at`（Val 枚举/PushBool/print）；`crates/auto-lang/test/vm/aavm2/corpus_m4/b1[349]*.at` |
| auto-shell-057 ✅ | vue codegen 五类缺陷阻塞下游构建（2026-08-24 外部报告）—— **已修复：Plan 444（合并 master de76581ea，feat 9ff6a38b9，2026-08-24）**。修复后下游复现 `auto gen → npx vue-tsc` 0 错 + `pnpm run build` 绿，无需任何手工补丁；修复明细与新约定（回调通道/emits 名册/emit 桥/__vmOnly 桩/any 通道）见 docs/plans/444-vue-codegen-ash-shell-057.md。原文存档： | 下游 auto-shell ash-gui 项目 `auto gen` 重生成后 vue-tsc 余 13 错 / 5 类，**Vue/浏览器渲染目标整体不可构建**（merged VM 目标不受影响）。443(defineModel 降级)/435 P0-P1 合入后复测构成不变。五类：① 子组件回调 props 生成 `on_delete`（snake_case 且必填）而父级绑定发射 `onDelete` —— 名字永不相配（043 R4 修过 PascalCase emit，`Delete` 形态漏网，3 错）；② 可空变体字段模板访问 `cell.Tagged.text` 在 v-if 守卫内仍报 TS18049，需生成 `?.`（2 错）；③ 多参 msg 的 emit 签名生成 0 参 —— `Sort(int,int)`/`Filter(str)` 调用点报 TS2554（043 B-1 只修单 payload，2 错）；④ VM-only stdlib 泄漏进 JS：widget handler 内 `fs.read_dir`/`File.is_dir`/裸 `await complete` 原样输出到 .vue script 产出坏 JS，无 JS shim 时应降级或显式报错（4 错）；⑤ str 模型字段的动态变体读 `__sse_status.Failed`（裸串或 {"Failed":msg} 二态）报 TS2339，需 any 通道或契约化（1 错）。另 gen 模板缺口：`auto gen` 重写 package.json 丢 `@vueuse/core`（shadcn ui 组件引用它）；无引用残留 CodeEditor.vue 不清理。**复现**：`cd auto-shell/ash-gui/ash-gui-auto && auto gen && cd gen/front/vue && pnpm add @vueuse/core && rm src/components/CodeEditor.vue && npx vue-tsc`。详见 auto-shell DEBTS.md「Vue 产物构建引擎侧阻塞」（2026-08-24，含逐类行号）。 | `ui_gen/vue.rs` prop_to_ts_type/sub_widget_event_to_vue（①③）、模板 emit（②）、ts_adapter handler 转译（④⑤）；auto-shell docs/plans/057 §Phase 5 |
| 446-R1 | codegen applyAccent 撞名（TS2440，2026-08-29 下游回传） | Plan 409§8/458:store 拥有 `accent_color`/`dark_mode` 字段时 vue codegen 注入内嵌 applyAccent 助手（ACCENT_PALETTE+watch 同步行）；若该 store 同时 `use back.api:` 导入同名 fn，生成 TS 即 TS2440（import vs 局部声明冲突，vue-tsc 红灯）。下游 auto-os-config 已将 back.api 的 applyAccent 改名 saveAccent（与 loadAccent 对称）规避。修法：注入前检测 use 清单撞名并避开/告警。 | `ui_gen/vue.rs` store 注入块（~3206 ACCENT_PALETTE_JS）+ widget 路径（~2575）；docs/plans/reports/446-downstream-settlement.md §五.1 |
| 484 | 包组件 Init 内 prop 字符串比较破坏 codegen（静默） | official 包组件（`use {package}` 路径）Init handler 内出现 `if curve == "monotone"` 类字符串 prop 等值比较时,**整个子组件 Init 静默失效**——全部几何输出回落 model 默认值,零诊断输出。实证:484 M1 bisect（干净 HEAD 通过,仅加 prop 比较即崩,git stash 双向验证）。同形态 `use widget:` 导入路径不受影响（013-todo todo_list.at 的 model 比较/带参 handler 均正常）——**包加载链特有**（lib.rs P4-4/D13 child_decls 单 VM 编译 vs use-widget 编译链的 handler codegen 差异）。绕开（已落地）:Init 内双算双存变体（segs/segsM、segs/segsS）,view 侧按 prop 选边（view 内 prop 比较正常,如 `if .axis == "auto"`）。回归锚:cargo test -p auto-lang --features ui-iced gallery_chart_components + plan484_chart_component_tests | `components/{line,bar,area}_chart.at` 头注;`crates/auto-lang/src/lib.rs` P4-4/D13 块;plan 484 M1 记录 |
| 484 | 包子组件带参 msg 声明破坏整包编译（静默） | official 包组件 `msg { Init, Hover(int) }`（带参 msg 声明）使整包加载后所有子组件 Init 失效（静默,同上形态,零诊断）。去掉 msg 参数声明、仅保留裸带参 handler `.Hover(i) -> {}` 一切正常——事件经 DynamicMessage::Typed args → encode/decode_payload → call_handler_for 走通（mouse-area hover 已实证）。对照:`use widget:` 路径 013-todo todo_list.at `msg { ..., ToggleTodo(int), ... }` 正常——同上,包路径特有。绕开（已落地）:包组件一律 `msg { Init }`,带参 handler 裸挂。根治:查 load_package → child_decls → 单 VM 编译链对 messages 的处理（疑 codegen 为带参 msg 生成的桩/表在包路径下错配,与上条可能同根）。回归锚同上 | `components/*_chart.at`（msg 均为裸 `msg { Init }` + 裸挂 handler）;`ui/dynamic.rs decode_payload`;plan 484 M1 记录 |
| 484 | tooltip 逐 index 锚点定位（降级为固定右上角） | chart tooltip 的锚点坐标需要动态像素定位（`top-[{.tipY}px]` 类插值类名/StyleBinding 动态值),vue 侧 f-string `{}` 形式在 handler 生成中不插值、tailwind 任意值类需 JIT 扫描源码——两轨均不可靠。v1 降级:tooltip 固定于绘图区右上,内容随 hover 索引变化;逐 index 锚定待 StyleBinding 动态值或 v2 canvas。回归:plan484 冒烟 + charts-gallery 目检 | `components/*_chart.at` tooltip col;plan 484 后续复审目检记录 |
| 484 | f-string 含字面量 `[`/`]` 时 `${}` 插值破坏组件编译（静默） | `f"w-[${slot}px] h-full"`（dollar-brace 插值 + 字面量方括号）使包组件整体失效（静默形态同上）;同语义 `f"w-[{slot}px] h-full"`（brace 插值）正常。437 时代 donut `bg-[{color}]` 一直用 brace 形式故未触雷。疑点:lexer f-string 模式对 `${` 的 fstr_expr 消费与字面量 `[` 的交互（lexer.rs:629/724 两处 FStrNote 分支）。绕开（已落地）:含字面量 `[]` 的 f-string 一律用 `{}` 插值（bar/line/area band 样式 + tooltip 锚点 style 全部改造）。根治:f-string lexer 最小复现单测（`f"w-[${x}px]"` 解析层即可触发,无需 VM）。回归锚同上 | `components/{line,bar,area}_chart.at`（band 样式/tooltip style）;`crates/auto-lang/src/lexer.rs:615-745`;plan 484 M1 记录 |
| 446-R2 | merged 模式链接面双 api.at 无诊断（2026-08-29 下游回传） | back.api 符号链接以**外部 back 工程**（如 auto-os-config-back/api.at）的导出清单为准，in-project auto/src/back/api.at 只供实现体——改名/增删 fn 须两份同步，只改一侧即 boot 崩 `Undefined symbol: api.X in module App`，报错不指向第二份文件（下游实测定位成本高）。修法：诊断信息补"检查外部 back 的 api.at 导出清单"提示（或文档化双文件契约）。 | VM linker/merged 装载诊断（Undefined symbol 发射点）；docs/plans/reports/446-downstream-settlement.md §五.2 |

---

## 🟢 已知限制（设计决策，非 bug）

| 计划 | 类别 | 描述 | 引用 |
|------|------|------|------|
| 470 | use.rust deprecation 周期 | `use.rs` 为现行拼写（Plan 470），`use.rust` 仍解析但发 W0005。移除触发条件：外部仓（auto-musk/auto-ai/book 等 ~78 .at）随工具链升级完成迁移 + 一个发布周期零存量后，独立 plan 删 parser/scanner 分支改报错。本仓正式树 .at 已全部归零（2026-08-30 parser.at 注释亦迁；豁免仅剩 `docs/plans/reports/` 历史报告、docs/plans 与 specs plans.md/retrospective 历史页）。 | `docs/plans/470-use-rs-alias.md` D5 |
| 470 | auto/lib/parser.at 快照漂移 | AAVM v2 parser 同步快照（Plan 432，baseline b3bd64f5）钉在旧版 parser.rs，use 解析整体在 Missing 清单（无 use.rs/use.rust 分发代码，唯一命中为注释，已随 Plan 470 改为 use.rs 表述 2026-08-30）；快照随 parser.rs 演进的重新同步义务不变，归 Plan 432 同步链（M2 闸门本被字符串池 RC 回归阻断）。 | `auto/lib/parser.at` 头注 + Plan 432 |
| 478 | 实机键流补采 | switcher 键盘流/pager 点击/send_to 的 OS 键注实机截图缺采（前台竞争 frontmost_pid_mismatch，472 同款先例）；逻辑 headless 全链覆盖（19 新测试含宿主臂 toast 门）。补采：前台空闲跑 `examples/ui/028-launcher/tests/test_478_t6.py`。 | `reports/478-t6-live-acceptance.md` + `reports/478-t1-blueprint.md` §8 R1 |
| 478 | Ctrl+Space 叠召唤不设防 | switcher 开启时 Ctrl+Space 仍会叠召唤 launcher（双 overlay 堆叠合法，Esc 逐层退可达）；v1 接受，M3 通知中心/表面仲裁时统一。 | `reports/478-t1-blueprint.md` §8 R4 |
| 476 | slot 范围外项（v1） | teleport、动态 slot 名（`slot(name: expr)`）、多层 widget 嵌套 slot 透传未实现（需求 009 §3.7 明示范围外）；`for` 循环体内直接出现的 outlet 走非拼接兜底路径（单子直通/多子 Column 包装，轴向不随容器）。 | `docs/plans/476-vm-slot-substitution.md` 目标 7 + D5 |
| 476 | 多子填充 probe 共享路径 | 源索引编号容器（scroll/container/grid tracked 双胎）下 outlet 的多个展开子视图共享 outlet 源索引——多子填充时后写 probe 条目覆盖先写（MCP 快照少一行绑定，vtree/渲染/事件不受影响）；musk 现网具名槽填充均为单子节点，未触发。 | `ui/aura_view_builder.rs` expand_children_spliced_source 注释 |
| 476 | VM registry 仅注册 use 导入 widget | `auto run -r vm` 的 WidgetRegistry 只收 `use` 导入的 widget（lib.rs run_file_dynamic_ui_inner）——同项目隐式组件调用（无 use）落 tag fallback（children 包装直渲染，组件自身视图丢失）；033-slots 此前即此假阳性（填充"可见"实为 fallback 直出）。示例须 `use x: X` 导入才走真组件路径。 | `crates/auto-lang/src/lib.rs` 2b 段 + 033 app.at Plan 476 注释 |
| 476 | ui 模块测试盲区 | `cargo check -p auto-lang`/`cargo t`/`cargo tf` 默认特性不含 `ui` feature——ui/（aura_view_builder、iced renderer 等）源码与测试**不在日常档编译运行**，须显式 `--features ui-iced`（本计划 T5 发现：首轮 check 对 ui 改动是空转）。 | `crates/auto-lang/src/lib.rs` `#[cfg(feature = "ui")] pub mod ui` |
| 472 | dock 实机注入项（×关闭/切换条/键盘流） | 沙箱输入注入与前台竞争：click dispatch 身份校验失败/坐标过期、键盘注入需前台被用户会话抢回——×关闭、切换条点击、召唤 launcher、布局热键四项实机未点验（464 同款先例）。语义 headless 全覆盖（T2 分区 7 测+T3 投影反射测+动词解析测+wm_remove_win 测），后续注入通道升级或真机人工点验补跑。 | `reports/472-t5-live-acceptance.md` §1 #5-8 |
| ~~472~~ | ~~dock 焦点窗标题文本未做~~ | **✅ 478 收口**：标题展示归 switcher rows（图标+title），472 遗留 debt 兑现。 | `assets/switcher.at` + Plan 478 |
| 472 | shell.at dock 双分支重复 | top/bottom 两个 if 分支各持一份任务栏标记（DSL 无局部模板复用）；v1 自证瑕疵，M3 pack 化时收敛（478 pager 升格维持双分支现状）。 | `assets/shell.at` view 注释 |
| ~~472~~ | ~~workspace 条显示原始下标~~ | **✅ 478 收口**：pager 1 基人读标签（宿主投影 `label`）+ 当前分区高亮，472 遗留 debt 兑现。 | `assets/shell.at` + `schema/projection-protocol-v1.md` v1.1 §2 |
| 465 | a2vue 组件路径 prop 直通 | shadcn 组件路径对任意自定义 prop（如 virtual_window 的 `win: w`）不透传（v-for :key 路径仅子组件/外部门组件走 v-bind 形态）；宿主叶子自读 WmStore 不依赖该透传，.at 直书桌面为后续需求时补齐。 | `ui_gen/vue.rs` is_shadcn_component 属性发射路径 + `reports/465-t4-wm-dom-leaf.md` §3 |
| 465 | reka portal 族窗口逸出 | dialog/modal/dropdown 等经 reka DialogPortal **DOM 重挂** document.body（T2 实测：CSS containing block 不可收敛，DOM 搬家≠CSS fixed）；v1 登记限制清单+启动警告，不改写 portal 语义；正规解 = 生成器 DialogContent 模板 `DialogPortal :to` + provide/inject 注入窗容器（generator 级改写，后续）。 | `ui_gen/vue.rs:14483` DialogContent 模板 + `reports/465-t2-containment-spike.md` §1① |
| 465 | document 监听跨窗广播 | App 自注册的 `.window/.document` 全局监听天然跨窗触发（T2 实测：单键双窗计数同涨）；桌面热键走捕获段+stopImmediatePropagation 自保；受害 App 白名单先行。 | `assets/wm/keyboard.ts` + `reports/465-t2-containment-spike.md` §1③ |
| 465 | pkg 失败吞错启发式 | run_command_live 的「node_modules 存在即 Ok」启发式（pnpm v9 ERR_PNPM_IGNORED_BUILDS 误报兼容）在 pnpm v11 下仍可能吞掉真失败（本批仅修 add --dev→-D 根因；启发式收窄待后续）。 | `pkg.rs run_command_live` + `reports/465-t8-parity-record.md` §9 |
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
| 422 | popover 覆盖边界 | rust 模式 codegen 未覆盖 popover 标签(`ui_gen/rust.rs`)——VM/iced 语义完整,vue 走 shadcn 映射;子菜单 z 序嵌套(面板内再弹)远期不承诺;两项实机人工验收未跑(041 右键菜单落点/gallery popover 开合,MCP 无鼠标注入无法自动化)。另(2026-08-29 下游回传)**vue 半缺口**:popover 在 vue codegen 为惰性 div 透传(:open 不门控→面板常显;@dismiss 不接线)——VM 渲染半已修(Plan 446 C1 点锚+ondismiss),vue 半修法=shadcn 式 v-model:open 门控或 v-if 输出;下游暂以 regen 部署侧 sed 补偿(:open→v-if,剥 :x/:y/@dismiss)。 | `ui_gen/rust.rs` + `ui_gen/vue.rs` popover 臂 + 041 + gallery;docs/plans/reports/446-downstream-settlement.md §五.3 |
| 414 | auto-edit UX Phase B 族 | toolbar 右对齐被 VM Row 渲染器限制阻塞(§7.2,解除后一行启用);真折叠 Phase B(fill_raw 整缓冲绘制无法跳行,倾向 core 自绘,单独立项);action 声明块 Phase B(§6.1,parser/view/aura/a2r 四端改造)。menubar overlay 一项已由 Plan 422 P2 解除。 | `ui/iced/renderer.rs` + 041 |
| 423 | RC 安全挂账 ×2 | ARRAY_LEN 弹栈不释放引用操作数(每调用泄漏一个份额;持久列表无感,临时列表会长存);裸 i32 堆 id 编码与 TAG_OBJECT 双轨并存,pop 侧无法区分"带份额压栈"与"历史裸压",全量收口需一次性迁移(419 遗留议题)。 | `vm/engine.rs` ARRAY_LEN + 堆 id 编码 |
| 449 | vm 组件渲染三缺口 | 回调 props 退化/快照组件子树不可见/片段参数化条件不求值(§3.1-3.3)——修好后 041 tab 条/确认弹层可继续组件化;vue 侧 action 配置(vue codegen 全局 keydown+menubar/toolbar 合成)另立计划。 | `ui/render*` + 041 tab_bar/confirm_dialog 设计 |
| 446-R3 | regen store_import_prefix 未暴露 CLI（低优先） | 下游 vue 轨需改 store 导入前缀（撞名规避）时，codegen 无 CLI 开关，只能 regen 部署侧 sed——暴露 `--store-import-prefix` 类参数即可退役下游 sed。 | `ui_gen` CLI/regen 入口;docs/plans/reports/446-downstream-settlement.md §三 G1 |
| 446-R4 | VM state 投影对象数组显示 `[<vmref>]` | e2e/快照 state 对数组只投影 vmref 占位（内容不可见），下游门禁被迫改快照口径断言实体名——做对象列表摘要投影（如 `[{id,name},…]` 截断）可回收下游断言强度。 | VM state 投影（autoui_snapshot/state dump）;docs/plans/reports/446-downstream-settlement.md §六 |
| 449 | VM 字节码越界读 bug | store handler + `code_editor_set_text`(疑及同族 set 类内建)编译路径产出越界读,根因在 handler_codegen/vm codegen 对 store decl 的合成;041 目前以根 handler 规避,值得专项修复。 | `vm/handler_codegen.rs` + 041 |

---

## ⏸ 延期（finish-plan 登记的未竟项，Type=延期）

| 计划 | 类别 | 严重度 | 描述 | 根因/理由 | 引用 | 登记日 |
|------|------|--------|------|-----------|------|--------|
| 470 | 下游任务 | 🟢 | 批次三：外部 auto 系列仓 use.rust→use.rs 迁移顺延（用户 2026-08-30 裁定"先不做，之后单独做"）。清单：auto-musk 30 .at+10 md（backend/ 为主）、auto-ai 22 .at+8 md（crates/）、book 22 .at+21 md（rust/17+tapl/5，书稿）、auto-down 2 md（4 .at 全在 tmp/ 探针可豁免）、auto-shell 3 md（主树 .at 零命中；158 .at 全在 .worktrees/ 陈旧分支，随活跃分支合并时按告警迁）、auto-code-rs 2 md、auto-forge 1 md | 前置未满足：各仓工具链须先升级到含 use.rs 的 auto-lang（本仓 2026-08-30 合入待发版部署；先行文本替换会因旧工具链不识 use.rs 而破坏构建）。执行方式：升级后以 W0005 告警为迁移信号，机械替换+逐仓跑各自验证；完成后连同"一个发布周期零存量"触发 use.rust 移除 plan（见 🟢 节 P470 deprecation 周期行） | docs/plans/470-use-rs-alias.md 批次三 T11-T16 | 2026-08-30 |
| 463 | 环境限制 | 🟡 | worktree 内 `cargo tb`/book_listing_tests 全数不可运行（61 项"失败"为环境假象） | book_listing_tests 经 `CARGO_MANIFEST_DIR/../../../book` 读仓库外兄弟目录 `D:\autostack\book`，该相对路径从 worktree 解析必然落空（结构性，非代码问题）。缓解：`.worktrees/book → D:\autostack\book` junction（复审时已建）；长期可考虑 book 路径支持 env 覆盖 | docs/plans/463-desktop-shell-auto-arrange.md §10.2 | 2026-08-28 |
| 398 | 下游任务 | 🟢 | ash-gui-native M0.5 测试骨架（conftest/desktop_mcp/test_smoke）+ M1 in-process 后端 | 属 auto-shell 仓的下游任务，本仓计划仅负责 VM 侧修复（已完成） | docs/plans/archive/398-*.md §14.3/§14.4 | 2026-08-20 |
| 408 | 功能缺口 | 🟢 | P5-4：纯 module fn 文件不被 codegen（ui_gen/api.rs:456 报错） | 低优先 + 既有 workaround（塞进 widget/store 文件）；根治需先设计 codegen 入口扩展 | docs/plans/archive/408-*.md §11 P5-4 | 2026-08-20 |
| 406 | 审计矩阵 | 🟢 | 全量 nanbox 生产者-消费者类型配对审计矩阵（docs/audit/vm-type-audit.md）未产出 | 立项驱动的 4 个目标 bug 已全部由审计批次 A4/B4 根治，矩阵价值让位 | docs/plans/archive/406-*.md Phase 1 | 2026-08-20 |
| 473 | 环境限制 | 🟢 | 非 Windows 目标 `cargo check -p auto-lang --target x86_64-unknown-linux-gnu` 未本机验证（openssl-sys 交叉编译缺 OpenSSL，reqwest native-tls 既有依赖链） | native_dock 非 Windows 路径以同名 no-op 模块（win32_noop.rs）API 面逐一对照保证；环境性限制非代码缺陷，CI Linux runner 即可闭环 | crates/auto-lang/src/ui/native_dock/win32_noop.rs；plan 473 复审记录 验收#2 | 2026-08-29 |
| 473 | 验收顺延 | 🟡 | native dock 真人冒烟清单顺延（用户 2026-08-29 裁定 E2E 代验）：B1 拖拽手势（Phase 1 无 dock 用户触发面）、B5 实机模态、B6 IME、B9 多显示器缩放矩阵（仅单屏 200% 覆盖）、D1 Chrome 实机、B8 退出恢复整链实机、C1 真提权进程。**486 清偿（2026-08-30 实机执行留痕）**：B1 ✅ 真拖 Notepad 入全屏桌面→高亮→收编→任务栏条目→× 关闭（t5_smoke 驱动+截图）；B5 ✅ docked fixture 弹 MessageBox 置顶可交互；B8 ✅ Esc 退出→双窗 chrome/bounds 整链恢复；D1 ◐ Chrome docked ~12s 后其自移触发 C4 自动恢复+toast（Chrome 自身行为，处理优雅，"常驻"未完全达成）；G2 附带 ✅（C4 undock 实机实证）。**仍待用户**：B6 IME 中文输入（自动化无法忠实模拟输入法）；C1 真提权（UAC 交互不可自动化；UIPI 拒收路径有单测）；B9 双屏矩阵（本机单屏 4K@200%；自动化框架已就绪=t5_smoke 驱动+native_dock_e2e） | Phase 1 假洞的可自动断言部分已由 fixture E2E 六测试覆盖（B1/B2/B3/C3/C4/C5/B7）；触发面（shell UI/手势）已随 Phase 1.5（Plan 486）落地 | docs/plans/486-native-dock-trigger-surface.md 步骤9；tools/native-fixture/README.md | 2026-08-30 |
| 486 | 性能观感 | 🟡 | native dock 事件泵吞吐不足：`native_dock_event_subscription` 16ms 短轮询每拍仅 `try_recv` 一条（≈62 事件/s），系统级 LOCATIONCHANGE 噪声（多窗环境）下 MOVESIZESTART/END 排队滞后——T5 实机实测拖拽松手到 dock 落位延迟可达数秒 | 473 设计按"事件低频"假设单发轮询；486 手势引入后拖拽终态对延迟敏感。修复方向：每拍 drain-while-empty（通道排空为止）+ 可选分级（START/END 优先出队） | crates/auto-lang/src/ui/session.rs:1220 段；486 复审记录 | 2026-08-30 |
| 486 | 环境限制 | 🟢 | `ui::i18n_lookup::tests::plan050_i18n_lookup_loads_flat_json_and_misses_gracefully` 在 `--features ui*` 档红（i18n/zh.json 装载返回 None）——**非 486 引入**（master 同命令同红；默认档不编译该测试，cargo t/tf 日常门不受影响） | 疑似 cwd/feature 相关装载路径问题，独立于 486 文件范围；留待独立小修 | crates/auto-lang/src/ui/i18n_lookup.rs:72；486 计划待澄清 | 2026-08-30 |

*最后更新：2026-08-30（486 清偿 P473 真人冒烟行：B1/B5/B8 ✅ 实机留痕、D1 ◐、B6/C1/B9 仍待用户）；2026-08-29（446 下游结算回执收口:数据损坏条目 ✅（下游 parse-first 8a7b85f+soul.md 重写 639B）、渲染回归降级观察（03/04 在 c83435764 未复现）;新登 446-R1 applyAccent 撞名/446-R2 merged 链接面诊断（🟡节）、422 并入 vue popover 半缺口、446-R3 store_import_prefix CLI/446-R4 state 对象数组投影（增强节）;specs ui/vm plans.md 446 行对齐 archived;2026-08-27（复核归档批:413/414/421/422/423/449 六计划归档,遗留登记 7 条(413 人工验收/421 natives 桥接/422 popover 边界/414 Phase B 族/423 RC 安全×2/449 组件三缺口+越界读 bug),418 menubar 估位条目随 422 P2 落地退役;Plan 330 归档裁定：核心诉求被 199+MCP 工具族取代,剩余缺口登记 2 条——VM 内省三件套/trace 无 CLI 暴露;设计沉淀 design/14;Plan 332 同日改写聚焦 Serialize 方向;2026-08-25：plan-447 部分① 收尾登记 5 条：is 值语义 let 位返回 0/嵌套 fn 静默失效/struct 误用报 E0201/plan-444 3 红 golden/并行偶发测试；同日早前：vm-files-ci.yml 落地:六道闸门+goldens+conformance 接入 CI;ffi_dual_014 补 std 臂 VM 回归网+19_rust_std 10 ignore 解除;plan-430-fixes 清偿复审高危 4 条:compile_dep_methods 吞错/指纹声明版本/剔环上限+前缀误伤/泛型自由函数假签名——全部 ✅ 并补单测;aavm 系列 429-434 复审+归档:新增复审条目 9 条;Plan 434 AA2R 合并入库;Plan 444 修复 auto-shell-057;Plan 433 登记 4 条;2026-08-22 Plan 417-E3;2026-08-20 归档复审）*

### P482（2026-08-29，Plan 482 nav 组件族复审登记）

- **P482-1 musk rail 双端视觉复跑**：web 全栈冒烟（需 backend+auth 流程）与
  VM 轨 rail 截图未随 052 执行（build+产物核验替代）；归 musk PLAN-050 parity
  线复跑。nav-item 组件本体已在 auto-lang 015/018 双端实测。
- **P482-2 015-notes 搜索过滤缺位**：nav(search:) 已把死输入框接线到
  store.Search，但笔记列表过滤逻辑在重构前后均不存在（行为保持，非回归）；
  store 侧投影补过滤为后续小改。
- **P482-3 015-notes 置顶标记丢失**：原 NoteItem 的 📌 pinned emoji 在
  nav-item 化时未迁移（icon 槽被文件夹树语义占用，条件 icon 表达式 VM 求值
  未验证）；置顶语义仍可经 Pinned 页签触达。
- **P482-4 nav-item title/desc tooltip 通道**：os-config regen.sh 仍以部署侧
  :title 补偿截断 desc 悬停提示——组件属性候选（title: prop 双端映射），
  上游组件演进项。
- **P481-5 482 nav-item/nav-group 围栏红（✅ 已清偿,2026-08-29）**：
  482 落地漏三件：① LOCAL_UI_PKGS 只登记 nav-link 漏 nav（组件实现实存于
  crates/auto-man/assets/shadcn-ui/nav/,nav_contract.rs 单测锁契约）——已补
  白名单；② schema.rs 漏 nav-item/nav-group 元素 insert（aura.at/vb/render
  三表有）——已按 aura.at 忠实转录；③ baseline 漏 vb/render 两维的下划线
  臂拼写容忍（codeEditor 先例）——已裁剪 +4 行。schema_drift/docs_gen/
  component_registry 三件套 master 全绿。流程改进随清偿落地：三件套纳入
  cargo t/tf 日常与全量档（.cargo/config.toml,cargo t 3271 绿/15s）。
- **P481-6 实机 Ctrl+C→系统剪贴板末步**：单测+simulator 双证键路，实机
  最后一步被用户在用桌面阻断（详见 archive/481 §T5 留痕）；桌面空闲 30 秒
  可闭环。〔Plan 485 rider 复验尝试 2026-08-30：**未清偿**——001-helloworld
  实机起跑后，三种合成输入均无法建立拖选（SetCursorPos 拖动不进 winit
  raw-input 光标流；mouse_event MOVE 相对增量同样未达；CUA 拖拽对该窗口
  identity mismatch 拒发）。043 实机同时证明合成定点点击对 iced 按钮有效
  （handler 正常触发），即问题特定于"移动流"而非窗口焦点。结论：实机末步
  需人工手动拖选复验，或先建 winit 兼容的输入注入 harness；键路单测
  （text_selection_ctrl_c_writes_clipboard）保持绿。〕

### P483（2026-08-29，Plan 483 VM 双 input 双焦点/键盘双投递复审前登记）

- **P483-1 aura_N 快照流 scroll/container/grid 源索引编号错位嫌疑**：
  `expand_children_spliced_source`（源索引编号+视觉空后置过滤）与
  traverse_view 的幸存槽位编号在「inputs 前有被丢弃空兄弟」位形下可整体
  偏移（D-GAP-4 已修 column/row 同型，scroll/container/grid 未跟进）——
  仅影响 aura_N 派发流，本计划只修了 vnode_N 流（find_view_by_path 委托
  extract_children_ref）。后续：三容器改对齐 traverse_view 编号。
- **P483-2 VM storage 测试密闭性（cwd 落盘 + libtest 单进程顺序污染）**：
  stdlib storage 镜像按 CWD 哈希落盘跨进程存活（stdlib.rs storage_file），
  desktop_shell dock 测试的 storage_raw_remove 只清内存、get 惰性合并磁盘
  可带回旧值——desktop 运行过的检出 CWD 下 dock/notif 族测试假红（本次
  实锤：master 全量 6 败全为该环境基线，清理 %TEMP%/auto-vm-storage/
  {cwd-hash}.json 的 shell.dock.* 后转绿）。后续：涉 storage 的测试统一
  AUTO_VM_STORAGE_FILE 隔离，或 remove 同步清磁盘镜像。
- **P483-3 真键盘 Tab 双投递实机复验顺延**：根因链（text_input 无 Tab 臂
  →__focus_prompt→focus 同 Id 全置焦）已由 iced_test 机制级复现+六测锁定，
  但本环境 OS 级键盘注入对 winit 0.30 无效（PostMessage WM_CHAR/WM_KEYDOWN
  均不达）、computer-use 前台通道被并行会话抢占——真人键盘 Tab 复验
  （042 README 步骤 + musk admin/admin 全流程）列入真人清单。
  **2026-08-30 追记（Plan 491 T8）**：Tab/Shift+Tab 焦点环遍历已在 483
  登记表基建上机制级交付（renderer.rs focus_traverse+FindFocusedInput 探针，
  p491 七测全绿；点击直聚可见、无聚焦回落首项、单框自环、Captured 不达
  fallback）；真人清单**追加「musk 登录页 Tab 流」**（username→password
  真键盘切框；archive/483 与 auto-musk 011 §七 两处注记同日落），本债
  不闭合——真键盘通道阻塞依旧（491 T6 实录：前台再遭并行会话抢占）。
- **P483-4 MCP autoui_type 对 closure oninput 的正向派生怪癖（master 既有）**：
  003-converter 经 autoui_type 输 celsius=100 → fahrenheit 落 0 而非 212
  （master 二进制同表现；反向 F→C 正确）——MCP type 路径对闭包 oninput 的
  写序/清空语义与真键盘路径有差，非 Plan 483 回归，converter 真键盘联动
  以真人清单兜底。
- **P483-5 462 多 App 焦点命名空间现状**：input_ids 登记表已按 App 会话
  隔离（devtools per-App），但跨 App 全局 Tab 顺序/焦点分区仍为 v1 边界
  （Plan 462 立项跟进，本计划未扩展）。

- **P480-R1 enable_broker 停机旗标无生产调用点**：`DesktopSession::enable_broker`
  的 stop 旗标 boot 期置 false 后无显式停机接线（serve 线程进程级常驻为
  v1 设计，代码注释注明；probe 连接可唤醒退出）——显式停机归桌面退出
  流程后续接线。
- **P480-R2 内存基线为 debug 测试宿主口径**：480-memory-baseline.md 数字
  采自 nextest debug 测试二进制（非 release 产品 `auto`）；release + strip
  复核点已在报告 §3 明示（不影响"度量+判定"验收形态，Private 口径
  4.81MiB/App 临界达标结论随口径标注）。

### P485（2026-08-30，Plan 485 原生剪贴板 Phase 2 复审登记）

- **P485-1 clipboard 集成测试外部竞写偶发**：实剪贴板 set→get 断言在重负载
  （与全量套件并行）+ 外部剪贴板监听器（WPS/微信后台）竞写下 1/16 偶发红
  ——set→get 窗口内剪贴板内容被外部进程改写。T9 的 GlobalClipboardTestLock
  跨进程命名互斥已消除自仓测试间互清；对外部进程无管辖。缓解可选：受影响
  断言加单次重试。正常负载复审连跑 11+ 次全绿。
- **P485-2（出计划外观察，非本计划引入）master cargo tv 红**：
  `tests::aavm2_m4::test_aavm2_m4_codegen_corpus`（b13_is_enum.at 字节码
  对拍失配）在 master @3a4aacf19 即红——嫌疑 051-C7/484 并行合入线（本计划
  tf 3275 全绿、分支增量不触 codegen 生成路径）。建议尽快专项定位修复，
  归属线确认后可改挂对应计划段。

### P487（2026-08-30，Plan 487 shell-track M4 设置面板复审登记）

- **P487-1 面板可视交互实机照留待重跑（OS 注入受阻变体）**：齿轮开面板/
  dock 热切换可视/Esc 实机照被 CUA 像素身份守卫阻断——窗口域坐标点击
  identity mismatch、全屏域 live-owner stale（激活前台 + AUTOUI_MCP_DISABLE=1
  停帧泵复测仍复现），与 472/478/479 前台竞争同族但机制不同（像素身份 vs
  前台校验）。语义链 headless 全覆盖（settings_* 七测）；T4 报告
  `docs/plans/reports/487-t4-live-smoke.md` §2 指针成文，前台空闲可补采
  （479 P479-2 同款）。
- **P487-2（复审新发现，非本计划引入）ui-iced 特性档 4 既有红测试 +
  标准门禁盲区**：`--features ui-iced --lib` 全量下 master 即红 4 测——
  plan442_ext_link `plan050_void_stub…` / `ui::i18n_lookup::plan050_i18n_lookup…`
  （i18n 查表文案不上屏族）/ `ui::desktop_protocol::broker::adjudicate_three_steps`
  / `vm::native::code_editor_natives…e2e`。全部挂 `cfg(all(test, ui-iced))`
  门后，而日常/全量门禁（`cargo t`/`tf`）跑默认特性档看不见——与 P485-2
  同族「门禁盲区放过红」。
  **〔2026-08-30 已修——Plan 489 四测清零〕**：①② corpus `i18n/zh.json`
  被 .gitignore `*.json` 吞（落测未入库）→ 入库 + `!test/**/i18n/*.json`
  否定；③ adjudicate 测试固定管道环境干扰 → `adjudicate_on` 参数化缝 +
  pid 管道 hermetic（A/B 实证）；④ print bool 现语义 true/false 断言对齐。
  ui-iced 档 4074/4074 两连绿。「档纳入周期门禁」流程面仍开放（另议）。
  **〔merge 追记〕** 同族第五例当场现形：`desktop_dock_edges…`/
  `desktop_shell_at_builds…` 两测无隔离，实机桌面用过 487 设置面板后
  （store 落 `shell.dock.*` 键）即必红——merge 补隔离热修（L0），
  ui-iced 档 4081/4081 全绿。
- **P487-3 shell.at 双任务栏分支重复（既有 v1 瑕疵延续）**：top/bottom 两
  分支各一份任务栏标记（shell.at:63 注释自认，M2 pack 化收敛）——本期齿轮
  按同款双份落码（+11 行 ×2），非本期引入的新债。
