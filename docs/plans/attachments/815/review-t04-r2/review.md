# Plan 815 T-04 R2 定向复审

**needs_fix：R1 的 01..05 已关闭，06（P3 格式）继续开放；无新增功能 finding。**

受审 Plan r3；冻结 SHA `dbd295f050c05ef37450adbd0f8f835a150e077f`，修复基面 `90aed7fc7df875987cf9014a37fdc256e83fdae6`，T-04 原交付基面 `7ffa2d88d9f5ee500a1f759f0a388f6899e825f2`。树 `D:/autostack/.wt/lang-815/auto-lang`，分支 `codex/plan-815-dev`，首尾实现 clean。依赖 M814-A=f6e07aceeeea3cb12698c747d61db5be43f7458c；M814-B 尚非本次 T-04 的前置要求。首尾身份、规范/锁/配置/修复文件 hash 见 baseline.json.txt 与 final-baseline.json.txt。

审查者 Codex `/root` 延续 R1 独立审查上下文，有早期合同/账本上下文限制，未参与本轮实施，未创建另树/agent 或假称其他模型。本次为 phase/提前定向复审，不是 T-09 完整终审。

## 六项关闭表

| R1 ID | 本轮结论 | 独立证据 |
|---|---|---|
| 01 交付缺来源 | CLOSED | 25 项清单与必需路径检查通过；真实 A/B 各 26 package，locked/offline metadata 无越界 path；独占副本分别删 model/toolchain 两个 Cargo.toml 均 exit101，恢复两项 exit0。 |
| 02 compiler 测试缺依赖 | CLOSED | tempfile 已挂新 owner；store_deps 回归与 nav/sidebar 两条锚均真实执行通过，不再被 E0433 阻断。 |
| 03 旧公开路径 | CLOSED | 两个永久外部兼容测试通过，覆盖六个 Man/三个根级旧入口及相邻生成支持面；新增模块是纯 pub use 转发。 |
| 04 资产消费断线 | CLOSED | nav/sidebar 2/2、布局 17/17、viewer 离线 frozen-lock 安装通过；画廊来源围栏指向唯一新源且未删断言。扫描残留仅历史/说明性路径，无再读旧资产的现行实现。 |
| 05 plan734 金样 | CLOSED | 夹具搬至 tests/fixtures/*.rs.txt，真实生成器逐字比较仍保留并通过；原 .rs 自动测试目标不再误编译，不以放宽比较规避。 |
| 06 格式 | **OPEN，P3** | 原多余空行已清，新增 `crates/auto-lang/src/lib.rs:3479` 转发行仍须展开；单文件 rustfmt --check exit1。 |

唯一待修复行：

```rust
pub use auto_ui_compiler::store_deps::{extract_store_deps_from_file, extract_store_imports_from_ast};
```

格式器期望：

```rust
pub use auto_ui_compiler::store_deps::{
    extract_store_deps_from_file, extract_store_imports_from_ast,
};
```

复现（在唯一 815 树）：`rustfmt --check --edition 2021 --config skip_children=true crates/auto-lang/src/lib.rs`，证据 fmt-root.log.txt。按完整 T-04 净交付范围的同式检查也只有此处差异（fmt-delivery.log.txt）。修复差异中恢复到原基面的六个示例文件不在净交付范围，不能把它们既有格式差异再算本计划新问题。

## 本轮执行和证据复用

- affected-tests.log.txt：5/5（闭包、store_deps、nav/sidebar、plan734）；compat-integration.log.txt：5/5（两兼容、三媒体 consumer）；request-seams.log.txt：2/2。不是整档，筛掉的其他测试不计通过。
- layout-parity.log.txt：17 个几何用例通过；viewer-lock.log.txt：离线 frozen-lock EXIT0，关闭安装脚本。
- 777 strict：PASS，0 violations/4 grandfathered；未刷新基线。git diff --check 通过。
- narrow-tool-protocol.log.txt：35/35（18 协议 + 17 真工具进程）；fixture-process.log.txt：4/4；fixture-closure.log.txt：闭包 PASS。合计本轮 51 条 Rust 定向测试通过，另有 17 布局用例；并非全包或整档。

792 work 证据的可复用性经过内容核验：原收据 25 条全成功，A fresh build 159s、B 重定位 locked build 138s；本轮**没有重复这两次冷构建**。A/B 各自 5131 个源文件与当前树逐文件对照，差异只有两份规定的 Cargo.toml 投影与三个文件的格式：两投影与实际算法（含相邻注释剥除）逐字相等；三文件经相同纯输出 rustfmt 后完全相等，其中 rust_ui.rs 的生产前缀本来就逐字相等。根五输入、producer root lock SHA256 与当前相同，A/B 已构建应用存在；本轮 metadata 重新检查单源、包数与 root 边界。见 existing-delivery-identity.json.txt、existing-delivery-differences.patch.txt、projection-check.json.txt、retained-source-format-identity.json.txt、reuse-identity.json.txt。收据的 producer.git_head=旧修复基面、dirty=true 是 work 在提交前验证的历史事实，不冒称它已记录新冻结 SHA；当前来源绑定由本轮内容核验补齐。

先前工具包三项内容红未被本轮修复触及，不重复全包，也不宣称转绿。work 的 gallery_vue_golden 内容红归因仍是其提供的基面/814 探针说明，本轮只关闭搬迁读取路径，没有独立重跑重型画廊内容围栏；完整 B 真实生成/运行矩阵与正式归因仍按 T-09/merge 所属验收办理。

一次初始组合探针误把 auto-lang 内部 lib-test 纳入且未加别名要求的 test-cases，因 auto_test_case 缺失失败；随后改为真正需要的三个集成目标与单独 Man 请求缝，全部通过。该配置错误日志保留在 compat-and-seams.log.txt，不作为产品新 finding，不据此启动整档。

## 验收与下一步

AC-01/02 当期生成主体与请求/旧兼容范围通过；已触及的 AC-04 修复范围通过。AC-05 卫生不变量因 F06 尚未闭合，因此不能给本次阶段 pass。完整 B 的 AC-01..04 其余范围、T-05 runtime、T-08 规范与 T-09 全计划终审继续 pending；既有 A/L 发布记录保留。本轮规范判断见 spec-delta-reviewed.md。

重开/保持 T-04 未完成，按实际已勾九项校正 current_step=9/17、status=executing。原唯一树仅修该转发行格式并提交；下轮代码、依赖、配置、输入不变时可复用本轮五项关闭和 792 绑定证据，只复查格式与来源稳定性。最终 pass 后 merge-owned 冻结裸 t 与必要触面门仍 pending；本轮没有跑整档、合入、归档、清树或写 canonical/live ledger。
