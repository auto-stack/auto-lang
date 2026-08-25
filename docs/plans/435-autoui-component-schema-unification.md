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
- **P2 schema 接入与校验**(✅ 2026-08-24 落地,见文末执行结果):loader 扩展四字段 + include_str! 内嵌 + AuraSchema 切换数据源;`auto build`/`auto ui inspect`/LSP 接入 schema 驱动告警与补全。
- **P3 派生翻转**(🟡 2026-08-24 部分落地,见文末执行结果;vue import 翻转经第五表发现重定向 P4):render_support 已 schema 驱动;确定性修复 + golden 闸门建立;**golden 验收:widgets-gallery vue 输出 byte-identical(跨进程确定性从无到有)**。
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

### P2 schema 接入与校验(2026-08-24,worktree `plan-435-schema-unification`)

- **数据模型**:`ElementTier`/`BackendMatrix`/`ElementMeta` 落 schema.rs;`AuraSchema`
  增 `meta` 侧表(硬编码 fallback 无 meta,loader 填充)——不动 192 个构造点。
- **loader 解析四字段**:tier/aliases(引号列表)/backends(三元组)/sub_widgets;
  `resolve_tag()` 三级匹配(精确 → 声明别名 → 折叠键)。
- **数据源切换**:WidgetValidator 已走 load_default_schema(include_str!);P2 补
  加载失败回落硬编码 fallback + 日志,不因 schema 损坏阻塞构建。
- **校验接线**:`validate_aura_against_schema()` 进 ui_gen/validators ——
  未知 tag → S002 Warning(levenshtein 建议,本地 widget/子件/ext 组件名折叠豁免);
  已声明 props 元素上的未声明 prop → S001 Info(通用 prop 豁免:class/style/id/key/
  on*/​*-if)。挂进 generate_component_from_file:`auto ui inspect` 与 `auto build`
  同源展示,--strict 下 Warning 阻断。
- **LSP**:completion 增 UiElement 上下文(view 块内元素位),330 元素按
  tier(builtin→native→web→unclassified)排序,detail 带 tier/category/描述。
- **unclassified 60→17**:TIER_OVERRIDES 归类表(生成器内,可再生)——
  builtin_widget 6(nav-link/toast-provider=list 等,派发表外隐藏机制或 VNodeKind 词汇)、
  native_html 8(a/audio/canvas/tfoot/li/summary/code/video)、web_component 29
  (dialog-*/dropdown-menu-*/tooltip-*/avatar-*/skeleton/navigation-menu 等 shadcn 家族);
  余 17 为待定词汇(a2ui concat 变体与 gallery 应用域 tag),保留 unclassified 比错分更诚实。
- **验收(实测)**:gallery button.at 零告警;故意写错(btton/tex)→
  S002"did you mean button?" + S001 列已声明 props;lib 3131 绿(+2 测试);LSP 17 测试绿。
- **顺带修正**:button 的 variant/size/icon 自此声明(实现与 gallery 实际消费,
  P1 审计"实现比声明多"的实证收口)。
- **发现挂账**:view 顶层裸兄弟(UI scenario)解析失败(单词/连字符 tag 皆然,
  嵌套 col 内正常)——gallery 全嵌套所以无感;疑与 plan-015 "顶层裸兄弟修复"
  相关,待单独归因(不影响本计划)。

### P3 派生翻转(2026-08-24,worktree `plan-435-schema-unification`;render_support ✅ / vue 映射重定向 P4)

- **前置:生成器确定性修复(根因级)**。Vue SFC 发射层 19 处 HashMap 迭代
  (props/events/handlers)+ used_handlers(HashSet)桩函数序跨进程不确定
  (RandomState)——**即 ui_snapshots 预存债"SFC 字节数漂移"的根因**。
  `sorted_entries()` 统一排序后跨进程 diff=0;golden byte 基线自此才可能成立。
- **golden 闸门**:`tests/gallery_golden.rs` —— gallery 全部 71 个 .at 逐文件
  生成 SFC,长度+稳定哈希入 `tests/fixtures/gallery_vue_golden.txt`
  (GALLERY_GOLDEN_UPDATE=1 再生成,人工复核;TOTAL 行盯全文)。
- **render_support 翻转 ✅**:`get_support` 级别改为 schema 权威
  (default_schema_cached OnceLock 缓存,resolve_tag 三级折叠解析);
  静态表降级为详情来源(ignored_props/note 不在 schema)+ 回退。
  围栏新增一致性硬断言:静态级别 ≡ schema backends.iced(逐 tag)。
- **顺带修真 bug**:render_support 存在**同表遮蔽死臂**(L160 partial 先匹配生效,
  L238 fallback 永不可达,code/codeblock 级别交叉)——删除死臂;扫描器改
  first-wins 对齐 Rust match 语义;围栏加同表遮蔽臂检测。
- **第五表发现(审计勘误)**:`ui_gen/widget/registry.rs`(3566 行,181 个 vue
  BackendMapping + ark/jet 后端)才是**活的**组件注册表;vue.rs 的
  `components.insert`(ShadcnRegistry)已 deprecated 仅测试引用——P0 审计的
  "vue.rs 245"部分是死表。围栏立即抓到 **78 个此前完全不可见的组件**
  (area/bar/line/donut-chart 全家、scroll-area 全家、toast-* 家族、data-table、
  markdown、mermaid、swiper、calendar 部件、navigation-menu 部件等)——
  已作为**第 11 提取源**入册(schema 330→368),围栏加 spec 覆盖 +
  vue 映射数下限断言。
- **vue import 映射翻转 → 重定向 P4**:翻转对象更正为 registry.rs;其
  BackendMapping 含 prop/event 重写与 extra_components,完整翻转实质是
  ComponentRegistry 收敛工程(与 deprecated ShadcnRegistry 清退同期),归入 P4。
- **别名归一**:resolve_tag 折叠匹配已在校验(S001/S002)与 render_support 生效;
  parser/vb 的归一入口统一(normalize_tag)记 P4 顺手项——行为等价已由
  golden 保护,统一收益是入口唯一。
- **验证**:gallery golden 绿(byte 级)、lib 3131 绿、围栏绿(含新维度)。

### P3 续:与 master 并行确定性双修的合流(2026-08-24)

- **双修合流**:master 侧 plan-015(44afea19/690abfc2/831c9ec3)与本计划独立修复
  同一非确定性——master 走源头改造(widget.handlers → BTreeMap)+ 内联排序,
  本分支走发射层 sorted_entries;合并时 13 处冲突取 master 完整结构,
  props/events 的 HashMap 发射排序保留 sorted_entries。两法互补,合流后跨进程
  diff=0 复验通过。
- **ui_snapshots 三例预存债清偿**:字节漂移根因(非确定性)根治 + plan-443
  defineModel 降级收窄的合法输出变化,快照重接受后 3/3 绿。
- **golden 跨 worktree 可移植**:输出内嵌绝对路径有两种形态(canonical 与
  crates/x/../../ 拼接),分别归一为 <ROOT>/<CRATE>;主仓与 worktree 双绿。
- **S002 假阳性修复**:PascalCase tag = 组件引用语义(内置 tag 全小写),
  跳过告警——单文件模式下跨文件子组件(NavTree/EditorPanel)不再误报。
- **挂账(待单独归因)**:① ~~view 顶层裸兄弟(UI scenario)解析失败~~ → **已销账:master plan-015(831c9ec3)已修复**,master 实测三兄弟含连字符 tag 解析通过;
  ② combobox/command/datepicker/toggle/togglegroup 五个 gallery components 在
  generate_component_from_file 路径有预存解析错误(UnexpectedToken RParen ':',
  页面生成不受影响);③ golden 基线随 plan-443 重采样一次(defineModel 收窄,
  626 行合法变化)。

### 挂账②清偿:gallery 5 组件解析失败(2026-08-25)

- **五层根因逐层揭开**(每修一层露出下一层,全部根治):
  1. **msg 变体命名参数** `Change(value: str)`:parser 支持(名字入新增的
     `MsgVariant.payload_names`/`AuraMsgVariant.payload_names`,位置形态 None;
     为 P4 组件包 API 文档面预留);
  2. **管道文本简写** `tag | bare text`:`text | Slide 1` 此前靠垃圾解析碰巧
     能过(|/Slide/1 成为垃圾子元素),多词带句点(`ComboboxEmpty | No framework
     found.`)直接报错 —— 现按语义实现(VBar 后读整行,挂 primary prop 或
     text 子节点);
  3. **on 块无点前缀带参 handler** `Change(pressed) ->`:参数收集与点前缀一致
     (抽 `parse_handler_params` 共用),pattern 保持无点兼容旧语法;
  4. **连写块** `div { style: "..." } { children }`:children/props 块 if→while;
  5. **model channel 非可写值硬错 → 降级**(Plan 443 哲学补全):字面量/表达式
     喂通道降级为单向 :prop 绑定(记录 emitted_model_bindings 使子件侧仍编译
     defineModel,值经 prop 流入)+ R016 Info 告警;旧严格性测试改断言降级。
- **验证**:5 组件零错误生成;combobox/015-notes 页面零告警;golden 重采样
  (5 文件错误文本→真实 SFC);lib 3140 绿(+2 测试);围栏绿;ui_snapshots 3/3。
- **并发合流第二例**:master 在本修复合并期间又进 plan-444(emits 收敛:
  defineEmits 带全部 payload 类型),无冲突并入;golden 基线随之重采样
  (`Change: [string]` 形态)。多会话并发下的基线采样必须在**合并后**的主仓做。
- **操作教训**:Windows 增量构建指纹在高频提交下偶发陈旧(cargo 未重编),
  双树输出不一致时先 touch 强制重建再归因;本节排查曾因此绕路。

### P4 核心:统一注册表 + 官方包自举 + 包引用(2026-08-25,P4-1/2/3 落地;P4-4/5 续)

- **P4-1 ComponentRegistry**(`ui_gen/widget/component_registry.rs`):
  `ComponentSource{Builtin,Local,Package}` + `resolve()` 优先级显式化
  (**Builtin > Local > Package**,Plan 408 语义推广);内置折叠名冲突的注册被拒
  并记 `ShadowViolation`(api 层 S004 Info 告警——Warning 会误伤既有合法
  shadow 如 a2vue 语料的 Card widget,规则在解析层强制,告警咨询化)。
- **P4-2 官方包自举**:gallery components 目录 + `package.at` 清单
  (pac.at 同款 key:value 约定,不发明新语法)= 第一个 .at 组件包
  "official@0.1.0";经与第三方**完全相同**的 load_package 机制注册(零特例)。
- **P4-3 包引用语法与接线**:`use { package: official from "dir" }`
  (ExtImportKind::Package);api 主链路加载包 → 组件名并入 sub_widgets
  (tag 走组件生成路径);加载失败 S003 告警不阻塞。
- **折叠桥接**(map_tag 两处 + 元素路径同源):kebab tag(copy-button)↔
  Pascal widget 名(CopyButton),返回规范名保证 import/文件名大小写正确。
  两道守卫:①内置可解析的 tag 不桥接(builtin 优先);②**命名空间形状守卫**
  —— 裸小写单词(pre/code/div)属元素命名空间不桥接(否则 `pre {}` 变成
  自引用 `<Pre/>`,实测抓到);kebab/含大写才是组件形态。
- **发现:官方包组件名与 rs shadcn 家族折叠冲突**(carousel-content ↔ 内置
  carousel_content)——builtin-first 下包组件被内置声明占用,正是 §3.1 补强④
  "shadcn 长尾归宿"要解决的:组件改写为官方 .at 时应同步退役 schema 的
  web_component 条目(P4-4/P5 期间逐族处理)。
- **验收(测试 4/4)**:优先级链、shadow 拒绝(Button/AlertDialog)、
  官方包自举(零特例)、e2e(use package + copy-button → SFC 引用 CopyButton)。
  golden 重采样(唯一差异 = package.at 清单跳过);lib 3140;围栏绿。
- **挂账**:① 全量并行下偶发 flaky 两例(单跑均绿,共享状态/顺序敏感,
  待单独归因):ring_caps(ui_console)+ ffi_dual_014_std_generated_segment
  (plan-430 系);② P4-4(registry.rs 181 vue 映射 schema 派生)与
  P4-5(ShadcnRegistry 清退 + unclassified 17 清零)未动;③ 并发会话持续
  在 master 落 gallery 页(line-chart)——golden 基线已并入(72 文件)。

### P4-4/P4-5(部分):registry vue 映射 schema 派生 + unclassified 清零(2026-08-25)

- **P4-4 完成**:registry.rs 的 181 个手写 `backends.insert("vue")` 退役(-1389 行)。
  数据链:生成器从 registry.rs 提取 → schema/aura.at 的 `vue: { component/import/
  extras/npm }` 声明(191 条)→ loader 解析进 `ElementMeta.vue` →
  `apply_schema_vue_mappings` overlay 重建(resolve_tag 三级折叠匹配)。
  实测 vue 后端 181 映射中 props/events 重写为 0(ark/jet 才有),派生面干净。
- **golden byte-identical 零回归** —— 派生翻转的核心验收达成。
- **carried_vue 保留机制**:手写退役后再生成,存量 vue 行从当前 aura.at 保留
  (防数据丢失);围栏三重守护:手写数==0 + schema vue 量>=160 守恒 + 迁移期对账。
- **排障实录(挂账为操作教训)**:① 删除脚本的 needle 撞名把 overlay 自身函数体
  吃掉 —— 修复:函数区间守卫删除;② carried 解析 needle 缺引号曾把 191 行写成
  空(component: "")—— git 恢复 + needle 修正;③ 安全点提交时 overlay 已被早前
  checkout 洗掉(手写数据掩盖了缺失)—— 分相位提交纪律。
- **P4-5(部分)**:unclassified 17→15 —— grid-item/stack 归 web_component
  (registry vue 映射可得);余 15 个无任何实现数据(parser/a2ui 词汇),
  诚实保留待定(错分比待定更糟)。
- **验证**:golden byte-identical + P4 4/4 + 围栏绿 + lib 3160 全绿。
- **待续(P4-5a)**:deprecated ShadcnRegistry 清退 —— vue.rs 245 条
  components.insert + ~6 测试改用 WidgetRegistry/schema;围栏 vue.components
  维度与 vue_imports 提取源同步改造(改从 schema vue 行取,carried 同源)。

### P4-5a:ShadcnRegistry 清退(2026-08-25,死表终结)

- **删除**:vue.rs 的 deprecated `ShadcnRegistry`(struct + 245 条 components.insert
  map + Default impl,共 -648 行);五个 phase 测试重写为**活链路清单**
  (schema/aura.at → overlay → WidgetRegistry:get_primary_component/backend API)。
- **死数据显形**:死表 245 条中 ~64 条是活链路从未使用的死映射
  (tab→TabsTrigger、divider→Separator、thead 家族、radio、spinner...)——
  负断言存档(如 thead 家族按 Plan 408 P8 刻意保持原生 HTML)。
- **围栏源改造**:vue 成员资格与 import 路径改从当前 aura.at 的 vue 行
  (carried_vue)取;vue↔rs 交叉维度退役(baseline -277 行,两表已统一于
  schema);幻影检查归约到 canonical 判 provenance(别名随 canonical 许可)。
- **carried 元素保留**:死表独有元素(combobox_anchor/select-separator 家族件等
  10 个)经 carried_elements 保留,再生成不丢(描述标注 retired dead-table entry)。
- **发现修复**:loader 的 extras 解析首元素被 ` [` 前缀卡掉(bracket 剥离),
  chart 家族 ChartTooltip 曾静默丢失。
- **验证**:5 测试重写后绿;golden byte 稳定;P4 4/4;围栏绿;lib 3162 全绿。
- **组件注册终态**:生产表三处(view_builder 两表 + vue map_tag 兜底表)+ registry
  spec 名 + gallery 消费侧 + **schema 自身(carried)** —— 新增组件唯一入口 =
  schema/aura.at(手写或经 registry spec 提取),运行时全部经 schema 重建。

### P4-5a 合流注记(2026-08-25)

- **跨会话挂账**:master 上 `native_catalog_ids_and_names_unique` 确定性红
  (catalog ID 2780 撞号,auto.rand.int 与既有条目重复)—— 来自并发
  plan-442/musk 会话的 native 注册,与本计划 UI 链路无关,待该会话修复。
- P4-5a 全闸门绿:golden 稳定 + P4 4/4 + 围栏绿 + lib 3161/3162
  (除上述并发红)。

### P5(主体):schema 驱动文档系统 —— 生成器 + 三道围栏 + VitePress(2026-08-25)

- **P5-1 核心参考生成器**(`tests/docs_gen.rs`):tier ∈ {builtin_widget,
  native_html} 的元素生成 `docs/components/core.md`(tier/后端徽章 + props 表 +
  别名/家族;`DOCS_GEN_UPDATE=1` 再生成);同步围栏保证与 schema 一致。
  shadcn 家族(web_component)的活文档继续由 gallery 页承载 —— 分层与 tier 语义对齐。
- **P5-2 文档覆盖围栏**:主组件(非 subwidget、tier ∈ {builtin,web})必须有
  gallery 页;三层豁免 = 页面 ∪ 白名单(他处文档化,24 项:语义 HTML 在 Layout
  组、form 家族件在 form 页等)∪ 基线(文档债冻结,34 项:autodown/chart 全家/
  modal/toolbar/list/spacer...)。**新增未文档化组件即红** —— 与 P0 围栏同哲学。
- **P5-3 Properties 对拍一致性**:gallery 页手写 Properties 表不得与 schema 矛盾
  (手写 prop 必须 ∈ schema 声明;空 props 元素跳过)。实测通过 —— button 等
  页与 schema 一致(P2 补声明的 variant/size 直接吃到红利)。
- **P5-4 VitePress 接线**:prepare-content 的 DOCS_INCLUDE 增 `components`;
  core.md 自动流入 website 并生成侧栏条目(产物 gitignored,源在 docs/)。
- **验证**:docs_gen 3/3 + 围栏/golden/P4 全绿 + lib 3162。
- **待续(P5b)**:kitchen-sink demo 页(schema 驱动全 props 展示 + playwright
  视觉回归);gallery 空 props 元素的 schema 回填(文档表是现成数据源,反向生成)。

### P5b:尾巴 —— gallery props 回填 + kitchen-sink demo + loader 类型修复(2026-08-25)

- **gallery props 回填(P5b-1)**:生成器新源 `scan_gallery_props()` 从 gallery 页
  手写 Properties 表(td 五列:Property/Type/Default/Values/Description)提取,
  为 rs 未声明的空 props 元素回填(当前 15 元素/34 props;类型归一
  boolean→bool/str→string,Values 列→one_of,深度计数的表解析器防 thead 嵌套
  误断)。**props 从此有三个来源的优先级:rs > gallery 回填 > 空**。
- **kitchen-sink demo 页(P5b-2)**:`generate_kitchen_sink()` 生成 gallery
  `pages/kitchen-sink.at`(23 个 builtin_widget 含字面化 props 的元素,每元素
  默认 + 每取值一变体至多 3;one_of 拆分为独立变体,bool 不加引号);
  同步围栏 `kitchen_sink_page_in_sync`。golden 基线重采样(75→76 文件)。
- **loader 类型解析修复(P5b 排障副产物)**:
  1. `split_by_comma` 不感知引号 → `type: "one_of:a,b,c"` 在首个逗号被腰斩,
     one_of/union 静默降级为 string(description 含逗号也丢尾)→ 加 `in_str`
     状态跟踪;
  2. `extract_value(part, "type:")` 同样在逗号截断 → 改 `extract_string_value`
     优先(引号内不截断);
  3. **`Q` 绑定模式陷阱**:Rust `match ch { Q => ... }` 中裸 `Q` 是变量绑定
     (匹配一切),不是字符字面量 —— 状态机失灵且不报错;改 `'\"'` 字面量。
  三修后 `variant` 正确返回 `OneOf(["default","secondary",...])`,
  core.md/kitchen-sink 同步产出正确类型。
- **验证**:docs_gen 4/4 + golden(76 文件) + 围栏 + P4 4/4 + lib 3172 全绿。

## 6. 全量复审发现的缺陷与修复 Phases(2026-08-25 审查产出)

> 三路独立探针审查(围栏/数据质量、实现质量、确定性/安全)汇总,
> 按严重程度分级为 P6(紧急)/P7(高优)/P8(中优)三个修复阶段。

### 6.1 缺陷总览

| # | 严重度 | 缺陷 | 根因 |
|---|---|---|---|
| D1 | 🔴 | vue 维度围栏自指——carried_vue 从当前 aura.at 读回,vue 映射可静默漂移无独立校验 | P4-5a 删死表后无替代交叉源 |
| D2 | 🔴 | api_functions_used(HashSet)未排序,≥2 个 API 函数的 SFC 字节不确定 | P3 确定性修复遗漏此点 |
| D3 | 🔴 | render_support 翻转缝隙:新 schema tag 有 iced 级别但静态详情表无臂 → Full level + "unknown tag" note 自相矛盾;反向围栏缺失 | P3 翻转只查静态→schema 方向 |
| D4 | 🟡 | builtin-first 封锁 gallery 自己的 Carousel 全家(5 个子组件折叠名冲突),carousel 页渲染纯 div | 补强④(shadcn 条目退役)未执行 |
| D5 | 🟡 | normalize_tag 统一入口(P4 顺手项)完全未做;vue.rs 和 jet/generator.rs 两份私有实现互相不一致 | P4 跳过了 |
| D6 | 🟡 | ComponentRegistry 的 LoadedPackage 纯平 HashMap,无 sub_widgets 家族建模 | 补强①只做了 schema 侧 |
| D7 | 🟡 | load_package 一坏全坏:单文件解析失败丢弃整个包(包括合法组件) | 无 reject-and-continue |
| D8 | 🟡 | LSP UiElement 补全位置盲:全文 contains("view {") 子串匹配,非 view 块内裸标识符也触发 | 启发式非结构化 |
| D9 | 🟡 | kitchen-sink 页生成但未路由;codeeditor canonical 拼写命中不了 vue.rs 特判(渲染 div);progress "0-100" 伪枚举 | 路由/拼写对齐/范围编码三个子问题 |
| D10 | 🟠 | 44% web_component / 41% builtin_widget 元素零 props 声明;gallery 回填仅 15 元素/34 props | 回填覆盖不足 |
| D11 | 🟠 | unclassified 103 个(不降反升:P2 时 17 → registry/gallery 源灌入后 103) | P4 提取源无分类步骤 |
| D12 | 🟠 | DOC_TODO_BASELINE 33 条冻结不裁剪,≥4 条已有 gallery 页面 | prune 纯 println 不失败 |
| D13 | 🟠 | 计划承诺未交付清单(见 §6.4) | 各阶段部分降级 |

### 6.2 Phases

#### P6:紧急修复(半天)

- **P6-1 确定性漏洞**(D2):`vue.rs:1783` `api_functions_used` 加 `.sort()`
  (同 L1622 lucide_icons 模式);删死代码 `sorted_entries` 助手(L137,零调用点,
  doc 声称"统一经此"为假——更新注释指明各调用点内联排序的约定)。
- **P6-2 vue 围栏去自指**(D1):为 schema 的 vue 映射引入独立交叉校验源。
  方案(择一,推荐 a):
  a. **import 路径文件存在性检查**:gallery 生成 SFC 后,提取全部
     `import ... from '@/components/ui/...'` 行,对照 `packages/widgets/registry/`
     + gallery gen 目录的文件存在性(golden 测试内即可做,不需要新基建);
  b. gallery golden 的 import 集合快照:当前所有 import path 冻结为一个
     集合,新增/删除 import 路径必须显式更新。
  同时:P4-4 断言(scan_registry_vue ≡ schema)已空转,改为"schema vue 行的
  component 名必须出现在对应 import 路径的包导出列表中"(packages/widgets
  的 registry JSON 已有);失败消息清理过时的 `rs/vue_duplicate_insert` 维度名。
- **P6-3 render_support 反向围栏**(D3):schema_drift.rs 增加反向断言——
  schema 中每个有 `backends.iced != "unknown"` 的元素,其 resolve_tag canonical
  必须在静态详情表有臂;否则红("新增 schema 元素须同步 render_support 详情")。
  同时:get_support 的 note 在 schema overlay 生效时,如原 note 以 "unknown tag"
  开头则清空(避免 Full + unknown 自相矛盾)。

#### P7:高优修复(1-2 天)

- **P7-1 shadcn 家族退役 / shadow 白名单**(D4):
  对 gallery 已用 .at 组件实现的 shadcn 家族(carousel 全家 5 个),从
  schema/aura.at 退役其 web_component 条目(或标记 `retired_by: "official"`),
  使 builtin-first 不再与官方 .at 组件冲突。**退役 = 在生成器的
  TIER_OVERRIDES 或 carried_elements 逻辑中加排除,再生成时不再发射**。
  验收:gallery carousel 页生成的 SFC 引用 CarouselContent 组件(非 div),
  golden 更新。同时处理 progress "0-100" 伪枚举:schema 生成器对
  gallery 回填的 Values 列做合理性过滤(含 `-` 或纯数字范围的跳过 one_of)。
- **P7-2 kitchen-sink 路由 + 拼写对齐**(D9):
  a. `app.at` 加 kitchen-sink 路由(或 layout 分组),golden 重采样;
  b. `codeeditor` 的 canonical 统一为 `code_editor`(修 schema 生成器的
     canonical 选择逻辑:别名组内有下划线形态时优先下划线,与 vue.rs
     map_tag 的 `"code_editor" | "codeEditor"` 特判对齐);
  c. kitchen-sink 生成器对非 literal props 的元素跳过变体发射(codeeditor
     的 key/content 在真实组件上不存在,目前靠 schema 正确但实际渲染 div)。
- **P7-3 load_package 容错**(D7):`load_package` 逐文件 try-parse,失败的
  文件记 warning 收集,不丢弃整个包(与 register_local 的 rejected Vec 模式
  对齐);S003 告警列出具体失败文件路径。
- **P7-4 normalize_tag 统一入口**(D5):
  在 `crates/auto-lang/src/aura/schema.rs` 增加公开的
  `pub fn normalize_tag(tag: &str, schema: &AuraSchema) -> Option<&'static str>`
  (= resolve_tag 的 canonical 返回);vue.rs 和 jet/generator.rs 的私有
  normalize_tag 改为薄包装调用此函数;两者的差异(col→column 等)由
  schema aliases 驱动。**行为零变更**(golden 守护),只是入口统一。

#### P8:中优修复(择机)

- **P8-1 `auto docs` CLI 化**(D13):
  将 docs_gen.rs 的 `generate_core_reference` + `generate_kitchen_sink` 提取
  到 `crates/auto-lang/src/ui_gen/docs_gen.rs`(库代码);`crates/auto` 加
  `auto docs gen` 子命令调用。测试继续走库路径。
- **P8-2 ComponentRegistry 家族建模**(D6):
  LoadedPackage 增 `families: HashMap<String /*parent*/, Vec<String> /*children*/>`;
  解析包内 widget 时,通过 schema sub_widgets 折叠匹配推导家族关系。
- **P8-3 unclassified 分批归类**(D11):
  103 个 unclassified 按 registry/gallery 来源逐批给 TIER_OVERRIDES;
  目标降至 <30(真正无实现数据的待定词汇)。
- **P8-4 DOC_TODO_BASELINE 裁剪**(D12):
  fence 改为"已覆盖的基线条目 = 红"(不再纯 println);首批裁掉
  areachart/barchart/donutchart/navmenu(4 条已有 gallery 页)。
- **P8-5 LSP 位置感知**(D8):
  UiElement 上下文检测改为:从 cursor 位置向上扫描最近的 `{` 块头,
  判定是否在 view 块内(需要 LSP 提供行级上下文,可能需要 AST 增量解析)。
- **P8-6 web+vm 双端冒烟**(D13):
  ComponentRegistry 的 resolve() 接入 vm_bridge / aura_view_builder 的
  tag 分发路径(至少 web + iced 两端),确保包组件在桌面端也可用。
- **P8-7 VitePress ↔ gallery 互链**(D13):
  core.md 的每元素加 `[demo →](/examples/widgets-gallery/<tag>)` 链接;
  gallery 页 Properties 段旁加 `[API →](/components/core#<tag>)` 反向链接。

### 6.3 验收

- **P6**:确定性漏洞修复后,golden 双跑 diff=0 含 API 函数场景;
  vue 映射有独立校验源(不再自指);render_support 无自相矛盾诊断。
- **P7**:gallery carousel 页 SFC 引用真实组件(非 div);
  kitchen-sink 可通过 URL 访问;codeeditor 渲染正确;
  load_package 容错(单文件失败不影响其他);normalize_tag 单一入口。
- **P8**:`auto docs gen` CLI 可用;包家族可查询;
  unclassified <30;基线自动裁剪;LSP 不误触;
  双端冒烟通过;VitePress 双向链接上线。

### 6.4 计划承诺未交付清单(原始承诺 → 实际状态)

| 承诺 | 原文 | 实际 | 归属 |
|---|---|---|---|
| `auto docs gen` CLI | P5 | 测试内环境变量触发 | P8-1 |
| Properties 段生成替换 | P5 | 仅对拍(检查不矛盾),未替换页内 | P8-7 顺带 |
| kitchen-sink playwright | P5 | 页面生成但未路由/无 spec | P7-2 路由;playwright spec 待基建 |
| VitePress ↔ gallery 互链 | P5 | core.md 单向 | P8-7 |
| 多后端一致性冒烟 | §3.1 补强③ | 仅 web | P8-6 |
| `syntax: special` 标记 | §5 风险 | 未实现(零出现) | —(低优,语法特判仍在 Rust) |
| parser/vb 别名归一统一 | P4 顺手项 | 未实现(两份私有不一致) | P7-4 |
| shadcn 长尾"核心集 props 全声明" | §5 风险 | 44% web_component 零 props | P8-3 顺带 |

## 7. §6 缺陷修复执行结果(2026-08-25,worktree `plan-435-schema-unification`,P6/P7 全量 + P8-3/P8-4)

### P6 紧急修复(D1/D2/D3)✅

- **P6-1(D2)确定性**:`vue.rs` `api_functions_used` 发射前 `.sort()`(import 行与
  deprecation warning 同源同序);删除零调用点的 `sorted_entries` 死代码,注释改为
  "各发射点内联排序"约定。新增单测 `cap_435_p6_api_import_sorted_and_deterministic`
  (≥2 API 函数,8 轮生成字节一致 + 字母序断言)。
- **P6-2(D1)vue 围栏去自指**:
  - `schema_drift.rs`:aura.at 每条 `@/components/ui/<pkg>` vue 行必须三源之一可解析
    —— ①官方包 `packages/widgets/registry/<pkg>`(component+extras 必须真在导出面:
    index.ts `as <N>` 或 `<N>.vue` 文件);②`cmd_vue.rs` `detect_shadcn_components`
    安装表(构建期 shadcn-vue add 物化);③`LOCAL_UI_PKGS` 白名单(data-table/
    nav-link/toast,app 本地手写,登记 = 显式事件)。原 P4-4 空转断言
    (registry≡schema,两侧同源永真)由此替换;失败消息清理过时的
    `rs/vue_duplicate_insert` 维度名。
  - `gallery_golden.rs`:SFC 实际发射的全部 `@/components/ui/*` import 同款三源
    校验(先于 UPDATE 采样执行)。
  - **围栏落地即抓真缺陷**:官方包 `registry/dialog` 缺 DialogTrigger/DialogClose/
    DialogTitle/DialogDescription 四个导出(schema 声明了但包里没有)。dialog 库模板
    `extra_support_files` 补 4 个 reka-ui 薄包装(index.ts 自动重导出),重生成官方包
    (顺带物化此前缺失的 `chat_message/` 目录,其余组件字节不变)。
- **P6-3(D3)render_support 反向围栏**:schema 中 backends.iced 解析为具体级别
  (full/partial/fallback/unsupported)的元素,其折叠键必须在静态详情表有臂
  (首跑抓到 6 个缩合拼写违规 —— tabscontent/codepane/form_item/list_item 等,
  与 baseline 既有 rs_not_in_render 漂移互为镜像,按 resolve_tag 折叠语义收敛);
  `get_support` 在 schema overlay 生效时清空 "unknown tag" 兜底 note
  (消灭 Full+unknown 自相矛盾)。

### P7 高优修复(D4/D5/D7/D9)✅

- **P7-1(D4)carousel 家族退役**:schema.rs 删 5 个 carousel ElementDef +
  生成器 `RETIRED_OFFICIAL_FAMILIES` 排除 6 个折叠键 + aura.at 再生成。
  gallery carousel 页 SFC 现发射真实组件
  (`<CarouselContent>/<CarouselItem>/<CarouselPrevious>/<CarouselNext>` + import,
  非 div)。附带:progress `one_of:0-100` 伪枚举过滤
  (`plausible_enum_values`:含 `-`/`~`/`…` 或纯数字集合不发射 one_of,回退原类型)。
- **P7-2(D9)kitchen-sink 路由 + 拼写对齐**:app.at 加 `/kitchen-sink` 路由;
  canonical 选择规则改为"带分隔符形态优先(kebab > 下划线 > 无分隔缩合)"
  —— codeeditor→`code_editor`(连带 autodown→`autodown_editor`、
  numberinput→`number_input`,kebab 约定不受影响);kitchen-sink 生成器对 app 本地
  命令式外壳组件(vue import 非 `@/components/ui/*`:CodeEditor/AutoDownEditor/
  ChatMessage)跳过变体发射(key/content 在真实组件上不存在)。
- **P7-3(D7)load_package 容错**:逐文件 try-parse,失败文件记
  `LoadedPackage::parse_warnings`(路径+错误)继续加载;全部失败才报错(带文件清单);
  api.rs 对 Ok 包逐文件发 S003 告警。新增单测 `load_package_survives_single_bad_file`。
- **P7-4(D5)normalize_tag 统一入口**:`aura/schema.rs` 新增
  `pub fn normalize_tag(tag, schema) -> Option<&'static str>`(= resolve_tag 的
  canonical);vue.rs / jet/generator.rs 私有实现改薄包装。要点:
  - 跨元素别名 **div/hr 保形**(div 是 container 的渲染等价别名,但类发射层历史
    区分两者 —— 归一会让普通 div 吃到容器默认类,golden 曾红);
  - jet 保留后端私有 Compose 词汇回退层(LazyColumn/TabRow/FlowRow 不在 schema);
    jet 派发臂补 `img` 双拼写(Image 走回退表不变,Img 走 canonical);
  - 围栏侧:Pascal 归一表退役 → `EXPECTED_VUE_TABLES=1`,其拼写面由
    `carried_spellings`(当前 aura.at aliases 自携带,P4-5a carried 先例)接管,
    Col/Column 等变体声明再生成不丢。

### P8 择机执行(P8-3/P8-4 ✅,其余挂账)

- **P8-4(D12)基线裁剪围栏硬化**:DOC_TODO_BASELINE 已被 gallery 页覆盖的条目
  = 红(原纯 println 从未触发裁剪);首批裁 areachart/barchart/donutchart
  (navmenu 经查为 nav_menu 元素别名,与 navigationmenu 页非同元素,诚实保留)
  33 → 30 条。
- **P8-3(D11)unclassified 分批归类:97 → 15(目标 <30 达成)**。85 个 shadcn
  家族件经 `TIER_OVERRIDES` 批量归 web_component(toaster 归 builtin_widget 对齐
  toast-provider;loading 归 web_component 对齐 skeleton);配套 DOC_EXCLUDE 按家族页
  登记 83 条(折叠键形态,注明"何处文档化")。剩余 15 = 真待定词汇
  (category-section/component-card/frame/media/menu-item/nav_item/navigation/
  notification/overlay/text_input 等)。

### 验收数据(全部绿)

- `cargo test -p auto-lang`:**3174 passed**(含新增 D2/D7 单测);
  `cargo test -p auto`:4 passed;`cargo test -p auto-man`:229+6 passed(复跑×2)。
- gallery golden **双跑 diff=0**(终态复验;中途每阶段均双跑验证)。
- 围栏全绿:schema_drift(含 D1 三源校验 + D3 反向围栏)/ docs_gen(4)/
  gallery_golden(含 import 存在性)/ component_registry_test(5)。
- drift baseline 顺带裁 71 行(carousel 退役 -10 条 + vue_mt1 维度退役 -61 行)。

### 未竟事项

- P8-1 `auto docs gen` CLI 化、P8-2 ComponentRegistry 家族建模、P8-5 LSP 位置感知、
  P8-6 web+vm 双端冒烟、P8-7 VitePress↔gallery 互链(见 §6.4 承诺差距表)。
- 预存(与本计划无关):`cargo test --workspace` 在 master 即有 E0080
  (`renderer.rs:11058` iced Subscription 捕获闭包,lib test cfg 特定 feature 组合
  触发)——按 crate 跑测试不受影响,待 iced 侧单独修。

## 8. P8 剩余项执行结果(2026-08-25 续,worktree `plan-435-schema-unification`)

### P8-1 `auto docs gen` CLI 化(D13)✅

生成器本体提取到库 `crates/auto-lang/src/ui_gen/docs_gen.rs`
(`generate_core_reference(root)` / `generate_kitchen_sink()` / `load_elements()`,
root 参数为 P8-7 的 demo 链接服务);tests/docs_gen.rs 降为纯围栏
(覆盖/对拍/同步),生成走库路径;`crates/auto` 新增 `auto docs gen`
子命令(`--only core|kitchen-sink` 可选),产物头部提示同步更新。
两产物 CLI 一键再生成,围栏绿。

### P8-2 ComponentRegistry 家族建模(D6)✅

`LoadedPackage` 增 `families: BTreeMap<parent, Vec<children>>`:
①schema sub_widgets 折叠匹配;②包内严格前缀兜底(Carousel ←
CarouselContent/…)。访问器 `family_children_of()`。单测验证 gallery
components 包的 Carousel(5 子件,含 CarouselDemo)/Combobox 家族。

### P8-5 LSP 位置感知(D8)✅

UiElement 补全上下文从全文 `contains("view {")` 改为**块头栈扫描**:
`full_prefix_upto`(全文到 cursor)→ `{`/`}` 维护开放块头栈,任一栈帧
首词为 `view` 才触发。on/model 块内裸标识符不再误触(单测三场景钉住:
view 内触发 / on 内不触发 / model 内不触发)。字符串内花括号属可容忍
启发式噪声,AST 化留待增量解析基建。

### P8-6 桌面端接入(D13)✅

- `WidgetRegistry::get` 折叠桥接兜底:kebab/大小写变体 tag(copy-button)
  在 iced 端命中 CopyButton(与 vue.rs map_tag 同语义;组件形态守卫防
  缩合小写误聚;内置优先不受影响 —— 派发表先查内置臂)。
- VM 运行时包加载:lib.rs 渲染启动路径处理 `use { package: x from "dir" }`
  —— `LoadedPackage.full_widgets`(P8-6 扩展 parse_package_widgets 返回
  (decl, widget) 对)视图注册进 WidgetRegistry、decl 并入 child_decls
  编入单 VM;加载失败 log::warn 不阻塞。与 vue 侧同源同机制。
- 单测:折叠桥接 + full_widgets 完整性(ui-iced 特性门控下验证)。

### P8-7 VitePress ↔ gallery 互链(D13)✅(正向)

core.md 每元素徽章行下加 `[demo →](/examples/widgets-gallery/<page>)`
(18 个有 gallery 页的核心元素;`gallery_page_stems(root)` 库助手)。
**反向链接(gallery 页 → core.md)未按原文实现**:gallery 是独立 AutoUI
应用(pac.at render: vm,3024 端口),不是 VitePress 宿主;页内
`/components/core#tag` 路由必然 404,且 pages/*.at 是手写源(70+ 文件),
机械插入跨应用死链得不偿失。若 website 侧后续挂 gallery 反代,再加反向。

### 验收数据(全部绿)

- `cargo test -p auto-lang`:3175 passed;`--features ui-iced`:
  component_registry(7)+ desktop_behavior(8)passed;
  `auto-lsp`:10;`auto`:2+2;`auto-man`:229+6。
- gallery golden 双跑 diff=0(终态)。
- `auto docs gen` CLI 实跑产出与围栏一致。

### §6 缺陷清单终态

| 缺陷 | 状态 |
|---|---|
| D1-D5, D7-D9, D11, D12 | ✅(P6/P7/P8-3/P8-4) |
| D6 | ✅ P8-2 |
| D8 | ✅ P8-5 |
| D13 | ✅ P8-1/P8-6/P8-7(正向);playwright spec 待基建(原计划即标注) |
