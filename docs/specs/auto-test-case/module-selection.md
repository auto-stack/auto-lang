# 固定Cargo包与Rust模块测试

> PLAN-896：替换888/895动态叶/顶级/自动消费者图执行体系。旧目录仅保迁移审计价值。

## 固定组与命令

`scripts/test-groups.toml`直接列Cargo workspace/package/lib/test/bin、features/profile、Rust命名空间
及必要工具，职责对应实际源码。小crate整体一组；parser/vm/a2r/a2c/a2py/a2ts等是大crate内的Rust
模块组，不是新的Cargo package。不按20例/一分钟切叶，不维护逐测试新鲜度/动态消费者图。

- `cargo t`：原固定日常范围，保原适用条件，ACE宿主改为显式组。
- `cargo t parser`、`cargo t parser vm`：完整命名空间子树及配置内明确的集成测试。
- `cargo t ace`：独立ACE workspace常规自动测试与必要SDK私有桥接；不叠加语言日常包。
- `cargo t ace-performance`：明确保留的原正式计量/资源campaign，原样本、阈值、ignore及平台条件保持；占用当前nextest全池，避免同轮其他测试干扰。
- `cargo t all`：显式语言仓全部固定组，不含UI/Man外仓批次，仍保ignore/平台/里程碑条件。
- `cargo t --list`：只读配置组名，不触发Cargo元数据、发现或构建。
- `cargo t <组> --select-only [--out <新目录>]`：只输出固定调用计划，不认证实际发现/通过。Case组静态预览只输出stdout，不支持--out，以免绕过需真实owner工具的输出域审查。
- `cargo t aavm --ignored [--test-name <名字>]`：明确手动重放已忽略实验；原平台/T3条件保持。

同配置的多个组OR并集一次nextest调用、由原nextest并行调度；不同package/target/feature/profile/
环境/工具合同分别调用。正常Rust发现只针对选择的Cargo目标，新增同namespace测试自然纳入，零匹配
/未知组/缺终态失败。模块过滤缩小运行范围，不保证同crate测试二进制独立编译。

计划按修改明确完整职责组和必要集成/消费组，review核理由。parser局部改动不同于共享AST/语义/IR
变更；后者须显式补下游验证，但Cargo包依赖不自动触发所有消费者单测。不再有tm --changes自动闭包。
未知范围具名解释扩大，不能静默全编experimental，也不能省去已知真实消费者。

## 真实结果与条件

新入口仅薄分发给Cargo/nextest、Python unittest和TestCase原suite/比较器。nextest真实结构化逐测试
终态必须与所选实际枚举相等；重复/额外/缺失、异常退出和清理失败不得冒绿。父包装exit0不能替代原输入/
check的完成；原VM缓存及A2R/C/TS/Cookbook包装通过当前源绑定只读inventory和原execution_trace校验。
只在选到相关包装时读取其inventory，不为parser准备全仓工具。

Python保标准loader真实发现/逐ID执行收据；TestCase保source/oracle身份、逐check原比较，明确的
负控meta-oracle仍独立核实。失败/首红、NOT_RUN、NOT_APPLICABLE及重放结果分列；全组仅条件排除
不认证通过。60秒为成本目标，不是强杀时限，不统一三warm/P90或改变正式样本/预算/金样。

AAVM/AA2R自举实验测试按用户裁定默认ignore，保实现/源码/夹具/期望及显式手动入口；普通VM/Rust A2R
及真实产品消费者不误关。ACE宿主测试位于ace-conformance，语言侧私有协议单元桥接test-ace默认关闭，普通test-cases仍可独立运行，非ACE请求ACE worker
构建前拒绝；真正相关任务显式选ACE包/宿主/套件。用途标签不成为批量停用开关。

## 迁移与验证职责

888/895叶目录、逐义务合同保历史/迁移审计；不在新入口加载，不维持第二套公共执行器。tm/旧叶ID
命令明确报退役，原t/tv/tt/tb/th及tf/taa/ta/t3承接到固定组，外仓UI/Man保owner腿。一次迁移覆盖
对账证明原身份/包装check/条件/oracle去向；不要求新职责组不相交，兼容并集去重。

开发与repair只做反例/完整受影响组；独立审查后merge持冻结固定组/必要集成并集门，保有效证据复用。
周期到期规则与单实例主检出入口不变，不在每次提交附加全局门。原heavy_gate、HTTP串行和资源配置保持。
构建/缓存生命周期见[build-storage-lifecycle](build-storage-lifecycle.md)，A/B/C仍遵守原试点合同。

## 沿用的底层运行与判定约束

以下保留既有API/工具隔离和语义合同；涉及旧“module-case”的名称是现存显式API，不恢复叶调度。

Windows执行经原生Job计父子CPU/峰值、输出上限和截止杀树；HTTP族串行，原nextest资源组、
heavy gate与里程碑条件保留。发现/选择/编译/实际执行/通过分别报告，首次失败、NOT_RUN及
既存红不被重放绿覆盖。结果scope仅认证实际选面，不认证全环境/全生态。

## 门禁与旧入口迁移

计划在最终代码/验收/规范审查通过后，冻结执行实际受影响完整模块及消费者、集成与必要慢
模块并集。替代旧裸t+触面档叠跑；目录守恒用实际枚举/负控核，迁移用代表性真实入口和局部并集核。
可复用本次有效单轮计量与定向验收，不因横切runner再重复全目录。统一门仍由merge持有。

tv/tt/tb/th截至2026-11-30保留原Cargo配置及profile过滤，旧列表只作迁移审计。
语言周期入口tf/ta/t3显式调用固定all并集，taa调用默认ignored的aavm组；不读取旧recipe schedule或生成叶清单。
UI/Man保留owner腿。周期触发仍为计划号%5、或>48h且有合并、缺回执即到期；主检出单实例，
不成为每计划第二个全局门；T3/手动/ignored条件不绕过。显式全仓语言测试使用cargo t all。

关联：[批量试点](batch-execution.md)、[治理守恒](governance-and-inventory.md)。

准备、发现（包括Git/Cargo元数据）和实际执行都使用有界的Windows Job；父进程退出0而后代仍在运行属于基础设施错误，记录实际PID/映像并回收整树，不能据此发布工具就绪或测试通过。临时目录相关TEMP/TMP/TMPDIR参与有效环境指纹。

TestCase在系统短临时目录运行，避免SQLite等原生后端的长输出路径限制；命令仍保留真实runtime argv。退出后先逐文件SHA256校验复制到请求的输出目录，再按目录所有权标记回收临时目录，复制/清理计入模块墙钟。`case_artifacts`直接提供可读报告与复制清单，原始报告/收据字节及其原runtime路径不改。reparse point在遍历前拒绝，不向worktree复制链接，复制失败保留原目录供排查。

原计划的历史写入范围审计绑定其实际可达实施基面和归档交付SHA，不能审查当前计划相对master的无关改动；原命名空间禁区断言保持，当前产品依赖边界继续由实时结构检查负责。

编译准备/发现可显式接受compiler-auxiliaries.json钉住的完整路径+SHA256辅助映像，但必须先实际回收Job并确认active=0；保原错误/PID/映像/回收收据。未知残留、错hash、清理未完成仍失败，实际测试负载不启用此例外。参考机MSVC遥测辅助的原签名核验和实际编译对照见Plan888验收收据。Cargo运行器的包标签不算执行配置；Cargo home与仓内env默认按生效值绑定，真实覆盖保持失效。

Windows受计量命令禁用RUSTC_WRAPPER/RUSTC_WORKSPACE_WRAPPER，保原rustc及编译选项，让编译器实际处于本Job中；不停止系统共享缓存daemon。有效环境指纹反映此策略，冷准备成本单列。

独立module-case调用使用线程局部、panic-safe恢复的私有session作用域：源码闭包fixture的fresh/restart共享本次run输出域内的store，worker仍按原session ID区分存储。普通run_suite/run_suite_selected保留显式session跨调用共享，既有闭包20次restart采样及scalar性能合同不变。真实原fixture连续两次module-case调用都必须满足原fresh miss与restart hit，普通suite的原20次restart也须通过；不修改session字面量、原样本或expected来获得绿。

慢测试沿用原资源/截止/条件并具名报告；本轮不以partition-policy或叶模块重新切分VM。进一步拆分由实际成本和职责另行决定。
既有专项验证脚本仅通过ace-cargo.ps1将声明的语义模块转换为一个conformance测试目标的原生libtest过滤并集；不发现全仓、不查询叶目录、不推消费者。每个声明模块必须有真实通过项（或显式list项）；原必需ID/minimum/oracle仍由各验证器校验，未知旧目标失败。普通生产入口不经过此过渡适配。


真实比较收据的 PID 不能单独视为进程生命期身份：同一次大集合执行中 Windows 可回收 PID。每次 Invocation 先用 create_new 原子保留 PID-sequence.claim，保留至整 trace 目录回收；仅 claim 的 AlreadyExists 分配下一序号，其余 I/O 直接失败。start/finish JSON 仍 create_new，重复事件、已有 JSON 无 claim 等情况不被吞掉；不改变 schema、原输入绑定或判定。


合并并行不改变功能反控中的原墙钟/协议预算。完整nextest配置仅对三个具名短时限反控（native生命周期的cap/阻塞stdin、collections真实漏释放及实际整前端/降低/复制退化）保留本次test pool排他，使用num-test-threads随实际-j生效；普通组继续原并行。它不隔离其他会话，失败原收据保留，定向重放不能覆盖首次红，也不扩大时限或用笼统ERROR代替原fault物证。
