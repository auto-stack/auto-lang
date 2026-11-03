# Native 分配故障原子性与受检 Place 一致性（PLAN-923）

> **Status**: experimental（真实 native collections PE 上的逐 seam 故障注入与
> 受检 List-place 机器链；不改变产品默认引擎、不改变原 source 语义）

## 故障 seam 与注入契约

真实可达分配图冻结为 9 个 seam（S0 全局兼容 + S1..S8），每 seam 独立计数、
独立注入，不双计同一字节：

| seam | 名称 | 分配点（collections-runtime） | 账类 |
|---|---|---|---|
| S1 | node-header | new_node Node 结构 | guest 逻辑 attempt |
| S2 | data-buffer | new_node List/Record 初始 data（含每元素 gen 数组） | guest 逻辑 attempt |
| S3 | grow-buffer | List insert 扩容新 buffer | guest 逻辑 attempt |
| S4 | frame-cells | frame_open 帧槽 | runtime bookkeeping |
| S5 | clone-work | push_clone_work CloneWork 节点 | runtime bookkeeping |
| S6 | text-backing | text_alloc 的 text 数据（arena） | guest（text hook 显式处理） |
| S7 | text-meta | text ledger 必要 metadata（text-runtime 内部） | runtime |
| S8 | token-alloc | Place token 注册表分配 | runtime bookkeeping |

- 注入入口 `mn_coll_fail_nth_seam(seam, n)`（S1..S5/S8）；S6/S7 走嵌入
  text-runtime 自身通道 `ac_rt_fail_nth_alloc`/`ac_rt_fail_ledger_nth`
  （ERR_ENV_OOM → trap 22）。S4 frame-open 折叠进通用 null 路径，其 trap
  码为 RESOURCE(22)，由 `MN-COLL-INJECT seam=/nth=` 事实行区分注入与真实
  预算拒绝；其余 seam 注入命中 trap INJECTED(25)。
- 错误分态：attempts 在 allocate 入口 +1；precheck 失败该 attempt 记
  RESOURCE(-22)，fail_nth 命中记 INJECTED(-25)；取消 -26；STALE -27
  （Place 代际）。注入不是正常语言负例——不能用任意 exit70 或父 kill 满足。
- 每次注入写具名事实行 `MN-COLL-INJECT seam=<N>` + `nth=<N>`（成对，
  无 seam 的 nth 或无 nth 的 seam 是坏事实）；每 seam 逐 1..N、N+1 正控
  （N+1 不失败），清 fault 同进程重试正确。已实际 commit 后的错误不谎称
  未 commit。

## 原子性与清理

- 保持源/旧目标 logical 值、所有权/handle 有效性、List len/cap/元素、
  epoch/gen 与旧产物可用；prepare 所有 fallible 工作再 commit，未提交
  move 不 kill 源。
- clone/扩容分配旧新同时驻留计预算（precheck 双计），失败部分逆序/工作栈
  清理；drop 不轮询已耗尽燃料/取消。半树部分成功仍无泄漏；合法 view/外部
  owner 还活时不得清掉 backing；最后 owner 后 node/text/arena
  committed/ledger 全归基线。
- 两腿证据：source 全流程 PE（逐 seam 注入，语言 trap 终止）+ 同 runtime
  原语受控 probe（持合法源/目标比较前后快照，清 fault 同进程重试）；probe
  不替代原 source 完整机器链，只重跑 source 不能证明中断原子性。

## 受检 Place 与访问前代际校验

- 共享 MIR 的 ListPlace/PlaceGet/PlaceSet 语义不变；native 准入冻结为
  Copy 元素（i32/bool）List place——List<Str>/owned record 的 Place 类型
  在 native 准入具名拒绝（Unsupported "place" "must be Copy"），不放开
  源码跨变更 view 逃逸，不冒全部 profile4 指令面。
- Place token 是受控注册表条目（arena S8 计费）：记录 list node bits、
  buffer 身份、structural epoch、index、元素 gen——**不保存会被先解引用的
  悬空 frame/list/buffer 指针**。访问前经 node registry 活验证（frame
  激活 UID、slot/owner、buffer 身份、epoch、index、元素 gen）再实际访问；
  计数/UID 溢出具名拒绝，不复用旧世代。
- 结构变更 push/insert/remove bump node.epoch，退役全部旧 token；in-place
  set/PlaceSet bump 目标元素 gen，废本 place 与兄弟 place。无影响元素/新
  token 保持合法（不能把全部旧 view 判坏）；预算/注入失败未 commit 时
  epoch/gen/有效 token 不变。
- 静态 loan 违规仍在 source/verify 前门拒绝；运行期 STALE(27) 只处理合法
  MIR 的过期 token，且必须在实际 buffer read/write 前拒（read-first/
  write-first 反控）。

## ABI / 布局 / 计量

- `mn.collections.runtime/2` → `/3`；`mn.collections.cell/2` → `/3`
  （Place 三指令发射）；layout/source/codegen 世代不变。旧 COFF 缓存随
  ABI 变化正确 STALE，不承诺旧工件命中。
- Node 增 `epoch: u64` + 每元素 gen 数组（cap 条，随 data buffer 分配，
  S2 seam 同计）；PlaceToken 注册表（arena 分配，S8 seam）。
- 预算沿用：guest 8MiB、native 物理 32MiB（新代际/token 表、text
  ledger/header/CloneWork 同计）、compiler managed 48MiB（16/24/8）、父
  PE private 128MiB；先限额后物化，观察溢出明确拒绝，不静默截断。

## 恢复 → 续编辑

`restore_collections_session` 复用完整校验链（manifest schema/profile/ABI/
target/producer/source identity、runtime build+content、source blob+坐标、
link tools、逐 unit MIR 深验 + COFF hash/len/pages、entry 闭包），随后从
恢复 source 经有界冷前端重建 env/regions/mint_origins，装配
`CollectionsIncrementalSession`，支持 prepare_source 的下一 body 编辑（真实
差额、暖未改 P0）。不恢复任何活 Place/guest 值；运行结果永不缓存。
