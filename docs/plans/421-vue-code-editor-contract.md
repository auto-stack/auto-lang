# Plan 421: vue 端 code_editor 契约补齐(props 消费 + oncursor/oncontextmenu)

> **状态**: 📋 已立项待实施(2026-08-22,源自 editor 残留盘点:codegen 传的 props 脚手架不消费、事件不生成)
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
