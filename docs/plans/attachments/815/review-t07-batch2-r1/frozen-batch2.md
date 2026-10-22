# PLAN-815 T-07 完整验收 子批2（792 发行矩阵 / 资源许可审计 / V-01..04 字段兼容失败矩阵）

2026-10-22，work 阶段交付。基线=子批1 交接 f6fd0ce7c（code 661c0573c）。
本批含**两个实缺陷的发现与修复**（均由本批验收面暴露，非审查移交）。

## 0. 冻结与身份

- 唯一树 `D:/autostack/.wt/lang-815/auto-lang`，分支 `codex/plan-815-dev`。
- 本批生产代码改动 3 文件：`crates/auto-ui-tools/src/rust_ui.rs`、
  `crates/auto-ui-tools/src/vue.rs`、`crates/auto-api-gen/src/rust_workspace.rs`
  （均为生成器发射的清单/壳层文本与请求路径门，非运行时行为）；测试改动
  3 文件；文档改动 handoff.md（勘误+指纹）与本目录。
- 修复后工具二进制 SHA256 =
  `e319c5364523dbfac0c61d61ecb087b11a8d0261cdfc767bd1dfacc1b4b803f2`
  （worktree target/debug/auto-ui-tool.exe，本轮全部测试即用该形态构建）。
- 代码提交与 docs 提交 SHA 见 §6；merge 门保持 pending（裸 t+UI 触面 tu
  归 merge 冻结后统一执行；本轮未整档，2026-10-14 测试时序裁定）。

## 1. 实缺陷发现与修复

### F-815-T07-B2-01（P1）：交付/dev 工程清单缺 canonical 双直依赖 → 792 fresh/locked 真实构建红

- **发现**：792 driver 首跑（[driver-run.log](driver-run.log)）V01-real-build
  红交付 app `uiminimal` E0433——814 T-05（2aa0529ca）把 rust 生成器组件
  import canonical 化（`auto_ui_runtime::ui`/`auto_lang_core`，
  import_policy.rs 明文要求生成工程 manifest 直依赖两包），但清单发射方
  未随包图：`generate_delivery_app_manifest` 与 `generate_cargo_toml` 均
  仍只声明 legacy `auto-lang`。在库 oracle 即预存红
  `generated_ui_cargo_uses_canonical_double_dependency`（814-B 窗口，
  T-04/T-05 记录在案）。此前无真实 rust 构建面暴露它（lifecycle 真构建
  均为 vue/pnpm；tool_process 的 rust run 为 stub cargo）。
- **首轮修复后**（清单补两行）二跑（[driver-run2.log](driver-run2.log)）
  V01 转为 E0252：壳层 `wrap_example` 头仍发 legacy
  `use auto_lang::ui::{Component, View}`，组件段发 canonical 同名导入——
  双导入撞名。
- **终修**（三处）：
  1. `generate_delivery_app_manifest`：交付 app 直依赖
     `auto-ui-runtime`/`auto-lang-core`（同交付根 path，闭包成员 814 T-02/03
     已随包；default 含 ui-iced 无需 feature 转发）；
  2. `generate_cargo_toml`（dev 成员）+ `ensure_shared_workspace`
     （auto-api-gen，workspace.dependencies 同根窄接线，沿 811/814 先例）：
     `auto-ui-runtime.workspace = true`/`auto-lang-core.workspace = true`；
  3. `wrap_example` 壳层头 canonical 化 +
     `prepare_generated_components` 去重双前缀（legacy+canonical 的
     per-block 头都剥，壳层头唯一供 import）。
- **关闭证据**：driver 三跑 **25/25 全绿**
  （[driver-run3.log](driver-run3.log)+[driver-receipt.json](driver-receipt.json)），
  含 V01 fresh resolve+真实 locked build、V02 重定位 locked build+metadata/
  资源身份、V03 负控族（等长篡改/坏hash/缺失/未登记/漏patch/坏pin/外来
  目录/失败保留恢复/sidecar 外部与根相对/固定名同级/raw TOML 单引号/他应用
  交付存活/恢复复绿）、V04 dev 同源+模板逐字节一致；预存红
  `generated_ui_cargo_uses_canonical_double_dependency` 转绿（补强断言钉
  workspace 形态+SDK+交付清单三面）。

### F-815-T07-B2-02（P2）：vue 请求路径残留 pac.at 存在性门（主声明第二真源）

- **发现**：字段矩阵测试删除主 pac.at 后同请求再生成，报
  `project context failed: pac.at not found in workspace`——
  `VueProject::from_workspace_with_request` 除存在性检查外已不读 pac 文本
  （`_pac_path` 参数弃用），该门是遗留残留；jet/ark/vscode 请求路径均无。
  旧 vscode fields 测试未暴露（pac 一直在盘上）。
- **修复**：移除该门（vue.rs）；SD-04 after 规则
  （主声明磁盘移除不改变已求值请求快照）由此在 vue 面闭合。
- **关闭证据**：`generate_ui_field_matrix_and_pac_snapshot_independence`
  ——删 pac.at 后再生成整树**逐字节一致**（DefaultHasher 全文件对账）。

### 勘误：handoff.md 来源清单 "Inter 字体" 行与实际内容不符

全仓核实无任何 Inter 字体资产、无 @font-face/woff/ttf/base64 内嵌、无
OFL 许可文本（SIL/OFL 命中全为 SILENT/EPSILON 误匹配）。该行已删并在
handoff.md 原位留勘误注记；逐源对账见
[resource-audit.md](resource-audit.md)。

## 2. V-01..04 字段/兼容/失败矩阵（本批新增测试）

| Case | oracle | 结果 |
|---|---|---|
| `generate_ui_field_matrix_and_pac_snapshot_independence`（tool_process，重串行） | 全字段请求（Unicode+空格根/名、zh-only 标题链、theme{}、npm_deps、styles、i18n locale 文件、本地 dep 根带独立 front、front 节点形态）逐字段落生成物：package.json 名/dayjs^/vue-i18n、index.html `<title>中文标题示例</title>`+`__AUTO_COMPOSED_THEME__`+主题名、src/assets/index.css、styles 逐字节拷贝+main.ts 引用、locale 逐字节拷贝+i18n-instance.ts、dep 组件 MatrixDepWidget 编入、front 节点目录即编译源；删 pac.at 后同请求再生**逐字节一致** | PASS（0.67s） |
| `lifecycle_unicode_space_project_builds_dist`（lifecycle，重串行） | V-04：`示例 app` 目录+`示例应用` 包名真实 pnpm install+build → BuildOk completed/error=None + dist 可消费（入口引用的每个 assets/ 文件在盘且 ≥1 个 .js 加载）+ dist 携带 zh 标题 | PASS（lifecycle 串行 4/4，46.9s） |
| 兼容面（存量） | auto-man 145/145、legacy_build_semantics 2/2（--ignored 串行）、tool_process 快速面 21/21 | 全绿 |
| 协议/闭包 | tool-protocol-consumer 7/7（含子批1 四回执）、ui-tool-consumer 5/5（AUTO_UI_TOOL_BIN 真进程）、两 verify-deps.sh deny-by-default PASS | 全绿 |

失败矩阵存量面（负控）沿用子批1/R8 已验集合（坏帧/超限/未知版本/缺能力/
未服务 render/路径角色/安装失败具名/dist oracle 五控/取消树/PM 矩阵），
本批未改动其行为面；792 失败族见 §1 driver V03。

## 3. 资源审计（块 B）

见 [resource-audit.md](resource-audit.md)：逐源对账表（gallery 1 文件/
wm 9+drawlist 5/shadcn 346 含组级 LICENSE+SNAPSHOT.md）、唯一源与历史副本
钉住、embed 面=登记源新钉扎
（`resources_manifest::embed_consumption_matches_registered_sources`，5/5）、
许可旅行边界如实记录（组级 MIT 随组+随二进制；按组件 materialize 不拷
组级许可入用户工程——上游同形，T-09 如需再裁）、handoff Inter 勘误。

## 4. scoped 验证汇总（本轮实际执行）

| 面 | 结果 |
|---|---|
| `cargo nextest -p auto-ui-tools --lib`（全量 no-fail-fast） | **178/181**：3 红=已登记预存（plan593 金样/shell_pack freshness/merged_api crud）；第 4 预存红 canonical 双直依赖**转绿**；零新增红 |
| 裸 `cargo test -p auto-ui-tools --lib` | plan609 族/incremental 在单进程 libtest 串扰红（plan646 已裁定家族，nextest 进程隔离全绿）——不入账 |
| tool_process 快速面 / 重串行面（--ignored --test-threads=1） | 21/21 / **9/9**（8 存量+1 新；[日志](tool-process-serial.log)） |
| lifecycle 重串行 | **4/4**（3 存量+1 新 Unicode；首跑 1 红为负载 flake——隔离复跑 16.5s 过+整批复跑 4/4，[日志](lifecycle-serial.log)） |
| resources_manifest | 5/5（含新钉扎） |
| legacy_build_semantics（--ignored 串行） | 2/2 |
| `cargo nextest -p auto-api-gen` | 54/55：唯一红 `test_plan734_tauri_fixture_matches_generator` **基线预存**（stash 本批改动后同红；TauriGenerator 空白漂移 vs fixture，归 plan734/api-gen 家族，非本批引入） |
| `cargo nextest -p auto-man` | 145/145 |
| 两 consumer fixture + 闭包门 | 7/7、5/5、双 PASS |
| 792 driver 真实全程 | **25/25**（三跑演进见 §1；收据+双轮红日志留档） |
| rustfmt --check 六个净改文件 / git diff --check | 0 diff / clean |

777：本批零 Cargo.toml/依赖边变化（改动均为生成器发射的文本模板与源码），
分支 strict 基线不受影响；master 并集按分工归 T-10。

## 5. 遗留与边界（如实）

- `test_shell_pack_lib_freshness` 预存红根因已定位（本批同族）：
  `generate_shell_pack_lib_sources` 壳层仍发 legacy 前缀，而入库
  `crates/shell-pack/src/lib.rs` 已被 814 T-05 canonical 化（0 处 legacy）。
  未在本批修：该测试在 worktree 内实际运行依赖 auto-os 镜像解析（本组
  无兄弟，经解析序落到主检出 auto-os），修后无法在唯一树内闭环验证；
  修复形态与本批 wrap_example 同款（canonical 头+qualified 再导出映射），
  登记 KNOWN-DEBT 跟进（T-09/batch 归因裁定）。
- `plan734_tauri_fixture` 预存红（§4，stash 验证）归 plan734/api-gen 家族。
- 壳层 `wrap_example` 体内仍有限定路径 `auto_lang::ui::*`（mem_guard/
  iced 入口/AppResult/render_cli/rqhost/client_entry）——合法（auto-lang
  直依赖保留，限定路径无导入冲突）；import_policy 的零 legacy pin 约束
  生成器组件面，壳层模板全面 canonical 化属 814 跟进面，不在本批收缩。
- T-07 整任务勾选归复审；777 master 并集归 T-10；T-08 据本批更新指纹与
  来源清单后仍待完整 T-07 后定稿；merge 门（裸 t+触面档）pending。

## 6. 提交

- code commit：`e22fc7592`（fix(ui-tools): PLAN-815 T-07 子批2 —— 交付/dev
  清单 canonical 双直依赖 + 壳层头 canonical 化 + vue 请求路径 pac 门移除）
- docs commit：本目录 + handoff.md 勘误/指纹更新。
