# PLAN-784 旧仓保全收据（repo preservation receipt）

> 生成：2026-10-15，work 阶段（plan-784-dev 执行树）；**r2 修订**：R1 三项已验证
> 闭合，R2 新增 F784-R2-01（快照身份遗漏 refs——同 HEAD 仅新增标签时复用旧
> bundle，恢复缺该标签）已修复：完整 refs→objectID 映射纳入状态身份
> （state_schema 2），创建/复用/复核三处断言 bundle 实际承载 refs 覆盖冻结映射。
> Category A：全程未改任何 Rust/构建输入，未运行 cargo 门禁。工具：
> `scripts/legacy-preservation.py` v2.1（inventory / preserve / anchor / verify /
> recover-drill / snapshots / drill）。
> 所有结构化证据在 [attachments/784/](../attachments/784/)；payload 在仓外独立
> 恢复位置（§2）。本收据只写元信息，敏感文件内容不入库。

## 1. 三仓状态冻结（T-01）

明细：[t01/inventory-auto-ui.json](../attachments/784/t01/inventory-auto-ui.json)、
[t01/inventory-auto-man.json](../attachments/784/t01/inventory-auto-man.json)、
[t01/inventory-auto-lang.json](../attachments/784/t01/inventory-auto-lang.json)。

| 仓 | 主检出 | 分支 | tip | WIP | stash | 其他 worktree | 适用指令 |
|---|---|---|---|---|---|---|---|
| auto-ui | `D:/autostack/auto-ui` | master | `f7d92eef273c`（衍星居士 2026-02-28，AURA pipeline） | ` M Cargo.lock`；ignored：`.claude/` | 0 | 无 | CLAUDE.md（其"master 已清空"表述过时——实测 `ls-tree HEAD` 满树） |
| auto-man | `D:/autostack/auto-man` | master | `51fafe82e8e0`（puming.zhao 2026-01-21） | 无跟踪改动；ignored：`Cargo.lock`、`.vscode/iar-vsc.json` | 0 | 无 | CLAUDE.md |
| auto-lang | `D:/autostack/auto-lang` | master | `7d51845ff`（含 Plan 787 草案簿记提交） | 主检出另有 783 复审会话的 docs/plans 簿记 WIP 与根目录游离 `ledger_770.py`（非本计划所有，未触碰） | 0 | lang-770/lang-781/lang-783/lang-784 | AGENTS.md |

refs 冻结：auto-ui 原始 8 条（master + origin/{HEAD,gpui2,gpui3,iced,master} +
tags v0.1/v0.2）；auto-man 原始 26 条（master + origin/{HEAD,master,v0.0.x} +
tags v0.0.1–v0.0.20 含 v0.0.4.fix/v0.0.18.1）。与计划快照（inputs.md）比对：
两仓 tip/WIP 一致。仅读本地 refs，未 fetch，不声称远端同步状态。

在途写入范围核对：781（executing 18/26，lang-781 树）、783（修复中，lang-783 树
+ 主检出簿记）、787（drafting 草案）；均与旧仓 refs/工作区无写入交集。

## 2. 保全载荷（T-02；r1 修订：状态寻址快照）

恢复位置：`D:/autostack/.recovery/784-legacy-preservation/`（三仓之外，含 README
与恢复指引）。**v2 起保全为状态寻址快照** `<repo>/snapshots/<snap-id>/`
（snap-id 由 tip+WIP+保留内容 hash 派生）：同态重跑幂等复用、状态变化新增独立
快照、既有快照永不被覆盖（F784-R1-01 修复）；`<repo>/LATEST` 指向最近保全。
**v1 遗留布局（首轮松散文件）原样保留**为已冻结恢复点，R1 复审已独立验证可
恢复，工具 v2 不读取也不改动。

当前快照 = **state_schema 2**（身份含完整 refs→objectID 映射；元信息副本
[t03r2/snapshot-manifest-auto-ui.json](../attachments/784/t03r2/snapshot-manifest-auto-ui.json)、
[snapshot-manifest-auto-man.json](../attachments/784/t03r2/snapshot-manifest-auto-man.json)）。
历代恢复点（全部保留、互不覆盖、各自可独立恢复）：

| 代 | auto-ui | auto-man | 说明 |
|---|---|---|---|
| v1 松散布局（2026-10-15 首轮） | bundle `c3ef028f…`（9 refs） | `2fc84d89…`（27 refs） | R1 复审独立验证可恢复；清单副本 t02 |
| v2 快照（r1，state_schema 1） | `efe2102a435580ae` / bundle `96fa5d3b…` | `1567f76ca3ecb7cb` / `7dfa47cc…` | R2 复审独立验证可恢复（11/29 refs）；清单副本 t03r1 |
| **v2.1 快照（r2，state_schema 2，LATEST）** | `2c2ec22a85dcea56` / bundle `a18a2d7c…` | `e143d51c828956ec` / bundle `7dfa47cc…` | 身份含 refs；11/29 refs 全恢复 |

保留载荷同前：auto-ui Cargo.lock work/head/diff + `.claude/settings.local.json`；
auto-man ignored `Cargo.lock` + `.vscode/iar-vsc.json`。注：同 refs 两次 bundle
打包字节可差（pack 非确定），故 snap-id 不含 bundle 字节、每代快照各持自己的
bundle 与 hash。

未提交/未跟踪内容不依赖 bundle 兜底——bundle 只含提交历史与 refs；字节级副本
即 V02 的恢复 oracle。ignored 其余项（构建产物类）仅列清单于 manifest，未备份。

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

## 4. 演练与反控（V01–V04；r1 修订后全量重跑）

r2 证据目录 [t03r2/](../attachments/784/t03r2/)（state_schema 2 含 refs 身份）；
r1 证据保留于 [t03r1/](../attachments/784/t03r1/)（state_schema 1）；首轮证据保留于
[t03/](../attachments/784/t03/)（v1 布局）——三代均仍为有效恢复点。

| 用例 | 结果 | 证据 |
|---|---|---|
| V01 disposable clone 恢复 | **pass**：两仓**当前快照（state_schema 2）** bundle → 临时 bare fetch `refs/*:refs/*`，快照冻结 refs sha 全等（auto-ui 11、auto-man 29，均含 legacy 锚+准备分支），tip tree 相等（`31ede719e46f…` / `4017c1c84e5b…`），旧 tag 全部可追溯（3 / 23）；临时 clone 已删 | [t03r2/v01-recover-drill-auto-ui.json](../attachments/784/t03r2/v01-recover-drill-auto-ui.json)、[v01-recover-drill-auto-man.json](../attachments/784/t03r2/v01-recover-drill-auto-man.json)（r1 代证据 t03r1 同绿） |
| V02 原字节与原工作区不变 | **pass**：verify 两仓 ok——载荷 hash 与快照 manifest 相等；**全部保留原文件（tracked WIP / untracked / ignored-keep）逐一存在性+hash 校验通过（F784-R1-02 修复后的完整 oracle）**；原工作区 status/HEAD/symref 与快照一致；保全 refs 无漂移；**bundle list-heads 实际覆盖冻结 refs（F784-R2-01 修复后 oracle）** | [t03r2/v02-verify-auto-ui.json](../attachments/784/t03r2/v02-verify-auto-ui.json)、[v02-verify-auto-man.json](../attachments/784/t03r2/v02-verify-auto-man.json) |
| V03 幂等/冲突/快照保留反控 | **pass 8/8**：D1 同 tip anchor 幂等 / D2 异 tip 拒绝且原 ref 不动 / D3 HEAD 漂移拒绝 / D4 同 HEAD 变 WIP=两个独立快照 / **D5 A→B 重复保全：旧快照目录聚合 hash 不变、A 字节可读、第三次同态重跑 reuse 不新增**（F784-R1-01 闭环）/ **D6 保留原文件全矩阵：ignored/wip/untracked 漂移与删除四反控全判红、恢复后复绿、备份字节不动**（F784-R1-02 闭环）/ D7 快照 bundle 损坏：同态重跑拒绝且不改写快照 / **D8 同 HEAD refs 新增（tag 指旧 bundle 不可达提交）/移动/删除矩阵：身份分裂、bare fetch 实际恢复断言（新 bundle 带该 tag、旧 bundle 无且字节不变、删除后正确复用初始态快照、不新增目录）、同态重跑 reuse**（F784-R2-01 闭环）；另真实仓 preserve 重跑=reuse、anchor 重跑=reuse | [t03r2/v03-drill-controls.json](../attachments/784/t03r2/v03-drill-controls.json)、[v03-anchor-rerun-auto-ui.json](../attachments/784/t03r2/v03-anchor-rerun-auto-ui.json)、[t03r2/v01-recover-drill-auto-ui.json](../attachments/784/t03r2/v01-recover-drill-auto-ui.json)、[v01-recover-drill-auto-man.json](../attachments/784/t03r2/v01-recover-drill-auto-man.json)、[v02-verify ×2](../attachments/784/t03r2/)、[snapshots-auto-ui.json](../attachments/784/t03r2/snapshots-auto-ui.json)、[snapshots-auto-man.json](../attachments/784/t03r2/snapshots-auto-man.json) |
| V04 清单/读取失败反控 | **pass**：777 审计对基线缺失/配置缺失/根缺失均显式 exit 3 失败不吞空；快照模型下另验证 preserve 对损坏 bundle/不一致快照 exit 4 拒绝零写入 | [t04/v04-negative-controls.txt](../attachments/784/t04/v04-negative-controls.txt)、t03r2 D7 |

## 5. 777 边界门刷新与迁出入口清单（T-04，首轮已交付，r1 未变）

- 冻结基线重锚 `7d51845ff`（39 声明边；777 归档时登记的"master 干净时重绑"
  常规运维在本执行树完成）；strict **pass**（0 违例 / 2 grandfathered）。
- 迁出候选路径→未来 owner→资源/生成物/测试/消费者→在途计划表：
  [t04/migration-inventory.md](../attachments/784/t04/migration-inventory.md)。

## 6. 三仓前后指纹

| 仓 | 前（T01） | 后（r1 修复后） |
|---|---|---|
| auto-ui | master@`f7d92eef273c`，` M Cargo.lock`，8 refs | tip/分支/WIP/status 全部不变（verify ok）；refs 面仅新增 legacy branch/tag 与准备分支；v1/v2/v2.1 三代恢复载荷并存 |
| auto-man | master@`51fafe82e8e0`，干净，26 refs | 同左不变；同上（v1/v2/v2.1 三代并存） |
| auto-lang | master@`7d51845ff` + 他计划簿记 WIP | master tip 不变；lang-784 组与 plan-784-dev 分支承载本计划交付物 |

## 7. 恢复指引

任一仓任一快照全量恢复：见恢复位置 README（init bare + fetch 该快照 bundle
`refs/*:refs/*`）。未提交载荷：按该快照 manifest sha256 校验后复制回原路径。
legacy 锚被误动时以标签 `legacy-pre-v06-*` 重指。v1 松散布局按其根 manifest
同法恢复（工具 v2 不再写该层）。

## 8. r1/r2 修复对账（F784-R1-01/02/03、F784-R2-01）

**两次 final 失败诊断（技能两连败检查点）**：R1（快照覆盖/漂移漏检）与
R2（身份遗漏 refs）同源——快照模型把"仓状态"窄化为 HEAD+工作区文件态，把
refs 当作 bundle 的派生元数据，既未进身份也未进复用/复核比较面；各轮修复都
闭合了被举出的反例，但未重审"状态↔manifest↔bundle 实际内容"的完整对应。
本轮覆盖调整：身份=HEAD+完整 refs 映射+全部保留内容 hash（state_schema 2）；
创建/复用/verify 三处统一接入"bundle 实际承载 refs（list-heads）覆盖冻结映射"
的真恢复 oracle；D8 以 bare fetch 实际恢复断言覆盖 refs 新增/移动/删除三类
状态变化。该 oracle 覆盖面以 manifest.state 与 bundle 实际内容的一致性为锚，
不再依赖任何单一反例。

| finding | 修复 | 永久控制 |
|---|---|---|
| F784-R1-01 重复保全覆盖旧备份 | preserve 重写为状态寻址快照：staging+原子 rename 落位、同态完整性校验通过才复用（零写入）、失败 exit 4 不动旧快照；真实仓 v1 载荷未动，新增 snapshots/ | D5（A→B 旧快照聚合 hash 不变+A 字节可读+同态 reuse）、D7（bundle 损坏拒绝不改写） |
| F784-R1-02 verify 漏检 ignored 原文件变化 | verify 对 tracked WIP/untracked/ignored-keep 原文件统一存在性+hash 校验，变化/删除判红非零退出，备份不动 | D6 全矩阵（ignored/wip/untracked 漂移+删除四反控+复绿+备份字节断言） |
| F784-R1-03 SD-01 三个链接失效 | `docs/specs/repository-boundaries/project.md` 三处 `../plans/`→`../../plans/`；三份交付文档全量本地链接检查 0 失效 | [t05r1/link-check.txt](../attachments/784/t05r1/link-check.txt)（3 文档 0 失效）；r2 复查 [t05r2/link-check.txt](../attachments/784/t05r2/link-check.txt) |
| F784-R2-01 快照身份遗漏 refs，仅新增标签复用旧 bundle | **完整 refs→objectID 映射纳入状态身份**（state_schema 2：仅 refs 变化即产生新快照；回到已见状态正确复用对应旧快照）；创建后、复用前、verify 中三处断言 bundle list-heads 实际覆盖冻结 refs（非 manifest 自证）；真实仓产生新代快照（ui `2c2ec22a85dcea56` / man `e143d51c828956ec`），v1/v2 代原样保留 | D8 全矩阵（新增 tag 指旧 bundle 不可达提交：新快照 bundle 经 bare fetch 实际带该 tag、旧快照字节不变；移动：bundle 中 tag 指新 sha；删除：复用初始态快照且 bundle 无该 tag、目录数不增） |

## 9. 边界与后续

- 本计划未授权也未执行：push、远端操作、默认分支切换/更名、旧仓代码树替换、
  生产路径迁移。785/786 的入场条件之一（784 完成）由此收据交付面满足。
- 准备树/refs 显式转交后续迁移计划接管（merge 阶段记录）；本计划 merge 只清理
  auto-lang 执行树，不删除旧仓准备树与恢复 payload（含 v1/v2 全部快照）。
