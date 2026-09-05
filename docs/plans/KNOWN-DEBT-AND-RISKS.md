# KNOWN-DEBT-AND-RISKS — 已知技术债与风险登记簿

> **用途**：统一记录已归档计划中遗留的 workaround、一致性遗漏、架构风险和未来增强。
> 避免未来需要全扫归档计划才能找到这些隐患。
> **维护规则**：每次计划归档时的复审发现新遗留/风险，在此追加条目。
> **格式**：`[计划号] 严重度 | 类别 | 一句话描述 | 引用位置`

---

## 🔴 高风险（可能在特定场景导致 UB 或数据损坏）

| 526 | 崩溃（复现 2/2，2026-09-03/04） | 任务栏铃铛二次开合通知中心 → 桌面进程静默退出 code 1（无 panic 输出）。疑似 VM 层 `Process.exit`（stdlib.rs shim_process_exit）或未打印的 abort；RUST_BACKTRACE=full 复现实例仍无栈——进程性退出非 panic。跟踪于 535 D 项 | renderer.rs:8176 toggle_notification_center；stdlib.rs:683 shim_process_exit |

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

| 526 | 一致性 | 布局件级 hover/右键公共基建（wrap_layout_onclick）未做——launcher 用 button、桌面右键用 mouse-area 替代挂点，逐点特设；任意 .at 布局件要 hover/右键仍需逐个特设 | 526 待澄清③（用户核准延后，独立立项候选） |

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
| ~~474-旁支~~ ✅ 已清偿（2026-09-03，Plan 525 W1） | aavm bool 显示 parity（值模型级） | Val+VBool 载体落地（PushBool/比较/逻辑结果 ev_vb、ev_truthy/ev_str true-false、ev_cmp bool 位）；b13/b14/b19 恢复裸 bool 断言（A2R_BLESS 期望翻转评审位，g26 护航）。ADD/STR_CAT 混 bool 的宿主哨兵退化形态不镜像（语料不覆盖，engine.at DIVERGE 注记）。原登记：宿主 print(bool) 已改 true/false 而 aavm 以 1/0 承载——b13/b14/b19 曾改写为条件分支形态过渡。 | `auto/lib/engine.at`；`crates/auto-lang/test/vm/aavm2/corpus_m4/b1[349]*.at` |
| auto-shell-057 ✅ | vue codegen 五类缺陷阻塞下游构建（2026-08-24 外部报告）—— **已修复：Plan 444（合并 master de76581ea，feat 9ff6a38b9，2026-08-24）**。修复后下游复现 `auto gen → npx vue-tsc` 0 错 + `pnpm run build` 绿，无需任何手工补丁；修复明细与新约定（回调通道/emits 名册/emit 桥/__vmOnly 桩/any 通道）见 docs/plans/444-vue-codegen-ash-shell-057.md。原文存档： | 下游 auto-shell ash-gui 项目 `auto gen` 重生成后 vue-tsc 余 13 错 / 5 类，**Vue/浏览器渲染目标整体不可构建**（merged VM 目标不受影响）。443(defineModel 降级)/435 P0-P1 合入后复测构成不变。五类：① 子组件回调 props 生成 `on_delete`（snake_case 且必填）而父级绑定发射 `onDelete` —— 名字永不相配（043 R4 修过 PascalCase emit，`Delete` 形态漏网，3 错）；② 可空变体字段模板访问 `cell.Tagged.text` 在 v-if 守卫内仍报 TS18049，需生成 `?.`（2 错）；③ 多参 msg 的 emit 签名生成 0 参 —— `Sort(int,int)`/`Filter(str)` 调用点报 TS2554（043 B-1 只修单 payload，2 错）；④ VM-only stdlib 泄漏进 JS：widget handler 内 `fs.read_dir`/`File.is_dir`/裸 `await complete` 原样输出到 .vue script 产出坏 JS，无 JS shim 时应降级或显式报错（4 错）；⑤ str 模型字段的动态变体读 `__sse_status.Failed`（裸串或 {"Failed":msg} 二态）报 TS2339，需 any 通道或契约化（1 错）。另 gen 模板缺口：`auto gen` 重写 package.json 丢 `@vueuse/core`（shadcn ui 组件引用它）；无引用残留 CodeEditor.vue 不清理。**复现**：`cd auto-shell/ash-gui/ash-gui-auto && auto gen && cd gen/front/vue && pnpm add @vueuse/core && rm src/components/CodeEditor.vue && npx vue-tsc`。详见 auto-shell DEBTS.md「Vue 产物构建引擎侧阻塞」（2026-08-24，含逐类行号）。 | `ui_gen/vue.rs` prop_to_ts_type/sub_widget_event_to_vue（①③）、模板 emit（②）、ts_adapter handler 转译（④⑤）；auto-shell docs/plans/057 §Phase 5 |
| 446-R1 | codegen applyAccent 撞名（TS2440，2026-08-29 下游回传） | Plan 409§8/458:store 拥有 `accent_color`/`dark_mode` 字段时 vue codegen 注入内嵌 applyAccent 助手（ACCENT_PALETTE+watch 同步行）；若该 store 同时 `use back.api:` 导入同名 fn，生成 TS 即 TS2440（import vs 局部声明冲突，vue-tsc 红灯）。下游 auto-os-config 已将 back.api 的 applyAccent 改名 saveAccent（与 loadAccent 对称）规避。修法：注入前检测 use 清单撞名并避开/告警。 | `ui_gen/vue.rs` store 注入块（~3206 ACCENT_PALETTE_JS）+ widget 路径（~2575）；docs/plans/reports/446-downstream-settlement.md §五.1 |
| 484 ✅ | 包组件 Init 内 prop 字符串比较破坏 codegen（静默）—— **已修复：Plan 492（2026-08-30，M4 定位真机制+M5 显式诊断+M6 绕开摘除）**。M4 定案:非 codegen 分叉（包链与 use-widget 链同 Parser 同合成器）,真机制=裸 prop 名在赋值 RHS 位触发 undefined variable 解析错 → `parse_package_widgets` per-file try-parse **静默整文件丢弃**（点前缀 `.curve` 直接形态 Init 内全链可用）;M5 落地装载/合成/链接三层显式诊断（组件名+原因,静默消除）;M6 摘除全部绕开,Init 内直用 `.type`/`.curve` prop 比较,六道门禁全绿（chart 专项 lib 12/ui-iced 28/golden/cargo t 3292/ui-iced 4116/tf 3293）。原文存档： | official 包组件（`use {package}` 路径）Init handler 内出现 `if curve == "monotone"` 类字符串 prop 等值比较时,**整个子组件 Init 静默失效**——全部几何输出回落 model 默认值,零诊断输出。实证:484 M1 bisect（干净 HEAD 通过,仅加 prop 比较即崩,git stash 双向验证）。同形态 `use widget:` 导入路径不受影响（013-todo todo_list.at 的 model 比较/带参 handler 均正常）——**包加载链特有**（lib.rs P4-4/D13 child_decls 单 VM 编译 vs use-widget 编译链的 handler codegen 差异）。绕开（已落地）:Init 内双算双存变体（segs/segsM、segs/segsS）,view 侧按 prop 选边（view 内 prop 比较正常,如 `if .axis == "auto"`）。回归锚:cargo test -p auto-lang --features ui-iced gallery_chart_components + plan484_chart_component_tests | `components/{line,bar,area}_chart.at` 头注;`crates/auto-lang/src/lib.rs` P4-4/D13 块;plan 484 M1 记录;docs/plans/492-engine-view-text-fixes.md M4-M6 |
| 484 ✅ | 包子组件带参 msg 声明破坏整包编译（静默）—— **已闭环（不可复现,同上行真机制）：Plan 492 M4（2026-08-30）**。master 上带参 msg 声明 VM+vue 双轨均正常（484 记档不可复现）;484 现场实为同文件裸 prop 名 RHS 解析错致整包静默丢弃（见上行）,归因到了 msg 形态。M6 恢复 `msg { Init, Hover(int) }` 带参声明（三副本）,回归锚钉住（c2_param_msg_declaration_both_tracks_alive）。原文存档： | official 包组件 `msg { Init, Hover(int) }`（带参 msg 声明）使整包加载后所有子组件 Init 失效（静默,同上形态,零诊断）。去掉 msg 参数声明、仅保留裸带参 handler `.Hover(i) -> {}` 一切正常——事件经 DynamicMessage::Typed args → encode/decode_payload → call_handler_for 走通（mouse-area hover 已实证）。对照:`use widget:` 路径 013-todo todo_list.at `msg { ..., ToggleTodo(int), ... }` 正常——同上,包路径特有。绕开（已落地）:包组件一律 `msg { Init }`,带参 handler 裸挂。根治:查 load_package → child_decls → 单 VM 编译链对 messages 的处理（疑 codegen 为带参 msg 生成的桩/表在包路径下错配,与上条可能同根）。回归锚同上 | `components/*_chart.at`;`ui/dynamic.rs decode_payload`;plan 484 M1 记录;docs/plans/492-engine-view-text-fixes.md M4/M6 |
| 484 | VM 长页面无滚动容器（内容剪裁） | 超出窗口高度的 VM 页面内容被剪裁（无滚动包装):实测给 charts-gallery 页面包 `scroll (style: "h-screen")` 后 VM 渲染为空页（iced Scrollable 与全屏内容组合异常),已回退;vue 轨浏览器原生滚动不受影响。缓解:页面控制内容高度(限宽/紧凑卡);根治:排查 iced Scrollable + min-h-screen 组合的布局塌陷 | `examples/charts-gallery/src/front/app.at`(revert a997e9f65);plan 484 后续目检 |
| 484 | donut 扇区直接 hover 仅 vue 可用 | svg 子元素(path)挂 `onmouseenter` 事件在 VM 轨破坏组件渲染(实测 bisect:事件撤下即恢复),svgdoc 为静态位图无交互。现状:donut 悬停**图例行**出 tooltip(双端一致,Legend-hover 也是 BI 工具常规交互);vue 轨可另行加 path 事件增强。根治:svgdoc 交互能力(svg-text 同族,待 svg-text 别名/492 族修复后评估) | `components/donut_chart.at` 图例 mouse-area;plan 484 后续目检 |
| 484 | tooltip 逐 index 锚点定位（降级为固定右上角） | chart tooltip 的锚点坐标需要动态像素定位（`top-[{.tipY}px]` 类插值类名/StyleBinding 动态值),vue 侧 f-string `{}` 形式在 handler 生成中不插值、tailwind 任意值类需 JIT 扫描源码——两轨均不可靠。v1 降级:tooltip 固定于绘图区右上,内容随 hover 索引变化;逐 index 锚定待 StyleBinding 动态值或 v2 canvas。回归:plan484 冒烟 + charts-gallery 目检 | `components/*_chart.at` tooltip col;plan 484 后续复审目检记录 |
| 484 ✅ | f-string 含字面量 `[`/`]` 时 `${}` 插值破坏组件编译（静默）—— **已闭环（误归因,用户裁定 2026-08-30）：Plan 492 M1**。五层验证不可复现:①词法 token 探针 ②parser/单 VM 链 ③生产包链（charts-gallery 真源+load_package,bar Init 存活） ④Vue SFC ⑤金丝雀负对照（未定义变量补丁确实杀死 bar Init,证明夹具有检出力）。真因同"prop 字符串比较"行——同 Init 内裸 prop 名 RHS 解析错致文件静默丢弃,误归因到 f-string 形态;且 484 绕开形态 `f"w-[{slot}px]"` 的 `{slot}` 实为纯字面量不插值（无害垃圾类,布局靠 flex-1 意外生效）。M6 已恢复 dollar 形态 `f"w-[${slot}px]"` 并全回归绿;若后续发现 484 时另一复现路径,凭路径重开本条。原文存档： | `f"w-[${slot}px] h-full"`（dollar-brace 插值 + 字面量方括号）使包组件整体失效（静默形态同上）;同语义 `f"w-[{slot}px] h-full"`（brace 插值）正常。437 时代 donut `bg-[{color}]` 一直用 brace 形式故未触雷。疑点:lexer f-string 模式对 `${` 的 fstr_expr 消费与字面量 `[` 的交互（lexer.rs:629/724 两处 FStrNote 分支）。绕开（已落地）:含字面量 `[]` 的 f-string 一律用 `{}` 插值（bar/line/area band 样式 + tooltip 锚点 style 全部改造）。根治:f-string lexer 最小复现单测（`f"w-[${x}px]"` 解析层即可触发,无需 VM）。回归锚同上 | `components/{line,bar,area}_chart.at`（band 样式/tooltip style）;`crates/auto-lang/src/lexer.rs:615-745`;plan 484 M1 记录;docs/plans/492-engine-view-text-fixes.md M1/M6+待澄清③ |
| 446-R2 | merged 模式链接面双 api.at 无诊断（2026-08-29 下游回传） | back.api 符号链接以**外部 back 工程**（如 auto-os-config-back/api.at）的导出清单为准，in-project auto/src/back/api.at 只供实现体——改名/增删 fn 须两份同步，只改一侧即 boot 崩 `Undefined symbol: api.X in module App`，报错不指向第二份文件（下游实测定位成本高）。修法：诊断信息补"检查外部 back 的 api.at 导出清单"提示（或文档化双文件契约）。 | VM linker/merged 装载诊断（Undefined symbol 发射点）；docs/plans/reports/446-downstream-settlement.md §五.2 |
| 492-R1 | text 内容位置引用循环变量记录字段的 **VM 轨**渲染缺口（492 复审入账） | `for li in .items` 内 `text (text: li["name"])` 类"文本内容=Index 表达式"形态:vue 轨 Plan 492 M3 已修（Index 字符串键保留引号+不支持形式 R046 告警）;**VM/iced 轨仍不渲染**（43956041e 实证两轨均不渲染,M3 只补 vue 臂）。后果:chart 组件刻度/图例维持 yTick0..4/legendColor·Text0..3 槽位字段形态（484 后续 R006 绕开,M6 按计划范围明确保留）。根治:iced 侧文本内容表达式求值补 Index 臂（对齐 M3 的 vue 语义）;根治后 chart 组件可再摘槽位字段改直写 for+text。回归锚:plan492_m3_tests.rs（vue 侧）+ 需新增 VM 侧锚 | `crates/auto-lang/src/ui/iced/renderer.rs` 文本内容求值;`components/*_chart.at` 槽位字段;docs/plans/492-engine-view-text-fixes.md M3/M6 |

---

| 525-1 | aavm 主 a2r 转译洞两处（W1/W4 实证已顺修入仓） | ① merge 后处理盲替换 `&&"`→`&"` 误吃 "&&" 字面量（W1 顺修:前导引号保护）；② 链式变更 `a.tys.get(i).methods.push` 转译成 `.clone().methods.push` 丢变更（W4 规避:库内显式写回 D25 范式;`cg_compile_files` 播种环 E0382 同族两次实证）。根治=主 a2r 对链式变更/字面量敏感 pass 的 AST 级改造,超出 525 范围。 | `crates/auto-lang/src/trans/rust.rs`(22841/23050 附近)；`auto/lib/a2r.at`(ext 并入写回) | lib 自身已规避;新 lib 代码需遵守写回范式 |
| 525-2 | 闭包/嵌套 fn 的 m4 反汇编层不可对拍 | 宿主 closure(0x90 族)反汇编乱码级(`??? 0x0d` 形态);嵌套 fn 的作用域释放组规范化(排序组 vs jmp 目标互算)超本轮。裁定:g31-g33 迁位 corpus_a2r,判据面=发射闸+四路执行锚(四路 29/29 含之)。 | `crates/auto-lang/src/tests/aavm2_m4.rs`(normalized_dump);语料 `corpus_a2r/g3[0-3]` | 行为/发射已验证;m4 层对拍待宿主 closure 编码或规范化规则先行 |
| 525-3 | 生成器 yield 与 `??` NullCoalesce 延后 | 生成器:W0 盘点 lib 用量=0,按待澄清③裁定延后(宿主 Plan 321 在位);`??` 已入 Pratt 表但无语料面(未实现码 gen)。May 最小面(?T/Some/None/is 臂)已交付(g34)。 | `auto/lib/codegen.at`(?? 臂缺);宿主 `vm/codegen.rs` | 后续波次按需领取。**531 实测注记(2026-09-03)**:主 a2r 已支持 `??`→`unwrap_or`;原生 VM codegen 无臂(`auto run` 静默空输出——比报错更隐蔽,值得独立观察项)+自举 lib 三件(codegen.at/engine.at/a2r.at)全无臂;非便宜量级,Plan 531 显式维持延后。 |
| ~~525-4~~ ✅ 已清偿(2026-09-03,Plan 531) | 宿主 May 裸值 return 发射不编译——?T fn 内裸标量 return 包裹 Some(...)(主 a2r return 位+AA2R ar_return 镜像[Ar 增 cur_ret];仅裸标量形,Some/Ok/None/Unknown 不动);g34 补裸值语料 find_bare 臂(金样 30/none/30/none)。原描述: | `fn f() ?int { return n*10 }`(无 Some 包裹)主 a2r 发 `return n * 10;` 于 Option<T> fn——rustc E0308。525 语料取显式 Some 构造规避(g34);宿主发射修复(裸值自动包 Some)待后续。 | `crates/auto-lang/src/trans/rust.rs` | 语料已规避;宿主修复后可补裸值语料 |
| 525-5 | ⑤腿塔顶程序 rc=1 快死(P517-1 族再现,W2-W5 折叠点) | 折叠②起矩阵两次+手动塔顶均 rc=1 快速返回无输出(P517-1 文档形态一致);lib 增长至 ~879KB 后贴线加剧。折叠①时点矩阵 46/46(10m4.6s)健康;各折叠点四路全绿+语料腿全绿为替代证据链(517 折叠①先例)。恢复后终局复跑一次成功(13m56s 全程无 error,W1 基线 10m4.6s 的 +39%,健康带);紧接确认性复跑又快死——**间歇性**实锤。 | `parity/crates/auto-parity/src/aavm.rs`(build_aa2r_bin);环境负载 | 维持 P517-1 观察项;复现则独立分诊(非 525 改动引入——语料腿/四路全绿) |
## 🟢 已知限制（设计决策，非 bug）

| 526 | 视觉 | window_thumbnail 快照懒捕获前显示空（fallback icon 兜底；命中预抓已在 summon 链）| 526 T18 记录（KNOWN-DEBT 候选） |
| 526 | 视觉 | Popover 首次打开横向锚点偏左（任务栏菜单/icon 菜单同族；功能与消失正常，497 hover 缩略同族先例）| 526 波间回归记录（KNOWN-DEBT 候选） |
| 540 | 兼容: 旧 storage 配置键只读回退保留一个版本 | 桌面配置单源迁至 `~/.config/autoos/apps/desktop/config.at`（8 键：dock.position/enabled/pinned、desktop.wallpaper/wallpapers_dir、appearance.theme、desktop.transparency、notes.enabled），boot 一次性迁移后旧键**不再读不再写但未删除**——按 D4 定案保留一个版本防回滚双源，下一版本随清理 plan 删键（届时旧版桌面回滚将丢设置,属预期）。 | `ui/desktop_config.rs` LEGACY_STORAGE_KEYS + `docs/plans/540-desktop-settings-osconfig-unify.md` D4 |
| 540 | 范围边界: shell.desktop.hidden/icons 键留 storage | 桌面图标面可见性（`shell.desktop.hidden`/`shell.desktop.icons`）不属本期 8 键单源范围，仍走 storage 直写（desktop.at 右键隐藏链）——与 config.at 并存双轨；若未来图标面配置也要进 os-config 插件体系，随通用"桌面面配置"扩展再迁。 | `assets/desktop.at:174`；`docs/plans/540-desktop-settings-osconfig-unify.md` T2 勘察注 |
| 540 | 验收余项: 实机齿轮点击链路留给复审/用户在场环节 | 真桌面（ui_desktop）齿轮→设置窗的交互实机验收在自主执行轮受阻于焦点窃取保护（用户正用机，SetForegroundWindow/SendInput 不生效且不宜强抢）——已验证替代面：桌面 boot 含 045 条目（registry 42 entries 日志）、桌面全量渲染 PrintWindow 截图、无头端到端测试（真 shell.at 齿轮→open_settings→launch-or-focus→播种→config 落盘，settings_shell_at_smoke_gear_to_panel 等 10 测）；复审批准前建议用户在场点一次齿轮 + 拖拽/× 关闭。 | `docs/plans/540-desktop-settings-osconfig-unify.md` T11；`scratch/p540_vm_desktop.png` |
| 540 | 语义边界: daemon 直改 config.at 重启生效（无文件监视） | D1 定案桌面宿主 boot 直读 config.at（进程内 `DesktopConfig::load`）——设置窗写路径经宿主臂即时热生效 ✓，但 **auto-os-config 通用编辑器直改文件后，运行中的桌面需重启才吸收**（无 file-watch 通道）。若未来要求 daemon 编辑即时生效，需 boot 后增量 file-watch + apply 扩展（新计划立项，涉 M1 装载层改造）。 | `ui/desktop_config.rs` load()；`docs/plans/540-desktop-settings-osconfig-unify.md` D1 |
| 540 | 清理余项: HostCtx.settings_fields 死字段 | is_settings windowless 拆借路退役（T9）后 `HostCtx.settings_fields`（ShellFields）仅构造无人读写——pub struct 字段无编译告警；下个清理批随其它 ShellFields 家族（launcher/switcher/notification 同型仍在用）一并审视。 | `ui/session.rs:1685` |
| 552 | 边界: custom 钉选非策展 app 的显示元数据回退 | 桌面图标格/dock 的 customs 槽位（storage `shell.desktop.icons`）行为不回归——槽位仍在、点击启动走全量 `app_resolver` 不受策展限制；但 icon/label 查表源是策展后的 `registry_entries`，非策展 id（如 001–010 教学 demo）回退 `app-window`/裸 id。PLAN-552 架构注记"自定义图标经全量 resolver 解析"仅对启动成立、对元数据显示不成立（注入段查表从未走 resolver——代码为准）。若需完整元数据：boot 在 DesktopState 保留全量快照供注入段查表（小改动，随下个桌面批）。 | `ui/iced/renderer.rs` inject_desktop_surface/inject_dock_pinned reg 查表；`docs/plans/552-desktop-app-curation.md` 复审记录 |
| master | 存量红: test_charts_gallery_compiles 裸名折叠失效（552 复审发现） | 2026-09-05 PLAN-552 复审全量门禁发现：`ui_gen::vue::tests::test_charts_gallery_compiles` 在 master（5ff92f364）与 552 分支同败——`examples/charts-gallery/src/front/app.at` 裸名 chart 标签未折叠为包组件 SFC 引用（生成物落 `<div :data=.../>` 空标签，`<LineChart` 断言 miss）。非 552 引入（其 diff 未触及 charts-gallery/ui_gen）；此前无台账行。疑与并行 plan（549 ui-gallery/551）合入期破坏有关。需独立修复立项〔与 P555-D4 同源，台账以 P555-D4 为准〕。 | `crates/auto-lang/src/ui_gen/vue.rs:18002`；复现 `cargo t test_charts_gallery_compiles`（master 同败） |

| 计划 | 类别 | 描述 | 引用 |
|------|------|------|------|
| 448 | 边界: `__evt_*` 铸名跨兄弟冲撞面 | 内联 lambda 铸名 `__evt_<event>_<n>` 按每 widget 计数,而 registry 级 `input_state_map` 仅按 handler 名索引(first-wins)——两个兄弟子组件各铸同名 `__evt_oninput_1` 时第二个绑定静默失效。C 轮新铸名 `__bind_<W>_<n>` 已织入 widget 名规避,B 族 `__evt_*` 维持现状(用户显式命名惯例分散,低概率);实际案例出现时把 B 族铸名同样织入 widget 名即可。 | Plan 448 §3 C.5;`parser.rs` mint_events_inline + `ui/dynamic.rs` extract_input_state_map_with_registry |
| 448 | 边界: plain 生成器 grid cols 死属性 | `VueGenerator::new()`(非 shadcn)路径的 grid 元素走 extract_classes+通用透传,`cols` 落成无意义 `:cols="N"` HTML 属性(字面量时代即如此,448-I 未扩战);真实 `auto build` 走 shadcn 路径已支持动态/字面量 cols。plain 路径若被启用需补 grid 臂。 | Plan 448 §8 I.4;`ui_gen/vue.rs` push_passthrough_attrs |
| 448 | 边界: computed 块内 store 方法调用未接消歧 | H2 的 `__computed_<W>_<p>` 合成复用 handler 的 state-ref 重写,但未纳入 store 多仓消歧重写(handler 的 store 重写机制独立)——computed 块体调用 `store.Xxx()` 在多 store 场景可能错路由;表达式 computed 同边界(内联求值器同样无消歧)。单 store 项目无感。 | Plan 448 §7 H.4 边界补充;`ui/handler_codegen.rs` synthesize_computed_fns |
| 518 | planned-debt: backdrop 真模糊渲染挂 RenderQueue | `backdrop-blur-*`/`backdrop-saturate-*` 毛玻璃词汇已声明冻结（共享 parser `StyleClass::BackdropBlur/Saturate`,Plan 518 G8）,但 iced/gpui/headless 三渲染臂为视觉 no-op（装饰性降级非错绘,不报错不 not-yet）——真 backdrop-filter 渲染推迟 **RenderQueue 宿主栅格化**:窗口根容器 → 宿主 WM 窗口级 glass 属性（queue/pixels 双臂通吃）;应用内面板 → `DrawOp::BeginBackdrop/EndBackdrop` 追加式 tag 对（线格式零变更）。已验 iced 0.14 源码:无 backdrop primitive、无 pass 干预口;`window::screenshot` 为整场景重渲+阻塞读回+上一帧玻璃反馈污染,只适合快照;fork iced_wgpu 可真解（screenshot 代码即施工图）但 RenderQueue 在途,裁定不投。vue 臂类串直通（Tailwind JIT content 直扫,零登记即生效）出真毛玻璃——样张 `examples/capability-tests/p518-glass-sample`（stella 配方直译,VM 降级为既定语义;PLAN-552 探针清退迁出）;parity 双端对拍中玻璃卡为已知分歧（VM 降级,RenderQueue 期翻转）。glass 配方另两腿已就绪:半透明底 `bg-white/10`（parse_color_with_alpha 既有）+ border 既有。 | `crates/auto-lang/src/ui/style/class.rs` BackdropBlur/Saturate + iced/gpui no-op 臂注释;`docs/plans/518-desktop-visual-phase2.md` §8 |
| 518 | 架构缝: os-config 逐 app 主题 × shell 全局主题共享 dark_mode thread-local | 全局 `DARK_MODE` thread-local 是 process-wide 单例:dynamic_view 每帧读各 App 的 `dark_mode` 声明变量回写全局——504 的逐 app 用户配置（如 `~/.config/autoos/apps/calculator/config.at` theme=dark,osconfig seed 在 allocate_app 同步**之后**合法覆盖）会把 **shell chrome 一并翻深**（浅色桌面开 calculator 实测:titlebar/dock 变深,而 calculator 自身视图浅——构建时序交错成混色窗）。518 缓解:boot 读回+allocate_app(desktop 宿主门控)同步已声明变量;根治 = per-app color context（渲染时按 App 路由各自主题,而非全局单值）,与 RenderQueue 色彩上下文重构一并。 | `ui/iced/renderer.rs` dynamic_view dark_mode 同步 + `ui/session.rs` allocate_app Plan 518 注;`docs/plans/reports/518-t3-visual-parity.md` 注记④ |
| 470 | use.rust deprecation 周期 | `use.rs` 为现行拼写（Plan 470），`use.rust` 仍解析但发 W0005。移除触发条件：外部仓（auto-musk/auto-ai/book 等 ~78 .at）随工具链升级完成迁移 + 一个发布周期零存量后，独立 plan 删 parser/scanner 分支改报错。本仓正式树 .at 已全部归零（2026-08-30 parser.at 注释亦迁；豁免仅剩 `docs/plans/reports/` 历史报告、docs/plans 与 specs plans.md/retrospective 历史页）。 | `docs/plans/470-use-rs-alias.md` D5 |
| 470 | auto/lib/parser.at 快照漂移 | AAVM v2 parser 同步快照（Plan 432，baseline b3bd64f5）钉在旧版 parser.rs，use 解析整体在 Missing 清单（无 use.rs/use.rust 分发代码，唯一命中为注释，已随 Plan 470 改为 use.rs 表述 2026-08-30）；快照随 parser.rs 演进的重新同步义务不变，归 Plan 432 同步链（M2 闸门本被字符串池 RC 回归阻断）。 | `auto/lib/parser.at` 头注 + Plan 432 |
| 478 | 实机键流补采 | switcher 键盘流/pager 点击/send_to 的 OS 键注实机截图缺采（前台竞争 frontmost_pid_mismatch，472 同款先例）；逻辑 headless 全链覆盖（19 新测试含宿主臂 toast 门）。补采：前台空闲跑 `examples/ui/028-launcher/tests/test_478_t6.py`。 | `reports/478-t6-live-acceptance.md` + `reports/478-t1-blueprint.md` §8 R1 |
| 478 | Ctrl+Space 叠召唤不设防 | switcher 开启时 Ctrl+Space 仍会叠召唤 launcher（双 overlay 堆叠合法，Esc 逐层退可达）；v1 接受，M3 通知中心/表面仲裁时统一。 | `reports/478-t1-blueprint.md` §8 R4 |
| 476 | slot 范围外项（v1） | teleport、动态 slot 名（`slot(name: expr)`）、多层 widget 嵌套 slot 透传未实现（需求 009 §3.7 明示范围外）；`for` 循环体内直接出现的 outlet 走非拼接兜底路径（单子直通/多子 Column 包装，轴向不随容器）。 | `docs/plans/476-vm-slot-substitution.md` 目标 7 + D5 |
| 476 | 多子填充 probe 共享路径 | 源索引编号容器（scroll/container/grid tracked 双胎）下 outlet 的多个展开子视图共享 outlet 源索引——多子填充时后写 probe 条目覆盖先写（MCP 快照少一行绑定，vtree/渲染/事件不受影响）；musk 现网具名槽填充均为单子节点，未触发。 | `ui/aura_view_builder.rs` expand_children_spliced_source 注释 |
| 476 | VM registry 仅注册 use 导入 widget | `auto run -r vm` 的 WidgetRegistry 只收 `use` 导入的 widget（lib.rs run_file_dynamic_ui_inner）——同项目隐式组件调用（无 use）落 tag fallback（children 包装直渲染，组件自身视图丢失）；033-slots 此前即此假阳性（填充"可见"实为 fallback 直出）。示例须 `use x: X` 导入才走真组件路径。 | `crates/auto-lang/src/lib.rs` 2b 段 + 033 app.at Plan 476 注释 |
| 476 | ui 模块测试盲区 | `cargo check -p auto-lang`/`cargo t`/`cargo tf` 默认特性不含 `ui` feature——ui/（aura_view_builder、iced renderer 等）源码与测试**不在日常档编译运行**，须显式 `--features ui-iced`（本计划 T5 发现：首轮 check 对 ui 改动是空转）。 | `crates/auto-lang/src/lib.rs` `#[cfg(feature = "ui")] pub mod ui` |
| 509 | desktop_protocol Linux 传输桩 | `transport::listen/connect` 非 Windows 为接口形状一致但**调用即错**的桩（PendingServer 空壳）；shm 非 Windows 为进程内映射表占位——跨进程协议功能（attach/孵化）在 Linux 运行时报错。三处 cfg 修补只达"ui-iced 全量在 Linux 可编译"（Windows 行为零变化，159 警基线持平）。真实现 = UDS 传输 + POSIX shm，属 Stage 2 增量（transport 头注既有"Linux 侧单独生长"预留）。 | `ui/desktop_protocol/transport.rs` pipe_stub + `reports/509-smithay-route-verdict.md` §5/§6 |
| 509 | WSLg 窗口不上浮 | 本机（远程会话）WSLg 协议层全通（窗口创建/EGL/swap）但窗口不浮 Windows 桌面（xeyes 对照证伪）——图形取证走 Xvfb+llvmpipe 闭环；后续交互类验证（Stage 2+ 输入/IME 实机项）需物理机或 WSLg 修复。另：WSL 内 cargo 需 `CARGO_HTTP_PROXY=` 空覆盖（git 全局代理指 Windows 侧 localhost）。 | `reports/509-t5-env-baseline.md` |
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
| ~~447-①~~ | ~~VM 函数内嵌套 fn 静默失效~~ ✅ 已清偿(2026-09-01, Plan 514 W1 步骤4) | 真实静默失效在**捕获位**（嵌套 fn 引用外层局部→静默 0）；改报 E0201（infer context fn_scope_idxs 边界栈+no-capture 查找+check_symbol Bina 臂），d01b 转绿解除 ignore。已知限：嵌套 fn 引用全局运行期静默 0=预存 VM 行为，master 同态非回归。 | `parser.rs`（Plan 514 步骤 4 证据） |
| ~~447-①~~ | ~~`struct` 非关键字误用报 E0201~~ ✅ 已清偿(2026-09-01, Plan 514 W1 步骤4) | 改报 E0007 语法错（parse_stmt_inner 顶部拦截+check_symbol Ident 臂，裸/带名形态覆盖），d02 守卫绿。 | `parser.rs`（Plan 514 步骤 4 证据） |
| 444 | master 预存红 a2r golden（514 复审实测 28 件） | `cargo tt`（test-trans）长期不在折叠门禁（tf 不含该 feature）。514 复审对照基线 87dda951b 实测：基线即 28 件红（444 期登记 3 件后经 447/511 各发射批累积扩大，无人重跑 tt）；514 净引入 0（`.field` 读位误发 `.field()` 缺陷被 514 方法族修复顺带根治，2 个陈旧 golden 已重生成 ff86babf4）。偿还：一次性 golden 重生成批（对照 live 输出逐件人工核验）。 | `trans/rust.rs` + `test/a2r/` 陈旧 golden 组 |
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
| 486 | 性能观感 | 🟡 | native dock 事件泵吞吐不足：`native_dock_event_subscription` 16ms 短轮询每拍仅 `try_recv` 一条（≈62 事件/s），系统级 LOCATIONCHANGE 噪声（多窗环境）下 MOVESIZESTART/END 排队滞后——T5 实机实测拖拽松手到 dock 落位延迟可达数秒 | 473 设计按"事件低频"假设单发轮询；486 手势引入后拖拽终态对延迟敏感。修复方向：每拍 drain-while-empty（通道排空为止）+ 可选分级（START/END 优先出队） | crates/auto-lang/src/ui/session.rs:1220 段；486 复审记录 | 2026-08-30 | **【已清偿 2026-08-31，Plan 505 S1】**：每拍 drain-while-empty 成批上行 + 批内 MoveSizeStart/End 稳定分区前置（`drain_slot_events` 纯函数；T1 单测 100 噪声+2 边界一拍排空/优先序/快甩即判）。
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
  **已清偿（2026-08-31，Plan 505 S5-B5）**：`shutdown_broker`（旗标置位 +
  探测连接唤醒，幂等）接五个桌面退出点（Esc / exit 命令×2 / 全窗关闭×2），
  serve 线程由"进程级常驻"收敛为随桌面停机；单测
  `shutdown_broker_sets_flag_and_is_idempotent`。
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
- **P485-2（✅ 已清偿,2026-08-31,Plan 495）master cargo tv 红——488 T11
  分诊已归属（2026-08-30）**：`tests::aavm2_m4::test_aavm2_m4_codegen_corpus`
  （b13_is_enum.at 失配）在 master @3a4aacf19 即红。**分诊定案=双后端
  `.line` 发射真分叉**（非 corpus 期望过期——该测试是 rust 编译器 vs
  aavm .at 实现的实时对拍，无静态期望文件）：rust `Codegen::emit_source_line`
  的同线去重（`current_source_line`，codegen.rs:10041）不发 `.line 10/11`，
  aavm 侧（AUTO_LIB_FILES_V2 的 .at 编译器）逐语句无对齐去重。修复=对齐
  aavm lib 的 .line 去重语义（或反向），转独立专项。488 分支 codegen 触碰
  仅两行 intrinsic 表注册（dnd_start，与 .line 无关，实证红先于 488 存在）。
  〔Plan 495 清偿注记：实测**分诊左右标注颠倒**——rust 侧发射 `.line 10/11`
  （parser `parse_expr_or_body` 给单表达式 is arm 记行号经 `Stmt::Block`
  发射），aavm 侧缺失（`cg_is_arm_body` 裸 `cg_expr` 不发）；且 rust 的
  同线去重（`current_source_line`）aavm 确无对应状态机。修复（定案=以
  rust 为规范）：`auto/lib/codegen.at` 增 `cur_line` 状态+`cg_line` helper
  （镜像 emit_source_line）+ arm 体行发射；b14_line_dedup.at 回归钉。
  cargo tv aavm2 全系绿（b13 转绿）；证据表见 plans/495 §执行证据。〕

### P495（2026-08-31，Plan 495 .line 对齐全档复跑观察登记）

- **P495-1（出计划外观察，非本计划引入）cargo tv 档 2 cookbook 既有红**：
  `cb_asynchronous_channel`（输出空 vs 期望两行 Title）/`cb_devtools_log_error`
  ——master 默认 checkout 单跑同红（双证）、失败形态一致，与 .line 改动
  无关（本计划零 Rust 源码改动）。与 P487-2 同族「非默认档门禁盲区」：
  挂 `--features test-vm-files` 档，`cargo t`/`tf` 看不见。aavm2 全系
  （m1-m5 corpus）在 tv 档全绿。待独立分诊（疑似近期合入破坏或环境依赖）。
- **P495-2（复审新发现，潜在分叉未暴露）is 块体 arm 作用域语义双端
  分叉**：rust arm 体统一走 `Stmt::Block`（codegen.rs:3790，push/pop_scope，
  depth>2 时 arm 块尾发槽释放组）；aavm `cg_is_arm_body` 块体路径走
  `cg_body_inline`（auto/lib/codegen.at，不推作用域，arm 内 var 归外层
  域）——arm 块体内声明 var 时释放组位置/时机分叉；corpus 现无块体 arm
  语料故对拍未暴露。附注：表达式级 `.line` 位（`Expr::Block`/表达式
  is arm，codegen.rs:9905/9934）aavm cg_expr 尚无实现，属 M4 fn-only
  能力边界，未来实现时需同款 `cg_line`。

### P488（2026-08-30，Plan 488 OLE 拖放双向复审登记）

- **P488-D1（VM 缺陷，488 载具调试发现）if 分支内 var 重赋值表达式调用
  `.str()` 内建 → 字符串累加破坏**：前缀丢失 + 错误码 -2147483647 混入
  （desktop_behavior.rs `#[ignore]` 探针存档复现，修复后转正）。044 示例
  用直接状态赋值 + join 绕开。
- **P488-D2（VM 缺陷，同线发现）heap-record Str 字段与 nil 的 `!=` 比较
  破坏求值栈**：488 注入面改空串哨兵（缺省恒 ""）绕开——**on_native_drop/
  paste 事件载荷契约按空串哨兵定稿**（text/image_path 缺省即空串）。
- **P488-D3（边缘时序，合成拖拽实证）毫秒级快甩拖入首轮可滞留**：
  DragEnter 未及送达（其自身也经主线程泵）即松手 → WM_NULL ticker 未上膛
  → Drop 滞留至下次输入。真人速度（≥0.5s）拖拽不受影响（E2E + 实机截图
  端到端实证）；如需消除，候选=注册期预上膛 ticker 或 DragEnter 之外的
  上膛时机。 **同族治理（2026-08-31，Plan 505 S1/S2）**：native dock 拖拽臂
  的快甩滞留已清——泵 drain 成批 + 批内 START/END 前置使同批终态即判
  （`dragwatch_fast_flick_same_batch_end_judged_immediately`）；OLE 拖入臂
  （DragEnter/WM_NULL 上膛时机）机制未改动，OLE 快甩若仍滞留余留观察。
- **P488-D4（增强候选）on_dnd_finished 交付取完成时焦点 App**：VM 侧无
  VM→AppId 通道，发起方追踪需会话记账——拖出期桌面持焦点与发起方一致，
  偏差场景未观察到实际影响。 **已清偿（2026-08-31，Plan 505 S8）**：
  DoDragDrop 在发起方 handler dispatch 内联阻塞至完成（488 步骤 9 定案）
  ——dispatch 环按拖出会话代号变化锚定发起方 AppId，交付序 = 发起锚定
  （取走）> 完成时焦点 > primary（v1 回退保持）；单测
  `dnd_finished_delivery_anchors_at_initiator`。
- **P488-D5（观察项）Ctrl+V 实机路由未留痕**：T5 单测绿（485 测试锁串行
  剪贴板往返），用户三轮未显式按 Ctrl+V 验证——语义链等价由单测背书，
  T6 重跑时可顺带补一行留痕。

### P487（2026-08-30，Plan 487 shell-track M4 设置面板复审登记）

- **P487-1 面板可视交互实机照留待重跑（OS 注入受阻变体）**：齿轮开面板/
  dock 热切换可视/Esc 实机照被 CUA 像素身份守卫阻断——窗口域坐标点击
  identity mismatch、全屏域 live-owner stale（激活前台 + AUTOUI_MCP_DISABLE=1
  停帧泵复测仍复现），与 472/478/479 前台竞争同族但机制不同（像素身份 vs
  前台校验）。语义链 headless 全覆盖（settings_* 七测）；T4 报告
  `docs/plans/reports/487-t4-live-smoke.md` §2 指针成文，前台空闲可补采
  （479 P479-2 同款）。 **已清偿（2026-08-31，Plan 505 S7）**：505 验收
  通道（内进程 MCP 注入 `autoui_desktop`，AUTOUI_ACCEPTANCE=1 门控——
  CUA 像素身份守卫阻断族统一解）实拍三帧归档
  `docs/plans/reports/assets/505/p487/`：开面板 / 位置热切换（任务栏置顶
  border-b）/ Esc 自隐。
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
  按同款双份落码（+11 行 ×2），非本期引入的新债。 **已清偿（2026-08-31，Plan 505 S3）**：双分支
  ~150 行收敛为参数化单份——flex-col-reverse 数据级翻转（iced col_reverse
  412 已有臂）+ 边线类宿主投影 `__dock_border` 拼接（taskbar 注册件的 if
  条件样式在实机装配层不稳，S7 补拍实拍发现后修正）；shell.at 440→308 行，
  desktop_mcp 装载测 3/3。

### P505（2026-08-31，Plan 505 桌面 DEBT 批处理一期复审登记）

- **P505-1 注册件/容器 if 条件样式在 live 装配路径丢失（根因未明，已规避）**：
  shell.at taskbar（注册件 → row 语义）上的 `style: if … {} else {}` 使
  任务栏在实机 iced 装配中零高度不可见——视图树单测两路（tracked/
  untracked）皆断言样式类在位（`shell_root_col_position_classes_and_taskbar_present`），
  仅真窗装配可见差异；静态串/拼接表达式（Bina Add）均正常。505 B1 以宿主
  投影拼接（`__dock_border`）规避。根因（视图树与 iced 元素装配间的样式
  丢失环节）未定位；偿还路径：装配层样式探针对拍定位后修 extract/装配臂，
  或明文禁用该形态（lint/文档）。影响面：所有在注册件/容器上写 if 条件
  样式的 .at（当前 shell pack 已清零，examples 检索未见同型）。
- **P505-2 事件泵实机拖拽体感复跑受阻（环境族，随 C 通道边界注记）**：
  A 族 drain+优先级的实机拖拽复验（t5_smoke SendInput 管线）在当前会话被
  阻断（caption 拖拽链 false——P504-3/P496-1 同族）；时延改善由单测数学
  背书（100 噪声+2 边界一拍排空）。偿还路径：OS 注入通道可用窗口（物理机
  人手或注入环境恢复）跑一轮 t5 拖入 + C 通道截图留痕。

### P494（2026-08-31，Plan 494 原生真洞 Phase 4 复审登记；用户已批准为债务）

- **P494-1 G4 覆盖层洞边裁剪（Region 机制代价）**：真洞经 `SetWindowRgn`
  排除实现（双 spike 证伪透明 swapchain 与 HTTRANSPARENT 跨进程后的机制
  替换，见计划待澄清①②），洞区窗口不存在——ghost 拖拽预览与 toast 在
  洞边界被硬裁剪（G4 目标降级：画到洞边、不进洞内）。偿还路径：物理机
  复验 DxgiFromVisual 透明成立后改「Region+透明」复合形态（洞内 alpha=0
  绘制），或独立置顶子窗承载洞区覆盖层（另立计划）。
- **P494-2 T5 实机清单环境受限延期（物理机复验）**：真洞实机视觉
  （Explorer/夹具 dock 洞区透出、四角/边框观感）、ghost/toast 裁剪边界
  目检、双屏不同缩放、洞内原生菜单/弹窗置顶——本机（ToDesk 远程显示 +
  Chrome Legacy Window 输入拦截）四条拖拽/点击注入路径（caption 拖/候选
  位探测/SC_MOVE 前台化注入/客户区点击）实测全部不可达 fixture。机制
  证据由 T3 E2E 跨进程铁证（洞心 SendInput 精确穿透 ±6、洞外零泄漏）+
  真实 win32 测试矩阵（z 不变量/Region 穿透/复位）承载。复验宿主：
  `AUTO_DESKTOP_HOLE=1` env（ui_desktop 示例）或 `shell.native.hole`
  storage 键。与 P494-1 同一物理机复验批。
- **P494-3 透明路径物理机复验（spike① 环境分支）**：vendored 三处补丁
  实证链在 `tmp/spike494-transparent/`（gitignored，含 vendor/iced_wgpu
  + iced_winit 一行级 diff：backend_options 读 env / PreMultiplied 优先 /
  with_no_redirection_bitmap）。本机 DComp 内容不上屏疑 ToDesk 特有；
  成立则 P494-1 可经「Region+透明」复合偿还。

### P496（2026-08-31，Plan 496 桌面本体 M5 复审登记）

- **P496-1 T4 交互实机照受阻（472/478/479/487 前台竞争家族延续）**：496
  桌面本体 T4 交互项（双击启动/聚焦、右键菜单三项、空白点击、settings
  壁纸写手）OS 注入通道同族阻断（CUA 像素身份守卫对活渲染面拒绝）。
  headless 全链覆盖（desktop_surface_* 三测 + settings_appearance_* +
  activate 两臂既有测）；实机证据 = 预写键 boot 单帧三断言（渲染帧）+
  T4 报告 `docs/plans/reports/496-t4-live-smoke.md` §2 对表成文。前台
  空闲可补采（P479-2/P487-1 同款排队）。 **已清偿（2026-08-31，
  Plan 505 S7）**：验收通道实拍两帧归档 `assets/505/p496/`——壁纸写手
  实变（#1e3a5f 同会话热生效）+ 图标 ActivateApp 激活 calculator；右键
  菜单/空白点击可经同通道复跑（IconMenu/BlankPress 在注入面）。
- **P496-2（复审新发现，非本计划引入）plan492 fstr 金丝雀基点红**：
  `--features ui-iced` 档 `plan492_tests::m1_pkg_fstr::pkg_canary_undefined_
  var_kills_bar_init` 在 39abc730f 基点即红（临时 worktree 探针实证），
  master 新顶（495 合并 08d060cba）已绿——分支合并时随 rebase/merge
  自愈，无需本计划处置。与 P487-2「门禁盲区」同族提醒：该测试挂 ui-iced
  门后，`cargo t/tf` 默认档不可见。
- **P496 待澄清（壁纸热切换）定案（2026-08-31，Plan 505 S8/D-2）**：探针
  结论 = **天然支持**——SaveWallpaper → storage 写 → `__desktop_bg` 投影
  重注入（指纹门控）→ 桌面本体面同会话热刷新，无需额外管道；实机证
  `assets/505/p496/p496-01-wallpaper-writer-applied.png`（#1e3a5f 实变）。

### P497（2026-08-31，Plan 497 shell-track S3 复审登记）

- **P497-1 pager 网格 ≤4 截断未实现（v1 全量显示）**：计划文本定"分区
  缩略网格条目 ≤4 截断 + '+N'"，实现为该分区全量窗口（每格 w-28 h-16
  + 标题 truncate）。根因：.at 无"过滤后截断"原语——for+if 过滤无局部
  计数器、数组无 take/slice；宿主派生平列表面临新投影字段违反本计划
  "协议零改动"约束。分区窗口语义少量（workspace_close 非空提示先例），
  视觉风险低。偿还路径：`.at` take(N) 数组原语（语言增强，另行计划），
  或 v1.5 协议字段（分区窗口派生面）。
  **裁定（2026-08-31，用户）**：不单独立项清偿——take(N) 原语增强归入
  债务批量清理批次（与 P497-2 等一起清），届时再定计划。 **已清偿（2026-08-31，
  Plan 505 S4）**：取两路径中的"协议字段"——投影协议 v1.5：`__wm_wins`
  条目增 `pager` 旗标（每分区 z_order 前 4）+ `__wm_workspaces` 条目增
  `more` "+N" 标签（宿主派生，指纹零扩展）；take(N) 语言原语按 505
  "无新架构"约束不做，有通用诉求时另立语言增强计划。
- **P497-2 a2vue window_thumbnail props 不透传 DOM**：金样 SFC 中
  wid/fallback_icon 被丢弃（class/v-for/:key 透传正常）——与 465
  virtual_window（win prop 同不透）先例一致的转译器 v1 局限。占位
  组件（icon+边框，待澄清①）不需要动态 wid；真缩略 web 路径
  （transform 缩放复制子树）落地时一并补 props 透传。 **已清偿
  （2026-08-31，Plan 505 S5-B3）**：a2vue `generate_shadcn_attrs` catch-all
  补 props 排序遍历透传（原只透 class）；window_thumbnail
  （wid/fallback_icon）与 virtual_window（win，465 先例同修）双金样更新，
  a2vue 14/14 绿零连带。
### P501（2026-08-31，Plan 501 os-config 集成复审登记）

- **P501-1 daemon 发现序第 3 级（PATH）v1 留扩展位**：`ensure_ready_io`
  生产路径 `lookup_path` 恒 `None`（osconfig_daemon.rs 成文注释「v1 不做
  PATH 扫描」）——发现序实际生效两级（storage `shell.osconfig.daemon` >
  相邻仓 target），Offline 原因文案如实只列两级。理由：本计划非目标明确
  排除「daemon 安装器/打包分发」，PATH 发现仅在安装态有意义；开发机形态
  被相邻仓探测全覆盖。`resolve_daemon_path` 的注入缝已就位，安装态立项时
  接宿主 `which` 语义即可。 **已清偿（2026-08-31，Plan 505 S5-B4）**：`which_in`
  纯逻辑（unix 可执行位判断）+ `which_daemon` 生产包装接 `ensure_ready`
  注入缝，发现序三级全生效、Offline 文案列全；单测
  `which_in_scans_dirs_in_order` / `ensure_ready_falls_back_to_path_tier`。
- **P501-2 T4 人手点击链残差（479/487/496 前台竞争家族延续）**：齿轮 →
  系统 → 打开系统设置的 GUI 像素自动化与 iced 活渲染栅格竞态不可靠未强
  驱（P496-1 同族）；headless 等价链全绿（T2 三态徽标注入 + 派发解析 +
  T3 launch 全链 + boot 35vs34 冒烟 + live spawn 2.52s/复用 774µs）。
  runbook：仓根起 `cargo run -p auto-lang --features ui-iced --example
  ui_desktop` 人手 30 秒抽查；前台空闲可补采（P479-2/P487-1/P496-1 同批
  排队）。 **已清偿（2026-08-31，Plan 505 S7）**：验收通道实拍两帧归档
  `assets/505/p501/`——系统分区 + osconfig 三态徽标、OpenSystemSettings →
  launch	os-config → 外部仓 os-config App 实拉起整窗（齿轮→os-config
  GUI 全链实机照）。

### P504（2026-09-01，Plan 504 calculator fit-window/os-config/stdlib 复审登记）

- **P504-1 18_ffi ignore 档 float/bool 格式化腐烂（存量，非本计划引入）**：
  `test/vm/18_ffi` ignore 档中 019 返回 `"1610612736"`（float 位样）、
  052/053 bool 打印 `"true"` vs 期望 `"1"`——master 基点同败实测确认，
  系存量腐烂。本计划新增 VM 文件测试 056_math_pow/057_str_is_digit 不涉
  该档。偿还路径：float/bool 值格式化语义统一立项（另行计划）。
- **P504-2 fit 为首帧一次性测量（内容动态增高不重测）**：`window: "fit"`
  语义 = 宿主首帧测量内容尺寸并收缩窗口一次。011 切 Scientific 模式后
  内容增高，窗口不重测，底部 `=` 键轻微裁剪（实机截图可见）。候选后续：
  内容尺寸变化信号 → 宿主重测（协议/宿主增强，另行计划）。
  **【已清偿 · Plan 512】** view 重建打标（dispatch 漏斗 fit_dirty）→
  ServiceTick 节拍重测 → 滞回 8px 双向跟随（standalone iced resize +
  desktop vwin rect 双路径，用户手动 resize 一次性锁定）。实证：
  011 Scientific +44px/回缩基线（tests/test_512_fit_remeasure.py）、
  005 校验错误行 +39px/回缩（005 同名探针）。两处实证改案留痕计划
  步骤 3：standalone ServiceTick 订阅断链补齐；活树布局受当前窗口钳制
  → 锚点外套 scrollable 量真实自然尺寸（宽度方向仍受视口钳制，见
  P512-1）。
- **P504-3 desktop 真实 launch 实况 e2e 未通（合成输入打不进 winit）——✅ 已清偿（Plan 515 G4 C3，2026-09-01）**：505 验收通道 p515 场景=DesktopBus launch 记录 → 011-calculator 真启动，实跑 PASS 截图留痕（docs/plans/reports/assets/505/p515/——窗口置顶+任务栏高亮）；旧障碍经通道 MCP 注入臂绕过（原描未通原因：
  desktop fit/seeding 覆盖到 session 级单测 + standalone 实机；ui_desktop
  宿主可起、MCP 截图可见桌面，但 SendInput 键鼠（即便 AttachThreadInput
  置前后）打不进 winit 窗口，launcher 召唤/桌面图标双击均无响应；MCP
  键盘工具 key 枚举无 Space/字母且只达 primary app（与 P496-1/P501-2
  前台竞争家族同族）。偿还路径：desktop 模式 MCP 加 launch/输入通道
  （可测性增强）。
- **P504-4 desktop boot 直传 comps 无 pac 语境（架构边界，记录）**：boot
  直开 app 路径不经过 pac 解析，故无 fit、无 os-config 播种。属既有架构
  边界而非缺陷；若将来要求 boot 直开也享受 fit/os-config，需在 boot
  路径引入 pac 语境（另行计划）。
- **P500-1 T3 `auto_exe()` 优先取现存二进制，陈旧产物伪装成回归——✅ 已清偿（Plan 515 G4 C2，2026-09-01）**：e2e_exe 共用体 mtime 对账 crates/ 树最新源文件，陈旧 eprintln 警告不阻断（防"回归"误读）+ `AUTO_FRESH_EXE=1` 强制重建；stage3/remote 两处委托 + 三态单测。原描：
  `t3_independent_pixels_and_dual_mode` 经 `auto_exe()` 拉起真
  `target/debug/auto.exe`，命中现存文件即用、不校验新鲜度——P500 折回
  master 首跑即中：该二进制系 Plan 504 会话所建（不含 500 代码），旧版
  cmd_autodesk 不识别 `--autodesk-render=` 且孵化记录缺第三字段模式位，
  双模断言（Pixels 臂缺失）失败；`cargo build -p auto` 后即绿，代码零缺陷。
  偿还路径：`auto_exe()` 加 mtime/版本探测（新于当前 lib 构建或带版本
  stamp），否则强制重建（另行小计划）。

### P503（2026-08-31，Plan 503 桌面视觉刷新复审登记）

- **P503-1 虚拟窗 min/max 视觉位无动词**：窗口 chrome 三色圆点组
  （VM `virtual_window.rs::traffic_light` + vue `VirtualWindow.vue`）中
  yellow(#febc2e)/green(#28c840) 为纯视觉预留位——session `WmCommand`
  无虚拟窗 Minimize/Maximize 动词（仅 native 槽位有 NativeSlotMin/Close），
  red=Close 是唯一有动词的灯。偿还路径：WM 增 min/max 动词 + 布局态
  （maximized 标志现以「窗矩形 ≥98% 桌面」几何判定近似），另行计划。
- **P503-2 vue 轨桌面无图片壁纸层——✅ 已清偿（Plan 515 G3，2026-09-01）**：assets/wm/Wallpaper.vue 三档组件（图片 bg-cover + scrim `bg-background/10 dark:bg-background/35` 与 VM 对齐 / #hex 纯色 / 空不渲染）+ host App.vue 生成期配置注入（storage 同键 `shell.desktop.wallpaper`；vue 无 storage 桥=降级判定，运行期改动经下次生成生效；token 级对齐钉于单测，视觉截图对拍挂验收通道）。原描：VM 轨壁纸 scrim（renderer.rs
  `desktop_wallpaper_scrim`，图片壁纸上叠 bg-background 10%/35%）在
  vue 轨无锚点——desktop host 桌面区为纯色 bg-background，无壁纸图层。
  归 P2（token 体系化/壁纸 parity 决策）一并评估 vue 壁纸层。

### P498（2026-09-01，Plan 498 chart 交互状态机复审登记）

- **P498-1 view 条件对负数 int 字面量比较恒假**：`if .v == -1` 在值确为
  Int(-1)（read_state 实证）时仍走 else——写入侧正常，疑条件串渲染形态
  导致 rhs 解析偏差（负数字面量被拆成 `- 1` 之类）。plan498 最小复现：
  model init -1 + `== -1` 分支不命中、`== 9` 命中。组件侧以越界哨兵 9
  规避（hov 族无悬停=9）。偿还路径：`eval_condition_with`
  （aura_view_builder.rs:6810 族）对负数字面量 rhs 归一 + 回归用例。
- **P498-2 VM 单态架构同名字段跨组件串扰（chart 交互态）**：Plan 320
  单 VM 单根状态下，子组件 handler 写入根状态、渲染期按字段名同步回各
  子组件——charts-gallery 四图族同名字段（hoverSeries/visible0..3）联动
  实证（一次 LineChart.Toggle(0) 六个图例项同时落 opacity-40）。组件侧
  以图族专属字段名（hovLn/hovAr/hovBr/hovDn + visLn/visAr/visBr/visDn）
  解耦四族；**同族多实例仍共享**（两个 line-chart 实例联动），vue 轨实例
  隔离无此现象——跨轨行为差异。偿还路径：子组件状态按实例隔离
  （prepare_child_render_state / on_with_input_for 的 state 路由，架构级）。

### P499（2026-09-01，Plan 499 chart v2 canvas 交互 M5 收尾登记）

- **P499-1 timer `when:` 门在调度器层未生效（恒 30Hz 空转心跳）**：
  `timer { AnimTick (every_ms: 33, when: .anim) }` 的 when 门在两轨均
  落在 handler 体内（vue 生成端闭包 `if (anim.value){…}` 内嵌；VM 轨
  [UI_EVENT] 日志实证无悬停时 AnimLnTick/AnimDnTick 持续派发）——
  setInterval/调度器无条件起拍,handler 早退。功能正确但每图族常驻
  ~30Hz 空事件(全 gallery 合计可观数量)。偿还路径:timer 调度器在
  派发前求值 when(状态变化时启停 interval),或 handler 空转计数熔断。
- **P499-2 donut tooltip 角落锚定不跟随(max-w-md 缩放对位风险)**：donut
  tooltip 维持 `absolute top-[20px] right-[20px]` 角落锚定(M4 决议)——
  line 的跟随定位(cx-80 钳制)不适用于 donut 的 max-w-md 随宽缩放(逻辑
  px 定位与渲染 px 有缩放差,面向 px 定位有对位风险)。同源遗留:svgdoc
  每次指针移动整串 path/d 重解析(vm dump 链路,由 M2 限频 33ms+量化
  0.5px 去重背书)。偿还路径:跟随定位走百分比/容器相对单位 + svgdoc
  增量更新通道。
- **P499-3 实机帧率基线对照未量化(验收 #2 以单元级限频证据背书)**：
  持续 mousemove 流「无帧率塌陷」以 M2 PointerArea 限频单测(125Hz 流
  →25Hz 发布/静止单发/量化去重)+vue 臂不经 RenderQueue 的架构事实背书;
  实机 fps 数字未采(VM 轨无 mousemove 注入工具,autoui MCP 仅
  press/type/keyboard)。偿还路径:MCP 增 autoui_mouse_move 注入 + 帧率
  探针(与 P499-1 空转心跳一并量测)。
- **P499-4 路线 B(两进程)widget 树输入注入**:设计文档 §8 已定界——
  输入通道 PointerMoved 已备、消费方仅编辑器 FrameSource,widget 树
  级输入注入不在本计划。归 diagram 边命中/节点拖拽后续计划。
- **P499-5(移入,非本计划引入)LaunchSpec osconfig_integration 潜伏
  E0063**:master 侧 Plan 504 为 LaunchSpec 增 name/fit 字段时只修了
  stage3.rs,`tests/osconfig_integration.rs:220` app_resolver 初始化器
  仍缺两字段——`cargo test -p auto-lang --features ui-iced`(非 --lib)
  编译即红(master 与 499 worktree 同现,499 M5 复验时发现)。日常
  `cargo t` 走 --lib/nextest 滤镜未触发。偿还:补 `name:None,fit:false`
  两行(master 侧一行修复)。**已偿还**:master 029a5f7ea
  (P505 S3-S4 顺带修,tf 档不含 tests/ 目标漏网同因)。
- **P499-6(移入,非本计划引入)widgets-gallery kitchen-sink.at 解析
  错阻断 vue serve**——**已偿还**(2026-09-01,Plan 510 worktree 提交
  d0c23388d):真因=生成器对**视图关键字名元素**(link,Plan 105
  parse_view_link 独占)发射标签简写 `link "sample" {}`(解析错级联
  20×"Expected term, got RBrace"@EOF);gallery_golden 的
  Err 臂把生成错误串哈希进基线「洗白」(kitchen-sink 条目 2.5KB 错误串
  vs 正常页 10-67KB 真 SFC)是漏网机制。修复=生成器对 link/tag/use
  (schema∩all_keywords 实测三撞)禁发简写 + kitchen-sink 再生成 +
  golden 重采样并**拒绝 !!GENERATION ERROR!! 样本入库** +
  gallery_pages_compile_tests(lib 级全页可编译冒烟,补上 cargo t/tf
  不跑 tests/ 集成目标的门禁盲区)。

- ~~**P499-7(移入,非本计划引入)cookbook `cb_asynchronous_channel` tv 档
  失败**~~——**已核销 ✅ 划线**(2026-09-01,Plan 513 债务簿核销;偿还于同日,Plan 510 worktree 提交 7a8ac1d2e):
  与「channel 收发时序」无关,双因:①native_catalog 把 Log 族登记
  1800-1803,与 Shell 族(NATIVE_SHELL_SYSTEM..EXIT,Plan 011,engine.rs
  显式注册后覆盖 inventory 同槽绑定)撞号——`#error(...)` 经 CALL_NAT
  1803 派发到 shim_shell_exit → ExitRequested(-1)(cb_devtools_log_error
  同病);②`asynchronous/channel/expected.out` 自诞生(66f6e78b6)即
  0 字节(生成期空输出被原样提交,VM 实际输出正确——债务原记载
  「输出空 vs 期望两行」左右读反)。修复=Log 四名移段 1805-1808 +
  expected 补齐 + plan510 双回归钉(行为/数据级)。**遗留讨论**:tv 档
  (test-vm-files)与 tests/ 集成目标均不在 cargo t/tf 运行集,盲区
  归 P499-5/P495-1 同族,是否扩日常门禁另行立项。

- **P510-1 字符串池生命周期债(索引 u32 化/池 GC/arg 侧残余)**:
  over-release 注入源已由 Plan 510 清偿(19 处无计数引用收口 +
  BUILD_FSTR/TYPE_TO_* 消费配平 + StakeGuard 池份额扩展;详见
  docs/plans/510-vm-pool-over-release.md)。仍在账:①池索引 u16 截断
  (natives get_string(u16) 上限 65535)——dedup 缓解,根治=索引
  u32 化;②池无 GC(freelist 复用已恢复,但峰值=并发不同串数);
  ③native 弹池串实参**不经 StakeGuard** 的裸 pop_arg_nv 路径仍漏
  配平(over-retain 慢泄漏,soak 量化后按需清偿)。旧引擎债指针
  docs/plans/060(闭包语法,主题不符)已接正至本条目。

- **P507-1 queue 臂投影保真边界集(登记在案,非静默)**:scroll 无裁剪
  (DrawOp v1 无 scissor)——**✅ 已清偿(Plan 515 G1,2026-09-01)**:
  DrawOp Scissor/ScissorPop(tag 3/4 追加式)+ 投影器 scroll 裁剪栈 +
  宿主 with_clip + TS 渲染器 save/clip/restore + e2e(p515-scroll-overflow
  构造示例,嵌套 2 层);typography bold/italic——**✅ 已清偿(Plan 515
  G2)**:DrawOp TextStyled(tag 5:weight/italic)+ 宿主 iced Font 差分 +
  TS font 前缀 + 金样差分行;余项仍在账:ul/ol li 列表标记不载;grid 缺省列数 2 且
  末行 gap 按满行扣;竖向 divider 无交叉轴可用高度(无 h- 声明取 24 近似);
  icon/avatar/image 为占位 Quad(位图与字形归宿主栅格化)。均在
  element_coverage.rs reason 串/代码注释随注——升格时机:DrawOp v2
  (clip/font 权重/位图通道)。

- **P507-2(pre-existing,非本计划引入)`demo::counter_loopback_demo_parity_with_direct_mount`
  libtest 并行竞态**:断言 `wid == 1` 假设进程内首个窗口,iced window-id
  全局计数器在同进程多测试并行下被抢占(507 新增测试改变调度后首现;单跑
  两版本均绿;nextest 每测试独立进程=日常档天然隔离)。偿还:demo 测试改
  相对断言(取孵化返回 wid 而非硬编码 1)。

- **P507-3 覆盖率数字默认档不可见——✅ 已清偿（Plan 515 G4 C1，2026-09-01）**:fence 测试写 `target/queue-coverage.json` 侧信道 + `cargo run --bin queue-coverage` 随时直读(open 项逐条)。原描:nextest 隐藏通过测试的 stdout,
  `[queue-coverage]` 行需 `--success-output immediate`(命令已档
  .cargo/config.toml 注释)。可选偿还:element_counts 挂 bin 或写
  状态文件。

- **P506-1(pre-existing,非本计划引入)038-minesweeper Reveal 触发 VM
  RC use-after-free**:首点格子即崩——`[RC canary] use-after-free: heap
  object ... was freed`(crates/auto-lang/src/vm/rc.rs:530,plan-453 T6
  边界捕获),app update/view 双 panic 后 MCP 连接断。主 checkout 与
  506 worktree 同现(与 fit/header 改动无关);触发面=store Reveal 的
  struct 字面量整板重建(`.board[idx] = { x: old.x, ... }`,Plan 511
  NEW_INSTANCE 构造字面量路径,疑 511 回归,待 511 复审验证)。影响:
  038 desktop_mcp.py T3-T6 被挡(506 已加防御:记 FAIL 不死锁 +
  flow skip);fit 桌面化验证由独立 probe 补证(647x878 vs 默认
  1293x836)。偿还:VM 侧修 UAF(likely rc.rs 池生命周期/构造字面量
  over-release 家族,与 P510-1 相邻),修后 038 套件应回全绿。

- **P506-2(pre-existing,非本计划引入)MCP rendered-vtree 快照无事件
  注记**:`autoui_snapshot`(styled_vtree 通路)不输出 `onclick:` 行
  (computed events 仅 F12 通路填充)——038 旧定位法
  (find_all_elements_by_event / `"onclick: .X" in snap`)在 master 已
  失效(T1 结构断言 0 命中)。506 已将 038 改 label 定位法
  (`button #id "label"`)修复一处;其余示例脚本若依赖事件行定位将踩
  同坑。偿还:styled_vtree 快照补 events 元数据(或 MCP 侧文档标注
  事件注记仅 F12)。

### P514（2026-09-02，Plan 514 W3 lib 方法化（γ4）执行期登记）

1. **~~P514-W3-1~~ ✅ 已清偿（2026-09-02 用户裁定：移除换行流式链糖）**：
   落地 commit 9264344cf：句首 `.method()` 语句不再自动合并进上一条语句，
   旧形态报非恢复语法错（单错无级联）；同一行链/方法体首语句/表达式位
   句首点不受影响；仓内遗留链迁移 5 处；cargo tf 3350 绿。W3 重启时 lib
   方法体语句位可自由用前导点（歧义源已除）。原始定性存档：
   P 方法化 lib 转译路径的挂起/CALL_SPEC 'Token.next' 崩溃根因是宿主 parser
   的**流式链糖**（parser.rs parse_body_inner ~:6846：块体内句首 `.` 语句合并
   进前一语句——musk 流式链设计）——方法体内**语句位**的 `.method()`（如
   skip_empty_lines 的 `count = count + 1` 后跟 `.next()`）被链成
   `(count+1).next()`，写丢失→死循环/错型分发。最小对：pairC（`i=i+1` 后
   `.bump()` 挂起）vs pairD（显式 `self.bump()` 正常）。**修复=lib 书写约定
   （零宿主改动）**：方法体内语句位方法调用用显式 `self.`；表达式位/赋值位
   前导点保留（链糖各守卫已豁免，m 探针族全绿实证）。已验证：type P 方法体
   3 处改 self. 后 99_unit 13 绿+corpus g 逐字符绿。
2. **~~P514-W3-2 主 a2r 方法体习语实参发射缺口级联~~ ✅ 已清偿
   （2026-09-02，Plan 514 W3-2 补丁续修，commit 8aef313ca）**：实际
   清偿量远超登记——主 a2r 8 处（存档补丁 a–d + e 真因=未注解局部注册
   Type::Unknown 而非缺失 + 字面量 .as_str() 误加 E0658 + 用户方法返回
   类型 qualified 键推断（含 merge 跨模块注入）+ 裸 self 变异判定 + 跨方法
   &mut 传递闭包 + 两实参环 infer 兜底）+ AA2R 镜像 3 组（ar_method_call
   方法表实参强转/返回类型、ar_scan_method_writes 变异调用扫描、
   ar_fixpoint_mutates 传递闭包）。五级联验证全绿（cc 编译红逐位清零/
   corpus 逐字符/19+13/⑤腿/矩阵 46/46）。原始定性存档：
   ②腿（主 a2r merge）编译 lib 方法体暴露连续缺口，逐条修复至 rustc 仅剩
   1 处：a) Vec 接收者 `.get()` 借用臂误命中字段实参（→索引形+as usize）；
   b) `self` 未注册宿主类型（方法体发射补 User 占位注册）；c) str 参位拼接
   Bina（→format! String）缺 .as_str()；d) Plan 376 StrSlice 误否决（仅
   str 参数应豁免）；e) 未完：`p.decl_lookup(name)` 位——另一条实参发射环
   （method_spec_flags 环，:8788+）未覆盖同款 str 强转。补丁（158 行，含
   a–d 全部修复）存档 `scratch/p514_w3_maina2r_methodfixes.patch`；翻转
   脚本存档 `scratch/p514_p_methodize.py`（类型体方法+全库 regex 翻转+
   语句位 self. 约定需补入脚本）。W3 重启清单：套 patch→补 e) →复跑本条
   五级联验证序列（cc 编译红逐位消）→W3-12 塔顶样板验证即通。

4. **P514-R1（复审登记，2026-09-02）`var list = List.new()` 两侧注解
   分歧**：主 a2r 发射 `let mut list = Vec::new();`（无注解+`.get(0)`），
   AA2R 发射 `let mut list: Vec<String> = Vec::new();`（注解+索引形）——
   预存文本形状分歧（非本计划引入，g17 探针实证后收窄为对齐子集规避）；
   语义侧由⑤腿 rustc 门覆盖（双方产物均编译）。处置随 P514-20 ②③ 同批
   （dump 判据层文本对齐另立计划时一并）。
3. **P514-20 Phase 11 收账余项三项（2026-09-02，Plan 514 步骤 20 显式
   登记处置；447 归档 447-aavm-prerequisites.md §10.4 原文在案）**：
   ②AA2R 单语句块臂不内联（主 a2r write_match_arm_body 内联）——语料
   无该形态暴露，对齐属 dump 判据层重构（与 D23/D27 同则另立计划）；
   ③List 实参克隆 parity（主 a2r is_owned_list_arg 无条件 clone vs
   AA2R last-use）——矩阵行为一致（46/46），文本形状差异随②同批处置；
   ④臂值位赋值表达式 aavm cg 不支持——写法规范（值位须纯表达式）保留，
   b32/g06 以绕开形态落盘。

### P517（2026-09-02，Plan 517 W1 折叠点①执行期登记）

1. **P517-1 ⑤腿塔顶程序环境敏感死亡/超时（非 517 引入）**：parity ⑤腿
   （AA2R 自译整 lib 塔顶程序，350KB 级）在当前高负载环境下两种形态均
   复现——rc=1 快速返回无任何错误输出（非 panic，RUST_BACKTRACE 空）与
   900s+ 不完成（447 期记录正常需 240-420s 且"超时线贴边"）。**master
   基线（未含 517 改动的 lib）同双形态挂**（scratch 对照实证）→ 非本
   计划改动引入；内存充足（11G/33.6G 空闲）排除 OOM；触发因子=并行
   会话构建负载下解释执行 700KB 程序的 CPU 竞争。**〔2026-09-02 核销注记：
   W2 折叠点②矩阵 46/46 全绿（环境恢复后⑤腿塔顶通过），矩阵补验即此清偿；
   环境敏感观察项保留（高负载下塔顶程序仍可能贴线）〕原登记：
   会话构建负载下解释执行 700KB 程序的 CPU 竞争。处置：折叠点①以替代
   证据链放行（g18 逐字符+rustc 组合运行+全量 19/0/0+99_unit 13/13+
   ③腿等价程序绿），**矩阵全绿补验挂账至折叠点②/复审**（环境空闲时
   复跑；届时若仍红则独立分诊）。447 期"环境相关异常终止族"同族。
2. **~~P517-2 矩阵运行前置纪律（自 P511-5 坑引申）~~ ✅ 已清偿（2026-09-03，
   Plan 524 W2）**：parity 启动闸门（`parity/crates/auto-parity/src/freshness.rs`）
   ——`--auto-binary` 统一解析为绝对路径（缺档报错含绝对路径+cwd，不再裸
   os error 3；顺带实锤原坑根因：`parity/../../target` 多退一级）+ mtime
   对账 `crates/` 树最新源，陈旧**硬失败**（文案含指认最新源+重建命令+
   `--allow-stale` 逃生）。三态单测 5 例 + 陈旧/逃生/缺档/相对路径四场景
   实测留档见 Plan 524 步骤 5。**原登记**：parity ②/⑤腿经
   `--auto-binary` 调 auto.exe——lib/trans 改动后必须重建该二进制
   （master target/debug/auto.exe 曾因陈旧+进程锁定致矩阵假红）；且
   worktree 跑矩阵需绝对路径传 `--auto-binary`（相对路径解析挂）。
   债候选：parity 启动时校验二进制新鲜度/统一路径解析。

### P511（2026-09-01，Plan 511 aavm 中阶语言能力执行期登记）

1. **~~AA2R ignored 腿 7 件预存转译债~~ ✅ 已清偿（2026-09-01，Plan 514
   W2 步骤 10）**：双根因修复——① lexer.at tokenize 主循环加 cur_char
   哨兵（码点自洽）；② 真因为主 a2r 通用 Bina 臂缺优先级括号（补
   auto_op_prec 表）。`test_aavm2_compile_corpus` 37/37 绿，
   --include-ignored 19/19 绿（原登记的 arr_flag 根因②定性有误，实为
   Bina 括号塌缩级联）。
2. **~~tv-aavm2 闸门口径偏差~~ ✅ 已清偿（2026-09-01，Plan 514 W2
   步骤 11）**：test_aavm2_compile_corpus 去 ignore 入常规门禁；
   vm-files-ci 第一层去 --include-ignored（口径收窄落地）。
3. **b41 语料处置（D1）**：`a[i] += e` 宿主编译错误（无发射序），
   aavm 同文本拒绝；b41 不入 corpus_m4，以 L3 99_unit 错误文本件承载
   （t_d1_index_compound_reject/t_d1_field_compound_reject）。
4. **限定调用占位泄漏镜像**：`db.fn()` 发射 const.i32 0 接收者占位
   （宿主 Ident 模块兜底，Plan 437 已知泄漏）——aavm 镜像之，帧 bp
   锚定下无害；宿主若清偿该泄漏需同步。
5. **~~P511-5 五方矩阵②腿编译期红~~ ✅ 已清偿（2026-09-01，Plan 514 W1
   步骤 3）**：主 a2r maybe_module_method 加 "File" 臂（std 直映）；
   修复后矩阵全程绿并作为 W3 行为不变判据贯穿（折叠点②③④各留档）。

### P512（2026-09-01，Plan 512 fit 动态重测 + 批二迁移执行期登记）

1. **P512-1 fit 宽度方向量测受视口钳制**：动态重测的 scrollable 方案
   （fit_aware_root 锚点外套 vertical scrollable）只给内容侧无限**高**
   约束；宽度仍钳在当前窗口/视口宽——内容自然宽超出窗口时量不到、
   窗口不横向跟随（011 Scientific 6 列 grid 实测未超 384 宽，未触发）。
   偿还路径：双向 scrollable（横向滚动语义侵入）或自定义测量 widget
   （layout 以宽上限排版、operate 回报子树自然尺寸）——后者语义干净，
   待有宽度增长需求载体时立项。
2. **P512-2 用户锁定后内容余量裸露出窗口底色**：fit 窗常驻 Shrink 根
   （512 前为首测后回 Fill），用户手动放大窗（锁定）后内容左上对齐、
   余量区无 Fill 背景填充（v1 语义，fit_aware_root 文档注释在案）。
   解锁增强（如双击标题栏恢复 fit）与锁定态视觉语义一并留待裁定
   （512 待澄清③）。
3. **P512-3 `p508_g2_outproc_arm` 并发偶红**：512 门禁全量跑时红一次
   （17.7s outproc 孵化用例，并行会话高负载下超时族），单跑即绿；
   与 512 改动面无涉（508 进程模型），按 489 族 flake 注记。

### P502(2026-09-01,Plan 502 diagram Phase 1 复审登记)

1. **P502-1(master 既有,非本计划引入)`plan055_strip_html_tests::strips_tags_and_decodes_entities` ui-iced 档红**:`strip_html_tags("<span>@x</span> 你好")` 产出双空格(`" @x  你好"`),期望单空格(aura_view_builder.rs:11263 断言)。本机 master(eda3a5a5e)同红复证——musk PLAN-055 批(c964ffa81)引入面;`cargo tf` 无 ui-iced 不触发,唯日常档 `cargo t` 可见。偿还:strip_html_tags 标签/实体后空白规范化(或按语义修正期望),归 musk 批主理方裁定。
2. **P502 执行期发现#9 处置核验**:kitchen-sink `@autodown/engine` scaffold 依赖缺失担忧经复审核验**已上游愈合**(P499-6 d0c23388d kitchen-sink 再生成,import 不复存在)——不入债。

### P513(2026-09-01,Plan 513 整合清理批——归档计划残留转账登记)

> 来源:407/412/420 三家归档时残留显式转账(G2 零静默);处置详情见
> `archive/407-minesweeper-rust-backend.md`/`archive/412-layout-gallery.md`/
> `archive/420-auto-edit-tabs-workspace.md` 头部终态注记。

1. **P513-1(转自 Plan 407)扫雷 rust 后端两项残留**:R7 动态窗口
   resize(difficulty 切换→窗口尺寸——ui_gen/rust.rs 零 resize 逻辑,生成
   main.rs 固定 `window_width: 370, window_height: 506`)+ Phase 4 三后端
   对比验证与 015/011 回归(无执行记录)。两项均可执行、无阻塞。
   P1–P3 大部已交付(merge `f863be5e`)。| `ui_gen/rust.rs` + `examples/ui/038-minesweeper` + archive/407 状态行
2. **P513-2(转自 Plan 412)layout gallery §10.4 视觉通道未闭环**:结构通道
   已绿(`4bfd6d27`/`6ee1a5e1`/`f51c8882`/`ccf9ac0e9`——12 layout 页全落地
   +rederive_layout 全路径+grid_row_placements 分配器);视觉+交互验证未
   执行——全页双端并排截图与像素测量(≤1px)、scroll/Overlay 交互抽验,
   §9.2/§9.3 验收标准未闭环(归桌面会话/autoui-verifier 轨道)。| archive/412 §10.4
3. **P513-3(转自 Plan 420)auto-edit 三项残留**:P4 tab 拖拽排序(纯 app 层
   可执行);挂账 #3 ActNew 偶发 InvalidOpCode(待复现);挂账 #4
   `.tabs[i].dirty` bool 读回乱码。P1–P3 已合并 master(`78aae68bb`)。
   | `examples/ui/041-auto-edit` + archive/420 §6

---

> **债务批二期候选清单指引（2026-09-01，Plan 513 文末）**：
> ① 桌面线开放工作项清偿 = **Plan 515**（desktop-debt-batch-2，2026-09-01
> 已立项；与 513 的账目域/清偿域划界见其变更摘要）；
> ② 语言/VM/测试红族等其余 ~25 条"值得近期做"清偿候选（P506-1 UAF、
> P499-1 timer 空转、测试红族 444×3/P487-2/P495-1/P496-2/P504-1/P507-2/
> P502-1 等）不在 513/515 两批范围，待另立批次（无既设计划）；
> ③ Plan 442 观察期 2026-09-03 到期——✅ 已执行（2026-09-03 当日
> /auto-plan:review 通过：到期日实跑 PARITY_TARGET=vm parity 6/6 绿、
> 无回滚证据，status → reviewed 待 merge 归档；债务入本簿 P442 小节）；
> ④ `scratch/schema_drift_audit.py` 为 schema_drift 门禁脚本唯一副本（门禁
> 引入 `78a9f138c`，检查已闭合在 cargo t 内）——是否 promote 至 `scripts/`
> 正式化**候裁定**（Plan 513 待澄清②，复审落格；默认不动）。

### P515（2026-09-01，Plan 515 桌面 DEBT 批处理二期——清偿/判定/D 族收拢登记）

- **清偿五项**：P507-1（scissor + typography 两子项，见上条内联 ✅）、
  P503-2（vue 壁纸层）、P507-3（覆盖率可见）、P500-1（auto_exe 陈旧
  防护）、P504-3（真 launch e2e 通道留痕）。
- **D 族判定收拢**（六项散落增强逐项定案，判定表见
  docs/plans/515-desktop-debt-batch-2.md 详细设计 §5）：
  - **纳入已执行**：D1 HICON 真图标（473/486 两度延期）——提取链
    （WM_GETICON 哨兵容错 + GCLP 兜底 + CopyImage 回退）+ native_icon
    缓存 + `hicon:<slot>` 渲染方案串；真窗口 e2e 绿。
  - **显式不做（理由成文）**：D3 外来虚拟文件拉流（低频边缘场景 +
    跨进程文件流协议/生命周期复杂度高）、D4 真延迟回调（合成 tick 已
    满足动画；定时器线程+跨线程回调安全复杂度/收益不成比例）。
  - **挂起（触发条件明确）**：D2 窗口选择器面板（触发=真机使用反馈
    出现多窗口切换需求）；D5 native DWM 缩略（触发=494 真洞翻默认后
    缩略保真反馈）；D6 vue 真缩略 web 路径（触发=vue 远程形态落地）。
    挂起项非永久悬置——触发即重开。
- **顺带修复（本计划施工中发现）**：layout_block 垂直块高度 500 期
  bug（误取 cross_max=最大子宽，多层嵌套容器高度消费一直拿宽度值；
  parity 金样 2 份几何修正=card 紧包内容/重叠消除）；geometry scratch
  类名 `\0` 转义 bug（非 NUL 终止，RegisterClassW 靠未定义读界）；
  drawlist-renderer package.json 漏提交（508 期 manifest 不在库，
  vitest 验证命令不可复现已补齐）。
- **P515-R1 vue 壁纸层渲染级对拍缺位（复审登记）**：515 交付的壁纸层
  验证止于 token/markup 级（scrim 双段类名与 VM pct 常量同源钉 + 三档/
  层序/转义单测）；渲染级截图对拍与 vitest 组件测未做——根因=vue 桌面
  宿主（生成项目）无 in-repo 视觉证据通道与测试装配（505 验收通道仅
  覆盖 iced 轨）。触发=vue 桌面宿主证据通道落地（与 D6 vue 远程形态
  同源），届时补双轨壁纸截图对拍。

### 调度会话（2026-09-02，vm 桌面实机核查发现）

- **SCHED-1 `auto run --desktop` 对 vm 渲染静默无效（CLI UX 缺陷）**：
  `--desktop` 仅置 `AUTO_DESKTOP=1`（crates/auto/src/main.rs:1023），唯一
  消费方是 auto-man vue 生成（Plan 465 脚手架宿主）；`-r vm` 组合下该
  flag 被静默忽略——仓库根跑则报"No pac.at"，app 目录跑则开单 app 窗口，
  **均不是 vm 虚拟桌面**（vm 桌面唯一入口 = `cargo run -p auto-lang
  --features ui-iced --example ui_desktop`）。用户因此入口混淆曾长时间
  误判"视觉刷新无变化"。修复方向：`--desktop`+vm 渲染组合显式报错指路
  ui_desktop，或 auto run 增 vm 桌面宿主路由。**二次实锤（同日）**：
  `cargo run … | head -30` 管道在编译中途截断 cargo（head 吃满 30 行
  warning 即退），构建未完成，随后直跑 `ui_desktop.exe` 仍是 08-31 陈旧件
  （mtime 铁证；注册表 36 条 vs 新构建 37 条亦为版本指纹）——用户当场
  识破"窗口变回 503 前旧 chrome（右侧单 ×）"。P500-1 的 mtime 防护若在，
  两次事故都能拦下。附：实机核查同时证实
  503 七项样式在新构建中确有渲染（圆角图标格/窗口细边框/主题协调），
  但 4px 运行圆点/细竖条常规尺度不可感知、dock 图标 glyph 占位色块质感
  （HICON 债族表现）、默认无图片壁纸、阴影弱——"观感未变"主因回到
  503 的设计层裁剪（无 blur/无动效/token 级范围），视觉二期（P2/P3）
  立项依据。
- **新观察（pre-existing，非本计划引入，供复审/后续定界）**：
  ①`style_migration_probe` 基线即红（token `underline` 上游已能解析
  但 449 迁移表标注 gap——测试自述"更新 MIGRATION.md 支持度列并翻
  ok"）；②`d8_toggle_dark_mode` 基线即红（断言 initial dark_mode
  =false 但读回 true——疑全局主题态测试间泄漏或 458 线回归）；③
  `plan055_strip_html_tests::strips_tags_and_decodes_entities` 基线即
  红（双空格 vs 单空格，strip_html 段落归并口径）。三处均经干净基线
  复跑证实（git stash 后同败），域属 449/370/055，未在本计划处理。

### P522（2026-09-02，Plan 522 helper-fn-into-vue-sfc 复审登记）

- **P522-1 同文件 module_fns / store composable 路径尾表达式体丢 return**：
  Plan 522 给 use 导入 fn 发射路径换用 `transpile_body_as_return`（尾
  表达式即返回值，修 `is_leap` 类谓词在 Vue 运行时返回 undefined），
  但**同文件** module_fns 发射（vue.rs module_fns 块）与 store
  composable 的 module_fns 发射（generate_store_composable*）仍用
  `transpile_handler_body`——同形态 fn 在两条路径语义不一致，裸表达式
  体的同文件 helper 在 Vue 侧仍会静默返回 undefined。偿还路径：两处
  同步换 `transpile_body_as_return` + 回归 goldens（发射文本变化面）。
  引用：`ui_gen/vue.rs` module_fns 发射块与 `generate_store_composable_full`；
  对照 `emit_use_module_fns`（已修路径）。
- **P522-2 auto-man components/ 通道 module_fns 未挂**：components/ 包
  通道（auto-man vue.rs ~2450）重生成 SFC 时已挂 use-fn 池（Plan 522
  T4），但同文件 module_fns 仍未挂——组件文件内的同文件顶层 fn 在该
  通道依旧丢失（主通道 generate_component_from_file 路径正常）。
  偿还路径：with_module_fns 同步挂入。引用：`auto-man/src/vue.rs`
  components_pkg_dir 通道。
- **P522-3 024 图表族 50 个既有 vue-tsc 错误（非本计划引入）**：024
  gen 工程 `pnpm run build` 报 50 错（四图表组件：withDefaults 推断
  TS2322 ×4、`__timer_*` 隐式 any、`e.currentTarget` 可能 null ×N），
  全部为 Plan 522 之前形态；dc/ds 迁移零新增错（复审逐条甄别证实）。
  437 记录的"dist 正常系陈旧产物"与此互证——024 vue 构建从未干净过。
  偿还路径：图表组件生成面专项（props 推断/timer 类型/currentTarget
  断言三类）。引用：`examples/ui/024-charts/gen/front/vue`（再生即现）。
- **P522-4 纯 fn 模块在 components/ 通道每轮告警噪音**：`chart_geom.at`
  类纯 helper 模块（无 widget/store 声明）在 `auto run` 的 components
  通道每轮报"Failed to compile … No widget or store declarations"
  （Warning 级、不阻塞、产物正确）。actions-only 模块在 front/ 通道有
  优雅跳过先例，components/ 通道未对齐。偿还路径：通道内对无声明文件
  静默跳过（镜像 actions-only 分派）。引用：`auto-man/src/vue.rs`
  compile_at_to_vue Err 臂。
- **基线红测试交叉引用**：`d8_toggle_dark_mode` / `strips_tags` /
  `style_migration_probe` 三处基线即红已由前计划登记（见本节之前
  "新观察"条目）；Plan 522 复审实证 d8/strips 在 master e6885460b 仍
  红（style_migration_probe 已由 Plan 518 496032e21 修复，522 merge
  后转绿）。

### P442（2026-09-03，Plan 442 跨平台合龙复审登记）

- **P442-1 rust/a2r adapter 轨递延（§6.1 用户裁定，非静默）**：B 阶段五域端口的
  `X.rust.at` adapter（platform/composables/icons/renderer/upload）与 A5 sched 的
  rs.at shim 均未落——VM 轨全部落地或显式降级，rust 轨待 a2r 消费面明确后批量补。
  偿还路径：musk 启用 a2r 后端轨时按 `ports/*.rust.at` 映射表补齐（442 §4-B 各条
  已列内容物）。引用：`auto-musk ports/`（web 侧 re-export 壳为参照）。
- **P442-2 svg 能力 partial（A4 登记面）**：动态 svg 属性/动画不支持；SVG text
  子元素未支持（与 DSL text→span 冲突）；musk icons 渲染层解除待 Icon widget
  实现 + renderToString 对拍升级（musk 侧独立小任务，musk KNOWN-DEBT 038 条已
  更新"解除条件已达成"）。引用：`crates/auto-lang/src/ui/render_support`（partial
  登记）。
- **P442-3 VM marshalling 语义债（442 §C1 wave-1 登记）**：PathBuf 句柄
  `.starts_with` 返回值错（0）；HashMap str 键 insert/get 往返空。**疑似已被
  08-31~09-01 PLAN-053 字符串池/freelist 整改与 Plan 510 over-release 清偿顺带
  修复（同窗口大改 marshalling 咽喉）——偿还时先复测再修**。引用：
  `ffi_dual 015_musk_backend_wave1`（原回归锁定处）。
- **P442-4 e2e 家族双层撞号（2026-09-03 复审发现，同日随 Plan 442 修复——
  提交 87eb8a730）**：双层根因——① e2e 测试端口撞号 ×3（18744/18745/18736，
  nextest 并行下输家连到赢家服务器空 body 假红，串行不暴露）；② native
  catalog ID 撞号（真产品缺陷，08-27 起）：PLAN-044 将 musk_extern_dispatch
  登到 3129 与 442 的 value_get_bool 撞号，id→shim 后写覆盖，
  `e2e_value_accessors` 净树 500 一周。修复：端口去重 + e2e_ports_unique 守卫 +
  dispatch 移段 3143 + catalog 家族靶向钉（negativity 三态验证）。**残余（未偿）**：
  `test-http-e2e` 家族仅存 `cargo th` 手动档，`cargo t`/`tf` 均不编译——本次
  产品级 ID 撞号漏网一周正因于此。偿还路径：th 家族纳入日常档或 CI 定期跑。
  引用：`crates/auto-lang/src/vm/ffi/http_server.rs`（e2e_ports_unique）、
  `crates/auto-lang/src/vm/native_catalog.rs`（家族钉）。
- **P442-5 native ID 通用撞号检测在别名设计下不可行（2026-09-03 登记）**：
  NATIVE_ID_ENTRIES 有 108 组同 native 多名别名有意共享 ID（限定名/短名/类型
  名/裸名，如 auto.hashmap.new/Map.new @120），静态表无法区分别名对与真撞号；
  且 5 组别名组返回 Type 不一致（103/120/170/175/1516，含 str.len 双登记），
  2850 组 auto.cell.once_new×auto.io.write_text_async 同挂疑为历史误登记。
  P442-4 修复只能做 442 家族靶向钉。偿还路径：注册侧运行时唯一性（bind_shims
  后 id→shim 单射断言 / shim 注册带 native 身份标识）；顺带审计 5 组 Type
  不一致与 2850 组。引用：`crates/auto-lang/src/vm/native_catalog.rs`
  （catalog_integrity_tests）。

### P524（2026-09-03，Plan 524 宿主小修批——回归期存量红发现登记）

- **P524-1 `cargo t`（ui-iced 日常档）两处 master 存量红（非本计划引入，
  db9bfc977 基线同红实证）**：① `plan370_015_behavior_tests::d8_toggle_dark_mode`
  ——1f7313e93（2026-09-01，015-notes 暗色主题默认化）把 store `dark_mode`
  初值翻 `true`，plan370 测试期望 `"false"` 未跟（断言三连首条即挂）；②
  `plan055_strip_html_tests::strips_tags_and_decodes_entities`——strip_html
  空白折叠行为漂移（`" @x  你好"` vs `" @x 你好"` 双空格）。两者均
  ui-iced/ui-interpreter 门控，`cargo tf`（不带该 feature）不可见——故
  PLAN-041 合入门禁（tf 3396/3397）未拦住，日常档自此带红。偿还路径：
  ①更新 d8 断言或回退示例默认（语义裁定归 015-notes 属主）；②strip_html
  空白折叠语义对照 055 期单测意图裁定。同族存量（在档）：schema_drift_fence、
  aavm2 m4/m5 corpus。
- **P524-2 CLI crate（crates/auto）单测不在 `cargo t` 别名内**：`cargo t`
  固定 `-p auto-lang`，crates/auto 的 clap 解析单测（Plan 524 步骤 3 六例）
  需显式 `cargo test -p auto --bin auto`。偿还路径：日常档扩展 `-p auto`
  或 CI 显式补跑（改动 nextest 别名影响全仓日常档节奏，留裁定）。

- **~~P523-1 `auto build`(pac) 最小工程 rust target 生成缺口~~ ✅ 已清偿(2026-09-03,Plan 531)**:三缺口修复——①rust 后端路径补 `pac.resolve()`(autos 扫描填充;缺位时 has_auto 全 false 转译空跑,CargoBuilder 落桩 main.rs);②无名目标包名回落 pac.name;③auto-lang 依赖按生成代码实际 a2r_std/auto_lang 引用条件添加(无条件 `"*"` 注册表兜底在仓外工程必败)。验收:`scripts/aavm_build_smoke.sh` 增 pac 正路双例,四例全绿(b07=55/b34=10-20 × trans+pac 双路)。**api-example 前端 strict 红独立处置维持**(与 target 生成链不同根因,裁定留档 531 待澄清③)。
- **~~P523-2 aavm.at a2r 模式入口两洞~~ ✅ 已清偿(2026-09-03,Plan 531)**:①argv.get 解包——`infer_type_from_expr` 增 `process.args()` 臂(List<str>,P524 契约),绑定型驱动 `.get(1)` 索引化+实参 `.as_str()` 借用,merged 转译 aavm.at(全 lib)无垫片构建 `b07→55` 实测(回归锚:`aavm_at_mode_tests.rs`,#[ignore] 验收档);②转译版 struct 字段表——**实测不重现**(无垫片 b34→10/20 全链复证;判断 525 codegen 早注册批已顺带清偿,登记后未复测)。
- **~~P523-3 tt 档(a2r test-trans 金样)28 件存量过期~~ ✅ 已清偿(2026-09-03,Plan 531)**:两件真缺陷修复(pointer/004 `x.@` 误入 dot_arg_owned_param clone 支路→排除;arc_dyn_spec/008 `self.<field>` 无条件 List 索引化→`recv_is_list_like` 实型解析,`current_impl_type` 新增)+其余 27 件批量 bless(逐件 diff 评审归类留档 531 计划文件)+tt 入复审清单(`.cargo/config.toml` tf 注记块:复审另跑 `cargo tt`,沿 507 desktop_protocol 先例)。`cargo tt` 3746/3746 全绿。
- **P524-3 plan worktree 深度下 autodown-core 跨仓 path 依赖解析断裂**：
  `crates/auto-lang/Cargo.toml` 的 `autodown-core = { path = "../../../auto-down/..." }`
  按主检出深度写死——`.worktrees/plan-NNN-dev`（两层深）内任何 nextest/
  `--all-features` metadata 解析都会指向 `.worktrees/auto-down/...` 而挂
  （Plan 524 执行期实证；本地以 junction
  `.worktrees/auto-down → D:\autostack\auto-down` 临时解）。偿还路径：path
  改相对 repo 根不可表达（Cargo 限制），候选=cargo `[patch]` 档案/环境级
  `CARGO_AUTODOWN_DIR` 文档化/构建脚本预检；至少把 junction 手法写进
  auto-plan work 技能的环境注意事项。

### P527（2026-09-03，Plan 527 VM 轨 Tailwind 全量覆盖契约——不做/受限台账）

与 `docs/style-coverage.md`（UNSUPPORTED 白名单 + Parsed-only 豁免台账，测试
同源再生）互链；逐类状态常驻机器断言 `crates/auto-lang/tests/style_parity.rs`。

- **P527-1 原生无语义族（永久不做）**：float/clear（无浮动文档流）、
  columns-*/break-*（无分栏/分片宿主）、print/伪元素（content-none 等）、
  aspect-ratio（无原生）、table 显示模式与 tables 家族、SVG fill-/stroke-
  （图标走字体/着色管道）、isolation/appearance/pointer-events/touch-/
  will-change/forced-color-（宿主/OS 层接管）、混合模式（mix-blend-/bg-blend-）。
- **P527-2 宿主上限受限族**：whitespace/word-break 细分（cosmic-text 换行
  策略仅 word/none，nowrap 已接）、装饰线型/粗细/偏移（decoration-* 家族）、
  text-transform（渲染层无字形变换）、object-position、inset/top 等百分比与
  auto/full 档（无容器查询）、w/h min/max/fit（无内容尺寸查询）、tracking
  全档与 focus 变体按钮面（iced 0.14 无 letter_spacing / button Status 无
  Focused）、单侧边框宽/色分档（PLAN-050 C2 1px 填充条模拟边界）、滤镜系
  （Plan 518 G8 声明冻结先例）。
- **P527-3 w/h 分数 Fill-ratio 近似口径（待澄清③裁定）**：无容器查询语义
  下 N/M → iced FillPortion(n)——同分母互补分数比例保真，混分母组合退化
  等分；显式像素计算需父宽布局期注入，偿还路径=renderer 布局期二阶段解析。
  引用：`class.rs` SizeValue::Fraction / `iced_adapter.rs` convert_size。
- **P527-4 变体管道分期消费**：focus/active/disabled 可声明可合并
  （IcedStyle::merged_with_variant），v1 仅按钮族真消费（Hovered/Pressed/
  Disabled 状态回调）；focus 在按钮面为 iced 0.14 上限（Status 无 Focused
  档）；非按钮 widget 的变体类登记 parsed-only（coverage 表可见不静默）。
  偿还路径：text_input/radio 等 iced 状态样式面接入。
- **P527-5 渲染层分期消费的存字段类**：order-N（按源码序渲染）、flex-basis、
  ring 宽/色/inset（focus 环模拟）、inset-N 四向 offset（无 Stack 上下文时
  内联降级）、line-clamp（行高裁剪）、letter_spacing（tracking）——IR 在册
  applied，renderer 消费排期。引用：`iced_adapter.rs` 对应字段注释。

### P529（2026-09-03，Plan 529 worktree 分组平铺布局迁移——复审登记）
- **P529-1 wt-guard-hook 相对路径盲区**：hook 层仅对命令中以绝对路径（盘符/`/` 开头）
  出现的目录参数做 reparse 扫描；相对路径形态的高危命令（如事故原命令
  `git worktree remove .worktrees/plan-058-dev`）不会被 hook 拦截。缓解在位：
  fold/merge 流程（技能与 AGENTS）强制先定位绝对路径再跑 `wt-guard.sh`，hook
  本身是可选第二层。偿还路径：hook 内解析 cwd+相对路径（需 hook 输入携带
  工作目录）或改用 fsutil 全盘 watcher 级方案。
- **P529-2 new-plan.sh 不自动建组**：建 `.wt/<repo>-<NNN>/<repo>` worktree 仍由
  会话按 AGENTS 命令模板手动执行，约定依赖指令面（6 文件）+复审流程兜底。
  偿还路径：new-plan.sh 加 `--wt` 直接建组建 worktree。
- 观察项（非债）：PreToolUse hook 样例（D:/autostack/hooks-config.sample.json）
  未启用，等用户决定；`.worktrees/auto-down` 为 17:08 某会话创建的普通目录
  （非链接，无穿透风险），首个新布局计划落地时可顺手清理。
- **P529-3 specs.json 本地台账事故后重建仅含 P529 六项**：原 `.autoos/specs.json`
  （含 P442/P517/P523/P524/P526/P527 等历史沉淀条目）随 2026-09-03 删除波丢失
  （gitignored 不入库），Plan 529 merge 时以六空节骨架重建并仅沉淀 P529-1..6。
  偿还路径：对 docs/plans/archive/ 各归档计划逐一重跑 deposit 脚本（scratch/
  p515/p517/p509 等脚本留存可改造为通用遍历版），幂等 id 保证不重不漏。

### P530（2026-09-03，VM 双份绘制/721GB 崩溃专项——执行期登记）
- **P530-D1 Breadcrumb 页栈溢出（存量 master 缺陷，非 530 回归）**：导航到
  breadcrumb 页必现 `thread 'main' has overflowed its stack`，主检出
  master@96586cca 对照构建同样复现（归因实验留档 scratch/p530/）。此前被
  OBS-1 721GB 崩溃掩盖（扫描先死于 code-editor 页），B-2 修复后暴露。
  偿还路径：breadcrumb 页视图构建递归（疑 ForLoop/嵌套 link 链）单独
  立项 bisect，debug 构建抓栈。
- **P530-D2 图表 timer 路由切换不退订**：LineChart/DonutChart 的
  `AnimLnTick/AnimDnTick`（every_ms:33）订阅随首页卸载后仍以 30-60/s 投递
  （离页后日志持续刷 tick），`when` 门控丢弃但每条消息仍触发一次 view()
  全量树重建（空转功耗+泄漏倍增器）。偿还路径：路由/组件卸载时同步
  退订 widget_event_tick 订阅（renderer 订阅面按存活组件过滤）。
- **P530-D3 Element 缓存快速路径架构性失效（空转重建）**：dynamic_view
  末尾 store-then-take 使 cached_rendered 恒 None，`dirty=false` 帧仍走
  cached AbstractView → 全量 iced 树重建（实测 47k tick 仅 7 次 dirty，
  4.1 万次全重建）。iced Element 不可 Clone，注释承诺的"同 Element 复用"
  不可达；偿还路径：评估 iced `lazy`/组件化包层做帧间跳过，或接受重建
  但以 D2 退订消除空转触发源。
- **P530-D4 诊断门控留档**：`P530_TRACE=1`（LayoutCollector 重复 id 记录
  + view/resize 宽度轨迹）、`P530_NOMCP=1`（跳过 per-frame MCP 同步/
  capture 路径，A/B 判别用）两 env 门控留存于 renderer/layout_collector，
  零成本（env 缺省关）；后续排障可复用，偿还（删除）非必需。

## P528 债务（widgets-gallery 检查跟踪,2026-09-03 复审登记）

- **P528-D1 浅色主题硬编码区**（源 OBS-4）:页面内硬编码深色样式(codeblock
  代码体 bg-zinc-950 等)在浅色主题下不翻转;token 驱动区正常。浅色逐页适配
  属示例资产工程,量级 60+ 页。优先级低。
- **P528-D2 主题状态持久化**（源 OBS-5）:dark_mode/accent 为 App 内存态,
  刷新/深链回默认深色;可接 localStorage(015-notes storage 先例)。优先级低。
- **P528-D3 model 逗号分隔声明解析缺陷**（源 OBS-8）:`model { a str = "1",
  b str = "2" }` 解析出空名变量+名字错位(生成 `const , = ref`)。家规为换行
  式;parser 增强候补(容错或报错),非阻断。
- **P528-D4 bundled shadcn-ui 快照滞后**（源 OBS-11）:auto-man/assets
  shadcn-ui 快照为旧版(toggle-group 无新版连体设计);其余组件可能同样滞后
  上游。快照升级影响全工程,关联 OBS-10 远期"脱离 shadcn 基底"战略,届时
  统一重议。
- **P528-D5 环境事实二则**（源 OBS-3/7）:①AutoUI MCP 默认端口 9247 与本机
  musk.exe 冲突,须 AUTOUI_MCP_PORT 绕开;②.auto/ui-cache.json 内容寻址,
  mtime touch 不触发生成——改 codegen 后必须删缓存重生成(误判高发)。
- **P528-D6 存量双红**（源 OBS-6/9,2026-09-04 复核更新）:tf 档 2 红——
  docs_gen kitchen_sink_page_in_sync(fork 预存在案)+schema_drift_fence
  (漂移已消除,baseline 待 SCHEMA_DRIFT_UPDATE_BASELINE=1 裁剪)。原始 4 红
  中 d8_toggle_dark_mode/plan055/gallery_vue_golden 已在重建历史他席修复。
- （注）vue 包装 exit 127 静默退出（源 OBS-2）疑与 530-B 泄漏族同源,530
  修复后待复跑观察,暂不单列。

### P539（2026-09-04，Plan 539 PyTorch FFI——执行期存量红/缺口登记）

- **P539-D1 py_list `test_sorted_getitem` master 存量红**（非本计划引入，
  master 二进制同形复现：`[P053-8] phantom freelist entry dropped: slot 41
  (live holders, rc=4294967295)` + `got d`）。py_list 属 p7 相位，近期
  各计划门禁只跑 p5/p8/p9 相邻相，从未显形。疑与 ADD 字符串拼接臂的
  双重 rc_release（engine.rs ADD string-concat 分支两对 release）或
  Plan 510 G 系池工作交互有关——待专项排查（rc 配平 forensics）。
- **P539-D2 `.len()`/方法分派对 py 返回值不可靠（存量，类型谎言）**：
  py 调用返回被 codegen 谎记类型（fn_return_types=StrFixed 等），`.len()`
  静态路由到 str.len，把句柄/列表 id 解码为字符串池索引（垃圾但常在
  界内，读到池内真串长度——实测"20"）。规避：for-in 计数、`x[0]` 索引
  （GET_ELEM tag 分派）、`py_call(x, "__len__")`。RuntimeArray 谎言翻转
  试验无效已回退。根治需动态分派（独立计划）。
- **P539-D5 py_call_may 仅位置实参**：kwargs 与 May 通道组合未支持
  （py_call_kw 是 strict 语义；py_call_may 弹参走位置约定）。需要时用
  `py_call_may(py_call_kw 形态兼容路径)` 前先以探针定 ABI；影响面小
  （训练循环捕获路径用 try-catch 或 `.?` 兜底即可）。关联存量：
  a2py 语句体闭包降级为 set 字面量（`(x) => { x * 2 }` → Python set），
  表达式体必需——见 libs/python README 回调节。
- **P539-D3 a2py 复合接收者无括号**（存量）：`py_call(t == t, "sum")`
  发射 `t == t.sum()`（优先级错）；套件用中间变量规避。
- **P539-D4 py_subclass 类派生延期（计划内预案路径）**：自定义
  nn.Module/Dataset 需 Python 侧类工厂（exec 生成类 + 方法绑回 Auto
  回调）。回调桥 T21 已通（thread-local 任务槽，map/apply_ 双探针
  实证：Auto 闭包经 PyCFunction::new_closure 回投当前任务），
  但类工厂的方法绑定面 + GIL/生存期约束审查超 W3 预算。组合式
  替代金样 = py_torch_train（Linear 裸栈 + seed 化收敛）已在案。
  调研节落 python-parity-roadmap.md §7.3。

### P537（2026-09-04，Plan 537 photo-gallery 执行/复审登记——examples 层实证的基建缺口二则）

- **P537-D1 VM lucide 图标闭集缺口**:`iced/renderer.rs lucide_svg(name)` 为
  84 项闭集,examples/ui 层 `icon (name:)` 只能消费表内名;计划 537 原拟的
  images/heart/mountain/building-2/cloud-sun/sparkles 在 lucide-vue-next
  存在但不在 VM 表（Vue 端正常/VM 端缺渲染=双端不一致）。绕开（已落地）:
  相册图标 emoji 文本（027 先例）;`icon` 仅用双端表内名（sun/moon/
  chevron-left/chevron-right）。根治:VM 表按 lucide-vue-next 常用面扩充
  （或建立单测围栏对齐两端名单）,建议随 widgets 双端 parity 批处理。
  引用:`crates/auto-lang/src/ui/iced/renderer.rs` lucide_svg;
  `examples/ui/029-photo-gallery/SPEC.md` 差异注记 2。
- **P537-D2 VM 语义 grid 的 cols/class 状态绑定不解析**:`grid { cols: .state }`
  回落 1 列（eval_u16_prop 对 widget 状态引用不解析,unwrap_or(1)）;`class:`
  状态绑定 VM 不消费;`style:`(if 静态臂 grid-cols-N)叠加语义 grid 曾实测
  尺寸异常一次（未深究根因）。绕开（已落地）:密度 2/3/4 以三臂静态
  `grid { cols: N }` if-链展开（028 静态 cols 同构造,双端一致实证）。
  根治:builder 的 cols/类仲裁路径补 widget 状态引用解析;补双端
  "动态列数"回归锚。引用:`crates/auto-lang/src/ui/aura_view_builder.rs`
  grid cols 提取（extract_u16/eval_u16_prop）;029 SPEC.md 差异注记 4/勘误 2。
- （注）`.photos` 主列表为 Init 构建的 write-only 状态（计划文本明确要求
  构建,view/handler 仅消费 view_list）——保留不删,登记为设计内冗余。

### P533（2026-09-04，Plan 533 VM 悬浮层运行时通道——执行登记）

- **P533-D1 overlay 家族 Phase 2 余量**：tooltip/hover_card/select/combobox/
  drawer/sheet/command/context_menu/menubar/nav_menu 等约 33 个 overlay 语义
  元素仍 iced:none/unknown（schema 回填仅覆盖 alert_dialog×9/dialog×8/
  dropdown×15 实现族）。Phase 1 视范围裁定为 musk 三场景所需三族;余量按
  musk PLAN-059 T5-T8 铺开节奏另批。
- **P533-D2 MCP 合成键盘不经 overlay**：autoui_keyboard 直派 VM handler
  （key_bindings→config→key_<k> 回退）,不进 iced overlay update——浮层 ESC
  自动化只能走真键（computer-use/SendInput）或 iced_test 单元。自动化验收
  口径需知悉;musk 侧 T9 联测如需 MCP ESC 需扩 mcp_server 路由。
- **P533-D3 snapshot 含关闭态 overlay 子树**：VTree/snapshot 恒含 Popover
  content 子树（open 与否）,open 断言以 autoui_state 为准;"弹层不在文档流
  父链下"的结构断言在现行快照口径下不可表达——open 态 overlay 层呈现口径
  （待澄清③）留 musk 侧验收自动化前定案。
- **P533-D4 gallery 整仓 rust 轨存量红**：examples/rust-workspace widgets-
  gallery 生成物含壳层词汇（SettingsPanel 自定义组件/icon/scroll/openSidebar
  on-only 带参等）超 rust 轨能力,12+ 编译错为存量（与本计划改动无关）;
  页级 codegen 产物断言（test_gallery_alertdialog_page_codegen_contains_modal）
  为现行门禁。rust 轨 gallery 全绿需专项（视图词汇覆盖面工程）。
- **P533-D5 模态面板宽偏离**：计划文 T4 为 min(480px,90vw),实现取 w-96
  （384px）与解释器轨 PLAN-530 W13 同串保双轨对拍一致。如需 480 档,两轨
  同步改面板 chrome 一处各一行。
- **P533-D6 显式绑定 dialog 的 ESC/外点不接管**：dialog (open: .x) 显式
  绑定形态 on_dismiss=None（自管语义,Phase 1 记录）;仅铸造形态（无 open
  绑定）折算 __dlg_close_N 回流。显式绑定需 ESC 关闭时用户自接 ondismiss。
- **P533-D7 丢失工作重做归档**：auto-musk-dev 分支（已删未合回）三件
  （PopoverPlacement::Modal/popover 模态三语义/aura_view_builder 臂）经查
  已由 PLAN-530 步骤8（源 W13）先行落地,本计划 T2 转为验证面+补 iced_test
  四断言;child_emit 大小写折叠（musk PLAN-059 T2）按重做处理（T1,净修好
  存量 1）。
- **P533-D8 on-only 带参 handler 悬垂**：rust 轨 on-only 带参 handler
  （gallery 壳层 openSidebar 形态）枚举注入零参变体与派发带参闭包不匹配
  →编译响亮失败（payload 类型无法从 on 块推断,保持显式失败不静默）。

### P543（2026-09-04，知识库同步基线独立复审登记）

- **P543-D1 源码数量缺少 canonical 计数口径**：PLAN-543 文档将核心 Rust 文件写为
  “约 528”；复审时 `rg --files crates/auto-lang/src -g '*.rs'` 得 520，而
  `git ls-files 'crates/auto-lang/src/*.rs'` 得 530（ignore/枚举口径不同）。现有文档已明确
  该值仅为审计数量级且不可长期手工维护，因此不阻断本计划；根治归属 Design 27 阶段 B
  Knowledge lint/catalog，以 Git 跟踪文件和 workspace metadata 生成唯一 inventory。
- **P543-D2 Design 26 遗留未勾选样板项**：`docs/design/autoplan-spec-ledger.md` 的
  “Plan 467 落地清单”仍保留一条历史 `[ ]` 首个完整循环样板项；PLAN-543 新增的 §9 已
  说明当前兼容期与后续工作包，但旧 checkbox 可能被误读为当前 blocker。归属阶段 C
  Auto-plan v3/Design 26 收敛时改成带日期的 historical outcome，不在本轮入口校准中扩 scope。

### P550（2026-09-05，null 家族审计与术语统一——行为翻转登记）

- **P550-D1 越界/TYPE_TO 行为翻转的存量语义变更**：GET_ELEM 越界
  （Auto 数组四型）从静默 `push_i32(0)` 翻为 `IndexError: index N out of
  range`；TYPE_TO_I32/F64 对 null 从静默 -1/-1.0 翻为 TypeError——这是
  **计划内行为变更**（对标 Python），`cargo tv` 3585/3585 全绿证明存量
  语料零依赖旧哨兵。若有仓外/未入库语料依赖 0/-1 哨兵，属预期翻案面。
- **P550-D2 越界翻转未覆盖 str 索引/by-name 缺字段**：GET_ELEM 的字符串
  索引越界（push 0）与 ObjectData/GenericInstanceData 按名缺字段（push 0）
  未翻 IndexError/KeyError——守卫矩阵仅列 Auto 数组；str 越界对标 Python
  IndexError 属后续波次收口面。
- **P550-D3 历史 i32 哨兵编码不在算术守卫范围**：算术族守卫只拒 TAG_NULL
  （null/nil/None 三拼写经 PUSH_NIL 同落 tag-null，PLAN-053 归一）；历史
  i32 哨兵（-1 / i32::MIN+1）与真实整数在算术槽不可区分，无法守卫——
  持久化旧数据经算术仍产垃圾（EQ 判等的 null-family 兼容语义不变）。
- **P550-D4 CALL null 守卫无 .at 探针面**：正常模式 null callee 被静态
  解析在编译期拦下（E0401），VM 层 CALL_CLOSURE 守卫（'NoneType' object
  is not callable）仅动态/脚本路径可达，由 Rust 单测
  tests_null_guards::test_null_callee_not_callable 钉住（W1 脚本管线落地后
  才有端到端面）。
- **P550-D5 null.len() 顺带翻转**：ARRAY_LEN null 静默 0 臂（array 通道
  for-in 的长度探针）翻为 not iterable TypeError 时，同臂承接的
  `null.len()` 发射点一并翻转（Python: None 无 len）——顺带翻案，tv 全绿
  佐证无存量依赖。
- **P550-D6 生产者门控 lint 信号面窄**：三信号仅 use.py / null / nil 字面量
  （None/Some 不计入——Option 构造器合法）；#[script] pragma 经 fn 注解
  通道解析（文件任意位置的 #[script] 均标记整文件）。W1 .as 管线落地时
  需复核 pragma 位置语义与 .as 扩展名联动。
- **P550-D7 master ui-iced 档编译断裂修复（非本计划病灶）**：plan051 合入
  （26211362c）在 renderer.rs 留下对 autodown_editor 模块的无条件调用而
  模块双 feature 门控——`cargo t` 别名档（ui-iced only）断裂。本计划以
  同 cfg 补门修复（随 plan-550-dev 折入）；plan051 复审方如认为与原意图
  不符请回馈（详见 550 待澄清#4）。

- **P551-D1 master tf 双红（schema 漂移，非 551 回归）**：`schema_drift_fence`+
  `docs_gen kitchen_sink_page_in_sync` 红——alert-dialog/dialog/dropdown shadcn
  tag 表（93d933a62，Plan 530 W12/W13）未被 schema/aura.at 覆盖；551 全量门
  实证（3412 跑 3410 绿，551 diff 未触及 schema.rs/aura_view_builder.rs）。
  归属 548 会话收口（主检出 schema/aura.at 已有未提交修改疑似修复中）；
  修复路径=`SCHEMA_DRIFT_GENERATE_AT=1` 重生成+复核。
- **P551-D2 auto-down 挪包致 master cargo 全红**：auto-down e7d079e（052 前置，
  2026-09-05）把 autodown-core `packages/core/rust`→`packages/engine/rust`，
  auto-lang master Cargo.toml 的 path 依赖当场失效（任何 cargo 命令清单解析
  即败）。需 052 侧或 auto-lang 随行修（指向 packages/engine/rust 并核 API 面；
  551 worktree 以 pinned e7d079e~1 worktree 解析、未启用该 feature 绕行）。
- **P551-D3 os-config vue 构建 tsc 红（基线既有）**：`auto build` vue 轨
  `Cannot find module '@/lib/api'` 等——gen 树缺 lib/api.ts 生成，auto.exe
  （主检出 master）与 os-config 已提交源码版本偏斜，pristine 同红。阻塞
  Desktop 页 vue 对拍（551 待澄清②，follow-up 先修）。
- **P551-D4 通用编辑器字段级 widget 挂载**：ConfigEditor 内联按 widgets 声明
  替换控件需 per-render 取 widgets 映射——vm 无状态 fn 面下每次 HTTP 不可接受,
  需缓存设计（551 T5 跟进项；desktop_page 自定义视图已演示机制全链）。
- **P551-D5 wallpaper_picker 点选 click-through e2e 缺注入面**：验收通道
  autoui_desktop handler 只达 app root（settings 槽已重绑 os-config root，
  但 DesktopPage/子 widget msg 不可达）——picker 点选链当前以组件级实证
  （数据源/写路径三组件均实机验证）+渲染截图覆盖，click-through 需 vm 子
  组件注入能力。
- **P551-D6 主检出 daemon 二进制落后部署坑**：桌面 daemon 发现序指向相邻仓
  target/release——源码推进后旧二进制仍在位（540 期实机踩坑：缺 desktop 模块
  注册）。551 已加 boot registry 自检日志（模块数+id 一行）；merge 后主检出
  侧 daemon 需重建。
- **P536-D1 schema/aura.at 再生成 canonical 形态振荡**（2026-09-05，PLAN-536
  T11 实录）：`SCHEMA_DRIFT_GENERATE_AT=1` 连续四次重生成，nav-destination/
  swiper 两元素在 kebab 小写形态与 NavDestination/Swiper Pascal 形态间
  **交替翻转**（读 Pascal 文件产 kebab、读 kebab 产 Pascal，反不动点振荡，
  无收敛不动点）。机制在 tests/schema_drift.rs 的 AtGenInput 组装链——
  alias 组根选择（build_alias_groups 字典序最小，Pascal<'n'… 即 ASCII 序
  Pascal 在前）与注册表第 11 源 uncovered_buckets 的"canonical 取小写成员"
  两路径分歧，经"当前 aura.at 拼写回喂 prod_union/carried"通道放大为振荡。
  已证与运行期随机无关（同输入两次运行仍翻转）。现仓状态=kebab 形态 +
  element_coverage 对齐；**手动再生成可能翻面并打红 queue_coverage_drift_
  fence（双向围栏）**，处置=按新形态再对齐登记表（登记动作即过）。根修
  方向：两条 canonical 选择路径统一排序口径（例如别名组根同样"取小写
  成员"），属围栏生成器专项，需全量 diff 评估（牵动全部 alias 组）。
- **P536-D2 跨模块 store handler 帧内 SET_FIELD 重绑定不可达根态**（2026-09-05，
  PLAN-536 T12 实机立案——KD-057④/c13c250"循环副本写读侧不更新"家族的引擎级
  根因候选）：musk 实机 E2E 时序对拍（autoui_state 逐拍采样+VM 日志）实证：
  `SendInput`（dispatcher 派发,root state_obj_id=4000029/4000030）帧内经
  `store.Send/store.StartStream` **跨模块调用**执行的 SET_FIELD——
  `.streaming = true`（Send 与 StartStream 两处）、`.pre_stream_len = ...`——
  **不达根态**（PollStream when 门 104 拍全被 `.streaming==false` 拦截、
  无一 VM_HANDLER_CALL;autoui_state +3s/ +40s 读恒 false）,而**同帧同
  handler** 的 `.messages.push(...)`（列表 vmref 共享突变）与
  `.pre_stream_len = .messages.length`（?——见下）可见性存在矛盾样本
  （pre_stream_len=18 曾在读侧出现）,指向跨模块调用帧的 self 绑定/写
  提交路径分裂（方法突变经共享 vmref 可达根、字段重绑定落在子作用域
  副本或被父向子同步回写覆盖）。timer 派发的 store handler（PollStream
  经 fire_timer→dispatcher,root 帧_init LoadSessionList 同）写入可靠
  ——两种进入路径可见性不一致是核心矛盾。**影响面**：凡"父 handler 体内
  `store.X()` 跨模块调用"对 store 字段的标量重绑定（musk streaming 门控、
  KD-057④ computed 读侧、c13c250 循环副本写、051-T8 列表 prop 族）。
  musk 侧已以 deadman 窗+列表承载绕行（forge_store.at T12 四修,8b1ae23）。
  **偿还方向**：VM codegen/engine 的跨模块调用 self 绑定统一到根态
  （merged 模型下子件 state 即根态,Plan 370 D-GAP-4 语义收口）,需
  state-scope 专项立项（engine.rs/codegen.rs 调用帧与 rc 语义联动,非小修）。
  **【2026-09-05 Phase 2 T13 定性修正】**：仪器化复跑（AUTO_DEBUG_POLLTRACE）
  实证**全部 handler 帧（含跨模块调用产生的执行）均绑根态单对象**
  （VM_EXEC state_obj_id=4000030 全量一致）,`.streaming=true` 等 SET_FIELD
  在四修后代码下**可达根态**（autoui_state +3s 实证 true）——"重绑定不可达
  根态"的旧定性系四修前代码（when 门+头部 StopStream）复合症状的误读;
  "列表突变可见 vs 标量重绑定不可见"的同帧分裂现象**不再复现**,残余疑点
  （RC/同步时序）降级为观察项不阻塞直显链路。真最后一里=musk 画布投影
  chatActivePath 链（KD-057④ 族,061 D25 在修）。POLLTRACE 诊断设施留存
  （env 门控,auto-lang master）。

### P555（2026-09-05，脚本模式 W1 动态分派地基）

- **P550-D6 销号**：`#[script]` pragma 位置语义与 `.as` 扩展名联动已由
  PLAN-555 T02 裁定——文件级信号，任意位置的 `#[script]` 标记整文件；
  `.as` ≡ 隐式 `#[script]`；`#[rust]` 显式压回；优先序 `#[rust]` >
  `#[script]` > 扩展名；八格矩阵单测钉死（mode.rs tests_script_mode）。
- **P550-D4 期望面更新**：CALL null 守卫的端到端探针面仍归 W2——W1 落地
  的是模式信号管线（passthrough），动态分派**语义**（.as 糖激活）在
  W2 lowering 批，届时 null callee 可经动态路径到达 VM 守卫。
- **P555-D1 obj_call 的 Auto 臂仅响亮拒绝**：组合子 obj_call 对 Auto 值
  报 "object is not callable"（对标 550 守卫语义）；Auto 闭包经组合子的
  动态调用面（fn 值/闭包 id 分派）归 W2 糖批接线。
- **P555-D2 s2s W1 帧形约定**：规则表 token 粒度、单遍首中、identity
  逐字节发射。W2 需裁定：链式多规则语义、AST 级发射器（token 面对
  表达式重排类规则（如 E2 with 展开）表达力不足）。
- **P555-D3 ForeignObject.obj_set 借道弹栈封送**：value nv 经
  push/pop_auto_py_arg 转换——裸 f64 单槽 nv 会被当 TAG_NULL→None
  （2-slot padding 是调用方约定；组合子通道罕见面，在案知悉）。
- **P555-D4 master 双既有红（非本计划）**：d8_toggle_dark_mode
  （plan370_015，示例 015 在途工作）与 test_charts_gallery_compiles
  （vue 图表生成）——branch diff 零相关文件 + 输入逐字节同 fork 点
  甄别在案（前者另经 revert 双侧复跑证实）；归并行会话收口。
- **P555-D5 CALL_PY 传输形态命名误导**：组合子发射复用
  is_py_ffi_call→CALL_PY（实为"带实参数字节的通用原生调用"约定），
  命名与 py 耦合是历史包袱；W2 顺手重构为中性命名（如
  CALL_NAT_COUNTED）属低风险清理。
