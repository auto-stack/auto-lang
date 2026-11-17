# 文档式迁移收口（Document Migration Closure）

> 状态：**delivered（PLAN-1066 B 期收口，2026-11-17；`plan_revision` r4）**。本文为该迁移的
> 权威收口口径。历史：PLAN-1062 于 2026-11-15 交付四类语料迁移告 `delivered`，其“全量收口”
> 结论其后被证伪；2026-11-16 更正为“部分交付、四类残留交 PLAN-1066”，PLAN-1066 的 A 切片
> 先行交付正确性（IF-1066-A-CORRECTNESS，master `7c131286d`），B 切片完成剩余载体迁移后
> 于 2026-11-17 收口（交付提交与复审收据见归档计划
> [1066-autodown-migration-gap-closure](../../plans/archive/1066-autodown-migration-gap-closure.md)）。
> 保留原文件的“1062 冻结集合”表述（75 组/176 份/1174 位置）作为历史事实；当前口径以下文为准。

## 当前口径（B 收口后的实测事实）

| 面 | 实测事实 | 证据 |
|---|---|---|
| 固定 case 组 | **76 组**的活跃 `include` 全部是文档式：183 个被 include 的 case 文件 = document 183 / 旧围栏 0 / 无法识别 0；968 调度器 11 项已迁 `scheduler.document.case.ad` | `scripts/test-governance/document_case_active_scan.py`（`infra-fixed` 常驻 2 测） |
| 原生目录（五根） | 发现集 **1060 项**（vm 506 / a2r 250 / a2c 118 / a2ts 85 / cookbook 101），**1171 判决角色**（1161 现存文件 oracle + 9 空 H + 1 个 doc-only stdout 角色） | `crates/auto-lang/test/governance/plan1066_a_obligations.json`（A-DEN 逐键核） |
| 其他消费者边界 | **70 份**文档不属 N（a2r 36 / cookbook 32 / a2c 2）；A 期 78 = 70 + 已入 N 的 10 份 doc-only VM 文档 − 2 份新增 project 文档 | 同上 `e78` |
| 四族发现/执行 | `discover_{vm,a2r,a2c,a2ts}` 与 `entry::run_*_file_test` 以 `<stem>.case.ad` 为唯一作者源；缺文档具名报缺（VM 的 doc-only 项无 sidecar oracle 也保留可见并具名失败）；doc-only 树照常发现执行 | `crates/auto-lang-core/src/{test_runner,entry,legacy_doc_for_trans}.rs` |
| 多文件工程（P） | 两工程改为 `main.project.case.ad`（5 源内联 + 19 条 `exists`/`contains` 有限期望）；受管物化后调用原 `transpile_rust_project`；旧 `*.expected.rs` 仅历史 | `crates/auto-lang/src/tests/a2r_tests.rs::run_project_case` |
| 250 编译门 | 文档源 + 显式 prepare/verify 两段式收据（`plan1066-a2r-compile-receipt/1`）：159 compiled / 55 external-dep-skip / 36 known-broken / 0 obsolete / 0 unexpected = 250；默认入口不 spawn rustc | 显式组 `a2r-compile`（`--ignored --test-name` 重放） |
| 生成入口 | 普通 bootstrap/`generate_expected`/`GENERATE_EXPECTED` 及全部 `.wrong.*` 旁路退出；唯一作者入口是显式 `conformance_proposal`（30 例闭集、只写显式 scope、部分失败即失败） | `crates/auto-lang/src/tests/conformance_tests.rs` |
| 身份 | trace/inventory/关联合同以文档为身份（`legacy_document_identity`、`cookbook_doc_identity`）；`verify_inputs` 为活跃门；旧 sidecar 仅冻结历史 | `crates/auto-lang-core/src/test_cases/{execution_trace,inventory,inventory_wrappers}.rs` |

## 阶段门与证据复用（不变）

- 每迁移阶段：语义门（v2 投影 vs 冻结 manifest 全等）→ 运行门（新旧双侧实跑，expected-red
  元 oracle：故意红控制组两侧同轴）→ 活入口核验 → 受影响固定组。
- 证据按身份复用（同代码/输入/配置）；表征白名单（origin 表征键、artifacts 载体、聚合
  `oracle_sha256` 仅内容等价时可异）；expected-red/环境归因/原条件排除分列。
- 迁移对照记录：`document_case_migration.py`（manifest/pair/verify/controls）+
  `auto-case migration-inventory --schema-version 2 / helper-inventory`；v2 为默认，历史 v1
  仅显式查询，缺字段/失败**不回退** v1。

## 边界与后续准入

- 旧围栏/`.at`/`.as`/`.expected.*` 为冻结历史：显式回放可用；五根活跃路径零**未具名**回读。
  回读点由常驻分类审计（文件级 allowlist + ±2 行读窗口）逐族具名：migration 比较工具静态旧侧、
  NC-27 doc↔sidecar 对拍控、AAVM 实验语料、外仓 `book`、五根外 `ffi_dep`、sidecar 合同合成
  fixture、`a2r-actor-tests` 外消费者、相邻 `a2gd`/`a2j`/`tscn` 族。新增未具名读者按文件拒绝。
- 具名债务（不因归档消失）：
  - **a2ts 85 例金样早于现行 emit**（`console.log`→`globalThis.console.log`），整族显式重放统一红且
    日常 `#[ignore]`；a2r 1 例（`16_interop/024_nested_async_await`）、a2c 8 例同类漂移。
    关闭=重生成金样或修 emit，改写期望需用户显式许可。
  - **VM shim/runner CWD 敏感挂起**：`test_file("test/runners/vm_tests.at")` 全量重放在
    `99_plan230/001_struct_f64` 后不再推进（>20min），shim 路径与 wrapper 的 CWD/path 通道不同；
    文档权威不改变该判定，shim 取证以镜像/路由等价控制度。
  - **外仓 book 生成器**：`book_listing_tests::generate_book_expected` 仍非 ignored 且写 `book/` 仓。
- 新增语料准入：新 suite/case 一律文档式；新 file-corpus 读者须登记（`include_str/include_bytes/`
  字面 `test/` 路径扫描口径）；support fixture（单文件解析器/库资产）具名不迁；外仓
  （book/UI/Man/auto-term）归原 owner。
- 生成器：`conformance_bootstrap` 与 `generate_expected` 已退役，普通路径零作者树写入；显式
  `conformance_proposal` 是唯一候选生成入口，33 项独立验证与 trim 比较不变。
