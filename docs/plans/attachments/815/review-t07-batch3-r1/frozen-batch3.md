# PLAN-815 T-07 完整验收 子批3（复审 review-t07-batch2-r1 三项 P2 修复）

2026-10-22，work 阶段交付。基线=子批2 复审对象（docs `503c9f67f` /
code `e22fc7592`）。本批 = 复审
[review-t07-batch2-r1](../../review-t07-batch2-r1/review.md) 三项 P2 的
一次修齐；无生产代码改动（driver.py 为验收驱动器扩展，两个 .rs 改动
均为测试）。**T-07 勾选仍归复审。**

## 0. 冻结与身份

- 唯一树 `D:/autostack/.wt/lang-815/auto-lang`，分支 `codex/plan-815-dev`。
- 改动面：`test/fixtures/modularity/generated-ui-consumer/driver.py`（V05
  扩面+framed I-TOOL Python 客户端）、`crates/auto-ui-tools/tests/tool_process.rs`
  （新增 boundary 测试+字段矩阵扩展）、`crates/auto-ui-tools/src/rust_ui.rs`
  （测试模块新增请求权威钉）、三份审计文档纠正（batch2.md /
  batch2/resource-audit.md / handoff.md）+ 本目录。
- 生产代码（src 非测试区）零改动；工具二进制身份不变（driver V05 构建自
  交付包源，收据内含该构建产物 SHA256 `b040c820…`——这是**包内构建的**
  工具身份，与 worktree 构建身份区分）。

## 1. F-815-T07-B2-R1-01 关闭：随包新 tool 构建 + 真实外部消费

driver **V05** 八例扩面（[driver-run.log](batch3/driver-run.log)、
[收据](batch3/driver-receipt.json)，全新 scratch `scratch-b3-r2`，**33/33**
——旧 25 项原样保留全绿）：

| 例 | oracle | 结果 |
|---|---|---|
| V05a | 交付包 A（V01 fresh 锁）内 `cargo build --locked -p auto-ui-tools`，`auto-ui-tool.exe` 在位 | PASS 83s |
| V05b | 重定位包 B 内同构建（tool 从受管交付来源可建） | PASS 79s |
| V05c | B 的工具 describe：`tool_name=auto-ui-tool`、protocol=1、五操作集 | PASS |
| V05d | validate_project 合法请求 → ok | PASS |
| V05e | generate_ui 真实生成（frontends=[vue]、19 文件）+ **资源控制**：shadcn button 从交付工具资源面物化，字节=包内 vendored 源 | PASS |
| V05f | 未知协议版本 → 具名 `unknown_protocol_version`（client=99），零副作用，同流后续 describe 仍可答 | PASS |
| V05g | 不服务 profile（plan9）→ `invalid_field` 于任何副作用前，消费根零产出 | PASS |
| V05h | 坏帧 → 流致命非零退出（exit=2） | PASS |

发行形态未新增：既有约定=源码包 workspace（members=app+26 crate +
根级输入），V05 即在包内构建并运行工具。

## 2. F-815-T07-B2-R1-02 关闭：资源审计事实纠正 + 完整闭包盘点

- 三处文档原位纠正（勘误链如实保留）：`batch2.md` §勘误、
  `batch2/resource-audit.md`（§1 补 Inter/壁纸/壳声明三行、§3 勘误链、
  §4 四路径区分）、`handoff.md` 来源清单（Inter 行恢复正确 owner +
  壁纸/壳声明行）。
- 新增 [batch3/resource-audit-closure.md](batch3/resource-audit-closure.md)：
  交付闭包 26 crate **生产**编译期/embed 资源完整盘点（10 组：
  Inter 三字重+OFL、壁纸 ×2、桌面壳 .at ×5、gallery/wm/drawlist/
  shadcn、根级输入 ×5、根锁）+ 实物对账
  [embed-closure-reconcile.json](batch3/embed-closure-reconcile.json)
  （重定位交付包逐字节，八组全 match）。
- 事实更正要点：Inter 三字重及 OFL 文本实存于 auto-ui-runtime
  （renderer.rs:1850-1852 include_bytes）且随包字节一致；三组 UI 资产
  源**确实直接进 792 包**（vendored 构建输入路径，原"不进包"断言混淆
  了 app 资产 allowlist 与 crate 源随包两条路径）。

## 3. F-815-T07-B2-R1-03 关闭：字段 × profile × 操作消费矩阵

新增 [batch3/field-profile-matrix.md](batch3/field-profile-matrix.md)：
22 字段组 × 三操作 × 七 profile 的逐字段适用性/唯一源/锚定/消费点/限定
依据/证据对照；五条语义总则先行（op≠mode、布局不按 output_root 重定向、
锚定=project_root、请求在场即权威、携带字段非缺陷）。新增区分性测试
三个（全绿，见矩阵 §4）：

- `validate_field_rules_output_root_and_input_sources_boundary`（真实
  工具进程，run 正控/file 角色负控/dir 负控/相对锚定区分）
- `external_backend_request_is_authoritative_over_disk_decl`（请求值 vs
  盘值区分 + exe_name 同款三态）
- 字段矩阵测试扩展（output_root 声明根不被创建+报告根=实际布局根）

复审点名未断言项逐一落位：mode（§2 mode 行+正控）、output_root
（角色/布局钉）、input_sources（锚定+角色）、has_rust_sidecar（冻结
语义如实声明+载荷消费证据）、external_backend（消费链+权威钉）、
逐 profile 适用性（§2 每行+§3 代表证据）。

## 4. scoped 验证汇总（本轮实际执行）

| 面 | 结果 |
|---|---|
| 792 driver（V01..V05 全程，全新 scratch-b3-r2） | **33/33** |
| 新增/扩展测试 | boundary 1/1；external_backend 权威钉 1/1；字段矩阵扩展 `--ignored --exact` 1/1（0.66s） |
| embed 闭包对账（源 vs 重定位包） | 八组全字节一致 |
| 详见计划 §9 子批3 work 记录（lib/工具面回归 + fmt） | 见 §9 |

777：本批零生产 manifest/边改动；master 并集按分工归 T-10。

## 5. 遗留（不变，如实）

- `shell_pack freshness` 预存红（KNOWN-DEBT，根因定位在子批2 §5；
  T-09/batch 归因裁定）、`plan734_tauri_fixture` 预存红（归
  plan734/api-gen 家族）。
- T-07 整任务勾选归复审；T-08 完整前置待本批复审通过后定稿；T-05/T-06
  勾选随本批（联合 finding 已全闭）一并归复审；merge 门（裸 t+触面档）
  pending 归 T-10。
