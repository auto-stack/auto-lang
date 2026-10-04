# PLAN-743 执行上下文记录（run context）

> 本文件是 PLAN-743（ACC 自举能力盘点与 HIR 阶段契约）的执行期固定上下文快照。
> 性质：审计记录，不是需求。后续重跑盘点以 `source-manifest.json` 的受管输入 hash 为准，
> 本文件的 HEAD 记录仅定位"本次盘点在哪个提交上做的"，不要求执行永远停留在此提交。

## 1. 执行身份

| 项 | 值 |
|---|---|
| 计划 | PLAN-743 plan_revision 1 |
| 执行 worktree | `D:/autostack/.wt/lang-743/auto-lang` |
| 开发分支 | `plan-743-dev` |
| 基分支 | `v0.6-dev`（主检出） |
| 执行基点 commit | `c1ac219e73ee2ed1ef6ba8dfec49c131bfbf1a75` |
| 起草时 HEAD（计划 §4.1） | `a6c0f9691dbb8e176b028291de758bbffa499fe3` |
| 起草→执行差异 | `git diff --stat a6c0f9691..c1ac219e7` = 仅 `docs/plans/.next-id`（742→743）与新增本计划文件；七份证据输入 hash 零变化（见 §3） |

## 2. 离线与范围限制

- 本机（Windows）与主机器（739/740 等在途计划的产出）无网络同步；本盘点不推断主机器未 push 的工作。
- 主机器若恢复并带来新输入：保留本次 manifest，先做差异报告与结论失效标记（计划 §10.4）；
  重跑盘点属本计划内校准，扩大仓库/目标/验收范围则须修订合同。
- 本计划为 Category A（纯调研工具/文档）：不修改 `crates/**` Rust 源码、不改 auto/lib/*.at、
  不跑 cargo 测试档、不动生产 Cargo workspace/lock、不写外仓、不建外仓 worktree。
- 盘点工具只用 Python 标准库（本机 Python 3.14.2，兼容 3.11+），不下载包、不访问网络。

## 3. 固定输入指纹（执行基点实测 SHA-256）

### 3.1 七份证据输入（起草时已在计划 §4.1 登记，执行期逐项复核）

| 来源 | 执行期 SHA-256 | 与起草差异 |
|---|---|---|
| docs/specs/auto-hir/project.md | `04e95f7ce4805b7e409bf89af49fe9c43be0632157043470099e76e1fe83bee6` | 无 |
| docs/specs/auto-ac/project.md | `31f20c811950600a1afcdb5fb05a8199713188e7e3b54e1872dbd336f9870b17` | 无 |
| docs/specs/aavm/project.md | `79f913d0ea673fb85e6db2d50bb6e7aefd354855183db12b0089a57187aca5be` | 无 |
| docs/design/strategy/auto-native-backend-evolution.md | `becb2f45d66bffc81b98bea33c3c6c0bb40418be2f958cd1b9a58de283a5be27` | 无 |
| docs/design/strategy/auto-hir-design.md | `c51caa7dd454010904c1f2055f57539ce01343ee5e7fee1fe4613e38e8c3ba0d` | 无 |
| experimental/ac-core/src/hir.rs | `251d973f75a991675c7331bbc9dd683e61b7e830a9615120cd5cd6d5c6f29e63` | 无 |
| experimental/ac-core/src/verify.rs | `1ddc9cf6669ead02948222347e009ec8922a6183271dda178b43e21be5dc0ae2` | 无 |

结论：七份证据输入起草→执行零漂移；变化分类 = 空（仅计划簿记新增，无源码/Spec 变化）。

### 3.2 被盘点的 Auto 自举源码（8 个源模块）+ 注册表

| 来源 | SHA-256 | 角色 |
|---|---|---|
| auto/lib/token.at | `963ccc1df73b86e510db95f13dda0e3ea17462f7c3f4b7652947e9f9f3381d74` | 源模块 |
| auto/lib/lexer.at | `c92bcce8c6432ffd80050f013ac3e53af0f6c5133a9c16ec47faef1df2bf9a01` | 源模块 |
| auto/lib/parser.at | `377129b9b06afec72b5ccbc947b6d50c778b7a41f3df2f22ad6c99048b2ea559` | 源模块 |
| auto/lib/typeinfo.at | `9eed6f5f243225c17d4f79e5c53918d03a525e2e2bd99998b6ba6f95c3bb254a` | 源模块 |
| auto/lib/codegen.at | `a95a484913b95084903f45ddcf4891664c2e080e520511b87c692cd1d8fe55e0` | 源模块 |
| auto/lib/engine.at | `d8db6813edf82e0d6a763d7e707d33f37535190980399a6aa989d2ad5d748328` | 源模块 |
| auto/lib/a2r.at | `e3eddb46ab2e5d3e188a8f7ee8a78feda1891931cb99bc2a1392f694a459e2ae` | 源模块 |
| auto/aavm.at | `0ff06f099253e1aa1c8d83ef2810771f10d5a01e5388729354dd9fa249d4ee9a` | CLI 入口（第 8 源模块） |
| crates/auto-lang/src/lib.rs | `4d6c4703c667f99cfe55e6f96471e89c7e82346c788a56b907447260786e8fbe` | 注册表宿主（AUTO_LIB_FILES_V2 证据段） |

参考脚本（非盘点主体，供工具边界对照）：`scripts/aavm_lib_xref.py`
（`b049f1c6…`）、`scripts/aavm_shim_inventory.py`（`f62f3913…`）。
索引文档：`docs/design/00-intro.md`（`af85aa94…`）、`docs/specs/overview.md`（`37762159…`）。

## 4. 741 API 身份（事实基线，只读引用）

PLAN-741 已归档（`docs/plans/archive/741-ac-hir-native-core.md`），其交付即本计划的输入基线：

- Schema 身份：`auto.hir.core.draft` / revision `1` / profile `core-i32-draft`
  （descriptor：`experimental/ac-core/schema/core-i32.atom`；未知 tag/case/字段/能力一律拒绝；
  文件内 `checked` 类标记无授权作用）。
- 类型面：仅精确 `i32`/`bool`；运算 `add_i32`/`mul_i32`（必须 `overflow: trap`）与 `lt_i32`。
- 构造唯一入口：`verify::verify(bundle: hir::Bundle) -> Result<CheckedModule, Vec<Diagnostic>>`；
  `CheckedModule` 私有构造，仅暴露 `bundle()` / `into_bundle()` / `required_capabilities()`。
- 后端能力门：`native::capabilities_check`（文档 `requires` 须由 `--capability` 显式提供，
  缺失报 `capability.missing`）；入口选择 `native::find_entry`（EntrySelection）；发射
  `native::lower_object`（Cranelift 0.126.2 ObjectModule → COFF）。
- 链接/运行：`link::{find_rust_lld, find_sdk_um_dir, link_object, run_exe, LinkReceipt}`；
  PE 以 `/entry:ac_start /subsystem:console kernel32.lib` 无 CRT 链接；溢出 trap = `ExitProcess(70)`
  （测试 profile 约定，不是完整异常 ABI）。
- CLI：`ac-probe check <file>`（bind+verify，退出码 0/1/2）与
  `ac-probe build <file> --entry <DefId> --output <exe> [--capability N]... [--support-lib L]...`；
  失败构建不覆盖既有成功制品。一键验证：`scripts/verify-ac-741.ps1`。
- 已知边界（741 自述）：单模块、无源码 adapter、无跨模块/泛型/所有权/字符串聚合、
  不宣称 Auto 源码→native 或 ACC 自举。

## 5. 注册表快照（AUTO_LIB_FILES_V2，lib.rs @ 4d6c4703）

拼接消费面清单（Plan 517 W2 剥离 `use auto.lib.*` 行后拼接）：

```text
auto/lib/token.at → auto/lib/lexer.at → auto/lib/parser.at → auto/lib/typeinfo.at
→ auto/lib/codegen.at → auto/lib/engine.at → auto/lib/a2r.at
```

AAVM v1 归档路径 `AUTO_LIB_FILES`（`auto/lib-legacy/*.at`，12 文件）不属本次盘点主体。
盘点工具须将该注册表段作为受管证据：注册表内容变化 = 漂移，`--check` 非零。
