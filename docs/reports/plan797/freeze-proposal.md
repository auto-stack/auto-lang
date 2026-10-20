# PLAN-797 freeze proposal（候选 v0.1）

> 状态：**提案（R9 修复后重置）**——R9 复审的 5 项开放 findings 已
> 修复并附永久反例（F-R4-02 descriptor 如实化：出参列表/三出参
> ledger/two_step handle 出参/callback ctx/i64 元素视图 + record
> mut/out 生产面与原生提交回滚控制，header 文本为独立 oracle；
> F-R4-07 include 闭包改为消费 `cl -showIncludes` 真实依赖并在命中时
> 逐一重验（注释隐藏/续行/宏 include 三兄弟反例 + 系统头闭包为永久
> 测试）；F-R4-09 每 step 独立完整发布父路径请求计时，body-edit 样本
> =edit+query 请求对，逐样本先落盘（含 Job 峰值内存/I/O/meter）再断
> 言；F-R4-10 精确表示字节暂存门（原 R5 长字段名输入在分配前拒绝）+
> worker 累计产物预算前后检查 + 并发 GC 全链；F-R4-12 本计划 crate
> 自身告警清零、freeze/verify/SD 事实对齐；R9 追加：缓存适用域
> （源/include 目录·target·工具链·flags·负依赖清单入信封重验）、
> 遮蔽三态反例、计时边界到判定+落盘与一请求一行、memo/索引入预算、
> GC 收据断言）。本文件仍**不**发布 ABI、
> 不改 canonical Spec/ledger；冻结登记以 T-08 独立 review 零阻断 +
> T-09 merge 门禁通过为条件。

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
| 正式 AutoDown 绑定 | `abi/interop-v1` recipe 注册（contract 追加）+ `abi-worker`（abiworker/1）+ 父侧 `test_cases/abi.rs` + 可执行语料 `crates/auto-lang/test/cases/autoabi/`（4 checks 含跨进程 restart 命中与类型化负例） | T-06 `35794e5ec` |
| 真实语义绑定 | `ac-sema-contract` DeclRef/TypeDesc/ContentVersion；退役收据 `attachments/797/semref-retirement-receipt.md` | T-06 `35794e5ec` |
| 性能正式复测 | R9：C 消费腿·单 case·共享 store·统一冷体（mix-alt<seed>），**每个 query 请求携带编辑后 deck**（计时边界=父入口→该请求 typed 判定完成→原始行落盘；edit 为父侧本地替换、耗时入样本 edit_ms）：body-edit(edit+request) P50/P95/P99=592.0/767.0/774.0ms、reject P50/P95=87.0/121.0ms（31/30 样本；数字随采样时段浮动、原始行为准）；compute=31+1 逐样本断言；**一请求一结果一原始行**（成功/失败/infra 均有；阶段/cache/compute/Job 进程+树峰值内存·I-O/meter 含 GC 收据）由适配器在下一请求运行前持久化 | R9 本轮 |

## 3. 版本与身份分层（拟）

1. ABI 域 `autoabi`
2. target / profile `autoabi.win-x64.v0.1-candidate`
3. 接口 revision = descriptor canonical fingerprint（blake3）
4. runtime 服务表版本（descriptor `RuntimeReqs`）
5. 语义 schema 版本（真实 `ac-sema-contract` 内容版本；descriptor semref
   = `binding_hash()`（probe797/autoabi 两 decl 的 logical+content blake3）；
   临时快照已退役，收据 `attachments/797/semref-retirement-receipt.md`）

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

## 7.5 终稿身份绑定

- `spec-delta.md`（R8 重置稿，descriptor/3 同步）SHA256：`E67557BA253D51BA5C19C1233C7EC4E4CB89BAE88A6CF0B1CEC519FC3A64366E`
- canonical 输入 hash 明细见 spec-delta.md 输入表（worktree 终态实测，
  R8 复核未漂移）。
- work 侧 V-01–11 工作面证据（R8 后）：probe workspace 全绿（t02 20+
  t03 7 + t04 6 + t05 9 + t06 6，含 R8 新增：header 独立 oracle、
  mut/out 原生控制+负控、原长字段名暂存反例、三兄弟 include 反例+
  系统头闭包、并发 GC 全链）+ 统一 runner 语料 + test-cases 家族 +
  父生命周期/预算反例族 + 语义反例族。
- R8 报告：`docs/plans/attachments/797/review-r8/review.md`（reviewed
  commit `bf2455758`）；修复提交见计划 §9 R8 修复轮记录。

## 8. 冻结条件（缺一不可）

1. ~~T-06~~ **已满足**（`35794e5ec`：真实绑定/退役收据/正式执行/复测双门）。
2. ~~T-07~~ **已满足**（本终稿 + support-matrix/spec-delta 终稿 hash +
   verify 脚本 acceptance/closeout 扩面；V-01–11 工作面收口见 §5 与矩阵）。
3. T-08：独立 review 零阻断，AC-07 pass。
4. T-09：冻结提交门禁（ABI 完整套件/真实 C 三腿 + 实际触面兼容）通过，
   Spec 沉淀与归档合入。

全部满足前，对外表述一律 “`autoabi.win-x64.v0.1-candidate`”。
