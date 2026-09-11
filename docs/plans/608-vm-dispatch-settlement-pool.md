---
plan_id: PLAN-608
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: vm-dispatch-settlement-pool
author: [ZCode]
created_at: 2026-09-11
updated_at: 2026-09-11

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: ["auto-lang/vm"]
current_step: 0
total_steps: 7
---

# [PLAN-608] vm-dispatch-settlement-pool

## 0. 变更摘要

VM 引擎 RC/池结算收口第二批（PLAN-604 直接后继），两件合一批：

1. **KD-VM5（604 收据候选①展开）**：CALL_SPEC **内联方法分发区**
   （engine.rs ~6904-7600，`type_name=="str"`/`"List"` 各臂）raw pop
   不结算 + 内联 `"push"` 臂**容器不 retain**（复核 2026-09-11：
   engine.rs `list.push(elem_val)` 裸存）——若对引用元素可达即悬垂
   引用（UAF 级）而非单纯泄漏。先可达性定罪（resolve 覆盖差集 +
   trace 实证），可达臂按 604 协议收口；**不可达臂按代码逻辑建立
   金样测试钉死行为并守护**（用户指令：路径若未触及，以测试检测）。
2. **KD-VM6（604 执行期新发现）**：**字符串池份额**结算缺口——
   `shim_list_push` 字符串元素入列时暂存池份额不释放（604 只结算了
   堆 stake；池份额不入影子、`elem_stake` 恒 0），且 CALL_SPEC→
   resolve 路径无死区结算（字符串本该由死区按内容释放）→
   `List<str>.push` 每调用孤儿 +1 池份额，池条目被永久钉死无法回收
   （StrPush 探针测堆对象显示 0，池增长不在 `heap_live_objects`
   口径）。内联 str 臂接收者同型。修 shim + 可达内联臂，pool_soak
   增加字符串列表 churn 相位做守护。

## 1. 目标

- **G1（可达性定罪）**：内联 str/List 分发区每臂的可达代码形态清单
  落档（臂 × 触发条件 × 结论，行号级 + trace 实证）——判定哪些臂
  活跃、哪些为死码。
- **G2（可达臂结算收口）**：活跃 List 臂容器写语义对齐 604 协议
  （元素转移/补 retain、暂存份额结算）；活跃 str 臂接收者/实参
  份额结算（堆按影子、池按内容）。
- **G3（不可达臂行为钉死）**：对定罪为不可达的臂（含内联 "push"
  的不 retain 形态），按代码逻辑建立金样测试钉死当前行为——路由
  变化（registry 增删）触雷时测试即红。
- **G4（池份额零孤儿）**：`List<str>.push` 与 str 方法调用 churn 的
  池 `live_shares` 增量有界；pool_soak 增加字符串列表 churn 相位
  （Plan 510 health 口径：underflow=0、稳态有界）。
- **非目标**：CALL_NAT 已有死区结算路径的改动；Q-03
  construct.instance 双步合并；不可达臂的主动删除/重构（只钉行为、
  登记移除候选，删除与否留用户裁定）；604 已收口面（shim 堆 stake、
  ARRAY_LEN、CONSTRUCT_INSTANCE）的返工。
- **成功判据**：AC-01~06 全绿；字符串列表 churn 的池守护相位
  进日常档。

## 2. 架构方案

不引入新组件；在 Plan 419「copy-on-load 所有权协议」+ PLAN-604
「结算语义」框架内补两个执行面（偏差纪律不变：漏 retain=悬垂致命、
漏 release=泄漏安全，先定罪后动手、每步回归）：

```
现状（604 执行期复核 + 本计划 T-01 定罪对象）：
  CALL_SPEC 分发链（engine.rs ~6800-7600）：
    opaque_native → math_native → resolve(&func_name) [→ shim，
    604 已收口堆 stake] → type_name=="str" 内联区(6904) →
    type_name=="List" 内联区(7220)
  缺口 A（KD-VM5）：内联 List 区仅 resolve miss 可达。registry 覆盖
    List.push/len/join/...（canonical 归一 ~27 shim），但内联臂
    清单含 last/count/insert/remove/...——差集=可达性关键。
    内联 "push" 臂 list.push 裸存不 retain：可达即 UAF 面。
  缺口 B（KD-VM6）：池份额。
    ①shim_list_push 字符串分支：容器经 list_i32_elem_retain/
    负哨兵 retain 池（容器份额 ✓），但暂存拷贝的池份额无
    pool_release（elem_stake 对字符串恒 0；resolve 路径无死区
    按内容结算）→ 每调用 +1 孤儿。
    ②内联 str 臂接收者：copy-on-load rc_push 池计数 +1，臂内
    raw pop 无 pool_release → 每调用 +1 孤儿（dedup 使重复串
    共享条目，但孤儿份额使条目永不可回收）。

修复面：
  A. T-01 定罪 → 可达臂按 604 纪律收口（retain/transfer/release）；
     不可达臂不动代码，T-05 金样钉死。
  B. shim_list_push 字符串分支补 pool_release（暂存份额结算，
     容器份额保留）；可达 str 臂接收者收尾补 pool_release。
     pool_soak_churn_short 增加字符串列表 churn 相位守护。
```

## 3. 技术栈

Rust（auto-lang crate：vm/engine.rs 内联分发区、vm/native.rs
shim_list_push、vm/tests_string_pool.rs pool_soak）；测试用既有
nextest 分层 + `P419_UAF_TRACE` 窄窗口 + `PoolHealth` 快照
（underflow_events/live_shares）。无新增外部依赖。

## 4. 需求分析与背景调查

- **授权记录**：用户 2026-09-11 指示「第1+2件合并成一个计划立项。
  如果代码没触及到，可以考虑根据代码逻辑建立对应的测试用例来检测
  它们」；范围=vm/engine.rs 内联分发区、vm/native.rs shim 池结算、
  tests_string_pool.rs pool_soak 相位；工作流=标准 auto-plan 四技能
  （worktree `D:/autostack/.wt/lang-608/auto-lang` 分支
  plan-608-dev）；预算未设，按阶段推进。
- **Spec 现状**：`docs/specs/auto-lang/vm/overview.md` §RC 生命周期
  协议（PLAN-604 落稿）已载「消费型 raw pop 必须收尾」契约，但执行
  点清单未含内联分发区与池份额语义——本计划补（SD-01/SD-02）。
  Plan 510 池记账节（同文件现状列表）为池口径来源。
- **背景证据（全部可复现）**：
  - 分发链顺序：engine.rs ~6800-6904 opaque→math→**resolve**→
    str 内联(6904)→List 内联(7220)——resolve 命中即走 shim
    （604 backtrace 实证 List.push 走 resolve→shim_list_push）。
  - registry 覆盖：native_registry.rs canonical 归一
    （"List.push"→"auto.list.push"），shim_list_* 家族 ~27 个
    （native.rs 2093-8436）；内联臂清单含 last/count/push/insert/
    remove 等——**差集未盘点**（T-01 定罪对象）。
  - 内联 "push" 臂容器不 retain：engine.rs `list.push(elem_val)`
    裸存（604 执行期复核 2026-09-11）。
  - 池份额缺口：native.rs shim_list_push `elem_stake =
    take_stake_at(sp)` 对字符串恒 0（池份额不入影子，
    virt_memory.rs 251-256 注记）；`list_i32_elem_retain` 为容器
    负哨兵 retain 池——暂存份额无配对释放；CALL_NAT 死区
    （rc_release_slot_range is_string 臂按内容释放）不在
    resolve 路径上。
  - 内联 str 臂 raw pop：`{ for _ in 0..=arg_count { pop_nv(); } }`
    形态（starts_with/trim/includes/len/count 等臂）。
  - Plan 510 守护通道：`PoolHealth`（underflow_events/phantom_drops/
    live_shares）+ `pool_soak_churn_short/long`
    （tests_string_pool.rs）——本计划扩展相位。
- **相关 Plan**：PLAN-604（前作：sp-1 取槽/shim 堆 stake transfer/
  ARRAY_LEN 收尾/探针方法论）、Plan 419（RC 协议）、Plan 510（池
  记账）、Plan 403/053（内联臂历史成因：last 族、P053-6 web 生态
  字符串方法族）。

## 5. 详细设计

### 5.1 KD-VM5 内联分发区（T-01 定罪 → T-02 收口/T-05 钉死）

- **T-01 定罪对象**（bounded，产出决策工件）：
  ①resolve 覆盖差集盘点：`AutoVMNativeRegistry` 已注册名清单 vs
  内联 str/List 臂方法名清单（机械枚举，静态可做）；
  ②运行期实证：P419_TRACE/AUTO_DEBUG 临时打印于内联区入口
  （6904/7220），跑 tv 语料 + 探针 + sys-monitor 冒烟，记录哪些臂
  真实命中；③结论矩阵（臂×可达性×风险级）写入 §9。
- **可达臂收口**（T-02）：List 臂容器写按元素类型 retain/transfer
  （604 shim_list_push 同款）；str/List 臂 raw pop 后按 DROP 纪律
  收尾（堆按影子、池按内容）。每臂改后即跑 `cargo tv` 分层回归。
- **不可达臂钉死**（T-05）：金样测试按**代码逻辑**直接构造分发态
  （如绕过 registry 构造 resolve miss 的调用形态，或直接以
  AutoVM::run_one_instruction 喂字节码序列）钉死当前行为——测试
  断言的语义=现行为，并加注「registry 覆盖变化会使本测试路由
  改变」的守护注释；内联 "push" 不 retain 形态若判不可达，测试
  钉死 + KNOWN-DEBT 登记移除候选（删除留用户裁定，本计划不删）。

### 5.2 KD-VM6 池份额（T-03 shim + str 臂、T-04 soak 相位）

- **shim_list_push 字符串分支**：在容器池 retain 之后补
  `vm.pool_release(idx)`（暂存份额结算；容器份额独立保留）——
  释放序在容器 retain 之后，杜绝 rc 短暂下探；ListData<String>
  分支（存字节拷贝、容器不持池份额）同样释放暂存份额。落空分支
  同 604 口径一并结算。
- **可达 str 臂接收者**：raw pop 后 `pool_release(decode_string)`
  （暂存池份额收尾；与 604 ARRAY_LEN 的 is_string 收尾同款）。
- **T-04 pool_soak 相位**：`pool_soak_churn_short` 增加字符串列表
  churn 相位（循环 `List<str>` 构造 + f"..." 唯一串 push + len/
  contains 调用 + 丢弃），断言该相位 `live_shares` 增量有界、
  `underflow_events==0`——先红（现状孤儿 +N/拍）后绿。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/vm/overview.md（§RC 生命周期协议·消费型 raw pop 执行点清单） | before: 执行点=ARRAY_LEN/GET_FIELD 尾/GET_ELEM/CALL_NAT 死区/native pop_arg+StakeGuard；after: 增补「CALL_SPEC 内联分发区臂（str/List）与 resolve→shim 臂同守结算契约；池份额按内容结算属于消费型收尾的一部分」 | KD-VM5 定罪后的契约固化；内联臂此前无记载（604 同型缺口第二处） | AC-02/AC-05 |
| SD-02 | modify | docs/specs/auto-lang/vm/overview.md（§RC 生命周期协议·容器写条目） | before: 「容器写=新值转移或补 retain」仅堆份额口径；after: 增补池份额口径「字符串元素入列/消费时，暂存池份额按内容结算，容器份额独立保留」 | KD-VM6 根因即池份额语义未入契约 | AC-03/AC-04 |

## 6. 测试设计

- **可达性探针**（T-01，临时）：内联区入口环境门控打印
  （AUTO_DEBUG_DISPATCH）+ tv 语料/探针/sys-monitor 冒烟跑批，产出
  命中矩阵；定罪后移除临时打印。
- **行为金样**（T-05）：直接字节码构造（`AutoVM` + flash 字节序列，
  沿 vm_types_tests 形态）或 resolve-miss 形态语料，钉死不可达臂
  当前行为；守护注释绑定 registry 覆盖。
- **池守护**（T-04）：pool_soak_churn_short 增相位
  （List<str> push/len/contains churn ×N），断言 live_shares 有界 +
  underflow 0；现状跑应红（孤儿 +N/拍）作为 red 证据，修复后绿。
- **回归分层**：每任务 `cargo tv`；收尾 `cargo tf`；探针/池相位纳入
  日常档。

## 7. 验收标准

- **AC-01（定罪矩阵）**：§9 落臂×可达性×风险结论矩阵，含 resolve
  差集清单与 trace 实证摘录（行号级）。验证：人工审阅 + trace 日志
  摘录在案。
- **AC-02（可达 List 臂收口）**：可达的引用元素入列路径，容器恰持
  一份、暂存份额结算（探针/trace 断言 rc 轨迹配平）；churn 增量
  ≤64。验证：`cargo test --features ui-iced probe_`（或 T-01 产出
  的对应形态测试）。
- **AC-03（池孤儿归零）**：`List<str>.push` churn 相位 live_shares
  增量有界（断言进 pool_soak 相位，非打印）。验证：`cargo test -p
  auto-lang pool_soak_churn_short`。
- **AC-04（str 方法池结算）**：resolve-miss 形态的 str 方法调用
  （如 includes/trimEnd 族）接收者池份额零孤儿（trace 断言
  pool_retain/release 配对或 live_shares 稳态）。验证：T-02 产出
  的专项测试。
- **AC-05（不可达臂钉死）**：定罪为不可达的臂各有金样测试（按代码
  逻辑构造，断言现行为），注册于日常档；内联 "push" 形态的
  KNOWN-DEBT 移除候选登记在案。验证：`cargo test -p auto-lang`
  新增测试全绿。
- **AC-06（回归零漂移）**：`cargo tv --no-fail-fast` 唯一红=
  charts_gallery 在案预存；收尾 `cargo tf` 同口径。

## 8. 执行步骤

- **T-01 可达性定罪（bounded，产出决策工件）**：resolve 差集静态
  盘点 + 内联区入口临时打印 + tv/探针/sys-monitor 冒烟跑批 → 臂×
  可达性×风险矩阵写入 §9。验证：矩阵覆盖内联区全臂，活跃/死码
  各有 trace 证据。依赖：无。
- **T-02 可达臂结算收口**：engine.rs 内联区按 T-01 矩阵逐臂收口
  （List 容器写 retain/transfer；str/List raw pop DROP 纪律收尾，
  池按内容）。验证：`cargo tv` 每臂分层零新增红 + AC-02/AC-04 专项
  测试。依赖：T-01。
- **T-03 shim_list_push 池份额结算**：native.rs 字符串分支补
  pool_release（暂存份额；retain 后释放）。验证：AC-03 相位红→绿。
  依赖：无（可与 T-01 并行，shim 路径已证可达）。
- **T-04 pool_soak 字符串列表 churn 相位**：tests_string_pool.rs
  增相位 + 硬断言。验证：AC-03。依赖：T-03。
- **T-05 不可达臂行为金样**：按代码逻辑构造测试钉死现行为 +
  KNOWN-DEBT 移除候选登记。验证：AC-05。依赖：T-01。
- **T-06 回归门**：`cargo tv --no-fail-fast` → `cargo tf`（唯一红=
  在案预存）。关联 AC-06。依赖：T-02/T-03/T-04/T-05。
- **T-07 SPEC 回写**：SD-01/SD-02 落 vm/overview.md；KD-VM5/6 入
  KNOWN-DEBT 附录（604 附录续节）。依赖：T-06。

## 9. 复审记录

- 2026-09-11 draft v1（auto-plan:new）：PLAN-608 立案。stage: new，
  outcome: pass，next: work。证据基础=PLAN-604 执行期复核（内联
  "push" 臂裸存 2026-09-11）+ 604 收据候选① + 池份额缺口分析
  （shim_list_push elem_stake 对字符串恒 0、resolve 路径无死区）。
  用户指令：不可达路径以代码逻辑测试钉死守护。

## 10. 待澄清事项

- **Q-01**：定罪为死码的内联臂（含不 retain 的 "push" 形态）是否
  直接删除——本计划倾向只钉行为+登记候选（删除面广、涉及 Plan
  403/053 历史成因），删除决策留用户。
- **Q-02**：str 内联臂的池孤儿量级在真实应用尚未实测（dedup 使
  重复串共享条目、只影响可回收性不影响条目数）——T-01 跑批时顺带
  量化，若量级可忽略则 AC-04 收窄为「有界」口径。
