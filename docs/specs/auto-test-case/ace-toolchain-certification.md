# ACE 工具链认证（显式组）

> PLAN-1098 SD-06：把"真实 rustc/C/Python 编译认证及其负控"从日常回归里分离出来的显式入口。
> 登记处仍是 [scripts/test-groups.toml](../../../scripts/test-groups.toml)；本文只钉该组的合同。

## 入口与选择

- 组名 `ace-toolchain`，`runner = nextest`，`workspace = experimental/ace`，
  `all = false`：**不被** `cargo t`（默认日常）、`cargo t all`、`cargo t ace` 任何选择隐含选中。
- 选择集（`-E` 表达式逐名列出，不用位置参数——`cargo nextest archive` 不接受位置过滤）：
  `a2c_cert`、`a2c_coll_cert`、`a2c_rt_cert`、`cert_coll_904`、`cert_rt_902`、
  `worker_ct_frontdoor_958` 六个模块，加上 `tests::rust_lowering::a2r_hir_certify`，
  以及 19 个 AV C 类跨端项（`bstr_1064`/`kv_zip_1064`/`stdlib_*`/`module_packages_960`/
  `plan966_ct_product`/`console_coverage_1041` 等）。
- 这些身份由 `test-toolchain` feature **逐测试门控**：不开 feature 的构建里它们既不存在也选不到，
  因此日常组不需要额外的排除表达式。

## 义务

- **真编译**：每个 C 类项必须真实经过 rustc / MSVC cl / Python 解释器，reference 腿与 actual 腿各自独立；
  原 family 的证书、oracle、provider 身份与负控（错 oracle、错 reference、缺工具、坏 golden）全部保留。
- **预算**：显式组不适用日常 60/120 秒；冻结实测 56 例 **88.58s**（另有历史测量 64.2–95.6s），
  父截止沿用组配置 1800s。真实编译组对并发敏感，争用窗口内的红按环境归因并隔离重放。
- **依赖**：工具仍需先按 [module-selection](module-selection.md) 的准备/执行握手准备（debug/release
  profile 见该文的工具表）；认证动作本身是允许的"被测动作"，其依赖不得在测试体内 Cargo 构建。
- **必选性**：触及生成器、ABI、注册的 C/Rust/Python provider、runtime 摘要 pin 的修改，必须显式选该组；
  未运行不得以"日常组全绿"代替。

