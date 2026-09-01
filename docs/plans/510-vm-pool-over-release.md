---
plan_id: PLAN-510
status: execution_done          # drafting → executing → execution_done → reviewed → archived
feature_name: vm-pool-over-release
author: [zhaopuming]
created_at: 2026-09-01
updated_at: 2026-09-01

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm]
current_step: 6
total_steps: 10
---

# [PLAN-510] VM 字符串池幻影 freelist 注入源根修（over-release 族审计清偿）

> **出身**：本问题由 auto-musk PLAN-053（VM 上游跟踪伞，已归档）批4
> 实机调试发现并完成**防线修复**；注入源本体按边界规则归还 auto-lang
> 自有流程。证据全文见 musk 仓 `docs/plans/archived/053-vm-upstream-tracking.md`
> 红清单 P-053-8 行 + 执行步骤 22/23（防线上下文：musk main db5ea5d、
> auto-lang master a5f879286 / 58aa4530d）。

## 变更摘要

VM 字符串池的引用计数账本存在**多扣款**缺陷族：某些路径对同一槽位的
`pool_release` 次数多于 `pool_retain` 次数，使 rc 被提前打到 0，**存活
槽位被错误塞回复用队列**（幻影 freelist 条目）。复用时槽内容被覆写、
rc 被清零，仍在使用该字符串的代码从此读到别人的内容；孤儿 release 又
把 rc 打到下溢（0xFFFFFFFF），某个孤儿再看到 prev==1 时再次触发 free，
槽再入 freelist——**伤害自续成风暴**。musk 实机实测：点击会话的实参由
会话 id 依次漂移成会话名「你好」、HTTP 404 JSON；一个会话期内清扫器
捕获 **1896 个**不同槽位的幻影首见。

PLAN-053 已在 master 落下**三层防线**（内容校验/墓碑先行/弹出清扫），
用户可见腐坏已清零——但防线治标：注入源仍在持续产生幻影条目（被清扫
器丢弃），且受创槽位 rc 永久坏死造成**慢性槽泄漏**。本计划审计清偿
注入源本体，让记账不变量（每条引用恰好一次 retain/release；freelist
槽恒 rc==0）重新自持，防线降级为纵深防御而非唯一屏障。

## 背景与证据链（musk PLAN-053 批4 实测）

**池机制**：字符串存全局池按槽位索引；栈/堆上的字符串值 = 槽位索引
（NV TAG_STRING，负 i32 编码）。入栈引用 `pool_retain` +1，消亡
`pool_release` −1，归零 → `pool_free_idx`（清内容/删 dedup 键/置墓碑/
进 freelist）；`add_string` 先查 dedup（内容→槽），miss 则从 freelist
弹槽复用（**复用时 rc 重置 0**）。

**实机时间线**（P419_POOL_LOG=1 全池日志 + canary）：

1. 症状①：musk 点击会话「你好」（id `8f20…`），handler 实参变成
   「你好」。canary 实锤：`add_string("8f20…")` 经 dedup 残键命中槽
   2348，而槽内容已是「你好」——dedup 表残留指向已换内容槽位的键；
2. POOLLOG #222→#223：槽 2348 的 `8f20…` 条目**存活（rc=1）**时，被
   freelist 弹出复用给 `http://127.0…`，**期间无任何 FREE 日志**——
   freelist 含指向存活槽的条目（幻影条目）；
3. 症状②（第一刀修复后暴露更深一层）：store 兄弟调用的实参变成
   `{"error":"HTTP 404","status":404}`——又一个活槽被偷，后落进槽的
   是 404 响应 JSON；
4. 风暴实拍：签名日志成对刷出槽 49299 以 **rc=5**（五个存活持有者）
   被弹出复用，紧接 rc=**4294967295**（0−1 下溢回绕）；单会话期
   清扫器 1896 次幻影首见。

**自续循环**（为什么刹不住）：复用重置 rc=0 并覆写内容 → N 个真实
持有者的孤儿 release 把 rc 打下溢 → 若复用者 retain 过（0→1），某孤儿
release 又见 prev==1 → 再次 free → 槽再入 freelist → 下次弹出又是
幻影……日志里 rc=5/rc=0xFFFFFFFF 成对刷屏即此脉搏。

**master 既有防线**（本计划不得回退，收尾后降级为纵深防御）：

| 防线 | 落点 | 拦截 |
|:---|:---|:---|
| dedup 命中侧槽内容校验（残键→重内化自愈） | `engine.rs add_string` | 错串返回 |
| rc 归零即写锁复核 + 墓碑先行 | `rc.rs pool_release` | 释放中途复活竞态 |
| freelist 弹出清扫（rc>0 丢弃条目绝不复用） | `engine.rs add_string` | 活槽内容覆写 |

## 问题定义

**over-release**：某槽位的 release 调用总数 > retain 调用总数。充分
条件路径（已知/嫌疑，按优先级）：

1. **裸池写绕过记账**：`vm/ffi/http_server.rs` ~L2044/L2055 仍直接
   `vm.strings.write().push(bytes)` + `push_nv(encode_string(idx))`——
   不走 `vm.add_string`（无 dedup 注册）、不做 `pool_state.ensure_len`
   （无 rc 数组覆盖）。P-053-5（master 366075f17）把 `native.rs`/
   `stdlib.rs` 全量改 `add_string` 时**未含 http_server**。这种引用
   天生无计数，任何后续 release 即凭空多扣；
2. **绕过咽喉函数的引用入栈**：Plan 419 规定所有 TAG_STRING 引用
   入栈必须走 `rc_push_str_idx`（入栈即 +1）。全仓扫
   `push_nv(auto_val::encode_string` / `push_string` 直呼点，漏网者
   制造无计数活引用，其消亡 release 即多扣；
3. **同一引用双重释放**：opcode/CALL 错误回退路径在正常分支与错误
   分支各 release 一次（如 intercept_error 重入边界）；
4. **多段 free 竞态残余**：墓碑先行已关主窗（a5f879286），但若存在
   其他向 freelist 写入的暗桩（engine.rs L935 兜底回推等）仍可注入。

## 目标

- **G1 嫌疑清单全核实**：上述 4 项逐一取证（修复/证伪/降级为观察项），
  结论回写本计划。
- **G2 注入源清零**：长跑浸泡下幻影签名 `[P053-8] phantom freelist
  entry dropped` **0 次**（当前基线：musk 单会话期 1896 次）。
- **G3 记账自持**：debug 构建下 rc 无下溢（审计钩子或浸泡断言）；
  防线三层保留但不再触发。
- **G4 债务归位**：`engine.rs` 注释中悬垂的 `docs/plans/060` 指针接正
  （现指向 archive/060-closure-syntax.md，主题不符）；`DEBTS.md` 补
  池生命周期债条目并指向本计划。
- **G5 零回归**：auto-lang 全量测试绿（基线：settings/storage/dock
  并行 7 红）；musk vue 三门禁 + 实机会话链路不回退。

## 架构方案

```
现状(防线兜底,注入源活跃)                本计划后(记账自持,防线降纵深)
──────────────────────────              ─────────────────────────────
over-release ──→ 幻影条目 ──→ 清扫器     over-release 清偿(路径级修复)
                 (每会话1896次丢弃)                │
                    │                             ▼
防线三层 ──→ 用户可见腐坏=0        freelist 不变量自持(rc>0 不可入)
慢性槽泄漏(rc下溢槽永不回收)        防线三层保留为纵深防御(不触发)
```

**方法论**（沿用 PLAN-053 批4 已验证的手段）：

- **取证**：`P419_POOL_LOG=1` 全池日志时间线重建 + `P419_TRACE_POOL=<idx>`
  单槽生死链追踪 + `call_handler_for` 既有双 canary（intern 漂移/入栈
  编码，58aa4530d 落）；
- **最小复现**：池/引用层单测直接构造（PLAN-053 的
  `musk_vm_track_tests.rs` 残键自愈/幻影清扫两测试为模板）；
- **浸泡**：musk 前端 VM 长跑（vm-link-probe 形态扩展为 soak 模式：
  ≥30min churn 或等效请求量），断言幻影签名计数 0。

## 详细设计

### Phase 1——记账面全审计（G1）

1. 全仓扫描 TAG_STRING 值的**产生点**（`encode_string` / `push_string` /
   `push_str_idx` / `write_i32` 伪装负数等形态），逐一核对是否走
   `rc_push_str_idx`/`vm.add_string` 咽喉；http_server 裸推两处（L2044/
   L2055）首先改 `vm.add_string`；
2. 全仓扫描 `pool_release` 调用点，核对与 retain 的配对性（重点：
   错误回退/intercept_error 重入路径、CALL 实参弹栈后的双释放）；
3. 每项结论三态：**修复**（附测试）/ **证伪**（附证据）/ **观察项**
   （登记残留风险）。

### Phase 2——记账可观测（G3 的仪器）

4. debug 构建增配对审计钩子：per-slot retain/release 调用栈采样
   （复用 P419 UAF trace 设施），首次 over-release（release 时 rc==0）
   即打印双栈——把"哪条路径多扣"从推断变实锤；
5. soak 测试设施：`cargo test` 外挂长跑目标（或 musk 侧脚本），断言
   幻影签名 0 + rc 无下溢。

### Phase 3——清偿与收线

6. 按 Phase 1/2 结论逐项修复 + 回归（每项一个失败测试先行）；
7. 长跑浸泡验收（G2/G3 数值达标）；
8. 防线去留复核：三层保留（纵深防御），但文档明确"不触发"为健康态；
9. 债务归位（G4）：060 指针接正 + DEBTS.md 条目；
10. 批末门禁：auto-lang 全量 + musk 三门禁 + musk 实机会话点击链路
    复验（悬停 tooltip id 正确 + 切换加载 + 无幻影签名）。

## 测试设计

- **单元**（沿 `musk_vm_track_tests.rs` 或迁入 `vm/` 就近测试文件）：
  - 残键自愈（已有）/ 幻影清扫（已有）保持绿——防线上Regression钉；
  - 新增：http_server 参数入池走 add_string 后 rc/dedup 可见性；
  - 新增：每个修复点一个最小失败测试（TDD 先红后绿）；
- **浸泡**：soak 模式断言（幻影签名 0 / 无 rc 下溢 / 池规模增长 ≤ 阈值
  ——顺带量化慢性泄漏的修复收益）；
- **不回归面**：auto-lang 全量测试（基线 7 红不新增）；musk vue 三门禁
  （build strict / vitest / vm-link-probe）；musk VM 实机会话链路。

## 验收标准

1. G1 清单四项全部出结论（修复+测试 / 证伪+证据 / 观察项+登记），无
   悬空项；
2. 长跑浸泡（≥30min 或等效 churn）：`phantom freelist entry dropped`
   签名 **0 次**，rc 无下溢记录；
3. 池规模在浸泡期稳定（复用恢复，慢性泄漏消失或量化到可接受阈值并
   登记观察项）；
4. auto-lang 全量测试绿（7 红基线不新增）；musk 三门禁绿；musk 实机
   会话点击链路复验通过；
5. `engine.rs` 060 悬垂指针接正；`DEBTS.md` 池生命周期债条目落位并
   双向指向本计划。

## 执行步骤

1. 建工作 worktree(按 auto-lang 计划流程惯例命名)。
   [✅ 已完成] `.worktrees/plan-510-dev`(branch plan-510-dev),状态
   drafting→executing 已翻转。
2. Phase 1 审计:http_server 裸推改 `vm.add_string`(先行,独立可合);
   全仓 TAG_STRING 产生点/release 配对扫描,G1 清单结论回写。
   [✅ 已完成] 提交 7a8ac1d2e/b190a5b33。**G1 四项结论**:
   ①裸池写绕过记账——**属实且规模远超立项估计**:19 处收口咽喉
   (http_server 同步臂 3 + async 簇 5〔裸写池+rc_push_str_idx,但新槽越
   rc 数组时 retain 静默 no-op〕+ stdlib 两个重复派发拷贝 4 + native.rs 5
   〔add_string 后裸 push_nv;dedup 命中他人活槽时 POP 直接把活槽打到 0
   进 freelist = 幻影主通道〕+ stdlib file_stem 1 + ffi.rs 1 + py_ffi 3)
   ——全部统一 push_str_arg/intern_runtime_str/rc_push_str_idx;
   ②绕过咽喉的引用入栈——**属实**(同①清单,已修);③双重释放——
   **证伪**(POP/POP_N/intercept_error/slot_range/task_stack 释放后槽位
   清零,结构性防护;无实例);④freelist 暗桩——**证伪**(唯一运行时
   写入者 pool_free_idx 墓碑先行 + engine.rs 兜底回推 rc==0 前提安全;
   musk_vm_track_tests:1250 为清扫器测试刻意注入夹具)。顺带删除零调用
   死代码 push_tagged_value(非 rc 版)。
3. Phase 2 仪器:debug 配对审计钩子 + soak 断言设施。
   [✅ 已完成] (a) PoolState 增 underflow_events/phantom_drops 计数 +
   pool_release 下溢探测(P510_AUDIT=1 双栈实锤)+ sweeper 总数计数 +
   AutoVM::pool_health() 快照;(b) soak 设施:pool_soak_churn_short
   (800 轮,入日常门禁)+ pool_soak_churn_long(#[ignore],P510_SOAK_ITERS
   可调)。**soak 短跑首跑即抓到真泄漏**:live_shares=800(每轮 1 份)——
   二分定位两处 over-retain 家族:BUILD_FSTR 弹运行期串 part 无 release
   (pop_tagged 家族 6 消费点同病,pop_tagged_rc 配平修复)+ StakeGuard
   只释放堆引用漏池串实参(扩展 pool_idx 字段 Drop 配平)。修后全部
   bisect 用例 live_shares=0/underflow=0/phantom=0/池复用恢复。
4. Phase 3 按结论逐项修复（TDD）+ 长跑浸泡验收。
   [✅ 已完成] 提交 20899f3c8/5d9700cb1。修复即 Phase 2 二分矩阵结论落地
   （6 处 pop_tagged 消费配平 + StakeGuard 池份额扩展,顺序=先拷贝后释放;
   BUILD_FSTR 外层读守卫重构防同线程自锁）。**浸泡验收**:短跑 800 轮
   (日常门禁)绿;长跑 2M 轮 231s 绿——underflow_events=0 /
   phantom_drops=0 / 终态 live_shares=0 / freelist 复用恢复 / 池规模有界
   (等效 churn 远超 musk 单会话 1896 幻影签名基线)。
5. G4 债务归位（060 指针 + DEBTS.md）。
   [✅ 已完成] engine.rs add_string 文档债指针接正(docs/plans/060 系闭包
   语法,主题不符→指向 KNOWN-DEBT-AND-RISKS.md P510-1 条目 + 本计划,
   随 20899f3c8 入库);KNOWN-DEBT-AND-RISKS.md 增 P510-1 池生命周期债
   (索引 u32 化/池 GC/裸 pop_arg_nv 残余)双向指向本计划,P499-6/7
   同步改记已偿还。
6. 批末门禁全绿 + 红清单/文档收口。
   [✅ 已完成] 三档门禁全绿:cargo t 3341/3341;cargo tv 3482/3482
   （P499-7 两测转绿）;cargo tf 3342/3342（含 1M churn 档
   str_churn_bounded_large 21.4s + docs_gen/schema_drift）。浸泡:2M 轮
   绿。防线三层保留未触发（soak 断言 phantom_drops==0 即健康态文档化）;
   红清单收口:KNOWN-DEBT P510-1 条目（u32 化/池 GC/裸 pop_arg_nv 残余
   观察项）。musk 实机复验为后续动作（见待澄清）。

## 待澄清事项

- **范围裁定**：060 原始愿景（索引 u32 化/池 GC）是否并入本计划？
  默认**不并入**——本计划聚焦 over-release 清偿（记账正确性）；池 GC/
  索引宽度是独立工程，待本计划闭环后按泄漏量化数据决定是否立项。
- **soak 载体**：auto-lang 仓内 soak 测试 vs musk 侧脚本驱动实机长跑，
  执行时按最小成本裁定（musk 侧复现真实 churn 分布，auto-lang 侧可控
  可重复，可能两者都要）。**已裁定（执行期）**：auto-lang 仓内 soak
  （pool_soak_churn_short 入日常门禁 + long 档 ignore 显式触发）为主
  验收载体；musk 实机复验（实机点击链路 + P053-8 签名观察）因 musk 仓
  不在本工作区，登记为**后续动作**，不阻塞本计划闭环。
- **P499-6/7 关联裁定（2026-09-01 分诊）**：两枚债务与字符串池
  over-release **均不同源**——P499-7 真因=native ID 撞号（Log×Shell
  1800-1803）+ channel 用例空期望文件；P499-6 真因=kitchen-sink 生成器
  对视图关键字名元素（link）发射标签简写。两者已随本计划 worktree 提前
  清偿（提交 7a8ac1d2e / d0c23388d），债务账本同步改记。
