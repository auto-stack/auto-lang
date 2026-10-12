# PLAN-767 r2：Codex 独立最终复审

- 日期：2026-10-12；review_scope：final；outcome：**needs_fix**。
- 复审者：Codex，本对话；本会话此前参与规划，未参与 767 实施。本次从提交、合同、代码和真实执行重构结论，不以实施会话的 final pass 为证据。
- 受审路径：`D:/autostack/.wt/lang-767/auto-lang`；分支：`plan-767-dev`。
- reviewed_commit：`033642a27d7404dee5797b195a9b7c680fe6ece5`。
- base_commit：`a9225e8de6e13cc0bd0c519e7c4b380c7276ebfa`。
- 合同：主检出 PLAN-767 revision 2，冻结副本 [review-codex-contract-r2.md](review-codex-contract-r2.md)，SHA-256 `f9ba251f8a6d7f19e30bc9d26bae453fe11ea5ddd90a29fdd0257236cbbc9204`。
- 冻结规范增量：[review-codex-proposed-spec-delta.md](review-codex-proposed-spec-delta.md)。
- 指令、Specs、依赖、受审实现、实际二进制/rlib/支持库指纹：[review-codex-inputs.tsv](review-codex-inputs.tsv)。
- 入口/结束复核：HEAD 相同，受审树无未提交改动；未修改实现或 canonical Specs/ledger，未合入、归档或清理树。

## 1. 实际重跑的门禁

全部工作树命令的 cwd 为受审树：

| 命令 | 结果与边界 |
|---|---|
| `cargo test --manifest-path experimental/ac-core/Cargo.toml --locked` | 143 pass / 0 fail；另有 1 个既有显式 ignored 的 60s 执行器专项，不把它计为通过 |
| `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-scalar-767.ps1` | 14/14 步、45 例 pass；含 locked all-targets 零 warning、fmt、真实 PE、阶段/码与收据 |
| `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-741.ps1 -SkipMainGates` | 13/13 步 pass，原型用例保留；只跳根档 |
| `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-source-745.ps1` | 13/13 步、74 例 pass；含旧 profile、事务与执行器 failure controls |
| `python scripts/acc_inventory.py --root . --output docs/reports/743-acc-hir-contract --check --require-decisions` | exit 0 / CHECK-OK，47 条人工结论严格绑定新鲜 |

原型变更未触及 `crates/`、root Cargo/config、旧 auto-lib；按合同不跑根 t/tv/docs_gen。`git diff --check base HEAD` 只报告专门 CRLF fixture 的 CR 字符；这是该用例的输入证据，不能为消除提示改成 LF。Rust fmt 门禁实际通过。

### 独立原生观测

- [review-codex-arithmetic.at](review-codex-arithmetic.at)：32 组有界参数组合，138 项独立数学断言，真实 PE exit 0、空 stdout/stderr。[逐项 oracle](review-codex-arithmetic-oracles.tsv) 使用精确整数，商通过绝对值整数除法与符号计算、余数用 `a-q*b`，不借旧 VM 作裁判。
- [review-codex-shortcircuit.at](review-codex-shortcircuit.at)：交错 `&&/||`、命名实参先源码序求值、3 轮条件 for、continue 后重判、false 条件零次执行；真实 PE exit 0，stderr **`abaabaabababaabaaba`**，stdout 空。
- 执行入口：[review-codex-native.ps1](review-codex-native.ps1)。复用受审脚本中的 contained executor，build/run 各 60s 截止，不用无期限 `Command.output`。
- 可持久保存的运行结果与源/产品/工具 hash：[review-codex-native-results.tsv](review-codex-native-results.tsv)。本轮 CLI binary SHA-256：`2d81403a9cda6980ac9fb55e376960d76be54987177a9862d9b4fb9ddb1a1c28`；支持库：`70aeb8c8737956f2d23a3e094e5a0a8e7246cba8a9cc7ec7beb19a4998a4e124`。不沿用上轮不同二进制 hash。

## 2. 阻塞发现

### P767-R1 / P1：未知三重身份丢失，canonical writer 可以换壳

- 原要求：§5.1、AC-01/05、T-02——权威 schema/revision/profile 必须随行；caller 传另一 descriptor 时 writer 显式拒绝身份冲突，不仅比较一个有损分类。
- 代码：`experimental/ac-core/src/descriptor.rs:245`，`Descriptor::domain()` 仅识别 scalar 精确三元组，其余全部返回 Core；`:2111` 的 writer 用该分类判相等。
- 最小反例：bind 正常 core 文档取得 Bundle；clone core descriptor，只改 schema 为 `auto.hir.unregistered`、revision 为 99、或 profile 为 `unregistered-profile`；`write_canonical(core_bundle, altered_descriptor)` 三种都返回 Ok，并输出不同身份头。
- 进一步实际观测：用相同 altered descriptor bind 对应改头文档，再 `verify::verify`，三种都获得 Core Checked。公共 registry `bind_document` 正确拒绝未知头，故问题在程序化 binder/writer 路径，CLI 拒绝不能覆盖这些 API。
- 可重放证据：[review-codex-probes.rs](review-codex-probes.rs)，`WRITER-schema/revision/profile` 与 `UNKNOWN-schema/revision/profile`，共 6 个未满足要求的观测；有效 core/scalar、错误 verify 入口、已知 core/scalar writer 冲突及 registry 拒绝控制均成功。
- 修正方向：对每个已注册三元组做精确、可失败的映射；未知身份不能归到 Core。binder、Bundle/Checked 上下文与 writer 均保留/核查权威身份，任一维变化都受控拒绝。保持旧 core 与新 scalar 正控及 CLI 0/1/2。
- 性质：当前合同内可信身份缺口，不是提出第三种 profile 的新需求；必须修复后复审。

### P767-R2 / P1：函数签名 TypeId 越界通过 verify，native panic

- 原要求：§5.3、AC-04/05/06、T-02——程序化 Bundle 仍受真实类型/图引用可信门管控，非法输入不能得到 Checked、不能进入后端导致 panic。
- 代码：`experimental/ac-core/src/verify.rs:226`，签名 TypeUse 查表失败默认 I32；`:255` 在签名建表时直接使用该结果。新增 `check_reference_bounds` 只检查 body 的 locals/places/exprs/blocks 引用，未覆盖 module function signature 的 params/result。
- 最小反例：bind `scalar-hir/valid/s02-two-bodies.atom`（正常正控通过），将 `module.types[1]` 的 Function 签名 `params[0]` 或 `result` 分别改为 `TypeUse::Ref(TypeId(9999))`；其余输入完全不变。
- 实际结果：两种 `verify_scalar` 都返回 Ok(CheckedModule)。`find_entry(checked, "d_main")` 后 `lower_object` 均在 `src/native.rs:91:49` panic：`index out of bounds: the len is 3 but the index is 9999`。探针 catch_unwind 只为收集后续观测，没有修正或掩盖产品行为。
- 可重放证据：同一 [review-codex-probes.rs](review-codex-probes.rs)，`SIGNATURE-parameter/result`；另有无效 ExprId 被当前 bounds gate 正确拒绝的控制，证明调用的是真实 verifier。
- 修正方向：在签名类型求值前验证所有 TypeUse 的归属/索引/允许类型，不能 fallback 成合法 I32；覆盖 params/result、内建/引用正控、错误类型与后端 admission。相同风险的其他未知引用路径一并作作用域审计，不以只修 9999 两个样例结束。
- 性质：底层默认逻辑部分继承旧实现，但 767 明确认领程序化 Bundle 可信门；不能以预存或 CLI 文本 binder 已检查为由豁免本合同缺口。

### P767-R3 / P2：设计收据的减法护栏公式与正确实现冲突

- `docs/reports/767-scalar-i32/design.md:112` 写 `(b>=0 && a<min-b)||(b<0 && a>max-b)`；该公式错误。
- `native.rs::checked_sub` 实际为 `(b>0 && a<MIN+b)||(b<=0 && a>MAX+b)`，与精确减法规则一致，算术正控与 trap 矩阵通过。
- 修正方向：只修设计收据使其与已通过验证的实现/数学规则相符，重新冻结受影响证据；不要照错误收据修改正确后端。归 T-01 的文档收口缺口。

## 3. 探针重放

先在受审树完成本报告列出的 Cargo build/test，随后 PowerShell 中：

```powershell
$reviewTree = 'D:/autostack/.wt/lang-767/auto-lang'
$reviewSource = 'D:/autostack/auto-lang/docs/plans/attachments/767/review-codex-probes.rs'
$reviewOut = Join-Path $reviewTree 'experimental/ac-core/target/review-codex-767'
New-Item -ItemType Directory -Force -Path $reviewOut | Out-Null
$reviewRlib = Get-ChildItem "$reviewTree/experimental/ac-core/target/debug/deps/libauto_ac_prototype*.rlib" | Sort-Object LastWriteTime -Descending | Select-Object -First 1
$reviewExtern = 'auto_ac_prototype=' + $reviewRlib.FullName
rustc --edition=2021 --crate-name review767 $reviewSource --extern $reviewExtern -L "dependency=$reviewTree/experimental/ac-core/target/debug/deps" -L native=C:/Users/zhaop/.cargo/registry/src/rsproxy.cn-e3de039b2554c837/windows_x86_64_msvc-0.52.6/lib --out-dir $reviewOut
& "$reviewOut/review767.exe" $reviewTree
```

native 搜索路径来自受审树 `target/debug/build/windows_x86_64_msvc-*/output` 的真实 `cargo:rustc-link-search`，其他环境应读本机的相同输出，不猜 registry hash。初次手动链接未补该 native 搜索路径时报 LNK1181，这是探针链接环境问题；补实际路径后才形成上述产品反例证据。

受审 baseline 上：8 个正/负控制 PASS，8 个合同失败观测（R1 六个、R2 两个），进程 exit 1。[原始观测输出](review-codex-probe-output.txt) 保留了两次 native panic 与各项结果。修复版应当所有项 PASS、exit 0；不可更改 oracle 或吞掉产品 panic 让它变绿。

原生补充探针：

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File D:/autostack/auto-lang/docs/plans/attachments/767/review-codex-native.ps1 -Worktree D:/autostack/.wt/lang-767/auto-lang
```

## 4. AC 与 SD 对账

| AC | 当前结论 | 证据/剩余问题 |
|---|---|---|
| AC-01 | fail | 注册 CLI 隔离通过；API 三重身份/换壳违反 R1 |
| AC-02 | pass（此 commit） | 源码/HIR 运算矩阵、7 trap 时序、138 独立算术断言与真实 PE |
| AC-03 | pass（此 commit） | 源码/HIR 六极性、嵌套效果、命名实参、精确 trace 与循环重判 |
| AC-04 | partial | 源码静态名字/类型与原 token 负例通过；直接 HIR 签名类型可信门 R2 未满足 |
| AC-05 | fail | 现有图/域/roundtrip 控制通过；程序化身份与签名引用不完整（R1/R2） |
| AC-06 | partial | 约定深输入/Drop、旧帧、循环与事务/执行器全部重演通过；合法 Checked 不应 native panic，R2 未满足 |
| AC-07 | partial | 实际收据/脚本/严格盘点有据；受影响 SD 不能按现状定稿，设计收据 R3 待修 |
| AC-08 | fail / needs_fix | 独立复审完成，合同内发现未清零，不能维持 pass |
| AC-09 | pending | merge-owned；本次未合入、沉淀、批回归、归档或清理 |

SD-01 的数值/短路/条件 for 规则与已观察实现一致；SD-02/03/04 的完整身份与可信门结论需修 R1/R2 后重新绑定；SD-05 有限域边界正确，未关 GOAL-017。supersedes 四个 modify 路径与 new source-scalar 文件均与合同匹配，touched_goals=[] 的理由仍为不新增/关闭目标。canonical/ledger 不发布。

任务路由：重新打开 T-01（R3）、T-02（R1/R2）、T-06（增加反例及重绑验证）、T-07（冻结修正证据/SD）、T-08（修复后的独立复审）。T-03/04/05 在此 commit 的已验证成果保留；修复后受影响路径必须重新验证，不能让本次局部 pass 覆盖未来代码。

Plan 状态回到 `executing`，current_step=3/9；历史 final pass 保留但由本记录取代当前结论。下一责任角色：`auto-plan-work` 在**同一工作树**修复并提交，再 `auto-plan-review` 重放当前反例、有效控制及受影响门禁。无需换树，不修改合同 revision，不将这些缺陷挂债当完成。
