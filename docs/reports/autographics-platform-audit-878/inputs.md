# 0. 固定输入与身份核验（inputs）

PLAN-878 XP-0 静态审计的输入身份与核验记录。审计合同见
[attachments/878/work/audit-contract.md](../../plans/attachments/878/work/audit-contract.md)。
原始命令输出（本地保留，不入库）：`D:/autostack/plan-artifacts/plan-878/`。

## 1. 输入身份表

| 输入 | 固定身份 | 核验方法 | 结果 |
|---|---|---|---|
| AG 源（AGP + driver） | auto-lang 仓 commit `8b12632d7b5b1b0df72fbb4eef84297a054de31d`（2026-10-28，Plan 861 冻结，"flatten remaining report IndexMut sites, clean probe exit"） | `git ls-tree` 12 个规划基线 blob（877 baseline.md 表）逐项比对 | **12/12 匹配** |
| AG 导出树（只读审计输入） | `D:/autostack/plan-artifacts/plan-878/ag-export/experimental/autographics-{prototype,auto-driver}` | `git archive 8b12632d -- <两个子树>` 解出；对 12 个固定文件重算 `git hash-object` | **12/12 与 blob 一致**；非工作副本、无 git 状态污染 |
| 旧 SDK 保留副本 | `D:/autostack/plan-artifacts/plan-860/consumer-package/experimental/autographics-auto-driver/.sdk-source`；逻辑 pin `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467` | `verify_sdk_copy.py`（复刻 8b12632d `tools/prepare_sdk.py --verify-only` 判定：source-lock.json 逐文件 SHA-256 + symlink/junction 禁止 + root lock 哈希），双侧只读 | **681 文件全匹配、17 本地包、root lock sha256 `5192d08d…` 与 baseline.json.txt 一致**。pin 对象在当前 auto-lang 仓不可达（`git cat-file` 失败）——按合同只使用保留副本，未以当前 master 冒充 |
| SDK 导出内副本（供 metadata 解析） | `<ag-export>/…auto-driver/.sdk-source` | 由 plan-860 已验证副本普通复制（无链接）；复制后重跑同一逐文件校验 | **681/681 匹配** |
| auto-ui 固定对象 | `7095f536747e5180338a470a7d73452c6b550ed1`（规划时 master；现仓 HEAD 已前移至 `f489c7bd`，Plan 876） | `git ls-tree` 6 个 manifest blob（根 + 5 crates）比对；`git status` | **6/6 blob 匹配**；工作副本 clean；只读固定对象 |
| 语言主检出基线 | `8ae51d6afa5c2b2cc3bc72833c6408d212f5680b`（master） | worktree `D:/autostack/.wt/lang-878/auto-lang`，分支 `plan-878-dev` | 建树成功；主检出无非 docs 改动（仅计划簿记） |

## 2. 工具链与解析环境（影响 metadata 结论边界）

- cargo/rustc **1.98.0**；解析仅 `cargo metadata --locked --offline --format-version 1`。
- 已安装 target：`x86_64-pc-windows-msvc`、`x86_64-unknown-linux-gnu`、
  `wasm32-unknown-unknown`、`wasm32-wasip2`。
- `aarch64-linux-android`、`aarch64-apple-ios` **未安装**（按合同不下载）。
  实测 `--filter-platform` 对未安装 target 仍可完成 cfg 级解析（5/5 全部 exit 0），
  但这**只证明 manifest/锁的 cfg 可解析，不证明该 target 可编译**——std 未装、
  无 NDK/Xcode 工具链，编译级结论全部为未测假设。

## 3. 命令与退出码（2026-10-28 执行）

| 命令（摘要） | 退出码 | 证据（本地） |
|---|---|---|
| `git ls-tree 8b12632d -- <12 文件>` 比对 | 0（12/12） | `plan-878/verify_inputs` 会话记录 |
| `git archive 8b12632d …` + 12×`git hash-object` | 0（12/12） | 同上 |
| `python plan-878/verify_sdk_copy.py` | 0（PASS 681 files; 17 packages） | `plan-878/verify_sdk_copy.py` |
| `cp -r <plan-860 .sdk-source> <ag-export>` + 复验 | 0（MATCH 681 files） | 同上 |
| `cargo metadata …` ×10（AGP/driver × 5 target） | **10/10 exit 0** | `plan-878/meta-{agp,driver}-*.json`（含 .err.txt 均空） |

## 4. 输入限制（如实记录）

- AGP `research/`、两仓 README、driver `research/` 历史证据只作上下文引用，不做分类对象。
- SDK `vendor/`（autodown-test-parser）与 `auto-test-case` 不在 driver 依赖闭包内，
  仅记录在场（见 dependency-map §4）。
- 未读取 registry 依赖的源码缓存（`~/.cargo`）逐包核销第三方实现——第三方按
  metadata 图 + 已知平台后端结构分类，关键边标 unresolved。
- 本文件所有结论可由上表命令在固定对象上重放；重放不依赖任何网络。
