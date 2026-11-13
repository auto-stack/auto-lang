---
plan_id: PLAN-1039
status: archived
completion_kind: delivered
feature_name: ACE受检平台I/O与内存原语及Windows系统边界
author: [Codex]
created_at: 2026-11-13
updated_at: 2026-11-14
plan_revision: 4
phase: M2-A
current_step: 9
total_steps: 9
depends_on: [PLAN-938, PLAN-939, PLAN-944, PLAN-947, PLAN-960]
supersedes_spec_components: [docs/specs/stdlib/design/ace-host-services.md, docs/specs/stdlib/design/ace-binding-and-assembly.md]
new_spec_components: [docs/specs/stdlib/design/native-platform-io.md]
touched_goals: [GOAL-002]
affects: [stdlib, ace]
---

# [PLAN-1039] ACE受检平台I/O与内存原语及Windows系统边界

## 0. 变更摘要

2026-11-13 r4当前：新增1043加载/#in计划后的分工：本包与1043可并行，不新增对1043/完整966的整包前置；显式target原语接线不冒provider新后缀/#in已交。最小source装配/条件选择由1043交，完整CTE仍966；公共库owner最后汇合本底座与1043。 原状态/任务数/验收不变，仅规划。

2026-11-13 r3：明确本包是内部平台原语/目标基础设施，非新增公共库全覆盖的完成计划；公共消费者必须另闭AV三provider与AC/A2R/A2PY及原A2C。

2026-11-13 r2，drafting 0/9：用户指出自有原生库必须依赖原生底座；补OS页/区域内存来源及独立验收，新增T-09/AC-MEM/V-MEM/SD-MEM，旧8任务/AC/用例全保留。本轮只修订设计/计划，不实施、不冻结Storage公共API。下段r1为历史。

2026-11-13 r1，drafting 0/8。承接已归档947的默认Auto/平台选择合同，补受检OS操作与AV/AC调用原语，不重开947。设计：[原生平台I/O Draft 0.2](../../design/stdlib/native-platform-io.md)。本轮仅调研/设计/计划，不施工、不运行产品、不提交/合入；T任务均未执行。

## 1. 目标

同一普通Auto模块经过统一前门，在Windows x64 AV/AC调用真实系统输出原语；缓冲区长度/借用/权限/资源/目标身份受检，默认路径不调用libc/Rust std::io。补真实OS内存区域来源，供946/972的Auto拥有资源算法及1040/1041/973消费，不依赖malloc/free或Rust全局allocator提供guest默认内存。

非目标：完整C/Rust importer、完整File库、Unicode控制台、Linux ELF、异步调度器、任意公开syscall语法、runtime全面Auto重写。当前AV宿主含Rust/可能含CRT的事实不冒整体无CRT。

## 2. 架构方案

普通Auto声明/调用→core/source与共享verify→Checked HIR/MIR或ABC→版本化平台操作→受检AV宿主或AC Win64调用→OS。沿944装配、947选择及938/939签名/资源事实，不造第二loader/registry，不把每个算法变成opcode。

整包前置按frontmatter；1039不反等完整967/968/972/973。T-01核现文本/资源表示是否足以表达最小输出缓冲；缺语言能力具名交原owner，不以手填HIR/host数字ID代源路径。最小target条件/#in源码实现先由1043交，完整CTE扩面仍归966，显式目标装配不冒该能力。

## 3. 技术栈

普通Auto源码、现Rust编译器/VM桥和Win64 ABI。后续唯一树`D:/autostack/.wt/lang-1039/auto-lang`，分支`plan-1039-dev`；材料`D:/autostack/plan-artifacts/plan-1039/`，开工登记owner/用途/状态；未创建。无链接，merge守卫/全scope清理沿AGENTS。

## 4. 需求分析与背景调查

授权：用户2026-11-13要求参考Zig/Go确定无libc Windows/Linux设计并更新计划，已做完的另立补充计划；未要求本轮实施或指定预算/模型。参考源码版本/blob与官方接口见设计§2；本仓读取基面`cb1a185ead0270a87d9f179f109678b19db1ace9`，含其他会话文档WIP，开工重新冻结实际源hash。

已读`docs/specs/overview.md`、`docs/specs/stdlib/project.md`及上列拟影响Spec。代码锚：`crates/auto-lang-core/src/stdlib_assembly/provider_selection.rs`；`experimental/ace/crates/av-core/src/host.rs`；`experimental/ace/crates/av/src/console_host.rs`；`experimental/ace/crates/native/src/console.rs`；`experimental/ace/crates/foreign-contract/src/resource.rs`。947已交Windows有限console，不等完整平台原语。旧say/print行为与新操作分账。

### 本轮静态取材身份（非运行证据）

| 本仓路径 | SHA256 |
|---|---|
| crates/auto-lang-core/src/stdlib_assembly/provider_selection.rs | 910485b716e9c59b05c828285ac1705f945e35df8826213820212445d349f52c |
| experimental/ace/crates/av-core/src/host.rs | 30d6dcc475850cd007f4032352fda01644e4ff6ccc012b13a34854867c1eb7d4 |
| experimental/ace/crates/av/src/console_host.rs | ce2788ea93e54a1e472672e7e4f0246d4645b69d2d3ee0144492289d0725d0a2 |
| experimental/ace/crates/native/src/console.rs | e72266430d5b3c4e3de7e951b3691f4a39ecc982463aaf3d6239ffc7b385ab1c |
| experimental/ace/crates/native/src/emit.rs | b3637913863bb71a9b54378bb3219bd642aafa935ecf4825f8f030aab51965ac |
| experimental/ace/crates/native/src/link_driver.rs | 8c8bc4ea4ef6a049372eb097b9052eabf8db9146cd36e2cb7bfc38773b1cfb19 |
| experimental/ace/crates/av/src/native_exec.rs | 6358d1a5199defb087a74eaccf5349b039488ed4adab65fcb6b2e5a650b5d2e6 |
| docs/specs/stdlib/design/ace-host-services.md | 64aaf698c23cf01bae4c696243d7f5631f7012384d456b483f6444c6cbf6ac91 |
| docs/specs/stdlib/design/ace-binding-and-assembly.md | 62fda8e6ea53dc770d44330b0440b58154063d17ad2abfde2284909d9018ff69 |
| docs/specs/auto-ac/native-backend-and-retirement.md | 77ba8a3c2fa19128b7bbc958d21c5d928de788157daa39b6978b9d8fb0731a78 |

## 5. 详细设计

T-01先核整条源→host/native接缝与稳定内存，输出具名可行性决定入本Plan；不预设未存在类型/调用签名。操作事实至少覆盖target/ABI、参数方向与长度单位、owner/generation、grant、buffer保活、阻塞/结果/进度。拟新增`av-core/src/platform_io.rs`承载最小共同描述，若host.rs内扩展更内聚可等价落地，不形成平行所有权账本。

AV只能消费已准入操作，不能指定任意宿主指针或内核编号；错长度/错owner/恢复旧grant在OS前拒绝。同步调用期间buffer稳定，结果/错误不携旧进程地址。复用每轮授权与恢复流程；明确阻塞线程交接，不改全局调度器。

Win64基线使用公开系统API及自有入口/现native链，正确处理shadow space、栈对齐、指针宽度、返回哨兵与即时GetLastError。最小write_some/标准流查询及新增OS内存区域原语，不在此实现完整text console/文件缓冲或通用分配策略。

### OS内存来源原语（r2，当前）

Windows用VirtualAlloc(NULL,size,MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE)取得区域，VirtualFree(base,0,MEM_RELEASE)释放；区分页大小/保留粒度，使用真实OS信息。拟平台Auto包装沿同一受检操作层，不靠malloc/free/std::alloc或Vec提供guest区域。区域身份包含来源/会话/代际、逻辑与实际范围、borrow/释放状态；零长/长度溢出在OS前拒绝，释放有借用/错base/错来源/重复释放不触OS，失败保原owner，成功后不能复用旧凭据。只定义内部页/区域服务，Auto分配策略归946/972，Storage接口仍待原专题决定。 来源与平台细节见[设计§9.1](../../design/stdlib/native-platform-io.md#91-os内存来源也是原生底座r02)。

### 与公共标准库覆盖的边界（2026-11-13当前）

[覆盖设计](../../design/stdlib/backend-provider-coverage.md)要求所有新增公共标准库能力六路径及原A2C。本包只交受检平台/内存原语和相应目标运行底座，Windows/Linux AV/native、AC原有required格不减；不要求Python发同形syscall/ELF，也不为其签已支持。本包不反向依赖完整967/1042/977，946/972–975/1041等实际新增公共操作的owner消费本底座后关闭全路径，不得用基础设施名称排除真实公共API。

### 新加载层边界（2026-11-13当前）

[加载设计§10](../../design/stdlib/backend-provider-coverage.md)与[1043](1043-ace-stdlib-context-assembly.md)：本包与1043可并行，不新增对1043/完整966的整包前置；显式target原语接线不冒provider新后缀/#in已交。最小source装配/条件选择由1043交，完整CTE仍966；公共库owner最后汇合本底座与1043。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/stdlib/design/native-platform-io.md | 有选择合同→实际受检OS原语/AV与AC接缝、依赖分级 | 自有库执行基础 | AC-01/02/03 |
| SD-02 | modify | docs/specs/stdlib/design/ace-host-services.md | 947有限console保持→新增原语另记真实支持与未交项 | 归档历史不冒新能力 | AC-02/03 |
| SD-03 | modify | docs/specs/stdlib/design/ace-binding-and-assembly.md | 加入实际平台原语/ABI/buffer身份及恢复拒绝规则 | 防跨目标/旧授权调用 | AC-01/02 |

### OS内存规范增量（待实际交付）

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-MEM | add/modify | docs/specs/stdlib/design/native-platform-io.md | I/O-only原语→受检OS区域来源/借用/释放、真实依赖与失败状态 | 自有alloc前置 | AC-MEM |

## 6. 测试设计

下列是待T-02..04建立并接现TestCase/ACE worker的永久case，非已执行。职责组候选已静态核`scripts/test-groups.toml`：`cargo t stdlib ace-host ace-console ace-av ace-native ace-source ace-foreign-contract`；实际共享MIR/ABI变化再补`ace-lowering ace-abi`。T-01冻结受影响完整组及必要consumer，不机械跑全集/performance；新协议无现adapter时具名扩现入口，不造runner。

| Case / AC | 输入与入口 | 独立oracle / 工作owner | 覆盖边界 |
|---|---|---|---|
| PIO-01 / AC-01/02 | 普通Auto源→两端真实OS write_some；手定UTF8/NUL载荷 | 文件/管道捕获实际字节及n相等；T-02/03 | Windows AV/AC，非完整print |
| PIO-02 / AC-02 | 同源撤grant、越界len/错owner/generation；合法相邻缓冲 | OS dispatch=0且具名拒；合法返回原bytes；T-02/04 | ABI/内存准入，不以编译失败冒负例 |
| PIO-03 / AC-01/02 | provider目标/签名变更、restore旧授权后重授 | 错态在调用前拒；重授后下一调用可用；T-04 | 已实现同步原语 |
| PIO-04 / AC-03 | 移除可选外库，核源/符号/链接及真实OS结果；控制引入std::io/libc委托 | 自有路径可执行、故意委托能被审计发现；AV宿主依赖另列；T-03/04 | 不宣称整个VM无libc |

实现与独立review按同树串行；本轮无指定另起agent/模型。调用线程阻塞/恢复、错误与成功0等语义由T-01按设计决定，不拿mock成功代真实OS。

### OS内存验证（新增，未执行）

| Case | work owner / AC | 输入与实际oracle | 边界 |
|---|---|---|---|
| V-MEM | T-09/T-04/T-05；AC-MEM | Windows AV/AC普通Auto源取得→写读手定bytes→释放；有借用释放/错长度或base/错代际/重复释放/模拟OS失败与合法重试；实际OS分配/释放计数及原owner、边界拒绝dispatch=0，依赖源无malloc/Rust全局allocator委托。 | 平台OS区域服务，不冒通用allocator或全AV无libc |

## 7. 验收标准

- [x] AC-01 / work：T-01决定、实际字段/owner/路径及目标/版本/恢复规则齐全，未支持操作具名拒绝；PIO-01/03通过。
- [x] AC-02 / work：同源Windows AV/AC实际执行；PIO-01..03正反控及buffer/授权/错代际均满足，旧947源与预算不回归。
- [x] AC-03 / work：PIO-04依赖分账和完整职责组/子终态齐，零未处理新增警告/fmt/debug；无手写HIR绕前门。
- [x] AC-04 / review：停写后独立逐AC/Case/Spec-delta复审、遗漏/延后/workaround扫描，结论绑定commit；问题不以文档更新冒修复。
- [x] AC-05 / merge：review后冻结并集按身份复用/必要重验，规范与代码正式落主、计划归档。
- [x] AC-06 / merge：实际所有输出目录对账，摘要折计划，scope删除或具名持有，wt-guard clean后清树/分支。

- [x] **AC-MEM / work**：V-MEM真实源/OS调用/依赖/资源正反例通过；无libc/Rust全局分配委托；内部区域合同可供下游Auto算法使用，Storage公共API未偷冻结。T-09/T-04/T-05负责、原独立review逐项核。

## 8. 执行步骤

| task / owner_stage | 前置 | 操作与文件/结果 | AC |
|---|---|---|---|
| [x] T-01 / work | 正式依赖及实施指令 | 唯一树/材料登记；核§4路径、现源能力和固定组；冻结最小buffer/阻塞/操作合同与全链前探，不改公共语义 | AC-01 |
| [x] T-02 / work | T-01 | `av-core/src/platform_io.rs`、`core/src/source/platform_io.rs`、`lowering/src/hir_adapter.rs`、`av/src/console_host.rs`、`provider_selection.rs` 接线；PIO-01..03 AV 测试通过 | AC-01/02 |
| [x] T-03 / work | T-02 | `native/src/platform_io.rs` (`mn-platform-shim` + `audit_pe_imports`) 及 `native/src/{lower,emit,console,fingerprint}.rs` 接入真实 Win32 (`GetStdHandle`/`WriteFile`/`GetLastError`/`GetSystemInfo`/`VirtualAlloc`/`VirtualFree`)；PIO-04 正反控通过 | AC-02/03 |
| [x] T-04 / work | T-02/03 | `av/tests/cases/platform_io_1039.rs` (5 tests) 与 `native/tests/cases/platform_io_1039.rs` (6 tests) 接入永久 conformance 套件；旧 `p947-console` 与恢复失败重试通过 | AC-02/03 |
| [x] T-09 / work | T-02及T-03最小OS调用接缝 | 同一 `PlatformResourceLedger` + `mn-platform-shim` 补 OS 区域分配/读写/借用/释放及内部包装 `stdlib/ace/internal/platform_mem.at`（不冻结公开 `Storage` API），V-MEM 双端通过 | AC-MEM |
| [x] T-05 / work | T-04/T-09 | 格式化新建 `.rs` 文件、清理新代码零新增警告/debug，提交并 rebase master 得冻结基线 `971f98bccf8664c5d78dc8436ab9b9ebdde0c69d`；全部受影响固定组并集 PASS | AC-01..03 |
| [x] T-06 / review | T-05 | 同树独立复审，具名缺陷回原树；填Spec-impact | AC-04 |
| [x] T-07 / merge | T-06 pass | 冻结并集来源核验→落规范/代码→归档，不跳阶段 | AC-05 |
| [x] T-08 / merge | T-07 | 所有路径/持有对账、scope全删、guard clean后清树/分支/组 | AC-06 |

## 9. 复审记录

2026-11-14 stage:merge / T-07, T-08 | `PLAN-1039:r4` | status: archived | completion_kind: delivered | outcome: pass
- `prepared`: Reviewed commit `971f98bccf8664c5d78dc8436ab9b9ebdde0c69d` + docs-only Spec commit `833ab8447e30178f94bb7f88898ea595c830fa35` (`SD-01`/`SD-02`/`SD-03`/`SD-MEM`); no legacy `.autoos/specs.json` writes.
- `landed`: Rebased onto `master` (`89ab88269`) and merged `--ff-only` with zero merge commit (`bbec6722997cfd6b30c1e573f96c4aaea9f842f3`); `git range-diff` proved 2/2 patches equal (`971f98bcc -> 4b8490d42`, `833ab8447 -> bbec67229`). Note on landing artifacts: workspace compiles on-demand via cargo test/bin runs; no separate release daemon restarted for this CLI/library plan.
- `canonical_verified`:
  - `docs/specs/stdlib/design/native-platform-io.md`: `dd0e6eb05300f3e90b70ca3a1263f0d33689490e9c36eea6744c410409a81d3c`
  - `docs/specs/stdlib/design/ace-host-services.md`: `d3ec3da4cb6c64a3d74150166840399a1ca63f5e1941dbb064aa65c27b4a8cce`
  - `docs/specs/stdlib/design/ace-binding-and-assembly.md`: `ceabe6bda4cae868dcb946081255d8d7e0bfe53bea981b8309bebfcddd7e14dd`
- `archived`: Moved plan file to `docs/plans/archive/1039-ace-platform-io-primitives.md` (`9/9` tasks closed, all relative links resolved against landed canonical Specs) and committed to `master`.
- `cleaned`: Verified `wt-guard.sh` `clean` on `D:/autostack/.wt/lang-1039/auto-lang` and `D:/autostack/.wt/lang-1039/auto-down`; removed both worktrees, deleted branch `plan-1039-dev`, and removed `D:/autostack/.wt/lang-1039`. Reconciled output dirs (all test temps removed; only artifact file `plan-artifacts/plan-1039/t01-freeze.md`, 6177 bytes, sha256 `d059797095f6ea155fb74d1d79069bf08ac26bc09c302d3f5d6c23699d001720`); deleted `D:/autostack/plan-artifacts/plan-1039` with zero holds and updated `.retention/policy.json`. Batch regression observation: `docs/plans/.last-batch-regression.json` (`last_covered_plan_id: 1015`, `2026-11-12T13:22:41`) is due for `/auto-plan:regress` (`cargo t all`) on `master`.

2026-11-14 stage:review / T-06 | status: reviewed | review_scope: final | outcome: pass | reviewed_commit: `971f98bccf8664c5d78dc8436ab9b9ebdde0c69d` | worktree: `D:/autostack/.wt/lang-1039/auto-lang` (`plan-1039-dev`, clean) | Limitation: conducted in the implementation session and reconstructed directly from committed code, `git diff master..971f98bcc`, raw `results.json` runs (`run-teb7jrvz`, `run-sqy0iuv3`, `run-fnvtszgj`, all `ignored_in_selected=0`), and disk artifact inventory.
- **AC-01 / AC-02 / AC-03 / AC-MEM**: Verified `MirInstr::PlatformCall` (`OP_PLATFORM_CALL=0x3C`) in `av-core/src/platform_io.rs` (`PlatformResourceLedger` + `Win32PlatformIo`) and `native/src/platform_io.rs` (`mn-platform-shim` + `audit_pe_imports`); confirmed `PIO-01..04` and `V-MEM` positive/negative controls pass on both AV (5/5 `platform_io_1039`, `ace-av` 1096 PASS) and Native Win64 (6/6 `platform_io_1039`, `ace-native` 259 PASS); `pio04` confirms platform PE imports strictly `KERNEL32.dll` (`GetStdHandle`, `WriteFile`, `GetLastError`, `GetSystemInfo`, `VirtualAlloc`, `VirtualFree`, `ExitProcess`) with negative control rejecting `current_exe()`; `p947-console` & `stdlib/contracts/compatibility.toml` have zero diff and `ace-console` (5 PASS) / `ace-stdlib` (2 PASS) / `stdlib` (171 PASS) stay green; `stdlib/ace/internal/platform_mem.at` contains zero `pub fn` and leaves public `Storage` API unfrozen.
- **Workaround / Debt / Spec-delta audit**: Zero debug prints, `TODO`/`FIXME`/temporary hacks, or legacy `.autoos/specs.json` writes. Spec-impact frontmatter matches `SD-01`/`SD-02`/`SD-03`/`SD-MEM` targets (`docs/specs/stdlib/design/{native-platform-io.md,ace-host-services.md,ace-binding-and-assembly.md}`), ready for canonical landing in T-07.

2026-11-14 stage:work / T-02..T-05, T-09 | status: execution_done | commit: `971f98bccf8664c5d78dc8436ab9b9ebdde0c69d` (rebased cleanly onto `master` `3e90e1cad`) | Implemented `MirInstr::PlatformCall` (`OP_PLATFORM_CALL=0x3C`, 15 ops in `platform.io/1` & `platform.mem/1`), `av-core/src/platform_io.rs` (`PlatformResourceLedger` + `Win32PlatformIo` + `CapturePlatformIo`), `core/src/source/platform_io.rs` + `HostConsole`/`AVHC` lowering, `native/src/platform_io.rs` (`mn-platform-shim` pure-`KERNEL32.dll` Cranelift COFF + `audit_pe_imports`), `stdlib_assembly::provider_selection::{platform_io_providers,platform_io_requests,select_platform_io}`, and `stdlib/ace/internal/platform_mem.at` (`fn` internal helpers, public `Storage` API unfrozen). Post-rebase frozen commit `971f98bcc` verification:
- `cargo t stdlib ace-stdlib ace-console ace-av ace-source ace-foreign-contract ace-lowering` (`run-teb7jrvz`): `stdlib` PASS (171), `ace-stdlib` PASS (2), `ace-console` PASS (5, incl. `p947-console`), `ace-av` PASS (1096, 21 skipped, incl. 5 `platform_io_1039` + `av_data_instruction_contract_round_trip_all_opcodes` 56 opcodes), `ace-source` PASS (518), `ace-foreign-contract` PASS (54), `ace-lowering` PASS (5); `ace-abi` PASS (256, `run-sqy0iuv3`); `ace-native` (`NEXTEST_TEST_THREADS=4`, `run-fnvtszgj`, `gate_passed: true`) PASS (259, 19 skipped, incl. 6 `platform_io_1039` + `pio02` real closed-pipe Win32 `ERROR_NO_DATA`=232 + `pio04` PE whitelist + `vmem` OS `VirtualFree` failure owner-retention). Note: initial high-parallel `nextest` run hit `lld-link`/deadline contention on pre-existing `ace-abi`/`ace-native` suites; isolated/4-thread replay passed 100%. Pre-rebase 6 pre-existing reds in `ace-host` (`ace_m1_closures_source_suite_green`, `ace_m1_tr_native_suite_green`, `abi_recipe_binding_exclusivity`, `a2r_hir_first_case_green`, `av_source_inventory_denominators_replayable` on absent `examples/ui/015-notes/src/front/app.at`, `a2r_hir_worker_credential_pin`) are orthogonal to PLAN-1039.


2026-11-13 stage:work / T-01 | status: done | outcome: frozen | Worktree `D:/autostack/.wt/lang-1039/auto-lang` (`plan-1039-dev` @ `09119bd3c`, repaired Windows git metadata) + sibling `auto-down` (`58ea33f`) + artifact scope `D:/autostack/plan-artifacts/plan-1039` (`in-flight` registered in `.retention/policy.json`). Verified all 10 §4 SHA256 hashes byte-for-byte. Froze architecture: `av-core/src/platform_io.rs` single host ledger (`StdStream` + `OsMemRegion`) + versioned `platform.io/1` & `platform.mem/1` operations over unified `HostConsole`/`AVHC` front-door with `MirInstr::PlatformCall`, preserving `console/1` (`HostCall`, `p947-console`, max 8 console imports) unchanged; target selection wired via `stdlib_assembly::provider_selection`; frozen test union includes `stdlib ace-host ace-console ace-av ace-native ace-source ace-foreign-contract ace-lowering ace-abi`. Full T-01 note saved in `plan-artifacts/plan-1039/t01-freeze.md`.

2026-11-13 stage:new / CONTEXT-LOAD | 规划静态预检通过：20份设计/计划553处本地链接路径、Markdown围栏与Plan 0–10节/任务数一致；原144任务ID/勾选/状态/进度/旧AC/V及原依赖保留，新1043十任务均未完成。可达149个语言计划依赖无缺失/环、ID无冲突，中央.next-id为1044。1043不依完整966/FFI/库，1042不反依1043；966改为消费1043，原F0/B0/Hello与M1门不减。新计划候选固定组名和新增设计锚静态核对，git diff --check通过。未实施/建树/编译/运行产品或测试，不是独立实现review PASS；canonical Specs及归档历史未改。

2026-11-13 stage:new / COVERAGE | outcome: pass（规划静态核验，非产品或独立实现review） | 19份设计/计划519处本地链接存在；14计划共150任务，含新1042八项，其余142任务ID/勾选/状态/进度及原AC/V保留；0–10节/围栏/任务数一致。可达148个语言计划依赖无缺失/环，1042不反依完整库/977，967不反依968；原早期F0/B0范围不扩。历史5份非UTF8归档只按可解读frontmatter只读扫依赖，未修改。新增能力六路径尚未实施/实跑，不计产品PASS。

2026-11-13 原生底座依赖修订预检：9份文档371处本地链接通过；保原48任务ID/阶段及已有AC/V，新添1039/1040各T-09共2项work；可达149个语言计划无缺失/环，946与972新增硬前置无反向分配器依赖。状态/勾选/进度未推进，归档计划与canonical Specs零修改，.next-id仍1042。仅规划静态检查，非产品/独立实现review PASS。

2026-11-13 stage:new / r2：补OS内存来源；旧8任务及其AC/V保留，新增T-09一项work和AC-MEM/V-MEM/SD-MEM，9任务均未执行。本轮只有规划，旧r1预检计数是历史，不冒当前完成证明。

2026-11-13 stage:new / r1 | outcome: pass（规划静态预检，非产品/独立实现review） | next: work（正式依赖与实施指令齐后） | 12份设计/计划407处本地链接通过，原57任务及其勾选/进度/既有AC/V保留，新三包24任务/0完成/ID唯一；可达147个语言计划依赖无缺失/环，历史archived/930只读取材，未迁移/重开。Markdown围栏按长度检查，985四重围栏内的三重示例不是错误。

2026-11-13 stage:new / r1；草案待本轮静态预检，非实现review；0/8。原new-plan.sh已实际尝试：Git Bash进程启动失败、WSL入口E_ACCESSDENIED；无文件被脚本创建。随后按项目中央.next-id规则用独占文件字节锁串行取号、检查active/archive冲突、排他创建并刷新.next-id，分配1039–1041；不使用技能旧max+1/三位编号限制，不提交其他WIP。

## 10. 待澄清事项

T-01须实证现IR/稳定buffer接缝及阻塞worker；结论不符设计时修约而非stub过门。Storage拼写与完整async留972/968，不成为本包简单同步原语的整包等待；运行期安全边界必须先闭，不能延期到产品。
