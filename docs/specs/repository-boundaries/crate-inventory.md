# crate-inventory（三仓 workspace 成员检验清单）

> **Status**: active（PLAN-869 SD-02）
> 生成命令：`python scripts/crate-inventory.py`；差异检查：`python scripts/crate-inventory.py --check`（清单与仓库不一致即红）。
> 机读版：`scripts/crate-inventory.json`（含三仓 commit/成员指纹）。

七栏：职责 | 公开出口 | 重复实现 | 无消费者旧路径 | 依赖方向 | 测试与档位 | 文档与规范。行绑定生成时三仓 commit（见文首指纹）。

## auto-lang @ `a9c5af389`

| crate | 职责 | 公开出口 | 重复实现 | 无消费者旧路径 | 依赖方向 | 测试与档位 | 文档与规范 |
|---|---|---|---|---|---|---|---|
| `a2r-actor-tests` | Plan 387 行为等价测试：actor .at 转译→编译运行→对拍 VM actor 套件 | pub×0（P 1 文件/14 行，inline test 0） | — | 零真死项（stdlib_assembly/manifest 与 auto-lang tests 活跃消费） | 上游 1 | —（无档位下限锚定） | specs/a2r-actor-tests/ |
| `a2r-std` | a2r 转译 Rust 代码的运行时标准库（actor/task 容器与胶水） | pub×426（P 21 文件/10978 行，inline test 2710） | —（844 已清偿死纯 shim 3 函数） | 零真死项（aavm/taa 语料依赖面，F-9 保留有证） | 上游 0 | —（无档位下限锚定） | specs/a2r-std/ |
| `auto-api-gen` | API 服务端/客户端生成核心（PLAN-815 T-04，自 auto-man 提取的跨仓单源） | pub×225（P 5 文件/11774 行，inline test 1251） | —（844 F-9 保留有证：auto-api-gen 跨仓单源） | api_gen.rs ~31 真死项（scan4；最大文件 9,772 行） | 上游 3 | —（无档位下限锚定） | specs/auto-api-gen/ |
| `auto-ast` | AST 节点族与构造辅助（parser/编译器共享语法树） | pub×244（P 34 文件/9038 行，inline test 809） | — | ast/atom_helpers.rs 9 真死构造器（int/uint/float/str/cstr/ident/bool/char/null_node，零消费） | 上游 2 | —（无档位下限锚定） | specs/auto-ast/ |
| `auto-atom` | 静态 Atom 数据结构与解析器（Atom 语法层） | pub×36（P 4 文件/2006 行，inline test 599） | — | 零真死项 | 上游 1 | —（无档位下限锚定） | specs/auto-atom/ |
| `auto-bindgen` | C 头清单生成器（AutoLang C FFI，Plan 216） | pub×15（P 5 文件/697 行，inline test 103） | — | type_map.rs 2 真死映射函数（ctype_to_c_str/ctype_to_rust_str，近全死文件 49/57 行） | 上游 0 | —（无档位下限锚定） | specs/auto-bindgen/ |
| `auto-cache` | 全局构建缓存（跨项目产物共享） | pub×170（P 12 文件/5914 行，inline test 1341） | — | 零真死项 | 上游 1 | —（无档位下限锚定） | specs/auto-cache/ |
| `auto-cosmic-demo` | Plan 365：mock 端口驱动的时钟+电池小程序（验收样例） | pub×0（P 1 文件/93 行，inline test 0） | — | 同上：cosmic 族结构性候选（待签核） | 上游 3；跨仓→ auto-ui-runtime | —（无档位下限锚定） | specs/auto-cosmic/ |
| `auto-cosmic-host-libcosmic` | Plan 365 W3：libcosmic 宿主后端（VTree→Element lowering，Linux；Windows headless 回退）——509 裁定后观察态 | pub×5（P 3 文件/219 行，inline test 30） | — | 同上：cosmic 族结构性候选（待签核；libcosmic 直渲路线经矩阵证伪出局） | 上游 3；跨仓→ auto-ui-runtime | —（无档位下限锚定） | specs/auto-cosmic/ + docs/plans/reports/509-smithay-route-verdict.md |
| `auto-cosmic-host-smithay` | Plan 509 路线 B：Smithay 合成宿主（winit/nested 开发形态；Linux 门控，Windows stub） | pub×5（P 4 文件/222 行，inline test 0） | — | 同上：cosmic 族结构性候选（待签核；Stage 2 live attach 见债务簿 509） | 上游 0 | —（无档位下限锚定） | specs/auto-cosmic/ |
| `auto-cosmic-ports` | Plan 365：COSMIC 复刻系统端口接口（通知/电源）+ mock 框架 | pub×17（P 2 文件/254 行，inline test 42） | — | 零真死项；整族无 workspace 消费者但有活跃规范页与债务簿 509 条目（负控第五面不过）→ 结构性减法候选（待签核） | 上游 0 | —（无档位下限锚定） | specs/auto-cosmic/ |
| `auto-cosmic-ports-linux` | Plan 365 W4：Linux 真实适配（zbus/UPower/D-Bus）+ 跨平台回退 | pub×9（P 3 文件/259 行，inline test 34） | — | 同上：cosmic 族结构性候选（待签核） | 上游 1 | —（无档位下限锚定） | specs/auto-cosmic/ |
| `auto-diagnostics` | 诊断基础：AutoError 聚合 + 语法/类型/名称/运行时/comptime/警告族（PLAN-793 提取） | pub×21（P 2 文件/1259 行，inline test 0） | error-reply 采样族 ×10（844 残留候选→872 DISTINGUISH 裁定，零行为变更） | 零真死项 | 上游 2 | —（无档位下限锚定） | specs/auto-diagnostics/ |
| `auto-gen` | 代码生成编排（JSON/C/Rust 等目标，基于 auto-gen-core） | pub×63（P 6 文件/1569 行，inline test 183） | — | 零真死项 | 上游 3 | —（无档位下限锚定） | specs/auto-gen/ |
| `auto-gen-core` | 无解释器生成核心：守卫段/输出编排/通用错误/生成报告（PLAN-791） | pub×23（P 4 文件/870 行，inline test 358） | — | 零真死项 | 上游 1 | —（无档位下限锚定） | specs/auto-gen-core/ |
| `auto-lang` | 组合根主 crate：CLI/编译入口/测试面（auto run/test 等），聚合 lang-core 与 ui 三包（PLAN-814 组合根） | pub×231（P 16 文件/7808 行，inline test 11815） | lib.rs async run-glue 9 根（843 残项，T-05 立项审计） | tests/ 白名单 T2 面 + api_gen.rs 真死项（scan4：~31 项）；ui_gen 已迁 auto-ui-compiler（867） | 上游 19；跨仓→ auto-ui-compiler, auto-ui-dialect, auto-ui-runtime | lang-t(3192); lang-tv(163) | specs/auto-lang/ |
| `auto-lang-core` | 纯语言 SDK 单源：scanner/parser/type/compile/VM/trans 核心（PLAN-814，禁 UI/Man 实现依赖） | pub×4751（P 302 文件/251319 行，inline test 32856） | 三入口残留（stdlib.rs 5,679/http_server.rs 4,156/trans/rust.rs 5,960 生产行，843 超限残项）；call_inner 7,439 行 if 链（F-7，T-05 立项） | implicit_union.rs 整模块（521 行，Plan 125 遗留，零消费）；vm/ffi/websocket.rs 整模块（238 行，Plan 350 遗留，零消费）；database/mod.rs 11 项（844 Scope 退役后残项）；compile.rs/types.rs/module_cache.rs/http_upload_service.rs/vm/engine.rs/vm/generic_registry.rs/ffi.rs 等真死项（scan4 非 ffi 合计 ~60 项；vm/engine 与 generic_registry 标注 871 相邻待其结论） | 上游 15 | lang-t(3192) | specs/language-sdk/ + specs/auto-vm/ |
| `auto-lex` | 词法基础：Token/TokenKind/Pos + Lexer/LexerState（PLAN-789 提取） | pub×49（P 4 文件/2811 行，inline test 654） | — | 零真死项 | 上游 1 | —（无档位下限锚定） | specs/auto-lex/ |
| `auto-lsp` | AutoLang LSP 服务器（编辑器工具面） | pub×43（P 13 文件/4769 行，inline test 259） | — | 零真死项；外部消费者=auto-vscode（build-lsp.ps1/.sh + LSP 契约文档）——不可按无消费者退役 | 上游 6；跨仓→ auto-ui-compiler | —（无档位下限锚定） | specs/auto-lsp/ + docs/lsp-api-contract.md |
| `auto-macros` | 过程/属性宏（rust_fn 内在函数注册、test-case 宏等） | pub×7（P 2 文件/917 行，inline test 0） | — | 零真死项（宏展开式消费面，grep 不可见，任何减法须 cargo check+档位反例兜底） | 上游 2 | —（无档位下限锚定） | specs/auto-macros/ |
| `auto-media-core` | 媒体基础：渲染中立图像资产管线（注册/票据/缓存/调度/HTTP 规则）+ 本地媒体文件服务（PLAN-811 自 ui 提取） | pub×148（P 3 文件/4677 行，inline test 1839） | — | image_pipeline.rs 真死 pub 项（scan4：~9 项，约 221 行死质量） | 上游 0 | —（无档位下限锚定） | specs/auto-media-core/ |
| `auto-parser-api` | 方言移植 API：轻量 CompilerSession/Scenario + ExternalDialect + 对象安全 ParseContext（PLAN-801 依赖倒置） | pub×30（P 5 文件/582 行，inline test 42） | — | 零真死项 | 上游 3 | —（无档位下限锚定） | specs/auto-parser-api/ |
| `auto-playground` | Playground web API 服务器（lib-legacy 载荷宿主，PLAN-752 活跃） | pub×120（P 21 文件/4263 行，inline test 254） | — | 零真死项（F-9 保留有证：lib-legacy 活跃） | 上游 1 | —（无档位下限锚定） | specs/auto-playground/ |
| `auto-project-model` | pac.at 声明轻量静态项目模型（PLAN-780；无编译器/VM/UI 类型） | pub×73（P 11 文件/6021 行，inline test 122） | — | 零真死项 | 上游 2 | —（无档位下限锚定） | specs/project-model/ |
| `auto-terminal-core` | 终端宿主域核心：cells/几何契约/autoterm_core.dll 服务/会话生命周期/轻端口（PLAN-808） | pub×56（P 5 文件/1649 行，inline test 252） | — | 零真死项 | 上游 0 | —（无档位下限锚定） | specs/auto-terminal-core/ |
| `auto-test-case` | AutoDown TestCase 协议核心：profile/CaseIR/选择/比对/报告（Plan 768） | pub×154（P 12 文件/6282 行，inline test 560） | — | 零真死项 | 上游 2 | —（无档位下限锚定） | specs/auto-test-case/ |
| `auto-text-core` | 文本基础：持久 rope/文件分页/快照/diff 引擎（PLAN-807 自 ui/code_editor 提取） | pub×120（P 6 文件/5987 行，inline test 2189） | — | 零真死项 | 上游 0 | —（无档位下限锚定） | specs/auto-text-core/ |
| `auto-tool-execution` | 工具执行宿主抽象（子进程/预算/超时执行面，I-TOOL 配套） | pub×69（P 5 文件/2944 行，inline test 287） | — | 零真死项 | 上游 0 | —（无档位下限锚定） | specs/tool-execution/ |
| `auto-tool-protocol` | I-TOOL 轻量工具协议：版本/能力/预算/错误/帧契约（PLAN-815 T-03） | pub×69（P 9 文件/1581 行，inline test 2） | — | 零真死项 | 上游 2 | —（无档位下限锚定） | specs/tool-protocol/ |
| `auto-val` | 值/节点基础类型（Value/Node/Atom 载体，全生态地基） | pub×443（P 22 文件/9100 行，inline test 2719） | nv_to_value 双签名（844 残留候选→872 DISTINGUISH） | value.rs 16 真死项（with_value/PathComponent/empty_owned_str/into_data/from_data 等）；node.rs 12 真死项（is_arg/set_main_arg_unified/fill_node_body 等） | 上游 0 | —（无档位下限锚定） | specs/auto-val/ |
| `auto-vm` | AV 解释器宿主薄层（VM 执行入口挂接） | pub×0（P 2 文件/207 行，inline test 0） | — | 零真死项；871（旧 lowering 退役）相邻——处置权归 871，本批不动 | 上游 1 | —（无档位下限锚定） | specs/auto-vm/ |
| `shim-metadata` | 编译产物元数据 shim（auto-cache 消费的最小元数据面） | pub×49（P 7 文件/2737 行，inline test 353） | — | G 桶 1 生成文件；无真死项 | 上游 0 | —（无档位下限锚定） | specs/shim-metadata/ |

## auto-ui @ `f489c7bdd`

| crate | 职责 | 公开出口 | 重复实现 | 无消费者旧路径 | 依赖方向 | 测试与档位 | 文档与规范 |
|---|---|---|---|---|---|---|---|
| `auto-ui-compiler` | UI 编译器：AURA 分析/提取、design tokens 与 vue/jet/ark/rust 四生成器（PLAN-814 提取；生成产物 import 路径=代码输出契约） | pub×954（P 76 文件/105015 行，inline test 20662） | — | widget/registry.rs 7、aura/schema.rs 5、jet/project.rs 5、shared/tailwind.rs 5 及 12 个单死项文件（style/ts_adapter/ark 修饰与 state/bp/registry/jet 族/shared/registry/widget/spec/aura/element_coverage/import_policy）+ vue.rs with_uses_autodown（DEBTS 450 已记死代码）；lucide_generated 为 pub(super) 表+访问器（活） | 上游 5；跨仓→ auto-atom, auto-lang-core, auto-parser-api, auto-val | ui-tu(863) | specs/auto-ui-compiler/（867 迁移后 owner=auto-ui 仓） |
| `auto-ui-dialect` | AutoLang UI 声明语法（msg/model）单源（PLAN-806；仅依赖 auto-parser-api 轻闭包） | pub×158（P 18 文件/10547 行，inline test 3937） | — | ast.rs 1、grammar/view.rs 1 真死项（mint_*/parse_* 族有同文件活调用者，非死） | 上游 4；跨仓→ auto-ast, auto-lang-core, auto-parser-api, auto-val | —（无档位下限锚定） | specs/auto-ui-dialect/ |
| `auto-ui-runtime` | UI 运行时：VM 渲染轨/桌面宿主栈（iced renderer、桌面会话/协议、editor/terminal/media 部件、宿主适配与 provider）（PLAN-814 提取） | pub×3333（P 246 文件/258248 行，inline test 86363） | — | 10 个单死项文件（photo_service/vm_bridge/desktop_protocol host+sendinput+stage3/iced image_surface/native_dock win32/scroll state/terminal iced widget/code_editor highlight+render）；renderer.rs 42,337 行（F-5 最大文件，T-05 立项） | 上游 10；跨仓→ a2r-std, auto-lang-core, auto-media-core, auto-parser-api, auto-terminal-core, auto-text-core, auto-val, autodown-core | ui-ti(28); ui-tn(2) | specs/auto-ui-runtime/ |
| `auto-ui-tools` | UI 工程工具包（PLAN-815）：auto-ui-tool 进程宿主（describe/validate_project）+ vue/rust 生成与构建/运行编排 | pub×350（P 28 文件/36828 行，inline test 7780） | — | vue.rs 死质量 6,384（scan2 估计，含字符串花括号误报，逐项以 scan4/复核为准）；rust_ui.rs ~12 项 | 上游 9；跨仓→ auto-api-gen, auto-lang-core, auto-project-model, auto-tool-execution, auto-tool-protocol, auto-val | ui-ti(28) | specs/auto-ui-tools/ |
| `shell-pack` | 桌面 shell 应用包（auto-man CLI 消费的 shell 载荷单源） | pub×17（P 1 文件/224 行，inline test 2） | — | 零真死项（auto-man crates/auto 活跃消费） | 上游 2；跨仓→ auto-lang-core | —（无档位下限锚定） | —（无独立 specs 页；README 内嵌） |

## auto-man @ `873fa234f`

| crate | 职责 | 公开出口 | 重复实现 | 无消费者旧路径 | 依赖方向 | 测试与档位 | 文档与规范 |
|---|---|---|---|---|---|---|---|
| `auto` | auto CLI（包管理/构建/运行入口；F-8 最大文件 main.rs 3,222 行，T-05 立项） | pub×24（P 10 文件/6393 行，inline test 438） | — | tool_ui_ops.rs 等真死项（scan4 man 侧：~8 项） | 上游 10；跨仓→ auto-lang-core, auto-project-model, auto-tool-execution, auto-tool-protocol, auto-ui-compiler, auto-ui-runtime, auto-val, shell-pack | man-ti(43) | specs/auto-man/（语言仓托管） |
| `auto-man` | Man 库门面（模板/资产/工具链集成 API，兼容门面 F-9 保留有证） | pub×11（P 3 文件/512 行，inline test 200） | —（844 已清偿 autoui-skill 目录逐字双份→单源） | 零真死项 | 上游 15；跨仓→ auto-api-gen, auto-cache, auto-gen, auto-lang, auto-media-core, auto-project-model, auto-ui-compiler, auto-ui-tools, auto-val | man-ti(43) | specs/auto-man/ |
| `auto-man-assets` | Man 模板与 builder 资产单源 + 纯脚手架路径规划（PLAN-803 无语言提取） | pub×5（P 2 文件/361 行，inline test 2） | — | 零真死项（844 已退役拆仓残留资产 5 文件） | 上游 0 | man-tc(252) | specs/auto-man-assets/ |
| `auto-man-core` | 管理/构建规划规则核心（PLAN-809：pac.at 规范化/target 闭包/源选择/构建规划纯规则） | pub×137（P 14 文件/4004 行，inline test 915） | — | 零真死项 | 上游 4；跨仓→ auto-atom, auto-project-model, auto-val | man-tc(252) | specs/auto-man-core/ |
| `auto-man-runtime` | Man 运行时适配（Git/FS 观察、builder 执行、生成请求编排） | pub×310（P 43 文件/14254 行，inline test 3080） | — | automan.rs 5 真死项（create_capp/create_lib/create_clib/pac_window_title/scenario_override）+ cache.rs/error.rs/builder ninja resolver 单死项 | 上游 12；跨仓→ auto-api-gen, auto-cache, auto-gen, auto-lang-core, auto-project-model, auto-tool-execution, auto-val | man-tc(252); man-ti(43) | specs/auto-man/ |
| `auto-package-lock` | pac.lock 核心：锁模型/TOML-JSON 编解码/依赖校验规则（PLAN-795 Target-free 提取） | pub×18（P 4 文件/505 行，inline test 222） | — | 零真死项 | 上游 1；跨仓→ auto-val | man-tc(252) | specs/package-lock/ |
| `auto-package-resolver` | 已解析依赖视图与模块路径解析核心（PLAN-800 自 resolver.rs 提取） | pub×18（P 1 文件/606 行，inline test 267） | — | 零真死项 | 上游 0 | man-tc(252) | specs/package-resolver/ |
| `auto-tool-client` | I-TOOL 客户端（auto CLI 调用 auto-ui-tool 进程的桥） | pub×43（P 2 文件/2817 行，inline test 593） | — | lib.rs 1 真死项 | 上游 3；跨仓→ auto-project-model, auto-tool-execution, auto-tool-protocol | man-tc(252) | specs/tool-protocol/ |
| `auto-toolchain-core` | 工具链配置模型/旗标映射/命令模板核心（PLAN-805 自 builder/ninja 提取） | pub×56（P 4 文件/1212 行，inline test 417） | — | 零真死项 | 上游 1；跨仓→ auto-val | man-tc(252) | specs/auto-toolchain-core/ |
| `auto-core@core-ws` | core 子 workspace 的 core-feature 构建视图（cargo tcore；与 crates/auto 共享同一 main.rs/源，非独立代码） | pub×0（P 0 文件/0 行，inline test 0） | 与 crates/auto 同源双 manifest（构建视图非代码重复） | —（无独立 src） | 上游 7；跨仓→ auto-lang-core, auto-project-model, auto-tool-execution, auto-tool-protocol, auto-val | man-tcore(23) | specs/auto-cli/ |

## 候选队列（T-03 输入；按 可减行数×风险倒数×证据完备度 排序）

| 排序 | 候选 | 类别 | 证据 |
|---|---|---|---|
| 1 | auto-lang-core implicit_union.rs 整模块 + auto-lang lib.rs 再导出 | 无消费者退役（批2 实施项） | scan4 真死（12 pub 项零消费）+ git grep 三仓全跟踪文件仅 lib.rs 接线与归档文档；连带 02-type-system.md 一句话同步 |
| 2 | auto-lang-core vm/ffi/websocket.rs 整模块 | 无消费者退役（批2 实施项） | scan4 真死（WsConnection+4 shim 零注册零消费）；HTTP 侧 WebSocket（ApiReply::WebSocket，Design 33）是另一子系统不受影响 |
| 3 | auto-ui-compiler/auto-ui-runtime/auto-ui-dialect/auto-man/auto-lang-core(非 vm-engine/generic_registry) 真死 pub 项（scan4 清单） | 无消费者退役（批2 实施项） | scan4 真死判据（声明仅一次+三仓零引用）+ git grep 全类型负控；vm/ffi 宏注册族与 872/871 相邻面已排除 |
| 4 | auto-lang-core vm/engine.rs + vm/generic_registry.rs 真死项 | 延后（871 相邻） | 871 旧 lowering 退役将重构 VM 面；处置权归 871 结论 |
| 5 | auto-cosmic 五子 crate 整族（~1,047 P 行） | 结构性减法（待用户按批签核，D-3 四要件） | 无 workspace 消费者，但 specs/auto-cosmic 标 active + 债务簿 509 条目 + 509 裁定报告=公开承诺面；替代路径=无（需先裁定 Smithay 线去留） |
| 6 | 语言三入口收敛（stdlib.rs/http_server.rs/trans/rust.rs 超 843 限）+ call_inner 拆分 + lib.rs async glue | 结构立项（T-05 包，不在本批） | F-7 规模证据；属拆分重构非死码 |
| 7 | UI renderer.rs 42,337 / vue.rs 34,451 / aura_view_builder.rs 28,044；Man auto/main.rs 3,222 | 结构立项（T-05 包，不在本批） | F-5/F-8 规模证据；GPU 状态机/生成器巨型 match 需独立探针与审查面 |

> 候选处置规则见 net-slimming（负控五面/四要件/2,000 下限）；结构性减法项须用户按批签核（D-3 裁定）。
