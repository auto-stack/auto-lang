# PLAN-817 acceptance matrix (T-08)

每行证据绑定 worktree 提交（plan-817-dev）：T-02 `b249767d0`、T-03 `ef76a0b4a`、
T-04 `6d5e44ac3`、T-05 `6c92decfb`、T-06 `fa4f39c25`、T-07 `5b8abb004`（含后续 fmt/收尾提交）。
命令：`powershell -File scripts/verify-ac-cte-817.ps1 -Scope <scope>`。

| V | 内容 | scope/命令 | 测试族/证据 |
|---|---|---|---|
| V-01 | 真实闭环 sum42/bool/分支翻转、native 独立 expected | first-case | cte_first_ 6（native exit 42、bool 分支 2→1、HIR 常量证明）；正式语料 query-sum42/bool |
| V-02 | 顺序/阶段：未走 trap 潜伏、已走溢出含调用来源、consumer 未完成不抹执行事实 | schedule | cte_schedule_（latent/executed/helper-chain trap；execution-precedes-consumer） |
| V-03 | 环定位/合法 DAG/普通递归/fuel/取消分态 | schedule | cte_schedule_（全链闭合边诊断、单飞 DAG、递归合法、资源类、取消隔离+不发布） |
| V-04 | 信任：模型不冒领、坏块受控拒绝、未知 callee/caps AV 前 0 执行 | admission+freeze | cte_admit_ 9 + cte_freeze_（producer 隔离语、五类坏块） |
| V-05 | 真缓存 fresh→hit→新进程/跨工程、AV 禁调反控 | first-case+freeze | cte_first_（store 恢复新进程、跨工程、禁调）；restart 语料/adapter |
| V-06 | 依赖失效 body/签名/caps/负依赖/粒度复用 | cache | cte_cache_ 6（回滚命中旧条目、无关面 hit、meta 负依赖重键） |
| V-07 | 坏块/lease/GC/root 生命周期 | freeze+resources | cte_freeze_（lease/root/哨兵）+ cte_resources_（真实水位物理删除/收敛） |
| V-08 | 32MiB/512 修订/预算预置/spike/worker 树门 | resources | cte_resources_ 6（台账真红、深解码界限、384/512MiB） |
| V-09 | 六场景×30、两编辑 P95<1s、阈值 0/坏样本红 | turnaround | cte_turnaround_ 3（gate 4.3/1.5ms；raw hash 绑定） |
| V-10 | 正式 runner：未知 recipe/身份/stale/restart/ERROR 不绿 | autodown | 官测 5/5 + cte_adapter_ 3 + lint |
| V-11 | 边界审计：零他方写面/注册只追加/无 reparse point | closeout | verify 脚本 closeout scope（git diff 审计） |
| V-12 | 828 snapshot 车道四子case：生产观测/身份/依赖/失败控制（R1 修复后） | snapshot | cte_snapshot_ 9（含 115 函数/≥256KiB 驻留基线 30 修订：main 零重编译/无重整/P95 门；强制重建反控非零；**F-01 多 callee 正确性**） |
| R1 修复 | 复审反例重放 | — | two-callees 重放 x=2,y=12 ✓；--samples 0 → exit 2/target_met=false ✓；worker-edit 真进程全窗口；Meta→AV 官方用例绿 |

AC 绑定：AC-01=V-01/05/10、AC-02=V-01/02/04、AC-03=V-02/03、AC-04=V-04..07、
AC-05=V-07/08、AC-06=V-09、AC-07=V-10、AC-08=V-11、AC-11=V-12（SD-06）。
