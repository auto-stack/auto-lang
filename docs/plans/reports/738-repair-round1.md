# PLAN-738 R1 修复交接（2026-10-09）

本轮是原 `needs_fix` 的实施修复，**不是复审通过**。原 AC-01..08 与 SD-01..07 不变；T-02..08 仍有未完成项，不合入、不归档、不发布 canonical Spec。

## 修复范围与实证

| finding | 已落地且有反例测试的修复 | 尚未闭合 |
|---|---|---|
| P738-R1 | shared AssemblyPlan 在注册前拒绝公共 body + 目标 body/native 的双实现；原宽松成功断言删除 | 真实引用符号闭包的 production strict 门；目标声明与 producer 的双源签名冲突 |
| P738-R2 | 从 Rust callee 导出参数/返回 metadata；手工 TCP producer 与 io VmModule 有独立契约；公共 net/json 名归一；arity 1→999 必须 Unverified；替换实际 callee 会撤销旧签名证明 | 六核心完整 receiver/static/generic/async/feature/适配契约；真实 AutoVM 覆盖之后的绑定证明；不能仅凭基础参数/返回 metadata 勾全 |
| P738-R3 | 先解析当前来源再查询缓存；CWD A→B 同名模块不复用 A；活会话源/层变更要求新 epoch；递归 leaf→mid→top 失效 | clone 仍共享 TypeStore；来源与所有依赖闭包的不可变 snapshot、循环边的完整身份审计 |
| P738-R4 | 文件身份与层 kind 纳入指纹；provider 内容、部分 host 源与编译 feature 纳入；非法 UTF-8/read failure 保留条目且拒绝指纹；null/null 不再 fresh，生成器传播定位/指纹失败 | 当前 generation 指纹仍扫描全库而非实际选择闭包；完整 host 依赖版本/features/environment 身份与实际 receipt 消费证据 |
| P738-R5 | 普通与 persistent 使用同一源装配计划/根定位；真实 Rust/C 入口先 resolve use 并记录目标，避免为 host 发射编 VM bytecode；Rust net 在发射前给明确拒绝 | Rust/C emitter 的实际符号/provider lowering 与完整 manifest；当前 host unsupported 门仍按模块导入，尚未实现只拒被引用项；persistent 尚未共享完整依赖/manifest 管线 |
| P738-R6 | target/env 检查；未知模块非零；相关 parse fail 非零；actual jsonx 排除无关六核心；auto 前缀过滤与便携源 ID；CLI 注册不再切换 CWD | 模块闭包还不是引用符号闭包；真实 provider trace、双源诊断和用户同名核心模块 origin 防串台 |
| P738-R7 | 字段条目、真实 TypeDecl visibility、函数签名/源 span；读失败层与遍历错误不丢分母；async 公共源迁移到现有泛型语法；Browser 无适配不借 native 假绿；JSON 增加三目标×两环境逐符号格 | type/field 的完整公开身份与 span/约束；Rust/C 真实逐符号 producer 验证；完整公共分母审计；Markdown 摘要仍非最终六格报告 |
| P738-R8 | 冻结源拼接，按原始 error label 映射真实层偏移，删除失败后重 parse 共享 TypeStore 的猜测；双 body 诊断各带源段 | 无 label 错误、跨段/多 label、签名/native 冲突与非解析诊断的双源位置 |
| P738-R9 | 新装配文件/CLI 格式检查与差异空白检查；新反例与兼容门禁记录 | C 支持样例实编、generated API→Rust 编译→service/ready→改 assembly 拒旧/再生、最终 SD 正文及独立复审 |

这些未完成项继续由 PLAN-738 承担，没有移入 D3b 当作批准延期。历史 D1 的动态扫描 ID 碰撞已由保留 ID 分配修复；D2 的 json/io 名面已有接线，独立签名和实际活 callee 仍未闭合。

## 需要修订的设计假设

原实现以 `register_std_shims + register_stdlib_ffi + build_from_inventory` 作为实际生产绑定证明。该假设不成立：`vm/engine.rs::AutoVM::new` 随后将 JSON parse 的 inventory 透传 String shim 覆盖为 `shim_json_parse_vm`。同一个 ID 的编译登记、CLI 校验与最终执行 producer 并非同一对象。当前修复已撤销被替换 callee 的旧 contract，但尚不能把所有真实消费者统一到不可变绑定证明。

另一个确定差异：`stdlib/auto/net.at` 的 `TcpStream.read(self TcpStream, buf []byte) int` 与 `vm/ffi/stdlib.rs::shim_net_tcp_stream_read` 不同；后者弹出 `buf_size: i32`，新建字节缓冲，向栈推回 count **和** bytes。这不是靠参数数目或返回类别可证明的等价签名。参数默认 View，也不能在未规定借用/写回适配的情况下假称它实现了公开缓冲参数。

JSON 的 `JsonValue?` 声明本身**不能证明错误必须返回 null**。本轮不把它列为已证实错误语义冲突；实际 JSON materialization、可空值和错误路径的独立适配证据仍需补齐。

因此交接为 `needs_replan`：修订 T-01/T-03/T-04 的 producer 与引用闭包设计，保留所有原 AC。该路由依据 auto-plan-work 的“Design is invalid but the goal still stands → Record evidence and affected IDs; return needs_replan to new for a bounded revision”。不是因预存测试红项，也不是批准删除核心门禁。

## 有界修订提案

1. 将真实 native 初始化及后续 override 收敛为共同 factory；每个契约绑定具体 producer/选中 callee，替换即失效。CLI 与普通/persistent VM 查询最终选中 snapshot。
2. 独立 producer contract 分开描述逻辑签名和显式 target adapter：receiver/static、泛型约束/ownership/async/feature、输入/输出转换及错误形态。无法证明的核心调用仍为 Unverified，strict 引用时明确拒绝，不把既有能力改名 Unsupported 清账。
3. 编译/发射记录真实 resolved origin 与引用符号闭包，由共同 gate 消费；未使用的 unsupported 声明不阻塞整模块。Rust/C 在实际 lowering 点报告 provider，persistent 共用依赖与 manifest。
4. TCP read 的恢复支持需单独明确公共契约与 adapter/producer 修复边界；不在当前代码中擅改公开 API 或声称已兼容。JSON 按实际 VM materialization 补 producer 证据，不从可空类型猜错误策略。
5. 在上述 snapshot 上收口 portable closure fingerprint、完整六格公共矩阵、双源诊断及 C/generated-service 实证；完成最终 SD 正文后，再独立复审新提交。

实施基础仍保留在原 lang-738 worktree；无需另建“全库重写”计划。修订应只更新相关设计/任务依赖，不清空已有有效反例证据，也不降低验收。

## 验证与版本绑定

版本与最终命令结果在收据 `738-repair-round1.json`；完整日志位于原 worktree 的 `.repair738-*.log`。日志保留本机，不以无关主检出 WIP 或其他计划结果作为本次实现证据。

- `cargo t plan738`：41/41，23.622s；包含 9 个新增装配反例。
- `cargo t --no-fail-fast`：5139/5155，16 fail，61.463s。14 项与 R1/批回执已记录红项逐名一致；另两项 native_registry 的“从 100 连续编号”旧断言已按避开保留 ID 的新行为纠正，`cargo t native_registry` 13/13，0.160s。未把纠正断言之后的 scoped 结果冒称第二次全日常重跑。
- `cargo tv`：162/162，2.030s。
- `cargo tt --no-fail-fast`：5515/5526，11 fail，91.039s，1785 skipped；11 红均在 R1/批回执已记录，无本轮新建 baseline worktree 同命令的双树证明。上一轮 back_provision/dep_fields/plan502 观测本轮未复现。
- 最新 CLI stdlib：10/10，0.62s；含 Rust net、未知模块、实际 jsonx 闭包和 portable source 负测。
- `cargo test -p auto-man --lib assembly_freshness_truth_table`：1/1。
- `cargo th --test-threads=1 --no-fail-fast`：100/102，191.445s，6677 skipped；`http_e2e_back_proxy_real_routes_corpora_data_face` 与保存的 R1 log 具有同一 `:id` 参数/400 断言；`http_e2e_plan707_relay_frames_timed_single_termination` 两帧同到达 `[1007,1007]`，定向复跑仍失败 `[1009,1009]`（0/1，1.625s）。撤回本轮新增的 inventory 排序/首个 producer 选择，恢复历史覆盖次序，保留冲突记录和最终 callee 的 metadata；该修正后的定向复跑仍失败（0/1，1.639s），不能认为已证实排序为因。AGENTS 记录过该时序族负载 flake，R1 log 中本例曾通过；本轮未新做基线同命令对照，**保留未确定归因，不宣称已证明预存或零新增红**。后续需对照原版本/最终初始化绑定及运行负载，再决定归因；不为使本计划通过而修改无证据关联的 HTTP 实现。
- 初始基础提交=`4fbcfe9685bdf8b82266f0b95a5b0b44037be891`；恢复 callee 覆盖次序的最终提交=`4089b84183c6044ee68f1b6962752d4cb54da821`。上述 t/tv/tt/CLI/freshness/full th 在次序修正之前执行；不冒称它们在最终树全档重跑。
- 最终树定向 `test-http-e2e` binary 的 `plan738 | native_registry`：**54/54，23.669s**；最终 `cargo check -p auto-lang -p auto -p auto-man`：通过，9.53s。worktree clean，日志/源版本收据另存；本报告不将基础修复误报为整项 pass。
- 新装配文件与 CLI `rustfmt --check`、`git diff --check`：通过；未整仓格式化巨型旧文件。预存编译警告保留分诊，不能称全仓零警告。

未跑 tf/taa/tu/docs_gen 专项；tf 是主检出批量档，本轮不触 aavm/UI generator/文档生成器或语法参考。日常别名内已有 docs_gen 的既有选择面，未独立发起文档重生成。
