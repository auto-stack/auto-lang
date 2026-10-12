# PLAN-767 r2：Codex 独立修复重审

- 日期：2026-10-12；stage：review；review_scope：final；outcome：**pass**。
- reviewer_identity：原 needs_fix 的复审者 Codex，本会话。参与过规划和初审，未参与产品实现或三项修复；本轮未修改受审树。修复会话的自检记录保留为历史，最终独立结论以本记录为准。
- worktree_path：`D:/autostack/.wt/lang-767/auto-lang`；branch：`plan-767-dev`。
- reviewed_commit：`4cc4645021f029459a3b870c34a23eb16bc4ce59`；base_commit：`a9225e8de6e13cc0bd0c519e7c4b380c7276ebfa`。
- 修复差分：相对初审 `033642a27d7404dee5797b195a9b7c680fe6ece5` 为 7 文件 +281/-31；Descriptor、verify、native、scalar_hir 永久回归与三份收据。入口/结束 HEAD 一致、受审树 clean。
- dependency_revisions / spec_inputs：[新鲜输入指纹](review-codex-repair-inputs.tsv)，含与上轮 hash 比较。Cranelift 0.126.2、rustc 1.98.1；Cargo 锁、schema、canonical Specs 和门禁均未改变。
- contract_revision：2；主计划 §5..§7 与 [原冻结合同](review-codex-contract-r2.md) 内容相同，SHA-256 `f9ba251f8a6d7f19e30bc9d26bae453fe11ea5ddd90a29fdd0257236cbbc9204`。原 10 份冻结 artifact hash 全部保持，未覆盖历史失败证据或改变 oracle。

## 1. 发现闭环

| 发现 | 不变量与作用域 | 独立实际证据 | 结论 |
|---|---|---|---|
| P767-R1 / P1 | domain 仅对两个注册三元组返回 Some，其余 None；bind 构造 Bundle 前拒绝未知身份，writer 拒绝未注册/错域 | 原 6 反例 PASS；两个域各 7 个身份变异组合均 domain=None、bind.descriptor-unregistered、canonical.identity-conflict，合法控制保持 | closed |
| P767-R2 / P1 | 所有 Function 类型 params/result 先过模块边界门；body locals/places/exprs 已过边界门；defs 类型用 get；native 投影用 get/Option | 原两反例拒 verify.bad-reference；新探针覆盖两个域每个参数/结果槽、未使用签名、types.len 与 u32::MAX、local/expr 类型入口和合法 Ref 控制；永久回归通过 | closed |
| P767-R3 / P2 | 设计公式为 `(b>0 && a<min+b) || (b<=0 && a>max+b)`，与原正确实现一致 | 本轮 138 项独立数学断言及完整 trap/源码/HIR 门禁通过，未按错误文档修改后端 | closed |

Verifier 中旧 `unwrap_or(I32)` 尚存，但当前所有实际入口已先经过引用边界门；未发现越界签名/body 类型引用能取得 Checked 的路径，native 也不再直接索引类型表。结论限于当前 core/scalar 域，不推导未来数据资源类型。

## 2. 本轮独立执行

所有编译和测试均在专用 worktree 执行。

| 命令 / 探针 | 结果 |
|---|---|
| `cargo test --manifest-path experimental/ac-core/Cargo.toml --locked` | **145 pass、0 fail**；另 1 个既有显式 ignored 的 60s 执行器专项未计通过 |
| `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-scalar-767.ps1` | 14/14 步、45 例 PASS，fmt 与 locked all-targets 零 warning |
| `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-741.ps1 -SkipMainGates` | 13/13 步 PASS |
| `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-source-745.ps1` | 13/13 步、74 例 PASS |
| `python scripts/acc_inventory.py --root . --output docs/reports/743-acc-hir-contract --check --require-decisions` | exit 0、CHECK-OK，47 条人工结论严格闭合 |
| 原 [冻结 probe](review-codex-probes.rs)，在新 rlib 上重编译 | **16/16 PASS、CONTRACT_FAILURES=0、exit 0**；[原始输出](review-codex-repair-frozen-output.txt) |
| 新 [补充 probe](review-codex-repair-probes.rs) | **48/48 PASS、exit 0**；[原始输出](review-codex-repair-audit-output.txt) |
| 原 [native.ps1](review-codex-native.ps1)，传 `-Worktree D:/autostack/.wt/lang-767/auto-lang` | 138 项整数断言 exit 0、空输出；短路 exit 0、stdout 空、stderr 精确 `abaabaabababaabaaba`；[结果和产品 hash](review-codex-repair-native-results.tsv) |

本轮 CLI binary SHA-256 `7c845525027acf977bafc41407c807ba86b1383bf49fc4e5b2b76eac736e6fdf`；支持库 `70aeb8c8737956f2d23a3e094e5a0a8e7246cba8a9cc7ec7beb19a4998a4e124`。后续门禁重编译产物后，将原生程序及 16/48 项探针再次重放，绑定结束时实际 CLI/rlib 指纹。未冒用旧产品 hash。

补充 probe 沿用 [初审报告 §3](review-codex-r2.md) 的编译入口：取当前 rlib，rustc `--edition=2021 --extern auto_ac_prototype=<rlib> -L dependency=<deps> -L native=<实际 Windows native library path>`，替换源文件和 crate-name 为新 probe，运行时传 worktree 根目录。初次探针因诊断导入路径错误未编译，改为公开 `atom_text::Diagnostic` 后形成上述实际证据，产品未修改。冻结 probe 和原生源/脚本均原样重放；原生 build/run 各 60s 截止。

base 到 HEAD 没有 `crates/`、根 Cargo/config 或 `auto/lib` diff；按计划合同未跑根 t/tv/docs_gen/taa。显式 ignored 专项及相关执行器实现/门禁未变化，不宣称该 ignored 测试通过。

## 3. AC 与 SD 审定

| AC | 结论 | 绑定 |
|---|---|---|
| AC-01 | pass | 双身份及未知/错域拒绝，CLI 隔离/writer 控制 |
| AC-02 | pass | 数值/trap 门禁与 138 项独立数学 oracle 实际重演 |
| AC-03 | pass | 短路上下文/调用次序、条件 for/continue 门禁和精确 trace |
| AC-04 | pass | 原名字/类型/来源定位重演，签名引用可信门补齐 |
| AC-05 | pass | R1/R2 不变量闭合，图/归属/roundtrip/程序化永久回归 |
| AC-06 | pass | 145 项原型测试、旧 741/745、深输入/事务/执行器控制 |
| AC-07 | pass | 新鲜 hash、严格盘点、SD 审定、设计公式一致 |
| AC-08 | pass | 本轮独立最终复审，合同内缺陷清零 |
| AC-09 | pending | merge-owned T-09 未执行 |

SD-01..05：[新冻结副本](review-codex-repair-spec-delta.md)，SHA-256 `312462d19d4c8f03641ccb9ef0605ed6191c0f4eb69db7624bdd2ee2c541d491`，逐字对应受审提交。SD-02/03 精确身份映射、两个拒绝路径、签名边界/后端投影均有据；SD-01/04/05 保持原数值/CLI/收据与 ACC 未达成边界。四个 modify 目标存在，一个 add 目标待 merge 创建；canonical Specs 和 ledger 尚未沉淀。

遗漏/延期/workaround 审计：三项均同树修复并补永久回归，无以 CLI 已拒绝或继承旧实现为由豁免；无新增生产接线、目标关闭、ACC 代际承诺或未授权延后；没有新的合同内阻塞。KNOWN-DEBT 原三项历史记录标 closed。

## 4. 交接

Plan 为 **reviewed（8/9）**。保留历史 needs_fix 与修复自检，权威独立 reviewed_commit 为完整 `4cc4645021f029459a3b870c34a23eb16bc4ce59`。

pending_closeout_gates：T-09 实际合入、SD/导航/ledger/index、批回归到期判定、归档 9/9、wt-guard clean 后 worktree/分支/空组清理与收据。next_responsible_role：`/auto-plan:merge`，获合入授权后执行。本轮没有合入、归档、清理。若受审实现或语义输入再变化，应对新基线重新复审。
