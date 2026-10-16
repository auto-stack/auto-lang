# PLAN-787 拟议 Spec 增量（T-07 修复轮冻结稿，merge 时落 canonical）

对照计划 §"规范增量"表（SD-01..06）。本文件为 merge 输入；新模块名称为本仓
实验命名，不决定未来仓/产品品牌。**复审 R1+R2+R3 修复轮**：按 F787-R1-01..10 + R2-01..11 + R3-01..06
修复后的实际实现重写（替代历次审查冻结旧稿——旧稿未获通过）。

## SD-01 add docs/specs/auto-syntax/project.md

- **before**：无语法服务模块 Spec（783 的 A01/A02 为 Future、A03 为合成 ProbeSeam）。
- **after**：有限双 profile（auto-core / interface-mini-v1）语法服务：依赖方向
  snapshot→grammar-program/scanner→CST→结构查询；运行时不依赖 ac-core/
  Cranelift/auto-cache；Atom 配置为唯一规则源（编译期拒绝左递归/可空 repeat/
  未知引用/非前进 scanner）；实验边界（非完整 Auto 版 tree-sitter、非任意 CFG/
  regex、非动态插件、非 LSP）。
- **acceptance**：AC-01–06（V-01..V-13 全矩阵）。
- **证据**：verify 脚本 acceptance ALL PASS（16 族 90 项）；matrix.md。

## SD-02 add docs/specs/auto-syntax/grammar-contract.md

- **before**：无规则程序合同。
- **after**：grammar-contract v1 DSL（token matcher 五类；步骤/repeat/field/
  same-line；Pratt 表+调用配置；newline/sep/sync 旋钮）；生成期拒绝定位到节点
  路径；GrammarId=名称+canonical+四版本 hash；同名不同内容隔离；codec 往返
  身份绑定；关键字/优先级配置反控（V-02）。
- **acceptance**：AC-02（V-02/V-03）。

## SD-03 add docs/specs/auto-syntax/snapshot-cst.md

- **after**：16KiB 内容寻址页+长度前缀目录（头部插入后缀页 hash 复用）；**页
  由快照目录 `Rc` 真实保活——活快照压力下不丢页，快照释放后页可回收**；
  UTF-8 边界拒绝+UTF-16 代理中点 floor+CRLF 行内策略；分页行索引（**跨页
  续行行起点回溯——列号从真实行起点计**）；树会话/lease/跨代拒绝；**CST =
  不可变共享内容块（CstBlock）+长度目录（bases）+根孩子目录**——节点 POD、
  块内边为局部下标、块跨树 Rc 共享；声明单元相对跨度+块内声明序号归属；
  无损重建断言；恢复=吃进到 sync∪first（≥1 token 前进或终止）；增量=at-rest
  checkpoint（词法粘连安全边界）+8-token 流式收敛（**scanner 硬限窗口、收敛
  即停——成功路径真实读取有界，重扫计量=实际扫描位置**）+**嫁接=旧块整体
  共享采纳（零节点复制；前缀身份映射）**；迟到/取消版本化发布；预算（显式
  栈/work_bytes 256MiB 默认）。
- **acceptance**：AC-01/03/04（V-01/V-04/V-06..V-08/V-12）。
- **修复轮证据**：反例 7/7；1MiB 中部 body 编辑内容块节点复用
  1147999/1148131（99.99%）、新节点 132、真实源页读取 ≤48KiB。

## SD-04 modify docs/specs/auto-ac/project.md

- **before**：783 沉淀的探针导航（A00–A12，A01/A02 Future、A03 合成）。
- **after**：补 experimental/ac-syntax 导航节（双 profile 语法实验、core 投影
  adapter 仅对照不替换编译入口、生产 parser 零切换）；链接 auto-syntax 三份
  新 Spec。
- **acceptance**：AC-06。

## SD-05 modify docs/specs/auto-ac/artifact-contract.md

- **before**：A01/A02 Future、A03 ProbeSeam（合成造数）。
- **after**：A01/A02/A03 = Codec 且**真实生产者**（ac-syntax-artifact-adapter）：
  A01 语法 canonical 二进制（装载身份重验；可按（grammar-id, revision）从
  store 装载——跨进程恢复不编译 .at）；A02 token 流+**完整 scanner checkpoint**
  ——终态按末 token 拼写**重放**还原（嵌套注释真实深度/字符串转义挂起/
  mode），恢复续扫与 fresh 词法尾部全等；键含 grammar 身份（跨 grammar 真读
  拒绝）；**A02 --required--> A01+源页**。源字节以**内容寻址 SourcePage**
  持久化（快照 16KiB 页聚合为 ≤256KiB store 源页；跨快照/工程共享），manifest
  记源页表可流式重建原文。A03 页=内存内容块同构序列化+目录分页（4096 条/页，
  framed ≤512KiB）+按需页视图（重启小查询禁调 parser；视图 lease）；
  required 闭包（当前根 GC 存活覆盖可重开全数据）：put_cst 链 = manifest
  --required--> 数据页+目录页+A02（→A01+源页）；流式链 = manifest
  --required--> 数据页+目录页+**源页+A01**（流式无 A02 工件——直接闭包，
  不以 A03→A02 链解释流式保活）。**发布仲裁**：manifest 携带 revision，
  迟到旧任务（低 revision）只入库不发布——当前根不被覆盖；流式根按工程
  隔离。新 ChunkKind ×5（加法：SyntaxGrammar/TokenStream/CstPage/CstManifest/
  SourcePage；旧 kind 字节与拒绝面不变；GC 算法零改动）。
- **acceptance**：AC-05（V-09..V-11）。

## SD-06 modify docs/specs/auto-ac/artifact-probe.md

- **before**：探针边界（冻结预算/协议）以合成 A03 为准。
- **after**：记录 787 加法扩展（五 ChunkKind 含 SourcePage、三层 coverage 翻转
  （dep_audit 期望同步）、A02 key_inputs 补 grammar-id/page-hash、Future 拒绝面
  换 A05 起）；GC 边界、staging 8MiB、64/48MiB 水位等冻结值不变；787 的
  syntax_budget_ 在同预算合同下运行（builder 受管 ≤8MiB + hot ≤8MiB +
  manifest 索引计入 16MiB 合同；Windows 实测 OS private bytes 增量 ≤64MiB，
  非 Windows 宿主显式记录证据缺口）。
- **acceptance**：AC-05/06。
