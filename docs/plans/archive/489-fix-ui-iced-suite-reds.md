---
plan_id: PLAN-489
status: archived               # drafting → executing → execution_done → reviewed → archived（终态）
feature_name: fix-ui-iced-suite-reds
author: [zhaopuming]
created_at: 2026-08-30
updated_at: 2026-08-30

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: 新增注记——desktop_protocol broker 可测性缝 adjudicate_on(pipe,args,timeout)（adjudicate 生产签名委托固定 BROKER_PIPE 零变化；Broker::on_pipe 同型先例）：broker 管道类测试一律 pid 后缀管道 hermetic 化，禁止依赖生产固定管道的全局命名空间状态（本机桌面宿主 listen 即打穿 Standalone 断言，P487-2 间歇红教训）"
  - "docs/specs/auto-lang/ui/overview.md: 新增注记——测试 corpus i18n 资产入库约定：front/i18n/{lang}.json 属 corpus 必备资产（.gitignore *.json 母规则需 test/**/i18n/*.json 否定）；fresh-clone 绿是 corpus 落测的验收前提（P487-2 i18n 双测教训——落测时被 ignore 吞未入库）"
touched_goals: []

affects: [auto-lang/ui, auto-lang/vm]
current_step: 4
total_steps: 4
---

# [PLAN-489] fix-ui-iced-suite-reds——ui-iced 特性档 4 既有红测试清零

## 变更摘要

修复 P487-2 债（487 复审发现）：`cargo nextest run -p auto-lang --lib
--features ui-iced` 全量下 4 个既有红测试（master 同红，标准门禁
`cargo t`/`tf` 默认特性档盲区看不见）。四项根因已逐一诊断（2026-08-30）：

1. **i18n 双测**（plan442_ext_link `plan050_void_stub…` + i18n_lookup
   `plan050_i18n_lookup…`）：corpus `test/ui/plan050_stub_nil/src/front/
   i18n/zh.json` 被 `.gitignore:144` 的 `*.json` 规则吞——**从未入库**
   （ff7eb1261 落测时树里无此文件；作者本地有文件故绿过，fresh clone
   必红）。修：补文件 + .gitignore i18n 目录否定规则。
2. **broker `adjudicate_three_steps`**（间歇红，全量并发/环境下 2/3 次）：
   测试步骤③ 断言生产固定管道 `autodesk-broker` 无监听——本机任何桌面
   宿主进程（如存活 release auto.exe）或并行窗口期 listen 即打穿 →
   `Broker ≠ Standalone`。修：hermetic 化——`adjudicate` 参数化管道
   （新增 `adjudicate_on(pipe, args, timeout)`，原签名委托固定管道，
   生产零变化），测试全程 pid 后缀管道（`Broker::on_pipe` 既有先例）。
3. **`vm_code_editor_natives_end_to_end`**：VM `print` 的 bool 渲染已演进
   为 `true/false` 字面量（失败输出实证 `out2 = "true\n2\nfalse\n0"`），
   断言与注释停在旧 `1/0` 语义。修：断言对齐现语义（逐行精确比对）。

## 目标

- **G1**：`--features ui-iced --lib` 全量 **4066+ 零红**（4 修复项全绿，
  其余不回归）。
- **G2**：adjudicate 测试 hermetic——不再依赖全局命名空间状态（任何机器/
  任何并行负载下确定性绿）。
- **G3**：i18n corpus 资产入库（zh.json 进版本库），fresh clone 即绿。
- **非目标**：ui-iced 档纳入周期门禁（P487-2 建议的流程面，另行决策）；
  broker 生产行为变化（零变化）；print 语义回退（保持现 true/false）。

## 架构方案

```
①② corpus 资产：  test/.../front/i18n/zh.json（新，settings.title=设置）
                  + .gitignore 否定 !crates/auto-lang/test/**/i18n/*.json
③ 可测性缝：      broker.rs adjudicate_on(pipe,…) 新参数化入口
                  （adjudicate(args,t) = adjudicate_on(BROKER_PIPE,…) 委托）
                  测试：Broker::on_pipe(pid 后缀) + adjudicate_on 同管道
④ 断言对齐：      vm/native.rs:9395-9400 四断言改逐行精确比对
```

分界：③ 是唯一触生产代码的项——纯参数化抽取，无行为变化（`Broker::new`
→ `on_pipe` 同款既有缝）；①②④ 均为测试资产/断言侧。

## 需求分析与背景调查

（取材 P487-2 债登记 + 2026-08-30 复诊实证）

- **i18n**：i18n_lookup.rs `load_from_dir` 读 `i18n/{AUTO_LOCALE:-zh}.json`
  平铺点分 key；corpus app.at:31 `text i18n.t("settings.title")`；plan442
  测试断言文案「设置」上屏 + lookup 返回 Some("设置")。`.gitignore` 仅
  否定过 layout_cases.json 一例（:146），i18n 目录无否定。
- **broker**：`adjudicate`（broker.rs:40）三步裁决，②③ 探测固定
  BROKER_PIPE；transport::connect 对 ERROR_FILE_NOT_FOUND(2)/BUSY(231)
  重试至 deadline。测试 :225-248 步骤③ `adjudicate(&[], 30)` 期望
  Standalone。`Broker::on_pipe` 与 pid 后缀惯例已存在（:250-254 注释
  "防并行测试进程串扰"）——adjudicate 测试本身漏用。
- **code_editor**：native.rs:9380-9400 fold natives 往返；注释自述
  "print renders bools as 1/0"——失败输出证实现语义为 true/false。

## 详细设计

### 1. i18n corpus 资产（①②）

- 新建 `crates/auto-lang/test/ui/plan050_stub_nil/src/front/i18n/zh.json`：
  `{"settings": {"title": "设置"}}`（flatten 产点分键，两测试断言同源）。
- `.gitignore` 在 `!crates/auto-lang/src/ui/layout_cases.json` 邻位增：
  `!crates/auto-lang/test/**/i18n/*.json`（覆盖未来 i18n corpus）。

### 2. broker hermetic 化（③）

- broker.rs：`pub fn adjudicate(args, t)` 体重构为委托
  `pub fn adjudicate_on(pipe: &str, args: &[String], t: u32)`（②探测
  `connect(pipe, t)`；①③ 不涉管道）。doc 注明生产 = 固定管道。
- 测试 adjudicate_three_steps：步骤① 不变（无管道）；步骤②③ 改
  pid 后缀管道——③ 先证"该管道无人 listen → Standalone"，② 起
  `Broker::on_pipe(pid_pipe)` worker + `adjudicate_on(pid_pipe, &[], 2000)`
  → Broker；结尾唤醒 connect 也指向 pid 管道。

### 3. code_editor 断言对齐（④）

- native.rs fold 往返四断言改精确比对：`out2.lines() == ["true","2",
  "false","0"]`（首 toggle true/隐 2 行/次 toggle false/归 0）；注释改
  "print renders bools as true/false"。

## 测试设计

- 修复项自身即测试（4 红转绿）；无新测试面。
- T1 复现验证：i18n 双测 solo 红→绿；adjudicate 在"固定管道被占"模拟下
  依旧绿（hermetic 证明——起一个固定管道 Broker 再 solo 跑，修复前红
  修复后绿）；code_editor solo 红→绿。
- T2 回归：`--features ui-iced --lib` 全量 4066+ 零红（连续两次，防间歇）；
  默认档 `cargo t` 3282+ 全绿；`cargo check -p auto-lang` 零新增警告。

## 验收标准

1. `--features ui-iced --lib` 全量零红（连续两次）。
2. i18n zh.json 入库（fresh clone 可绿——git ls-files 可见）。
3. 默认档 `cargo t` 全绿；`cargo check -p auto-lang` 零新增警告。
4. 生产零行为变化：adjudicate 生产签名/语义不变（委托重构）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **i18n corpus 资产**：新建 zh.json + .gitignore 否定规则。
   验证：`cargo nextest run -p auto-lang --lib --features ui-iced
   plan050_i18n_lookup plan050_void_stub`（2 绿）+ `git ls-files` 可见。
   [✅ 已完成] zh.json（{"settings":{"title":"设置"}}）入库 +
   !crates/auto-lang/test/**/i18n/*.json 否定规则（check-ignore 实证
   命中否定）；双测 2/2 PASS（红→绿）
2. **broker hermetic**：adjudicate_on 参数化 + 测试改 pid 管道。
   验证：solo 绿 + 固定管道占用模拟下仍绿。
   [✅ 已完成] adjudicate_on(pipe,…) 参数化缝（adjudicate 委托，生产零
   变化）+ 测试全程 pid 后缀管道。**A/B 双向实证**：PowerShell 持有
   autodesk-broker 固定管道监听时——master 旧测 FAIL（步骤③ Standalone
   被打穿 0.07s）/worktree 新测 PASS（0.62s）；solo + broker 全组 2/2 绿
3. **code_editor 断言**：四断言逐行精确比对 + 注释对齐。
   验证：solo 绿。
   [✅ 已完成] native.rs fold 往返断言改 `lines() == ["true","2","false",
   "0"]` 精确比对（注释同步 print bool 现语义）；solo PASS（红→绿）
4. **全量收尾**：ui-iced 全量连跑两次零红 + 默认档 cargo t 全绿 + check
   零新增警告；P487-2 债状态更新（KNOWN-DEBT 加修复注记）。
   验证：两档全量输出留痕。
   [✅ 已完成] ui-iced 档 **4074/4074 两连绿**（含 broker 并发全量两连绿
   ——hermetic 生效）；默认档 `cargo tf` 3283/3283 全绿；check 警告 161 =
   master 基线 161（零新增）；P487-2 债条目加「Plan 489 已修」注记
   （「档纳入周期门禁」流程面仍开放另议）

## 复审记录

**(/auto-plan:review 2026-08-30，zcode；worktree plan-489-dev @ ae066497a，基点 92e2ba9b5；净 diff 4 文件 +41/-14 = 计划 §架构方案 声明集逐一吻合)**

### 逐项验收判定（复跑实证）

| # | 验收标准 | 判定 | 证据 |
|---|---|---|---|
| 1 | ui-iced 档全量零红（连续两次） | **PASS** | 复审自跑 `--features ui-iced --lib` **4074/4074 × 2 连绿**（含 broker 并发全量两连绿——hermetic 化消除间歇源） |
| 2 | i18n zh.json 入库（fresh clone 可绿） | **PASS** | `git ls-files` 可见 + `git cat-file -p HEAD:…zh.json` blob 直读（{"settings":{"title":"设置"}}）+ git archive 解包实际物化双证 |
| 3 | 默认档 cargo t 全绿；check 零新增警告 | **PASS** | 复审自跑 `cargo tf` 3283/3283；警告 161（分支）= 161（master）零新增 |
| 4 | 生产零行为变化 | **PASS** | adjudicate = adjudicate_on(BROKER_PIPE,…) 纯委托（体逐行同原，仅常量换参）；唯一生产调用点 dual_mode.rs:241 签名/实参不变；print 语义未动（断言侧对齐） |

**专项复核**：broker 间歇根因 A/B 双向实证（执行期）——PowerShell 持有
autodesk-broker 固定管道监听时 master 旧测 FAIL（Standalone 被打穿）/
本分支新测 PASS；hermetic 结论不依赖单次绿。

### 遗漏/延后/workaround 扫描

- 净 diff = 计划全部声明项，零缺漏；零 TODO/FIXME/HACK/dbg/println 新增。
- ③ 参数化缝为正规可测性设计（Broker::on_pipe 同型先例），非 workaround。
- 开放项一处（非静默，双留痕）：「ui-iced 档纳入周期门禁」= 计划非目标
  章节 + P487-2 债注记「另议」——流程决策面，不阻塞本计划。

### 结论

**通过。** 四项验收全 PASS、双档全量绿、零静默延后/零 workaround、
生产零行为变化。→ `status: reviewed`。

## 待澄清事项

- 无（四根因均已诊断实证；修复方案均为最小侵入）。
