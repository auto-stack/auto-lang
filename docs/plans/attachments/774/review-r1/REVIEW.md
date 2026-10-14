# PLAN-774 r3 第一轮独立最终复审

2026-10-13，`review_scope: final`，`outcome: needs_fix`。现有门禁全部通过，但合同内仍有缺口；不能放行 770/771。本轮没有实施修复、合入、归档或发布 canonical Specs / live ledger。

## 基线和独立性

- 审阅者：当前 Codex `/root` 会话；本会话没有实施 774，根据合同、提交、源码、断言和独立反例重构结论。没有另建 agent 或另一个 reviewer checkout，也不声明未暴露的模型/session 身份。
- 计划：主检出 `docs/plans/774-ac-data-resource-contract.md`，revision 3；审前全文冻结在 [plan-before-review.md](plan-before-review.md)。进度和本记录不改变语义 revision。
- 主检出观察 HEAD：`f799be60190739129eb2fbb41d500eb942fcd4a7`。主检出已有 776/777–780 等设计/计划 WIP 保留；本轮只增加 774 复审簿记及债务登记。
- 唯一实现树：`D:/autostack/.wt/lang-774/auto-lang`，branch `plan-774-dev`。
- reviewed_commit：`f1a26e1be51de2dbb0c0d983b5ec346a86475aed`；base_commit：`5e21561d2c303a02319ff7a29e4607933e09a38f`。入场/出场 HEAD 相同，实现树 tracked/untracked 均 clean，`git diff --check` clean。
- 依赖：`auto-ac-prototype` path dependency 同树 `experimental/ac-core`；原型源码/根定义/旧 gates 无 774 diff。当前 master 相对基面在所审 Specs、ac-core 与旧 gate 路径无差异。Cranelift 版本按锁文件 0.126.2；rustc 1.98.0；rust-lld 与 shim 的完整 SHA256 见 [774-gate-receipt.txt](774-gate-receipt.txt)。
- Spec 输入、合同、代码、锁文件和技能版本完整 hash 见 [input-hashes.tsv](input-hashes.tsv)。冻结 SD：SHA256 `17e1ee60194c9d9a510bac0f31640a6661d195a0fdb9398ad7d094c2c9da8df5`，全文见 [proposed-spec-delta.frozen.md](proposed-spec-delta.frozen.md)。

## 重跑和独立控制

| 入口 | 本轮结果 | 持久证据 |
|---|---|---|
| 774 PS gate | 6/6 步；check/fmt clean；39 probe tests；scope clean | [gate 输出](review-774-gate.log)、[收据](774-gate-receipt.txt) |
| 完整 probe `cargo test --manifest-path experimental/ac-core/probes/data-resources/Cargo.toml --locked` | 39 passed，0 failed/ignored | [原始测试日志](review-probe-full.log) |
| 741 `-SkipMainGates` | 13/13 步 | [日志](review-741.log) |
| 745 | 13/13 步，74 cases | [日志](review-745.log) |
| 772（历史脚本名 767） | 14/14 步，45 cases | [日志](review-767.log) |
| 完整 ac-core `cargo test --manifest-path experimental/ac-core/Cargo.toml --locked` | 145 passed；既有 1 ignored | [日志](review-core-full.log) |
| 独立规则/native 控制 | 7 测：3 合法/反控通过，4 合同断言失败 | [反例源](counterexamples.rs)、[原始输出](counterexamples.log)、[重放入口](replay.ps1) |
| PS 警告检测控制 | 实际警告在第二行时 matcher 返回 False | [控制输出](warning-control.txt) |

完整 ac-core 第一次尝试与仍在运行的 767 gate 争用同一 exe，遇 OS error 5；待 gate 结束串行重跑成功，归为本轮验证调度冲突，不归实现回归。未跑根 cargo t/tv/docs_gen：本计划无 crates/root 定义改动，遵守计划和仓库 Category A 范围门禁。

独立反例借用已提交 harness/shim，从外部 ignored manifest 运行，没有改 probe 源码。重放：在 PowerShell 执行 `& docs/plans/attachments/774/review-r1/replay.ps1`；可用 `-Tree` 指定仍是该修复树的路径。当前应 exit 101：失败是合同断言，不能修改 expected 来制造绿色。真实 PE / COFF 位于执行器临时目录，hash 清单已保存到 [native-artifact-hashes.tsv](native-artifact-hashes.tsv)；结论另有源码、输入和实际输出，不依赖临时目录继续存在。

## 发现与修复要求

### P774-R1-01 — P1：资源账本隐藏容量使预算不归还，检查仍假通过

类型：原要求缺口；AC-03/04；T-03/04/06；D-06；V-03/08。

`shim/src/lib.rs:170–188` 用固定 64 项 arena，表满后仍成功分配并静默丢弃 length 记录。65 个 1-byte buffer 同时存活、逆序全部释放后，真实 PE exit 0，SUMMARY=`65 65 65 0 1 65`，`p774_check_invariant`（:907–914）没有检查 live_bytes。64 个的合法兄弟控制全绿。

更进一步：budget=65，同一序列全部释放后再请求 65 字节，错误返回 `res.budget`、PE exit 1，SUMMARY=`65 65 65 0 1 65`。这违反“按存活字节记账、释放即时回收额度”，不是要求任意性能/规模，也不是要新增语言特性：已接受的资源不能丢账。现合同未声明 64 个并发对象的支持上限。

`p774_set_budget(0)` 又把 0 当 unlimited，与合同通用 `live+req≤budget` 规则缺少约定；1-byte 请求实际分配成功、attempts=1。若 0 要作特殊控制值，须显式裁定并与预算 0 的意义区分，不能把代码隐含缺省当冻结合同。

修复：提供完整有效的长度记账或在所有成功资源路径上保持准确账本；不静默丢记录。检查 live_bytes/实际释放及额度回收；补 64/65、释放后整额重用、0-byte budget 的明确 oracle。无需引入完整生产分配框架。

### P774-R1-02 — P1：借用返回来源未验证，声明的拒绝面没有全部执行

类型：原要求/证据缺口；AC-01/03；T-02/03/04/06；D-03/04；V-04。

`src/rules.rs:149–152` 的 `ReturnViewParam(_)` 无视 slot、来源和活跃状态，连 `Make(0) → Free(0) → ReturnViewParam(0)` 都接受。此反例证明模型只信操作的“param”标签，不能区分借入参数、move-in 拥有参数与局部；不是在声称产品 verifier 已存在或有该漏洞。

`tests/v04_reject.rs:149–180` 的合法兄弟 `view_of_param` 实际返回 `borrow_read` 得到的 i32（65），没有将借用 pointer/length 返回并在调用者使用；M5 合法兄弟只有模型结果，未运行该序列的真实 PE。`verify.field-borrow` / `verify.container-borrow` 只列在 `REJECTION_CODES`，没有可达拒绝操作/测试；V-01 却把 D-03 的容器禁令映射给只测 return escape 的 M4。`ownership-required` 的 Make 覆写检查不能证明 `.move` 参数调用要求。

修复：在受限模型中保留真实资源来源/参数访问模式，给声明支持与拒绝的存储/传参语境可执行入口和定位 oracle；借用返回需真实 pointer/length 跨调用及调用者读，拥有参数借用不得逃出清理边界；补合法兄弟 PE。无需实现产品全量借检查器。

独立多 view / mut 排他控制已通过：两 view 均结束后可 mut；活跃 mut 期间再 mut/view/move/free 均正确拒绝。这些通过面保留。

### P774-R1-03 — P2：全路径清理和 LIFO 的结论超过实际断言

类型：遗漏/证据缺口；AC-03；T-03/04/06；D-04/06；V-03/05。

合同要求出口 LIFO；`tests/v03_lifecycle.rs:97–98,134–136` 在先创建 p、再 clone q 后依次 free(p)、free(q)，恰为 FIFO，而计数 oracle 无法识别顺序。现 V03-4 只有单层 branch，V03-5 只有自然循环出口，没有嵌套分支/循环早退的 break/continue 清理；临时完整语句延长、move-in 参数的 callee 清理/return 转移亦没有区分证据。ABI-C 的 shim 返回与 caller free 已证明，应保留，不把它误列为未实现。

AP-1..4 失败均有基本控制，其中 clone 失败有原 owner 的回收，但 concat/字面量提升只有无其他 owned 资源的直接失败；不足以声称相应嵌套正常失败出口都按合同清理。clone 独立性当前仅 ptr 不同/计数，未检查复制内容及释放一个后另一个仍有效。

修复：用可区分资源 ID/释放 trace 的 oracle 覆盖 LIFO，补代表性嵌套 break/continue/return、临时与调用帧的资源边界；在已有 owner 下逐申请点注入并验证回收；clone 增加内容/存活控制。范围仍是探针可行性，不新增 Auto 编译证据。

### P774-R1-04 — P1：共同决定缺项且与下游/既有语言合同漂移

类型：合同缺口/未批准变更；AC-01/04；T-02/05；D-02/03/05/07；V-01/07。

| 子项 | 实际缺口和依据 | 要求 |
|---|---|---|
| Str 参数模式 | decisions.md:124 把“参数 Str”统一裁为 move；Design 04:43–61、ast/fun.rs:309 明确类型与访问模式分离、两侧默认 view，move 需要定义处显式修饰。事实表已读该规则却声称无既有承诺变化 | 保留默认 view，单独列 `view/mut/move Str`；或按既定规则提交用户决策/修订，不能以 a2r 的 String 映射代替源语义 |
| text/record 身份 | 774 冻结 `text-i32-draft` / `record-i32-draft`；770 §5.1 仍 `text-draft`，771 §5.1 仍 `record-draft`；精确 source 名字未进消费身份表 | 同步唯一身份表及对应下游合同修订；保留三维隔离，实施不能自改 |
| 字符转换 | 774 改为 `.to_codepoint()` / `from_codepoint`；770 §5.1 仍 `.to(int)`，并要求具体拼写由 774 核查、有变化先修订 | 区分机器 shim API 与源语言拼写，选定并核对实际语法/修订下游 |
| 跨行/转义 | 774 D-02 必须冻结；现只给 UTF-8/标量/slice，未列普通/跨行拼写、LF/CRLF、escape 白名单及非法 escape/多字符 char 的明确前门边界。770 字面量和 CRLF 测试仍依赖此决定 | 用现行规范/消费者明确规则和诊断责任，补前置冻结表；不要求在 774 实现 source parser |
| 嵌套 record 布局 | 774:174–175 统一 size=4n、offset=4i。771 已承诺非递归嵌套 record 与赋值 copy；例如 Inner 两个 i32，Outer{Inner,bool} 的 inline 值布局不能按 4×2 解释，若用指针则拥有/copy/调用义务也未定义 | 定义递归 size/alignment/offset 的 checked arithmetic 及值语义、out-slot 非重叠；给代表性嵌套/混合调用 oracle，不能把该下游支持降为后续 |

R2 比较的结论也应限定：当前 `h_get` 不查 active、槽位不带 generation 的简化实现确实静默错读；这不能证明“引用计数句柄本身无法配合静态前门”。R1 合理的选择可保留，但须以所比较候选的限制/复杂度作理由，不把故意欠验证的小 shim 缺陷泛化成方案不可能。

### P774-R1-05 — P2：释放计数不符合冻结口径，双 free 检测读取已释放内存

类型：oracle/实现缺口；AC-03；T-02/03/04/06；D-06；V-03/05。

合同和 probe-cases §0 要求 frees=实际成功释放数。`shim/src/lib.rs:149` 却先对每个 non-null free 调用累加 frees，:151 读取 buffer 已被 HeapFree 后的 magic，:156 忽略 HeapFree 返回值。V03-2n 用 `frees=2` 证明双 free，但物理上只应释放一次；“实际释放”与“释放尝试/违规次数”混用。对已释放存储读 magic 也不能保证文档所谓确定性安全检测，重用槽位时不再可靠。本项后半为源码证据，本轮未使用系统 page-heap/修改系统配置复现，不将其报告为已实测 crash。

修复：先从有效外部账本验证资源身份，分离 free attempts、successful frees、invalid/double free；核对 OS 结果再计成功。负控应以非法计数/不变量失败证明检测能力，保留成功释放口径，不读取 freed header。不要让此探针绕过“无重复清理”的门禁。

### P774-R1-06 — P2：门禁/收据仍有可信缺口，规范增量误称 List 已冻结

类型：验证/收尾缺口；AC-05；T-04/05/06；SD-01..04；V-06/09。

- `scripts/verify-ac-data-resources-774.ps1:89` 对整个多行字符串用 `^warning`，第二行 warning 无法匹配；独立控制已实测 False。现 clean 构建未发现真实 compiler warning，应保留其结果，但 helper 的 zero-warning 保证不成立。
- `src/lib.rs:449` shim build 截止为 120s，且内部 cargo build 未带 `--locked`，与计划“所有 PE/子进程 60s、locked”证据约定不一致。V06-2 固定前驱 `HANG-PRE live=1` 没有观察 oracle：`run_expect_deadline` 只返回标签，既有 executor 在 Deadline 分支丢弃输出；错误也不能按成功返回文本来冒称完整输出已保存。
- PS Invoke-Cargo 收集的 check/test 原文被丢弃；receipt 只有总数/摘要，未保存每 case 原始 native 输出/产品 hash/源合同 hash，也只要求 total>0。失败在写 receipt 前 throw。旧 verification.md 的完整 hash 声称与实际 receipt 内容不符，gate 只 hash rust-lld 和 shim。当前本轮另存日志/hash，不代表已修好了 work gate。
- SD-02 写“char/str/List 在新 profile 合同层已冻结”，但 List 明确是非目标/后续 owner；只能写 text/scalar record 共同合同，List 保持未冻结。SD-01/03 也不能把本轮未执行的八码/全路径清理全部写为探针实证。
- `touched_goals: []` 尚无解释；应明确此计划不推进/关闭全语言或自举目标，只交付 Rust 原型候选合同，或填确有影响的现存 Goal ID。不能因 GOAL-017 已达成的 AAVM 谱系而暗示 AC/ACC native 自举。

修复：多行 warning 检测带真实正反控制；保存失败与成功的可复验输出和 hash；保持 contained executor，按 60s/locked 合同执行；有可观察的 V06 前驱 oracle；修正 SD 和空目标解释。需要扩展 helper 时仅为当前门禁证据服务，不另建通用框架。

## 逐项覆盖与知识增量

| AC | Task | 实际覆盖 / 结论 |
|---|---|---|
| AC-01 | T-01/02/05 | partial：输入/七决定结构齐；D-02/03/04/05/07 尚有上述冲突/缺项，不能判全决定完成 |
| AC-02 | T-03 | partial：V02 五类平面 Win x64 ABI、参数/字段异质顺序控制、嵌套标量调用、out-return 均 pass；771 所需 nested record 合同未覆盖 |
| AC-03 | T-03/04 | fail：基本借用/move/clone/故障/trap pass；来源前门、实际释放口径/LIFO/全路径以及资源账本不满足 |
| AC-04 | T-02/03/05 | fail：冻结 Unicode 值、范围、0/1/4096 与预算16两侧 pass；独立严格 UTF-8/空串 pass；预算恢复反例失败，下游仍有必需前提缺项 |
| AC-05 | T-06 | partial：零产品/registry/schema/source diff、旧 gates 全绿；SD 误称、门禁/持久取证待修 |
| AC-06 | T-07 | fail：本轮独立重构已执行并记录；存在合同内缺口，不勾最终通过任务 |
| AC-07 | T-08 | pending：merge-owned，未执行且不预勾 |

D-01 精确源 i32 与显式长度转换基本通过；char 的独立名义类型仍是合同，非探针类型系统实现证据。D-02 基本 Unicode oracle 通过但 literal 决定缺项；D-03/04 partial；D-05 平面 ABI 通过、嵌套布局 partial；D-06 fail；D-07 partial。无把 full ownership、GC、公开 ABI、List/enum/use、生产 I/O 或 ACC 桥新增为本轮验收。

SD-01 add 目标原先不存在；SD-02/03/04 目标在主检出当前版本无并行语义漂移，hash 见输入表。提案边界总体正确（不宣称生产 feature），但 SD-02 的 List 状态错误及依赖未完成合同阻止 delta 通过。规范增量仅冻结待修，没有发布 canonical 或派生 ledger。

## 路由

`needs_fix → executing`；T-01 基线调查保持完成；重开 T-02..06，T-07/08 仍待办，`current_step: 1 / total_steps: 8`。原 work 历史记录不删、不把其“pass”当最终验收。六个 finding 登记至 KNOWN-DEBT-AND-RISKS，但不授权延后合同内缺口。

next：`/auto-plan:work` 在同一 tree 修复，先解决 D-02/03/05/07 与下游合同，再补模型/资源/全路径 oracle/门禁，提交新基线，重放本包合法控制和四个失败控制后重新独立 review。语义变化按 new/revision 与用户已授权边界处理，不能自改既有承诺；本轮只受托复审，不直接实施或合入。

pending_closeout_gates：实际 Spec/导航/ledger/index 沉淀、归档和任务总数、合入授权、批回归到期判定、wt-guard/同树清理。它们不替代以上 work-owned correctness。
