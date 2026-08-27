---
plan_id: PLAN-457
status: archived
feature_name: 内置 shadcn-vue 组件模板——冷启动免 pnpm dlx
author: [zhaopuming]
created_at: 2026-08-27
updated_at: 2026-08-27

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 0
total_steps: 8
---

# [PLAN-457] 内置 shadcn-vue 组件模板——冷启动免 pnpm dlx

## 变更摘要

`auto run`（Vue 后端）的 Step 4 通过 `pnpm dlx shadcn-vue@latest add <comps>`
获取 UI 组件源码：registry 往返 + CLI 下载 + 内部再跑一次 install，
冷启动多花约 8–15s，且 `gen/` 被清理后必然重放。这些组件本质是静态
.vue/.ts 源文件，本计划将其以**快照形式内置于 auto-man crate**，冷启动
时直接拷贝写入项目；未被内置覆盖的长尾组件保留 dlx 兜底。预期收益：
常规示例冷启动完全离线化、砍掉整条 dlx 链路；导出命令 `auto vue` 同步受益。

## 目标 / 架构方案

1. **快照资产**：`crates/auto-man/assets/shadcn-ui/<comp>/**` 收录
   detect_shadcn_components 全目录（~60 名）对应的组件源文件（从现行
   registry 实拍）。`SNAPSHOT.md` 记录上游来源/registry style/日期；
   `tools/shadcn-snapshot/snapshot.mjs` 提供可重放的抓取脚本。已知的
   上游兼容性手工补丁（Sonner 图标改名等）直接烘焙进快照文件，
   `fix_shadcn_compatibility_issues` 保留为兜底。
2. **嵌入机制**：auto-man 新增 build.rs，遍历 assets/shadcn-ui 生成
   `OUT_DIR/shadcn_bundle.rs` 的 include_str! 常量表（编译期嵌入，
   运行期零 IO、二进制自包含）；每组件额外附一行 deps 速查
   （`deps.tsv` 一并嵌入），记录该组件相对脚手架基线的额外 npm 依赖。
3. **新增依赖走既有模式**：不做事后 package.json 字符串手术；缺口的
   长尾依赖（预期 ⊆ {cmdk-vue、@unovis/vue、@unovis/ts 等}）按 Plan-442
   P0-1 先例扩展 `VueDependencyUsage` 检测位 + OPTIONAL_DEPS 表，由
   generate_package_json 在 install 前声明到位。
4. **流程集成（步骤重排）**：
   - run_vue_project 把组件物化挪到 Step 3 安装依赖**之前**
     （新 `materialize_ui_components`：write-if-missing 写入内置组件 +
     生成 components.json 缺件），Step 4 的 `install_shadcn_components`
     退化为兜底——仅对磁盘仍缺失的非内置组件走原 dlx 路径；
   - `crates/auto/src/cmd_vue.rs` 导出路径接入同一公开 API，逻辑去重。
5. **边界与兼容**：write-if-missing 尊重用户手改；检测目录 ↔ 内置目录
   由单测强制同步（未收录者必须登记 ALLOWED_FALLBACK 白名单，防
   catalog 漂移）。

## 测试设计与验收标准

- 单测（auto-man --lib）：
  a) detect 目录 ↔ bundle 覆盖同步闸门（含白名单）；
  b) materialize 幂等 + write-if-missing 不覆盖既有文件 + utils.ts 保护；
  c) bundle 内容 sanity（关键导出存在）。
- 全量 `cargo test -p auto-man --lib`、`cargo test -p auto-lang --lib`、
  docs_gen 保持绿；零新增编译警告。
- 功能冷启动验证：005-login 与一个长链示例 rm -rf gen 后 `auto run`：
  日志无 `shadcn-vue@latest add`（全内置场景）、出现物化日志、
  vite ready 且页面 HTTP 可达；（可选）autoui-verifier 双端对齐抽检。
- `auto vue` 导出路径同验：产物含 ui 组件且不再触发 dlx。

## 任务清单

- [x] T1 快照抓取脚本 + 现行 registry 实拍（components.json style 对齐）
- [x] T2 assets/shadcn-ui 落库 + deps 速查 + SNAPSHOT.md + Sonner 补丁烘焙
- [x] T3 盘点缺口依赖集 → VueDependencyUsage 扩展位 + OPTIONAL_DEPS 增补
- [x] T4 build.rs + 生成的 bundle 表 + vue_shadcn.rs（lookup/materialize）
- [x] T5 run_vue_project 步骤重排 + install_shadcn_components 兜底化
- [x] T6 cmd_vue.rs 导出路径接同一 API
- [x] T7 单测三件套 + 既有套件回归
- [x] T8 冷启动功能验证（login + 长链示例）+ 独立复审

## 待澄清

- registry 是否实有 chart-area/chart-bar 等 split 条目、auto-complete、
  input-group/kbd 等新条目：以 T1 实拍为准；无则进 ALLOWED_FALLBACK 走兜底，
  不阻塞主线。

## 执行记录

**快照（T1/T2）**: 61 名逐一 `pnpm dlx shadcn-vue@latest add` 实拍成功 57；
`toggle`、`chart` 目录经 registryDependencies 闭包随取（运行时互引校验自洽）。
default registry 缺货白名单 = {auto-complete, input-otp, native-select}
（闸门单测常驻）。Sonner 图标改名已烘焙进快照。出处与重放：
assets/shadcn-ui/SNAPSHOT.md + tools/shadcn-snapshot/snapshot.sh。

**依赖接线（T3）**: 外部依赖盘点后唯一缺口 = charts 家族 @unovis/vue +
@unovis/ts（^1.6.7）→ OPTIONAL_DEPS + VueDependencyUsage::chart（引号
收尾 marker 判别，五条 chart 路径任一命中即声明）。login/charts 冒烟语料
均不含 chart 标记 ⇒ 该组保持静默不产生死依赖；声明逻辑由
package_json_component_groups_conditional 单测锁定。

**集成（T4-T6）**: vue_shadcn.rs 沿用仓库 rust-embed 先例（无 build.rs，
资产编译期嵌入）；run_vue_project / build_vue_project 双路径把物化挪到
npm install 之前，install_shadcn_components 改为 remaining 过滤后的注册表
兜底；crates/auto/cmd_vue.rs 导出路径接同一 API 并同样过滤。

**验证（T7/T8）**: cargo test -p auto-man --lib 238 绿（含新增 5 项）；
-p auto-lang --lib 3216 绿；docs_gen 4 绿；构建警告均为存量，触碰文件零新
增。功能冷启动（rm -rf gen 后 target/debug/auto run）:
- 005-login ≈9s HTTP 200，日志 Step3 "Bundled ui components: 2 copied"、
  兜底段直接 already installed (skipping)，全程无 shadcn-vue@latest 调用;
- 024-charts 物化 4 文件、vite ready、同样零 dlx。

**复审补录（第二遍扫描）**: 全仓 `install_shadcn_components|npm_install`
调用点复扫发现 run_tauri_project（crates/auto-man/src/tauri.rs）仍为旧序，
冷启动会退化回全量 dlx 兜底（功能正确、优化缺失）→ 已补 materialize 步骤
对齐 run/build 两条路径并回归绿。vscode 后端经查本就无 shadcn 安装阶段，
非遗漏。其余项：vue_shadcn 无 TODO/FIXME/dbg!；运行时路径零 unwrap；
白名单三项为 registry 实拍缺货的设计决策（含闸门单测），非 workaround。

**排障注记（非本计划缺陷）**: Windows 待删句柄延迟释放曾令上轮 gen 树在
锁释放时被排队删除一并回卷；孤儿 node/esbuild 以 CommandLine 路径过滤点
名清除后复验通过。提示：脚本化轮询 vite 就绪时不应假定固定 :3000（占用
时 vite 自动跳端口）。
