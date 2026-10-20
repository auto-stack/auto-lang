# PLAN-815 r2 / M815-A 规范增量校对稿（R3）

对象：`bc61501c58d45dde28ee2c739425dbc3c911b38c`。本文件是可供后续合入使用的候选文本，**未发布 canonical Specs**。R3 总体结论仍因回归测试缺陷 needs_fix；不得用本稿绕过阶段审查/冻结门。

本稿校正当前 m815-a-candidate.md §4 中沿用的“完整JSON深度120/统一budget_exceeded”概括，采用其 §0 和 contract-baseline §4 已明确的 wire120 / frame122 分层及实际错误分类；不改变本阶段能力或静态扫描器支持域。测试计数属于审查证据，不进入长期规范。

| ID | 操作/目标 | 当前基准 | 本阶段增量/AC |
|---|---|---|---|
| SD-01-A | add `docs/specs/auto-ui-tools/project.md` | 当前目标不存在 | 窄tool/诚实边界与来源，AC-07 |
| SD-02-A | add `docs/specs/tool-protocol/project.md` | 当前目标不存在 | 协议/能力/预算/错误与退出，AC-03当期、AC-07 |
| SD-04-A | modify `docs/specs/project-model/project.md` | 当前hash见baseline.json；已发布I-PROJECT，无wire规范 | 同源wire feature与支持/拒绝域，AC-02当期、AC-07 |
| SD-06-A | modify `docs/specs/repository-boundaries/generated-ui-distribution.md` | 当前hash见baseline.json | 独立窄tool来源/发行身份，不替换792生成UI发行闭包；AC-04当期、AC-07 |

SD-03、SD-05与上述SD的完整B面仍pending，不提前沉淀“UI主体/资产已迁”“旧Man算法退出”。完整Plan元数据和GOAL-003/007保留。

## SD-01-A：auto-ui-tools 当前行为

`crates/auto-ui-tools` 提供 lib 与 `auto-ui-tool` 可执行入口；M815-A只公开 `describe` 与 `validate_project` 元数据操作。正常/构建/开发依赖闭包无 auto-man、旧auto-lang、VM/生成器实现。客户端经轻协议调用独立进程；CLI不因本阶段自动切换路径。

`describe` 返回tool名称、实现版本、协议版本、能力及生命周期。两个工作操作均 non-interactive、non-cancellable；`cancel` 为协议控制消息，对已服务的A操作明确NotCancellable，对未知target明确InvalidField。

`validate_project` 校验请求元数据、路径和请求约束，报告明确 checked / not_checked。它不解析/求值语言、不生成、不构建；静态unsupported不冒充已求值。路径根据请求内绝对根解析；不能由请求确定的相对/Windows盘符相对或无盘符根路径明确报告不可校验，不探测进程cwd；绝对路径和普通可锚定相对路径按角色探测。存在的输出目录无需提前创建，未知UI生成能力在副作用前拒绝。

资产当前仅登记gallery/wm/shadcn及drawlist外部输入的owner/source/license/目标；生产embed仍在原处，本阶段未复制/搬走资源。完整生成/build/run及生产资源迁移属于B。

## SD-02-A：I-TOOL 当前协议与预算

- 协议版本1；JSON消息以4字节大端u32载荷长度分帧。stdout为控制帧，stderr为日志；`--version`/`--help`为协议外文本。
- 请求 `ClientEnvelope{protocol_version,id,body}`，复用唯一I-PROJECT数据owner codec。未知版本拒绝且如实携带客户端u64值；未知操作拒绝；具备正确帧但非法payload时结构化拒绝并继续服务。
- 单帧载荷最多8MiB，不含长度前缀；输入坏帧/超限/不可解析JSON/帧深度超限为流致命，exit2。响应序列化/输出帧预算或控制流写入失败为内部退出，exit1；正常EOF为exit0。不能将这些路径一概描述为budget_exceeded响应。
- JSON深度按根为1，进入对象成员/数组元素加1，标量或空容器返回当前层计数。wire请求文档限120，真实信封增加2层，因此完整帧限122。writer、read_frame、parse_frame对完整帧同限；可解析但超122时具名envelope拒绝；JSON解析器自己的递归限是最后防线，拒绝亦属Malformed。完整信封下的边界需用真实请求检查。
- 为后续事件/执行器保留1MiB/op进度载荷、128条未消费通知预算和BudgetExceeded错误类型。A不实际发射progress、不提供交互stdin或运行进程寿命；这些预留数据不构成B能力已经实现的声明。

## SD-04-A：I-PROJECT wire增量

可选 `wire` feature引入serde_json，为同一个GenerationRequest提供版本1的具名JSON codec。纯数据变换，无FS/VM/生成副作用；基础静态扫描器与已发布动态adapter的职责不变，不增加pac.at语法支持。

请求和固定嵌套结构采用精确槽/键集；缺失可空字段须显式null，未知字段、错类型与非法tag受控拒绝。Value采用带标签表示；支持已声明标量/Array/Block/Obj/Pair/CStr等数据变体，不可表示变体、非Unicode路径、非有限数、数值越界与过深wire明确拒绝，不静默转换。窗口编解码共享已冻结范围/可表示性规则。

TypeMismatch诊断payload保留field，members/defines在编解码两侧采用同一封闭集合。`type-mismatch:`只消费一次，未知/空payload、重复前缀不规范化为合法诊断。原普通unsupported变体保持身份。

## SD-06-A：来源与发布边界

A阶段新增独立tool/protocol及data-owner codec来源，不改变 `rust_ui.rs` 的792 DELIVERY表或现行UI生成发行资源。T-14发布收据须绑定已审代码、协议/API版本、真实实现/资源输入指纹、精确Spec补丁与依赖来源，验证delivery commit为master祖先及consumer实际解析；阶段review pass本身不是发布收据。

阶段不归档Plan、不清理唯一树。必要冻结门及实际发布/Spec/ledger来源收据齐备后，才可放行816消费A能力；完整UI能力继续依B。
