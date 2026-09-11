---
plan_id: PLAN-598
status: archived              # drafting → executing → execution_done → reviewed → archived
feature_name: a2py-semantic-fix-batch
author: [ZCode]
created_at: 2026-09-09
updated_at: 2026-09-09
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-lang/frontend/overview.md: 修改 —— a2py 糖族恒括号纪律（py_call/py_call_may/py_getattr/py_matmul/py_getitem/py_setitem/py_call0 接收者恒括号发射）；语句体闭包：单表达式块（含裸 return e）lambda 化、含绑定/多语句块显式编译期诊断"
  - "docs/specs/auto-lang/vm/overview.md: 修改 —— 闭包局部帧预留（compile_closure 按需插 RESERVE_STACK n_locals，原缺失致 let 读回垃圾）；collect_free_vars 块内 Stmt::Store(Let/Const) 绑定识别（原误判为捕获变量，STORE 本地/LOAD 捕获错位）"
new_spec_components: []
touched_goals:
  - "GOAL-006"                # Consumer parity：py 套件语料编写体验与三轨语义面（review 定稿）

affects: [auto-lang/trans, auto-lang/vm]
current_step: 8
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

- [x] **T-01** 基线：master 提交骨架；建 worktree；记录修复前基线
  （`cargo tt` 现状、a2p lambda goldens 现状、`p7 py_list` 相位快照）。
  [✅ 已完成] master e8f67b9b2；worktree D:/autostack/.wt/lang-598/auto-lang
  （plan-598-dev）；基线=cargo tt 3840/3841（唯 charts_gallery 预存红,master
  同款）+ py_list 8/8。执行注记:①nextest 解析 autodown-core 需分组兄弟仓,
  按分组布局补建 auto-down detached 兄弟 worktree(D:/autostack/.wt/lang-598/
  auto-down @ b616bc2);②worktree 冷构建 auto(~3min)曾与 tt 后台任务抢锁
  竞态一次,重跑即过。
- [x] **T-02** P539-D3 糖族括号（D1）：trans/python.rs 五臂审计 + 统一
  `(RECV)` 发射 + 受影响 goldens 更新。验证：`cargo tt` 全绿；d3 探针
  产物核对。
  [✅ 已完成] **审计扩容:五臂→七臂**——py_call/py_call_may/py_getattr/
  py_matmul/py_getitem/py_setitem/py_call0(同族成员访问发射全改恒括号,
  emit_paren_expr helper);getattr_may/getitem_may 原本安全(getattr() 调用
  括号内/已有包裹)不改;py_callable 恒等发射不改。既有 goldens 零破坏
  (tt 3840/3841——py 套件产物不进 tt golden,parity 比 TAP 不比文本)。
  d3 探针:`print((t + tensor([1, 1, 1])).sum())` ✓。
  执行注记:修复过程中发现首个 build 竞态导致探针误读旧产物一次(cl3 陈旧
  文件事件),后续验证一律先删产物文件。
- [x] **T-03** VM 闭包尾值（D3）：有界调查（定位块求值返回通道，写决策
  注记）→ 定点修；时间盒 1h，超限走降级路径并回填 T-07 账面。
  验证：`cargo tv` 全绿；cl.as 形态返 6。
  [✅ 已完成] **实勘扩面+双根因修复(轻微超时盒,未走降级)**:VM 缺口并非
  "尾值传播"——追踪+探针矩阵钉死双根因:①collect_free_vars 不识别块内
  Stmt::Store(Let) 绑定,y 被误判捕获变量,STORE 落本地槽/LOAD 走捕获环境
  读垃圾(codegen.rs L12523 Block 臂,原代码 TODO 在案);②闭包体无局部帧
  预留(fn 有 FN_PROLOG+RESERVE_STACK,闭包没有),let 的 bp 相对寻址落
  未分配栈。修复=Block 臂纳入 let/Const 绑定排除(初值先走、绑定后生效)
  + compile_closure 按需插 RESERVE_STACK n_locals(镜像 fn 的
  max_locals reset/diff + 插入位移舞蹈:jump_placeholders/relocs/exports
  ≥func_addr +2)。探针矩阵全绿:iso(f=8/g=8/h=8)/cl(8)/cl2(6,6)/
  v(3,6,4,8,5,4)。cargo tv 3626/3627 唯预存红。**升级注记**:根因是
  闭包局部槽寻址专属(非块求值通用语义),未触发待澄清 #1 的强修禁令。
- [x] **T-04** a2py 语句体闭包（D2）：Expr::Closure 块体分类 + 单表达式块
  `(expr)` 发射 + 含语句块诊断。
  验证：`cargo tt` 全绿；cl 探针两形态产物核对。
  [✅ 已完成] Closure 臂分类:block_single_expr 递归解包(parser 对闭包体
  有嵌套 Block 层)——单表达式块/裸 return → `lambda x: expr`(实现取无括号
  等价形式,计划原文 `(expr)` 的括号非必需,偏差注记);含绑定/多语句 →
  显式诊断「a2py: statement-body closures are not supported…」。cl3 探针:
  f=`lambda x: x * 2` ✓;cl=诊断中止且不落产物 ✓。
- [x] **T-05** 语料：a2p lambda 块体 goldens（新/更新）+ cl/d3 探针固化。
  验证：`cargo tt` 含新 goldens 全绿。
  [✅ 已完成] 新语料×2:16_python_std/003_py_call_compound(括号发射 golden;
  初版撞号 002 已改 003)+05_expressions/013_lambda_block(单表达式块/return
  块);诊断单测 test_lambda_statement_body_diagnostic(assert 错误文案)。
  cargo tt 3843/3844 唯预存红。提交 1bec2147b。
- [x] **T-06** de-workaround：grep 定位 py 套件中间变量规避点，改回直写。
  验证：`parity` 下 `phase p7` + 触及套件单跑零回归。
  [✅ 已完成] **判定:零活跃规避点**——grep 全 py 套件 .as,py_call 接收者
  均为 Ident/调用形态(本就正确);P539-D3 的中间变量规避历史未落在现行
  套件(债条目示例出自 539 W 阶段探针)。验证式清零,证据=本条 grep
  (configparser/train 全量目视)。零改动。
- [x] **T-07** 账面收口（AC-05）：KNOWN-DEBT P539-D3 销账；
  known-divergences DIV-PY-CLOSURE-1 拆面更新；路线图 §7.4 ⑦ 核销。
  验证：三文件 diff 人工核对。
  [✅ 已完成] KNOWN-DEBT P539-D3 销账(七臂+golden 003+零活跃规避点复核);
  DIV-PY-CLOSURE-1 拆面:VM 面已修(双根因)+a2py 面已修+**句柄裸 id 面
  连带治愈**(hd.as 探针:闭包内 let + py 调用 = 15 ✓——free-vars 误判正是
  其根因;仍 open 缩为 Python 侧回调消费模型=W3/P539-D4);路线图 ⑦ 核销。
  提交 71b1962a7。
- [x] **T-08** 收口健康检查：`cargo tt` + `cargo tv` 全绿；`p5-p9` 相位
  回归；探针残留扫描；fold 前 `cargo tf` 留给 review 阶段。
  验证：三件套输出留痕进复审记录。
  [✅ 已完成] cargo tt 3843/3844 + cargo tv 3626/3627(唯 charts_gallery
  预存红,master 同款);**p5-p9 全相位 20 套件 179/179 case 零回归**
  (worktree 修复二进制实跑);探针残留扫描=0(PLAN598-DBG 已清,3 处
  unimplemented! 为 master 预存);提交 71b1962a7。

## 9. 复审记录

（起草交接：stage=new | plan_id=PLAN-598 | rev=1 | outcome=pass |
next=work。范围调整留痕：DIV-PY-CLOSURE-1 实勘扩为双轨缺陷，VM let-面纳入
本批（T-03 带时间盒与降级路径），句柄槽裸 id 面显式留 W3。）

```
stage: work | plan_id: PLAN-598 | plan_revision: 1 | outcome: pass |
code_commit: 15e0d0e80(T2-T4) + 1bec2147b(T5) + 71b1962a7(T6-T7)
（worktree plan-598-dev,基于 master e8f67b9b2;分组含 auto-down detached
兄弟 worktree @ b616bc2,nextest autodown-core 解析所需） |
task_ids: T-01..T-08 全部完成 |
evidence: cargo tt 3843/3844 + cargo tv 3626/3627(唯 charts_gallery 预存红,
master 同款);p5-p9 全相位 20 套件 179/179 零回归(worktree 修复二进制);
探针矩阵 iso/cl/cl2/v/hd/f1/d3 全绿或按设计诊断;P539-D3 销账+
DIV-PY-CLOSURE-1 拆面(句柄裸 id 面连带治愈,hd.as=15);路线图 ⑦ 核销;
探针残留 0 |
blockers: 无 |
next: /auto-plan:review(fold 前 cargo tf 由 review 阶段执行)
```

**复审（2026-09-09）。独立性声明**：实现会话内复审，按工件与复跑证据
重建结论。

**复审基线**：plan_revision=1；reviewed_commit=worktree plan-598-dev @
**71b1962a7**（15e0d0e80 + 1bec2147b + 71b1962a7，9 文件 +236/-23）；
base=master e8f67b9b2；工作树 clean；spec 目标文件实证存在
（docs/specs/auto-lang/{frontend,vm}/overview.md）。

### 验收标准逐条复验（verify, don't trust）

| AC | 结论 | 证据 |
|---|---|---|
| AC-01 糖族恒括号 | **pass** | 复审复跑 d3 探针（先删产物）：`print((t + tensor([1, 1, 1])).sum())`；tt 含 golden 003 绿 |
| AC-02 语句体闭包 a2py | **pass** | 复审复跑 f1：`lambda x: x * 2` / `lambda x: x + 5`；cl 诊断触发（grep 计数 1）；单测 test_lambda_statement_body_diagnostic PASS |
| AC-03 语句体闭包 VM | **pass** | 复审复跑 cl.as VM 腿 = 8（非 None，未走降级路径）；f1 = 6/6 |
| AC-04 回归零漂移 | **pass** | cargo tt 3843/3844 + cargo tv 3626/3627（唯 charts_gallery 预存红）；**cargo tf 3485/3486 唯同款预存红（复审复跑）**；p5-p9 全相位 179/179 |
| AC-05 账面收口 | **pass** | 三文件 diff 复核（KNOWN-DEBT P539-D3 销账/DIV-PY-CLOSURE-1 拆面/路线图 ⑦ 核销，措辞与实证一致） |

### 遗漏 / 延后 / Workaround 扫描

- **F1（低，接受）**：单表达式块闭包实现为无括号 `lambda x: expr`（计划
  原文 `(expr)`）——lambda 体整体即表达式，括号非必需，等价且更干净。
- **F2（低，接受）**：T-03 轻微超时间盒（~1.2h）未走降级——根因在时间盒
  末段定位且修复为 15 行级；待澄清 #1 的升级禁令（块求值通用语义）经探针
  矩阵排除（fn 同形态正常，根因闭包专属：free-vars 误判 + 帧预留缺失）。
- **F3（信息）**：T-06 验证式清零——py 套件无活跃规避点（grep 全量 + 目视
  复核），零改动即本任务的正向完成态。
- **F4（信息）**：T-02 审计五臂→七臂（py_matmul/py_getitem/py_setitem 同
  族成员访问发射同病）——计划 D1 的"审计清单以 grep 为准"条款预授权。
- **F5（信息，连带收益）**：collect_free_vars 修复连带治愈 DIV-PY-CLOSURE-1
  原文描述的"句柄槽裸 id"面（hd.as 探针：闭包内 let + py 调用 = 15）——
  仍 open 缩为 Python 侧回调消费模型（W3/P539-D4），账面已如实拆分。
- **剩余已知缺口（pre-existing，代码注释在案）**：collect_free_vars 对块内
  嵌套 If/For/Block 语句仍不遍历（本批修复前的历史行为，非本批引入）。
- **Workaround 扫描**：无。RESERVE_STACK 镜像 fn 既有机制；恒括号为
  Python 恒等变换。

### 结论

**全部验收标准通过（含 cargo tf 全量门），无阻塞性债务 → status:
reviewed**。可入 `/auto-plan:merge`（worktree plan-598-dev 基于 e8f67b9b2，
master 侧仅计划记账文件分叉，合并无冲突预期）。

### 沉淀收据（PLAN-598:r1，2026-09-09）

| 检查点 | 证据 |
|---|---|
| `prepared` | 复审基线 71b1962a7（base e8f67b9b2）；spec delta = SD-01 frontend/overview.md + SD-02 vm/overview.md 现状节；账本投影 = .autoos/specs.json 六节 P598-1..6 |
| `landed` | master **0d3e1881e**（merge plan-598-dev，含 delivery_commit 9cb7efabc docs-only 后代=spec 增量） |
| `ledger_refreshed` | .autoos/specs.json（runtime，gitignored 不提交）六节各插入 P598-1..6，file 指向本文归档路径；读回验证 6/6；INDEX.md 无实质变化（CRLF 噪音归一） |
| `archived` | 本文 git mv 至 docs/plans/archive/598-a2py-semantic-fix-batch.md，status: archived |
| `cleaned` | wt-guard 双 clean（auto-lang + auto-down 兄弟，均无 reparse point）→ 双 worktree 已移除、分支 plan-598-dev 已删（was 9cb7efabc）、组目录 .wt/lang-598 已清空删除（worktree list 复核 0 条目） |

### 规范增量（spec delta，merge 时沉淀；frontmatter 已定稿）

SD-01（docs/specs/auto-lang/frontend/overview.md）与 SD-02
（docs/specs/auto-lang/vm/overview.md）的 before/after 见 §5 规范增量表，
复审已按实际实现对齐（七臂清单、无括号 lambda 形态、RESERVE_STACK 机制、
collect_free_vars 绑定识别）。

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
