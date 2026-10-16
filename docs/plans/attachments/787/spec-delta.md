# PLAN-787 拟议 Spec 增量（T-07 冻结稿，merge 时落 canonical）

对照计划 §"规范增量"表（SD-01..06）。本文件为 merge 输入；新模块名称为本仓
实验命名，不决定未来仓/产品品牌。

## SD-01 add docs/specs/auto-syntax/project.md

- **before**：无语法服务模块 Spec（783 的 A01/A02 为 Future、A03 为合成 ProbeSeam）。
- **after**：有限双 profile（auto-core / interface-mini-v1）语法服务：依赖方向
  snapshot→grammar-program/scanner→CST→结构查询；运行时不依赖 ac-core/
  Cranelift/auto-cache；Atom 配置为唯一规则源（编译期拒绝左递归/可空 repeat/
  未知引用/非前进 scanner）；实验边界（非完整 Auto 版 tree-sitter、非任意 CFG/
  regex、非动态插件、非 LSP）。
- **acceptance**：AC-01–06（V-01..V-13 全矩阵）。
- **证据**：verify 脚本 acceptance ALL PASS；matrix.md。

## SD-02 add docs/specs/auto-syntax/grammar-contract.md

- **before**：无规则程序合同。
- **after**：grammar-contract v1 DSL（token matcher 五类；步骤/repeat/field/
  same-line；Pratt 表+调用配置；newline/sep/sync 旋钮）；生成期拒绝定位到节点
  路径；GrammarId=名称+canonical+四版本 hash；同名不同内容隔离；codec 往返
  身份绑定；关键字/优先级配置反控（V-02）。
- **acceptance**：AC-02（V-02/V-03）。

## SD-03 add docs/specs/auto-syntax/snapshot-cst.md

- **after**：16KiB 内容寻址页+长度前缀目录（头部插入后缀页 hash 复用）；UTF-8
  边界拒绝+UTF-16 代理中点 floor+CRLF 行内策略；分页行索引；树会话/lease/跨代
  拒绝；POD arena+声明单元相对跨度；无损重建断言；恢复=吃进到 sync∪first（≥1
  token 前进或终止）；增量=at-rest checkpoint（词法粘连安全边界）+8-token 收敛
  +64KiB 窗+嫁接（error 声明重验证）；迟到/取消版本化发布；预算（显式栈/
  work_bytes 256MiB 默认）。
- **acceptance**：AC-01/03/04（V-01/V-04/V-06..V-08/V-12）。

## SD-04 modify docs/specs/auto-ac/project.md

- **before**：783 沉淀的探针导航（A00–A12，A01/A02 Future、A03 合成）。
- **after**：补 experimental/ac-syntax 导航节（双 profile 语法实验、core 投影
  adapter 仅对照不替换编译入口、生产 parser 零切换）；链接 auto-syntax 三份
  新 Spec。
- **acceptance**：AC-06。

## SD-05 modify docs/specs/auto-ac/artifact-contract.md

- **before**：A01/A02 Future、A03 ProbeSeam（合成造数）。
- **after**：A01/A02/A03 = Codec 且**真实生产者**（ac-syntax-artifact-adapter）：
  A01 语法 canonical 二进制（装载身份重验）、A02 token 流、A03 活子树 BFS 分页
  +目录分页+按需页视图（重启小查询禁调 parser）；新 ChunkKind ×4（加法；旧
  kind 字节与拒绝面不变）；required 边参与 783 GC pin（GC 算法零改动）。
- **acceptance**：AC-05（V-09..V-11）。

## SD-06 modify docs/specs/auto-ac/artifact-probe.md

- **before**：探针边界（冻结预算/协议）以合成 A03 为准。
- **after**：记录 787 加法扩展（四 ChunkKind、三层 coverage 翻转、Future 拒绝面
  换 A05 起）；GC 边界、staging 8MiB、64/48MiB 水位等冻结值不变；787 的
  syntax_budget_ 在同预算合同下运行（hot ≤8MiB 自有计量）。
- **acceptance**：AC-05/06。
