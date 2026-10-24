# PLAN-827 T-01 合同决策（r1，冻结于 2026-10-24）

基面：master `369e1c349`（唯一实施树 `D:/autostack/.wt/lang-827/auto-lang` @分支 `plan-827-dev`，与基面同 commit）。
依赖门：**D798 已归档**（`docs/plans/archive/798-typed-mir-abc-av-probe.md`）、**D828 已归档**
（`docs/plans/archive/828-av-function-incremental-session.md`，merge 收据 11/11，ledger v70）。
本文按 T-02 刷新义务**直接写在 D798+D828 合入后的基面上**——不存在更早的 pre-D828 版本需要刷新，
r1 即刷新后版本。

## 1. 复用裁定（§2 有界复用决策，冻结）

**结论：D798 已交付的 source→AV 入口已覆盖本计划 G1 全部执行需求。本计划不新增
`experimental/av-source-probe/` workspace、不新增 worker 命令（无 `run-source`）、
不注册新 recipe（无 `av/source-run-v1`）**——执行计划 §2 的"若已覆盖，T-03 只登记
新程序清单/薄适配及反控"分支。证据（全部在合入 master 的代码中核实）：

| 需求（计划 §5.2/§6） | 已交付事实（file:line） |
|---|---|
| 真实源码文件字节 → source profile 编译 | corpus case 声明 `source: { file: …, syntax: "auto" }`，runner `resolve_case` 读盘得 bytes（`crates/auto-test-case/src/ir.rs:98`，`read_bounded` 带重解析点审计 `admission.rs:202`），父 adapter b64 入请求（`crates/auto-lang-core/src/test_cases/ac_av.rs:747`），worker 只编译请求内字节（`experimental/ac-av-probe/probe/src/bin/av_probe_worker.rs:242-258`）——**冻结快照语义天然成立**：准入后磁盘变更不影响运行 |
| source 编译→MIR→ABC→AV→typed 结果 | `Driver::request`（`probe/src/driver.rs:127`）= `compile_scalar_source`（`ac-core/src/source/adapter.rs:77`，profile `auto.source.scalar-i32.draft`）→ `lower_checked`（`probe/src/hir_adapter.rs:70`）→ verify → `encode_abc` → `execute_program`，typed `ExecOutcome` |
| 正式 recipe + 父侧判定 | recipe `ac-av/core-run-v1`（注册表 `crates/auto-test-case/src/contract.rs:1798`；adapter `ac_av.rs`；dispatch `crates/auto-lang-core/src/test_cases/mod.rs:756`）。oracle 三面：success+exit(i32) / error-run+message(trap code) / error-parse/verify+code（`ac_av.rs:983`）。trap→NativeRecord exit 70（`ac_av.rs:945`） |
| AP-23 整数除零 trap | `TrapKind::DivByZero.code() = "av.trap.div-by-zero"`（`core/src/contract.rs:150`） |
| 同 store fresh/hit/新 worker restart | case-serve `store_root`+`session`（共享 warm store，父侧拥有清理，`av_probe_worker.rs:279-298` / `ac_av.rs:722-725`）；跨进程恢复 `restore-run` + `worker_restore::restore_and_run`（按 manifest artifact id 重开 store、逐信封复验、闭包重组、再执行，从不收 source）；`request` 子命令=新进程整请求 |
| 源更新影响结果/依赖 | `edit-source` 步骤（exactly-once 字面量，worker 与父侧双验，`av_probe_worker.rs:305-338` / `ac_av.rs:841-861`）；每次 run 步回 `source_hash`（FNV-1a64 over bytes），父侧对请求当前修订重导并强绑定（`ac_av.rs:876-884`） |
| 反控注错缝 | `AV798_FAULT`（exit9/extra-frame/wrong-hash/delay-ms，`av_probe_worker.rs:410-456`）；负 oracle/解码负例既有先例 `crates/auto-lang/src/tests/ac_av_probe_tests.rs` |
| D828 函数级增量（T-09/V-13） | `session.rs`：`warmup`(417)/`prepare_edit`(2042)/`commit`(2404)/`PreparedEdit::{changed_name,unit_mir,content_version,code_weak}`(313-333)/`phase_walls/phase_io/resident_inventory`；持久 worker `session-serve`；先例消费者 `experimental/ac-mir-native/src/incremental.rs:18,65` |

**薄适配面（T-03..T-06 全部新增物，零公共定义写面）**：

1. corpus 目录 `crates/auto-lang/test/cases/av-source-827/`（suite `av-source-827.suite.ad` +
   case 文件 + AP 字节恒等副本 `src/*.at`，附 SHA256 回执对账 catalog）。命名沿兄弟套件
   平铺惯例（ac-av/ac-cte），计划原文 "test/cases/av-source/827/" 调整为平铺并在此记录。
2. 独立 workspace 测试 `experimental/ac-av-probe/probe/tests/av_source_827*.rs`（读**真实
   AP 原文件**路径——"现存文件冻结 bytes"在 Rust 侧直接满足；store/restart/overlay/身份/
   预算/增量轴在此，不占任何共享面）+ fixtures `tests/fixtures_827/`（变体/负控，AP 原件只读）。
3. 根 workspace 父侧门 `crates/auto-lang/src/tests/av_source_827_tests.rs`（注册于
   `crates/auto-lang/src/tests.rs` mod 表；feature `test-cases` 门内，`cargo t` 日常档
   经 `--features auto-lang/test-cases` 天然覆盖——`.cargo/config.toml:73`）：corpus 全绿门 +
   负 oracle + 准入分态负控（非法 UTF-8/缺文件=resolve/infra 态而非 guest trap）。
4. 验证脚本 `scripts/verify-av-source-827.ps1`（§5 Scope）。
5. inventory 复现物（§4）。

**写面登记（ACE 交接合同 §1/§4）**：本计划**无公共写面**——不改 `REGISTERED_PROBE_RECIPES`、
不改 dispatch/词表/共享协议、不改 ac-av recipe 语义、不改 ac-core/ac-av-probe 公共定义
（测试文件与 corpus 为新增消费面）。公共注册面当前在途 owner=PLAN-820（`av.data/1`，
未合入，contract.rs/mod.rs 同文件追加）；827 只读消费已合入稳定 API，不占核心窗口，
不与 820 相遇。若后续证据出现真实缺口需扩公共面（新 recipe 字段/协议步），按计划 §2
先回 new/窗口交接，不得就地扩。

## 2. 冻结的入口/身份/预算（实际交付值，file:line 于 `experimental/ac-av-probe/`）

- 协议 `avworker/1`，worker 版 `avworker/1.0`，AV profile `av.core-i32/1`，producer
  `plan798/av-probe`；请求 cap 4MiB（worker `:189`、父 `ac_av.rs:37`）；会话墙 60s、
  构建墙 300s（`ac_av.rs:39-40`）。
- 入口名：corpus case `entry: d_main`（`fn main() int` 的编译入口名）。
- 冻结 caps（`core/src/contract.rs`）：输入 4MiB(:54)/codec 页 256KiB(:56)/表项 65536(:59)/
  前端嵌套 128(:62)/verify 2M 步·5s(:64,66)/AV 1M 步·1024 帧·2s(:68,70,74)/槽 8MiB(:72)/
  cancel 间隔 1024(:76)/trace 64KiB(:78)/分析 16MiB(:89)/解码模块 16MiB(:93)；
  store 分区 hot8/staging16/exec8=32MiB + A 线前端瞬态 ≤224MiB + B 线残差 ≤128MiB
  （`probe/src/bench.rs:67-83`）。T-01 核对：128 层适用门=前端**token 流真实嵌套**
  （`driver.rs:1459 front_door_admit`），非表数值域；65536 为**每表**项数（mir_verify.rs:207）。
- 身份：run 步 `source_hash`=FNV-1a64(source bytes)（父侧重导强绑定）；case 身份=
  `source_sha256`=SHA256 over (virtual_path,sha256)*；worker 工具身份四轴
  protocol/recipe/exe-sha256(每次全量重读)/worker-version，stale 判定绑定本 recipe 行。
- 828 会话（T-09 消费）：`HISTORY_CAP=32`、`STAGES[11]`、`SessionCounters`；观察面
  `phase_walls/phase_io/resident_inventory/last_run_publication`；先例=ac-mn
  `incremental.rs`（warmup 一次全模块→每 edit 单 body 事务 prepare→门重验→commit→run）。

## 3. 程序分母与 oracle 冻结（AP-01..23 + 超种子项）

- 23 种子路径/性质/SHA256：`docs/plans/reports/av-source-program-catalog-2026-10-20.md` +
  `attachments/av-language-roadmap-20261020/program-inputs.json.txt`（身份回执源）。
  AC 源域=AP-01/02/03/22/23（`experimental/ac-core/fixtures/`，真实 .at 源码，无
  `.expected.*` 旁文件——oracle 由本计划手审冻结）；其余 18 个=旧 VM corpus/示例/脚本糖，
  属未来 G 门分母，本计划只登记 gap+owner。
- 超种子：`scalar-source/valid/` 另有 8 个 .at（p-a-minus-1/p-cmp-matrix/p-count-digits/
  p-crlf/p-for-loops/p-is-digit/p-short-circuit-contexts/p-short-circuit-trace）——入 G0 分母。
- **oracle 冻结（手审）**：
  - AP-01 `p-gcd-euclid.at`：gcd(48,18)=6、(18,48)=6、(7,5)=1、(0,9)=9、(9,0)=9 全对
    →`return 0`。oracle=success exit 0；gcd 各组为独立数学 oracle（非循环互证）。
  - AP-22 `p-binop-args.at`：1-2+3*4=11→add(−1,12)=11 ✓；(10/2,7%4)=(5,3)→8 ✓；
    (3,1+3)=7 ✓；multiline (3,3)=6 ✓；(3)* (3)=9 ✓ →`return 0`。oracle=success exit 0。
  - AP-23 `p-short-circuit-trap.at`：`false && (1/0>0)`——RHS 必不执行（无 trap）；
    `true && (1/0>0)`——RHS 必执行、i32 除零 trap。oracle=error-run +
    message contains `av.trap.div-by-zero`；**任意 Err/进程强杀/超时不充当负例**。
    false-only 衍生正控（fixtures_827 副本改造）=success（第一条路径独立正控）。
    native 通道（exit70）由 native_reference build/run 独立对拍，与 guest trap 分通道。
  - overlay 变体：AP-01 编辑字面量 `return 0`→`return 9`（原文恰一次）→新身份/新
    expected 9；源更新必须使实际 miss（非两个恒 0 文件对拍）。
- corpus 规模分母（2026-10-24 盘点）：`crates/auto-lang/test/vm/` 47 类目/517 case 目录/
  676 `.at`+8 `.as`；发现规则=两级扫描（`crates/auto-lang/src/tests/vm_file_tests.rs:31-117`，
  case 名=目录名首个 `_` 后段，`.at` 优先 `.as` 回退，sidecar
  `.expected.{out,result,error}`）。**在册 caveat**：23_actor/007/008/009 盘上存在但故意
  未注册（vm_file_tests.rs:2124-2130）——G0 枚举**数盘上文件**而非仅注册测试。
- stdlib 分母：`stdlib/assembly-providers.json`（PLAN-738 provider 目录）+ `stdlib/auto/`
  层文件扫描；23 AP 的 stdlib 消费符号表已逐条登记（AP-01/02/03/22/23=零依赖纯标量；
  AP-07=`auto.greet_mod`；AP-09=Json；AP-10=File；AP-15=Net；AP-18/21=use.py 桥等）。

## 4. G0 inventory 规则（可复现枚举，inventory Scope 的实现合同）

| 分母 | 复现规则 | 冻结基线数（r1） |
|---|---|---|
| 能力 C-01..43 | `docs/design/strategy/av-language-capability-matrix.md` §2 表逐行（状态 限/T/—+owner） | 43 项；限=9、T=7、—=27（手审计数，脚本核行数与 ID 连续性） |
| 入口 EP-01..10 | roadmap §5 表 | 10 项 |
| 门 G0..G8 | roadmap §3 | 9 门 |
| 程序 AP-01..23 | catalog 表 + program-inputs.json.txt SHA256 逐文件对账 | 23；AC 源域 5 |
| 超种子 scalar fixtures | `experimental/ac-core/fixtures/scalar-source/*/*.at` 目录扫描 | 11 于 scalar-source/valid（3 种子 AP-01/22/23 + 8 超种子；AP-02/03 分别在 record-source/text-record-source 目录） |
| VM corpus | `crates/auto-lang/test/vm/` 两级扫描（同 vm_file_tests 规则，数盘上目录/文件含未注册） | 47/517/676+8 |
| stdlib | `stdlib/assembly-providers.json` 条目 + `stdlib/auto/*.at` 文件数 | 以脚本输出冻结为 r1 基线 |
| 旧入口 call-site | 冻结锚点清单（13 处：auto main.rs run/eval/old-repl/mcp/rqhost/service/test/ui/stdlib/gen/watch、`auto_lang_core::script_entry`、SDK `run*` 族含 `run_with_capture_and_path_staged`、persistent/daemon/client/repl、test_cases::avm、vm_bridge、vm.rs） | 13（存在性 grep 核验） |

inventory 输出为 JSON 清单（脚本生成、随跑对账）：每项含 ID/路径/SHA256/性质/owner/
状态（supported/gap/unsupported）。扫描失败/无 expected/未测=保留 gap 行，不从分母删除。

## 5. Scope 与反控冻结（脚本 `scripts/verify-av-source-827.ps1`，T-03 建立）

| Scope | 内容 | 主要承载 |
|---|---|---|
| inventory | §4 全分母复现+对账 | 脚本+枚举测试 |
| first-case | AP-01 真实文件冻结 bytes 全路径（ephemeral+store put/hit+新进程 restore）+corpus 首例 | probe/tests `av_source_827_first_case.rs` + corpus case |
| semantics | V-01..03（AP-01/22 手审 oracle、AP-23 精确 trap、false-only 正控、实参变体改值、native exit70 分通道） | corpus cases + probe/tests |
| admission | V-04..06（坏 hash/同名异 bytes/冻结后改盘/非法 UTF-8/越根/缺文件=分态非 trap；AP-05 print/List→前门拒；好 hash 坏 MIR/ABC/伪 kind/错 producer→dispatch0） | 根 workspace 负控测试 + probe/tests |
| resources | V-09（cap 内/超 1、无限循环/cancel、双线内存、原 256KiB 编辑两路径 P95、新小文件冷态另报） | bench 复用（`bench-turnaround`/`frontend-attribution`）+新负载 |
| cache | V-07/08（fresh/hit/新 worker restart、overlay 0→9、改 profile/工具/依赖/grants、跨根、坏内层、位置重绑 span） | probe/tests |
| compatibility | V-11 旧 consumer 面（ac-av corpus 原样全绿、817/818 recipe 回归引用既有测试，不改其期望） | 既有测试重放 |
| autodown | V-10（av-source-827 corpus 经正式 recipe 父侧判定全绿+负 oracle+注错必红+漏子 case/skip 非绿） | 根 workspace 门测试 |
| acceptance | V-01..12 全必需子 case selected 计数核对+V-12 差异状态表 | 脚本汇总 |

0 例/漏例/skip/重复冒计/任意 Err→脚本非零退出，退出码透传。测试前缀冻结：
独立 workspace `av_source_827`（`cargo test --manifest-path experimental/ac-av-probe/probe/Cargo.toml av_source_827`，
selected=0 不得 PASS）；根 workspace 门测试 `av_source_827_*`。

## 6. T-09（V-13）设计冻结

827 的热编辑消费者=独立 workspace 内 `av_source_827_incremental` 测试族，**只读消费**
`session.rs` 公开 API（warmup/prepare_edit/commit + changed_name/unit_mir 观察缝），
镜像 ac-mn `incremental.rs` 先例但执行面=AV（无 native）。四子 case 按 ACE 合同：
(1.1) D828 冻结基线 273575B/115 函数×≥30 修订经本消费者双门（结构+P95<1s）；
(1.2) 本计划入口两函数（AP-01 派生 gcd+main）局部 body 更新——未改单元零重 lower/
装载、恰一安装；(1.3) 签名/接口/授权变化=精确失效/拒绝三路；(1.4) 强制整前端/
换对象不换 hash=结构门必非零。cold/整源编译入口（case-serve）单列不冒领增量。

## 7. 风险与边界（继承计划 §1 非目标）

不改 ac-core 语言能力（AP-01/22/23 在 scalar profile 内原样编译，无需语言变更——
若实施中发现需扩语言即退出范围回 new）；不切产品默认；oldAVM/native 仅独立参考
（native 对拍走 native_reference 通道，不参与新路径执行）；821+ 未来 profile 不因
本目录程序提前支持。

## 8. 修订

- r3（2026-10-25，review-r2 修复轮）：
  - **§5 汇总器规则修正（R2 判定 r2 的全域 skipped>0 红为无效解释）**：判别对象是**必需案例**——
    nextest 的 Starting/Summary 行中 filtered-out 总体 skipped 属过滤排除，不算必需案例跳过；真实跳过信号
    =nextest `SKIP [` 结果行（被选中而跳过）与 libtest `test result:` 行 ignored 计数；每 scope 绑定
    **必需测试 ID 清单**（缺失/改名/红即 whole-scope 红）；selftest 永久重放六控制（过滤正例必须绿/
    真 SKIP 红/缺必需 ID 红/ignored 红/零选择红/真实失败子 scope 同路径红）。
  - **§4 库存合同补全**：stdlib/auto 文件成员为独立行（逐文件 SHA+layer 分类 public/.vm/.rs/.c，
    ≥102 行——37 条 provider 声明只声明所有权，不替代成员分母）；vm-corpus-case 行绑定**源码字节
    SHA**（编辑即换行身份）+oracle sidecar SHA 于 note；入口锚点补 service（cmd_service）与 man
    （auto-man-runtime）共 15；读失败保留 scan-error 行（负控用 Win32 share_mode(0) 排他句柄真实
    触发）；源码变更负控=改 prog.at 后行身份必变。
  - 健康修复：R2 指出的 3 处 unused_mut 清零，本计划测试文件零新警告。
- r2（2026-10-25，review-r1 修复轮，T-01/T-06/T-09 重开后再冻结）：
  - **§4 升级为逐项库存合同**：分母成员必须是持久化 item 行（`{id,kind,path,sha256,status,owner,note}`，
    工件 `docs/reports/plan827/inventory.json`，当前 714 项：23 AP+14 fixtures+47 类目行+517 case 行+
    37 provider 行+43 C+10 EP+10 G（G2=G2a/G2b 拆分）+13 锚点行）；扫描失败保留为 `scan-error` 行、
    缺行补全为 `gap-missing-row` 行且必红、无 oracle 的程序行 owner=future-G 不删分母；负控（缺目录/
    坏 JSON/无 providers 数组/无 oracle case/缺表行）逐类可重放（`av_source_827_inventory_negative_controls`）。
  - **§6 升级 V-13 检测器合同**：未改函数绑定 `(Arc ptr, generation)` 双轴（源=`current()` 快照
    `find_code/generation_of` + 执行器 `last_run_publication` 五元组）；生产/IO 事实=每编辑
    `phase_io`（增量 rescan 上界 len/100、无 store 时 store_write_bytes=0）；**强制退化检测器=身份稳定门**
    （镜像 828 formal judge `edit_identity_gate`——warmup 冷路径零计数/零 IO 已实测，重 warmup 每编辑
    全体换对象→未改指针移动→门红，且退化路径值仍正确=结构门语义）；同 hash 换对象判别力=双 warmup
    同 content 不同 ptr（ptr 轴独立于 hash 轴）；`whole_module_findings` 计数器门保留覆盖会话内
    fallback 形态。全部落在 `av_source_827_incremental`（5 测）。
  - **§5 升级汇总器规则**：acceptance 必须串联**全部**已实现 scope 含 resources；`Test-Selected`
    对 skipped/ignored>0 必红（0 选择亦红）；重复 check id 由 corpus 门去重断言拒绝；`selftest` 档
    永久重放五类负控（valid/skipped/ignored/missing/真实失败子 scope 经同一 `Invoke-Scope` 路径）。
    （r3 修正：全域 skipped>0 红改判为无效——见 r3 节。）
- r1（2026-10-24，T-01/T-02 刷新合一）：按 D798+D828 合入基面冻结。复用裁定、薄适配面、
  oracle、分母、Scope、T-09 设计如上。无公共写面登记于 §1。
