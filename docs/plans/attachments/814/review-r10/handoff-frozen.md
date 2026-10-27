# PLAN-814 T-09 交接文档（handoff）

2026-10-21。stage: work（T-09 送审交接；B 阶段 work 全部完成）。

## 1. 冻结对象

| 项 | 值 |
|---|---|
| worktree / branch | `D:/autostack/.wt/lang-814/auto-lang` / `codex/plan-814-dev` |
| base | master `f6e07acee`（M814-A 合入点） |
| B 阶段提交链 | `4949c1a45`（T-04 主体）→ `49835196b`（T-04 收口）→ `30d397a13`+`a2fdef7f3`（777）→ `872a1b2c2`（fmt）→ `a75c616f5`（T-05）→ `26b8b33bd`（T-06）→ `257e02454`（T-07）→ `463fd029c`+`8afba8d01`（T-08 边修复+基线）→ `e1c0f4770`（T-08 生成器修复）→ `780415083`（T-08 driver 断言）→ `6063b7a94`（R8-F03 健康修复批）→ `236827f1c`（R9-F01..04 修复批） |
| 依赖 | auto-down `c39ac7f`（未变，组内兄弟 detached） |

## 2. B 阶段交付事实（T-04..T-08）

### 包图终态
- **auto-ui-runtime 0.1.0（真实能力发布）**：ui/ 全树（~20.4 万行：iced
  renderer/desktop_protocol/code_editor/terminal/mpv/native_dock/scroll/
  dynamic/session/shell/mcp_server 等）+ host_adapters + back_proxy +
  assets/*.at 五件 + UI 动态构建管线（dynamic_build.rs，原 lib.rs
  689-2588 区）+ plan370 支持件。装配扇入单点
  `install_ui_host_capabilities`（文法装配留旧兼容包；零 inventory
  submit——r3 §5.3 登记权边界）。
- **auto-lang-core（SDK）增补**：use_diag/take_use_diags（带外通道单源）+
  test_code/test_file 族下沉（纯 Core 执行面）+ `tests_seam` 跨 crate
  可见性勘误。
- **旧 auto-lang（兼容/组合层）**：全 UI feature 转发 runtime（组合面逐项
  保持）；→runtime 依赖边 default-features=false（no-UI 消费者缺席语义，
  T-08 捕获修复）；测试期 UI 依赖清零（T-04 临时 dev-deps 收口）。
- **canonical import 路径策略**（import_policy.rs 单源）：rust 生成器 136
  处 + auto-man 模板/manifest + 生成工程 manifest 双直依赖
  （auto-ui-runtime[ui-iced] + auto-lang-core）；新旧映射表在档。
- **非 UI 消费者切 SDK（V-01）**：auto-vm（A 期）+ auto-gen +
  auto-playground + a2r-actor-tests（B 期三枚，闭包零 UI 零旧包）。

### 生成器缺陷修复（T-08 再生暴露，预存缺陷非迁移引入）
1. truthy 过投射（PLAN-710 D-6 对比较位误包）——比较位改 as_str 值投影；
2. on_click 单元兜底与消息承载事件冲突——点击族在场跳过。

### 兼容退出清单（compat 退出=后续批次，本 Plan 保持）
- `auto_lang::ui::*` 全路径再导出（V-02 pin 4 测试锁面）——**保持**；
- auto-man 自身代码 ~110 处旧 facade 引用——**保持**（815 承接工具搬迁）；
- trans/a2r 泳道发射 `auto_lang::a2r_std::*`——**保持**（消费者仍依赖旧
  包；a2r 轨 canonical 化归后续批次）；
- 上层 inventory 登记例外（r3 §5.3）——**保持**（登记权 auto-lang+
  compiler 两处，runtime 零 submit）。

### 资产/测试归属终态
- assets/*.at → runtime（include_str 相对深度天然保持；compiler
  desktop.at 测试经 ../auto-ui-runtime 单源读）；
- 61+6 个 UI 域测试 → runtime ui_tests/（挂载门逐项保持）；plan446 批一
  → dynamic_build；ac_av 归属勘误回 auto-lang（零 ui 依赖）；
- 语料单源留 crates/auto-lang/test/ui（多消费者，相对路径引）；
- 九档选择面：别名扩包（t/tf/tt/tb/taa/ta/t3 含 runtime）+ r5 验证器 62
  条显式重写——missing=0/unresolved=0 全核销
  （[t07-closeout](t07-closeout/)）。

## 3. 验收证据矩阵（V-01..V-06，B 终态）

| Case | 结果 | 关键证据 |
|---|---|---|
| V-01 纯 SDK/单源 | ✅ | 四真实消费者（auto-vm/gen/playground/a2r-actor-tests）闭包零 UI 零旧包；language-sdk-consumer 5/5；SDK 三边 0 UI（每任务复钉）；SDK 1631/1631 |
| V-02 兼容/装配 | ✅ | V-02 pin 4 测试（Parser 17 字段/构造族/方言、桥 17 方法、VM/NativeInterface、门控根再导出+类型身份同一编译证明）；六 provider fixture 43 测（缺席臂恢复）；上层 --tests 全编译；**旧样式路径**（R9-F02：`ui::style::IcedStyle`/`iced_adapter::*` 门面恢复，reviewer old-path pin 过）；**声明面六形态**（R9-F03：no-default/ui/ui-headless/ui-dialog/code-editor/default 全 0 错） |
| V-03 真闭环 | ✅ | **双端对照完成**（[v03-dual-run](v03-dual-run/)：002-counter 同源，Vue Playwright 与 VM MCP 三状态两操作逐点一致 0→-1→1，截图+结构化状态在档）；tu e2e 生成工程真编译 2/2（canonical import+双直依赖）；792 driver 24/24（V01/V02 真实 cargo build）；shell-pack canonical 再生+freshness；**比较投影类型感知复修**（R9-F01：reviewer 四反例+numeric-silent 双侧类型化，根锁上下文全编译过） |
| V-04 余边/寿命 | ✅ | provider/frame/i18n/theme/mem_guard 91/91；SDK 生产码零 UI 引用复钉；777 ui_to_language=0 不变量（基线@463fd029c） |
| V-05 选择面/发行 | ✅ | 九档全核销（r5 验证器+selftest 负控）；777 strict violations=0（豁免 4→3：shell-pack→auto-lang 边消除）；792 24/24 EXIT=0；新 consumer fresh+locked |
| V-06 已被 B 全量取代 | n/a | A 期 Case；B 以 V-01..05 全量为准 |

## 4. 诚实限制与预存（非本 Plan 面）

- 双端对照覆盖=单示例三状态两操作（合同"有代表性"口径；非全组合
  逐像素 parity——合同未要求）；
- 预存红族（基线同在，未动）：auto-macros value 宏测试、shim-metadata/
  auto-cache lib 测试、plan734 tauri fixture、auto-cosmic-demo View 字段、
  auto-man merged_api_client/plan593 css golden（203e2ce91 同红）；
- 负载 flake 族（隔离绿，在案=plan707/plan484 charts/plan796 性能门）；
- KNOWN-DEBT 既有项（792 投影 ignore 感知等）不因本批变化。

## 5. SD 增量（可审定正文=[sd-b-proposals.md](sd-b-proposals.md)；审定通过后归 T-11/T-12 受管沉淀）

- **SD-03 全文**（拟落库 `docs/specs/auto-ui-runtime/project.md` 完整
  正文）：职责边界/DAG+default-features 语义/公开能力与 feature 面/
  装配单点+r3 登记权/单源状态与生命周期/资产与测试归属/canonical import
  与生成工程合同/发行与兼容——逐节可审定。
- **SD-01/02/04/08 增量**：逐目标 before-hash（A 发布版）+保留/新增规则
  /理由/AC；**生成器两修复具名反例**（truthy 比较位 `p.running == "1"`
  E0308 反例、oncontextmenu.prevent+单元兜底 E0308 反例）。
- SD-05/06/07：无 before/after 规则差异，随沉淀附 B 事实注记。

## 6. merge-owned 门禁（pending，冻结提交统一执行）

裸 `cargo t`（全日常面）+ 触面档：本批触及 VM/编译器/生成器 → `tv`；UI
代码生成 → `tu`；trans 无改动（a2r 泳道未动）→ tt 按需；文档生成器无改
动 → docs_gen/tb 按需。tf 主检出到期判定。门红须归因；改码后复审重冻。

## 7. next_responsible_role

**review（T-10）**：同树独立上下文，完整 DAG/公开兼容/生产/寿命/测试
选择与 SD 审查，绑定分支头 `780415083`（或其后 fmt 收口）+ 本 handoff。
