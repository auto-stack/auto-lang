# 不相交叶模块与影响选测

> PLAN-888 r2；成本与交付状态以计划验收/归档收据为准。

## 身份与目录

原子义务、唯一叶模块、调度组、命令分别计数。运行时发现语言旧九配置身份并集、ACE十个
workspace、正式TestCase suite及本仓Python治理控制；外仓UI/Man仅保留owner引用。
`scripts/test-governance/modules.json` 索引其内容哈希分片，每条运行身份恰属一叶，分区并集
守恒。recipe/后端/profile/oracle/平台条件不同的身份不可误去重。

原聚合输入另绑定原source/oracle及真实消费记录，包装exit0不能替代子项完成。
VM缓存及A2R/C/TS/Cookbook包装由真实Rust AST识别原helper调用，当前1,066条原包装/输入
关联由legacy-wrapper-contract.json冻结；刷新不得同时抹去关联。声明、当前注册、原ignore和
实际完成分开。消费端要求每个适用绑定的原始终态记录、身份/内容指纹和条数，缺失/重复/
错源/未完成失败；原诊断断言保留，诊断子项只在原assert成功后完成收据。未注册AST声明
与已注册运行身份分开；原ignored/milestone/手动条件明确NOT_APPLICABLE，不充PASS。

## 命令和影响闭包

- `cargo tm --list` 查看真实模块ID；无参数明确失败，不隐式执行全部。
- `cargo tm <module-id> [module-id ...]` 执行完整叶及消费者闭包的去重并集。
- `cargo tm --changes <逐行路径文件> --select-only --out <本地目录>` 生成可审查选择；
  去掉 `--select-only` 执行。`--refresh` 重新发现；`--catalog` 可显式指定目录。

Cargo元数据包含本地dev/optional/platform依赖，补充动态worker、Python及实际共享helper消费者。
源码路径按实际owner和最深测试模块映射；共享契约/输入、删除/重命名、manifest和配置变化
必须参与选择。未知触面保守扩大，过期目录重新发现，零选择/漏项/重项/缺消费边在执行前失败。
消费端再次构建图和校验分区，选择文件不能靠删项缩小义务。多个显式模块只执行一次并集。

## 时间与执行结果

至少三次完整warm，P90为排序后ceil(0.9N)项，常规目标≤60秒；模块墙钟包括运行时发现、
执行、原比较和清理，预构建、冷态及完整命令墙钟另列。超标可拆聚合继续细分；不可分割的
原正式性能采样/规模/生命周期oracle逐ID登记为具名慢模块，保原样本、断言、资源和触发条件。
不能强行60秒截止、缩输入或SKIP冒绿。未测不标达标。

Windows执行经原生Job计父子CPU/峰值、输出上限和截止杀树；HTTP族串行，原nextest资源组、
heavy gate与里程碑条件保留。发现/选择/编译/实际执行/通过分别报告，首次失败、NOT_RUN及
既存红不被重放绿覆盖。结果scope仅认证实际选面，不认证全环境/全生态。

## 门禁与旧入口迁移

计划在最终代码/验收/规范审查通过后，冻结执行实际受影响完整模块及消费者、集成与必要慢
模块并集。替代旧裸t+触面档叠跑；横切目录/runner迁移本身可能触及全部本仓模块，必须用
其真实影响面验收。work阶段只运行相关反例/模块，统一门仍由merge持有。

t/tv/tt/tb/th为截至2026-11-30的显式兼容导航，冻结旧身份映射完整叶；附加滤串只选择
完整模块，不抽取模块内部分测试。旧列表保留为历史守恒审计。tf/taa/ta/t3的语言执行已转
module runner；周期 `tf --batch` 消费 `periodic-module-schedule.json` 中逐个列出的recipe，
生成实际叶清单并去重执行一次，旧tt/tb不再另跑。schedule漏新recipe或重复/未知recipe失败。
UI/Man保留其owner腿与约定。周期触发仍为计划号%5、或>48h且有合并、缺回执即到期；主检出
单实例，不成为每计划第二个全局门。T3/手动/ignored条件保留，不要求新增cargo all。

关联：[批量试点](batch-execution.md)、[治理守恒](governance-and-inventory.md)。
