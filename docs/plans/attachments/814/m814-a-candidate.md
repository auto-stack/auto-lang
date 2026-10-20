# PLAN-814 M814-A 候选（T-13 冻结 @ `abe9c9900`）

2026-10-20。stage: work（T-13 完成冻结）。792 交付 driver 已跑通（25/25，
EXIT=0），候选 SHA 冻结于 `abe9c9900`。**T-13 完成不等于 AC-08 pass**——
AC-08/AC-09 由 T-14 独立阶段审查绑定本 SHA 判定；AC-10 归 T-15。

## 1. 冻结对象

| 项 | 值 |
|---|---|
| worktree / branch | `D:/autostack/.wt/lang-814/auto-lang` / `codex/plan-814-dev` |
| **冻结 SHA** | `abe9c9900`（交付合同更新至新包图——792 driver 25/25 恢复全绿的收口提交） |
| base | master `203e2ce91` |
| A 发布包 | `auto-lang-core 0.1.0`（纯 SDK）、`auto-ui-compiler 0.1.0`（UI 编译域）、`auto-ui-runtime 0.1.0`（骨架——**能力未发布**，B/T-04 迁入） |

## 2. A 阶段已发布事实（API/支持清单）

### auto-lang-core（纯 SDK）
- **公开面**：Parser（Core 场景/显式 session 装配；UI 文法门面签名恒在，
  feature off=显式诊断）、VM 全链（Codegen/AutoVM/VirtualFlash/
  native_registry/loader）、trans 出口（trans_rust/trans_c + with_session）、
  run 族入口（run_with_capture/execute_autovm 等）、config/i18n(hook)/
  frame(779 端口)/a2r_std、stdlib_assembly/database(UICache 指纹注入)、
  use_scanner/dep_scanner/os_paths。
- **特性**：default=[with-file-history]；`ui-dialect`（显式 UI 文法组合）、
  `code-editor`（a2r diff 双轨）、`python`/`streaming-http`/`mem-profile`/
  `test-cases`。
- **闭包**：normal/build/dev 零 UI 文法/UI 编译/UI 运行/Man/旧巨包
  （cargo tree 三边断言 + 消费 fixture 钉住）。

### auto-ui-compiler（UI 编译域）
- **公开面**：`ui_gen::api::generate_component_from_file`（.at→Vue SFC
  真实生成出口）+ ComponentGenOptions/GeneratedComponent；四生成器
  （vue/jet/ark/rust）+ widget registry/bp；aura 模型/提取/校验；a2ui；
  style DSL 分析（iced 适配器留上层运行时）；action_config；lucide 单源
  表；route 分析；`generator_fingerprint()`（build.rs 指纹公开）。
- **闭包**：SDK + 显式 ui-dialect + 轻前端 crate；零旧巨包反依赖。
- **测试选择**：ui_gen 862（tu 单进程 libtest）+ nextest 215（日常档含
  t/tf 的 `-p auto-ui-compiler`）。

### 上层兼容（auto-lang）
- default 转发 `ui-dialect`/`code-editor`——Scenario::UI 默认装配、旧
  Parser 公开 UI 门面、`auto_lang::{aura,a2ui,ui_gen,route,ui::style,
  ui::action_config,ui::iced::lucide_generated}` 旧路径经再导出全保持。
- A 期真实 runtime 仍在本包（ui/ 运行时未迁——B/T-04）；生成物 import
  指向当期真实可用实现（vue 生态路径策略）。

## 3. 证据矩阵（V-06 + A 涉及 V-01..05）

| Case | 状态 | 证据 |
|---|---|---|
| V-01 纯 SDK/单源 | ✅ | `language-sdk-consumer` 5/5（Core VM 执行 42 输出/手工字节码管线/trans_rust 产物）；auto-vm 真实消费者切换+冒烟；closure 三边 0 UI；SDK lib off 1627/1627 |
| V-02 兼容/装配 | ✅（A 范围） | 旧路径再导出编译 probe（上层 --tests 全编译）；plan806/dialect/scene 24/24；SDK on 1713/1713；文法拒绝负控（feature off 显式诊断不静默） |
| V-03 真生成出口 | ✅（A 范围） | `ui-package-consumer` 4/4（真实 SFC/script setup/状态/vue 生态 import）；ui_gen 862/862 金样；**当期真实 runtime 消费=A 期上层路径（旧行为保持），B 切新 runtime** |
| V-04 余边/寿命 | ✅（A 已知面） | frame 779 端口单源（plan716 探针）；i18n 路由恢复；store_extra/strip 下沉单源；autodown 残边已归因记 T-04 |
| V-05 选择面/发行 | ✅ | t/tf/tu 别名扩包（池 6851，ui_gen 862 保持选中）；777 strict pass ×4（基线 `d91948ee1`）；fresh（path+独立 lockfile）+ `--locked` 双 fixture 消费通过；**792 交付 driver 25/25（EXIT=0 @ `abe9c9900`）**——V01 fresh resolve+build/V02 A→B 重定位 --locked/V03a..j 全负控（资源缺失/坏 hash/漏 patch/不兼容 pin/外来目录/sidecar 外部 path/raw TOML）/V04 dev 同源；SD-08 三接缝修复：DELIVERY_CRATES 16→19、投影按 manifest 分 spec（含 SDK spec）、ui_gen 指纹源路径更新 |
| V-06 A 真实边界 | ✅（除 792） | 上表合成：禁止边确实消除+非 UI 消费执行成功+旧 API 同源+A 公开生成出口可用；缺新 runtime 能力处显式拒绝（ui_grammar_unavailable） |

## 4. 后续（T-14/T-15 归属）

1. ~~792 交付 driver~~ **已完成**（25/25，EXIT=0；首轮受管红暴露的三处结构
   漂移即 `abe9c9900` 修复面——投影 4 缺 1/ui_gen 源根/runtime 未 vendored，
   红不回退主检出语义保持）。
2. **T-14（next）**：同树独立阶段审查——绑定 `abe9c9900` + 本文件 + r2 §5.1
   阶段性质，验 AC-08/V-06 与 Spec 子集（SD-01/02/04 按 §2 事实备稿）；
   merge-owned 门禁（裸 t+tv/tu/tt）保持 pending 记录（2026-10-14 时序裁定）。
3. **T-15（T-14 pass 后）**：冻结门（裸 cargo t + 触面 tv/tu/tt）→ 实际
   master 合入 → Spec/受管 ledger 增量与发布收据；保原树/分支继续 B。

## 5. 已知限制（A 边界内的诚实声明）

- auto-ui-runtime 仅骨架：B/T-04 迁入真实 renderer/宿主 adapter 前不列
  任何已支持能力。
- desktop.at 等 shell 资产家族留上层（T-04 归属定夺）；vue.rs 测试经
  MANIFEST_DIR 引上层单源副本。
- autodown natives 在 SDK 为 fallback 拒绝形（跨仓依赖不进 SDK；autodown
  消费者迁移=T-04）。
- `code-editor`/`ui-dialect` 为 SDK 侧显式组合 feature：纯 SDK 默认闭包
  不含其实现，属设计内边界而非缺口。
