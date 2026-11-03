# auto-syntax snapshot & CST 合同（坐标/词法/无损树/增量/恢复）

> **Status**: experimental（PLAN-787 冻结；实现=`experimental/ace/crates/syntax-runtime`）
> 来源：PLAN-787；上游模块 [project.md](project.md)；持久化接缝见
> [../auto-ac/artifact-contract.md](../auto-ac/artifact-contract.md)。

## snapshot 与坐标

- 16KiB 内容寻址页 + 长度前缀目录（头部插入后缀页 hash 复用）；页由快照
  目录 `Rc` 真实保活（活快照压力不丢页，释放后可回收）。
- UTF-8 边界拒绝 + UTF-16 代理中点 floor + CRLF 行内策略；分页行索引
  （跨页续行行起点回溯，列号从真实行起点计）；无需全文扫描即可转一次位置。
- 树会话/lease/跨代拒绝；handle 含树身份，跨树使用受控拒绝。

## 词法与 checkpoint

- checkpoint = scanner **实际状态**（offset/mode/未闭合 mid token 含其**原始
  起始行**/line），不是 EOF 位置形状；恢复续扫与 fresh 词法尾部逐 token 全等。
- 持久化终态按末 token 拼写重放还原（嵌套注释真实深度/字符串转义挂起/mode）。
- PLAN-927（T-04 起 token.line=**结束行**——跨行 token 已计入内部换行）：
  checkpoint 终态推导——EOF 行=末 token 行（不加算 token 内换行）；未闭合
  mid 的窗口初始行=offset 处行（EOF 未闭合 token 的 offset==t.end 即结束行），
  `MidToken.start_line` 保起始行（F787-R3-04 错误叶路径）。
- **机器回退纪律（D-927-1 修复）**：`Save` 携带 pending trivia **内容**
  （Vec）而非长度——长度截断在 drain/重填后恢复过期前缀（丢语句分隔换行/
  重复 trivia 叶）；`expr_anchor`/`same_line_pending` 一并入 Save 随回退
  恢复（否则 alt 回退丢行纪律）。尾随未解析 token=ERROR 叶（非 trivia 保
  trivia 标志），不被投影当干净内容静默丢弃。
- 增量重扫 scanner 硬限窗口 + 流式收敛即停（成功路径真实读取有界，
  计量=实际扫描位置）。

## 无损 CST

- CST = 不可变共享内容块（CstBlock）+ 长度目录（bases）+ 根孩子目录；节点
  POD、块内边为局部下标、块跨树 `Rc` 共享；声明单元相对跨度 + 块内声明
  序号归属；封块只在声明边界（声明子树永不跨块），封块即 shrink_to_fit
  （倍增容量松弛是真实驻留）。
- 叶节点覆盖源码 token/trivia/error 字节；missing 零宽携带期望 kind；按文档
  序流式重建逐字节等于 snapshot（assert_lossless）；构建/查询/回收全路径
  显式栈迭代（深链不爆栈，无固定句法深度上限，预算控制）。
- **零宽坐标合同**：空产生式（epsilon，如空 args）由生产者放在已消费 token
  终点（零宽但 start 为真实坐标）；伪造 start=0 占位形零宽 child 在父外仍
  拒绝——合法 epsilon 与坏跨度可区分。
- 恢复 = 吃进到 sync∪first（≥1 token 前进或终止）；未闭合/坏声明返回当前
  revision partial tree + 诊断，可查询后续合法声明。
- strict 投影：error/missing/不支持构造受控拒绝（投影经迭代引擎
  projection_engine.rs，正常线程栈深链/真嵌套/调用范围通过）。

## 增量与迟到

- 嫁接 = 旧块整体共享采纳（零节点复制；前缀身份映射；reused_decls=实际
  嫁接计数）；回退（跨声明分隔/未闭合词法状态）记录原因且结果与 fresh 全等
  （canonical 比较含字段/跨度/flags；诊断全字段）。
- 1MiB 中部单 body 编辑：内容块节点复用 ≥99.99%、真实源页读取 ≤48KiB；
  200 轮确定种子混合编辑逐步 fresh 等价。
- 迟到/取消：修订仲裁适用域=编辑会话（见 artifact-contract 发布仲裁）；
  旧 revision 任务晚完成只入库不覆盖当前根/诊断；显式事件屏障复现逆序完成。

## 会话驻留消费（PLAN-828）

增量编辑/重扫（`incremental_reparse`+`project_decl_strict` 单声明严格投影）被 828
会话作为唯一编辑热路径消费：未改声明不重新投影（`project` 生产者计数对改函恰一；`syntax_reparsed_decls`/reused
单列有界词法/声明重扫，邻接声明重扫不等于重新投影其 body）；树/快照/页合同不变。scalar 表面由 `grammars/auto-scalar.at` 版本化承载，
与 ac-core scalar parser 的 AST+span 全等由 `scalar_projection_` 差分钉死——投影差分
是准入面，不支持面不减。
## 验证锚

snapshot_/generation_/lossless_/recovery_/core_projection_/depth_/
incremental_local_/incremental_replay_ 族；预算口径见
[../auto-ac/artifact-probe.md](../auto-ac/artifact-probe.md) syntax_budget_ 节。
