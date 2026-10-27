# PLAN-823 r8 规范增量候选（work 阶段，待验证和独立复审）

本稿是合入前候选，不修改 canonical Spec 或派生 ledger。代码基线为 `ea88414456bf89c98307c8fa2fe99a06dcaa1a5d` 加本轮尚未提交的工作区修复；最终提交、测试和依赖身份须在送审时重新绑定。下文关于历史回访、汇总脚本和跨工程身份的规则，必须先通过相应定向验证才能送审。

## 当前目标版本

| 目标 | 操作 | 主检出内容 SHA-256 | 目标最后提交 |
|---|---|---|---|
| `docs/specs/auto-av/closure-environments.md` | add | 不存在 | — |
| `docs/specs/auto-mir/closure-capture-profile.md` | add | 不存在 | — |
| `docs/specs/auto-av/cache-and-conformance.md` | modify | `F1E3D3D3010BCA1930119D28C5234918DAB14CFC1076789C8724ACF5D0222575` | `1e497ea48b4813a1850f88b3af4273d82a44b78c` |
| `docs/specs/auto-av/project.md` | modify | `61CC36CF38FD768DAE5B29DE1988E7D7B372A601A8CBF2A7881E48A9DFA2DEC3` | `1e497ea48b4813a1850f88b3af4273d82a44b78c` |
| `docs/specs/auto-test-case/architecture.md` | modify | `AB061AAA2E87DC5A47156192F2F1922F8C473311389352F3668BD7B255581BCC` | `1e497ea48b4813a1850f88b3af4273d82a44b78c` |
| `docs/specs/auto-av/resident-function-install.md` | modify | `C23BF7CFD62DD720388B3E2D34FD5C6D392C6CBDAC1E10DFAEB5A21DC25EC88F` | `1e497ea48b4813a1850f88b3af4273d82a44b78c` |
| `docs/specs/auto-test-case/ace-m1-conformance.md` | modify | `9F668F89F179EE63979D40B11EEA0C106F124302D0A1EA5808248F6BC2F405B6` | `81b4741458b276c509b24f960f3c7a0671703083` |
| `docs/specs/auto-ac/artifact-contract.md` | modify | `F3EEC456A5D7FCF779117A8951B65C3946D490ABE42BC450512C95D57E419A28` | `e3a233fbddef6076a0ed572fe1edc2ec82ffe46d` |

`supersedes_spec_components` / `new_spec_components` 对应上表；`touched_goals=[]`：有限实验 profile 不单独核销现有自举或产品目标。目标如有并发变更，须重新核对而不是覆盖。

## 拟议稳定规则

**SD-01 / AV 环境（AC-01/02/03/04）**：`av.closures/1` 是独立实验 profile；owned 环境由 move 转移或显式深 clone 构造，创建者退出后合法 owned 环境仍可调用。借用仅限验证器可证明的同步非逃逸路径；不支持隐式共享、跨线程、async、源码闭包扩面或 native ABI。构造失败保留未提交 move 源；部分构造、trap、取消及正常退出按实际所有权释放，环境字段不重复释放。已发布环境的身份与函数/签名/store 绑定；借用还绑定创建帧和槽代际。运行期句柄用于进程内保护，不持久化活环境或宿主地址。

**SD-02 / MIR 准入（AC-01/02/04）**：闭包体在函数单元声明 `env` 布局，`ENV_GET` 按声明定型；创建点的 capture 类型/模式与被调声明逐位一致，跨单元 callee 经 import，链接时核对声明及签名。环境拥有字段的本地别名沿 CFG may-alias 合流；普通 move 或聚合等不得洗掉字段所属关系，显式 clone 产生独立所有权，专用字段转移遵守读集/字段状态。错误索引、类型、状态及不支持 profile 受控拒绝，不以 panic 或未授信执行代替拒绝。必要类型/所有权/清理元数据不随 debug 剥离。

**SD-03 / 缓存恢复（AC-05/06）**：`plan823/av-closures`、`av.closures/1`、AVAD v4/profile-id 5、MIR/ABC/AV 语义版本 1/1/1 是独立身份；只缓存代码及可持久依赖，新进程从同 store 重新验证身份、payload、类型、entry 与 snapshot 后执行。好 hash 不能替代内部验证；未知/旧 profile 及不匹配身份在消费前拒绝。资源继续沿既有受管分区和本计划 worker 峰值口径，不以逻辑计数代替实测。

**SD-04 / AV 项目边界（AC-06/08）**：在实验 AV 项目登记闭包 profile 和两个细分规范入口，保留原有 scalar/data/owned/collections profile 与其消费者；不宣称旧 VM、默认产品、源码推导或 native/CTE 已支持闭包。

**SD-05 / 正式测试架构（AC-01/04/06）**：注册 `av-data-823` 独立 suite/recipe，沿已有父侧判决和受控 worker 链路比较结果与释放；坏结果/释放记录、缺 case/worker、协议或身份错误必须非绿。汇总验证应包含正式裁判、兼容与统一能力基线，不因零选择、跳过或漏必需项判通过。

**SD-06 / 常驻函数（AC-09）**：沿 828 受检单元和不可变 snapshot，未改函数沿用原对象/代际；更新先验证、链接、执行后提交，失败不覆盖旧版；活调用固定旧 snapshot，退出后由实际所有者释放。闭包 typed 会话的历史回访按已验证内容与 owner 命中，历史与当前内容受有界热区策略约束；重复当前内容的强制重建仍须使结构退化控制非绿。用户 2026-10-27 裁定本计划增量编译按 10 次实测、至少 9 次 ≤2000ms（P90≤2s）验收，全部正确性/结构/资源门仍成立。此规则不声称源码闭包增量或 native 在线加载。

**SD-07 / 852 映射（AC-11）**：保留 K01–K06、scenario R05/I01/02/04–07/10–12 的实际起点、独立 oracle 和缺口；typed MIR→ABC→AV 的闭包验证不能冒称相同 Auto 源码或 native 已支持。AV 旧调用暂停/恢复应按真实能力证据登记；native 在线加载与闭包语法/推导缺口保留 owner。修订历史回访、跨工程和父侧判决以实际检查结果为准，不把占位指针当通过。

**SD-08 / managed-store GC（AC-05/06/09/11）**：GC 在独占生存期锁内删除无工件引用、状态为 `deleted` 的已完成块元数据以及 `committed` 事务终态；保留 `tombstoned`、仍存活及非提交事务，确保中断恢复、受阻删除和再次发布不退化。真实 512 次不同编辑与周期 GC 须记录实际物理水位、用户哨兵和他方工件；根数量有界不等于物理空间有界。

该候选不扩大支持面；原 AC/V、单实例性能/资源门及 merge-owned 全档均保留。

2026-10-27 补充状态：本候选记录的 wire v4/profile 5、类型标签和指令编号与现已合并的 824 冲突；上表 canonical 内容版本和代码基线也须在 824 整合后重新计算。现阶段本稿仅供审阅差异，不能作为最终增量提交或发布依据。SD-08 的锁文件安全清理目前仅有 Windows 实测；非 Windows 并发/水位尚未闭合。798 门已依用户确认采用十样本 P90≤2s，整合后须对当前执行输入复验。
