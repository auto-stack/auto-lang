# Plan 435: AutoUI 组件统一声明 — schema 漂移治理与统一组件注册

> **状态**: 📋 已立项待实施(2026-08-23 漂移审计会话产出)
> **来源**: 2026-08-23 组件声明问题调查 + 漂移审计(脚本 `scratch/schema_drift_audit.py`,明细 `scratch/drift_*.txt`)
> **关联**: 098(aura.at schema 初版)/ 280(render_support)/ 320(WidgetRegistry.all)/ 361(validators)/ 408(路由别名不 shadow 内置)/ 412(widgets-gallery Layout 分组)/ 422(弹层语义,碰 view_builder)
> **基准原则**: **生产代码 + examples(widgets-gallery 62 页)是事实源**;`aura/schema.rs` 本身已漂移,仅作交叉参考,不作基准。

---

## 0. 一句话结论

把散落 8 处的内置组件定义收敛为「**一份 .at schema 声明 + 一个统一组件注册表**」:漂移先被 CI 拦截(P0),再被生成消除(P1-P3),内置/自定义/第三方组件最终走同一条解析链(P4)。

## 1. 现状盘点(审计事实,2026-08-23)

### 1.1 四张组件表,相互漂移

| 表 | 位置 | 数量 | 角色 |
|---|---|---|---|
| `schema/aura.at` | Plan 098 产物,此后冻结 | **42** | 声明式 schema,**孤儿**(schema_loader 无生产调用方) |
| `crates/auto-lang/src/aura/schema.rs` | 192 个 `elements.insert` | **192** | 验证声明,`WidgetValidator` 调用点全部在 `#[cfg(test)]` |
| `crates/auto-lang/src/ui_gen/vue.rs` | `components.insert` 映射 + `map_tag` 兜底 | **245** | **Web 路径事实清单** |
| `crates/auto-lang/src/ui/aura_view_builder.rs` | 两张 `match tag` 表(51/48) | **~51** | **桌面(iced)路径事实清单** |

- 桌面 ∩ Web 组件映射 = **14 个**;`.at` ∩ `.rs` = **21 个**(其中 5 个 props 漂移,如 `tabs`:at 声明 `value,onchange`,rs 声明 `defaultvalue`)。
- `class`→`style` prop 改名(04278d0b)只改了运行时层(view_builder 两者都读),schema.rs 仍 183 处 `class`,aura.at 全是 `class`。
- 别名归一(btn/column/codeEditor kebab/连写)散落 parser、view_builder、vue.rs 三处,命名规范三套(下划线/kebab/连写并存于 vue.rs 映射)。
- **实现比声明多**:`convert_button` 实际消费 `icon/label/variant`,两份 schema 均未声明。
- **同表内部也会静默冲突**:schema.rs 里 `popover` 被 insert 两次(L706:Overlay 类别,2 props;L2192:Plan 422 加的 Navigation 类别,6 props),HashMap 语义下后者覆盖前者,前一份声明是死代码——两个 plan 各自添加、无任何机制拦截。
- 组件分层(原生 HTML 直通 / 桌面内置 widget / shadcn Web 组件)由 vue.rs `map_tag` 解析顺序**隐式决定**(`known_sub_widgets > ext_components > shadcn 注册表 > HTML 兜底`),未在任何地方声明。

### 1.2 已有地基(可复用,不需重造)

- `aura/schema_loader.rs`:已能解析 aura.at 语法 → `ElementDef`,缺 aliases/tier/backends 字段。
- `ui/widget_registry.rs`:自定义 `AuraWidget` 的统一注册表,已处理"route module `button` 不得 shadow 内置"(Plan 408)——**自定义组件反而是有统一声明的**,本计划把内置组件拉平到同一模式。
- `packages/widgets`:从 AURA 定义生成 Vue 组件(shadcn 风格 + reka-ui),"从声明生成实现"链路已验证。
- `ui/render_support.rs`(Plan 280):per-tag 后端支持级别,手写待派生。

### 1.3 验收基准载体

`examples/widgets-gallery`:62 个组件页(`.at` 源码 → 生成 Vue + shadcn-vue,vue 端完全可用),覆盖 Layout/Form/Display/Feedback/Navigation/Overlay 全分组 + 自定义组件(carousel/combobox/command/datepicker 等 8 个 `.at` widget)。**P3 数据流翻转以它的输出 golden 零回归为硬验收。**

## 2. 方案要点

### 2.1 统一声明源:重建 `schema/aura.at`

- **内容基准 = 生产代码提取**,不以 schema.rs 为准:从 vue.rs 映射(245)、view_builder 两表(51)、parser 特判、render_support、a2ui import/export 提取 tag/props/别名/层级,交叉核对 schema.rs,以 widgets-gallery 实际用到的组件为优先核对清单。
- schema 格式在现有 `element` 块基础上扩展四个字段:
  1. `aliases: ["btn", ...]` —— 拼写变体统一声明,归一逻辑收进 schema;
  2. `tier: "native_html" | "builtin_widget" | "web_component"` —— 把 map_tag 隐式解析顺序显式化(native_html=直通层,类似 Vue 的 HTML_TAGS;builtin_widget=桌面有实现;web_component=shadcn 家族);
  3. `backends: { iced: "full|partial|fallback", web: "native|component|none", gpui: ... }` —— 支持矩阵,替代手写 render_support;
  4. `deprecated: [...]` —— `class`→`style` 这类历史 prop 迁移标记。
- 命名规范:**canonical = kebab-case**(与 HTML/shadcn 对齐),下划线/连写/PascalCase 一律进 aliases。

### 2.2 数据流翻转(schema 管契约,Rust 管行为)

- `schema_loader` 扩展新字段;`include_str!` 编译期内嵌(避免运行时文件依赖);`AuraSchema` 改为从 .at 构建。
- 以下数据面从 schema 派生,**不改任何行为代码**:
  - `render_support.rs` 的 TagSupport 表 ← `backends` 字段;
  - vue.rs 的 `components.insert` import 映射 ← `tier: web_component` 条目;
  - parser/view_builder/vue.rs 三处别名归一 ← `aliases` 字段(归一为单个 `normalize_tag()` 入口);
  - `WidgetValidator` 的 ElementDef 数据 ← schema 本体。
- convert 函数、渲染 match、事件语义留在 Rust —— schema 不描述行为。

### 2.3 校验接入(第一个用户可见收益)

- `WidgetValidator` 接进 `auto build` / `auto ui inspect`:未知 tag 给 schema 驱动的拼写建议(levenshtein 已有),未知 prop 给 per-tag 告警(现在 `auto build` 的告警来自 ui_gen/validators.rs,不查 tag)。
- LSP 补全/hover 从同一 schema 取数(aura.at 头注释当年声称的用途,这次真正接上)。

### 2.4 统一组件之家 + 第三方注册

- 显式化 `ComponentRegistry`,条目带 `source: Builtin | Local | Package` 判别:
  - `Builtin` ← schema/aura.at;
  - `Local` ← 项目内 `use` 导入(现 WidgetRegistry 路径,平移);
  - `Package` ← 第三方:`use` 解析从本地路径扩展到包名/registry 引用,Web 端复用 packages/widgets 的"声明 → 生成 Vue 组件"链路。
- 解析优先级由 tier/source 声明决定,内置不被 shadow 的规则(Plan 408)推广到全部层级。

## 3. Phases

- **P0 漂移围栏**:审计脚本 Rust 化为 drift test(进 auto-lang 测试套),比对 schema.rs ↔ vue.rs 映射 ↔ view_builder 两表 ↔ render_support,任一方孤立项即红。**先于一切统一工作落地。**
- **P1 基准提取与 schema 重建**:提取工具(一次性)从生产代码生成新 `schema/aura.at`;schema.rs 交叉核对;widgets-gallery 全部 tag 必须有声明;P0 测试在"新 schema vs 生产代码"维度转绿。
- **P2 schema 接入与校验**:loader 扩展四字段 + include_str! 内嵌 + AuraSchema 切换数据源;`auto build`/`auto ui inspect`/LSP 接入 schema 驱动告警与补全。
- **P3 派生翻转(行为零变更)**:render_support、vue.rs import 映射、别名归一改为 schema 驱动;**golden 验收:widgets-gallery vue 输出与翻转前 byte-identical**(`pnpm test:smoke` + `cargo test -p auto-lang -- ui_snapshots`)。
- **P4 统一注册表与第三方**:ComponentRegistry(source 判别)+ 解析优先级显式化 + `use` 包源解析 + packages/widgets 生成链打通第三方组件。

## 4. 验收

- **P0**:drift test 可复现本次审计的全部孤立项(红),后续任何新增组件不同步四表无法合入;同表重复 insert(如 popover 两处)直接编译期/测试期报错。
- **P1/P2**:四表数字对账(42/192/245/51 → 1 份 schema + 派生物);widgets-gallery 62 页全部 tag 在 schema 有声明;`auto build` 对故意写错的 tag/prop 给出 schema 驱动建议(LSP 同源)。
- **P3**:widgets-gallery vue 输出 golden 零回归;新增一个内置组件 = 改 schema + 一处实现(drift test 保证其余表同步)。
- **P4**:一个第三方组件通过"包声明 + use 引用"在 web 端可用(生成路径);内置组件不可被同名自定义组件 shadow(回归 Plan 408 语义)。

## 5. 风险

- **245 长尾迁移工作量**:分级控制——核心集(native_html 直通层 + 桌面 51 + 基础 form/display)进严格 schema(props 全声明);shadcn 长尾先按 `tier: web_component` 只声明 tag+import 映射,props 细节后续补。
- **vue.rs 2.1 万行生成器边界**:只翻转数据(import 映射/别名),不动行为;golden 对拍兜底。
- **parser 语法特判不能完全 schema 化**(button 文本简写、grid 的 row 语法等):保留 Rust 白名单,schema 加 `syntax: special` 标记位指路。
- **多后端支持矩阵维护成本**:backends 字段允许 `unknown` 缺省,gpui/cosmic 后端渐进补齐。
- **并行 plan 冲突**:422(弹层)/423(action_config)都碰 view_builder;P3 开工前同步,drift test 会拦住并行漂移。
- **schema 双源过渡期**:P2 完成AuraSchema 切换前,schema.rs 保留为 fallback,P0 测试盯住两源不一致。

---

## 附:审计产物

- 脚本:`scratch/schema_drift_audit.py`(P0 的 Rust 化蓝本)
- 明细:`scratch/drift_at.txt`(42)/ `scratch/drift_rs.txt`(192)/ `scratch/drift_vb0.txt`(51)/ `scratch/drift_vb1.txt`(48)
- 关键代码位置:vue.rs `map_tag`(L5486,解析顺序)/ view_builder 两表(L873 tracked / L1657 untracked)/ schema_loader(孤儿)/ widget_registry(自定义组件地基)
