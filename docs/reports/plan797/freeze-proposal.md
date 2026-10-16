# PLAN-797 freeze proposal（候选 v0.1）

> 状态：**提案草稿**（F-797-R1-02 可独立准备部分）。
> 本文件**不**发布 ABI、不改 canonical Spec/ledger。
> 最终 freeze 必须在 T-06 完成（真实 semantic contract + 正式 AutoDown
> + 性能复测）并经 T-08 独立 review / T-09 merge 门禁后定稿。

## 1. 拟冻结对象

| 项 | 值 |
|---|---|
| ABI 域 | `autoabi` |
| 候选 profile | `autoabi.win-x64.v0.1-candidate` |
| 首平台 | `x86_64-pc-windows-msvc` / Windows x64 |
| 实现树 | `D:/autostack/.wt/lang-797/auto-lang` branch `plan-797-dev` |
| 独立 workspace | `experimental/auto-abi-probe` |
| 依赖锁 | `experimental/auto-abi-probe/Cargo.lock`（强制入提交） |
| 冻结态声明 | **有限 profile**，非“所有 ABI/平台/未来类型稳定” |

## 2. 绑定交付物

| 交付物 | 路径 | 状态 |
|---|---|---|
| 合同冻结件 | `docs/plans/attachments/797/contract-decision.md` | T-01 已提交 |
| descriptor/layout/validate | `experimental/auto-abi-probe/src/{descriptor,layout,validate}.rs` | T-02 |
| emit/build/link + C 三腿 | `src/{emit,build,link}.rs` + `cside/*` | T-03 |
| 资源/错误/callback fixtures | `cside/interop_resource.*` + `tests/t04_*` | T-04 |
| cache/trace/周转 | `src/{cache,trace}.rs` + `tests/t05_*` | T-05 |
| 独立审定 C header/oracle | `cside/autoabi_v01.h` `autoabi_layout_v01.h` `interop_v02.h` `layout_probe.c` | T-02/03/04 |
| 支持/拒绝矩阵 | `docs/reports/plan797/support-matrix.md` | 本提案 |
| Spec delta 草稿 | `docs/plans/attachments/797/spec-delta.md` | 本提案 |
| 来源/并行边界审计 | `docs/plans/attachments/797/source-parallel-audit.md` | 本提案 |
| 验收脚本 | `scripts/verify-autoabi-797.ps1` | 本提案 |
| 正式 AutoDown 绑定 | T-06 | **pending D-796/D-788** |
| 性能正式复测 | T-06 | **pending** |

## 3. 版本与身份分层（拟）

1. ABI 域 `autoabi`
2. target / profile `autoabi.win-x64.v0.1-candidate`
3. 接口 revision = descriptor canonical fingerprint（blake3）
4. runtime 服务表版本（descriptor `RuntimeReqs`）
5. 语义 schema 版本（当前临时 `autoabi-semref-snapshot/0`；D-796 后
   改为 796 真实 contract 版本并留退役收据）

五者分开存在；任一错配在调用前拒绝，native 被调次数 0。

## 4. 工具链身份（拟冻结；本机实测见 contract-decision §5）

- cl.exe MSVC 19.44.35222（blake3 `443ac315…`）
- flags `-c -nologo -GS- -Zl -W4 -WX -O2`
- rust-lld + SDK 10.0.26100.0
- 缺工具 = infra 阻断，禁止 Rust-repr-C 伪通过

## 5. 正反 oracle 清单（拟）

**正**：三腿全套（含 C→AutoABI→C 嵌套）、fallible out-sentinel、
record 冻结 offset、资源账本/原方 destroy/失败回滚、同步 callback、
cache 命中禁调 producer / 失效矩阵 / 512 churn。

**反**：12 维 admission 拒绝矩阵（native 0）、swap/错 offset/漏调用
exit 恰 1、tampered descriptor、错 offset 控制、uninjected 与失败期望
不符、递归/超深/超预算拒绝。

完整用例 ID 映射见 Plan §6 V-01–V-11 与
`docs/reports/plan797/support-matrix.md`。

## 6. 与 774 / 旧内部 ABI 对照

见 support-matrix §5：774 ABI-A–E **不被修改**，与 `autoabi` **无兼容
关系**。freeze 后也不得回溯改写 774 接受面。

## 7. 明确不在本次 freeze 范围

- 全平台 / 未来类型 ABI 稳定
- Auto 源码 parser 支持全部 ABI 类型
- C header importer、Rust shim 产品、C++ ABI
- DLL loader / 热重载事务、跨线程持久 callback
- 把 candidate 直接改口成 product-supported

## 8. 冻结条件（缺一不可）

1. T-06：D-796/D-788 真实通路接入、临时 semref 退役收据、正式 AutoDown
   执行 + 性能复测。
2. T-07：V-01–11 收口、SD-01–05 终稿 hash、来源/并行审计终态。
3. T-08：独立 review 零阻断，AC-07 pass。
4. T-09：冻结提交门禁（ABI 完整套件/真实 C 三腿 + 实际触面兼容）通过，
   Spec 沉淀与归档合入。

全部满足前，对外表述一律 “`autoabi.win-x64.v0.1-candidate`”。
