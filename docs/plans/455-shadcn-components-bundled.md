---
plan_id: PLAN-455
status: drafting
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

# [PLAN-455] 内置 shadcn-vue 组件模板——冷启动免 pnpm dlx

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

- [ ] T1 快照抓取脚本 + 现行 registry 实拍（components.json style 对齐）
- [ ] T2 assets/shadcn-ui 落库 + deps 速查 + SNAPSHOT.md + Sonner 补丁烘焙
- [ ] T3 盘点缺口依赖集 → VueDependencyUsage 扩展位 + OPTIONAL_DEPS 增补
- [ ] T4 build.rs + 生成的 bundle 表 + vue_shadcn.rs（lookup/materialize）
- [ ] T5 run_vue_project 步骤重排 + install_shadcn_components 兜底化
- [ ] T6 cmd_vue.rs 导出路径接同一 API
- [ ] T7 单测三件套 + 既有套件回归
- [ ] T8 冷启动功能验证（login + 长链示例）+ 独立复审

## 待澄清

- registry 是否实有 chart-area/chart-bar 等 split 条目、auto-complete、
  input-group/kbd 等新条目：以 T1 实拍为准；无则进 ALLOWED_FALLBACK 走兜底，
  不阻塞主线。

## 执行记录

（执行期填写：批注式日志 + 最终验证结果）
