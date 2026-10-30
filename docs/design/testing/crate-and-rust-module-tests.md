# 基于Cargo包与Rust模块的固定测试分组

> target-state r1，2026-10-31；用户提出替换tm体系，实施合同为[PLAN-896 r4](../../plans/896-merged-invocations-and-consumer-closure-audit.md)，吸收897；尚未实施。当前Specs仍描述旧机制。

## 决策与问题

不再将全仓原子义务动态划成叶、再归并顶级模块并自动展开消费者图。该体系把元数据新鲜度、全工程发现/构建、依赖推导和逐叶执行耦合；一次局部修改可能先编译全部实验工程，再串行启动数百次测试。

采用源码已有边界：第一层Cargo package/target，小crate可整体执行；大crate内第二层使用真实Rust模块，如parser、vm、trans::rust。配置写死少量职责组，多个现有测试命名空间可共同保护一个职责；不按用例数量或每分钟目标继续切细叶。

分组配置不是逐用例目录；Rust/nextest正常列举选中目标测试仍存在，也可能需要构建。去掉的是事前全仓发现与自建新鲜度/闭包系统，不是Rust原生测试发现。

## 使用方式

拟`cargo t parser`、`cargo t parser vm`；一份静态配置展开package、target、features、profile、测试namespace和必要工具。命令直接运行，不要求先list。`cargo t --list`只读固定组名，不构建。无参数保旧日常选面，扩大默认包另作明确决定。

同执行配置的组交nextest一次并行调度，包/binary限定后OR过滤，结果自然去重。HTTP、共享资源、重内存和性能门的串行/截止保持。Python治理文件和TestCase已有suite按现成入口直接调度，保原oracle和包装check，不再套叶分片。

## 影响判断

计划根据真实修改写固定组和必要集成组，review核选择理由。parser局部算法修改以parser完整组及相应集成为主；共享AST/语法/IR契约可能明确加vm、a2r等。Cargo编译依赖不意味着要运行所有依赖方单元测试；反过来，“只改parser”也不能自动豁免已知受影响行为。

本轮不建立另一套静态消费者图/无限递归推导器。范围未知时明确解释、必要时运行原固定日常包；涉及ACE才显式选相关包/工具，不默认转全实验工程构建。周期回归继续保原全局发现/覆盖责任与owner。

## 编译边界与存储

模块过滤缩小运行范围，不使同crate的测试二进制按mod独立编译。新测试/宏/cfg/build.rs变化仍由Cargo处理。进一步物理拆crate或VM子组不属于本轮。

ACE宿主接线从通用test-cases解耦到显式feature/target，普通VM/legacy语料可运行而不准备ACE worker；真正相关ACE任务显式启用。必要experimental普通dev/test debug=0，完整调试debug=2显式保留，不改正式性能输入。同树兼容target复用，保regular/batch、triple/features/profile/锁和发布工具来源；不跨树共享可写目录。

安全清理先预览、按owner和活动/测量预留保护，原始证据与缓存分开，拒绝reparse逃逸。不得清他方活动现场或把可重建target搬入证据目录。实际容量与构建/运行成本按同工作量测量，不预先保证50秒或空间减半。

## 迁移与批准例外

旧tm目录只保历史/迁移审计价值，不在新日常路径读取；旧叶ID退役，不保两套执行器。一次真实枚举对账证明每个原测试/包装check/条件/oracle的去向，重叠职责取集合并集不要求不相交。之后新增同namespace测试直接由原生runner纳入，无需登记逐ID。

本仓t/tv/tt/tb/th、周期驱动、治理脚本、AGENTS及文档同步承接，外仓维持owner入口，在途计划记录准确迁移交接。退役tm时明确报错和迁移说明，不默默跑全量。

AAVM/AA2R是已获用户批准的唯一新增默认ignore范围：保实现/测试/夹具/期望和手动重放，按真实实验身份而非名字子串或全部parity关闭；普通VM/Rust A2R及实际产品消费验证保留。其他研究/专项没有获得批量停用授权。

## 交付顺序与证据

先静态lexer/parser最小真实链，再完整职责归组、迁移守恒与旧入口退出；同时落实必要ACE隔离/profile/target及自有闲置缓存迁移。固定完整组对照旧同选面一次nextest运行；报告准备/编译、实际执行、整命令、CPU/内存/存储，ignore减少工作另列。

896为唯一执行计划/权威worktree；897旧合同保历史并登记被吸收，不独立实施。不以规划合并冒称交付完成。设计/计划待用户确认，不修改当前实现或清理缓存。
