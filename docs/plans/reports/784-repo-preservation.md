# PLAN-784 旧仓保全收据（repo preservation receipt）

> 生成：2026-10-15，work 阶段（plan-784-dev 执行树）。Category A：全程未改任何
> Rust/构建输入，未运行 cargo 门禁。工具：`scripts/legacy-preservation.py`
> （inventory / preserve / anchor / verify / recover-drill / drill）。
> 所有结构化证据在 [attachments/784/](../attachments/784/)；payload 在仓外
> 独立恢复位置（§2）。本收据只写元信息，敏感文件内容不入库。

## 1. 三仓状态冻结（T-01）

明细：[t01/inventory-auto-ui.json](../attachments/784/t01/inventory-auto-ui.json)、
[t01/inventory-auto-man.json](../attachments/784/t01/inventory-auto-man.json)、
[t01/inventory-auto-lang.json](../attachments/784/t01/inventory-auto-lang.json)。

| 仓 | 主检出 | 分支 | tip | WIP | stash | 其他 worktree | 适用指令 |
|---|---|---|---|---|---|---|---|
| auto-ui | `D:/autostack/auto-ui` | master | `f7d92eef273c`（衍星居士 2026-02-28，AURA pipeline） | ` M Cargo.lock`；ignored：`.claude/` | 0 | 无 | CLAUDE.md（其"master 已清空"表述过时——实测 `ls-tree HEAD` 满树） |
| auto-man | `D:/autostack/auto-man` | master | `51fafe82e8e0`（puming.zhao 2026-01-21） | 无跟踪改动；ignored：`Cargo.lock`、`.vscode/iar-vsc.json` | 0 | 无 | CLAUDE.md |
| auto-lang | `D:/autostack/auto-lang` | master | `7d51845ff`（含 Plan 787 草案簿记提交） | 主检出另有 783 复审会话的 docs/plans 簿记 WIP 与根目录游离 `ledger_770.py`（非本计划所有，未触碰） | 0 | lang-770/lang-781/lang-783/lang-784 | AGENTS.md |

refs 冻结：auto-ui 8 条（master + origin/{HEAD,gpui2,gpui3,iced,master} + tags
v0.1/v0.2）；auto-man 26 条（master + origin/{HEAD,master,v0.0.x} + tags
v0.0.1–v0.0.20 含 v0.0.4.fix/v0.0.18.1）。与计划快照（inputs.md）比对：两仓
tip/WIP 一致，快照未过期。仅读本地 refs，未 fetch，不声称远端同步状态。

在途写入范围核对：781（executing 18/26，lang-781 树）、783（修复中，lang-783 树
+ 主检出簿记）、787（drafting 草案）；均与旧仓 refs/工作区无写入交集。

## 2. 保全载荷（T-02）

恢复位置：`D:/autostack/.recovery/784-legacy-preservation/`（三仓之外，含 README
与恢复指引）。清单副本（元信息）：
[t02-manifest-auto-ui.json](../attachments/784/t02-manifest-auto-ui.json)、
[t02-manifest-auto-man.json](../attachments/784/t02-manifest-auto-man.json)。

| 项 | auto-ui | auto-man |
|---|---|---|
| bundle（--all + HEAD） | `auto-ui/bundle/auto-ui-full.bundle`，1,451,022 B，SHA256 `c3ef028febeb85f2…fcb252c4ad8e6593`，含 9 refs | `auto-man/bundle/auto-man-full.bundle`，SHA256 `2fc84d897f21b4c2…47647277c56d269`，含 27 refs |
| 脏跟踪文件 | `Cargo.lock`：work `4b0b9e693084…` / head `f72645680830…` / diff `113aebcd57fe…`（wip/ 三份原始字节） | 无 |
| untracked | 无 | 无 |
| ignored 点名保留 | `.claude/settings.local.json`（`73ecb1010d11…`） | `Cargo.lock`、`.vscode/iar-vsc.json` |

未提交/未跟踪内容不依赖 bundle 兜底——bundle 只含提交历史与 refs；上述字节级
副本即 V02 的恢复 oracle。ignored 其余项（构建产物类）仅列清单于 manifest，未备份。

## 3. legacy refs 与隔离准备树（T-03）

refs（本地，未 push；同 tip 幂等、异 tip 拒绝移动）：

| 仓 | 分支 | 标签 |
|---|---|---|
| auto-ui | `legacy/pre-v06-extraction` @ `f7d92eef273c…` | `legacy-pre-v06-f7d92eef` @ 同 tip |
| auto-man | `legacy/pre-v06-extraction` @ `51fafe82e8e0…` | `legacy-pre-v06-51fafe82` @ 同 tip |

准备树（组 `D:/autostack/.wt/lang-784/`，无 junction/symlink，wt-guard 三树 clean）：

| 树 | 分支 | 基点 | 状态 |
|---|---|---|---|
| `auto-lang/` | `plan-784-dev` | master `7d51845ff` | 本计划执行树 |
| `auto-ui/` | `codex/v06-extraction` | `f7d92eef273c`（保全 tip） | 隔离准备树，未替换代码树 |
| `auto-man/` | `codex/v06-extraction` | `51fafe82e8e0`（保全 tip） | 隔离准备树，未替换代码树 |

两旧仓主检出保持在 master、WIP 原样未动（主分支未清空、未切换、未 push）。

## 4. 演练与反控（V01–V04）

| 用例 | 结果 | 证据 |
|---|---|---|
| V01 disposable clone 恢复 | **pass**：两仓 bundle → 临时 bare fetch `refs/*:refs/*`，全部冻结 refs sha 相等（auto-ui 8、auto-man 26），tip tree 相等（`31ede719e46f…` / `4017c1c84e5b…`），旧 tag 全部可追溯（2 / 22）；临时 clone 已删，不在任何受保护仓内 | [t03/v01-recover-drill-auto-ui.json](../attachments/784/t03/v01-recover-drill-auto-ui.json)、[v01-recover-drill-auto-man.json](../attachments/784/t03/v01-recover-drill-auto-man.json) |
| V02 原字节与原工作区不变 | **pass**：verify 两仓 ok——载荷文件在且 hash 与 manifest 相等；work 副本与当前原文件 hash 相等；原工作区 status/HEAD/symref 与 T01 冻结一致；保全 refs 无漂移 | [t03/v02-verify-auto-ui.json](../attachments/784/t03/v02-verify-auto-ui.json)、[v02-verify-auto-man.json](../attachments/784/t03/v02-verify-auto-man.json) |
| V03 幂等/冲突反控 | **pass**：真实仓 anchor 重跑=branch/tag 均 `reuse`（exit 0）；临时仓四控 D1 同 tip 幂等 / D2 异 tip 明确拒绝且原 ref 不动 / D3 HEAD 漂移拒绝 / D4 preserve 重跑复用既有 bundle（hash 不变）= 4/4 | [t03/v03-anchor-rerun-auto-ui.json](../attachments/784/t03/v03-anchor-rerun-auto-ui.json)、[v03-anchor-rerun-auto-man.json](../attachments/784/t03/v03-anchor-rerun-auto-man.json)、[v03-drill-controls.json](../attachments/784/t03/v03-drill-controls.json) |
| V04 清单/读取失败反控 | **pass**：777 审计对基线缺失/配置缺失/根缺失均显式 exit 3 失败不吞空；刷新后观察/strict 扫描 identity 可重放（39 声明边绑定 `7d51845ff`） | [t04/v04-negative-controls.txt](../attachments/784/t04/v04-negative-controls.txt)、[t04/audit-*.json](../attachments/784/t04/) |

## 5. 777 边界门刷新与迁出入口清单（T-04）

- 冻结基线重锚 `7d51845ff`（39 声明边；777 归档时登记的"master 干净时重绑"常规
  运维在本执行树完成）；strict **pass**（0 违例 / 2 grandfathered）。
- 迁出候选路径→未来 owner→资源/生成物/测试/消费者→在途计划表：
  [t04/migration-inventory.md](../attachments/784/t04/migration-inventory.md)。
  与 781/783/785/786/787 的重叠判定在表 §2（781 的 scroll 族、785 的 clipboard
  族、786 的 pac 读取、783/787 的 AC/cache 实验链，分界清晰）。

## 6. 三仓前后指纹

| 仓 | 前（T01） | 后（T05 verify 时点） |
|---|---|---|
| auto-ui | master@`f7d92eef273c`，` M Cargo.lock`，8 refs | tip/分支/WIP/status 全部不变（verify ok）；新增本地 refs 仅 legacy branch/tag 2 条 + 组内准备树 1 处 |
| auto-man | master@`51fafe82e8e0`，干净，26 refs | 同左不变；新增 legacy branch/tag 2 条 + 组内准备树 1 处 |
| auto-lang | master@`7d51845ff` + 他计划簿记 WIP | master tip 不变；新增 lang-784 组与 plan-784-dev 分支（本计划交付物） |

## 7. 恢复指引

任一旧仓全量恢复：见恢复位置 README（init bare + fetch bundle `refs/*:refs/*`）。
未提交载荷：按 manifest sha256 校验后复制回原路径（auto-ui Cargo.lock 的 work
副本即脏内容原字节）。legacy 锚被误动时以标签 `legacy-pre-v06-*` 重指。

## 8. 边界与后续

- 本计划未授权也未执行：push、远端操作、默认分支切换/更名、旧仓代码树替换、
  生产路径迁移。785/786 的入场条件之一（784 完成）由此收据交付面满足。
- 准备树/refs 显式转交后续迁移计划接管（merge 阶段记录）；本计划 merge 只清理
  auto-lang 执行树，不删除旧仓准备树与恢复 payload。
