# 拆分等价性验收（PLAN-867 SD-04）

> Status: active（2026-10-28 冻结；本规范是 v0.6 拆分路线 M1+M2 之后的**测试等价出口判据**）

## 命题

"拆分的新项目跑拆分前所有测试用例均能通过" 在本仓的形式定义：以拆分前提交
`abd34a355^`（=464e2316b，UI 五包与 auto-man 九包仍在语言 workspace）**实际枚举出的
测试 ID 全集**为唯一分母，新三仓在冻结组合上跑完三仓完整档后：

1. **闭包**：`分母 − (三仓现全集 ∪ 迁移账本 ∪ 退休账本 ∪ 处置表)` 的每一项都有
   MIGRATED / RENAMED / RETIRED 之一处置且证据可点开，**未解释缺口 = 0**；
2. **红集**：失败 ID 集合相对拆分前红集**新增红 = 0**（消失红须解释，不得以减少输入
   或放宽阈值实现）；
3. **防假绿**：三仓每个档位有最小选择数守卫，零选择或骤降即红。

## 冻结判据（写死）

- **分母只能来自实际选择面**：编译产物 + nextest `list --message-format json` /
  libtest `-- --list` 实测枚举，不得用源码 `#[test]` 计数替代。分母文件
  （`scripts/pre-split-universe/`）是**验收输入**：不得为接受删除/ignore/改名而重新
  生成；重新生成只在"基线被证明错误"时进行并记录原因与新旧哈希。
- **编译破面按档级红处理**：钉定提交上编译不过的档（如 818 T-02 门控破窗内的
  tf/tb/taa/ta/t3/th/tu、fastrand 泄漏期的 ui-tn）记录为该提交的既存红，其面在
  **最后可编译见证提交**枚举补全分母（窗口内测试代码差异须逐项归因）；拆分前后
  同为编译红的档（th）按参考面记录 + 债务登记，不算拆分致丢失。
- **退休通道**：有意退役的必选 ID 经 `scripts/full-regression-retirements-844.json.txt`
  类证据清单从基线必选集中移除（850 冻结文件本身不动），每行带计划引用与提交。

## 三仓完整档

单入口 `cargo tf --batch`（语言仓编排器 `scripts/run-full-regression.py`），固定序：
language → ui-full → tt → tb → man-t → man-tcore → man-tc → man-ti → ui-tu →
ui-ti → ui-tn；机器级单实例、零选择拒绝、首红停止、子门失败传播、收据绑三仓
HEAD/锁哈希/diff 与逐腿选择数、失败 ID 列表。`--frozen-trees` 是显式受控接口，
允许在干净冻结的三仓 worktree 组上执行（PLAN-867 T-07/T-11 形态）。ui-tn 为手工
native 门：批内只枚举不运行（`AUTO_UI_TN_RUN=1` 显式触发），记 pending 不记 pass。
无 `--batch` 的 `cargo tf` 保持 850 两 workspace 语义不变（per-plan 门禁不受影响）。

## 守卫与台账

- `scripts/check-tier-selection-minimums.py` + `scripts/tier-selection-minimums.json.txt`：
  每仓每档最小选择数（地板=冻结组合实测值，调整地板是显式行为，须记录理由）。
- `scripts/split-closure.py`：四类处置闭包复算（输入=分母 + 850 冻结基线 + 三仓收据
  + 跨仓日常面清单），LOST≠0 即非零退出。
- `scripts/split-golden-ledger.py`：金样/语料输入台账（tv/tt/tb/ui_gen/a2c/a2r/
  blueprints/widgets/aavm/book listings 目录级 diff，逐变更归因到计划提交；禁止以
  重生成金样自证通过）。
- 红集基线：`docs/plans/attachments/867/red-baseline.md`（钉定提交/见证树定向重跑
  采集；环境腿 th/tn 与 Windows aavm 跳过项按 pending 记录，缺环境不算 pass）。

## 出口

PLAN-867 收据（`tf --batch --frozen-trees` 在冻结三仓组合实跑 + closure/golden/
red-baseline 三台账全绿）构成拆分路线的测试等价出口；此后三仓测试面的演进以
`tier-selection-minimums.json.txt` 地板与 850 基线校验为持续守卫。

来源：[PLAN-867](../../plans/867-split-equivalence-test-closure.md)。
