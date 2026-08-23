# Plan 421: vue 端 code_editor 契约补齐(props 消费 + oncursor/oncontextmenu)

> **状态**: ✅ 已实施(2026-08-23,分支 `421-vue-editor-contract`;P1-P4 全落,vitest 未搭——用契约测试 + 手验清单替代)
> **来源**: auto-man `vue.rs` 脚手架(CodeEditor 组件)与 `ui_gen/vue.rs` codegen 的契约缺口;413 vue 壳已落地但只通最小集
> **关联**: 413(@codemirror 深化"属后续增强")/ 401(examples vue 升级)

---

## 0. 一句话结论

**让 DSL `code_editor` 的 vue 生成物与 iced 端语义对齐**:五个已传 props 真正消费,两个缺失事件(oncursor/oncontextmenu)进 codegen 与脚手架,`lang:"auto"` 有映射。

## 1. 现状盘点

- **脚手架**(`auto-man/src/vue.rs` CodeEditor 组件):只实现 `modelValue/lang/lineNumbers/wrap`;`lineNumbers` 声明了但 extensions 未接线(行号不显示);`@codemirror` 基础壳已落地(413)。
- **codegen**(`ui_gen/vue.rs` code_editor 臂):生成 `:vi/:highlight-current-line/:tab-width/:font-size/:search` 绑定——组件全部静默忽略;**该分支无事件生成**(oncursor/oncontextmenu 完全不产出,只有 v-model + data-editor-key)。
- **lang**:`lang:"auto"`(041 在用)无 CodeMirror 映射,静默降级纯文本。
- playground 的 `CodeEditor.vue` 是另一套调试组件(props=onRun/breakpoints),与 DSL 契约无关——不在本计划范围,但需防混淆。

## 2. 方案要点

- **props 消费**(脚手架):`lineNumbers`→`@codemirror/view lineNumbers()`;`highlightCurrentLine`→`highlightActiveLine()`;`tabSize`→`EditorState.tabSize`;`fontSize`→主题 facet/CSS;`search`→`@codemirror/search` panel + `openSearchPanel`(或 highlight matches 以对齐 iced 的 live-highlight 语义——418 §8.8 已在 iced 端补齐渲染,vue 端用 `search` extension 高亮所有 match + 光标跳转,语义一致)。
- **vi 模式**:评估 `@replit/codemirror-vim` 引入(体积/维护性);若引入则 `:vi` 生效,否则文档明确 vue 端不支持 vi(降级声明,galaxy 文案同步)。
- **事件**:codegen 臂补事件分支——`@cursor="handler"`(`EditorView.updateListener` selectionSet 节流)、`@contextmenu`(原生 contextmenu 事件透传坐标);脚手架 emit 对应事件。
- **lang:"auto"**:StreamLanguage 简易词法(与 iced 端 syntect AutoLang 近似的注释/字符串/关键字三色)或引入 `@codemirror/legacy-modes` 近似;至少不再静默无色。
- **契约测试**:vue 生成物快照(已有 insta 基建)覆盖 props+事件全组合;脚手架组件单测(vitest 若 playground 有,否则最小 e2e 手验清单)。

## 3. Phases

- **P1 props 五件套消费 + lineNumbers 接线修复**(纯脚手架)。
- **P2 事件 codegen + emit**(codegen 臂 + 脚手架)。
- **P3 lang:auto 映射 + vi 评估决策**。
- **P4 快照/手验 + galaxy 文案**(与 418 §8.9 C 项文案呼应)。

## 4. 验收

- DSL 片段(props+事件全开)→ vue 生成物含全部绑定;组件渲染行号/高亮当前行/搜索面板行为正确。
- `oncursor` 在 vue playground 实测触发;快照测试入库。

## 5. 风险

- vim 扩展依赖决策(P3 显式关门,不阻塞 P1/P2)。
- oncursor 高频触发性能(updateListener 需 rAF/节流)。

---

## 6. 实施记录(2026-08-23)

### 实际代码形态与计划的偏差(已适配)

- **"该分支无事件生成"已过时**:`ui_gen/vue.rs` 的 shadcn 事件通用回路(fallback `on*` → `@*`)在 413 后已能把 `oncursor`→`@cursor`、`oncontextmenu`→`@contextmenu` 产出。P2 的真实缺口是**位置 payload 透传**:生成物 `@cursor="Handler"` 不带参,而 on-block 声明参数的 handler 生成的函数签名是 `(line: any, col: any)` → 运行时拿不到值。已按"事件回传 payload"模式补齐(sub-widget 路径的 `$event` 转发惯例)。
- **tabSize 不用自建 Compartment**:vue-codemirror 6.1.1 自带响应式 `tab-size` prop(内部 Compartment,同时设 `EditorState.tabSize` facet + `indentUnit`),直接透传即可,计划中的"Compartment updatable"由组件内部满足。
- **lineNumbers/highlightCurrentLine 的实现方式**:vue-codemirror 的 `DEFAULT_CONFIG` 无条件给初始 state 装 CM6 `basicSetup`(含 lineNumbers()/highlightActiveLine()/searchKeymap),即使不 `app.use`。追加 extension 无法移除基线,所以两个开关的 **false 分支**用 CSS 覆盖(theme 隐藏 `.cm-gutters` / 透明化 `.cm-activeLine`),true 分支由基线提供。
- **cursor payload 为 1-based**(`{line, column}`,CodeMirror `line.number`);iced 端 `code_editor_cursor_line/col` getter 是 0-based(041 handler 自行 +1)。vue handler 直接消费 payload 时**不要**再 +1(已在脚手架注释与本文件声明)。
- **契约测试用 fragment 断言而非 insta 快照**:沿 `tests/vue_capabilities.rs` 头注释的既定结论(SFC 输出 HashMap 顺序不稳定 + insta 基建嵌绝对路径在 worktree 失败),不是新增基建。

### 落地内容

- **P1 脚手架(`crates/auto-man/src/vue.rs`)**:CodeEditor.vue 补全 props 五件套消费 + `lang:"auto"/"at"` StreamLanguage 三色词法 + cursor(rAF 节流)/contextmenu(坐标+preventDefault)emit + `@codemirror/language`/`@codemirror/search` 两个新依赖;`vi` prop 声明但显式降级(不引 `@replit/codemirror-vim`)。
- **P2 codegen(`crates/auto-lang/src/ui_gen/vue.rs`)**:`base_event_to_dom` 显式映射 `oncursor/on_cursor`/`oncontextmenu/on_context_menu`;新增 `handler_params` 索引(on-block 参数,pattern → 参数名);`code_editor_event_payload_call` —— 当 handler 声明 ≥2 参时转发 `$event.line, $event.column`(cursor)/`$event.x, $event.y`(contextmenu),1 参转发整个 `$event`,0 参保持裸引用;DSL 显式实参优先;v-for 内保持 loop-var 惯例。
- **P3**:auto lexer 同 `main.ts` Prism `languages.auto` 关键字表(+var/loop/is/break/match);gallery 文案更新(`examples/widgets-gallery/src/front/pages/code-editor.at`):vue 壳 props+事件全通、vi iced-only。
- **P4 测试**:`ui_gen/vue.rs` 单测 3 个(全 props 组合、payload 双参/单参、0 参裸绑定;顺带修复了 413 遗留的连叠 `#[test]` 使 `test_code_editor_rendering` 双跑、`test_autodown_editor_rendering`(354)丢属性从不跑的问题);`tests/vue_capabilities.rs` 新增 `cap_code_editor_props_and_position_events`(真实 parse 管线 + shadcn 模式);auto-man `package_json_includes_codemirror_deps` 扩展覆盖新契约。
- **041 实测 review**:`examples/ui/041-auto-edit/src/front/app.at`（原 041-code-editor）以 shadcn 生成器过一遍,输出 `<CodeEditor v-model="src_main" lang="auto" … :highlight-current-line="true" data-editor-key="tab-main" … @cursor="CursorMain" …>`(041 handler 无参 → 裸绑定,由 VM natives 读光标,契约一致)。

### 遗留 / 未做

- **vitest 未搭**(计划 P4 的条件项):playground/脚手架工程均无 vitest 基建,按计划降级为下方手验清单。脚手架组件为 Rust 内嵌字符串,组件级单测需先抽出实体 .vue 样本 —— 未做。
- **vue 端 `code_editor_*` VM natives 未接**:041 的 `code_editor_cursor_line("key")` 等 FFI 仍只在 iced 端有全局 registry;vue 端等价能力由 cursor payload 事件承担(natives 桥接属另计划)。
- **oncursor 在 vue playground 的实机验证**(§4 验收第 2 条)未跑 —— 需 node 环境起 scaffolded 工程,见手验清单。

### 验收清单(手验,scaffolded vue 工程)

1. `auto new demo && cd demo` → pac.at 设 `render: "vue"`,view 放 `code_editor (lang: "auto", tab_width: 2, font_size: 16) { content: .src, oncursor: .CursorMoved }`,on-block `.CursorMoved(line, col)` 里把 line/col 写进 text。
2. `auto build && auto run`:行号显示(默认)、`lang:"auto"` 注释/字符串/关键字三色、Tab 缩进 2 列、字号 16。
3. 点击/移动光标:状态栏 line:col 跟随(1-based,不要再 +1);拖选连续移动不卡(rAF 节流)。
4. `line_numbers: false` / `highlight_current_line: false`:行号槽隐藏 / 当前行高亮消失。
5. `search: .query` + input 绑定:输入正则即全量高亮匹配;Ctrl+F 弹出搜索面板且预填当前 query。
6. `oncontextmenu: .Ctx(x, y)`:右键弹自定义菜单坐标正确;`oncontextmenu.prevent` 不弹原生菜单。
7. `vi: true`:vue 端无 vim 键位(降级预期),iced 端 `render: "vm"` 同 DSL vi 生效。
