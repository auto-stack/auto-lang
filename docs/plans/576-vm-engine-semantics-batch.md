---
plan_id: PLAN-576
status: drafting
feature_name: VM 引擎语义修复批（nanbox 整值 float 位型保真 + use 子件三缺口）
author: [zhaopuming, ZCode]
created_at: 2026-09-06
updated_at: 2026-09-06

# Leave these EMPTY here — /auto-plan:review fills them:
supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 0
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

- [ ] **T1** nanbox 对照测试（红）：`crates/auto-val/src/nano_value.rs`
      测试模块增 encode→decode 恒等集。验证：`cargo test -p auto-val`
      新测红。
- [ ] **T2** nanbox 位型保真修复：encode 臂 f64 恒装 float 标签 + decode
      对应。验证：T1 转绿 + `cargo test -p auto-val` 全绿。
- [ ] **T3** 子件 computed 解析：handler_codegen 改写集合扩 computed
      （D2）。验证：directed 单测绿。
- [ ] **T4** 子件 emit 派发路由（D3）：child_emit 改 on_with_input_for
      生成。验证：directed 单测绿。
- [ ] **T5** 失配诊断（D4）：dynamic.rs 派发点 warn + 不执行。验证：
      directed 单测绿。
- [ ] **T6** 回归与折回：`cargo tf --no-fail-fast` + 折回 master +
      DEBTS 043 两行销号 + 绕道退役评估结论入复审。验证：计数落复审记录。

## 复审记录

（待 /auto-plan:review 填写）

## 待澄清事项

1. **nanbox 修复策略边界**：若位型保真需改动 NanoValue 表示策略（而非仅
   标签语义），性能/FFI 影响需用户裁定（默认仅标签语义修复）。
2. **绕道退役波次**：G1-G4 落地后 renderer frac 快道与 demo 双轨是否当轮
   退役（默认不当轮——退役触及 auto-down demo 侧，另行立项/随下个 VM 计
   划）。
3. **041 argstr 多参行关联性**：DEBTS 041 行"parser argstr 多参仅首参可
   靠"与 054 实测 $callout type+title 双参 OK 疑似失效——执行期顺手复核，
   失效则该行一并销号。
