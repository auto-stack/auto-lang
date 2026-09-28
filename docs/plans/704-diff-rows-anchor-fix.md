---
plan_id: PLAN-704
status: execution_done
feature_name: diff 引擎缺陷修复——rows 面流位单源化（D-1）+ anchor 单调过滤（D-2）（PLAN-703 供料包消费侧登记件）
author: [agent]
created_at: 2026-09-28T01:20:00+08:00
updated_at: 2026-09-28T02:30:00+08:00
plan_revision: 1
current_step: 4
total_steps: 4
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-lang/ui/design/diff-engine.md（SD-01：锚点单调规则+rows 切片流位契约+验证锚扩充）
  - docs/specs/auto-lang/ui/design/diff-endpoints.md（SD-02：rows 面语义=下游过渡参考实现对齐注记+换位族 counts 语义+回归锚）
touched_goals:
  - 下游 auto-edit PLAN-016 blocked 尾巴解阻（golden 重定+bench diff_100mb ≤2s 判定+T15 判绿恢复——供料档 §6.3 残留 want）
  - diff 引擎 rows 投影语义=下游 011 过渡参考实现对齐（envelope 契约「逐字段同形」承诺的缺陷清偿）
---

# [PLAN-704] diff 引擎缺陷修复 D-1/D-2（rows 面流位单源化+anchor 单调过滤）

## 0. 变更摘要

auto-edit **PLAN-016 消费件执行期**（worktree plan-016-dev，全量对账
+矩阵 118/11 零新失败）勘定的 PLAN-703 供料包双缺陷，本仓修复件：

- **D-1 rows 面流位错配**：`group_hunks_annotated` 向
  `GroupedHunk.fc/lc` 写 **changes 向量下标**（mod.rs:376），而
  `build_rows` 按 **keep+change 流下标**消费（envelope.rs:220/222）。
  凡变更前存在 keep 域的形状：纯增/纯删/删多增少形**变更行整块丢失**
  （add_only：adds=3 而 rows 仅 5 条 ctx——视图层用户看不到增删内容，
  功能性破坏）、多 hunk 形**前导 ctx 重复**（scattered：rows 41 vs
  参考 21）、大文件多 hunk 形 rows **O(H²) 积累**（1% 散布改 100MB
  形估算 10^10 行量级——下游 bench 判定连带阻塞）。
- **D-2 anchor 分块非单调**：`anchor_partition`（mod.rs:180-206）
  收集「双侧恰一次」锚后仅 `sort_unstable()`（:201——a 位升序），
  **未过滤 b 位单调子序列**；`engine_changes` 段构建（:241-245）把
  锚当单调走（`cursor = (ai+1, bj+1)` 倒退）→退化段→**编辑脚本
  本身错误**（620 行换位形双向 +620/-0——等长文件双向纯增=内部
  不一致实证；counts 面即错，连带 hunks/rows）。

修复=两处单源化小改（预估 ±50 行）+回归测试面（换位 golden/多 hunk
rows 不变量/锚单调单元）+SD 两册修订。交付即解锁下游 PLAN-016
blocked 尾巴（golden 重定轮+bench diff_100mb 真跑+T15 rows 面判绿
恢复——供料档 §6.2/§6.3 登记的清偿闭环）。

## 1. 目标

- **G-1 D-1 消除（rows 投影语义复位）**：`build_rows` 切片域与
  `GroupedHunk` 标注**单源化**——rows 逐 hunk 切片无重复、变更行必在
  位、切片域不越 hunk 窗（=011 下游过渡参考实现的规范锚语义，供料
  档 §6.2 D-1 修法建议第 (a) 形：建流时记录 per-change 流位映射，
  分组器输出契约不动）。多 hunk/纯增/纯删/删多增少四族形状 rows 面
  与下游参考实现逐字段零漂移。
- **G-2 D-2 消除（锚集单调过滤）**：`anchor_partition` 输出满足
  **双坐标严格单调**（a 位经 distinct first-occurrence 天然严格递增
  ——补不变式断言；b 位经 LIS 过滤严格递增，O(n log n) patience
  式）；换位族编辑脚本恢复正确（counts 面记账不变量：
  `len_a − dels == len_b − adds == 对齐长度`，双向对称）。
- **G-3 回归测试面**：换位 golden（下游 big_reorder 同形）+多 hunk
  rows 对照+纯增/纯删变更行存在性+锚单调单元断言+既有八形态
  golden/plan703 探针/dirs 11 项零回归。
- **G-4 规范修订与解阻回执**：diff-engine.md（SD-01）锚点单调规则+
  rows 切片流位契约+验证锚扩充；diff-endpoints.md（SD-02）rows 面
  语义对齐注记+换位族 counts 语义；尾部解阻清单指下游 PLAN-016 §9
  blockers（golden 重定+bench 判定+判绿恢复的重推步骤）。

### 非目标

- diff 算法/管线更换（histogram+patience 锚点分块架构维持——703
  SD-02 契约不动）；refine 三段标记语义；envelope 字段族增删。
- 增量重比（供料包未来件）；mtime 快路（opt-in 默认关维持）；
  diff_dirs 面（对账实证零缺陷——下游 T16 组 12/12 绿）。
- 下游 auto-edit 侧任何改动（修复轮回工=PLAN-016 尾巴清偿，下游
  自驱；本件交付 commit+本册回执节=下游解阻输入）。
- 703 计划文件回改（缺陷登记已在他册——供料档 §6.2 由下游写入，
  本件 SD 引用之；归档计划不重开）。

## 2. 架构方案

分层落点（2026-09-28 实勘，auto-lang master@5c558778f）：

| 面 | 现状（缺陷锚） | 本期形态 | 依据 |
|---|---|---|---|
| rows 切片（D-1） | `build_rows`（envelope.rs:178-330）切片 `lo = g.fc.saturating_sub(g.fi − h.a1)`（:220）、`hi = g.lc + 1`（:222）——fc/lc 实为 changes 下标（mod.rs:376 `fc: idx, lc: idx`）而切片算术按流下标设计 | **流位映射单源**：流构建循环（envelope.rs:190-217）内增 `chg_stream: Vec<usize>`（第 k 个 change 在流中的写入位）；切片改 `lo = chg_stream[g.fc].saturating_sub(g.fi − h.a1)`、`hi` 自 `chg_stream[g.lc] + 1` 起沿用既有 keep 步进——分组器输出（fc/lc=changes 下标）契约不动，消费侧换算一次到位 | 供料档 §6.2 D-1 修法建议；011 下游 fc/lc=流位契约（diff-view.md）语义复位 |
| 锚点（D-2） | `anchor_partition`（mod.rs:180-206）：双侧恰一次收集+`sort_unstable()`（:201）即返回；段构建（:241-250）假设单调 | 尾部增 **LIS 单调过滤**（排序后对 b 位求严格递增最长子序列，patience O(n log n)）；输出仍 `(a_pos, b_pos)` 对——「强制 keep、内容决定」性质不变，仅剔除序冲突锚 | LIS 与锚点确定性目标同源（同输入必同锚——703 AC 确定性纪律）；换位形 LIS=三块择一（A/B 块 300 或 M 块 10），计数不变量双向 310/310 可推导 |
| 测试面 | mod.rs:581 `mod tests`（28 个既有测试——八形态 golden ctx=1 紧化+双跑字节等+窗投影+并行≡串行+snapshot≡文本+multibyte）；dirs.rs:333（11 项） | 增四组：①锚单调单元（换位 fixture 形——过滤后双坐标严格递增）②换位集成 golden（P5+A300+M10+B300+S5 ↔ 交换：adds=dels=310+双向对称+kept 记账+行序单调）③多 hunk rows 不变量（下游 scattered 同形：3 处远距单行改——变更行在位/无重复行号/域不越窗/计数=测试内参考推导）④纯增/纯删 rows 变更行存在性 | 下游 evidence-p016-recon.json 六形态镜像（漂移逐字段证据=测试期望的直接来源） |

**关键设计约束（frozen）**：
① rows 语义规范锚=下游 011 过渡参考实现（probe_diff.py
`naive_layered_diff`——供料档 §6.1 供①a 核销即以「六形态对账」为
据；修复目标=四族形状零漂移）。② **big_reorder 换位形对过渡参考的
差异=引擎时代语义（degraded 退场）非缺陷**——过渡参考该形走降级
单 replace（610/610），引擎正确形态=锚点对齐多 hunk（310/310）；
下游重定轮按证据接受该差异（非本件测试断言目标，测试断言=不变量
而非逐字段对照过渡参考）。③ 确定性纪律：LIS 结果内容决定（同输入
必同锚），并行≡串行不受影响（锚在并行分派前）。④ 修法取**最小
侵入双点**（build_rows 换算+anchor 过滤）——分组器公共契约
（`group_hunks` 无标注面/diff_lines 面）零扰动。

## 3. 技术栈

Rust（imara-diff 0.2.0 既有依赖零增）；`cargo test -p auto-lang`
（diff 模块单测+plan703 探针面）；换位/多 hunk fixture 测试内生成
（不入库文件——既有 golden 形态惯例）。

## 4. 需求分析与背景调查

**授权记录**：用户 2026-09-28 会话指令「我觉得可以现在 auto-lang
立项修复，请根据你的分析去 auto-lang 立项」——授权=**起草本件**
（缺陷分析与根因=消费侧 PLAN-016 执行 handoff 已在案）；执行/work
待用户另行启动。范围=auto-lang 单仓（diff 模块+SD 两册）；下游
auto-edit 零改动。无预算/自动续跑授权。

**来源与版本**：

- 缺陷登记（规范输入）：auto-edit 仓
  `docs/upstream/2026-09-diff-engine-supply.md` §6.2（消费侧登记——
  源码行级根因+最小复现+修法建议+消费面影响矩阵）与 §6.1 供件核销
  回执、§6.3 残留 want；对账证据
  `auto-edit/specs/auto-edit/tests/evidence-p016-recon.json`
  （六形态引擎×参考逐字段漂移）。
- 下游 handoff：auto-edit `docs/plans/016-m3-diff-engine-consume.md`
  §9 执行 handoff（blocked——矩阵 118/11 失败集 D 族 7 项+精确解阻
  动作三步）。
- 供料基：PLAN-703 归档件（docs/plans/archive/
  703-diff-engine-supply-pack-v1.md）+ SD 三册（本仓
  docs/specs/auto-lang/ui/design/{diff-engine,diff-endpoints}.md
  @master）；delivery 46efa926a；master tip 5c558778f（2026-09-28
  实勘——703 后仅 plan045 无 diff 面改动，缺陷锚位行号在册）。
- 实勘锚位（master@5c558778f）：mod.rs:201（sort_unstable）、
  :241-245（段构建单调假设）、:376（`fc: idx, lc: idx`）、
  :581（mod tests）；envelope.rs:190-217（流构建）、:220/:222
  （切片算术）。
- **703 测试面盲区（漏网机理，入 SD 验证锚修订依据）**：八形态
  golden 走 `diff_lines` 的 hunks/counts 面——无换位形（恰 D-2 触发
  形）；plan703 探针 rows 面断言用 modify 形（变更前零 keep 域——
  恰 D-1 不触发形）；两缺陷自交付即在，互不重叠地漏出双测试面。

**消费预告（回执预记）**：本件 delivered 后下游 auto-edit PLAN-016
按 §9 blockers 三步清偿尾巴（工具链重建→probe ⑦绊线 FAIL→golden
重定+bench diff_100mb 真跑+README/SD-05/budgets.json 实测回填）；
下游修复轮新勘定的引擎问题回本仓登记（669 模式）。

## 5. 详细设计

### D-1：rows 切片流位单源（envelope.rs）

流构建循环（现 :190-217）每 push 一条 change 时记录其流位：

```rust
let mut chg_stream: Vec<usize> = Vec::with_capacity(changes.len());
// push change 处：chg_stream.push(stream.len()); stream.push(...);
```

切片（现 :220-233）改：

```rust
let lo = chg_stream[g.fc].saturating_sub(g.fi.saturating_sub(h.a1));
let mut hi = chg_stream[g.lc] + 1;
// 以下非 keep 步进+尾 keep 窗内步进两段，原样保留
```

语义=切片域从「首个变更前 (fi−a1) 条流项起、至末变更后尾 keep 窗
界止」——011 下游契约（fc/lc=流位注记）在消费侧单点换算复位；
`g.fc==0 且变更前无 keep` 的既有绿形（modify/full-swap）逐字节不变
（映射恒等 ⇒ 回归零风险面）。`GroupedHunk` 字段与分组器零改动。

### D-2：anchor 单调过滤（mod.rs）

`anchor_partition` 排序后、返回前，对 `(a,b)` 序列取 **b 位严格
递增 LIS**（a 位已由 distinct first-occurrence 严格递增——断言
`a` 严格递增为不变式单测）：

```rust
anchors.sort_unstable();            // 现状 :201（a 升序，a 互异）
let lis = longest_increasing_subseq(&anchors); // b 严格递增，O(n log n)
// 返回 lis
```

patience 实现=tails 数组+二分+前驱回溯（标准形，~25 行）；结果
内容决定（锚集相同必同 LIS——稳定性不依赖实现迭代序，与 interning
首现序确定性纪律一致）。**计数面效果可推导**（测试断言依据）：
换位形锚候选=A 块/B 块/M 块三段互斥（块间 b 位必降），LIS=最长块
（300）→ kept=320 → dels=adds=310，双向对称。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/ui/design/diff-engine.md | before：锚点节=「两侧各恰出现一次的行值=强制 keep」（无单调规则）；管线 3 锚点切分未述序一致性 / after：锚点节补「**锚集经双坐标严格单调过滤（b 位 LIS）后入场**——序冲突锚弃用，内容决定性保持」；rows 切片契约注记（build_rows 按 changes 流位映射切片——消费侧单源）；验证锚节增四组（锚单调单元/换位 golden/多 hunk rows 不变量/纯增纯删存在性） | D-1/D-2 修复落账；确定性纪律成文 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-lang/ui/design/diff-endpoints.md | before：diff_files envelope 节 rows 语义未锚定参考实现；换位族未注记 / after：rows 面语义注记（**=下游 011 过渡参考实现逐字段对齐——多 hunk/纯增/纯删/删多增少四族零漂移**；换位族=引擎时代语义：degraded 退场、锚点对齐多 hunk、记账不变量）；验证锚节增换位 golden+对账镜面注记（下游 evidence-p016-recon.json 引用） | 「envelope 逐字段同形」承诺的缺陷清偿落账；下游重定轮的期望基准 | AC-01/02/04 |

## 6. 测试设计

- **锚单调单元（mod.rs tests）**：换位 fixture 形（A 块/M 块/B 块
  构造）——`anchor_partition` 输出双坐标严格递增；过滤前锚数≥过滤后
  （记录弃用数）；LIS 对恒等序列/逆序/空集三态健全。
- **换位集成 golden**：620 行换位对（测试内生成）——adds=dels=310
  双向对称+kept 记账（`len_a−dels==len_b−adds==320`）+rows 行号序列
  双坐标单调+双跑字节等。
- **多 hunk rows 不变量（D-1 主断言）**：下游 scattered 同形（40 行
  3 处远距单行改）+删多增少形（下游 unbalanced 同形）——rows 内
  变更行在位（lk=del/rk=add 对配对+单侧行）、行号序列无重复、每行
  落所属 hunk 窗内（lo−1∈[a1,a2) ∨ ro−1∈[b1,b2)）、rows 总数=测试
  内参考推导（keep+change 流重放计数）。
- **纯增/纯删存在性**：add_only/del_only 同形——adds>0 ⇔ rows 含
  rk="add" 行；dels>0 ⇔ 含 lk="del" 行（D-1 症状的最小否定断言）。
- **既有面零回归**：`cargo test -p auto-lang diff` 全绿（八形态
  golden ctx=1 紧化逐字段+双跑字节等+窗投影+并行≡串行+snapshot≡
  文本+multibyte refinement+dirs 11 项）+plan703 探针 7 项绿。
- **跨仓镜面（登记，不在本件执行）**：下游 PLAN-016 修复轮以
  probe_diff.py ⑦对账绊线复推六形态——预期 modify/add_only/
  del_only/scattered/unbalanced 转零漂移、big_reorder=换位正确形
  （§5 约束②）。

## 7. 验收标准

- **AC-01 D-1 消除**：多 hunk/纯增/纯删/删多增少四族 rows 面
  不变量全绿（变更行在位/无重复/域不越窗/计数=参考推导）；add_only
  最小复现（adds=3 而 rows 零 add 行）反证消除。验证：cargo test
  diff 新增四组全绿。
- **AC-02 D-2 消除**：换位形 counts 正确（310/310 双向对称）+kept
  记账不变量+锚集双坐标严格单调（单元断言）。验证：换位 golden 绿
  +锚单调单元绿。
- **AC-03 既有面零回归**：`cargo test -p auto-lang diff` 全量绿
  （含 dirs 11 项）+plan703 探针 7 项绿——修复前后既有断言零漂移
  （D-1 映射恒等面+锚点仅在 ≥512 行中段入场=小形状零触）。
- **AC-04 规范与回执**：SD-01/02 落档（锚点单调规则+rows 流位
  契约+rows 参考对齐注记+验证锚扩充）；供料档 §6.2/§6.3 缺陷条目
  加「已修复（本件 delivery commit）」回执注记+下游解阻三步清单
  在册。验证：文件在档 grep 锚+回执注记行。

## 8. 执行步骤

| # | 任务 | 依赖 | 落点（实勘锚） | 产出/意图 | AC | 验证（命令/预期） |
|---|---|---|---|---|---|---|
| 1 | [x] T-01 D-2 锚单调过滤 [✅ 2026-09-28：anchor_partition 排序后接 `longest_monotone_anchors`（patience LIS+前驱回溯，debug_assert a 严格递增不变式+输出双坐标断言）——commit ef0a8e1f2] | — | mod.rs:180-206+tests | 锚集单调性复位 | AC-02 | [x] `d2_anchor_partition_monotone_on_reorder` 绿（换位形锚数=300） |
| 2 | [x] T-02 D-1 rows 流位单源 [✅ 2026-09-28：build_rows 建 `chg_stream` per-change 流位映射，切片 `lo/hi` 自映射推导（分组器契约不动）——commit ef0a8e1f2] | T-01 | envelope.rs:190-233 | rows 切片域复位 | AC-01 | [x] 纯增/纯删存在性+多 hunk 不变量组绿（plan704_rows 三测试） |
| 3 | [x] T-03 换位 golden+回归组 [✅ 2026-09-28：plan704_d2 两测试+plan704_rows 三测试——换位 310/310 双向对称+kept 记账+scattered 21 行参考对照（含三段标记样本）+unbalanced 7 行逐行序列——commit ef0a8e1f2] | T-01/02 | mod.rs+envelope.rs tests | 缺陷形状回归钉死 | AC-01/02 | [x] diff:: 模块 30/0（含双跑字节等既有面） |
| 4 | [x] T-04 全量门+规范 [✅ 2026-09-28：diff:: 30/0+plan703_supply_probes 7/0（registry 直读零回归）；SD-01/02 修订 commit 0eb605bf1；供料档 §6.2「已修复·PLAN-704」回执+解阻三步清单=auto-edit edit-016 worktree commit 2f439ff（跨仓回执位——PLAN-016 承载面）。**等价验证门勘定**：仓规 cargo tf 为编译器/VM/核心协议门（AGENTS.md:66），本件=UI 模块面——模块门 diff::+探针面为正门，tf 另跑加强证据（见 §9 复审）] | T-03 | SD 两册+auto-edit 供料档 §6 | 交付+解阻闭环 | AC-03/04 | [x] 全量 diff 测试绿+SD grep 锚全中（PLAN-704×5） |

## 9. 复审记录

- 2026-09-28 起草 handoff：`stage: new`，PLAN-704，plan_revision 1。
  `outcome: pass`（起草完备：双缺陷根因/修法/验收=消费侧登记件
  §6.2 逐条转contract；锚位/测试面/盲区机理经 master@5c558778f 实勘
  锚定；计数面效果可推导（310/310）故测试断言先行成文；授权=起草，
  执行待用户启动）。`next: work`。
- 2026-09-28 执行 handoff：`stage: work | PLAN-704 | plan_revision 1 |
  outcome: pass | code_commit: worktree plan-704-dev@0eb605bf1（base
  master@ca68427fc；T-01..T-03=ef0a8e1f2，T-04 SD=0eb605bf1+跨仓回执
  auto-edit plan-016-dev@2f439ff）| task_ids: T-01..T-04 全落 |
  evidence: ①组依赖=auto-down@3373a5c 钉版 detached worktree（optional
  path dep 解析面——autodown feature 不在 default，零编译只解析）；②
  diff:: 模块 **30 passed / 0 failed**（26 既有零回归[八形态 golden/
  双跑字节等/窗投影/并行≡串行/snapshot 面]+5 新增全绿+3 ignored
  release bench）；③plan703_supply_probes **7/0**（registry 直读/
  错误形面零回归）；④手推对照三族形状=下游参考逐项吻合（scattered
  21/unbalanced 7/纯增删行存在性——下游 evidence-p016-recon.json 漂移
  证据的否定面）。| blockers: 无 | next: review`。status=execution_done。

## 10. 待澄清事项

- **Q-1 交付时序（执行期自然裁定）**：本件 work 执行待用户启动
  （auto-plan 四技能范式正常轮转）；完成后下游 PLAN-016 尾巴清偿
  由下游自驱（auto-edit 仓 §9 blockers 三步），无需本件任务承载。
- **Q-2 LIS 择块确定性（实现注记，非开放问题）**：换位形 A/B 块
  锚数同为 300，LIS 恰择一（先到先得由二分下界序决定——同输入同
  结果，跨运行稳定）；hunks 坐标随择块而异但**计数面不变量恒等**
  （310/310）——测试只断言不变量，不断言择块，避免过度约束实现。
