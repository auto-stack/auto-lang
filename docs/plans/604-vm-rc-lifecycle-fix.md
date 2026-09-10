---
plan_id: PLAN-604
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: vm-rc-lifecycle-fix
author: [ZCode]
created_at: 2026-09-10
updated_at: 2026-09-10

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: ["auto-lang/vm/overview.md#rc-生命周期协议"]
touched_goals: []

affects: ["auto-lang/vm", "auto-lang/ui"]
current_step: 0
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
- **T-02 定罪调查（bounded，产出决策工件）**：以 P419_UAF_TRACE 窄窗口 +
  second-retain backtrace + `debug_disasm` 钉死 ①第二 retain 调用点
  ②CONSTRUCT_INSTANCE instance_stake 槽位对位 ③fff2 生成点（三问一体）。
  产出=定罪笔记（行号+栈+最小序列）写入本文件 §9 复审记录追加节。
  验证：三问各有行号级答案。依赖：T-01。
- **T-03 修 CONSTRUCT_INSTANCE stake 归属**：engine.rs ~4314–4520，按
  T-02 结论。验证：probe_litpush `it.pid` 读取正确 + LitTick 仍 0 增长。
  关联 AC-01/AC-03。依赖：T-02。
- **T-04 修容器写结算**：native.rs shim_list_push + engine.rs "push" 臂
  统一元素份额转移/补 retain；验证 SET_FIELD 覆写后旧列表级联回收
  （probe live_heap 覆写相增量 ≤64）。关联 AC-01。依赖：T-03。
- **T-05 修 KD-VM3 编码**：按 T-02 定罪修元素读路径（engine.rs
  GET_ELEMENT/for-each/GET_FIELD 相关）。验证：probe_readback
  lenSeen=100 且 sum=4950（AC-02/AC-03）。依赖：T-02（可与 T-03/T-04
  并行，若定罪独立）。
- **T-06 探针硬断言转正**：musk_vm_track_tests.rs probe_attribution 加
  assert（≤64/对照 0）+ readback 断言（4950/100）。验证：全绿。
  关联 AC-01/02/03。依赖：T-03/T-04/T-05。
- **T-07 sys-monitor 端到端 soak**：merged 1s × 10min 采样；验证 AC-04。
  依赖：T-06。
- **T-08 KD-VM4 as-cast 截断**：ui_gen/vue.rs as-cast→int 降级 Math.trunc
  + 单测。验证：AC-06。依赖：T-01（可并行）。
- **T-09 回归门**：cargo tv 全绿（在案预存除外）→ cargo tf 全绿。
  关联 AC-05。依赖：T-06/T-07/T-08 全部完成。
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

## 10. 待澄清事项

- **Q-01**：KD-VM3 编码损坏与 KD-VM1 的第二 retain 是否同一处代码缺陷
  （T-02 定罪合并或分拆修复）——T-02 产出时自答，不阻塞开工。
- **Q-02**：AC-04 容差（20MB/10min）是否过宽/过严——执行期以「对照旧版
  同窗曲线」相对值复核，必要时复审修订。
- **Q-03**：`construct.instance` 双步构造（new.instance + const.i32 N +
  construct.instance）是否可合并单 opcode（顺带消除 name_len 栈残留）——
  属优化项，不阻塞本计划，登记候选。
