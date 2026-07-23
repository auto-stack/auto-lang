# Specs 全局索引

> **本文件由 `scripts/spec-index.py` 生成，请勿手改。**
> 规约见 [README.md](README.md)；设计见 [docs/design/plan-spec-hybrid-model.md](../design/plan-spec-hybrid-model.md)。

## 语言核心

| Project | 状态 | 模块数 | 项目卡 |
|---|---|---|---|
| auto-lang（语言核心） | active | 9 | [auto-lang/project.md](auto-lang/project.md) |
| auto-val | active | 6 | [auto-val/project.md](auto-val/project.md) |
| auto-atom | active | 3 | [auto-atom/project.md](auto-atom/project.md) |
| a2r-std | active | 5 | [a2r-std/project.md](a2r-std/project.md) |
| stdlib | active | 8 | [stdlib/project.md](stdlib/project.md) |
| aavm | experimental | 8 | [aavm/project.md](aavm/project.md) |

<details><summary>auto-lang 模块明细</summary>

| 模块 | 职责 | 状态 |
|---|---|---|
| frontend | lexer/token/parser/AST/dialect/resolver/宏 | partial |
| types | 类型推断 infer/、typeck、ownership（borrow/lifetime/cfa）、trait_checker | partial |
| comptime | 编译期求值 | partial |
| interpreter | TreeWalker 解释器 | partial |
| vm | AutoVM：abt/codegen/engine/debugger/ffi/generic | partial |
| trans | 转译后端：C/Rust/JavaScript/TypeScript/Python/GDScript/r2a | partial |
| runtime | runtime/scope/session、libs/ 内建标准库绑定、ffi | partial |
| ui | ui/（app/component/gpui/headless）、ui_gen/（ark/jet/block/ts）、a2ui/、aura/ | partial |
| mcp | MCP server 集成 | partial |

</details>

<details><summary>auto-val 模块明细</summary>

| 模块 | 职责 | 状态 |
|---|---|---|
| value / nano_value | 值表示（Value 枚举及紧凑变体） | active |
| node | AST 节点结构 | active |
| obj / pair | 对象与键值对结构 | active |
| string / str_slice / owned_str / cstr | 字符串类型族（AutoStr 等） | active |
| array / linear / kids | 集合/子节点容器 | active |
| meta / path / shared / types / to_value / emit | 元信息、路径、共享类型与转换输出工具 | active |

</details>

<details><summary>auto-atom 模块明细</summary>

| 模块 | 职责 | 状态 |
|---|---|---|
| atom | Atom 数据结构定义 | active |
| parser | Atom 解析器 | active |
| error | 错误类型（thiserror） | active |

</details>

<details><summary>a2r-std 模块明细</summary>

| 模块 | 职责 | 状态 |
|---|---|---|
| list / hashmap / string_builder | 集合与字符串构建 | active |
| str | 字符串函数 | active |
| json | JSON 读写（serde_json） | active |
| http | HTTP 客户端（ureq） | active |
| fs / env / math / time | 文件系统、环境、数学、时间 | active |

</details>

<details><summary>stdlib 模块明细</summary>

| 模块 | 职责 | 状态 |
|---|---|---|
| auto | 核心标准库，多后端变体（.vm.at/.rs.at/.c.at） | active |
| auto/encoding | base64 / csv / hex 编解码 | active |
| auto/iter | 迭代器 | active |
| c | C 标准库绑定（stdio/stdlib） | active |
| aura | AURA 类型（Types.at）与 widgets 定义（data/display/feedback/form/layout/navigation/overlay） | active |
| collections | hashmap（C 后端） | active |
| may | 协程库绑定 | experimental（.at.skip，未启用） |
| result | option/result（C 后端） | experimental（.at.skip，未启用） |

</details>

<details><summary>aavm 模块明细</summary>

| 模块 | 职责 | 状态 |
|---|---|---|
| lib/lexer.at / token.at / pos.at | 词法分析与位置跟踪 | experimental |
| lib/parser.at / ast.at | 语法分析与 AST | experimental |
| lib/typeinfer.at / generics.at | 类型推导与泛型 | experimental |
| lib/eval.at | 树遍历求值 | experimental |
| lib/vm.at / opcode.at | 字节码 VM 与指令集 | experimental |
| lib/codegen.at / a2r.at | 代码生成与 Rust 转译辅助 | experimental |
| lib/error.at | 错误类型 | experimental |
| pac.at / greet_mod.at | 包定义与示例模块 | experimental |

</details>

## 工具链

| Project | 状态 | 模块数 | 项目卡 |
|---|---|---|---|
| auto-cli | active | 5 | [auto-cli/project.md](auto-cli/project.md) |
| auto-man | active | 11 | [auto-man/project.md](auto-man/project.md) |
| auto-gen | active | 6 | [auto-gen/project.md](auto-gen/project.md) |
| auto-lsp | active | 9 | [auto-lsp/project.md](auto-lsp/project.md) |
| auto-vm | active | 2 | [auto-vm/project.md](auto-vm/project.md) |
| auto-cache | active | 7 | [auto-cache/project.md](auto-cache/project.md) |
| auto-bindgen | active | 4 | [auto-bindgen/project.md](auto-bindgen/project.md) |
| auto-macros | active | 2 | [auto-macros/project.md](auto-macros/project.md) |

<details><summary>auto-cli 模块明细</summary>

| 模块 | 职责 | 状态 |
|---|---|---|
| main | clap 子命令定义、脚本执行/REPL 分发、JSON 错误格式化、转译子命令 | active |
| cmd_ui | `auto ui` 系列（list/select/install 等 UI 工程命令） | active |
| cmd_block | `auto block list/show/add/check`：blocks 目录浏览、参考实现拷贝、校验 | active |
| cmd_a2c_stdlib | `auto a2c-stdlib`：生成 a2c 标准库 | active |
| cmd_vue / cmd_tauri | Vue/Tauri 工程脚手架源码 | orphan（文件存在但未被 main.rs 挂接） |

</details>

<details><summary>auto-man 模块明细</summary>

| 模块 | 职责 | 状态 |
|---|---|---|
| automan | 核心编排：命令分发、工程上下文 | active |
| pac | pac.at 包定义解析与模型 | active |
| resolver | 依赖解析（ModuleResolver trait 实现） | active |
| builder | 构建调度：cargo / ninja / tool / vue 后端 | active |
| exporter | IDE 工程导出：cmake / ghs / iar | active |
| git / index / lock / pull | 依赖获取、注册索引、锁文件 | active |
| scanner / target / dir / cache | 工程扫描、target 目录管理、本地缓存 | active |
| vue / tauri / jet / ark / rust_ui | 各前端生态的工程集成 | active |
| api_gen / tauri_backend / vscode / pkg | API/后端/扩展代码生成器，包管理器抽象（bun/npm） | active |
| asset / fs / util / version / error 等 | 基础设施与公共类型 | active |
| up | 升级功能 | disabled（zip 依赖已移除，模块注释停用） |

</details>

<details><summary>auto-gen 模块明细</summary>

| 模块 | 职责 | 状态 |
|---|---|---|
| bin/autogen | CLI 入口：generate 等子命令、配置文件加载 | active |
| data | Auto 格式数据源加载（DataSource/LoadedData） | active |
| template | 模板解析与渲染（Template/TemplateEngine） | active |
| generator | 生成编排（CodeGenerator/GenReport/GenerationSpec） | active |
| guard | 保护区段冲突检测与保留（GuardProcessor） | active |
| test_framework | 测试辅助 | test-only |

</details>

<details><summary>auto-lsp 模块明细</summary>

| 模块 | 职责 | 状态 |
|---|---|---|
| bin / lib | 二进制入口与库导出 | active |
| backend | LSP 协议后端：请求分发、文档生命周期 | active |
| completion | 自动补全 | active |
| diagnostics | 诊断发布 | active |
| goto_def | 跳转定义 | active |
| hover_info | 悬停信息 | active |
| signature_help | 函数签名帮助 | active |
| inlay_hints | 内联提示 | active |
| workspace | 工作区/多文件管理 | active |

</details>

<details><summary>auto-vm 模块明细</summary>

| 模块 | 职责 | 状态 |
|---|---|---|
| main | CLI 参数、单文件编译链接执行流程 | active |
| dump_code | 字节码 dump/反汇编辅助 | active |

</details>

<details><summary>auto-cache 模块明细</summary>

| 模块 | 职责 | 状态 |
|---|---|---|
| storage | SQLite 存储层 | active |
| fingerprint | blake3 内容指纹计算 | active |
| registry | 缓存条目注册表 | active |
| gc | 垃圾回收 | active |
| sandbox | Rust FFI 沙盒（libloading 动态加载） | active |
| scanner / sig_code | 源码扫描与 syn AST 签名扫描 | active |
| automan / trans / aie_bridge | 与 auto-man / 转译 / AIE 的集成桥 | active |

</details>

<details><summary>auto-bindgen 模块明细</summary>

| 模块 | 职责 | 状态 |
|---|---|---|
| main | CLI 入口 | active |
| extractor | 从 Auto AST 提取 FFI 声明 | active |
| type_map | Auto ↔ C 类型映射 | active |
| manifest | manifest/头文件输出模型（serde） | active |

</details>

<details><summary>auto-macros 模块明细</summary>

| 模块 | 职责 | 状态 |
|---|---|---|
| lib | inventory 注册宏入口 | active |
| rust_fn_draft | Rust 函数声明解析（草稿） | draft |

</details>

## UI/Web 生态

| Project | 状态 | 模块数 | 项目卡 |
|---|---|---|---|
| auto-playground | active | 8 | [auto-playground/project.md](auto-playground/project.md) |
| widgets | active | 3 | [widgets/project.md](widgets/project.md) |
| forge-ui | active | 4 | [forge-ui/project.md](forge-ui/project.md) |
| lab-ui | active | 5 | [lab-ui/project.md](lab-ui/project.md) |
| playground-vue | active | 4 | [playground-vue/project.md](playground-vue/project.md) |
| website | active | 7 | [website/project.md](website/project.md) |
| blocks | active | 4 | [blocks/project.md](blocks/project.md) |

<details><summary>auto-playground 模块明细</summary>

| 模块 | 职责 | 状态 |
|---|---|---|
| main | axum server 启动、CORS/静态资源、路由挂载 | active |
| routes | HTTP/WS 路由：run / run_code / run_abt / trans / examples / notebook / agent_debug | active |
| code_runner / vm_runner | 代码执行与 VM 运行封装 | active |
| debugger | 调试会话：controller + session | active |
| agent_debug | AI agent 调试会话：controller + session | active |
| notebook | 单元格交互执行 | active |
| project | playground 工程/文件管理 | active |
| frontend | Vue3 + Vite SPA（playwright e2e） | active |

</details>

<details><summary>widgets 模块明细</summary>

| 模块 | 职责 | 状态 |
|---|---|---|
| registry | 生成的 Vue3 组件原语（10 个 widget 目录） | active |
| cli | `npx @auto-ui/widgets add/list` 命令 | active |
| styles | Tailwind 配置与预编译 CSS 构建（build-styles.cjs / src/input.css） | active |

</details>

<details><summary>forge-ui 模块明细</summary>

| 模块 | 职责 | 状态 |
|---|---|---|
| views | 页面视图（Agents/Chats/Specs/StreamingDemo） | active |
| components | UI 组件（Gate/Spec/Markdown/Streaming 渲染等，含 category/detail/editors 子目录） | active |
| composables | 数据与状态逻辑（useForge/useSpecs/useRelay/useStreamingDocument 等） | active |
| utils / types / styles | 工具函数、类型定义、样式 | active |

</details>

<details><summary>lab-ui 模块明细</summary>

| 模块 | 职责 | 状态 |
|---|---|---|
| components/atoms | 基础原子组件 | active |
| components/cells | 单元格组件 | active |
| components/layout | 布局组件 | active |
| components/notebook | notebook 容器组件 | active |
| composables/useNotebook | notebook 状态逻辑（含 __tests__） | active |

</details>

<details><summary>playground-vue 模块明细</summary>

| 模块 | 职责 | 状态 |
|---|---|---|
| AutoPlayground / AutoPlaygroundFull | 入口组件（index.ts 导出） | active |
| components | 编辑器/控制台/字节码/调试/回放/文件树等面板 | active |
| composables | usePlayground / useDebugger / useReplayPlayer 等 | active |
| lang | CodeMirror 6 语言支持（auto / abt）、暗色模式 | active |

</details>

<details><summary>website 模块明细</summary>

| 模块 | 职责 | 状态 |
|---|---|---|
| .vitepress | VitePress 配置与自定义主题 | active |
| docs | 英文文档（architecture/cli/features/guides/language/tutorials 等） | active |
| zh | 中文文档镜像（docs/books/ui 等） | active |
| books | 8 本书籍内容 | active |
| playground.md / ui / blocks / charts 等 | 专题页与内嵌 playground | active |
| scripts | prepare-content 等内容预处理脚本 | active |
| tests | Playwright e2e | active |

</details>

<details><summary>blocks 模块明细</summary>

| 模块 | 职责 | 状态 |
|---|---|---|
| form/login | 登录表单 block | active |
| data-display/note-list | 笔记列表展示 block | active |
| editor/note-editor | 笔记编辑器 block | active |
| navigation/sidebar-nav | 侧边导航 block | active |

</details>

## 外围/验证

| Project | 状态 | 模块数 | 项目卡 |
|---|---|---|---|
| parity | active | 6 | [parity/project.md](parity/project.md) |

<details><summary>parity 模块明细</summary>

| 模块 | 职责 | 状态 |
|---|---|---|
| auto-parity/main | CLI 入口 | active |
| auto-parity/runner | 三后端运行器 | active |
| auto-parity/compare | 输出比对 | active |
| auto-parity/report / tap | 报告与 TAP 格式输出 | active |
| libs | 20+ 三方库移植样例（一致性语料） | active |
| docs | parity-guide / known-divergences / dashboard | active |

</details>
