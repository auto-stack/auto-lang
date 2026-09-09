---
plan_id: PLAN-598
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: a2py-semantic-fix-batch
author: [ZCode]
created_at: 2026-09-09
updated_at: 2026-09-09
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals:
  - "GOAL-006"                # Consumer parity：py 套件语料编写体验与三轨语义面（暂定，review 定稿）

affects: [auto-lang/trans, auto-lang/vm]
current_step: 0
total_steps: 8
---

# [PLAN-598] a2py-semantic-fix-batch：py_call 糖族括号纪律 + 语句体闭包双面清偿

> python-parity-roadmap §7.4 队列 ⑦（2026-09-09 复核收窄版）。本计划起草期
> 实勘已把两个债项的现状钉到可执行粒度（探针证据见 §4），并发现 DIV-PY-CLOSURE-1
> 实为**双轨缺陷**（不只 a2py 发射面，VM 侧含 `let` 块体也静默返 None）——范围
> 已按"小批快胜"原则划定：a2py 两处发射修复 + VM 一处定点修复，句柄槽裸 id
> 面显式留在 W3（P539-D4 地界）。

## 0. 变更摘要

清偿两个存量语义缺陷（531 式债批，全部为发射/求值修复，无新能力）：

1. **P539-D3（a2py 糖族复合接收者无括号）**：`py_call(t + tensor([1,1,1]), "sum")`
   发射 `t + tensor([1, 1, 1]).sum()`——`.sum()` 绑到末操作数（探针 d3.py 实证）。
   修复：糖族接收者发射统一加括号（审计 py_call / py_call_may / py_getattr /
   py_getitem / py_call0 全族）。
2. **DIV-PY-CLOSURE-1（语句体闭包）双面清偿**：
   - **a2py 面**：`(x) => { let y = x + 1; y * 2 }` 发射 `lambda x: {\n y = x + 1\n
     y * 2\n }`——Python 里是 set/dict 字面量且语句在 lambda 内非法（探针 cl.py
     实证）；单表达式块 `{ x * 2 }` 更隐蔽，发射 `lambda x: {x * 2}` = **合法
     set 字面量，静默错类型**。修复：零语句块（尾表达式）→ `lambda x: (expr)`；
     含语句块 → 显式诊断（替代静默垃圾）。
   - **VM 面（实勘新发现）**：同一语料 VM 腿返 `None`（探针 cl.as 实证）——
     闭包块体含 `let` 声明时尾值丢失；单表达式块 `{ x * 2 }` 返 6 正常
     （cl2.as 对照）。定点修复块体尾值传播。
3. **de-workaround**：py 套件中因 P539-D3 使用的中间变量规避点改回直写。

非目标：VM 闭包槽/捕获中 py 句柄退化裸 id（W3 回调模型地界，P539-D4）；use.py
别名/子模块语法（队列 ②）；GPU/CUDA（队列 ⑧）。

## 1. 目标

- py_call 糖族复合接收者三轨语义对齐（a2py 发射合法且优先级正确）。
- 语句体闭包不再静默错值：VM 返尾值、a2py 合法发射或显式诊断。
- py 套件规避点回收；既有 goldens/相位零回归。
- 债务账面收口：P539-D3 清偿销账；DIV-PY-CLOSURE-1 拆面记账（a2py 面 + VM
  let-面清偿，句柄槽面 open 归 W3）。

## 2. 架构方案

```
trans/python.rs（a2py 发射，两处）
  ├─ py_call 糖族臂（L1097-1143 py_call_may / L1271 py_call0 / "py_call" 主臂）
  │    接收者发射 → 统一括号包裹 (RECV).method(...)   ← Python 恒合法，零判定风险
  │    （改动态价：现有 a2p goldens 中糖族发射文本更新——golden 校准豁免先例）
  └─ Expr::Closure 臂（L260-270）× Expr::Block（L193-202 裸 {...} 发射不动——
     语句块本义）
       块体分类：0 语句+尾 expr（或单 expr-stmt）→ lambda x: (expr)
                含 let/多语句        → a2py 显式诊断（新错误码，指向 py_with/map 糖）

vm/engine.rs（一处定点）
  └─ 闭包体块求值尾值传播：块含 let 声明时返回值丢失（返 None）
       → 有界调查（闭包注册 L8013-8074 / 执行 L1832 路径）→ 定点修
       （时间盒 1h；若根因超出定点范围 → 降级为 VMError 显式化 + 扩记 CLOSURE-1 open 面）
```

门禁归属：改 `trans/python.rs` → `cargo tt`；改 `vm/engine.rs` → `cargo tv`；
两者都触 → fold 前 `cargo tf`（review 阶段）；不触 aavm 面 → 不跑 `taa`。

## 3. 技术栈

Rust（auto-lang trans/python.rs + vm/engine.rs）、a2p golden 语料
（test/a2p/05_expressions/*）、.as 探针语料、parity p5-p9 相位回归（本机
Python/torch 环境已验可用）。无新依赖。

## 4. 需求分析与背景调查

**授权**：用户确认"⑦′ 先行，W3 紧随"（2026-09-09，基于路线图 §7.4 复核后的
队列推荐）。预算口径：一两天量级小批；无自动续跑上限特别授权。

**债务条目原文**：
- KNOWN-DEBT-AND-RISKS.md L1187：P539-D3——`py_call(t == t, "sum")` 发射
  `t == t.sum()`（优先级错）；套件用中间变量规避。
- parity/docs/known-divergences.md L586 DIV-PY-CLOSURE-1：py 句柄在闭包局部/
  捕获中退化为裸 id（VM 面；open，W1 探针 c7）。路线图 ⑦ 引用的是其 a2py
  发射面（语句体闭包降级 set 字面量）。

**起草期实勘（2026-09-09，全部探针复现）**：
- d3.as（`py_call(t + tensor([1,1,1]), "sum")`）→ a2py 发射
  `print(t + tensor([1, 1, 1]).sum())`（优先级错）；VM 腿报
  `PyException 'str' object has no attribute 'sum'`——句柄算术退化是另一条
  已知纪律（py_torch README 惯用法），复合接收者含句柄算术时 VM 本就不支持，
  **本计划只修 a2py 发射优先级面，不改句柄算术语义**。
- cl.as（`(x) => { let y = x + 1; y * 2 }`）→ a2py 发射 `lambda x: {...}`
  垃圾；VM 返 `None`。
- cl2.as 对照：`(x) => x * 2` 与 `(x) => { x * 2 }` VM 均返 6 → VM 缺口
  精确锁定为"块体含 `let` 时尾值丢失"；a2py 缺口为全部 `{...}` 块体。
- 糖族发射点：trans/python.rs "py_call" 主臂（接收者裸发射）、py_call_may
  （L1101，`_auto_may(lambda: RECV.m(...), None)` 内同病）、py_getattr_may
  （L1127）、py_callable（L1176）、py_call0（L1271）——T-02 全族审计。
- 代码位：trans/python.rs L193-202（Expr::Block 裸 `{...}`）、L260-270
  （Expr::Closure 只支持表达式体）；vm/engine.rs L211-241（Closure 注册表）、
  L8013-8074（注册）、L1832（执行路径）。

**关联计划**：539（W1 dunder 路由/W3 回调桥 T21）、560（T07 中缀糖——其
"解析层直产桥调用"先例与本批同为发射层修复）、567（may 通道族）、569（py
返回值分派——同族 py 语义面收口先例）。

## 5. 详细设计

### D1 py_call 糖族接收者括号（P539-D3）

新私有 helper `emit_receiver(expr, sink)`：恒定括号包裹
（`(RECV).method(...)`）——Python 语义下恒等价且零判定风险；Ident 接收者的
括号属无害冗余（golden 文本更新一次到位）。应用于：`"py_call"` 主臂、
py_call_may（lambda 体内 `(RECV).m(...)`）、py_getattr_may、py_call0、
py_callable。审计清单以
`grep -n 'py_call\|py_getattr\|py_getitem\|py_callable' trans/python.rs`
为准，逐臂核对。

### D2 语句体闭包 a2py 面（CLOSURE-1 a2py 面）

`Expr::Closure` 臂对 `Expr::Block` 体分类：
- 块内 0 条语句且尾为表达式（Auto 块值语义）→ 发射 `lambda p: (EXPR)`
  （单表达式块 `{ x * 2 }` 从静默 set 字面量变为正确 `lambda x: (x * 2)`）。
- 含 `let`/多语句 → 编译期诊断（复用语法错误出口，文案指引：表达式体改写 /
  py_with 语句糖 / map 回调糖）。**不做** Python 侧 def 提升（表达式级无 def，
  提升需作用域重写，复杂度与 ⑦′ 小批不称）。

### D3 闭包块体尾值传播（CLOSURE-1 VM 面）

有界调查：cl.as（let 块）VM 返 None、cl2.as（裸 expr 块）返 6——定位闭包
执行路径上块求值的返回值通道（engine.rs L1832 附近执行 + L8013 注册）。
预期根因形态：块含声明时走语句求值路径、尾表达式值未回传闭包调用栈。
定点修后 cl.as 应返 6；时间盒 1h，超限降级：该形态 VMError 显式化
（"statement-body closure value unsupported"）+ CLOSURE-1 扩记。

### D4 语料与 de-workaround

- 新 a2p golden：05_expressions 下 lambda 块体用例（单表达式块发射更新 +
  let 块诊断 golden）。
- 新 .as 探针语料（不入 parity 矩阵，进 test/a2p 或独立 fixtures）：
  d3 修后形态（`(t + tensor([1,1,1])).sum()`）、cl 两形态。
- de-workaround：`grep -rn "py_call" parity/libs/python/*/tests/auto/*.as`
  定位中间变量规避点，逐点改回复合接收者直写并以相位回归兜底。

### 规范增量

| delta_id | 操作 | 目标 | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/frontend/overview.md | a2py 糖族：接收者裸发射 → **恒括号纪律**（py_call/py_call_may/py_getattr_may/py_call0/py_callable）；语句体闭包：静默 `{...}` → 单表达式块 `(expr)` / 含语句块显式诊断 | 优先级错与 set 字面量静默错值均为错误语义面，收口为恒正确发射或显式报错 | AC-01, AC-02, AC-03 |
| SD-02 | modify | docs/specs/auto-lang/vm/overview.md | 闭包块体求值：含 let 块返 None → **尾表达式值传播**（与裸表达式块一致） | 静默 None 属"静默错值"族，与 592 收口同纪律 | AC-03 |

（spec 目标路径为暂定——560 的 spec delta 曾触及 frontend/overview.md 与
vm/overview.md 现状节，review 时按实际文档结构定稿。）

## 6. 测试设计

| 用例 | 层 | 断言 |
|---|---|---|
| py_call 复合接收者 | a2p golden | `(t + tensor([1, 1, 1])).sum()` 括号发射 |
| py_call_may/getattr_may/call0 同族 | a2p golden | 各臂括号发射更新 |
| 单表达式块闭包 | a2p golden | `lambda x: (x * 2)` |
| 含 let 块闭包 | a2p golden | 显式诊断（错误文案 golden） |
| 含 let 块闭包 | VM（cargo tv） | cl.as 形态返 6（非 None） |
| VM 回归 | cargo tv | 闭包/捕获既有语料零回归 |
| a2py 回归 | cargo tt | 既有 a2p/a2r goldens 除本批更新点外零漂移 |
| py 相位回归 | parity p5-p9 | 20 套件零回归（de-workaround 点双向核对） |

## 7. 验收标准

- **AC-01**：`py_call` 糖族（含 may/getattr/call0 变体）复合接收者发射恒括号；
  d3 形态 a2py 产物语义正确（`(t + tensor([1, 1, 1])).sum()` == 9）。
  验证：`cargo tt` + d3 探针转译产物人工核对。
- **AC-02**：单表达式块闭包 a2py 发射 `lambda x: (expr)`；含 let 块闭包 a2py
  显式诊断（不再静默 set 字面量）。验证：`cargo tt` + cl 探针。
- **AC-03**：含 let 块闭包 VM 返尾表达式值（cl.as 形态返 6）；时间盒降级时
  为显式 VMError 并在 CLOSURE-1 扩记。验证：`cargo tv` + cl.as 实跑。
- **AC-04**：既有回归零漂移——`cargo tt`/`cargo tv` 除本批 golden 更新点外
  全绿；parity p5-p9 相位零回归。验证：门禁命令 + 相位跑批。
- **AC-05**：账面收口——P539-D3 销账；DIV-PY-CLOSURE-1 拆面更新（a2py 面 +
  VM let 面清偿，句柄槽面 open 归 W3）；路线图 §7.4 ⑦ 核销注记。
  验证：两文件 diff 人工核对。

## 8. 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成]
证据。执行于 worktree `D:/autostack/.wt/lang-598/auto-lang`，开工前 master
提交本骨架 + .next-id；worktree 禁 junction/symlink。）

- [ ] **T-01** 基线：master 提交骨架；建 worktree；记录修复前基线
  （`cargo tt` 现状、a2p lambda goldens 现状、`p7 py_list` 相位快照）。
- [ ] **T-02** P539-D3 糖族括号（D1）：trans/python.rs 五臂审计 + 统一
  `(RECV)` 发射 + 受影响 goldens 更新。验证：`cargo tt` 全绿；d3 探针
  产物核对。
- [ ] **T-03** VM 闭包尾值（D3）：有界调查（定位块求值返回通道，写决策
  注记）→ 定点修；时间盒 1h，超限走降级路径并回填 T-07 账面。
  验证：`cargo tv` 全绿；cl.as 形态返 6。
- [ ] **T-04** a2py 语句体闭包（D2）：Expr::Closure 块体分类 + 单表达式块
  `(expr)` 发射 + 含语句块诊断。
  验证：`cargo tt` 全绿；cl 探针两形态产物核对。
- [ ] **T-05** 语料：a2p lambda 块体 goldens（新/更新）+ cl/d3 探针固化。
  验证：`cargo tt` 含新 goldens 全绿。
- [ ] **T-06** de-workaround：grep 定位 py 套件中间变量规避点，改回直写。
  验证：`parity` 下 `phase p7` + 触及套件单跑零回归。
- [ ] **T-07** 账面收口（AC-05）：KNOWN-DEBT P539-D3 销账；
  known-divergences DIV-PY-CLOSURE-1 拆面更新；路线图 §7.4 ⑦ 核销。
  验证：三文件 diff 人工核对。
- [ ] **T-08** 收口健康检查：`cargo tt` + `cargo tv` 全绿；`p5-p9` 相位
  回归；探针残留扫描；fold 前 `cargo tf` 留给 review 阶段。

## 9. 复审记录

（起草交接：stage=new | plan_id=PLAN-598 | rev=1 | outcome=pass |
next=work。范围调整留痕：DIV-PY-CLOSURE-1 实勘扩为双轨缺陷，VM let-面纳入
本批（T-03 带时间盒与降级路径），句柄槽裸 id 面显式留 W3。）

## 10. 待澄清事项

1. **T-03 根因深度**：VM 块体尾值丢失的修复点未预勘（有界调查+1h 时间盒 +
   降级路径已内置）；若根因牵涉块求值通用语义（非闭包特有），升级为独立
   发现并回报，不在本批强修。
2. **恒括号 vs 条件括号**：恒括号使既有糖族 goldens 文本全面更新（一次性
   噪音）；条件括号保 golden 稳定但引入判定表。默认恒括号（零误判纪律），
   review 时若 golden 噪音超预期可复议。
3. **含 let 块闭包的 a2py 处理**：诊断 vs def 提升。默认诊断（表达式级 def
   不存在，提升需作用域重写）；若执行期发现 py 套件确有 let 块闭包刚需，
   升级回报再议。
4. **de-workaround 点位清单**：以执行期 grep 实测为准（起草期未预枚举）；
   若某点改回直写后触发句柄算术退化（DIV-PY 另一条纪律），该点保留规避并
   注记——不算本批失败。
