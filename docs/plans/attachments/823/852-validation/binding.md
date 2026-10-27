# PLAN-823 T-11 — 852 统一能力验证 phase：binding 与证据

2026-10-27，work stage T-11（D852 = archived，landed 早于本树基面）。本文件按
[852 共同合同](../../reports/ace-852-baseline-and-capability-validation-2026-10-26.md)
§3 的 823 行执行逐行映射：**语义ID → 实际输入起点 → 具体V/子case → 正式recipe →
独立oracle → 证据**；binding.tsv 仅翻自有格（`wait:av-closures-823` →
`ok:av-closures-823`，7 行）+ 追加指针，schema/金样/他方行零改。

## source K01~K06（typed 证据关 av 层行；源码列保持 M1 缺口）

| ID | 特征 | 实际起点（真实 recipe 入口） | V 子case | 独立 oracle（手算） | 状态 |
|---|---|---|---|---|---|
| K01 | move闭包捕获 | `av_closures_823_first_case`（move 捕获 List\<Note\>，verify→ABC v4→AV）+ harness `t10_move_source_preserved` | V-01/V-05 | 结果 29；源保留可重试 | 执行 ✅ |
| K02 | 创建者退出后owned闭包 | `first_case`（make 返回→main 新帧调用→drop）+ `cache_compat restore`（新 worker 重建） | V-01/V-08 | 29/5alloc 5free/live=0；恢复同果 | 执行 ✅ |
| K03 | 同步非逃逸借用捕获 | `semantics borrow_legal_sync_call` | V-02 | 8（env view 4+源 view 4）；钉住纪律=静态前门 | 执行 ✅ |
| K04 | 借用闭包逃逸拒绝 | `semantics borrow_escape_rejected`（Return 逃逸/非调用消费者/源钉住/非 Str 域四负控） | V-02 负控 | 真实 verify 拒绝码 | 执行 ✅ |
| K05 | 闭包环境显式深clone | `semantics closure_clone_independent_env` | V-06 | 区分性 oracle **80**（共享 env 会得 90） | 执行 ✅ |
| K06 | owned闭包可变环境 | `first_case`（共享 List 句柄 push，len 1→2） | V-01/V-06 | 2*10+4+5=**29** | 执行 ✅ |

源码闭包语法/推导、native closure ABI、native 代码生成：**不因此加入 823**——
binding.tsv 源码列（ac/avm/a2r 的 gap/diff 与 `snapshot-only-cert-gap`）原样保留，
owner=a2r composite lowering / 823-future-source，M1 行不冒领。

## scenario R05 / I 系

| ID | 特征 | 实际起点 | 状态 |
|---|---|---|---|
| R05 | closure创建者退出 | `first_case` + restore（=corpus K02 配对） | 执行 ✅ |
| I01 | 稳定body编辑 | `incremental sub2`（身份/Arc::ptr_eq/生产零 IO）；30 修订全谱仍为 852 T-05 缺口 | 指针 ✅（全谱缺口保留 owner） |
| I02 | callee-only-body编辑 | `incremental sub2`（语义 5/8/13） | 指针 ✅ |
| I04 | type/layout改变 | `incremental sub3`（env 布局漂移=精确 link 拒绝+会话存活） | 指针 ✅ |
| I05 | 改回历史内容 | `incremental_history_revisit_hits`：body 1→2→1，原代码对象与代际复用、重新准入次数为零；`incremental_history_bound_and_failed_publish` 检查淘汰和失败发布 | 定向实测 ✅；运行汇总及复审待完成 |
| I06 | fresh/hot/重启/跨工程 | `cache_compat` fresh/new handle 重验；`cache_cross_project_content_identity` 在同一store以不同project发布相同工件、各执行29并重新验证恢复 | 定向实测 ✅；该测试不声称第二次请求跳过编译或发生缓存命中 |
| I07 | AV旧调用暂停/恢复 | `incremental_paused_call_pins_old_body`：闭包活帧暂停后安装新body，新调用=8、旧调用=5，资源释放与旧代码最后所有者释放 | AV定向实测 ✅；native在线加载仍属范围外缺口 |
| I10 | 512修订churn+GC | `incremental_store_512_revisions_gc_bounded`：512次不同body持久编辑，周期真实GC、会话根≤34、他方工件/用户哨兵保留、关闭清根 | 执行 ✅（closures typed轴；raw见主检出修复附件） |
| I11 | 单函数编辑 P90 | `incremental_warm_edit_p90`：10次 typed warm 会话修订，至少9次≤2000ms；不代替正式源码端到端门 | 已执行，最终提交证据待冻结 |
| I12 | 测试灵敏度负控 | `recipe`（judge 六类谎言必红）+ `incremental sub4`（强制退化门非零） | 指针 ✅ |

## 输入/版本身份

profile `av.closures/1`；ABC wire v4 / profile-id 5 / tag16 / opcode 0x2F-0x31；
MIR/ABC/AV semantics (1,1,1)；producer `plan823/av-closures`；基面 = 本树
（D852 已合入后）。零 a2r 认证、零 Rust 重编译（合同 §5 日常策略）。

raw 收据见 [raw.md](raw.md)。
