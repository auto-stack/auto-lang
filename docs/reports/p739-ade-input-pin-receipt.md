# PLAN-739 交付回执（F-037-R1 编辑回声×换页视面粘滞冻结 跨仓修复单）

日期：2026-10-03。产出：master `ef8eac37a`（分支 plan-739-dev
2d950e975+ef8eac37a，rebase 等价证明 range-diff `1: 4d1621a39 =
1: 2d950e975`，基面 9fcbdb5de 系）。本件是 jade-edit PLAN-037
F-037-R1 的对岸供料回执；jade 侧按其探针重跑 P2b/P5 转绿 = 缺陷闭合
（T-04/T-05 随后在其仓解锁，本仓不冒称已联验）。

## 1. 根因定谳（推翻缺陷单主假设）

缺陷单主假设「PLAN-057 回声守卫 sync 跳帧 × ADR-24 写队列/PLAN-708
ui_epoch 的 view dirty/epoch 传播竞态（epoch/序号被吞一次后失步）」被
代码勘察**否定**：update→view 主链对真实消息稳健（`on_with_input_for`
Ok 臂恒置 component.dirty → update 尾回填 view_dirty → 脏重建；VM 突变
臂/桥写全 bump 全局 `state_mutation_seq` → memo 快路径不可能陈旧命中；
500ms 热重载泵兜底 update 间隙异步写）。

真实根因 = **视面补丁钉死**，链条四环（master@59ff4e66f 实证）：

1. jade 编辑器 `oninput: .Edit`（ADE 元素）——`scan_node_for_inputs`
   只扫 `input|textarea|Input` 标签 → `.Edit` **不入 `input_state_map`**。
2. 键入消息带全文 → update 记账 `input_values["Edit"]=全文`；后续 nav
   消息 handler 后 retain 的谓词对不在 map 的事件**永不清除** → 条目永生。
3. 每次重建 `patch_input_values` 的 ADE 臂把模板求值的**新页 content
   覆写**回旧页键入全文。
4. 覆写值进 `autodown_editor_sync → sync_external` → 等于 `emit_document()`
   /`emitted_echo` 集内值 → 按回声 no-op → 编辑器核停留旧页文本。
   **粘滞 = 条目永生 × 每帧覆写**。

症状全映射：P2b（navOk/viewSwapped 分叉）✓；P5 粘滞（重按不复）✓；
对照（不键入换页正常——input_values 空）✓；snapshot vtree 编辑器子树
停留旧页 ✓；无 panic/dirty 异常 ✓。patch ADE 臂出身 = PLAN-019 批次九
（PLAN-057 回声守卫成熟前的遗产保护，动机已被守卫覆盖）。

## 2. 修复内容（master ef8eac37a）

- **D-1 摘臂（根修）**：`patch_input_values` 移除 `AutodownEditor` 臂
  （Input/Textarea/CodeEditor 语义零变化）。ADE 权威序列 =
  `state → sync_external 三层守卫 → 核`（canonical 沉淀：frame-pipeline-
  incremental.md §4c）。
- **D-2 记账单源**：update 主链 `input_values` 记账提取
  `track_input_text` / `retain_input_values_after_handler`（语义原样
  搬运），回归测试同驱动面零漂移。
- **D-3 回归围栏**：语料 `test/ui/plan739_input_pin/` + 三测试
  （真实节拍键入→换页断言 vtree/视面核随 store；ADE 摘臂语义锚 + Input
  反例锚；字面 content 守卫回退围栏）。**修前复红留痕**：临时臂形态下
  T-A 红输出 `left="第一页正文。XYZ" right="第二页正文。"`——与本仓最小
  复现（键入后换页视面钉死旧页+键入文本）逐字节同构。
- 供给面零触碰：PLAN-732 `ADE_LINK_DISPATCH`/PLAN-737 应用栅栏零改动
  （13+2 测试全绿实证）。

## 3. 门禁矩阵（fix-test-tiering 分级；Category B）

```
cargo check -p auto-lang（默认/autodown/ui-iced 三集）   ✅ 零错（警告=仓内既有基线）
cargo nextest -p auto-lang --lib --features autodown plan732   13/13 ✅ 零回退
cargo nextest -p auto-lang --lib --features autodown plan737   2/2  ✅ 零回退
cargo t autodown                                                80/80 ✅
plan739（worktree+master 各跑）                                  3/3 ✅
plan057/plan063 回声族（复审合并面）                             ✅ 29/29
裸 cargo t（--no-fail-fast，worktree@4d1621a39，56.6s）   5072 面 5057 绿；
                                              14 预存红与 master 基线逐名全等 ✅
                                              差异全为在案 flake（plan730 矩阵隔离
                                              复跑绿=737 点名 flake；master 侧
                                              plan484_024/plan502_m3 抖红=732 在案）
cargo fmt --check                                          我方文件零 diff（仓 28 处预存同基线）
```

## 4. 交付 binary 指纹（jade T-02 复验重绑目标）

- 路径：`D:/autostack/auto-lang/target/debug/auto.exe`（AUTO_EXE 默认）
- 版本：`auto 0.1.0+v0.4.2-2713-gef8eac37a`
- SHA256：`8860F25BBBD2DD6AB4DEFDFF69DD2DD57DB12B9FE16E8C69883D37AADDEA6B96`
- 说明：旧 binary 被共机运行进程持有，已 rename `auto.exe.pre-739`
  （其运行中镜像不受影响）；新 binary 由 master ef8eac37a 全量链接。

## 5. 非供给面残余（明文分界）

- **jade N4（别名页）**：消费方域（PLAN-737 回执 §5 既有分界，不属本计划）。
- **ECONNRESET 残余**：737 回执 §5 既有 KNOWN-DEBT，不属本计划。
- **F-1（新登记 KNOWN-DEBT）**：`crates/auto-cache` 测试目标仓内腐坏
  （`cargo check -p auto-cache --tests` E0063：`ShimMethod` 缺 `trait_name`
  ——PLAN-596 T-03 加字段未同步该测试构造）。日常档门禁只建
  `-p auto-lang` 目标，auto-cache 自身测试从不参与编译，腐坏不可见。
  master@9fcbdb5de 实证同破，非本计划引入。

## 6. 证据索引

- 计划：docs/plans/archive/739-ade-input-pin-page-freeze.md（复审记录
  /AC 绑定/range-diff 证明）
- 交付 commits：2d950e975（根修+测试）、ef8eac37a（SD-01 canonical），
  ff-only 落 master（无 merge commit）
- 规范增量：docs/specs/auto-lang/ui/design/frame-pipeline-incremental.md
  §4c（输入回写视面补丁边界）
