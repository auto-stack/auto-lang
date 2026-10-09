# PLAN-738 Phase 3 / T-09 收据：`2c1b4a763` 差异范围完整分类

- 任务：T-09（冻结入场版本并核清批量格式化范围）
- 审计对象：`2c1b4a763^..2c1b4a763`（566 文件，108488+/48935-，全部为 Modified、全部 `.rs`、全部位于 `crates/auto-lang/`）
- 基线核验：worktree `D:/autostack/.wt/lang-738/auto-lang` @ `plan-738-dev@2c1b4a763` clean；auto-down 兄弟 `895f8d0` 未动；与 [738-phase3-baseline.json](738-phase3-baseline.json) 一致。
- 最终合入 diff 比较基面：merge-base(master, plan-738-dev) = `6d69dbdc78d99f23ba90a37c9559f7fb54dd7be3`，分支自基面起 19 个提交；合入面以 `git diff 6d69dbdc7..plan-738-dev` 复核（master 后续移动由 merge 阶段重算）。

## 方法（可复核）

1. 纯格式机械核对：取每个文件的**父 blob**，用 rustfmt `1.9.0-stable (88d9e12ae1 2026-08-18)`、默认配置（本仓无 rustfmt.toml）、`--edition 2021`（auto-lang crate edition；两个 edition-2024 crate auto-xml/auto-vm 无改动文件）格式化，与**子 blob** 逐字节比对。相等 ⇒ `pure_format_verified`。
2. 37 个含 `mod X;` 声明的根文件在临时目录会因子模块解析失败——将父提交 `crates/auto-lang` 整树 `git archive` 解包后在真实模块结构下原地格式化再比对。
3. 残余差异（rustfmt(父) ≠ 子）逐 hunk 人工审查；注释/空白级残余用「去注释+空白归一后令牌等价」证明。
4. 逐文件 parent/child SHA256 与分类在 [738-phase3-scope.json](738-phase3-scope.json)；审计脚本（可重跑）在 [738-phase3-scope.audit.py](738-phase3-scope.audit.py)。

## 结果：566 = 552 + 4 + 2 + 8，全部归属，无未知语义差异

| 分类 | 数量 | 判定依据 |
|---|---|---|
| pure_format_verified | 552 | rustfmt(父)==子，逐字节等价 |
| comment_or_whitespace_only | 4 | fmt-parent vs 子仅注释缩进/空白；去注释归一后代码令牌等价 |
| golden_asset_update | 2 | a2r 金样：幻影 `a2r_std::io` 导入面移除（R2 诚实 provider 门） |
| functional_r2_fix | 8 | 语义差异逐条审查，见下节，与 R2 报告逐项对账 |

## 4 个注释/空白级文件

`lib.rs`、`tests.rs`（尾随注释对齐）、`libs/std.rs`（空行）、`vm/ffi/stdlib.rs`（纯注释：契约声明位置移至 production() 的说明）。

## 2 个金样文件

`test/a2r/14_modules/002_pub_use/pub_use.expected.rs`、`test/a2r/14_modules/004_wildcard_import/wildcard_import.expected.rs`——删除幻影 `use a2r_std::io::*` / `pub use a2r_std::io::say`（a2r-std 无该宿主模块，R2 金样更新项）。

## 8 个功能修复文件（语义面，全部进入复审）

| 文件 | 残余行数 | 内容 | 对 R2 报告项 |
|---|---|---|---|
| `vm/native.rs` | 199 | `declare_contract_identity`/`declare_method_contract_identity` API + production() 后 http/json 契约声明 + plan738_contract_identity_tests | 生产契约回填 |
| `stdlib_assembly/host.rs` | 109 | Rust 适配壳：cast 解包、包装块（状态侧信道/724 三参元数分派）提取、AsyncHTTPStream facade、(status,body) 元组适配、**包装形状 async 豁免 + 元数不匹配 Resolved-only 放行臂** | Rust 适配壳（Phase 3 T-10 修复对象，rv3 §4.3 已登记） |
| `stdlib_assembly/emission.rs` | 84 | C 发射诚实面：stdlib auto.* ext 面 skip（仅登记方法面）、`#[c]` 不透明类型注册、c.stdio 直绑 witness 重写 | C 目标 ext 面诚实诊断 |
| `trans/c.rs` | 24 | `c_opaque_types`（FILE 按符号名渲染）、`stdlib_ext_types`（ext 静态面 TARGET_UNSUPPORTED） | 同上 |
| `compile.rs` | 17 | C 会话 `c.*` 声明命名空间解析（VM/Rust 维持 not-found） | C 目标 c.* 解析 |
| `vm/disasm.rs` | 16 | CLOSURE 变长捕获描述符走查对齐（addr+count+n_args+captures×6B） | CLOSURE 走查对齐 |
| `trans/rust.rs` | 4 | 模块级 unsupported stdlib 不发射幻影导入（逐符号门不变） | Rust 发射门 |
| `back_proxy.rs` | 4 | 冗余 `register_std_shims`/`register_stdlib_ffi` 撤销（production() 已做；重复注册按 R2 撤销语义删同 ID 契约） | 冗余 merge 撤销 |

## 与 R2 声明的对账

R2 交接称「约 530 文件 bulk rustfmt（抽查纯格式化）」。本轮升级为**逐文件机械证明**：556/566 为非语义变更（552 纯格式 + 4 注释级），8 功能 + 2 金样构成全部语义面，与 R2 报告逐项列出的工作（生产契约回填/CLOSURE 对齐/冗余撤销/C c.* 解析与 ext 诊断/Rust 适配壳/金样更新）一一对应，**未发现 R2 报告之外的未声明语义变更**。`host.rs` 的包装豁免/Resolved-only 臂即 rv3 §4.3 已登记的 strict 缺口，T-10 修复。

## 证据边界

- 本任务不跑全档测试（格式性质由机械等价证明，非测试绿）。
- rustfmt 版本为本机当前 stable（1.9.0-stable 2026-08-18）；若 R2 执行时使用不同版本，任何版本敏感差异都会表现为残余差异并已在上方逐条审查——不存在未解释的残余。
- 映射：T-08（历史）/AC-08/SD-01..07 的证据边界——8 个功能文件即 Phase 3 后续任务（T-10..T-12）的触面清单；金样/格式面进入最终完整复审。
