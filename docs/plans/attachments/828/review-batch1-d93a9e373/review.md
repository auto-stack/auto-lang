# PLAN-828 第一批受影响面复审

2026-10-22；Plan r3；review_scope=phase（T-02/T-03/T-04）；**needs_fix**。保持 executing 0/11，不授予 reviewed 或 merge。

## 基线与证据边界

- 原第一批冻结点：`6f482efafdfbb712dbc65c503de7153e8c2edf85`；差异基面：已审 T-01 `27aaa34d565b11facab37008d42c6e509cc3e13d`。
- 实际同树测试基线：`D:/autostack/.wt/lang-828/auto-lang`，`plan-828-dev@d93a9e37386fc6f8cb079b31c546df6ce37fdb5e`，clean。进入时已含第二批 `168b6b565` 与 T-07 首切片；未 reset、换树或改实施代码。
- 后批对 session 的修改限 commit 安装顺序，codec 仅去掉无用 mut；本报告所列恢复、存放、close 和查询路径在原第一批即存在且未被后批修复。[第一批差异](first-batch-session.diff)、[后批差异](later-session.diff)保留。**运行结果绑定 d93a9e373，不能冒称在 6f482efaf 原树运行；后批完整验收仍 pending。**
- sibling auto-down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，未改动/消费；源码/依赖/config/Specs 指纹见 [baseline.json](baseline.json)，结束复核见 final-state.json。
- Reviewer=本 Codex 审核上下文；未实施本批代码，未启动其他代理；不可用的底层模型/会话身份不编造。技能与共享合同内容哈希已记录。
- [Plan 输入（含冻结 SD-01..06）](plan-input.md)、[接口决策输入](interface-input.md)保存。SD 仍 proposed：本轮不发布 canonical Specs/ledger；C01/C02/C03/C04 阻止 SD-01/02/05 的完成声明，C05 涉 SD-04。其他新增 Spec 与正式入口细节仍归 T-09/最终审查。

## Findings：一次修齐的五项既有合同缺口

### B1-C01 / P1：解码成功与不完整 canonical 比对替代了 HIR 语义准入

**规则/触面**：§5.2、AC-02/05、V-04、T-03/T-04；不可信 HIR 必须在当前上下文重验后才能成为可复用受检 body。既有 HIR stage-contract 也保留 verify 唯一授信路径。

**入口/重放**：`artifact-controls.rs` → 公开 store 写入 → `warmup_persisted`（同键冲突容忍）→ `restore` → `resident` → 3→4→3 history hit。用同一正确 envelope/key/内容版本，仅改 body.owner 或参数 index。

**实测**：越界 owner、范围内错误 owner 和越界参数 index 全部合法解码、`restore_hir_adopted=2`（应为 1，跳过 helper）；公开 resident 返回坏值。现有 `verify_function` 对同一 body 分别拒绝 `verify.owner-mismatch` / `verify.binding-invalid`。越界 owner/参数随后 history hit 还复用同一错误 HIR Arc。伪造 `intrinsics=["nonexistent.symbol"]` 也被采纳：该持久字段未被读取校验。

**根因已证**：`session.rs:757–779` 只比较 def/content/caps/context 字段和 canonical；canonical 是内容寻址表示，省略 owner、参数 index 等，不能证明 verifier 全部规则成立。`FnHirArtifact.intrinsics` 在恢复中无消费。当前代码执行结果仍由另外验证的 ABC 保持正确；不夸称本探针已让 AV 执行错误或 panic。缺陷是 invalid HIR 进入 resident/history 受检对。

**关闭 oracle**：上述反例在采纳之前拒绝，helper 冷回退正确、后续编辑与二次恢复不传播坏 HIR；对当前上下文完整重验或证明等价的完整准入，不能只补已知字段特判。真实校验所用 module/body/def/type 表必须对应；合法 owner=0 与 owner=1 均采纳，context/literal/entry 越界既有拒绝保持。私有凭证不持久化。

### B1-C02 / P2：span 摘要比较了不可信自报值，没有检查实际解码 body

**规则/触面**：§5.2 来源、V-02、AC-02/05、T-03；自己的 span-layout 校验承诺。

**重放**：`artifact_span_negative` 保留正确 span_digest 字段，仅改 body.span 及第一 expr.span 为 `999000..999010`。

**实测**：真实 body digest 与字段不等；恢复仍采纳 2，resident 出现超出源码的 span，并在 3→4→3 命中同一坏 Arc。应跳过坏 helper，仅采纳 main。当前 AV 的独立 ABC sidecar 仍正确，本例未证明 AV trap span 已错。

**根因已证**：`session.rs:772` 只比较 `a.span_digest` 和冷 body digest；canonical 有意无 span。即使补 `verify_function`，该例 verifier 本就接受，故 C01 的修复不能代替此规则。

**关闭 oracle**：验证实际解码对象与摘要、合法当前来源/必要 mint 映射相符；坏来源不进入 history，合法非零 offset、未移位 roundtrip 及原 span/reanchor 矩阵保持。不得只把摘要字段重写成正确值再继续采纳。

### B1-C03 / P2：T-03 的真实 body-fact 读取与精确失效仍未实现

**规则/触面**：§5.1、V-03、C01-03、AC-01/05、T-03 明确要求：实际读取 body 事实变化使消费者失效，未读事实不误失效；探针走真实查询核心，不要求新 CTE/inline 语法。

**静态入口**：`function_queries.rs` 全文、`CheckedFunction` 字段/`credential_current`、生产 `prepare_edit → interface_unchanged → hit/miss`，以及 T-03 diff。

**已证现状**：FnEnv 只有接口、整体声明指纹、domain、owner 和 intrinsic 声明事实；CheckedFunction 无 body-fact 查询依赖；没有实际 body-fact 读取、记录、版本检查或对应测试。仅证明“callee body 改动时只读接口 caller 不重编”的一半。T-03 新工件的字符串/指纹不提供缺失的查询机制。此项是原合同实施/必需 oracle 缺口，**没有伪造一次运行失败**。

**关闭 oracle**：提供一个生产查询核心上的 body-fact 消费者探针；读取 A 的事实后，A 的已读事实变化使结果正确失效/重验，未读取的 B 或未读事实变化保持可复用；只读 A 接口的 caller 保持 identity 和零重编。依赖身份包含原定 owner/domain/配置授权/语义版本，当前消费点真正核对，不自报 changed 计数。把真实 query → dependency record → credential/hit 消费链和永久断言补入合同当前行。

### B1-C04 / P2：合法 mixed intrinsic 工件使用另一张局部编号表，无法恢复采用

**规则/触面**：§5.2 的稳定引用/实际 A07 消费、原 scalar intrinsic 支持、AC-02/05、T-03；不能以“坏工件回退正确”掩盖自己生成的合法工件不可读。

**重放**：`roundtrip_mixed_intrinsic_caller_nonzero_owner`：helper=`mark_a()`，main=`helper()+mark_b()+1`，tail=9；caps=`hir.test.trace.v1`；main 中等宽 1→2，真实 edit→存盘→restore。同样的无 intrinsic 三函数输入作为正控。

**实测**：无 intrinsic 采纳 3/3；mixed 只采纳 2/3，ABC admitted=3、结果=5、trace=[MarkA,MarkB] 均正确。读回 main 工件可见 `callee=3`、`stored_intrinsics=["hir.test.mark_a"]`；当前冷表槽 3 是 mark_a，真正 mark_b 是槽 4。存放前 candidate 的 canonical 仍为 mark_b。context 与 span 字段/实际摘要全部相同，唯一已定位内容差异是上述引用解释。

**根因已证**：新 HIR body 携带单函数产物的局部 DefId；`persist_all_units:1051` 用 warmup 的 intrinsic_symbols 解释它，`restore:767–769` 又把 body 塞进 cold_module 的表。两张局部表未重绑定。现有只编辑无 intrinsic helper 的 roundtrip 没覆盖它。

**关闭 oracle**：保持原支持面，使函数 HIR 的依赖引用和相关类型引用在生产/持久/当前上下文恢复中身份一致；采用稳定身份或明确可验证的表映射，不把数组槽当跨产物身份。同例改函 main 采纳成功、实际依赖为 mark_b；未改 helper 保持 mark_a，结果/trace/二次恢复及后续 hit 对象身份均正确。保留 owner 非零、错误 owner、错误 intrinsic/effect/授权反控。不能靠禁用该工件或放松 canonical 校验获得 PASS。

### B1-C05 / P2：新 cold_module 拥有整套 HIR body，close 未释放

**规则/触面**：§5.7、C05-01/F-05b、AC-06、T-03 新所有权；既闭取消/关闭行为不自动覆盖新增 owner。

**静态所有权证据**：`warmup:489` 无论是否持久化都 `m.clone()`；`FunctionIncrementalSession:297` 直接持有 `hir::Module`。HIR `Module.bodies: Vec<Body>`，body 内 locals/places/exprs/blocks 等均是拥有型 Vec。`close:891–903` 清 env_core/store/fns/current，却不清 cold_module。`close` 返回后该副本继续由存活的关闭 session 拥有，直到 drop(session)。没有相应外部 pin 作为理由。

**测试边界**：旧 close/pin/PausedRun 控制通过，证明其既有 Arc 释放/保活规则保持；它们只观察 fns/current 链，不能观察新 deep clone。此项据真实字段所有权与全部 close 分支确认，**未声称已有 Weak 能观察该深副本，也未编造定量峰值**。

**关闭 oracle**：close 显式释放新增拥有的 body 载荷；完整盘点 cold_module/新增持久化索引的保留理由。用可观测该新 owner 的定向控制证明资源在 close 时、无需 drop(session) 即释放；合法外部 snapshot/candidate/PausedRun 保活至最后 owner 释放的原控制继续通过。不要求清除合同允许保留的纯签名诊断元数据。

## 覆盖与实际结果

| 合同/路径 | 本轮证据 | 结果/边界 |
|---|---|---|
| T-02 单 body 热路径、后置拒绝/取消/交错/关闭 | contract hotpath、[session-controls](session-controls.rs.log) | 通过；不覆盖后批完整 dispatch 验收 |
| T-03 A07 合法/坏 magic/错 context/内容/entry、owner、参数、span、依赖字段 | [artifact-controls.rs](artifact-controls.rs)、[原始日志](artifact-controls.log) | 10 项：5 过、5 败；初始 8 项扩展过程另存，不重复计数 |
| T-03 合法非首位/混合 intrinsic 的真实存读 | [roundtrip-controls.rs](roundtrip-controls.rs)、[诊断日志](roundtrip-controls.log) | 2 项：1 过、1 败；首次日志保留 |
| T-03 依赖读取两轴 | 全路径静态核查 | 接口轴已有控制保持；body-fact 核心/必需探针缺失，未完成 |
| T-04 单函数 lower、普通 hit identity/零阶段工作、来源 | [session suite](function_incremental_828.log)、[r1](function_incremental_828_r1.log)、[contract](function_incremental_828_contract.log) | 普通受检内存路径通过；受 B1-C01/02 的恢复污染路径失败，不能关闭整项 |
| T-01 原边界/事务/发布 | boundary 11；admission 13（含 boundary 11）；transaction 8；publisher 11（含 transaction 8） | 全绿；include 展开总数，非新增例数，不相加作独立覆盖 |
| F-05b 既有控制/所有权 | close-replay 共 96（含所 include 的旧测试）；新 cold_module 静态盘点 | 旧例绿，新 owner 留存 B1-C05；不能因 96 全绿称资源轴完整关闭 |
| 永久回归 | session 18/18（显式跳过两个后批 512）；contract 22/22；r1 86/86；t01r 4/4；t03 filter 2/2 | [目标命令、二进制哈希、退出码](target-results.json)；不是旧交接所称 19/21 原冻结计数 |
| 健康/冻结 | 精确 5 Rust 文件 rustfmt --check，diff --check | 通过，raw 已保存；本次 Cargo stderr 无 warning，但缓存未重编的依赖不据此声称全仓无告警 |

原始构建入口为 `cargo test --locked --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828 --test function_incremental_828_contract --test function_incremental_828_r1 --test function_incremental_828_contract_t01r --no-run --message-format=json`；之后调用 Cargo 明确返回的 executable。独立控制用 [run-controls.ps1](run-controls.ps1) 精确选择当前 Cargo rlib，SHA 见 linked-libraries.json；`artifact_` 与 `roundtrip_` 最终退出码均 101，不能由外层日志命令成功抹成 PASS。

重放旧 harness 初次缺相对 fixtures/include 依赖，未编译日志存 `*.initial-runner.log`；补齐相同原件后运行通过，归因审核装配，不登记产品缺陷。没有改实施以适配探针。

## 下一步与未测范围

原唯一树在当前实际基面上一次修齐 B1-C01..05，补永久控制、共享根因/关闭表与当前合同，冻结新 SHA 后复审；[GLM 下一轮任务单](glm-next-round.md)可直接交接。不要撤销已过机制，不让后批既有提交自动变成审核通过。当前报表/合同中“第一批完成”“F-05b 全闭”等表述须以本轮剩余项为准。

T-05/06 完整链接/派发/原子安装、T-07 manifest/GC/资源预算/恢复深接、T-08 worker/IPC/P95/双线内存、T-09 消费者/SD 和 T-10 最终验收尚未由本次 phase 审查放行。全量 daily/surface 门仍 merge-owned pending（最终代码验收 PASS 后冻结裸 cargo t + 必要 tv 等）；未执行整档、merge、归档或清树。
