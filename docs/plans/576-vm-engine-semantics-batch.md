---
plan_id: PLAN-576
status: reviewed
feature_name: VM 引擎语义修复批（nanbox 整值 float 位型保真 + use 子件三缺口）
author: [zhaopuming, ZCode]
created_at: 2026-09-06
updated_at: 2026-09-07

# Filled by /auto-plan:review (2026-09-07):
supersedes_spec_components:
  - "docs/specs/auto-lang/vm/design/bytecode-engine.md: 修改——算术/比较 opcode 混类型 tag 驱动解码（ADD/SUB/MUL/DIV f32×int 混算臂 nanbox_single_to_f32 + LT/GT/LE/GE nv_pair_is_float_numeric 有序比较 float 路由）"
  - "docs/specs/auto-lang/ui/design/aura-pipeline.md: 修改——handler 合成与派发链三点点修（改写集合扩 computed=COMPUTED_FN_NAMES；子件体内引号 emit 改 __emit_<W>_<msg> 桥派发路由=handler_codegen 改写+vm_bridge __emit_msg/__emit_payload 注入+dynamic 收尾；派发参数数失配 warn+跳过两守卫点）"
  - "docs/specs/auto-val/project.md: 修改——nano_value encode/decode 整值 float 位型保真对照契约钉（f64/f32 含 0.0/-0.0 按位可分对照集）"
new_spec_components: []
touched_goals:
  - "GOAL-003: VM 数值语义正确性收口——f32×int 混算/比较按值正确（043「VM handler 哑」家族存活根因清偿）"
  - "GOAL-007: VM 轨与 vue 轨行为对齐——子件 computed 解析/引号 emit 派发路由/失配诊断（043 滚动同步家族引擎缺口关闭）"

current_step: 6
total_steps: 6
---

# [PLAN-576] VM 引擎语义修复批（nanbox 整值 float + use 子件三缺口）

## 变更摘要

auto-lang DEBTS 043 两行 🟡 引擎债合批清偿：①**auto_val nanbox 整值 float
编码丢 float 标签**——整值 float（240.0/0.0）实参绑定读到垃圾、write_state
写入读回 0，043 期以 renderer 直写 + 数值 +1e-3 分数化绕道（绕道散布是
043 整轮"VM handler 哑"假理论的单一根因）；②**use 子件三缺口**——computed
在子件 handler 体内不解析（编译后 Nil 传播）、子件体内带计算实参 emit 无
派发路由（内联直调绕过 on_with_input_for）、handler 参数数与合成派发实参
失配时静默错位。全为 TDD 先行（对照测试红→修→绿），绕道退役另行评估不
混入本批。

## 目标

- **G1 nanbox 位型保真**：任意 f64（含整值 240.0/0.0/-0.0）encode→decode
  恒等且 float 标签不丢；VM 实参绑定/write_state 路径实测恒等。
- **G2 子件 computed 解析**：use 子件 handler 体内引用 computed 字段编译
  后取到计算值（非 Nil）。
- **G3 子件 emit 派发路由**：子件体内带计算实参的 emit 走
  on_with_input_for 派发（实参快照进消息），行为与顶层件一致。
- **G4 失配诊断**：handler 形参数与派发实参数失配时输出响亮诊断（不再静
  默错位）。
- **G5 零回归**：vm 全量 + tf 绿（唯一红=charts 既有）。

## 架构方案

不引入新机制：①nanbox 位型保真在 `auto-val` NanoValue 的 encode/decode
对内修复（保持既有表示策略与堆布局，仅标签语义保真——整值 f64 编码时强
制携带 float 标签位）；②子件三缺口在 handler 编译/派发链三点点修——
handler_codegen 状态引用改写集合扩至 computed 字段、child_emit 引号 emit
改走派发路由、dynamic.rs 派发点补参数数诊断。绕道退役（renderer frac、
demo 双轨）不混入本批，G1-G4 落地后另行评估退役波次。

## 技术栈

- 实现面：`crates/auto-val/src/nano_value.rs`（encode/decode）+
  `crates/auto-lang/src/ui/handler_codegen.rs`（状态引用改写集合）+
  `crates/auto-lang/src/ui/child_emit.rs`（子件 emit 路由）+
  `crates/auto-lang/src/ui/dynamic.rs`（派发参数数诊断）。
- 验证面：auto-val/auto-lang 定向单测 + `cargo tf`。

## 需求分析与背景调查

- **来源**：auto-lang `DEBTS.md` 043 两行 🟡（2026-09-03 登记）；
  auto-down `DEBTS.md` 043 同款行互链。
- **债务①锚点**：`crates/auto-val/src/nano_value.rs`（nanbox 编码）；
  043 期排障实证整值 float 实参绑定读垃圾/写读回 0，分数值正常——编码
  位型把整值 f64 误装 int 标签。
- **债务②锚点**：`ui/handler_codegen.rs` strip_callback_calls（状态引用
  改写只认本件 state_fields）；`ui/child_emit.rs`（子件 emit 内联直
  调）；`ui/dynamic.rs` C2① 回送段（0 参合成派发）。
- **绕道清单（本批不动，退役另行评估）**：renderer PLAN-043 T6 直写快道
  + frac +1e-3；demo custom_scrollbar.at 的 is_vm prop 双轨分派与 thumb
  几何内联（auto-down 侧）。

## 详细设计

### D1 nanbox 位型保真（G1）

对照测试先行：NanoValue encode→decode 对
{240.0, 0.0, -0.0, 1.0, -1.5, f64::MAX} 全集恒等（type tag == Float 且值
位相等）——红；修复 encode 臂对 f64 输入恒装 float 标签（整值不再误判
int），decode 对 float 标签按 f64 读出——绿。不改堆布局策略。

### D2 子件 computed 解析（G2）

handler_codegen 状态引用改写：改写集合从"本件 state_fields"扩为
"state_fields ∪ computed 字段（use 子件装配面导出的）"；directed 单测：
子件 handler 体引用 `.computed_x` 编译产物取值非 Nil。

### D3 子件 emit 派发路由（G3）

child_emit 引号 emit（带计算实参）改生成 on_with_input_for 派发调用（实
参在编译期快照进消息载荷），不再内联直调被调 handler 体；directed 单测：
emit 消息携带实参值且被调方经派收器触达。

### D4 失配诊断（G4）

dynamic.rs 派发点：实参数 ≠ 形参数时 log::warn 响亮输出（件名/handler
名/两侧参数数），调用照旧跳过（不静默错位执行）；directed 单测断言警告
与不执行。

## 测试设计

| 门 | 内容 | 命令 |
|---|---|---|
| 单测 | nanbox 对照集恒等（红先行） | `cargo test -p auto-val` |
| 单测 | 子件 computed/emit 路由/失配诊断 三 directed | `cargo test -p auto-lang --lib`（handler/child_emit/dynamic 模块过滤） |
| 回归 | vm 全量 + tf | `cargo tf --no-fail-fast`（唯一红=charts 既有） |

## 验收标准

1. nanbox 对照集全恒等（G1 单测绿，含 0.0/-0.0/整值边界）。
2. 子件三 directed 单测绿（G2/G3/G4）。
3. vm 全量 + tf 唯一红=charts 既有。
4. DEBTS 043 两行销号（互链本计划）；绕道退役评估结论入复审记录。

## 执行步骤

- [✅ 已完成] **T1** nanbox 对照测试（红）：`crates/auto-val/src/nano_value.rs`
      测试模块增 encode→decode 恒等集。验证：`cargo test -p auto-val`
      新测红。
  > 证据（2026-09-07）：auto-val 新增 `test_f64_integer_valued_roundtrip_tags`
  > （240.0/0.0/-0.0/1.0/-1.5/f64::MAX/f64::MIN_POSITIVE 全集 tag==Float +
  > 位型恒等 + 0.0/-0.0 按位可分）与 f32 同口径测试——**绿**（147+27 全绿）。
  > **执行偏差①（理论修正）**：encode/decode 纯往返在当前 master 已恒等
  > （043 后续 Plan 437 tag 驱动 decode/Plan 474 tag-first 转换已治愈编码
  > 路径），本半无法红。经动态件 harness 实测定位存活根因：**TAG_F32 操作
  > 数与 TAG_I32 混算/比较时 engine 算术 else 臂与 LT/GT/LE/GE fallback 臂
  > 按位型 decode_i32**——`240.0 + 1 → Int(1131413505)`（=0x43700000+1）、
  > `50.0 > 100` 恒真（043「比较守卫静默假」同源）。定向红测
  > `plan576_integer_float_mixed_arith_and_cmp`（dynamic.rs，含绑定/写读回
  > 钉契约段）红于混合算术断言，绑定恒等段绿。修复落点随之为 engine.rs
  > 算术/比较臂（D1"decode 对 float 标签按 float 读出"语义，非 nano_value.rs
  > encode——待澄清#1 默认"仅标签语义修复"边界内，不改表示策略/堆布局）。
- [✅ 已完成] **T2** nanbox 位型保真修复：encode 臂 f64 恒装 float 标签 + decode
      对应。验证：T1 转绿 + `cargo test -p auto-val` 全绿。
  > 证据（2026-09-07）：修复落在 engine.rs（见 T1 执行偏差①——encode 侧无
  > 标签丢失，存活根因是消费臂）：①新增 `nanbox_single_to_f32` 助手
  > （tag 驱动，f32 按位/int 按值）；②ADD/SUB/MUL/DIV 增 f32×int 混算臂
  > （DIV 含除零守卫），不再落 decode_i32 位型误读；③LT/GT/LE/GE 增
  > `nv_pair_is_float_numeric` 路由臂（双侧数值+一侧 float → `nv_as_f64`
  > 按值比较，纯 int/对象/字符串臂不变）。验证：定向测
  > `plan576_integer_float_mixed_arith_and_cmp` 红→绿（混合算术/比较/
  > 绑定/写读回全过）；`cargo test -p auto-val` 147+27 全绿；`cargo tv`
  > --no-fail-fast 3608/3609 绿（唯一红=charts_gallery 预存，G5 口径）。
- [✅ 已完成] **T3** 子件 computed 解析：handler_codegen 改写集合扩 computed
      （D2）。验证：directed 单测绿。
  > 证据（2026-09-07）：红测 `plan576_child_computed_resolves_in_handler`
  > （dynamic.rs：子件 handler 体 `if .half_h > 100.0 { .out = .quarter_h }`，
  > 含守卫形态与递归 computed quarter_h→half_h）红时 RuntimeError
  > "Field 'half_h' not found"（043②① Nil 传播/守卫静默假实锤）→绿后
  > out=60.0（half=120 守卫真，quarter=60）。实现（handler_codegen.rs）：
  > ①`COMPUTED_FN_NAMES` thread-local（STORE_FIELDS 同模式，per-widget 设置，
  > 双合成路径 AuraWidget/decl 均接线，尾部 clear）；②Phase 1 增两拦截臂
  > ——computed 裸 Ident 与 `.name`/`self.name` Dot 形态改写为
  > `__computed_<W>_<p>(__state)` 调用（state 字段优先、局部遮蔽规则同），
  > Call-name 递归保护防 Call-in-name 非法 AST；③`synthesize_computed_fns`
  > 扩表达式形态（单条 return，块体形态不变）——两种形态都有函数体供调用。
  > 回归：dynamic:: 34 绿 + child_emit 4 绿。
- [✅ 已完成] **T4** 子件 emit 派发路由（D3）：child_emit 改 on_with_input_for
      生成。验证：directed 单测绿。
  > 证据（2026-09-07）：红测 `plan576_child_quoted_emit_routes_with_payload`
  > （dynamic.rs：子件 `.Move` 体内 `let _ = ."moved"(v)`，v=track_h*0.5 计算
  > 局部变量；父 `Bar576e(onmoved: .Moved)`）红时组件链接失败——
  > `Undefined symbol: handler_Bar576e_moved`（043②②「内联直调、不经派发
  > 器」实锤）→绿后父 got=120.0（emit 载荷携带计算实参经派发器路由）。
  > 实现（toast/__toast、router.push/__current_route 同型三件套）：①
  > handler_codegen 改写拦截——`.Name(args)` 且 Name 是本件 msg 变体而本件
  > 无同名 handler（新 CURRENT_HANDLER_NAMES thread-local 消歧，有同名
  > handler 仍走 Plan 398 sibling 内联臂）→ 改写为 `__emit_<W>_<Name>
  > (__state, args)` 桥函数调用；桥函数体内写 `__emit_msg`/`__emit_payload`
  > 状态对（合成期 PENDING_EMIT_SYNTH 收集、双路径去重编译，msg 名含引号
  > 专字符按非字母数字折叠 _）；②vm_bridge 状态实例注入 `__emit_msg`("")
  > +`__emit_payload`(Nil)（__toast 注入同点）；③dynamic.rs on_with_input_for
  > Ok 臂收尾——读出清账后走 C2① 同一 lookup_route/dispatch_parent_route 链。
  > v1 限度（注记）：单 pending 槽（同 handler 多次 emit 末次生效）；多实参
  > 取首个位置实参。回归：dynamic 35 + handler_codegen 14 + vm_bridge 37 +
  > plan051 24 + plan412 24 全绿（sibling 两单测随消歧契约更新补设 handler 集）。
- [✅ 已完成] **T5** 失配诊断（D4）：dynamic.rs 派发点 warn + 不执行。验证：
      directed 单测绿。
  > 证据（2026-09-07）：红测 `plan576_dispatch_arity_mismatch_diagnosed_and_skipped`
  > （红=编译缺 take_last_arity_mismatch）→绿：`.Two(a,b)` 派发 1 实参 →
  > [VM-ARITY] 告警（件名/handler/两侧参数数）+ 跳过不执行（touched 不动）；
  > 对照 `.One(a)` 1 实参照常执行、无告警。实现（dynamic.rs 两守卫点+镜像）：
  > ①on_with_input_for 主派发点——handler_param_count 已知且 ≠实参数 →
  > log::warn + return（未知 arity 保持 legacy）；②dispatch_parent_route
  > 对齐升级——Some(0) 不塞载荷（T9 语义）/Some(1) 塞载荷/**Some(≥2) 此前
  > 也只塞 1 载荷（帧错位）**→ 现告警+跳过；③LAST_ARITY_MISMATCH thread-local
  > 镜像（directed 测试可断言，日志在测试运行器不可捕获）。回归：dynamic 36
  > + plan051 24 + plan446 28 全绿（B3/B12 既有口径未破）。
- [✅ 已完成] **T6** 回归与折回：`cargo tf --no-fail-fast` + 折回 master +
      DEBTS 043 两行销号 + 绕道退役评估结论入复审。验证：计数落复审记录。
  > 证据（2026-09-07）：①tf（tf 别名全口径 + ui-iced + no-fail-fast，
  > nextest-full 配置）4669 测 4662 绿 / 7 红——**零红归因本计划**：6×
  > ui::layout（预存环境红，master 主检出自跑复现 14 红——本机显示几何/
  > 任务栏依赖，数量随会话浮动）+ 1×charts_gallery（G5 既有口径）；②tv
  > 3608/3609（T2 时点，唯一红=charts）；③折回：master merge 745b9b06b
  > （plan-576-dev 五提交：400a81579/61e7d4bb5/59564b5f9/21bd96c66/
  > 501b35c25）；④DEBTS 三行销号（043 nanbox 行、043 子件三缺口行、041
  > argstr 行——待澄清#3 复核失效，现行 argValueAt 为段迭代+引号感知扫描
  > 器）；⑤绕道退役评估结论见复审记录执行注记。worktree（lang-576 组，
  > 含 auto-down detached 兄弟——cargo 路径解析用）保留待 /auto-plan:merge
  > 清理。

## 复审记录

- **复审人**：ZCode（/auto-plan:review，2026-09-07）
- **方法**：worktree（.wt/lang-576/auto-lang，HEAD=501b35c25，工作区 clean）实跑全部门禁；diff 全量核对（6 文件 +749/-15）；DEBTS/KNOWN-DEBT 文本核对。
- **逐条验收判定**：
  1. **nanbox 对照集全恒等（G1）——PASS**：`cargo test -p auto-val` 147+27 全绿（含
     `test_f64_integer_valued_roundtrip_tags`/`test_f32_…`：240.0/0.0/-0.0/1.0/
     -1.5/f64::MAX/f64::MIN_POSITIVE tag==Float+位型恒等+0.0/-0.0 按位可分）。
  2. **子件三 directed 单测绿（G2/G3/G4）——PASS**：`plan576_` 过滤 4/4 绿
     （mixed_arith_and_cmp / child_computed_resolves / child_quoted_emit_routes /
     dispatch_arity_mismatch）。
  3. **vm 全量 + tf 唯一红=charts——PASS（带环境红注记）**：tv 3608/3609
     （唯一红=charts_gallery，口径精确成立）；tf（tf 别名全口径+ui-iced+
     no-fail-fast）4669 测 4658 绿 / 11 红，**零红归因本计划**：9×ui::layout
     （本机显示几何环境红，master 主检出自跑复现 14 红，数量随会话 6↔9
     浮动）+ 1×osconfig_daemon::resolve_order_sibling_target_then_path
     （复审新发现：sibling_fixture 固定共享临时目录+remove_dir_all 开场的
     并行竞态——单测隔离复跑绿，该文件 Plan 505 期产物本计划零触及，
     已登记 KNOWN-DEBT）+ 1×charts_gallery（G5 既有口径）。字面口径
     「唯一红=charts」在本机不成立系环境红族（master 同在），意图口径
     （本计划零新增红）成立。
  4. **DEBTS 043 两行销号 + 退役评估入复审——PASS**：两行均已 ✅销号并
     互链 plan 576 T1/T2、T3/T4/T5（grep 核对）；041 argstr 行附带销号
     （待澄清#3 复核失效）；绕道退役评估结论在执行注记 2。
- **懒收敛排查（遗漏/延后/workaround）**：
  - **遗漏**：无——D1-D4 全落地（engine.rs 混算/比较臂 9+5 处、
    COMPUTED_FN_NAMES×4、__emit 注入×2、take_last_arity_mismatch×3、
    PENDING_EMIT_SYNTH×3 均在 diff 中核对）；6 新测全数在册；diff 无新增
    TODO/FIXME/HACK/临时标记。
  - **延后**：绕道退役不当轮=计划待澄清#2 预设默认（用户签认非静默），
    已入 KNOWN-DEBT 跟进行；C2② on_* 局部实参快照限制为 PLAN-051 v1
    既有范围（G1-G4 字面之外），复审登记为相邻债不阻塞。
  - **workaround**：无新增隐藏性绕道；__emit 桥 v1 限度（单槽/首参）为
    显式登记的设计限度（代码注记+执行注记 5+KNOWN-DEBT）。
- **偏差登记（code over plan）**：①修复落点 engine.rs 消费臂而非
  nano_value.rs encode（执行偏差①，实测证伪计划理论，D1 语义等价实现）；
  ②技术栈清单所列 child_emit.rs 未改——emit 路由经 handler_codegen 改写+
  dynamic 收尾实现，child_emit ROUTES 表被原样消费（T4 证据已记录）。
- **债候选登记**：KNOWN-DEBT-AND-RISKS.md 🟡 节新增 4 行（576 v1 限度 /
  576 相邻债 C2② / 576 绕道退役跟进 / 576-复审 osconfig_daemon+layout
  测试隔离防误归因）。
- **裁定**：四条验收全 PASS、无阻塞债 → **status: reviewed**，可入
  /auto-plan:merge。

### 执行注记（/auto-plan:work 收口，供复审与 merge 参考）

1. **执行偏差①（T1/T2 落点修正）**：计划理论"encode 丢 float 标签"经实测
   证伪——encode/decode 纯往返在 master 已恒等（Plan 437/474 治愈），存活
   根因是 engine 算术/比较臂的 TAG_F32×TAG_I32 混算位型误读（D1"decode
   对 float 标签按 float 读出"语义落在 engine.rs 消费臂而非 nano_value.rs
   encode；待澄清#1 默认"仅标签语义修复"边界内，表示策略/堆布局未动）。
2. **绕道退役评估结论（验收 4）**：G1-G4 已消除绕道的引擎侧前提——
   renderer PLAN-043 T6 直写快道 + frac +1e-3（renderer.rs/aura_view_builder.rs
   T9）与 demo custom_scrollbar is_vm 双轨在语义上均可退役；但物理退役触及
   auto-down demo 侧实机验证（滚动几何/双端一致性），按待澄清#2 默认**不当轮
   退役**，随滚动同步契约计划（或下个 VM 计划）另行立项。
3. **G5 口径注记**：tf 唯一非 charts 红 = ui::layout 6 测（预存环境红，
   master 复现 14 红，本机任务栏/显示几何依赖）；非本计划引入。
4. **组内 auto-down 兄弟 worktree**：detached（a6d5ecf），仅为
   autodown-core path 依赖的 cargo 解析（f5c86eeba 修复后的组内兄弟约定），
   本计划零修改 auto-down；/auto-plan:merge 清理组目录时一并移除（wt-guard 先行）。
5. **v1 限度登记**：引号 emit 桥单 pending 槽（同 handler 多次 emit 末次
   生效）；多实参取首个位置实参；computed 与 state 字段同名时 state 优先。

## 待澄清事项

1. **nanbox 修复策略边界**：若位型保真需改动 NanoValue 表示策略（而非仅
   标签语义），性能/FFI 影响需用户裁定（默认仅标签语义修复）。
2. **绕道退役波次**：G1-G4 落地后 renderer frac 快道与 demo 双轨是否当轮
   退役（默认不当轮——退役触及 auto-down demo 侧，另行立项/随下个 VM 计
   划）。
3. **041 argstr 多参行关联性**：DEBTS 041 行"parser argstr 多参仅首参可
   靠"与 054 实测 $callout type+title 双参 OK 疑似失效——执行期顺手复核，
   失效则该行一并销号。
