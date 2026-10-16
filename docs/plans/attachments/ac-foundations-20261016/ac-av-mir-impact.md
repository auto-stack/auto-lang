# AC/AV/AJ、CTE、Typed MIR、ABC及增量发布：在途影响和后续规划

日期：2026-10-16。性质：设计更新影响收据，不是新Plan、不替换在途执行合同。用户已授权整合design/strategy、更新v0.6及评估796/797，不授权本轮开始VM实现。

## 1. 调查与结论

主检出调查基面 `62feb29e5be4f531b6e29ea90bb53fc65cd2d2a3`；796 executing r1、0/9，797 drafting r1、0/9。主检出存在他方计划簿记及附件WIP，不修改/提交其内容；状态可能由并行会话继续更新，此快照不代替其最新实施收据。

读取现状入口：`docs/specs/overview.md`、`docs/specs/auto-hir/project.md`、`docs/specs/auto-ac/project.md`。目前有限HIR/profile/native为现状；本文未运行MIR/ABC/AV或CTE测试，设计确认不是实现证据，不将目标写成Spec现状。

当前执行合同：`docs/plans/796-ac-semantic-metadata-inference-probe.md` §0/1/5/6/8/10；`docs/plans/797-autoabi-v01-bidirectional-probe.md` §0/1/2/5/6/8；共同并行边界见同目录 `parallel-contract.md`。新目标的正式设计见 `docs/design/43-ac-av-typed-mir-and-abc.md`。

## 2. 是否需要暂停或修订796/797

| 计划 | 已约定产物 | 新路线关系 | 本轮处理 |
|---|---|---|---|
| 796 | 共同语义身份/类型和Meta、有限推导、查询/cache；CTRequest/CTResult及实例/runtime投影是模型合同 | HIR/MIR/ABC/AV消费同一语义来源；AV物理布局在下游，CT模型不等于真实引擎 | 可继续；不追加MIR/ABC/AV/真实CTE，保留r1与进度 |
| 797 | 有限Windows x64原生AutoABI、独立编译双方及C双向互调、布局/资源/cache探针 | 新AV内部调用和可移植模块不是物理AutoABI；未来AV↔native adapter消费其合同 | 可继续；不追加VM loader、沙箱、JIT或Rust shim产品，保留r1与进度 |

796合同明确排除HIR→ABC/AVM、MIR及真实comptime；§5要求模型计数标model_compute，不伪称vm_execute。797明确排除loader/热重载/MIR/优化/自举桥，其核心库不依赖AVM，target layout由ABI侧派生。两者均没有“必须使用旧VM私有值布局”或“CTE模型必须执行旧引擎”的硬依赖，故当前没有需要暂停的合同矛盾。

未来若实际冻结Schema把旧VmInterpreter/宿主地址/布局写成不可替换承诺，或所需CTE值/能力/依赖表达无法容纳，应拿具体字段/用例触发显式revision和受影响复审；不能在设计附件中偷偷改任务、验收、范围或旧证据。不能仅因文档出现历史AVM称谓就强制返工。

## 3. 接下来先规划什么

首选工作包：**Typed MIR与ABC/AV执行合同及首个全路径探针**。

- 目标：共同执行语义和可运行的有限 `Checked HIR → Typed MIR → ABC → Rust AV`，与已合入native profile对拍；不是单写格式说明，也不是完整新VM一次性交付。
- 先定界：i32/bool、函数调用、分支/循环/短路及精确trap；显式列出其他类型、资源、Meta、模块类别的后续面/unsupported。MIR控制流/值策略、ABC编码/装载/验证、值与调用模型、CTE接口版本、资源/缓存/性能预算先调查冻结，未决研究有独立成果和停止条件。
- 必要反例：未执行分支含trap、整数边界、调用/求值顺序、坏类型/跨owner ref、非法控制流/字节码、超限/取消；独立expected/native oracle不能由同一AV路径自证。
- 从首例有AutoDown源文件和真实定向运行证据；共享adapter正式接入遵守788/796合入门。首次落地即统一store、fresh/hit/restart/invalidation/corruption、真实执行计数、内存预算和端到端trace，不等性能pass阶段补。
- 可独立研究/用现有合入HIR起探针；真实类型/ABI/测试合同绑定合入的796/797/788，记录版本/hash。最小AV若不调用FFI，不人为要求797成为全部工作的前置。
- 新实验路径和API由正式Plan核查后决定，不改根crate成员、生产VM及拆分热点。先共同设计与可行性；若实现规模超出中型Plan，拆为最小MIR、ABC/AV等单向依赖计划，不让两个模块互等，也不要求完成全AC。

第二工作包：**AV驱动的CTE查询与冻结缓存闭环**。依赖真实AV及796语义合同；检查可执行HIR片段后经MIR/ABC执行，结果经portable冻结/消费校验回灌推导。记录Meta/helper/负依赖、目标与权限，cache hit真实执行次数为0，miss正控非0，环/取消/超限有独立状态，默认拒绝未追踪宿主访问。

随后是native接共享MIR、语言/运行时扩面与runtime反射投影；AAC/AAV以实际主体所需能力为门。完整插件平台/分层JIT和自主后端分别安排，不将“Rust→Auto迁移”与“旧VM→新模型”合并冒称完成。

## 4. AC/AV/AJ统一架构续评

续评核验：主检出HEAD为`3e5aa91f65eca89dab97d652f3782c72c5435526`；文档检查结束时796为executing r1、1/9，797为drafting r1、0/9。796进度由并行实施方更新，本轮未改其文件；§1的0/9保留为前次调查快照。

2026-10-16进一步确认：AJ快速/优化编译与AC提前编译共用MIR/优化/codegen、依赖和store；AV解释及AJ两层原生执行共享语义。开发和发布可复用满足完整配置和依赖的产物；发布计算完整闭包并补编，而不是默认清空cache从源码冷编。具体合同及分期见Design43 §8–10；本轮没有创建AJ命令或实现新runtime。

| 对象 | 本次新增设计影响 | 是否改变正在实施的合同 |
|---|---|---|
| 796 | 语义TypeRef/Meta和CT冻结结果需可供MIR/AV/AJ及发布投影共同消费；画像/物理布局不改语义身份 | 无新增实现项。当前合同不固定旧VM布局或代码地址；若实际Schema存在反例，凭字段/用例再显式revision，不提前宣称完全兼容所有未来类型 |
| 797 | 公开AutoABI与解释/JIT内部帧/调用域分开；后续跨层adapter消费target/布局/资源/回调合同 | 无新增JIT/loader/跨层帧/重定位/发布验收，继续原生三腿探针；签名兼容不自动保证旧代码可卸载 |
| 下一工作包 | 首MIR/ABC/AV设计加入共同编译服务、逻辑单元/内容版本、profile化缓存、代际/假设失效和原生产物可重定位/再生边界 | 尚未分配Plan，不是修订796/797；有限探针验证身份/缓存/原生reference，不以新需求扩张到完整分层JIT/发布器 |

首工作包追加的区分性用例建议：只改一个函数body导致相关产物失效、profile/target变化不误取旧机器码、依赖变化识别callee/内联输入、损坏/回收后受控再生；代际与假设是合同/有限模型时必须如实标记，不冒称在线AJ安装。测试recipe和实际入口在正式Plan中核查，未创建的脚本/计数不是通过证据。

后续时序：最小MIR/ABC/AV → AV驱动CTE闭环；native共用MIR/有限增量发布按依赖单列；完整T1/T2热点调度、跨层替换、OSR/deopt和通用portable发布另立计划。CTE可先仅解释执行，不要求AJ先完成。新增发布能力纳入v0.6已承诺native支持面，详细范围和预算需正式Plan；已有native热重载目标不因完整AJ延期而自动延期。

现在设计编译单元/共同服务、跨层签名/代际、依赖及发布闭包/profile；现在实现最小AV/MIR和首例cache/AutoDown/资源/trace，后续实现完整分层AJ。首次release可能大量补编；不保证发布亚秒，不把运行覆盖率当闭包，不把进程内JIT地址当持久机器码。仅改变优化/部署配置时尽量保留上游语义/CTE/MIR命中。

## 5. 文档一致性与验证范围

更新Design43及索引、Design40/41/42、HIR/后端/自举/ABI战略和v0.6路线；清除本次触面中“MIR全部延期v0.7”“CTE以旧AVM直接HIR→ABC为默认”的现行安排。原源码审计、历史快照、既有Plan及Spec现状保持事实，不改成已实现。

仅文档更新，按AGENTS.md Category A不运行cargo t/docs_gen；执行文档链接/章节/矛盾检查与定向diff检查。没有分配新计划编号、运行性能基准、冻结新ABI/ABC格式或启动实施。native-only旧计划数和日历不覆盖新增工作，需后续探针重新估算。
