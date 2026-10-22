# PLAN-833：语言 SDK 接口与缺口调查

2026-10-22；本轮为**静态源码调查与既有证据定位**，没有编译、执行脚本、开启窗口或重跑 814/830 验收。公开实现存在不等于已证明可组合成图形 SDK。

## 来源

- 主 HEAD `d89e1dff36c13839d20ad35ae9f76088ab7c05d7`；SDK 最近变更 `9c7e44855c88b936835d57111c06e629e30de0bf`，调查时 SDK 无工作区改动。逐文件指纹见 [source-observation.json.txt](source-observation.json.txt)，均不是 833 实施基线。
- 当前知识：[SDK Spec](../../../../specs/language-sdk/project.md)、[图形交接](../../../../specs/auto-graphics/integration-contract.md)。[814 归档](../../../archive/814-language-sdk-ui-package-separation.md) 已交付 SDK；归档恢复发现 F814-MERGE-01 已由 `f97a9e2a964198b366105cad68b246360a9c5211` 关闭。[R9](../../814/review-r9/review.md) 有无 UI 源码快照 fresh/locked 外部 consumer 5/5；[R15](../../814/review-r15/review.md) 在未变 SDK 边界复用。这里只引用，不冒称重跑。
- `autographics-dev` 当前是 829 落点 `ab56c32ec4cb28e01fbcca74e9861164ffad43f0`。830 [R2](../../830/review-r2/review.md) 受审候选 `dbaea29e122d2fa2a18d18ce3f09444583098efd` 尚在修复/复审，不消费其工作树 WIP 或把候选作为稳定发布。

## SDK 接口矩阵

路径相对仓库根，下列 SDK 符号都在 `crates/auto-lang-core/src/`。

| 能力 | 实际入口 | 观察与使用边界 |
|---|---|---|
| 纯语言包 | `crates/auto-lang-core/Cargo.toml`、`src/lib.rs` | 已交付，Core 默认无 UI 文法实现、旧 auto-lang/UI/Man；独立 fixture `test/fixtures/modularity/language-sdk-consumer/tests/sdk_surface.rs` 覆盖 source→VM、手工管线、trans 和 UI 拒绝 |
| 解析/编译 | `parser::Parser`、`session::CompilerSession::core()`、`compile::CompileSession` | 可复用，不另造 parser/typechecker；`api` 模块实际是 Web/API 代码生成，不是通用嵌入式脚本会话 |
| 保留 VM | 根 `create_vm_from_source`（调查行4411） | 返回 `(AutoVM, stdout, entry, ObjectType)`，负责依赖/导入、解析、CTEE、codegen/link；初始化与多次回调状态仍需探针 |
| 一次性执行 | 根 `run_with_capture`、`run_with_capture_and_path_staged`、deadline/input 族 | 适合 smoke/诊断，不直接构成图形持续会话；stdout 不能作为图形命令协议 |
| 调用脚本函数 | `vm::engine::AutoVM::{call_fn_by_name,call_fn_by_name_segment,call_fn_by_name_cpu_slice,resume_fn_by_name_cpu_slice}` | 真实公开实现；实参预入 `AutoTask.ram`、结果留栈。legacy 同步可能忙等，首选 CPU slice；不是调用导出表就算执行成功 |
| 值封送 | `vm::ffi::VMConvertible`（`ffi/convert.rs`）、`AutoTask`、`VirtualRAM` | 标量/字符串有公开转换；所需容器、状态引用和返回 RC 寿命未由外部 5 项 fixture 覆盖，须 whole-path 验证，禁止把所有 NanoValue 强制转整数 |
| 预算 | `CpuSliceBudget::d2_default()`、`SegmentOutcome`（engine 调查行338..407、2982..3084） | 当前常量每片1M指令天花板/4ms/64步查钟/累计10M；部分上方旧注释仍写4096。Completed/Runnable/Parked 必须分开处理，不把耗尽当成功；不能推断阻塞 native 可抢占 |
| 退出/超时 | `AutoVM::{set_deadline,remove_task}` | deadline 只明确在 run_task_loop sweep 检查，不能推断直接分片覆盖；remove_task 的通用对象清偿有现存边界。桥须有自己的 pump/终态并释放整个实例 |
| native 扩展 | `NativeInterface::{register_dynamic,register_dynamic_with_id,register_name,merge}`、VM `merge_native_interface` | 实现存在；ShimFunc 为 Send+Sync；typed by-name 最终要求 BIGVM_NATIVES 已登记名。优先宿主调用/拉取纯数据，不捕获GPU/Rc VM、不写全局目录/硬编码ID |
| 高层回调 | `AutoInterpreter::call` → `VmInterpreter::call` | **不可用**：后者 `interpreter/vm_interpreter.rs` 调查行224..227 为 TODO，忽略实参/名字，恒返 `Ok(Value::Nil)`。本 Plan 不修其全局实现，不能拿此路径当真实脚本驱动 |
| 便利状态层 | `VmInterpreter::{set_global,get_global,run_ast}` | HashMap 存在不证明 Auto 运行状态完整保持；不能以 has_function/get_global 成功替代同一 VM 重复调用证据 |
| 新 ACE/AV | `docs/specs/auto-ac/project.md` 与在途817/818 | 有限 profile，不是 SDK/AutoVM 同义词；没有已交付的本次图形回调支持承诺。不用在途实现，不顺带认领 AC/AV/AAC/CAP/AutoShader |

## 图形接缝与缺口归属

829 在 `autographics-dev:experimental/autographics-prototype/` 的实际接缝：`core/runtime.rs::{Driver,Runtime,TickEvent}` 供固定 tick/动作事件；`core/scene.rs::{Scene,SceneChange}` 管 session/代际对象，**submit 逐条处理，不是整包原子事务**。`games/breakout/sim.rs::BreakoutDriver` 是可复用的独立对照，不能代跑 Auto 模式玩法。冻结 Breakout 为 f32(1/60)、56砖、底墙反弹、无生命/掉球失败；reset为宿主扩展。

830 候选 `core/composition/`、`host/composition_input.rs`、`host/surface_runtime.rs` 提供未来组合/真实路由出口，必须等最终验收合入再核符号。其修复/复审期文本会变化，不能据此声称已发布。

| Gap | 缺口 | 归属/闭合 |
|---|---|---|
| G-01 | 高层call为桩，低层调用须管理帧/参数/返回/状态 | 833外部窄adapter，T-02最多两种公开接法；不满足则具名回SDK owner，不改SDK绕过 |
| G-02 | 无正式图形模块、命令/错误/寿命协议 | 833定义有限实验数据合同；VM Value/AST留桥内，GPU对象留host |
| G-03 | f32顺序、Auto状态、容器/返回RC尚需实证 | T-02先验证；冻结数值/状态策略，不能放宽原golden或把计算移回Rust |
| G-04 | 图形分支早于SDK，直接引用本分支不存在的SDK路径不可行 | T-01固定双来源，T-03准备受管纯SDK镜像/清单；不整体合master、不隐式读主检出 |
| G-05 | 长计算/调用公平性与退出未接图形host | CPU slice+有界pump/队列/终态；纯计算profile，不宣称安全沙箱/阻塞native抢占 |
| G-06 | 包、平台、生产UI/RQHost仍未交付 | 只交Windows独立实验入口；生产迁移/发行归原owner |
| G-07 | I1原分母为两个Auto游戏 | 本Plan首个Auto Breakout+小场景；Squash真实Auto仍pending，I1整体不置done |

SDK载体建议在833唯一实施树内，从明确已交付源生成只读镜像，记录固定git source、逐文件hash、workspace继承/patch、根锁和build/include输入。外部driver显式相对消费；不修改镜像语言实现，不新增第二个同仓实施worktree。镜像生成目录可gitignore，但准备工具/清单/重建步骤必须跟踪并可在隔离A→B复跑。若owner已发布等价可搬SDK包，可消费其固定版本并提供相同证据；不拷target/rlib，不携UI/Man假称纯SDK。

## 建议

先做公开SDK/AutoVM的真实回调探针，再交Auto小场景与Auto拥有玩法的Breakout；3D仅验证现有对象由脚本控制，不再复刻Squash。Rust负责host/render/audio，脚本动作和输出必须被真实消费。

可以完成预规划，实施仍等待830验收/分支合入、SDK来源重核和用户后续启动指令。该切片不修改生产迁移门，也不因独立Auto入口完成自动认领生产AutoUI/RQHost/Forge、完整基础引擎或新版ACE。
