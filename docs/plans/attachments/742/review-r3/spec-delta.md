# PLAN-742 rev 1，第三次复审的规范增量冻结稿

reviewed_commit=`8c9b2d3d96d648e3a0ffa7c40b4c25694a6da85d`；
base=`6d93cfdb73f5178b624846d46f88614ed999c158`；沿用 SD-742-01～04、五个目标路径与 AC IDs。
新组件为空，目标仍 GOAL-003/GOAL-007；原合同未削弱，plan_revision=1。

| 增量 / canonical 目标 | 当前可核查规则与边界 | 裁决 |
|---|---|---|
| SD-742-01；`docs/specs/auto-man/design/api-generation-integrity.md`、`docs/specs/auto-man/project.md` | api.rs qualifier/borrow 链增加代码区保护；JSON/File/Upload 体统一接 scope helper；Option 变量运行时 Some/None 包装，void 提前退出补状态。旧 13 例通过，但 marker 恢复仍改合法字面量、嵌套借用漏处理。 | partial；R3-01/02 未闭合，不批准完整 parity 申报 |
| SD-742-02；`docs/specs/auto-lang/trans/overview.md` | List 构造/语句分割的单行 mask 保留；本修复轮未修改 RustTrans。只描述实际发射形态；不声称 raw string/多行块注释/任意未验 pattern 支持。完整 API 后处理链的字面量保持仍受 R3-01 影响。 | 旧例通过，完整端到端规则仍 partial |
| SD-742-03；`docs/specs/a2r-std/project.md` | clone 共享 SQLite 连接与事务，owned String 输入，dir_names 的直接排序名称/点名排除/失败为空边界不变。源码、依赖与前次指纹一致。 | 已测 Rust 控制证据可重用；不扩大为新 Notes 双端验收 |
| SD-742-04；`docs/specs/auto-lang/ui/overview.md` | 已检测主题+store 的首 store 从 bootstrap 同源初始化。Vue 源码未改；当轮 Vue/VM 联动在交接时即明确未完成。 | partial，work-owned oracle 缺失；不能用历史图或 S10 读回替代 |

e35f9e99b 的历史知识保留，本轮不发布 canonical 或 live ledger。
db.rs 的旧正则后处理是 work 提出的邻接候选，本轮未运行独立反例，
不能将其源码观察写成确定故障，也不能将疑似边界写成已完成规则。
两连败后的诊断已按步骤核查，但“全链保持”和“完整借用覆盖”仍需依据反例修正。
