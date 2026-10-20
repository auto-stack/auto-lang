# PLAN-815 M815-L T-17 实际恢复收据

2026-10-20。T-16(L3 pass,绑定 `0a65ee2a7`)核准后,在正确 workspace(主检出 `D:/autostack/auto-lang`)以单写者窗口执行生产账本兼容恢复。不归档、不清树;Plan 保持 executing。

## 1. 执行身份

| 项 | 值 |
|---|---|
| 核准审查 | review-l3 pass @ `0a65ee2a706a5c1320655c1874f14876f743bc3d`(§"核准身份"四值逐一匹配) |
| 恢复工具 | worktree `codex/plan-815-dev` @ `0a65ee2a7` 的 `scripts/plan815_ledger_recover.py`;worktree clean(L3 指出的未跟踪 `patch_l2c.py` 已删除,未带入交付) |
| 四重冻结(执行前实测) | source=`c24b174e…`(逐字未变)、schema=`c124c844…`、loader-probe=`555b1744…`、stage-increments=`aa93694e…` |
| 命令参数注入 | 由 convert-summary.json 程序化生成(手抄命令曾把 source 第 25 位 f 抄成 c,CAS 门当场拒绝——门与注入机制均按设计工作;拒绝记录见 apply-output.txt 首段) |

## 2. 执行结果

- **checkpoint A(结构恢复+裁定改名)**:v57→**58**,写入后真实 `SpecsStore::load` 身份核验通过。
- **checkpoint B(四条 M815-A 阶段增量)**:58→**59**,同上核验通过;最终指纹门通过。
- **终态**:`.autoos/specs.json` SHA256 = **`33830b65bd58aebe8a15f94fb71906f64b56883d89140697e6496f32d8475063`**(= 核准 checkpoint-B,逐字节)。
- **CAS 备份**:`specs.json.bak-recover-c24b174ee3c0eba2`(实测 SHA=原件 `c24b174e…`,不覆盖);freeze/ 逐字冻结副本同原件。
- 完整输出:[apply-output.txt](apply-output.txt)。

## 3. 读回 / 幂等 / 内容核验

| 门 | 结果 |
|---|---|
| 真实 loader 读回(生产文件) | version **59**、六区 Pascal type、**1098 项**([readback-production.txt](readback-production.txt)) |
| 幂等(同命令重跑) | **exit 2 拒绝**(源 hash 漂移),账本保持 B 逐字节不变([idempotence-check.txt](idempotence-check.txt)) |
| 内容 | 四条 P815-A 恰好各 1;P014-1-R 改名在场;designs/P014-1 保留 |
| 失败路径 | 本轮生产执行无失败;失败/并发/部分写/指纹门行为由 L1..L3 审查反例与自检绑定(同代码 `0a65ee2a7`) |

## 4. 恢复链保全(L3 §7.4 要求)

| 附件 | SHA256(实测) |
|---|---|
| conversions.json(12187 行,规范 hash=`c0a4aac0…`) | 文件字节 `d849a1301446d00952da35b132d43e1fcba739747ffd59a115203abb3d8b097d` |
| unknown-fields-sidecar.json | `9007a11c48800018a66089aacf012b1a0eb4024219a3122c058a1f6c8ae1b93e` |
| candidate-a-recovery.json(=写入的 A) | `d571dbe3f87289c180f7e0d8cbfbe519409162ec860ad8d069ccf9b71bef8f43` |
| candidate-b-final.json(=终态) | `33830b65bd58aebe8a15f94fb71906f64b56883d89140697e6496f32d8475063` |
| stage-increments.json | `aa93694e5743c2a68ea6a8affe60a33442ce4cc378d0c13af056908c2efdb919` |
| 逐字原件 | CAS 备份 + freeze/specs-v57-live.json(均 `c24b174e…`) |

注:以上 *.json 按库惯例(`.gitignore *.json`)留盘不入库;身份以本表 hash 钉定,恢复链完整可再验。

## 5. 语义边界(不变)

0=未知时间哨兵、mapped 状态不构成新验收主张、未知字段仅存 sidecar、历史原值全表可查(conversions.json)。**T-14 的 ledger_refreshed 由此闭合**;AC-12 完成,交回 T-14 关闭阶段发布。
