# PLAN-742 第六次独立 final review（r6）报告

- 审面 `8d59f5e88962a51a7e608b5ffe9bd8d03ff22086`（plan-742-dev），基面
  `2174f0de28dd786369b2e44e94bfa1a1ac806499`；工作树
  `D:/autostack/.wt/lang-742/auto-lang`，diff = 单文件
  `crates/auto-man/src/api_gen.rs`（+72/-2），实现全部已提交。
- 审核人：Kimi Code 独立会话，未参与 742 任何一轮实施（前五轮审核为
  Codex /root；本轮为不同 agent 的独立上下文）。验证执行：两个只读子
  agent（门禁复跑、replay 重放/探针）+ 本会话直接代码检查。
- plan_revision=1，无语义合同变更；入口状态 execution_done（3/3，r6 work
  记录已提交于主检出 d7587dccd）。

## 1. 修复正确性独立核查（F-742-R5-01）

`split_top_level_args`（api_gen.rs:2764）两个调用点的输入均为**发射器输出**
（2837 = 生成的 db.rs/companion fn 声明行；3091 = 生成的 api.rs handler 体内
模块调用实参），不是用户原始 .at 文本。发射器形态规律经源码实证：
`crates/auto-lang/src/trans/rust.rs:3960` 二元操作符一律
`write!(out, " {} ", op_str)`（两侧空格）；泛型经 `Vec<{}>`/`{}<{}>` 格式串
紧贴标识符。因此"glued-to-ident `<` = 泛型开角"在该输入面上是可靠判别式。

闭合侧 `>` 保持饱和递减：比较 `>` 只出现在角深度已为 0 的位置（发射面不存在
`a > Vec<T>` 的错位组合——`<` 必随泛型标识符先开角）。字符串内 `<`/`>` 由
in_string 守卫排除；`'<'` 字符字面量前字节为 `'`（非 ident），不误开角。

## 2. 门禁复跑（子 agent 执行，本会话复核原始输出）

| 门禁 | 命令 | 结果 |
|---|---|---|
| api_gen | `cargo nextest run -p auto-man --lib api_gen` | **49 passed / 1 ignored**（49/49 复证；ignored = `regen_real_015_backend_db_delegation` 刻意工具，强跑亦过）。注：work 记录写的 `cargo t api_gen` 实际只匹配到 auto-lang 一个 openapi 测试——正确拼写为上述命令，结论不受影响 |
| tt | `cargo tt --no-fail-fast` | **5648 pass / 1 timeout**（plan707 `wait_generator_park_drive_count_static` 满池 120s 超时，定向隔离复跑 1/1 PASS）——与 r5 审核同款已知 flake 同处置，不将整档称全绿 |
| check | `cargo check -p auto-man` | exit 0；auto-man 27 warnings ≤ master 基线 29（净 -2，r1 清偿成果保持），触面文件零告警 |
| rustfmt | `rustfmt --edition 2021 --check crates/auto-man/src/api_gen.rs` | exit 0 |

## 3. r5 replay 独立重放（提交态）

子 agent 将 review-r5 的 replay.ps1/execute-generated.ps1/probe.rs +
worktree 的 auto-man-build.jsonl 复制到仓外 scratch
（`D:/autostack/tmp/p742-verify-r6-{evidence,scratch}`）运行，复用
8d59f5e88 构建的 libauto_man.rlib：**24/24 case 与
work-r6/replay-r5-generated-results.json 逐字段一致**（compile_exit=0、
actual、source/generated SHA256 全同）。`compare_less` 生成 handler 含修复
形态 `crate::repository::choose(1 < 2, &s)`（r5 主检出证据为漏借用形
`choose(1 < 2, s)`）——反例真实、修复真实。

CLI 身份：worktree `target/debug/auto.exe` SHA256 = `cc57d2c90b08…5341`，
与 work-r6 README 记录一致，mtime（10:00:22）晚于修复提交（09:59:44）。

## 4. 审核方独立新探针（非执行者回归输入）

在 scratch 副本 probe.rs 新增两个此前任何一轮都不存在的用例（实编+运行+精确
形态断言）：

| 用例 | 输入形态 | 发射结果（逐字） | 编译/运行 | 判定 |
|---|---|---|---|---|
| `cmp_glued_source` | 用户源码写**无空格**比较 `repository.choose(1<2, s)` | `crate::repository::choose(1 < 2, &s)` | exit 0，actual=2（=预期） | **PASS**——实证发射器归一化间距，glued-`<` 启发式在真实用户源下永不遇到 glued 比较 |
| `cmp_nested_call_arg` | 比较实参 + 第二实参为需借用的嵌套模块调用 `choose(2 > 1, repository.path())` | `crate::repository::choose(2 > 1, &crate::repository::path())` | exit 0，actual=10（=预期） | **PASS**——比较切分与嵌套借用组合正确 |

原有 24 例同轮全部通过，无相互干扰。

## 5. 验收映射

| AC ID | 结果 | 依据 |
|---|---|---|
| AC-742-01 | pass | 构造降级/字面量保持旧例全绿（replay 24/24），本轮无回归 |
| AC-742-02 | pass | F-742-R5-01 关闭（重放+两新探针）；R4-01 边界纪律保持 |
| AC-742-03 | pass（可考边界） | 沿用已接受的类别级归因；比较发射现由实编 oracle + 探针覆盖 |
| AC-742-04 | pass | 返回包装/出口矩阵旧例全绿，cmp 双端点实编运行=2 |
| AC-742-05 | pass（绑定注记） | 电池提交态 S0–S19 fails=0（收据 `work-r6/notes-v1-battery-summary.txt`，10:09:44，制品目录 79 文件在案）。注记：work 记录的 api.rs mtime 绑定（10:04）已被 auto-notes 另一会话 10:22 再生成覆盖，盘上不可复核——持久电池日志与制品保留；同 r5 处置，不冒称当前 Notes 工作区整体通过（该会话 S20 WIP 非 742 面） |
| AC-742-06 | pass（边界沿用） | Vue smoke 13/13、S10 全生命周期为 r5 审核已独立通过事实；VM 交互未验为 r4/r5 已接受边界，不新增门禁 |

历史 finding 闭合：R1-01/02/03/04、R2-01..04、R3-01/02/03、R4-01 均已于前轮
关闭且本轮无回退证据；R5-01 本轮关闭。**零新 finding**。

## 6. 规范增量审核

冻结稿 [spec-delta.md](spec-delta.md)，SHA256
`04AA21D108255F07A1F55D65EEB38A7B7D6F9F78506539E5EBD3C70013DA8803`。
五目标路径与 SD IDs 沿用；SD-742-01/02 的最后阻塞（R5-01）已关闭，本轮可
批准为完成；SD-742-03 目标文件在主检出已被 PLAN-773 沉淀改写
（720DFCDD…），merge 沉淀须基于当前 canonical 版本合入，不得盲覆盖。
`new_spec_components=[]`、`touched_goals=[GOAL-003, GOAL-007]` 维持。

## 7. 债务候选（非本轮门禁项）

- **新**：`crates/auto-man/tests/plan734_commands_fixture.rs` 预存编译失败
  （E0255/E0425/E0433），阻塞任何非 `--lib` 的 auto-man 测试运行。与 master
  逐字节一致（`git diff master HEAD -- <file>` 为空），非 742 引入；建议
  登记 KNOWN-DEBT 并归 PLAN-734 面。
- 沿用：db.rs 三个 Plan 399 遗留正则 pass（另案）、Upload 入口源码级接线、
  D-742-1（跨模块导入类型构造解析，另案）、VM autotest 前提（有头桌面 MCP）。

## 8. 结论与收口门禁

**outcome: pass（final）**。实现/复审判据全部通过，T-1/T-2/T-3 重计 3/3，
状态翻 `reviewed`。

merge 拥有的待办（非本轮缺陷）：
1. 规范沉淀按 §6 基于当前 canonical 执行（a2r-std/project.md 须 rebase 式
   合入）；`.autoos/specs.json` upsert + spec-index 刷新。
2. merge 前主检出已前移（PLAN-773 落地，含 trans/rust.rs 类型化错误改动，
   与 742 面不相交；`git merge-tree` 实测干净 exit 0）——merge 时按当时
   HEAD 重验门禁。
3. 批量回归到期判定（.last-batch-regression.json）。
4. worktree 清理：先 `wt-guard.sh`，注意 worktree 内未跟踪的
   `docs/plans/attachments/742/` scratch 副本（canonical 证据已在主检出）。
