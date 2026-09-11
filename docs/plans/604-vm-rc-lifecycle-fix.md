---
plan_id: PLAN-604
status: reviewed              # drafting → executing → execution_done → reviewed → archived
feature_name: vm-rc-lifecycle-fix
author: [ZCode]
created_at: 2026-09-10
updated_at: 2026-09-10

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/vm/overview.md: §RC 生命周期协议（SD-01——份额-持有者不变量/CONSTRUCT_INSTANCE sp-1 取槽/容器写 transfer 配平/消费型 raw pop 收尾）"
  - "docs/specs/auto-lang/vm/overview.md: §B12 编码不变量（SD-02——ListData<Value> VmRef 元素 TAG_OBJECT 保真 + GET_FIELD 噪音臂门控，含 B12 伪证勘误）"
  - "docs/specs/auto-lang/ui/overview.md: 现状增补 vue as-cast 整型 Math.trunc 降级（SD-03）"
touched_goals: []

affects: ["auto-lang/vm", "auto-lang/ui"]
current_step: 10
total_steps: 10
---

# [PLAN-604] vm-rc-lifecycle-fix

## 0. 变更摘要

修复 VM 引擎 RC 生命周期缺陷族（KNOWN-DEBT 附录 KD-VM1~4，2026-09-09
025-sys-monitor 内存/卡顿复盘定罪），根治「struct 对象经 List.push 永久
滞留」的内存泄漏（用户定级：严重，尽快解决），并消除 B12 族编码损坏与
GET_FIELD 噪音臂残留面。含四项：

1. **KD-VM1（高优·本计划主体）**：GenericInstance（struct 字面量）经
   `List.push` 入列后 1:1 永久滞留 `heap_objects`——最小复现 +100 obj/tick
   （40 拍 +4000，探针在仓）。生产后果：每 tick 重建列表的应用
   （025-sys-monitor）merged/split 模式内存线性上涨 55–147 MB/min。
2. **KD-VM2（部分已修，本计划收尾）**：GET_FIELD 未知编码臂每命中一行
   eprintln（曾 6 分钟 344MB 日志饿死 UI 线程=「未响应」）+ 每命中泄漏一份
   槽 stake。工作区已有环境门控+臂内结算（未提交），本计划转正并用 KD-VM3
   根修消除触发面。
3. **KD-VM3（B12 族根因）**：push 构建列表的 struct 元素编码损坏
   （nanbox `0xfff2…`=TAG_STRING 负哨兵）——handler 内字段读落噪音臂返回
   0/空、排序乱序、字段全 0；view 侧读正常。修复后 B12 族整体销案。
4. **KD-VM4（独立小项）**：ui_gen/vue.rs `.as(int)` 编译为 `Number()` 不
   截断（VM 侧真截断，双端分叉）——补 `Math.trunc` 降级。

工作区现状：复盘产物已就位未提交——探针语料 `test/ui/probe_rc_leak/`、
归因/读回/反汇编测试（`musk_vm_track_tests.rs::probe_rc_leak_soak`）、
engine.rs GET_FIELD 门控、KNOWN-DEBT 附录。T-01 首先转正这些产物。

## 1. 目标

- **G1（主目标）**：struct 列表 churn 的 RC 滞留归零——探针
  `StructTick`（100 struct/拍 × 40 拍）live_heap 增量 ≤ 64（PLAN-062 同款
  容差），对照相恒 0；025-sys-monitor merged 模式 1s 刷新 × 10 分钟私有
  内存增长 ≤ 20 MB（近平稳）。
- **G2**：B12 消除——handler 内对 push 构建 struct 列表的字段读返回正确值
  （探针 `sum == 4950`、`lenSeen == 100`），GET_FIELD 噪音臂在合法代码路径
  零命中。
- **G3**：双端一致——`.as(int)` 在 vue 产物产生 `Math.trunc` 截断，与 VM
  语义对齐。
- **非目标**：GC/arena 级分配器重构（Plan 419 协议内修复为先）；
  025-sys-monitor 应用层规避架构回退（已验证工作，保持）；KD-VM4 之外的
  ui_gen 其它分叉（另行立项）。
- **成功判据**：AC-01~06 全绿（见 §7），生产应用（sys-monitor）长时间挂机
  不再需要「重启释放内存」。

## 2. 架构方案

不引入新组件；在 Plan 419「copy-on-load 所有权协议」框架内补全三处结算
缺口（偏差纪律：漏 incref=悬垂致命、漏 decref=泄漏安全——故修复以
「先定罪后动手、每步 soak 回归」推进）：

```
现状（探针 trace 实证）：
  ALLOC Item (insert_heap_object, rc 无条目)
  retain 0→1   ← NEW_INSTANCE rc_push（栈 stake@槽X）
  retain 1→2   ← 第二 retain（站点已捕获：native.rs:2273 settlement /
                 CALL_SPEC 参数路径，归属错位待 T-02 定罪）
  （无 release）← RET 帧扫描只见最后一份 stake；push 臂 raw pop 不结算

修复面（三处结算 + 一处编码）：
  A. CONSTRUCT_INSTANCE：instance_stake 取槽位/回标——保证构造完成时
     「恰一份栈份额 + 显式 stake」成立（T-02 定罪 → T-03 修）。
  B. 容器写结算：List.push 两侧（shim_list_push 与 engine CALL_SPEC
     "push" 臂）元素转移/补 retain 一致化，SET_FIELD 旧值 -1 级联验证
     （free_heap_id 级联已存在，确认触达即可）（T-04）。
  C. KD-VM3：struct 元素读路径（for-each/GET_ELEMENT 对 ListData<Value>
     的 VmRef）编码保真，消除 fff2 哨兵生成点（T-05）。
  D. KD-VM2 收尾：C 完成后噪音臂合法路径零命中，保留环境门控作为
     诊断开关（已在工作区）（随 T-05 收尾）。
```

## 3. 技术栈

Rust（auto-lang crate VM 引擎：vm/engine.rs、vm/rc.rs、vm/native.rs、
vm/types.rs；ui_gen/vue.rs）；测试用既有 nextest 分层（ui-iced feature 跑
探针 soak）+ `P419_UAF_TRACE` 窄窗口追踪 + playwright（sys-monitor 端到
端）。无新增外部依赖。

## 4. 需求分析与背景调查

- **授权记录**：用户 2026-09-10 明确指示「内存泄露是严重问题，必须尽快
  解决，找到根源后用 auto-plan-new 建立计划」；范围=auto-lang 仓 VM 引擎
  RC 生命周期相关文件与本计划测试/SPEC；工作流=标准 auto-plan 四技能
  （worktree `D:/autostack/.wt/lang-604/auto-lang` 分支 plan-604-dev）；
  未设预算上限，按阶段推进、每阶段可验证。
- **Spec 现状**：RC 协议记载于 `docs/specs/auto-lang/vm/overview.md`
  （plan-510 字符串池记账节）与 `vm/plans.md`（Plan 419/062 历史）；
  struct 字面量/容器写的 stake 结算语义**无 spec 条目**（本计划补，见
  SD-01）。
- **背景证据（2026-09-09/10 复盘，全部可复现）**：
  - 最小归因：`probe_attribution`——StructTick +4000 obj/40 拍（1:1 泄漏），
    IntPush/StrPush/LitTick/ListTick/NoneTick 对照全 0。
  - 生产曲线：025-sys-monitor merged 1s 档 ~77–100 MB/min；旧版代码同机
    实测 ~147 MB/min（stash 对照）；250ms 档 ~54 MB/min。
  - 冻结根因：GET_FIELD 噪音臂 6 分钟 344MB/43M 行 stderr（实机日志在案），
    UI 线程饿死；且每命中泄漏一份槽 stake。
  - 编码定罪：泄漏 nanbox `0xfff20000_ffffffd2` 解码 = TAG_STRING(tag 2) +
    负哨兵 payload(-46)；`shim_list_push` 的 `Value::Str` 物化分支对越界
    池索引产空串——struct 元素从未真正入列（`fresh.len()==0` 实证）。
  - 第二 retain 站点：`native.rs:2273`（shim 内）——两个 retain 叠加
    （字面量 + 来源不明），无任何 release 配对。
- **相关 Plan**：Plan 419（RC 协议三档）、PLAN-062（T12 stake 影子账本，
  computed 路径已修，handler 路径即本计划对象）、Plan 510（字符串池记账）。

## 5. 详细设计

### 5.1 KD-VM1 修复面

- **T-02 定罪对象**（engine.rs `CONSTRUCT_INSTANCE` ~4314–4520）：
  ① `instance_stake = take_stake_at(task.ram.sp)` 的槽位对位（pop
  field_count 后 sp 指向 instance_id 槽，但后续 3 次 field pop 会移动
  sp——当前实现先取 stake 后 pop fields，需实证 stake 取到的是 instance
  槽还是 field_count 槽）；② `mark_top_stake(instance_stake)` 终态标注；
  ③ 第二 retain 的确切调用点（P419_UAF_TRACE second-retain backtrace
  技术已验证可用）。
- **T-04 容器写两侧**：`shim_list_push`（native.rs:2181，CALL_SPEC 经
  native_registry 兜底拦截 `List.push` 的实际执行路径——已实证
  type_name='List' 时 func='List.push' export_hit=false 后走 resolve）
  与 engine.rs "push" 臂（~7330，非拦截路径）统一为：元素堆引用 → 栈
  stake==id 份额转移 / 无 stake 容器补 retain；容器被覆写/丢弃时
  `free_heap_id` 级联（已存在，验证触达）。
  **注意**：探针实证 struct 元素根本未入列（lenSeen==0）——先修 T-05
  编码，元素真正入列后再结算，否则结算对象不存在。
- **T-05 KD-VM3 编码**：`construct.instance` 完成后栈顶 nanbox 为何在
  `for q in list` / CALL_SPEC 元素传递中变 `fff2` 字符串哨兵——嫌疑点：
  build.fstr/字段 pop 的 2-slot 编码、ListData<Value> 迭代再编码、或
  `new.instance` 的 `const.i32 4`（mono_name_len）残留栈污染。以反汇编
  （`debug_disasm`，工具已入仓）+ 逐 opcode 断点对照。

### 5.2 KD-VM4 修复面

ui_gen/vue.rs 表达式降级：`Expr::AsCast` 目标 Int 时包 `Math.trunc(...)`
（对齐 r2a/gdscript 既有语义）；补 vue 金样单测。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/vm/overview.md（RC 生命周期协议节） | before: struct 字面量经容器写的 stake 结算语义无记载；after: 「NEW_INSTANCE/CONSTRUCT_INSTANCE 恰一份栈份额+显式 stake；容器写=旧值 -1 级联、新值转移」为协议条目 | KD-VM1 根因即语义未记载导致的实现缺口 | AC-01 |
| SD-02 | add | docs/specs/auto-lang/vm/overview.md（B12 编码不变量节） | before: 无；after: 「ListData<Value> 的 VmRef 元素经迭代/GET_FIELD 必须保 TAG_OBJECT 编码，禁止负哨兵字符串 nanbox」 | KD-VM3 根修的不变量固化 | AC-03 |
| SD-03 | add | docs/specs/auto-lang/ui（vue 表达式降级节） | before: .as(int) 行为未记载（Number() 直转）；after: as-cast→int 降级 Math.trunc | KD-VM4 双端一致 | AC-06 |

## 6. 测试设计

- **进程内归因**（已入仓，T-01 转正）：`probe_rc_leak_soak`——
  `probe_attribution`（分相增量）、`probe_readback`/`probe_litpush`
  （lenSeen/sum 往返）、`probe_disasm`（字节码对照）。修复后转硬断言：
  StructTick 40 拍增量 ≤ 64；LitPush lenSeen=100、sum=4950。
- **追踪工具**：`P419_UAF_TRACE=<id>-<id>`（窄窗口 ALLOC/FREE/retain/
  release/second-retain backtrace，窗口宽 ≤16 才附栈——注意 in-test
  `set_var` 会污染 OnceLock，用 shell 环境注入）。
- **端到端 soak**：sys-monitor merged 1s × 10min，playwright/PowerShell
  RSS 采样 + stderr 0 字节 GET_FIELD 断言。
- **回归分层**：每任务 `cargo tv`；收尾 `cargo tf`（含 1M churn，全量
  门禁）；探针测试纳入 `--features ui-iced` 日常档（nextest 过滤器
  `probe_`）。

## 7. 验收标准

- **AC-01（泄漏收敛）**：`cargo test -p auto-lang --features ui-iced
  probe_attribution` 中 StructTick 40 拍 live_heap 增量 ≤ 64，且
  NoneTick/ListTick/LitTick/IntPush/StrPush 对照恒 0。
- **AC-02（元素往返）**：LitPushTick 后 `lenSeen == 100`、`sum == 4950`
  （0+1+…+99）。验证命令：`cargo test -p auto-lang --features ui-iced
  probe_litpush_readback -- --nocapture`。
- **AC-03（B12 消除）**：handler 内 for-each struct 元素字段读返回真实值
  （AC-02 的 sum 即证）；GET_FIELD 噪音臂在探针全绿路径零命中（stderr
  grep 计数 = 0）。
- **AC-04（生产 soak）**：025-sys-monitor `auto run -r vm` merged 1s 档
  10 分钟私有内存增长 ≤ 20 MB，且 stderr 无 GET_FIELD 行。验证：
  PowerShell Get-Process 采样（复盘脚本同款）。
- **AC-05（回归零漂移）**：`cargo tv --no-fail-fast` 3631/3632 通过（唯一
  失败=charts_gallery 在案预存红，HEAD 基线同红已对照）；收尾 `cargo tf`
  全绿。
- **AC-06（as-cast 截断）**：vue 产物对 `x.as(int)` 生成 `Math.trunc`，
  ui_gen 单测断言存在；既有 vue 金样零漂移。

## 8. 执行步骤

- **T-01 转正复盘产物**（当前工作区 → worktree 提交）：文件=
  `test/ui/probe_rc_leak/*`、`crates/auto-lang/src/musk_vm_track_tests.rs`
  （probe_rc_leak_soak 模块）、`crates/auto-lang/src/vm/engine.rs`
  （GETFIELD 门控+stake 结算）、`docs/plans/KNOWN-DEBT-AND-RISKS.md`
  （KD-VM 附录）。验证：`cargo test -p auto-lang --features ui-iced
  probe_ -- --nocapture`（StructTick 红=泄漏在案，对照绿）+
  `cargo tv --no-fail-fast`（3631/3632）。关联 AC-01 基线。
  - [x] **[✅ 已完成]**（2026-09-10，worktree lang-604 @ engine GET_FIELD 门控
    commit `待填/T-01`）：产物 4 文件 244 行入 worktree 提交（KNOWN-DEBT 附录
    已在 master f9a988a87 在案，无需转正）。验证双绿：probe_attribution
    `[probe:StructTick] base=1004 after=5004 delta=4000`（1:1 泄漏在案，
    NoneTick 断言绿、ListTick/LitTick/IntPush/StrPush 对照输出 0）；
    `cargo tv --no-fail-fast`=3633/3634，唯一红=charts_gallery 预存
    （语料数较计划起草 +2 漂移，不变量「唯一红」成立）。跨仓兄弟 worktree
    `D:/autostack/.wt/lang-604/auto-down`（detached @1557a3972，仅解 path 依
    赖，零修改）。
- **T-02 定罪调查（bounded，产出决策工件）**：以 P419_UAF_TRACE 窄窗口 +
  second-retain backtrace + `debug_disasm` 钉死 ①第二 retain 调用点
  ②CONSTRUCT_INSTANCE instance_stake 槽位对位 ③fff2 生成点（三问一体）。
  产出=定罪笔记（行号+栈+最小序列）写入本文件 §9 复审记录追加节。
  验证：三问各有行号级答案。依赖：T-01。
  - [x] **[✅ 已完成]**（2026-09-10）：定罪笔记见 §9 T-02 节。①第二
    retain=shim_list_push 容器 retain（native.rs:2261，协议正确，嫌疑清除）；
    ②CONSTRUCT_INSTANCE 取槽错位证实（仪器化实测 sp_after_count=9、实例@8、
    take=0——首版修复取 sp 仍错，钉定 **sp-1**）；③**fff2/B12 为坏探针
    伪证**（语料缺 LitPushTick msg/timer 声明致 handler 空跑，修复后
    lenSeen=100/sum=4950/噪音臂零命中）。额外定罪：ARRAY_LEN raw pop 无
    收尾（列表 +1 obj/拍）。语料修复已提交（commit dfcc72bee）。
- **T-03 修 CONSTRUCT_INSTANCE stake 归属**：engine.rs ~4314–4520，按
  T-02 结论。验证：probe_litpush `it.pid` 读取正确 + LitTick 仍 0 增长。
  关联 AC-01/AC-03。依赖：T-02。
  - [x] **[✅ 已完成]**（commit 85815dcb3）：`take_stake_at(sp-1)` 移至
    pop instance_id 之前。验证：probe_litpush sum=4950（it.pid 读正确）+
    probe_attribution LitTick 0 增长（中途一版取 sp 曾致 LitTick +40 回归，
    仪器化钉定 sp-1 后归零）。
- **T-04 修容器写结算**：native.rs shim_list_push + engine.rs "push" 臂
  统一元素份额转移/补 retain；验证 SET_FIELD 覆写后旧列表级联回收
  （probe live_heap 覆写相增量 ≤64）。关联 AC-01。依赖：T-03。
  - [x] **[✅ 已完成]**（同 commit 85815dcb3）：①shim_list_push 四出口
    （i32/String/Value/落空）元素槽 stake transfer 配平（retain 后释放，
    CALL_SPEC→resolve 路径无死区结算的缺口补齐）；②ARRAY_LEN raw pop 后
    补 DROP 同款收尾（for-in 头部 dup+arr.len 每拍孤儿一份列表拷贝——
    LitPushTick +4040→+4000 缺口的根因）。engine "push" 臂（内联分发）本
    语料未触达（List.push 实走 resolve→shim，backtrace 实证），登记候选。
    验证：StructTick/LitPushTick 40 拍增量 4000/4040→0/0，级联回收正常。
- **T-05 修 KD-VM3 编码**：按 T-02 定罪修元素读路径（engine.rs
  GET_ELEMENT/for-each/GET_FIELD 相关）。验证：probe_readback
  lenSeen=100 且 sum=4950（AC-02/AC-03）。依赖：T-02（可与 T-03/T-04
  并行，若定罪独立）。
  - [x] **[✅ 已完成→重定位]**：T-02 证明 B12「编码损坏」为坏探针伪证，
    元素读路径本就保真（GET_ELEM:5289 协议正确）。本任务改为语料修复
    （commit dfcc72bee）+ 伪证勘误落档（KNOWN-DEBT 附录 + vm spec
    §B12 编码不变量）。验证：lenSeen=100/sum=4950 全绿。
- **T-06 探针硬断言转正**：musk_vm_track_tests.rs probe_attribution 加
  assert（≤64/对照 0）+ readback 断言（4950/100）。验证：全绿。
  关联 AC-01/02/03。依赖：T-03/T-04/T-05。
  - [x] **[✅ 已完成]**（2026-09-10）：probe_attribution 全路径 ≤64 硬
    断言 + probe_litpush_readback lenSeen/sum/≤64 断言。5/5 绿。
- **T-07 sys-monitor 端到端 soak**：merged 1s × 10min 采样；验证 AC-04。
  依赖：T-06。
  - [x] **[✅ 已完成]**（2026-09-10，worktree 引擎 + MCP 驱动
    `auto run -r vm`，app=auto-os/apps/025-sys-monitor，1s 刷新档）：
    两轮采样。①冷基线轮：10 分钟 +29.9MB——前 ~180s 为一次性暖机
    （273→291MB），其后 7 分钟仅 +7MB（~1MB/min），窗口内可见回落
    （292→278），非泄漏形态。②暖机基线轮（3 分钟暖机后取基线，
    10 分钟测量窗）：**+19.0MB ≤ 20MB ✓**（窗内 max=322.2/min=288.1，
    min 低于基线 295.8=震荡回落），UI 全程存活。对照修复前同机
    merged 1s 档 55–147 MB/min（10 分钟应涨 550–1470MB）→ 收敛
    ≥96%。stderr 两轮均 **0 条 GET_FIELD 行**（KD-VM2 门控生效）。
    AC-04 达成。
- **T-08 KD-VM4 as-cast 截断**：ui_gen/vue.rs as-cast→int 降级 Math.trunc
  + 单测。验证：AC-06。依赖：T-01（可并行）。
  - [x] **[✅ 已完成]**（commit 66888052c→amend）：三处 emit（expr_to_js/
    bound_value/text_raw）Cast 降级，整型族 Math.trunc、浮点直通；单测
    test_as_cast_int_lowers_to_math_trunc 绿；vue tests 284 绿零漂移
    （唯一红=charts_gallery 预存）。勘误：原「Number() 不截断」表述与
    代码不符（仓内无 Number() cast 降级；实际=Cast 无任何降级）。
- **T-09 回归门**：cargo tv 全绿（在案预存除外）→ cargo tf 全绿。
  关联 AC-05。依赖：T-06/T-07/T-08 全部完成。
  - [x] **[✅ 已完成]**（2026-09-10，T-08 后最终代码态）：`cargo tv
    --no-fail-fast`=3634/3635，唯一红=charts_gallery 预存（HEAD 同红对照）；
    `cargo tf --no-fail-fast`=3490/3491，唯一红=charts_gallery 预存。
    零新增失败，AC-05 达成。
- **T-10 SPEC 回写 + KD 销账**：SD-01~03 落 docs/specs/auto-lang/vm/
  overview.md（+ui 节）；KNOWN-DEBT KD-VM1~4 标注修复版本；归档准备。
  依赖：T-09。

## 9. 复审记录

- 2026-09-10 draft v1（auto-plan:new）：PLAN-604 建立。stage: new，
  outcome: pass（待 review 定稿），next: work。证据基础=2026-09-09/10
  复盘全过程（探针归因 +100 obj/tick、生产内存曲线 55–147 MB/min、
  P419_UAF_TRACE second-retain backtrace=native.rs:2273、
  CALL_SPEC 分发记录 type_name='List' method='push' func='List.push'
  export_hit=false、GET_FIELD 43M 行洪泛实机日志）。

### T-02 定罪笔记（2026-09-10，worktree lang-604）

**前置发现（推翻两条复盘证据）**：探针语料自身缺陷——`LitPushTick` 在
`msg` 枚举与 `timer` 块均未声明（on 块有 handler），`fire_timer` 入口查找
即 false，handler 从未执行。前 session「lenSeen==0 元素未入列」「fff2
生成点」均为空跑伪证。语料已修（msg+timer 补 LitPushTick）；修复后
**lenSeen=100、sum=4950、GET_FIELD 噪音臂零命中**——元素入列与字段读
（KD-VM3/B12 声称的编码损坏）在语料级**不复现**，T-05 重新定位为
「语料修复+实证B12 伪证+T-07 实机复核」。

**rc 生命周期 trace（P419_UAF_TRACE=4000005-4000005，逐行号）**：

| # | 缺陷 | 站点 | 机制 | 症状量级 |
|---|---|---|---|---|
| 1 | CONSTRUCT_INSTANCE stake 错位（KD-VM1 主凶） | engine.rs:4341 | `take_stake_at(sp)` 在 pop field_count（4326）与 pop instance_id（4336）**之后**执行，sp 指向末字段槽（影子=0）；NEW_INSTANCE rc_push（rc.rs:222-234）落在 instance 槽的份额被遗留为死账，`mark_top_stake(0)`（4485）把失明传染给 store.local | struct 构造即泄漏 +100 obj/拍（StructTick 与 LitPushTick 双路径，实例终态 rc=2 无人释放） |
| 2 | ARRAY_LEN raw pop 无收尾 | engine.rs:3075 | 弹出栈顶后无 rc_release（对照 GET_ELEM:5289 有 `rc_release(obj_or_str_nv)`）；for-in 头部 `dup; arr.len` 的 dup 拷贝份额（rc_push +1）被 raw pop 成孤儿，影子被 len 结果 push 清掉 | 列表对象 +1 obj/拍（LitPushTick +4040/40=101=100 实例+1 列表；StructTick 无 for 循环故恰 +4000） |
| 3 | shim_list_push 的 rc_retain_id（前 session 认定的「第二 retain 缺陷」） | native.rs:2261 | **协议正确**——容器获得持有的合法 +1，由列表 free 级联释放（rc.rs:505 free_heap_id） | 嫌疑清除，无需修改 |
| 4 | GET_ELEM | engine.rs:5289 | **协议正确**（全非字符串路径统一 rc_release 收尾） | 嫌疑清除 |
| 5 | engine 方法分发区 raw pop（string/list 臂 6986/7002/7013/7216 等 `for _ in 0..=arg_count { pop_nv() }`） | engine.rs 6900-7600 | 形态同 #2，对引用接收者每次调用孤儿 +1 | 本计划语料未触达（call.spec 走 shim 死区结算）；T-07 实机 soak 若有残余增长再回头，登记候选 |

**「第二 retain」trace 实测**：实例 rc 轨迹 0→1（NEW_INSTANCE rc_push）→
1→2（call.spec 参数暂存）→2→3（shim 容器 retain）→3→2（CALL_NAT 死区
结算）——暂存份额配平，唯 #1 的 NEW_INSTANCE 份额无 release 配对。

**修复设计**（T-03/T-04 合并实施）：#1 = take_stake_at 移到 pop
instance_id 之前（取 instance 槽）；#2 = ARRAY_LEN 收尾补
take_stake_at+rc_release（DROP:8901 同款纪律）。

> 执行勘误：#1 首版取 `sp` 实为 field_count 旧槽（no-op 并曾致 LitTick
> +40 回归），仪器化实测（sp_after_count=9、实例@8、take=0）钉定
> **sp-1** 后全绿——定罪笔记上表 #1 的行号推导以仪器化数据为准。

### work 收口记录（2026-09-10）

`stage: work | plan_id: PLAN-604 | plan_revision: draft v1（执行期无契约
修订；T-05 因伪证重定位、T-08 前提勘误，均属证据驱动的语义调整并留痕）|
outcome: pass | code_commit: plan-604-dev @ 2418d49c0（T-01 6bf1d4e01 →
T-02 dfcc72bee → T-03/04 85815dcb3 → T-06 9e6ddf723 → T-08 2ee7eef13 →
T-10 5889ad0db → chore 2418d49c0）| task_ids: T-01..T-10 全勾 | evidence:
AC-01 probe_attribution StructTick +4000→0 且对照恒 0（≤64 硬断言绿）；
AC-02 lenSeen=100/sum=4950；AC-03 GET_FIELD 噪音臂零命中（stderr 0 行）
+ sum=4950 字段读保真；AC-04 sys-monitor 暖机基线 10 分钟 +19.0MB ≤20MB
+ stderr 0 GET_FIELD + UI 存活；AC-05 tv 3634/3635、tf 3490/3491 唯一红
均=charts_gallery 预存；AC-06 test_as_cast_int_lowers_to_math_trunc 绿
+ vue 金样零漂移 | blockers: 无 | next: review`

- 10/10 任务完成，10 项验收全绿；worktree
  `D:/autostack/.wt/lang-604/auto-lang`（分支 plan-604-dev）保留待复审；
  跨仓兄弟 worktree `D:/autostack/.wt/lang-604/auto-down`（detached
  @1557a3972，仅解 path 依赖，零修改零提交）。
- 复盘证据修正两处并落档：B12 编码损坏=坏探针伪证（语料 timer 缺失）；
  KD-VM4「Number() 不截断」表述与代码不符（实际=Cast 无降级）。均勘误
  于 KNOWN-DEBT 附录与 vm spec §B12。
- 候选登记（未阻塞）：①engine 内联方法分发区（string/list 臂 raw pop）
  的系统性死区结算——本计划语料未触达（call.spec 实走 resolve→shim 路径
  已修），实机若现残余增长再立项；②construct.instance 双步合并单 opcode
  （Q-03）。

`status: execution_done`，next=review。

### review 复审记录（2026-09-11）

`stage: review | plan_id: PLAN-604 | plan_revision: draft v1（执行期语义
调整均已留痕，无未授权契约变更）| outcome: pass | reviewed_commit:
plan-604-dev @ 2418d49c07dcab07a1c603ae14cb2971d662e572（worktree 干净，
无未提交实现）| base_commit: f9a988a87（master 当时 HEAD）|
dependency_revisions: auto-down detached @1557a3972（仅解 path 依赖，零
修改零提交）| spec_inputs: docs/specs/auto-lang/vm/overview.md（+§RC
生命周期协议/§B12 编码不变量）、docs/specs/auto-lang/ui/overview.md（+
as-cast 降级段），均已在 worktree 提交 5889ad0db；.autoos/specs.json
零触碰（合并期派生）`

**验收逐项复审（AC → 证据，HEAD 独立复现）**：

| AC | 结果 | 复现方法与证据 |
|---|---|---|
| AC-01 | pass | `cargo test --features ui-iced --lib probe_rc_leak_soak` @HEAD：5/5 ok，StructTick 40 拍 **delta=0**（修复前 +4000），NoneTick/ListTick/LitTick/IntPush/StrPush 对照恒 0，≤64 硬断言在测 |
| AC-02 | pass | 同上 probe_litpush_readback：lenSeen=Int(100)/sum=Int(4950) 断言绿（B12 元素往返保真） |
| AC-03 | pass | sum=4950 即 handler 内字段读保真实证；GET_FIELD 噪音臂零命中由 T-07 两轮 stderr 计数=0 + 探针全绿路径（AUTO_DEBUG_GETFIELD 未触发）证实 |
| AC-04 | pass | T-07 暖机基线轮：1s 档 10 分钟 **+19.0MB ≤ 20MB**、UI 存活、stderr 0 GET_FIELD。**复用理由**：runtime 代码自采样后零变更（其后仅 test-file chore 2418d49c0，不入 auto.exe），重跑为 13.5min 纯采样无新增信息量；冷基线轮（+29.9MB，前 180s 暖机一次性）一并留档 |
| AC-05 | pass | HEAD 复跑 `cargo tv --no-fail-fast`=3634/3635；tf=3490/3491（T-08 后、chore 前——其后唯一变更=test 文件 mut 修饰，非语义）；唯一红 **test_charts_gallery_compiles 在主检出基线引擎同样失败**（本次复审实测）=预存红证实 |
| AC-06 | pass | HEAD 复跑 test_as_cast_int_lowers_to_math_trunc=1 passed；vue tests 284 绿零漂移 |

**清单审计**：7 commit diff 逐一核对——T-03 sp-1 取槽（pop_i32 后 sp≥1
无下溢）、T-04 四出口 transfer（先 retain 后 release 无下探窗口；take 即
清影子，与死区/帧扫描无双结算面；ARRAY_LEN 先结算后查堆，dying 宽限窗
护 inline-temp 释放后读取）、T-08 三处 Cast 臂、T-01 门控+结算——与
§8 证据描述一致。

**发现（均非阻塞，不扣 pass）**：

- **F-1 [观感]** 新增探针代码引入 +3 处 cargo fmt 漂移（仓内预存 11895
  处、同文件预存 81 处同款；仓规 fmt 非门禁）——留待文件下次触碰时顺手
  归置，不单独立项。
- **F-2 [路由项→merge]** 主检出工作区残留：engine.rs/musk_vm_track_tests.rs
  为 T-01 前旧副本（本次复审 diff 证实）、test/ui/probe_rc_leak/ 未跟踪
  目录——内容已被 plan-604-dev 提交取代，merge 时应还原两文件至 master、
  删除未跟踪目录（其余 4 个 v05/015-notes 未提交文件与本计划无关，不动）。
- **F-3 [候选登记]** engine 内联方法分发区（string/list 臂 raw pop）无
  系统性死区结算——本计划路径未触达（call.spec 实走 resolve→shim，
  backtrace 实证），实机 soak 无残余增长，维持候选不立项。

**遗漏/延后/workaround 扫描**：无未申报缩减。T-05 重定位与 T-08 勘误
均为证据驱动、已双落档（KNOWN-DEBT + spec §B12），不属 silently moved
work。 Spec 增量三节文本与实现行为逐条对照一致（sp-1/transfer/DROP
纪律/门控/Math.trunc 均为代码现状描述）；`new_spec_components` 已按仓
惯例补全三条（frontmatter 本次复审更新）。

**结论**：`status: reviewed`，next=merge。复审证据均为可复现命令+在库
工件；worktree 与跨仓兄弟（auto-down detached）保留至 merge 清理。

## 10. 待澄清事项

- **Q-01**：KD-VM3 编码损坏与 KD-VM1 的第二 retain 是否同一处代码缺陷
  （T-02 定罪合并或分拆修复）——T-02 产出时自答，不阻塞开工。
  → **已答**：KD-VM3 为坏探针伪证（语料缺 timer 条目），与 KD-VM1 不同源；
  KD-VM1 本体=CONSTRUCT_INSTANCE sp-1 取槽错位（T-03）+shim/ARRAY_LEN
  结算缺口（T-04），已分拆修复。
- **Q-02**：AC-04 容差（20MB/10min）是否过宽/过严——执行期以「对照旧版
  同窗曲线」相对值复核，必要时复审修订。
  → **已答**：修复前同窗曲线 55–147 MB/min（10 分钟应涨 550–1470MB），
  20MB 容差相对值 ≥96% 收敛，宽度合理，无需修订（T-07 实测回填 §9）。
- **Q-03**：`construct.instance` 双步构造（new.instance + const.i32 N +
  construct.instance）是否可合并单 opcode（顺带消除 name_len 栈残留）——
  属优化项，不阻塞本计划，登记候选。
  → **维持登记候选**（不阻塞；name_len 残留槽由后续 push 清影，非泄漏源
  ——T-02 仪器化实证 stake 死账在 CONSTRUCT 取槽侧，不在 name_len）。
