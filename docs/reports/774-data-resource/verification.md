# PLAN-774 验证收据（verification）

- 计划：PLAN-774 r3 · T-01..T-06（work 阶段，AC-01..05 证据）
- 工作树：`D:/autostack/.wt/lang-774/auto-lang` @ `plan-774-dev`
- 基线：工作树基面 `5e21561d2c303a02319ff7a29e4607933e09a38f`（= master）；
  T-06 收据时 HEAD `69d3832c48a371c31256c69dd957e959986a43dc`
- 日期：2026-10-13
- 性质：探针实证收据。**探针是 Rust/Cranelift 可行性证据，不是 Auto native
  编译证据**；语言实现状态不变（AC-05）。

## 1. 交付物与提交

| 提交 | 内容 |
|---|---|
| 85a4c482f | T-01 基线刷新 + decisions.md 事实差异表（46/47 输入零漂移；00-intro 纯簿记漂移） |
| 51f3a82e9 | T-02 D-01..07 决策冻结 + probe-cases.md oracle 冻结 |
| f5a91c42e | T-03 探针 crate（harness + shim staticlib）+ V02/03/07/08 用例 |
| 590e44a78 | 探针 Cargo.lock 入库（--locked 门禁） |
| 736e5a5da | T-04 V04/05/06 用例 + rules.rs 前门模型 + PS 门禁脚本 |
| 69d3832c4 | T-05 interface-contract.md 冻结 + decisions §4 回填 |

## 2. 命令与结果（唯一工作树）

| 命令 | 期望 | 实际 | 收据 |
|---|---|---|---|
| `cargo check --manifest-path experimental/ac-core/probes/data-resources/Cargo.toml --locked --all-targets` | 零错误零警告 | PASS（零警告健康检查在 PS 门禁内） | gate-*.txt |
| `cargo test --manifest-path … --locked` | 全绿 | **39/39 通过**（smoke 1、v02 8、v03 5、v04 6、v05 10、v06 2、v07 5、v08 2） | gate-*.txt |
| `cargo fmt --manifest-path … --check` | clean | PASS | gate-*.txt |
| `powershell … scripts/verify-ac-data-resources-774.ps1` | 6 步全绿 | PASS（context/check=零警告/test=39/fmt/scope/hashes） | `probes/data-resources/target/ac-verify-receipts/774/gate-20261013-125204.txt` |
| `powershell … scripts/verify-ac-741.ps1 -SkipMainGates` | 13 步 PASS | **all 13 steps PASS** | 741 收据目录 |
| `powershell … scripts/verify-ac-source-745.ps1` | 全绿 | 见 §5（收据回填） | 745 收据目录 |
| `powershell … scripts/verify-ac-scalar-767.ps1` | 全绿 | 见 §5（收据回填） | 767 收据目录 |

所有 PE/子进程复用 ac-core `link::run_deadlined_split` 受控执行器
（CREATE_SUSPENDED + Job 约束 + 单一 60s 截止 + 全量输出收集）；V06-2 超时
分类控制用显式 5s 截止。宿主测试未直接调用 shim 冒充机器调用（全部断言来自
真实 PE 的退出码 + stderr trace）。

## 3. 用例索引（probe-cases.md §V-01..09 → 物化）

| Case | 物化位置 | 结果 |
|---|---|---|
| V-01 溯源审计 | decisions.md §2/§3 逐决定三元组 | 表齐，无无证据默认 |
| V-02 五类 ABI | tests/v02_abi.rs（8 测：含异质顺序 2 控制、字段交换控制、寄存器/栈边界、嵌套、out-return） | 全绿 |
| V-03 寿命计数 | tests/v03_lifecycle.rs（5 测：含双清理探测控制、分支早退双 PE、循环） | 全绿 |
| V-04 前门拒绝 | tests/v04_reject.rs（6 测：M1/M2/M5/M3/M4 + R2 静默错读淘汰证据；台账断言 rejected op native=0；合法兄弟例真实 PE） | 全绿 |
| V-05 故障注入 | tests/v05_fault.rs（10 测：AP-1..4、零长、concat 溢出先于读、转换边界、shim 错误、成功反控、borrow_read 界、类别矩阵） | 全绿 |
| V-06 trap/超时 | tests/v06_trap.rs（2 测：AV=0xC0000005 + TRAP-PRE + **无 SUMMARY**；hang → Err(Deadline) 5s） | 全绿 |
| V-07 文本边界 | tests/v07_text.rs（5 测：3/8、2/3、1/1 计数；非法 UTF-8=-7；char_at 65/20013/128512；slice 正反；码点转换±） | 全绿 |
| V-08 规模/预算 | tests/v08_scale.rs（2 测：N=0/1/4096 精确计数；预算 16 恰过/超一拒/释放回收） | 全绿 |
| V-09 隔离 | PS 门禁 scope 步（diff 基面..HEAD 限 probe/脚本/报告路径）+ §4 旧门禁重跑 | 全绿 |

oracle 冻结纪律：全部期望值在 T-02 手算冻结（probe-cases.md），T-03/04 实现后
一次通过；唯一实现期修正是 harness 侧违规标志极性 bug（`expect_eq_i32` 相等时
误报 1——测试基础设施缺陷，非 oracle 变更）与规则模型 EndView 归零回 Owned 的
状态机补全（模型缺陷，拒绝码/oracle 不变）。无任何 expected 从探针输出反写。

## 4. 关键实证结论（供 770/771 与复审）

1. 五类内部 ABI 在真实 COFF/PE 上可稳定实现（Win x64 寄存器/栈边界、指针 RAX +
   长度 out 通道）；异质参数顺序与字段交换均被观测通道识别。
2. R1 显式拥有/借用全路径可行：live=0、alloc=free、move 单清理、clone 独立
   （ptr≠ptr、peak=32）、借用不释放 owner；0/1/4096 规模无漂移。
3. R2 引用计数句柄**淘汰**：drop 后复用句柄静默错读回收槽位（v04_r2 实测
   v=34、无错误码），无前门性质。
4. 故障面完备：-1..-6/-11 与文本 -7..-10 全部实测；失败跨 ABI 为值（exit 0）、
   已分配资源按合同回收；预算按存活字节、释放即回收。
5. fatal trap：AV 退出 0xC0000005、前驱 trace 保持、清理未执行（无 SUMMARY）；
   死循环被执行器分类为 Deadline。不声称析构/unwind。
6. 文本规则可实现：byte/scalar 双计数不混；组合字符 2 标量；NUL 合法；
   空/反/越界范围三态码分离；非法 UTF-8 与非法标量分阶段分码。

## 5. 旧门禁重跑（V-09）

| 门禁 | 结果 |
|---|---|
| verify-ac-741.ps1 -SkipMainGates | **all 13 steps PASS**（2026-10-13，工作树 69d3832c4） |
| verify-ac-source-745.ps1 | **all 13 steps PASS（74 cases）**（2026-10-13，同一 HEAD） |
| verify-ac-scalar-767.ps1 | **14 steps, 0 failed（45 cases）**（2026-10-13，同一 HEAD） |

未运行根 `cargo t`/`cargo tv`/`docs_gen`：无 crates/root 定义改动（scope 干净，
PS 门禁 scope 步断言）。

## 6. 哈希绑定（收据时）

```
worktree HEAD: 69d3832c48a371c31256c69dd957e959986a43dc
base (master): 5e21561d2c303a02319ff7a29e4607933e09a38f
decisions.md            5b553756b81e14ea…
probe-cases.md          a6938207659b7648…
interface-contract.md   79e4fbb595e10314…
probe src/lib.rs        7b38dcc61f0072c7…
probe src/rules.rs      b2e79ad92e6fb0ea…
shim  src/lib.rs        c468208c4d4cb909…
PS gate script          079f533eb2e70789…（修订计数正则后另见最终收据）
probe Cargo.toml        647329be2cf61f77…
rust-lld (stable toolchain) f436ddda2f519c5a…（完整值见 gate 收据）
shim staticlib artifact     fcea5572e8848b00…（同上）
```

（16 位前缀；完整 SHA256 在 `probes/data-resources/target/ac-verify-receipts/774/gate-*.txt`。）

## 7. 遗留与边界

- canonical Specs 未发布（SD-01..04 见 proposed-spec-delta.md，merge 时沉淀）。
- 探针 staticlib 链接需要 freestanding memcpy/memset/memcmp/memmove（shim 自带
  volatile 字节循环实现）——这是 no-CRT PE 链接的通用事实，770 落地 runtime 时
  同样需要，已记入 interface-contract 消费面。
- 后续 owner：text-record 组合、List/enum/use、生产 ABI、ACC 桥（Design 36 §7）。

## 8. r1 复审修复轮（2026-10-13，needs_fix → executing 修复后）

复审报告：[REVIEW.md](../../plans/attachments/774/review-r1/REVIEW.md)（六发现
P774-R1-01..06）。本轮修复与证据：

| 发现 | 修复 | 证据 |
|---|---|---|
| R1-01 账本容量/预算 | shim 改堆支持可增长完整外部账本（键=指针，无容量上限，增长失败按 env-oom 回滚）；check_invariant 收紧（live_bytes==0 ∧ invalid_frees==0）；budget 0=零字节预算、负值清除 | v08_capacity_64_and_65 / v08_budget_restored_after_65 / v08_zero_budget_and_negative_clears |
| R1-02 来源/入口 | rules.rs 来源追踪（LocalOwned/ParamBorrowed/ParamOwned/Viewed(base)/MutBorrowed(base)）；ReturnViewParam 验证存活借入来源；field-borrow/container-borrow/ownership-required 均有可达入口；真借用返回 PE（(ptr,len) 跨帧+调用方读取）；M5 兄弟真实 PE | v04_m4b/m4c/m5_sibling_real_pe/field_and_container_entries/move_param_mode_requirement |
| R1-03 LIFO/全路径 | FREE id 顺序 oracle（LIFO+IFO 可区分控制）；嵌套 break/continue 清理；临时语句尾（使用先于清理 trace 序）；move-in callee 清理+return 转移；clone 内容独立（原释放后 BSUM=409）；AP-3/4 带 owner 注入 | v03_6..v03_10、v05_13 |
| R1-04 合同缺项 | decisions §3.8：Str 参数默认 view+三显式模式（对齐 Design 04/F8）；身份表唯一化+下游改齐责任；源级拼写 to_codepoint/from_codepoint；字面量/跨行/转义冻结表（5 码）；嵌套 record 递归布局+rec2 物化；R2 理由限定+checked 对照 | decisions.md §3.8、interface-contract §7、v02_9、v04_r2_checked_variant_detects_stale |
| R1-05 释放口径 | frees=成功释放（账本+HeapFree 非零）；FREESTAT 行分离 free_attempts/invalid；双释放不读已释放内存 | v03_2n（exit 1 + FREESTAT 2/1/1 + FREE-INVALID） |
| R1-06 门禁/收据 | PS 门禁：逐行警告匹配+内置正反控制；check/test/fmt 原文全量落盘（raw-*）；失败也写收据；shim 构建 60s+locked；V06-2 前驱 trace 文件重定向可观察 | gate-20261013-*.txt + raw-*/{check,test,fmt}.txt、v06_2 |

**本轮结果**：探针全套 **55 测试全绿**（v02 9、v03 10、v04 12、v05 11、v06 2、
v07 5、v08 5、smoke 1）；**复审反例重放 7/7 全绿**（含原 4 失败控制——
`powershell docs/plans/attachments/774/review-r1/replay.ps1` exit 0）。收据时 HEAD
与哈希见最新 gate 收据（含 probe 源/报告/gate 脚本哈希与 shim 工件哈希）。
本节哈希以 gate 收据为准（旧 §6 的 16 位前缀对应修复前版本）。

**旧门禁重跑（本轮修复后）**：741 `-SkipMainGates` 13 步 PASS、745 13 步/74 例
PASS、767（772）14 步/45 例 PASS（2026-10-13，同一修复 HEAD）。最终门禁收据：
`gate-20261013-134941.txt` + `raw-20261013-134941/`（check/test/fmt 全文）。

**遗留模型缺口（明示）**：规则模型未建模显式 `mut str` 借用参数（D-03 声明合法，
模型内拒绝 mut-conflict 并注释）——模型保守性说明，非规则声明；770 实现时以其
Spec 为准。
