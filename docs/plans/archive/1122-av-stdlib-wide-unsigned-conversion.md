---
plan_id: PLAN-1122
status: archived
completion_kind: delivered
feature_name: "AV 标准库宽无符号数值转换与旧方法兼容"
author: [Codex]
created_at: 2026-11-22
updated_at: 2026-11-23
plan_revision: 1
current_step: 4
total_steps: 5
depends_on: [PLAN-972, PLAN-1079, PLAN-1096]
supersedes_spec_components: [docs/specs/ace-migration/m2-stdlib-core.md, docs/specs/auto-av/language-parity.md]
new_spec_components: []
touched_goals: [GOAL-001, GOAL-002]
affects: [stdlib, auto-av, auto-semantics]
---

# [PLAN-1122] AV 标准库宽无符号数值转换与旧方法兼容

## 0. 变更摘要

2026-11-22 起草。承接 [AV 对 AutoVM 的迁移差额](../../design/strategy/av-autovm-language-parity.md)的本领域条目，使用 [PLAN-1096 已交语言基线](1096-av-parity-full-language-certification.md)。

## 1. 目标

交付 Windows AV 普通源码上有明确失败结果的 `Str → u64` 转换及必要的旧方法迁移，实际值贯穿局部、参数、全局、运算及插值。

## 2. 架构方案

用已有宽整数、受检运算及所有权通道承载结果。优先扩展共享 `stdlib/ace/asm/conv.at` 的核心算法，方法接面复用现有绑定。以可区分失败的公共返回形保留有效零。

## 3. 技术栈

`stdlib/ace/asm/conv.at`、`experimental/ace/crates/core/src/source/resolve.rs`、旧 `crates/auto-lang-core/src/vm/ffi/stdlib/strings_json.rs` 及标量 AutoDown 语料。

拟实施树为 `D:/autostack/.wt/lang-1122/auto-lang`，分支 `plan-1122-dev`。

## 4. 需求分析与背景调查

[核心标准库](../../specs/ace-migration/m2-stdlib-core.md)当前 `to_int(Str) -> ?int`。[旧 shim](../../../crates/auto-lang-core/src/vm/ffi/stdlib/strings_json.rs)的 `Str.to_uint` 签名返回 `i64`，实现为 `trim().parse::<i64>().unwrap_or(0)`。旧 `legacy.25_method_u64.001–011` 还涉及宽结果推导、混合数值、全局和插值，由 [旧用例表](../../../crates/auto-lang/test/cases/language/migration/legacy-map.tsv)逐行实证。

取材基线为主检出 `f0d9dcaf3` 及同一检出的活动计划。用户于 2026-11-23 以 `/goal 完成计划1122` 授权实现、独立复审、约定范围内修复、合并与清理。本轮基线为 `8384ea064a78dd3d0b1de3d9c1c47bafd4d061f0`，专属树 `D:/autostack/.wt/lang-1122/auto-lang`，兄弟 auto-down 只读依赖 `456c86ac886f9781e56f6153ce9aa41cd59e5c8e`。

## 5. 详细设计

普通 AV 源码以 `str/Str.parse_u64() -> ?u64` 显式迁移旧方法，装配入口 `std.conv.parse_u64` 复用同一纯 Auto 解析函数。有效输入是 ASCII 十进制数字，允许单个前置 `+` 和前导零。空白（含 Unicode）、负号、空串、单独正号、其他字符与超范围值返回 `None`，有效的零返回 `Some(0u64)`。调用方可按业务规则先 `.trim()` 或显式指定默认值。装配 `std.conv.from_u64` 与普通 AV 的 `u64.to_str()`、插值均输出精确十进制。

旧 VM `to_uint()` 实测调用 `trim().parse::<i64>().unwrap_or(0)`，接受正负号，越 i64 上界或坏输入返回 0。旧拼写留在原语料并给出迁移诊断。混合算术保留既有 AV 同型要求，旧浮点案例显式转 `f64` 后执行，宽整数主路径保持 u64。旧 `to_int` i32 面保留。普通 AV 的局部与项目全局通过受检 Option 取值，再走现有标量绑定、module init/get/set 和 U64 显示。

### 规范增量

| delta_id | 方式 | 目标 | 规则与理由 | 验收 |
| --- | --- | --- | --- | --- |
| SD-01 | modify | [`docs/specs/ace-migration/m2-stdlib-core.md`](../../specs/ace-migration/m2-stdlib-core.md) | 增加受检宽无符号解析/格式化及公开错误合同 | AC-01/02 |
| SD-02 | modify | [`docs/specs/auto-av/language-parity.md`](../../specs/auto-av/language-parity.md) | 记录旧方法拼写、返回值和混用的迁移落点 | AC-01/02 |

## 6. 测试设计

下列验收尚未执行。用例由对应 work 任务落到真实公开入口，沿 [共享 AutoDown 语料合同](../../design/testing/ace-feature-organized-autodown-corpus.md)保留唯一作者。旧结果作迁移参照，新增断言使用独立预期。

| 用例 | 验收与责任任务 | 输入与路径 | 独立预期 |
| --- | --- | --- | --- |
| V-01 | AC-01 · T-02 | 受检解析 0、5000000000、9007199254740993、18446744073709551615 | 逐值精确。最大值经格式化往返，超过最大值 1、负号、垃圾和空串显式失败。 |
| V-02 | AC-01/02 · T-03 | 旧 25.001–011 的局部、参数、全局读写、比较、插值与混合算术 | 有效 64 位结果及可比输出逐行保持。给旧失败/不兼容形保留原源和精确诊断。 |
| V-03 | AC-02 · T-03 | 旧 `to_int` i32 正负边界和新增 u64 再进入窄类型 | 旧解析结果保持；窄化溢出受检，禁止以截断或浮点近似取得相同显示。 |

冻结验证命令为 `cargo t ace-stdlib ace-source ace-semantics ace-lowering ace-av case-av-language-parity`。按实际改面核对固定组和下游，分列实跑、原红、排除与未跑。Windows 在 Windows 环境验收，Linux 已有标准库义务进入 [1063](../1063-ace-stdlib-linux-deferred-validation.md) 的独立执行台账。

## 7. 验收标准

- [x] AC-01 · owner_stage=work。u64 边界、失败和往返经普通 AV 实跑，满足 V-01。
- [x] AC-02 · owner_stage=work。旧方法族及既有 i32 兼容逐行归档，V-02/03 有独立 oracle。
- [x] AC-03 · owner_stage=review。独立复核所需用例、从源码到执行的真实接线、原 oracle 和规范增量，重放关键正反例。
- [ ] AC-04 · owner_stage=merge。冻结受影响固定组证据，按 review 认定的规范增量及交接记录归档。

## 8. 执行步骤

实施前于可用的现有任务跟踪入口登记任务 ID。T-01 可在已归档依赖上先核实基线和公开合同。

- [x] T-01 冻结旧解析语义与公开迁移形（§5、§9）
  - owner_stage=work。运行旧关键输入和当前 AV 近邻，确认方法返回、词法和异常值差异。先固定迁移表，再定公共签名。
- [x] T-02 实现受检宽解析与输出
  - owner_stage=work。依赖 T-01。沿已有宽数字能力实现精确解析/格式化和必要方法绑定，运行 V-01。
- [x] T-03 承接旧方法族与窄化反例
  - owner_stage=work。依赖 T-02。接入共享标量语料和原映射，运行 V-02/03。
- [x] T-04 独立复审
  - owner_stage=review。依赖 T-03。核 AC-03、SD 增量、实测结果和旧用例映射。
- [ ] T-05 冻结验证与归档
  - owner_stage=merge。依赖复审。执行 §6 固定组并集，核规范与主线结果，折叠证据摘要并按生命周期规则收尾。

## 9. 复审记录

2026-11-22 · stage: new · PLAN-1122 r1 · outcome: pass（草案结构与交接）。next: 计划确认后进入 work/T-01。

2026-11-23 · work/T-01 · `auto-case run --adapter avm` 执行旧 001–011 和词法探针（0.438 s）。旧文档 VM result 映射到 result 通道产生包装 exit 1，实际 stdout 工件逐行比较。9 条字节匹配，005/011 仅布尔显示为 true，文档旧快照为 1。探针依次实际输出 0/42/-1/9223372036854775807/0/0/0/0/2147483647/-2147483648。对账 SHA256 `970e4059f235a7be3f5f283ce46cbdd2cdfb9b9624b7f787e84ce463724ec8d2`。普通 AV 用基线专属 tree debug worker（SHA256 `d954c7ea2d1d111d6ebf039de4d879f817be02ade3ce109ece9d229587de0ce4`）重放 15 项，1.760 s。主观察：parse_u64/旧 to_uint 缺失，隐式 u64/f64 绑定拒绝，现有 AV i32 边界通过；原源诊断表随 T-03 归档。原始 AV 报告 SHA256 `112bdea59e35b2a1a08268f4185d9de1e38e071a0f94cacb8dc05df6d76c6d71`。任务跟踪采用本计划复选框与会话 T-ID。

### 工作交接（AC-01/02，T-01–T-03）

实现冻结为 `69949c573043777f61a92b8649f6a7be58a1d24b`，主线基面 `9c0f5e0fc2d62276cce580721a67a1b2098afff4`。
从 `8384ea064` 搭建，`b50211c28 → 69949c573` 的 `git range-diff` 等价，期间主线只动了 1121 归档与债务文档。依赖 auto-down `456c86ac886f9781e56f6153ce9aa41cd59e5c8e`。无实现待办，AC-03/T-04 进入复审，AC-04/T-05 等待冻结固定组。

复用装配模块的纯 Auto 解析器，普通 AV graft 把公开的拥有型 `Str` 入口改为借用 `str`，用户接收者处理走标准参数适配。项目编译使用同一 helper，并把源和坐标纳入绑定身份。运行路径为受检类型 `?u64` → 现有 variant 解包 → u64 参数、局部、module init/get/set → checked MIR 与文本显示。格式化的最大值在普通 `TextDisplay` 与装配 `from_u64` 两条路径各自用独立十进制 oracle 观察。

| 旧行 | AutoVM 实际 stdout | 未改的原源在当前 AV | AV 迁移检查 |
| --- | --- | --- | --- |
| 001 | 42/50/40 | bind/`source.legacy-to-uint` | legacy-local 1–3 |
| 002 | 11.5/20 | 同上 | legacy-local 4–5，显式 to(f64) |
| 003 | 100/123 | 同上 | legacy-local 6–7 |
| 004 | 7/14 | 同上 | legacy-local 8–9 |
| 005 | 145/55/true | 同上 | legacy-local 10–12，旧文档布尔快照写作 1 |
| 006 | year: 2024 / next: 2025 | 同上 | legacy-local 13–14 |
| 007 | 5/3/15 | unit 成功，stdout 字节相同 | legacy-neighbors 1–3 |
| 008 | 1/5/1 | bind/`source.console-argument`，直接 print Option | legacy-neighbors 4–6，解包下标 |
| 009 | 5000000000/10000000000/10000000000/5000000000/5000000001 | text/`source.literal-range`，裸宽字面量越 i32 | legacy-exact 1–5，给宽字面量加 u64 后缀 |
| 010 | 5000000000/10000000000/5000000000 | bind/`source.legacy-to-uint` | legacy-global 1–3，另断言 MAX 参数写入、读回与显示 |
| 011 | 281474976710656/281474976710656/true/true | 同上 | legacy-exact 6–9，旧文档布尔快照写作 1 |

| 入口 | 结果 | 准备 | 发现 | 执行／清理 | 墙钟 |
| --- | --- | --- | --- | --- | --- |
| `auto-case run --suite <work.suite.ad> --target av`，共享语料字节副本 | 10/10，退出 0 | debug worker，SHA256 `a962fc2b4fb17c85d0359b789bf92924bff9d7554fe8389617150a2831feaa6c` | 未单列 | 个案见下，清理未单列 | 1.680 s |
| `auto-case run --suite <legacy-av-current.suite.ad> --target av` | 首次 10 通过及 007 包装缺 unit，退出 2。补 unit 后单例 1 通过、10 NOT_RUN，退出 0 | 复用同 worker | 未单列 | 007 本体 0.445 s，原源亦由 worker 直接确认 0.122 s | 1.256 s + 0.507 s |
| `cargo nextest run --manifest-path experimental/ace/Cargo.toml -p ace-av --test conformance -E 'test(=stdlib_asm_972::stdlib_asm_972_conv_parse_av)'` | 1/1，1477 未选 | 测试编译 14.09 s | 未单列 | 0.554 s，清理未单列 | 整命令未单列 |

保留中间失败：auto 回路拼写用条件 `for` 后关闭解析红。定向 nextest 首次在 Windows 与同一 av-worker 文件并发使用遇锁，顺序执行修复，随后过滤器按发现名称修正。装配首次借用形参数被 `source.text-entry-param` 拒，恢复装配 owned 参数并沿现有方法 graft 适配借用后通过。正式语料报告 SHA256 `d4ee22b3e53b3801841494cc56b1a4b6afaf8b4174a72f435be33ada2bc6e83f`。初次原源诊断报告 SHA256 `eac58ef2dd54f62167255a409df71c2d463e6314c5c1461d749693fab2ab62c3`。

| 新增义务 | 断言 | Debug/test 耗时 s |
| --- | --- | ---: |
| av.parity.1122.checked-u64 | 零、5e9、2^53+1、MAX，格式化与往返，词法与两类溢出 | 0.632 |
| av.parity.1122.legacy-local | 001–006 数值与 stdout | 0.572 |
| av.parity.1122.legacy-neighbors | 007–008 长度、原整数运算和查找 | 0.754 |
| av.parity.1122.legacy-exact | 009/011 宽局部、参数、算术与比较 | 0.501 |
| av.parity.1122.legacy-global | 010 初始化与读取、参数写入 MAX、格式化 | 1.074 |
| av.parity.1122.signed-i32-and-narrow-control | i32 正负边界、u64 收窄的零和最大合法值 | 0.467 |
| av.parity.1122.narrow-overflow | 2147483648 收窄触发 int-overflow，同时覆盖成员空白 | 0.619 |
| av.parity.1122.legacy-spelling-rejected | 旧拼写具名迁移诊断 | 0.593 |
| av.parity.1122.implicit-float-rejected | 无隐式 u64/f64 提升 | 0.620 |
| av.parity.1122.option-needs-handling | Option 直写 u64 必须拒绝 | 0.572 |
| stdlib_asm_972::stdlib_asm_972_conv_parse_av（新增断言） | 装配公开模块的 MAX 格式、往返、溢出和零，保留签名与既有 i32 判据 | 0.551 |

`stage: work | plan_id: PLAN-1122 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1122/auto-lang | branch: plan-1122-dev | base_commit: 9c0f5e0fc2d62276cce580721a67a1b2098afff4 | code_commit: 69949c573043777f61a92b8649f6a7be58a1d24b | dependency_revisions: auto-down@456c86a | task_ids: T-01–T-03 | evidence: 上表 | blockers: 无 | next: review`

`stage: goal | plan_id: PLAN-1122 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1122/auto-lang | commit: 69949c573043777f61a92b8649f6a7be58a1d24b | next: review`

### 最终复审（AC-03/T-04）

复审范围 `final`，在同一会话基于已提交工件重新核验，采用另外生成的输入和独立十进制 oracle。提交 `69949c573043777f61a92b8649f6a7be58a1d24b`，diff 基面 `9c0f5e0fc2d62276cce580721a67a1b2098afff4`，auto-down `456c86ac886f9781e56f6153ce9aa41cd59e5c8e`。专属树手工写入已停止，进入和离开时工作区干净。使用 `auto-plan-review` SHA256 `a39b42ab756177299aa6619c648373ffe2bc01f33547a7e363dc78553070802b`。

- AC-01/T-02 通过。纯 Auto 上界判断 `acc > MAX/10` 或 `acc == MAX/10 && digit > MAX%10` 先于受检乘加。读了两个真实入口之间的 graft/调用路径、Option ABI、现有 U64 显示和装配源码。复跑最大值正负与 55 项另选输入（25 精确值和 30 拒绝，任意精度整数独立预期），5/5 组。
- AC-02/T-03 通过。核 001–011 的原文件 blob、旧 oracle 字段与源身份完全不变，仅 11 条映射改变。核解包、显式浮点混算、全局初始化、经参数写入、再读、旧签名诊断、窄化失败和 i32 正例。无遗漏或延期。新代码及本次新增测试行编译无新警告，保留仓库已有警告。
- AC-03/T-04 通过。规范 SD-01 blob `268781634fc7babb101dca3af78a2fa167957162`，SD-02 blob `28c7edcccfcae21d0ded9fa558c3f91a4a208f88`。两份均核当前行为、出处及公开语料路径。69 个模块导出独立计数通过，保留既有 V08 判据并更新公共面清单。
- AC-04/T-05 待合并阶段。六组选择通过，配置 SHA256 `2d3cd7f908e494a618b6d97c2550f65dffa9326c6843f50b3b5b0af99be5167e`。标准库、源码、语义、降载、AV 和公开入口分别覆盖实际受影响路径。Windows 排除 Linux/WSL，正式 ACE PERF 依裁定排除。

| 复审实际范围 | 结果及计时 |
| --- | --- |
| 正式语言套件八次 `--case av.parity.1122.<id> --target av` | 8/8，准备复用 debug worker，个案和整命令 s 分别为 checked-u64 0.482/3.38、legacy-local 0.477/2.74、legacy-global 0.576/2.95、signed-i32 0.467/2.69、narrow-overflow 0.461/2.62、旧拼写拒绝 0.441/2.54、隐式浮点拒绝 0.446/2.56、Option 直写拒绝 0.448/2.57。未选其他项目，发现/清理未单列。报告汇总 SHA256 `e50c33bbfd43c03d066b607015a95b8c261823c315fd735c25c3d091fce6d800` |
| 独立材料目录 holdout | 25/30 输入分为五组，5/5，墙钟 0.888 s，准备复用。初版同一主体放 25 个 Option 触发 `av.resource.verify-memory`，分组初版误消费同一 Option 两次，MIR 正常拒绝。按相同输入分组并只消费一次后通过。报告 SHA256 `969bf5e0aaa1727235f7955ac03c4dd3440cbe345817a5cd832c7c0110f1bc8f` |
| Git 结构/文本镜像审计 | 11 原文件 blob、10 唯一语料 ID、69 导出和 2 规范链接检查，0.697 s，退出 0 |
| 已测装配、另外两个语言用例 | 复用同一代码与输入的工作证据。全部 11 个新增义务均低于 10 s，本轮无新增永久用例 |

`stage: review | review_scope: final | reviewer_identity: 同会话独立重建证据 | plan_id: PLAN-1122 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1122/auto-lang | branch: plan-1122-dev | reviewed_commit: 69949c573043777f61a92b8649f6a7be58a1d24b | base_commit: 9c0f5e0fc2d62276cce580721a67a1b2098afff4 | dependency_revisions: auto-down@456c86a | acceptance_results: AC-01/02/03 pass | pending_closeout_gates: AC-04/T-05 六固定组及归档清理 | findings: 无开放项 | next: merge`

`stage: goal | plan_id: PLAN-1122 | plan_revision: r1 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1122/auto-lang | commit: 69949c573043777f61a92b8649f6a7be58a1d24b | next: merge`

### 合入门回退 F1122-01

冻结 `69949c573043777f61a92b8649f6a7be58a1d24b` 的五 Rust 组 debug 工作包准备 279.410 s，准备清单 SHA256 `cfd257eda62eb5657b98376285c9eb9b1e032cdb9048e260b66217317aacd597`，构建后等待 UI-117/096 等关联作业结束。运行 79.385 s，退出 1。stdlib 2/2、semantics 295/295、lowering 9/9。source 985/990，5 条 `text_record_source` 同 1120 已登记基线。AV 1612/1627，其中两个新红。

- `v10_layers_972::v10_core_alloc_layers_mn` 对未引用宽转换的 core-only 旧程序失败。AC/native 链接器见到了宽整数操作，拒 `PLAN-972 narrow typed arithmetic awaits the numeric backend unit`。
- `bstr_1064::bstr_1064_v01_large_64k_a2r` 对未引用宽转换的字串旧程序报告 `conversion U64 -> I32 is outside the frozen matrix`。
- 新增装配 AV 用例通过 2.600 s。根因归于全量装配把未使用的宽函数传给旧后端。修复应在实际装配调用和导入上决定是否发射宽扩展，两条旧判据及 AV 的实际引用都保持。旧 AV 红项及新增域在修复后重新核对。公开语料固定组尚未运行。

`stage: merge | plan_id: PLAN-1122 | plan_revision: r1 | reviewed_commit: 69949c573043777f61a92b8649f6a7be58a1d24b | outcome: needs_fix | finding: F1122-01 | next: work`

`stage: goal | plan_id: PLAN-1122 | plan_revision: r1 | round: 3 | from: merge | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1122/auto-lang | commit: 69949c573043777f61a92b8649f6a7be58a1d24b | next: work`

### F1122-01 修复交接

提交 `fc95c64f07cff24046ac2eb4c4a82a3c2bd5b082`。在拥有已绑定导入、引用和模块来源的装配 aggregates 接缝，只有显式导入宽接口或已绑定引用时才联入这三个函数。成员包含公开的两个 API 和格式化私有函数，出口同时核 program 与 origin/reference/read/walk 元数据。std 的源码认证、旧导出/诊断和普通 AV 源码绑定维持原合同。AV 正例改为两个命名 alias 导入配置。

| 定向范围 | 准备 | 执行证据 |
| --- | --- | --- |
| `ace-av --test conformance --features ace-av/test-case-batch` 三个命名原子 | `cargo nextest archive` test profile 10.013 s，`runtime-warm.exe` 明示 prepare-debug 2.426 s | `cargo-nextest nextest run --archive-file …` 9.186 s，退出 0，3/3，1476 未选，提取 0.420 s，清理未单列 |
| `stdlib_asm_972_conv_parse_av`（新增断言） | 同上 | 0.657 s，AV 使用显式导入 MAX 解析、格式、往返、溢出和零 |
| `bstr_1064_v01_large_64k_a2r` | 同上 | 2.224 s，通过旧 64 KiB 字串独立回归 |
| `v10_core_alloc_layers_mn` | 同上 | 8.225 s，通过旧 core/alloc/故障注入/预算，样本保持 |

首次定向出现模块别名语法探针与 runtime stamp 环境不一致，修正为既有命名导入 alias，且采用同一执行环境先明确准备 debug 运行时再消费 archive。准备工件 SHA256 `d131def7693916e36c44de516a4cb8fd7179d44588ddbA5239c320fe4419ceae`，结果日志 SHA256 `cfa355dd38fa60c832f3f8d1afa01a951700119c1105af1b32eed1bf7dd875ba`。复用 1122 原有十个不变的普通 AV 用例证据，合入仍须跑冻结六组。

`stage: work | plan_id: PLAN-1122 | plan_revision: r1 | outcome: pass | code_commit: fc95c64f07cff24046ac2eb4c4a82a3c2bd5b082 | finding: F1122-01 closed | affected: T-02/AC-01 | dependency_revisions: auto-down@456c86a | next: review`

`stage: goal | plan_id: PLAN-1122 | plan_revision: r1 | round: 4 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1122/auto-lang | commit: fc95c64f07cff24046ac2eb4c4a82a3c2bd5b082 | next: review`

### 最终复审补记（F1122-01）

基于同一专属树的不变提交 `fc95c64f07cff24046ac2eb4c4a82a3c2bd5b082` 与 auto-down `456c86a` 重建交接。核装配绑定保留完整源身份、公函导入判据、按实际已绑定引用保留接口和私有格式化依赖，适用关闭状态只删除新宽转换。普通 AV 方法接线、conv.at、旧映射与共享语料对原复审提交逐路径 diff 为空，未变证据复用。编码无新增告警。无开放缺项。

复跑命名 nextest（使用 SHA256 `d131def7693916e36c44de516a4cb8fd7179d44588ddba5239c320fe4419ceae` 的 debug/test archive）3/3，1476 未选，准备复用，提取 0.28 s，运行 5.195 s，墙钟 6.356 s，清理未单列。AV 装配主体 0.520 s、A2R 64 KiB 1.714 s、AC/native core/alloc 5.194 s。正式 AV 项目全局用例复跑 1/1，墙钟 2.936 s，未选 736、原 SKIP 161；使用修复树 debug worker SHA256 `4f94f24d7cac1c475712addf7b9c91f9c54cf454a40601b6d6cd4e12d30f3b60`。本轮无新增用例。复审 nextest 日志 SHA256 `10983ebeb34c167a9bb45af5587fb097971124e5c0c0903af69b759464c1f251`。SD-01 更新为 Git blob `fba6d3ef01ded9b1020f1d8cb09657f2eccf6232`，SD-02 沿用 `28c7edcccfcae21d0ded9fa558c3f91a4a208f88`。六固定组仍待合入执行。

`stage: review | review_scope: final | reviewer_identity: 同会话独立重建证据 | plan_id: PLAN-1122 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1122/auto-lang | branch: plan-1122-dev | base_commit: 9c0f5e0fc2d62276cce580721a67a1b2098afff4 | reviewed_commit: fc95c64f07cff24046ac2eb4c4a82a3c2bd5b082 | dependency_revisions: auto-down@456c86a | findings: F1122-01 closed | acceptance_results: AC-01/02/03 pass | next: merge`

`stage: goal | plan_id: PLAN-1122 | plan_revision: r1 | round: 5 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1122/auto-lang | commit: fc95c64f07cff24046ac2eb4c4a82a3c2bd5b082 | next: merge`

### 合入门与规范核验（AC-04/T-05）

交付和复审提交均为 `fc95c64f07cff24046ac2eb4c4a82a3c2bd5b082`。主线 `9c0f5e0fc2d62276cce580721a67a1b2098afff4` 快进至交付提交。共同冻结 source-state `02a11c90c21c37a52de36d7a8d33be7f3c38c7d452a01dc5b34446997a9402d7`，固定配置 SHA256 `2d3cd7f908e494a618b6d97c2550f65dffa9326c6843f50b3b5b0af99be5167e`。auto-down `456c86ac886f9781e56f6153ce9aa41cd59e5c8e`。

复审后在冻结树执行 `cargo t ace-stdlib ace-source ace-semantics ace-lowering ace-av` 与 `cargo t case-av-language-parity`。前者 `--debug-tools --prepare-only`，后者采用 Case 既有 debug 准备。两者使用 `--run-prepared`，校验实际原子终态。共 3467 项执行，3447 通过，20 个基线失败。Linux/WSL 按 Windows 裁定排除，ACE PERF 按固定组合同排除。

| 固定组 | 原子终态 | 发现 s | 子运行 s |
| --- | --- | ---: | ---: |
| ace-stdlib | 2/2 通过 | 0.827 | 3.079 |
| ace-source | 985 通过，5 原红，1 跳过 | 0.485 | 14.769 |
| ace-semantics | 295/295 通过 | 3.068 | 5.951 |
| ace-lowering | 9/9 通过 | 0.320 | 0.442 |
| ace-av | 1614 通过，13 原红，23 跳过 | 2.090 | 48.234 |
| case-av-language-parity | 542 通过，2 原红，193 NOT_RUN，161 SKIP | 未单列 | 49.336 |

Rust 与 Case 准备墙钟分别为 119.863 s、12.023 s，执行总墙钟分别为 80.984 s、54.158 s，清理未单列。各阶段启动前查原生测试、编译器和 worker 进程，机器检查及等待共 147.082 s。准备清单 SHA256（Rust / Case）为 `b449926090a56be552088d68dc51f66119b94733ac139a1f83e81f17c5b3aebc` / `773a078ff4e63c58214c2834be4e14d5f27eca65387d81d37cd768d3d24f4110`，报告 SHA256 为 `be4610934970d46ad62ba788391946b624567d89d9a1719aff246ee864c1d188` / `915763e4f5854f8773839ac0c15217b744443aa96e0b0edd3a327974b72e84a4`。两次父命令退出 1 对应上述基线原红。

18 条 Rust 原红与 [PLAN-1121](1121-av-stdlib-map-set.md) 的合入门一致。source 为 `text_record_source` 五条（flagship、generated_atom、move_arguments、named_ctor、positional_ctor）。AV 为 av_method_modes、av_method_owned 四条、av_owned_first_case、av_owned_recipe、av_test_batch 的 c_data_profile、component_event_sessions_1106、layout 和 resource_915 三条。AV 语料的两条 `foreign.prepared` 为 `av.parity.1093.explicit-c-library`、`av.parity.1093.c-header-spelling-rejected`，与 [PLAN-1110](1110-ace-a2py-python-exception-mapping.md) 和 [PLAN-1126](1126-ace-python-numpy-json-parity.md) 现有证据一致，保留 PLAN-1125 归属。F1122-01 两条旧测试在冻结门恢复通过（A2R 6.495 s、AC/native 8.234 s）。

| 新增义务 | 冻结实跑 s（debug/test，均通过） |
| --- | ---: |
| checked-u64 · 宽值、词法、上界和往返 | 0.683 |
| legacy-local · 001–006 兼容输出 | 0.713 |
| legacy-neighbors · 007–008 i32 与查找 | 0.853 |
| legacy-exact · 009/011 精确算术与比较 | 0.795 |
| legacy-global · 010 和 MAX 全局写读 | 0.863 |
| signed-i32-and-narrow-control · 旧 i32 和安全窄化 | 0.654 |
| narrow-overflow · 窄化溢出受检 | 0.818 |
| legacy-spelling-rejected · 旧名称诊断 | 0.751 |
| implicit-float-rejected · 同型限制 | 0.736 |
| option-needs-handling · Option 类型边界 | 0.715 |
| stdlib_asm_972_conv_parse_av · 受检装配接口新增断言 | 2.380 |

快进前两树 guard clean。落地后用冻结 debug worker（SHA256 `e48709b5f55c2f7d96ea9e640dca98ccbb6ee720b601eac5a4d7e6099af85621`）对主线两例冒烟，`checked-u64` 2.677 s、`legacy-global` 2.809 s，均退出 0。主检出的 debug/release worker 文件时间分别为 2026-11-23 04:05:06 和 2026-11-22 20:52:01，属于交付前产物。主检出生产、测试和规范路径干净。其它计划与设计文档已有编辑保留。批次收据覆盖至 1126，1122 无新周期门。

`prepared → landed → canonical_verified`：确认交付提交中 SD-01 `docs/specs/ace-migration/m2-stdlib-core.md` blob `fba6d3ef01ded9b1020f1d8cb09657f2eccf6232`，SD-02 `docs/specs/auto-av/language-parity.md` blob `28c7edcccfcae21d0ded9fa558c3f91a4a208f88`。应用派生索引使用上述源文件。成功状态移至本归档文件，路径核验和清理结果分两次提交留痕。当前 T-05 标记等待实际清理。

## 10. 待澄清事项

T-01 已确定显式迁移，现有宽度规则和旧零哨兵差异见 §5。
