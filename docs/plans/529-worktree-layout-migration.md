---
plan_id: PLAN-529
status: reviewed                # drafting → executing → execution_done → reviewed → archived
feature_name: worktree-layout-migration
author: [zhaopuming]
created_at: 2026-09-03
updated_at: 2026-09-03

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [worktree-group-layout]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: []                   # 约定层变更，不动代码
current_step: 7
total_steps: 7
---

# [PLAN-529] worktree-layout-migration

## 变更摘要

worktree 布局从「仓内嵌套 `<repo>/.worktrees/plan-<NNN>-dev`」迁移为「按计划分组的平铺布局
`D:\autostack\.wt\<repo>-<NNN>\<repo>`」，使 `../<sibling>` 相对路径在主检出与任何 worktree
中一致成立，**从结构上消灭在 worktree 内放置跨仓 junction 的动机**，并以 wt-guard 脚本 +
PreToolUse hook 样例构成机械防线。背景：2026-09-03 14:59 事故（git worktree remove 穿透
plan-058-dev 内 junction，连锁删除 auto-lang/auto-down/auto-auto-ai 三仓 .git，Plan 526/525
重建恢复），根因与恢复全程见当日会话记录。

## 目标

1. 新计划一律使用 `.wt/<repo>-<NNN>/<repo>` 分组平铺布局；跨仓联动计划在同组内建兄弟仓 worktree。
2. 全部 agent 指令面（2 个 AGENTS.md + 4 个用户级 auto-plan 技能）同步新布局与红线。
3. 机械防线：fold 前强制 reparse point 扫描（wt-guard）；提供用户级 hook 样例拦险命令。
4. 依赖解析统一序：`$AUTO_LANG_ROOT 等 env 覆盖 → 组内 ../auto-lang → D:/autostack/auto-lang 主检出`。

## 架构方案

```
D:\autostack\                     # 主检出（现状不动，../auto-lang 天然成立）
  auto-lang/ auto-musk/ auto-ai/ auto-down/
  .wt\                            # worktree 分组根（新增）
    lang-530\auto-lang\           # 例：auto-lang 的 Plan 530 worktree
    musk-060\{auto-musk, auto-lang}  # 跨仓联动组：同组并排
```

性质：组内任一 worktree 中 `../auto-lang` 解析到组内兄弟（若有）或为空回退主检出；
worktree 内**禁止任何 junction/symlink**（红线）；`.wt` 加入 clean-targets 扫描。

## 需求分析与背景调查

- 指令承载面盘点（2026-09-03 实测 grep）：auto-lang/AGENTS.md 5 处、auto-musk/AGENTS.md 3 处、
  用户级技能 auto-plan-work 35 / merge 11 / review 6 / new 2 处；CLAUDE.md 与 auto-ai/auto-down
  无此类引用。脚本既有"布局自适应"雏形（vm-link-probe.mjs / style-parity run.mjs 的
  ../auto-lang 与 ../../auto-lang 双路径 + STYLE_PARITY_LANG_ROOT 覆盖）。
- 事故根因链：junction(`mklink /J`) 进 worktree → `git worktree remove` 递归删除穿透（沙箱
  复现实证，git 2.47.1.windows.1）→ 按 NTFS 字母序吃掉各仓 .git 后于深路径中止。

## 详细设计

1. 路径公式与命令模板：
   - 建组：`git -C D:/autostack/<repo> worktree add D:/autostack/.wt/<repo>-<NNN>/<repo> -b plan-<NNN>-dev`
   - 跨仓：同组再 `worktree add` 兄弟仓（分支名 `<repo>-<NNN>`）。
   - 折入清理：`bash D:/autostack/wt-guard.sh <组路径>` 通过后 `git worktree remove`（组空后删组目录）。
2. wt-guard.sh：`cmd //c dir /s /b /a:l <path>` 原生枚举 reparse point，非空即列出并 exit 1。
3. hook 样例（不默认启用）：PreToolUse 拦 `git worktree remove|git clean -f|rmdir /s|rm -rf`，
   先对目标跑 guard。
4. 解析序写入约定：脚本侧统一 env → 组内 sibling → 主检出。

## 测试设计

- wt-guard 正例（干净 worktree 通过）与反例（造 junction 后拒绝）实测。
- 布局性质验证：组内 `ls ../auto-lang` 指向组内兄弟；主检出 `../auto-lang` 不变。

## 验收标准

1. 6 个指令文件无残留旧路径（grep `.worktrees/plan-` 仅剩历史性描述）。
2. wt-guard 双例实测通过。
3. hook 样例就位且含启用说明。
4. clean-autolang-targets.ps1 覆盖 `.wt/`。
5. 在途 plan-525/526 不受影响（就地跑完旧布局，fold 时过 guard）。

## 执行步骤

1. [✅ 已完成] 修正 .next-id 撞号（526→529），创建本 plan 骨架。
2. [✅ 已完成] auto-lang/AGENTS.md：5 处路径迁移 + 红线 + 解析序 + guard 步骤。
3. [✅ 已完成] auto-musk/AGENTS.md：3 处同步。
4. [✅ 已完成] 用户级 4 个 auto-plan 技能同步（54 处）。
5. [✅ 已完成] D:/autostack/wt-guard.sh 编写 + 正反例实测。
6. [✅ 已完成] hook 样例 + 启用说明；clean-autolang-targets.ps1 加 .wt。
7. [✅ 已完成] 提交 auto-lang（0d48f0433）/auto-musk（8dd0758）master；验收①复检：6 文件 grep 旧路径仅剩"在途计划沿用"的刻意说明；wt-guard 正反例 4/4、hook 3/3 实测通过（见上文证据）。

## 复审记录

**复审（2026-09-03，/auto-plan:review，独立会话）——结论 PASS，status → reviewed**

验收逐条复验（全部重跑，不信勾选框）：
1. ✅ 6 指令文件旧路径残留复检——仅剩 5 处刻意保留（AGENTS 路径映射说明 + 技能中
   "legacy/在途计划"语境），无真实遗漏。
2. ✅ wt-guard 双例重跑——干净目录 clean/exit 0；含 junction 目录 BLOCKED/exit 1。
   （执行期另实测带空格路径与真实 worktree 两例，共 4/4。）
3. ✅ hook 样例就位——wt-guard-hook.sh(1.7KB)+hooks-config.sample.json(537B)
   含启用方法说明；未启用（用户决策项）。
4. ✅ clean-autolang-targets.ps1 `.wt` 发现逻辑假树干跑——正确发现
   `.wtev-checkuto-langeal	arget`；另证通用 clean-targets.ps1 天然覆盖
   .wt（根=D:utostack 全树递归）且自带 ReparsePoint 跳过守卫（line 41）。
5. ✅ 在途 worktree 无扰——plan-525-dev/plan-526-dev 注册健康；复审期间另证
   plan-525 W4 已由执行会话折入 master（711722837，tf 绿+tv 3559），重建分支
   可正常续作；.worktrees/auto-down 为普通目录非链接。

门禁归类：Category A（docs/convention-only，未触 crates/）——按仓规免 cargo 档，
计划自有测试（guard/hook/grep/干跑）全绿。

遗漏/延后/Workaround 扫描：无未批准延后；两项设计内局限登记为债（KNOWN-DEBT
P529-1 hook 相对路径盲区、P529-2 new-plan.sh 不自动建组），均不阻塞。

spec-impact：new_spec_components=[worktree-group-layout]（跨仓工作流约定，
非代码 spec）；supersedes/touched_goals 留空（无对应在册组件/目标）。

## 待澄清事项

- 在途 plan-525-dev/plan-526-dev 就地跑完；下一个新计划（lang-530 起）首次使用新布局。
- auto-down/auto-ai 无 AGENTS.md 此类引用，无需改动；如其后新建约定文件直接按新布局书写。
