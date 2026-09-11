---
plan_id: PLAN-608
status: archived                # drafting → executing → execution_done → reviewed → archived（终态）
feature_name: vm-dispatch-settlement-pool
author: [ZCode]
created_at: 2026-09-11
updated_at: 2026-09-11

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/vm/overview.md: §RC 生命周期协议·消费型 raw pop 执行点清单（SD-01——CALL_SPEC 分发区①resolve→shim 死区②内联消费臂弹窗结算③不可达臂路由守护）"
  - "docs/specs/auto-lang/vm/overview.md: §RC 生命周期协议·容器写条目池份额口径（SD-02——暂存池份额按内容结算归属死区单点、容器份额独立保留、shim 不做池释放）"
touched_goals: []             # 同 604 先例：RC 结算协议硬化为 vm 内部内存协议工作，不挂接 roadmap 目标行（GOAL-001 生命周期族为松邻，无直接交付物映射）

affects: ["auto-lang/vm"]
current_step: 7
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

结算原语（2026-09-11 复核确认，无需新写）：rc.rs `rc_release_slot_range`
（rc.rs:418）即「堆按影子、池按内容」的死区结算——CALL_NAT 臂
（engine.rs:7980 形态：记 sp_before → 执行 → release(sp_after, sp_before)）
同款。T-02 内联臂收尾直接复用它覆盖弹出窗口，与 §5.1 口径逐字对应。
```

## 3. 技术栈

Rust（auto-lang crate：vm/engine.rs 内联分发区、vm/native.rs
shim_list_push、tests/plan510_pool_tests.rs pool_soak——复核更正：
非 tests_string_pool.rs）；测试用既有
nextest 分层 + `P419_UAF_TRACE` 窄窗口 + `PoolHealth` 快照
（underflow_events/live_shares）。无新增外部依赖。

## 4. 需求分析与背景调查

- **授权记录**：用户 2026-09-11 指示「第1+2件合并成一个计划立项。
  如果代码没触及到，可以考虑根据代码逻辑建立对应的测试用例来检测
  它们」；范围=vm/engine.rs 内联分发区、vm/native.rs shim 池结算、
  tests/plan510_pool_tests.rs pool_soak 相位；工作流=标准 auto-plan 四技能
  （worktree `D:/autostack/.wt/lang-608/auto-lang` 分支
  plan-608-dev，2026-09-11 建立于 master beb9b57e2）；预算未设，按阶段推进。
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
    live_shares，rc.rs:188）+ `pool_soak_churn_short/long`
    （tests/plan510_pool_tests.rs，churn 载体已含 List<str>.new/get
    但**无 push 相位**——T-04 增量定位由此证实）。
- **静态差集预答（2026-09-11 work 前复核，T-01① 的静态部分已完成；
  运行期 trace 实证仍由 T-01 执行）**：resolve 覆盖面 =
  native_catalog.rs shim 表第四字段 canonical 名 ∪ ffi rust_fn
  inventory 名（build_from_inventory 注册 canonical+小写变体）∪
  register_std_shims 手写别名。据此刻出：
  - **str 内联臂可达**（"str.X"→"auto.str.X" miss）：as_bytes、
    to_uppercase、lower、to_lowercase、chars、graphemes、
    trimEnd/trim_end、includes、indexOf/indexOf_str/lastIndexOf/
    last_index_of、substring/substring_str、char_code_at/charCodeAt、
    to_int、to_float、`_` 未知兜底（**含类型名接收者形态**：未注册
    静态调用如 JSON.stringify，接收者字符串经 type_name 判定落
    "str"，其池份额被裸弹）；to_string/to_str/clone 为恒等臂
    （接收者留栈，无消费无孤儿）。
  - **str 内联臂不可达**（resolve 命中 shim）：upper/to_upper/
    to_lower/split/is_empty/starts_with/ends_with/contains/len/
    trim/replace/parse_int/parse_float。
  - **List 内联臂可达**：count（count|len 臂——"len" 命中而
    "count" miss）、last、dedup（sort|dedup|reverse 臂——sort/
    reverse 命中而 dedup miss）、恒等名单（collect/rev/filter_map/
    flatten/into_iter/iter_mut/par_iter/par_iter_mut/fold/to_array，
    仅弹实参接收者留栈）、`_` 未知兜底。
  - **List 内联臂不可达**：**push**（UAF 面——auto.list.push 恒命中
    shim_list_push）、get/remove/pop/insert/sort/sort_by/reverse/
    len/map/filter 等全部 registered 名。
  - **复核新发现①**：可达 List 臂（count/last/dedup/_ 兜底）的
    接收者 list_id 槽同样带 copy-on-load 堆份额+影子（LOAD_LOCAL
    rc_push，engine.rs:8779），raw pop_nv 不清影子不释放 → 每调用
    heap 孤儿 +1（泄漏向，安全方向但违反 604 契约）——§5.1「堆按
    影子」口径的必要性由此证实，T-01 矩阵加此列。
  - **复核新发现②**：内联 "push" 臂头注释声称「mem 复盘修复…
    必须结算其槽位 stake」（engine.rs:7343-7348）但实现无结算——
    注释与代码脱节，T-05 钉死时一并修正注释。
- **相关 Plan**：PLAN-604（前作：sp-1 取槽/shim 堆 stake transfer/
  ARRAY_LEN 收尾/探针方法论）、Plan 419（RC 协议）、Plan 510（池
  记账）、Plan 403/053（内联臂历史成因：last 族、P053-6 web 生态
  字符串方法族）。

## 5. 详细设计

### 5.1 KD-VM5 内联分发区（T-01 定罪 → T-02 收口/T-05 钉死）

- **T-01 定罪对象**（bounded，产出决策工件）：
  ①resolve 覆盖差集盘点：**静态部分已由 work 前复核完成**（§4
  静态差集预答——可达/不可达臂清单在案），T-01 只需运行期 trace
  佐证活跃臂（静态 miss ≠ 运行期有真实调用方；如 count/dedup 的
  实际使用面、str `_` 兜底的类型名接收者形态）；
  ②运行期实证：P419_TRACE/AUTO_DEBUG 临时打印于内联区入口
  （6904/7220），跑 tv 语料 + 探针 + sys-monitor 冒烟，记录哪些臂
  真实命中；③结论矩阵（臂×可达性×风险级，含接收者堆份额孤儿列）
  写入 §9。
- **可达臂收口**（T-02）：List 臂容器写按元素类型 retain/transfer
  （604 shim_list_push 同款）；str/List 臂 raw pop 后按 DROP 纪律
  收尾——**统一复用 `rc_release_slot_range`** 覆盖弹出窗口（堆按
  影子、池按内容，与 CALL_NAT 死区 engine.rs:7980 同形态：记
  sp_before → 臂逻辑 → release(sp_after, sp_before)；注意各臂
  弹出形态不一：`0..=arg_count` 顺弹/`pop_nv` 后 receiver 单弹/
  恒等臂不弹，收尾窗口按臂实际 sp 轨迹计）。每臂改后即跑
  `cargo tv` 分层回归。
- **不可达臂钉死**（T-05）：金样测试按**代码逻辑**直接构造分发态
  （如绕过 registry 构造 resolve miss 的调用形态，或直接以
  AutoVM::run_one_instruction 喂字节码序列）钉死当前行为——测试
  断言的语义=现行为，并加注「registry 覆盖变化会使本测试路由
  改变」的守护注释；内联 "push" 不 retain 形态**静态已判不可达**
  （auto.list.push 恒命中 shim），测试钉死 + KNOWN-DEBT 登记移除
  候选 + 修正其头注释与实现脱节（复核新发现②，删除与否留用户
  裁定，本计划不删代码）。

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

- **T-01 [x] 可达性定罪**：静态差集（§4 预答）+ AUTO_DEBUG_DISPATCH
  临时探针跑 tv 全量（3650 全绿带探针）+ probe_attribution UI 冒烟
  （零新增命中、六 verdict delta=0）→ 结论矩阵落 §9（臂×可达性×
  风险，含接收者堆份额孤儿列）。探针已移除。
- **T-02 [x] 可达臂结算收口**：内联 str 消费臂 20 站点 + List
  count|len/last/未知兜底 3 站点（合计 23）弹毕压结果前
  rc_release_slot_range 结算弹出窗口（spec_sp_entry 顶界捕获）。
  验证：AC-04 专项 red→green 双向；tv 3654/3654 零新增红。
- **T-03 [x] resolve 路径死区结算（设计执行期修正）**：原设计「shim 内
  补 pool_release」在 CALL_NAT 路径与死区构成**双释放**（underflow
  +N/调用，soak 实证）——池份额无影子可清，shim 内释放无法路径无关。
  修正为 CALL_SPEC resolve 分支补 CALL_NAT 同款死区
  （rc_release_slot_range(sp_after, sp_before)，engine.rs，单点镜像），
  池份额结算归属死区、shim 不参与（堆 stake 才走 shim take_stake 转移）。
  验证：AC-03 专项 red→green 双向（stash 撤修复=50 孤儿红）；tv/tf 全绿。
- **T-04 [x] pool_soak 字符串列表 churn 相位**：plan510_pool_tests.rs
  载体增 typed push 相位；AC-03 专项测试（索引接收者形态，trace
  实证 50 CALL_SPEC）；harness 收尾补 reap_all（dying 宽限队列
  静止点收割——ListData<i32> 负哨兵容器份额终态断言失真修复）。
  验证：pool_soak_churn_short 绿（+1 相位）。
- **T-05 [x] 不可达臂金样**：plan608_dispatch_golden_tests.rs——
  路由守护（List 13 名 + str 13 名 resolve 命中+shim 绑定断言，
  registry 覆盖回退→先红）+ 未知方法兜底行为钉死（None 返回/栈
  平衡/池配平）；内联 push 臂注释与实现脱节修正（复核新发现②）；
  KD-VM5 移除候选入 KNOWN-DEBT（删除留用户裁定）。验证：2 测试绿，
  注册于日常档（--lib）。
- **T-06 [x] 回归门**：cargo tv 3654/3654、cargo tf 3510/3510
  （含 1M churn）、probe_attribution（ui-iced）全绿——零红
  （AC-06 所列 charts_gallery 预存红已不在，本轮全绿）。注：
  ffi_dual_019 一次并行抖动（串行+复跑双绿，与 599 复审记录的
  竞态模式同族，非本计划改动面）。
- **T-07 [x] SPEC 回写**：SD-01（执行点清单增 CALL_SPEC 分发区①②③）
  + SD-02（池份额口径：容器份额独立保留、结算归属死区单点）落
  vm/overview.md §RC 生命周期协议；KD-VM5（移除候选）/KD-VM6（销账）
  + 域外观察（opaque_native 分支无结算候选）入 KNOWN-DEBT 608 续节。
  随 worktree 分支待 merge。

## 9. 复审记录

### T-01 定罪矩阵（2026-09-11，静态差集 §4 预答 + AUTO_DEBUG_DISPATCH
### 运行期实证：`cargo tv` 3650 全绿带探针跑批 + 604 探针
### probe_attribution UI 冒烟零新增命中、六 verdict 全 delta=0）

| 臂 | resolve | tv 运行期 | 结论 | 结算风险（可达臂） |
|---|---|---|---|---|
| str: as_bytes | miss | ✓×1 | **可达** | 接收者池份额孤儿/call |
| str: upper/lower 族 | upper/to_upper/to_lower hit；to_uppercase/lower/to_lowercase miss | — | to_uppercase 等静态可达（无语料命中） | 同上（低频） |
| str: chars/graphemes | miss | chars ✓×1 | **可达** | 接收者池孤儿 |
| str: split/is_empty/starts_with/ends_with/contains/len/trim/replace | hit | — | 不可达（shim 接管） | — |
| str: trimEnd/trim_end | miss | ✓×3 | **可达** | 接收者池孤儿 |
| str: includes | miss | ✓×1 | **可达** | 接收者+实参池孤儿 |
| str: indexOf 族 | miss | ✓×2 | **可达** | 接收者+实参池孤儿 |
| str: substring 族 | miss | ✓×1 | **可达** | 接收者池孤儿（实参 int） |
| str: char_code_at/charCodeAt | miss | ✓×2 | **可达** | 接收者池孤儿 |
| str: to_string/to_str/clone | miss | ✓×2 | 可达（恒等臂，接收者留栈） | 无孤儿，免收口 |
| str: to_int/to_float | miss（parse_int/parse_float hit） | — | 静态可达 | 接收者池孤儿（低频） |
| str: `_` 未知兜底 | — | ✓×3（bar/foo/parse） | **可达** | 接收者+字符串实参池孤儿 |
| List: count（count\|len 臂） | count miss / len hit | ✓×1 | **可达** | 接收者堆份额孤儿（影子在、裸弹不清） |
| List: last | miss | —（tv 语料无命中；calculator 历史用例） | 静态可达 | 接收者堆孤儿 |
| List: push | hit（auto.list.push 恒命中 shim） | — | **不可达** | 路由若变=UAF 面 → T-05 钉死+KNOWN-DEBT |
| List: get/remove/pop/insert/sort/sort_by/reverse | hit | — | 不可达 | — |
| List: dedup（sort\|dedup\|reverse 臂） | dedup miss | — | 静态可达 | 该臂仅弹实参、接收者留栈——无孤儿，免收口 |
| List: sort_by_key（sort_by\|sort_by_key 臂） | sort_by_key miss | ✓×1 | **可达** | 同上仅弹实参（闭包 int）——无孤儿，免收口 |
| List: 恒等名单/`_` 未知兜底 | 部分名 miss | par_iter ✓×2、frobnicate ✓×1 | **可达** | 未知兜底弹接收者+实参 → 堆孤儿；恒等名单元孤儿 |
| （str.into_iter/filter_map 落 str `_` 兜底而非 List 恒等臂） | miss | ✓×2 | 证实接收者类型决定臂归属 | 已含 str `_` 行 |

T-02 收口范围（消费接收者/实参的可达臂）：str 全部消费型臂
（as_bytes、upper/lower 族、chars、graphemes、trimEnd、includes、
indexOf 族、substring 族、char_code_at、to_int/to_uint/to_float、
`_` 兜底）+ List count/last/未知兜底。恒等臂（to_string、dedup、
sort_by_key、恒等名单）接收者留栈免收口。
### work 记录（2026-09-11）

stage: work | PLAN-608 | plan_revision v1.1（work 前设计复核版） |
outcome: pass | code_commit: plan-608-dev 23b16d870
（worktree D:/autostack/.wt/lang-608/auto-lang，基线 master beb9b57e2；
依赖 auto-down 组内 detached 工作树 1557a39） | task_ids:
T-01..T-07 全 | evidence: tv 3654/3654 + tf 3510/3510 + probe 六
verdict 0 + AC-03/04 专项 red→green 双向（stash 撤修复各 50 孤儿红）
| blockers: 无 | next: review（/auto-plan:review）

**执行期设计修正（T-03）**：shim 内池释放方案被 CALL_NAT 死区双
释放否证（underflow 800/800 拍实证）——修正为 resolve 分支单点补
CALL_NAT 同款死区；池份额结算归属死区、shim 不参与。该修正同时
覆盖全部 resolve 家族 shim（不只 list_push），语义与 CALL_NAT
完全同构。

**执行期新发现**：①typed `.push` 静态编译恒 CALL_NAT（is_native
按方法名命中，接收者类型无关）——CALL_SPEC→resolve 形态需索引
接收者（arr[0].push，trace 实证 50 CALL_SPEC）触发，AC-03 专项
测试据此构造；②pool_health 不含 reap_all，dying 宽限队列中
ListData<i32> 负哨兵容器的池份额残留在终态读数（harness 收尾补
reap_all 修复，pool_health 本体不动——autovm_persistent 有
mid-session 调用者）；③cargo fmt -p 全仓重排事故（499 文件）已
完全回滚重放，最终 diff 321 行/6 文件+新测试文件，零 fmt 噪音。

**Q-02 收口**：AC-04 维持 live_shares==0 硬口径（达成），无需
收窄。**Q-01 维持**：死码删除留用户裁定（KD-VM5 在案）。
### review 记录（2026-09-11，/auto-plan:review）

stage: review | PLAN-608 | plan_revision v1.1 | outcome: **pass** |
reviewed_commit: 23b16d87056ab3e6dbbb939fcffcdb92cc33fa5b |
base_commit: beb9b57e2 | dependency: auto-down detached 1557a39（组内） |
spec_inputs: docs/specs/auto-lang/vm/overview.md §RC 生命周期协议
（plan-604 版基线）+ Plan 510 池记账节 | acceptance_results:
AC-01 pass / AC-02 pass / AC-03 pass / AC-04 pass / AC-05 pass /
AC-06 pass | findings: F-1/F-2/F-3 全 info 级（见下） | evidence:
本复审与实施同会话（非独立上下文）——按技能要求从工件重建：全量
diff 逐 hunk 审（git show 23b16d870：resolve 死区/23 内联站点/
金样/测试/规范六文件 321 行）、双释放与结果槽重叠等 11 类危害
形态逐一静态推演（含 pop_arg 清内容/take_stake 清影子/StakeGuard
配对的既有防双释放机制核对）、AC 验证全数复现重跑（probe_ 46/46、
pool_settles×2、pool_soak、plan608 金样×2、cargo tv 3654/3654、
cargo tf 3510/3510）| next: merge（/auto-plan:merge）

**AC→任务→证据映射**：AC-01→T-01（§9 矩阵 inspection，静态差集
+AUTO_DEBUG_DISPATCH tv 全量 trace 计数）；AC-02→T-02（probe_
46/46 重跑，六 verdict delta=0）；AC-03→T-03/T-04
（pool_settles_callspec_list_push_str——索引接收者形态 trace
实证 50 CALL_SPEC，red→green 双向）；AC-04→T-02
（pool_settles_callspec_str_method_recv，同上双向）；AC-05→T-05
（plan608_dispatch_golden_tests 2/2：路由守护 List 13 名+str 13 名
resolve+shim 绑定断言；未知方法兜底 None/栈平衡/池配平钉死；
KD-VM5 移除候选在案）；AC-06→T-06（tv/tf 全绿零红——AC-06 所列
charts_gallery 预存红已不在当前基线）。

**Findings**：
- **F-1（info·已闭合）**：执行期 cargo fmt -p 全仓重排（499 文件）
  ——已完全回滚重放；复审 diff 审计确认最终 commit 仅 6 文件 321 行
  意图内改动，零 fmt 噪音、零语义漂移（tv/tf 复跑双绿佐证）。
- **F-2（info·债候选·范围外）**：List `_` 恒等分支（fold/collect/rev
  等未注册名）raw pop 实参不结算——T-01 矩阵裁「恒等臂免收口」依据
  为实参=闭包 int，但 fold(init, f) 的 init 可为字符串/引用（孤儿向
  泄漏，非 UAF）；tv 运行期仅 par_iter（0 实参）命中，无现实语料
  佐证。不在任何 AC 范围=非本计划缺陷；与 KNOWN-DEBT 域外观察同族
  登记，后续计划候选。
- **F-3（info·在案）**：CALL_SPEC opaque_native 分支手动弹出无结算
  ——已在 KNOWN-DEBT 608 续节登记为域外观察候选，非本计划范围。

**复审注记**：①执行期 T-03 设计修正（shim 内池释放→resolve 分支
死区）已复核其正确性论证：与 CALL_NAT 死区结构同构、结果槽落窗口
底不重叠、pop_arg 清内容/take_stake 清影子的既有防双释放机制在
resolve 路径同样成立——修正后语义强于原计划设计（覆盖全部 resolve
家族 shim），属计划 §5.2「修 shim + 可达内联臂」目标等价实现，
非范围收缩；②master 已被并行会话推进至 1c9926050（609/610 在途），
merge 时计划文件台账（主检出未提交改动）与 worktree 分支需按
merge 技能收口。



- 2026-09-11 draft v1（auto-plan:new）：PLAN-608 立案。stage: new，
  outcome: pass，next: work。证据基础=PLAN-604 执行期复核（内联
  "push" 臂裸存 2026-09-11）+ 604 收据候选① + 池份额缺口分析
  （shim_list_push elem_stake 对字符串恒 0、resolve 路径无死区）。
  用户指令：不可达路径以代码逻辑测试钉死守护。
- 2026-09-11 work 前设计复核（用户指令「重新分析设计后再实施」）：
  全部断言对当前代码复验成立（engine.rs:6904/7220 内联区、
  native.rs:2190 shim_list_push 四分支堆 stake 已结算而池份额
  全缺、virt_memory.rs:255 池不入影子、LOAD_LOCAL rc_push 池
  retain、内联 "push" 臂裸存+注释与实现脱节）。新增三件：
  ①静态差集预答完成（见 §4——T-01① 静态部分免做，运行期 trace
  佐证保留）；②确认结算原语 rc_release_slot_range 可复用（T-02
  统一收尾通道）；③新发现可达 List 臂接收者堆份额孤儿面（count/
  last/dedup/_ 兜底，每调用 heap +1，T-01 矩阵加列、T-02 收口）。
  计划文件同步修正：pool_soak 实际位于 tests/plan510_pool_tests.rs
  （非 tests_string_pool.rs）。结论：设计成立、无需 re-plan，
  按 T-01..T-07 原序进入 work。


### merge 收据（2026-09-11，PLAN-608:r1.1）

stage: merge | PLAN-608 | plan_revision v1.1 | outcome: pass |
delivery_commit: merge 2eacabcb5（reviewed_commit 23b16d870 为其祖先，
ANCESTRY-OK 实证；master 并行推进 609/610 全为文档提交，KNOWN-DEBT
两侧异区自动合并）| canonical specs: docs/specs/auto-lang/vm/overview.md
§RC 生命周期协议（SD-01/SD-02，随 delivery 落地；落地后主检出定向
冒烟 pool_settles 2/2 + plan608 金样 2/2 + pool_soak 1/1 绿）|
ledger: docs/specs/auto-lang/vm/plans.md 608 行（commit c783f443e）+
INDEX 重生幂等无变化 + .autoos/specs.json P608-1..6 六 section 原子
发布回读验证 | archive: docs/plans/archive/608-vm-dispatch-settlement-pool.md
（git mv + status: archived）| completion_kind: delivered | cleaned: 见
后续 cleaned 行 | 注：主检出存在并行会话脏文件（iced/renderer.rs、
iced/snapshot.rs、website/v05 等 + 未跟踪 607）——非本 merge 范围，
未纳入未丢弃，原样保留。

## 10. 待澄清事项

- **Q-01**：定罪为死码的内联臂（含不 retain 的 "push" 形态）是否
  直接删除——本计划倾向只钉行为+登记候选（删除面广、涉及 Plan
  403/053 历史成因），删除决策留用户。
- **Q-02**：str 内联臂的池孤儿量级在真实应用尚未实测（dedup 使
  重复串共享条目、只影响可回收性不影响条目数）——T-01 跑批时顺带
  量化，若量级可忽略则 AC-04 收窄为「有界」口径。
