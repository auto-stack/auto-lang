# Plan 435: AutoUI 组件统一声明 — schema 漂移治理与统一组件注册

> **状态**: 🚧 实施中 —— P0 已落地并复审通过(2026-08-24,分支 `plan-435-schema-unification`);P1-P4 待实施,P5 新增(见 §3.1 需求映射与补强);P0 分支尚未合并 master
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

- **P0 漂移围栏**(✅ 2026-08-24 落地,见文末执行结果):审计脚本 Rust 化为 drift test(进 auto-lang 测试套),比对 schema.rs ↔ vue.rs 映射 ↔ view_builder 两表 ↔ render_support,任一方孤立项即红。**先于一切统一工作落地。**
- **P1 基准提取与 schema 重建**(✅ 2026-08-24 落地,见文末执行结果):提取工具(一次性)从生产代码生成新 `schema/aura.at`;schema.rs 交叉核对;widgets-gallery 全部 tag 必须有声明;P0 测试在"新 schema vs 生产代码"维度转绿。
- **P2 schema 接入与校验**:loader 扩展四字段 + include_str! 内嵌 + AuraSchema 切换数据源;`auto build`/`auto ui inspect`/LSP 接入 schema 驱动告警与补全。
- **P3 派生翻转(行为零变更)**:render_support、vue.rs import 映射、别名归一改为 schema 驱动;**golden 验收:widgets-gallery vue 输出与翻转前 byte-identical**(`pnpm test:smoke` + `cargo test -p auto-lang -- ui_snapshots`)。
- **P4 统一注册表与第三方**:ComponentRegistry(source 判别)+ 解析优先级显式化 + `use` 包源解析 + packages/widgets 生成链打通第三方组件。
- **P5 文档与 Demo 系统(2026-08-24 复审新增)**:以 widgets-gallery 为唯一载体扩展,不另做一套——Properties/Installation 段从 schema 生成(`auto docs gen`),叙述段保留手写;每组件一张 schema 驱动的 kitchen-sink demo 页接入 playwright 视觉回归;VitePress 加 schema 生成的静态 API 参考(`website/docs/components/*.md`,tier/backends 徽章 + 链接部署版 gallery 活 demo);**文档覆盖围栏**:每个注册组件(内置/Local/Package)必须有 gallery 页面,新增未文档化组件即红(与 P0 围栏同哲学)。

## 3.1 需求映射(2026-08-24 复审,目标形态三层声明体系)

> 需求原文:**基础组件用 schema;官方实现的组件用 auto 代码;第三方库里的 auto 组件用统一方法注册声明——所有组件注册进 widgets 库,并有对应文档和 demo 页(widgets-gallery)。**

| 需求层 | 声明方式 | 对应阶段 | 状态 |
|---|---|---|---|
| 基础组件(native_html + builtin_widget) | schema/aura.at 声明契约,Rust 留行为 | P1-P3 | 覆盖 |
| 官方组件(shadcn 家族 + gallery 组件) | `.at` widget 源码即为声明(AuraWidget) | P4(Local/Package)+ 补强①② | 需补强 |
| 第三方 auto 组件 | `use` 包源引用 + 包 manifest | P4 + 补强③ | 需细化 |
| 文档 + demo 页面 | schema/registry 生成 + gallery 手写叙事 | **P5(新增)** | 原计划缺失 |

**补强项(P4/P5 开工前并入)**:

1. **组件家族声明**:carousel 家族(CarouselContent/Item/Previous/Next)目前只是命名惯例上的松散 `widget`,父子关系靠 vue.rs `known_sub_widgets` 隐式发现;ElementDef 只有 `allows_children: bool`,无 child 关系建模。schema 需加 `sub_widgets: [...]` 字段(或等价的 family 声明),官方/第三方复合组件同用。
2. **官方组件库定位**:明确官方组件库 = 一个 `.at` 包(候选:packages/widgets 从"生成物"升级为"源码库",gallery components 收编入内),它本身作为第三方注册机制的第一个消费者(自举验证)。
3. **第三方注册细节**:包 manifest 格式(入口 .at + 版本 + namespace)、`use` 解析顺序、shadow 规则(Plan 408 语义推广)、多后端一致性(至少 web+vm 两端冒烟)。
4. **shadcn 长尾战略归宿**:`tier: web_component` 的 245 项,写明迁移预期——渐进重写为官方 `.at` 组件(进家族声明+严格 props)或永久停留 codegen 映射(仅 tag+import);两者在 schema 中可区分,避免"永远漂在中间"。

**P0 复审补丁(半天量级,可与 P1 并行)**:围栏补 parser.rs tag 特判维度与 a2ui import/export 维度;vue.rs `components.insert` 补重复检测(现仅 schema.rs 有 `rs_duplicate_insert`,不对称);baseline 建议逐条加理由注释约定。

## 4. 验收

- **P0**:drift test 可复现本次审计的全部孤立项(红),后续任何新增组件不同步四表无法合入;同表重复 insert(如 popover 两处)直接编译期/测试期报错。
- **P1/P2**:四表数字对账(42/192/245/51 → 1 份 schema + 派生物);widgets-gallery 62 页全部 tag 在 schema 有声明;`auto build` 对故意写错的 tag/prop 给出 schema 驱动建议(LSP 同源)。
- **P3**:widgets-gallery vue 输出 golden 零回归;新增一个内置组件 = 改 schema + 一处实现(drift test 保证其余表同步)。
- **P4**:一个第三方组件通过"包声明 + use 引用"在 web 端可用(生成路径);内置组件不可被同名自定义组件 shadow(回归 Plan 408 语义);官方 `.at` 包通过同一机制注册成功(自举)。
- **P5**:全部注册组件(含第三方示例包)在 gallery 有文档页 + demo 页,文档覆盖围栏红→绿;button 等核心页的 Properties 表改为 schema 生成后与手写版逐字对拍一致;VitePress 组件参考页上线并与 gallery 活 demo 互链。

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

## 执行结果

### P0 漂移围栏(2026-08-24,worktree `plan-435-schema-unification`)

- **`crates/auto-lang/tests/schema_drift.rs`**:审计脚本 Rust 化,零新依赖。手写扫描器:
  HashMap insert 表扫描(map 名精确匹配,排除 `shadcn_components_used.insert` 前缀撞名)+
  `match tag` 派发表扫描(skeleton 剥离行注释与字符串内容后数括号深度,区分臂头/臂体,
  `let x = match tag` 小表与 `match tag_lower` 刻意不收)。覆盖:
  schema.rs `elements.insert`(192)/ vue.rs `components.insert`(245)+ map_tag 两表 /
  view_builder 两张派发表(52/49;审计 51/48 之后 Plan 442 双侧加 svg)/
  render_support(113),另带 aura.at(42)对照维度。共 14 个漂移维度 + 各文件表数量结构断言。
- **围栏语义 = 只拦新增漂移**:审计当日孤项冻结为 baseline
  (`tests/fixtures/schema_drift_baseline.txt`,980 行;`SCHEMA_DRIFT_UPDATE_BASELINE=1 cargo test -p auto-lang --test schema_drift` 再生成)。
  与验收原文"复现孤项(红)"的偏差是**有意决策**:常红测试会破坏其他 agent 的
  "全量绿"闸门;baseline 即孤项清单的可复核形态,围栏拦新增、P1-P3 收编时逐项裁剪。
- **同表重复 insert 无 baseline 豁免,直接红**;顺带清除 schema.rs popover 死块
  (L706 旧 Overlay 声明被 L2192 Plan 422 anchored popover 覆盖,HashMap 后写胜出,
  删除零行为变更)。
- **扫描器与审计 dump 交叉核对**:rs 集/at 集与 scratch/drift_*.txt 完全一致;
  vb 镜像差集 = 审计的 {menubar, popover, toolbar}。
- **负向验证**:render_support 注入假 tag → `[render_not_in_rs][render_not_in_vb]` 红;
  回注 popover 死块 → `[rs_duplicate_insert] popover` 红;清理后均恢复绿。
- lib 全量 3129 绿;围栏测试绿。**预存债(非本计划引入,已在 master 基线 c76011ec 复验,待另行归因)**:
  ui_snapshots 三例(app/editor/sidebar)红——snap 内嵌绝对路径(换 worktree 必红)
  + SFC 字节数漂移(app 2876→2920);vue_capabilities 一例 `cap_widget_map_model_init` 红
  (widget map init 语义漂移)。

### P1 基准提取与 schema 重建(2026-08-24,worktree `plan-435-schema-unification`)

- **生成器进围栏测试**(`SCHEMA_DRIFT_GENERATE_AT=1`,复核流程同 baseline):10 个提取源 ——
  9 张生产表(schema.rs 192 元素含结构化 props 解析/vue components 245+import 路径/
  vue map_tag 两表/view_builder 两表/render_support 113 含级别/parser/a2ui 双向)+
  **widgets-gallery 消费侧扫描**(第 10 源:gallery 实际使用而任何生产表未登记的 tag,
  按"生产代码 + examples 是事实源"以 unclassified 入册,如 dialog-* 全家、dropdown-menu-* 全名形态)。
- **新 `schema/aura.at`**:330 元素(旧 42),四新字段齐备 ——
  tier(builtin_widget 40/native_html 25/web_component 204/unclassified 60)、
  aliases(臂组+折叠变体)、backends(web/iced/gpui 矩阵,iced 级别来自 render_support)、
  sub_widgets(vue 同 import 路径 + 前缀规则,38 个家族,如 menubar→menubar_* )。
- **别名归并规则(迭代出三道守卫)**:①臂组只收渲染行为派发表(vb/vue)——parser
  get_primary_prop 是归类表(text 臂曾把 button 并进 alert 组)、a2ui 是序列化等价表
  ("select"|"dropdown" 臂曾把 select 吞进 dropdown);②臂内 ≥2 个 rs 元素不合并
  (text|h1|p|span 共用转换器≠别名);③臂 >4 成员不合并(svg 图元/HTML5 语义标签
  各自成元素)。折叠键 = 剥 `-`/`_` + 小写(Card≡card,AlertDialog≡alert-dialog);
  canonical 偏好:rs 声明 > kebab > 短。
- **覆盖断言转硬闸(无 baseline 豁免)**:生产表 tag ⊆ aura.at(tags∪aliases),
  新增组件必须重生成;反向幻影检查防手改漂移;旧 at↔rs 维度退役。
  schema_loader 可解析性作为常驻断言(P2 接线前置)。
- **验收**:widgets-gallery tag 覆盖缺口 0(折叠匹配策略下);围栏绿;loader 冒烟过。
- **发现与挂账**:
  1. **命名三套的实证**:gallery 写 kebab、生产表登记 underscore/concat、管线靠隐式
     归一弥合 —— schema 将归一显式化为**折叠匹配策略**(写入文件头,P2 校验器实现)。
  2. **隐藏机制第三形态**:vb 的 `is_svg_shape_tag` `matches!` 白名单(Plan 442)不在
     任何派发表,svg 图元 iced 支持级报 none(实为 convert_svg_image 支持)——谓词式
     知识入栏留 P2。
  3. `+` 是 map_tag 真实 DSL 简写(→span),作为元素入册。
  4. 60 个 unclassified 待 P2 人工归类(parser/a2ui-only 与隐式 fallback 家族)。
  5. **aura.at 并非全孤儿**:`load_default_schema()`(include_str! 内嵌)→ `WidgetValidator`
     是真实运行时链路,只是终点目前只有测试调用;P1 重生成后其单测 `test_new_elements`
     (断言 Plan 098 旧内容,如 textarea cols)红 —— 已按新 schema 事实重写(props 断言
     限 rs 支撑元素,其余存在性断言)。include_str! 接线机制已在,P2 只差消费方接入。
