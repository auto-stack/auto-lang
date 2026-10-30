# 不相交叶模块与影响选测

> PLAN-888 r3；成本与交付状态以计划验收/归档收据为准。

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
  去掉 `--select-only` 执行。`--refresh` 重新发现；`--catalog` 可显式指定目录。自动/显式刷新只写本次`target/modules-*/runtime-catalog`，不自动修改跟踪目录；执行与计量消费选择收据内的`catalog_path`。发布目录由计划worktree显式生成。

Cargo元数据包含本地dev/optional/platform依赖，补充动态worker、Python及实际共享helper消费者。
源码路径按实际owner和最深测试模块映射；共享契约/输入、删除/重命名、manifest和配置变化
必须参与选择。未知触面保守扩大，过期目录重新发现，零选择/漏项/重项/缺消费边在执行前失败。
消费端再次构建图和校验分区，选择文件不能靠删项缩小义务。多个显式模块只执行一次并集。

## 时间与执行结果

先提供完整模块归属、职责、独立命令及注册用例/执行义务/原条件数量清单；真实枚举守恒不要求为统计执行全目录。然后逐模块完整单轮测总墙钟、父子累计CPU及峰值内存，常规目标尽量≤60秒。构建准备、冷态、背景负载和整命令另列；不统一重复三遍，不用一个或三个样本宣称P90。发现异常及时定向排查修复，仅重验受影响模块；接近或超过目标才按需复测。可拆聚合继续按职责拆；特别长的原子列ID、已观察耗时/资源与来源时点、原因、触发、优化/保留/暂关建议，交人决定。既有明显长例可先列人工待定测量队列，但原测试启用状态/原样本/断言保持，未测不标达标或PASS。

单轮CPU为模块runner进程CPU加发现/执行Job的workload CPU，不重复加meter CPU。内存列顺序发现/执行子树的原生Job峰值最大值及单进程峰值；共享Python runner内存不在该峰值内，范围明确写入收据。AWAITING_HUMAN为测量队列状态，普通模块门必须失败而非跳过即绿；NOT_APPLICABLE仍只表示原条件排除。

Windows执行经原生Job计父子CPU/峰值、输出上限和截止杀树；HTTP族串行，原nextest资源组、
heavy gate与里程碑条件保留。发现/选择/编译/实际执行/通过分别报告，首次失败、NOT_RUN及
既存红不被重放绿覆盖。结果scope仅认证实际选面，不认证全环境/全生态。

## 门禁与旧入口迁移

计划在最终代码/验收/规范审查通过后，冻结执行实际受影响完整模块及消费者、集成与必要慢
模块并集。替代旧裸t+触面档叠跑；目录守恒用实际枚举/负控核，迁移用代表性真实入口和局部并集核。
可复用本次有效单轮计量与定向验收，不因横切runner再重复全目录。统一门仍由merge持有。

t/tv/tt/tb/th为截至2026-11-30的显式兼容导航，冻结旧身份映射完整叶；附加滤串只选择
完整模块，不抽取模块内部分测试。旧列表保留为历史守恒审计。tf/taa/ta/t3的语言执行已转
module runner；周期 `tf --batch` 消费 `periodic-module-schedule.json` 中逐个列出的recipe，
生成实际叶清单并去重执行一次，旧tt/tb不再另跑。schedule漏新recipe或重复/未知recipe失败。
UI/Man保留其owner腿与约定。周期触发仍为计划号%5、或>48h且有合并、缺回执即到期；主检出
单实例，不成为每计划第二个全局门。T3/手动/ignored条件保留，不要求新增cargo all。

关联：[批量试点](batch-execution.md)、[治理守恒](governance-and-inventory.md)。

测得可拆慢聚合由partition-policy.json明确细分，builder与消费端复核同一规则，不能丢例或悄悄停止应用。故障控制使用panic-safe测试环境恢复，既不能被预建worker绕过准备器反控，也不在恢复正控时误丢调用方工具路径。成本参考保留原AV/MN 30样本及2000ms门，相关参数与oracle生成开关参与源/环境指纹；成本验收不减样本、不放宽门、不再生期望。

准备、发现（包括Git/Cargo元数据）和实际执行都使用有界的Windows Job；父进程退出0而后代仍在运行属于基础设施错误，记录实际PID/映像并回收整树，不能据此发布工具就绪或测试通过。临时目录相关TEMP/TMP/TMPDIR参与有效环境指纹。

TestCase在系统短临时目录运行，避免SQLite等原生后端的长输出路径限制；命令仍保留真实runtime argv。退出后先逐文件SHA256校验复制到请求的输出目录，再按目录所有权标记回收临时目录，复制/清理计入模块墙钟。`case_artifacts`直接提供可读报告与复制清单，原始报告/收据字节及其原runtime路径不改。reparse point在遍历前拒绝，不向worktree复制链接，复制失败保留原目录供排查。

原计划的历史写入范围审计绑定其实际可达实施基面和归档交付SHA，不能审查当前计划相对master的无关改动；原命名空间禁区断言保持，当前产品依赖边界继续由实时结构检查负责。

编译准备/发现可显式接受compiler-auxiliaries.json钉住的完整路径+SHA256辅助映像，但必须先实际回收Job并确认active=0；保原错误/PID/映像/回收收据。未知残留、错hash、清理未完成仍失败，实际测试负载不启用此例外。参考机MSVC遥测辅助的原签名核验和实际编译对照见Plan888验收收据。Cargo运行器的包标签不算执行配置；Cargo home与仓内env默认按生效值绑定，真实覆盖保持失效。

Windows受计量命令禁用RUSTC_WRAPPER/RUSTC_WORKSPACE_WRAPPER，保原rustc及编译选项，让编译器实际处于本Job中；不停止系统共享缓存daemon。有效环境指纹反映此策略，冷准备成本单列。

Cargo env默认实际应用于直接子命令和指纹；共享runner/调度配置的global_inputs触面展开全部受影响模块，显式命名基础设施叶仍可独立运行。889集合suite及独立collections worker与原scalar端口分开准备，运行身份保持各自归属。

独立module-case调用使用线程局部、panic-safe恢复的私有session作用域：源码闭包fixture的fresh/restart共享本次run输出域内的store，worker仍按原session ID区分存储。普通run_suite/run_suite_selected保留显式session跨调用共享，既有闭包20次restart采样及scalar性能合同不变。真实原fixture连续两次module-case调用都必须满足原fresh miss与restart hit，普通suite的原20次restart也须通过；不修改session字面量、原样本或expected来获得绿。

慢聚合可以按具名长原子/周转矩阵隔离，其他快速反控保留同组；partition-policy的isolate_tests使用精确原测试名，生产与消费都拒绝缺失目标或重新混组，原子并集保持。

目录发布使用显式LF序列化，Git属性也固定索引和分片为LF，使分片原始字节SHA256在工作树、Git对象及新检出之间一致。生产端自读回不足以证明发布可用，必须经过实际Git归一化后再次加载；编辑分片仍按原精确哈希拒绝，不放宽校验。
