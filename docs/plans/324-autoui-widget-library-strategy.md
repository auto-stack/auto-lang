# Plan 324: AutoUI 通用 Widget 库战略建议（npm 可发布的 Vue 组件库）

> **类型**：战略建议（非立即实施）
> **来源**：auto-musk 前端架构分析 + auto-forge 前端技术债评估
> **状态**：待 auto-lang 团队评估。**建议在 a2vue 关键缺陷修复 + "生成库"能力具备后启动**，不立即建库。
> **关联计划**：312（#[api]+HTTP）、313（SSE）、319（unify-vm-rust-rendering）、314（autoui-mcp）

---

## 1. 背景与问题陈述

### 1.1 当前 AutoUI 的定位空白

AutoUI 生态目前有三种产物，但**缺少最关键的"可复用组件库"这一层**：

| 产物 | 位置 | 定位 | 问题 |
|---|---|---|---|
| **component-gallery** | `examples/component-gallery/` | demo / 文档站 | **不是库**：无 package.json 的 name/exports/发布配置；展示组件但不可安装；从未成功 gen 过（app.at 用了未实现的 nav-link/theme-toggle，无源文件） |
| **packages/auto-forge-ui 等** | `packages/` | 私有应用 | **不是库**：3 个包（auto-forge-ui/auto-lab-ui/auto-playground-vue）都是 `"private": true` 未发布，互相 `file:` 本地依赖，是应用壳而非库。auto-forge-ui 含 marked/mermaid，是 auto-forge 前端的抽取尝试 |
| **a2vue 生成应用** | 各项目 `gen/front/vue/` | 单个应用 | 每个应用各自生成、各自 `shadcn-vue add`、组件无法跨项目共享 |

**空白**：没有"由 Auto 生成、可 npm 安装、给所有 Vue/a2vue 项目复用"的通用组件库。

### 1.2 痛点（来自 auto-musk 实践）

- auto-musk 每用一个新的 shadcn 组件，要手动 `npx shadcn-vue add`（plans/002 §2.2 缺陷 4：a2vue 检测但不自动装）。
- 跨项目（auto-musk / 未来 auto-forge 重构版 / 其它 a2vue 应用）无法共享一套统一组件。
- component-gallery 的组件用法没有"库的 API 文档"形态，只是散落的 page demo。
- auto-forge 前端因各模块独立实现积累了 ~1100 行重复（R1-R5），若有共享库本可避免。

---

## 2. Widget 泛化潜力分析（哪些该进库，哪些不该）

基于 auto-forge 前端提炼的 5 个候选 widget（详见 `auto-musk/designs/001-frontend-widget-design.md`），按"通用 vs 业务专属"分类：

### 2.1 该进通用库（L1，通用 UI 范式）

| Widget | 通潜力 | 理由 | 库中的形态 |
|---|---|---|---|
| **EmptyState / LoadingState / ErrorState** | ✅ 强 | 纯 UI，任何应用必备的状态态；和 shadcn 的 skeleton/progress 同级，但 shadcn 没有现成的"空态/错误态"完整件 | `<EmptyState icon="..." text="..."/>` 等 |
| **MasterDetailLayout** | ✅ 较强 | 主从布局是常见模式（邮件/文件管理/设置），比基础件抽象一层但通用 | `<MasterDetailLayout sidebar-title="...">` slot 化 |
| **StatusBadge（可配置）** | ⚠️ 半通用 | "状态→颜色/文案"映射通用，但 auto-forge 的 22 种 Spec Status 是业务专属。库中提供"可配置映射表"的 Badge，业务层填映射 | `<StatusBadge :status="..." :map="..."/>` |

### 2.2 不该进库（L2，业务专属，留应用层）

| Widget | 理由 |
|---|---|
| **SpecSectionWidget** | 绑死 Spec 的 7 类 section + 状态机，auto-forge/auto-musk 专属 |
| **CrudEntityWidget** | "通用 CRUD 视图"听起来通用，但每实体字段/交互差异大，过度泛化反而僵硬。更适合留应用层 |

### 2.3 component-gallery 现有组件的库化

component-gallery pages/ 里的 ~60 个已验证组件（card/table/dialog/avatar/badge 等，见 auto-musk plans/002 §2.1）本身是 shadcn-vue 映射——它们**不需要重新造**，库化时直接纳入（作为库的导出件 + 文档）。

---

## 3. 三层架构建议

```
L1  通用 UI 库（@auto-stack/ui，npm 发布）
    ├─ 基础件：EmptyState/LoadingState/ErrorState/MasterDetailLayout/StatusBadge
    ├─ component-gallery 的 ~60 shadcn 映射件（card/table/dialog/...）
    └─ Auto 生成 → Vue 库（package.json exports，build 出 dist）
        ↑ 由 a2vue "生成库"模式产出（新能力，见 §4）
        │
L2  应用业务组件库（各应用自维护，不上 npm）
    ├─ auto-musk: SpecSectionWidget/CrudEntityWidget（designs/001）
    └─ auto-forge 重构版: 同类业务件
        │
L3  单个应用（auto-musk / auto-forge / ...）
    └─ pages + 路由 + 业务逻辑
```

**库的文档站 = component-gallery 转正**：把 component-gallery 从"散落 demo"升级为"L1 库的官方文档站"（类似 shadcn-vue 官网），既展示又可复制用法。

---

## 4. 前置条件（启动前必须具备）

### 4.1 a2vue 关键缺陷修复（阻塞库 API 稳定性）

来自 auto-musk 实践（plans/002 §2.2），这几个不修，库的 API 会反复变：

1. **input 生成 `:value` 非 `v-model`**（输入框不可用）——库的表单类组件依赖此。
2. **无 DSL 层编程式路由跳转**——影响 Layout 类组件的导航交互。
3. **shadcn 组件不自动安装**——库化后应由库统一提供，不再各应用 add。
4. **outlet 易漏导致空白页**——生成器应在有 routes 时自动补。

### 4.2 【最关键新能力】a2vue "生成库"模式

**查证结论**：a2vue 当前**完全没有"生成库"的概念**（`crates/auto-man/src/vue.rs` + `crates/auto-lang/src/ui_gen/vue.rs` 全文搜 build.lib/library/exports/package.json name/publish = 零命中）。它只生成"应用骨架"（package.json 是应用型，dev/build/preview 脚本，无库的 exports/main/module 字段）。

**库要落地，a2vue 需新增**：
- **pac.at 的 `scene: "lib"`（或 `output: "library"`）** 区分库 vs 应用。
- **库型 package.json 生成**：`name`/`version`/`main`/`module`/`types`/`exports`/`files`/`peerDependencies`（vue/reka-ui 作 peer）、`scripts: { build: "vite build --lib ..." }`。
- **vite library mode 配置**（`build.lib` + rollup external）。
- **每个 widget → 独立导出**（tree-shaking 友好，`exports` map）。
- **可选**：widget 源（.at）作为库的"源码"也发布（让消费方 a2vue 工程能引用 .at 而非只 .vue）。

这是启动 L1 库的**决定性前置**——没有它，库无法由 Auto 生成。

### 4.3 多 render 目标的考量（关联 Plan 319）

L1 库当前只面向 `render: vue`。若未来要支持 vm/rust/jet/ark 等所有 render 目标（用户明确期望），库的组织需考虑：组件的 .at 源是 render 无关的，各 render 后端的"库适配层"分别生成。这与 Plan 319（unify-vm-rust-rendering）相关，建议协同设计，但**不阻塞 vue 版先落地**。

---

## 5. 建议路线（分阶段，低风险）

### 阶段 A（现在，零代码）：评估与对齐
- 本文档供 auto-lang 团队评估 L1 库的必要性与可行性。
- 对齐：是否认同 L1/L2/L3 分层、库的命名（`@auto-stack/ui`?）、归属仓库。
- **不立即建库**。

### 阶段 B（前置，a2vue 侧）：修缺陷 + 加"生成库"能力
- 修 §4.1 的 4 个 a2vue 缺陷（plans/002 §2.2 已记录）。
- 实现 §4.2 的 a2vue "生成库"模式（pac.at scene:lib + 库型 package.json + vite lib mode）。
- 这两项是 auto-lang 的工作，可独立于库的内容推进。

### 阶段 C（库 MVP）：最小可发布库
- a2vue "生成库"能力就绪后，先做**最小 L1 库**验证全链路：
  - 3 个状态态组件（EmptyState/LoadingState/ErrorState）+ component-gallery 的 card/badge/button 等基础件
  - Auto 生成 → Vue 库 → `npm publish`（或本地 file: 引用验证）
  - 在 auto-musk 实际 `import { EmptyState } from '@auto-stack/ui'` 验证消费
- 验证"Auto 生成 → 库 → npm → 消费"整条链路通后，再扩充。

### 阶段 D（扩充）：完整 L1 库 + 文档站
- 纳入 MasterDetailLayout/StatusBadge + component-gallery 全套组件。
- component-gallery 转正为库的文档站（展示 + 用法 + 可复制）。

---

## 6. 风险与反对意见

| 风险 | 缓解 |
|---|---|
| **库 API 在 a2vue 缺陷修复前不稳定** | 阶段 B 先于 C，不抢跑 |
| **"生成库"能力开发成本** | 这是 a2vue 的合理扩展（应用生成器→库生成器），且对 auto 生态长期价值高；可参考 vite library mode 标准做法 |
| **库 vs 应用边界模糊** | L1/L2/L3 分层明确；业务件坚决留应用层 |
| **维护负担** | L1 库内容有限（基础件 + shadcn 映射），不是大库；文档站复用 component-gallery |
| **shadcn-vue 已是成熟库，为何再造** | 不造重复件——L1 库的 shadcn 映射件是"Auto 语义封装"（.at 可用），价值在 Auto 生态内的一致性与 a2vue 集成，非替代 shadcn |

---

## 7. 决策清单（请 auto-lang 团队确认）

1. 是否认同"当前 AutoUI 缺通用组件库"的空白判断？
2. 是否认同 L1（npm 库）/ L2（应用业务件）/ L3（应用）三层划分？
3. 库的命名与归属？（建议 `@auto-stack/ui`，放 auto-lang 或独立仓库）
4. 是否同意"先修 a2vue 缺陷 + 加生成库能力，再启动库"（阶段 B → C）？
5. "生成库"能力是否纳入 a2vue 路线图？（这是决定性前置）

---

## 附录：证据索引

- **现状空白**：`examples/component-gallery/`（无 package.json）、`packages/auto-forge-ui/package.json`（private:true，未发布）、a2vue 无生成库能力（vue.rs 零命中 library/exports）
- **widget 泛化分析**：`auto-musk/designs/001-frontend-widget-design.md`
- **a2vue 缺陷**：`auto-musk/plans/002-frontend-framework-plan.md` §2.2
- **auto-forge 重复模式**：`auto-forge/docs/plans/023-fix-bare-fetch-auth-bugs.md`（R6）+ auto-musk 前端架构分析会话（R1-R5）
