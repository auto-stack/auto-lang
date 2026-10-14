# PLAN-742 rev 1：第四次复审规范增量冻结稿

reviewed_commit=`2c40e1fe4673c2041ae3c366037d7eab38017d81`；
base=`8c9b2d3d96d648e3a0ffa7c40b4c25694a6da85d`。
沿用 SD-742-01～04、原五个规范目标与 AC IDs，无新组件，目标 GOAL-003/007。

| 增量 / 目标 | 实际行为与仍需闭合的规则 | 本轮裁决 |
|---|---|---|
| SD-742-01；`docs/specs/auto-man/design/api-generation-integrity.md`、`docs/specs/auto-man/project.md` | 取消 placeholder 恢复，代码区单趟优先级限定化；borrow worker 对参数递归，内层按签名借用；上一轮17例全部通过。扫描仍可在较长标识符的后缀命中a2r_std，把custom_a2r_std调用改错，需真正的token/模块身份边界。 | partial；R4-01 未闭合，不能批准完整限定/调用 parity |
| SD-742-02；`docs/specs/auto-lang/trans/overview.md` | List构造/语句分割mask不变，宿主发射路径限已验形态；嵌套借用两层/三层正例实编运行。合法模块名保持仍受R4-01影响。 | 已验形态通过，完整API路径仍partial |
| SD-742-03；`docs/specs/a2r-std/project.md` | SQLite clone共享连接/事务、owned String输入，dir_names排序/略点名/失败为空规则不变。独立S10有强杀、下线、同制品重启、同目录/DB指纹及值读回。 | 原Rust控制可重用；S10当前制品独立通过，不扩大为VM端UI交互 |
| SD-742-04；`docs/specs/auto-lang/ui/overview.md` | 首store按bootstrap种子同源初始化；独立Vue主题/CRUD smoke13/13通过。VM标准驱动45秒未获得就绪MCP，未签交互/视觉一致性或19场景通过。 | Vue范围通过，VM交互边界保留 |

VM19不是本计划新追加的固定数量门禁；仍不能把启动日志称为已经做过交互。
原验证清单的Vue auto run/legacy/v1电池、当前双端联动的实际证据范围分别记录，不静默删去未验项。
规范增量待真实限定化不变量闭合后再定稿；不改canonical或live ledger。
D-742-1继续另案。db旧regex及未实编Upload范围保留原调查身份，不冒称当前完整覆盖。
