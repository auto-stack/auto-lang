# Plans 未归档状态审计（2026-08-20）

> **审计范围**：`docs/plans/` 下 30 个非归档计划文件（不含本文件与 `plans-status-audit-2026-08.md`、`plans-360-369-status-summary.md` 两个旧审计文档）
> **审计方法**：6 个并行核查代理对每个计划做代码级验证——计划自述状态 vs master（`f21dc88f`）实际代码（文件/函数/golden 测试/git 历史），重点甄别**遗漏功能点**与**workaround**
> **审计结论**：✅ 完全完成并归档 8 个；🟡 部分完成 17 个；📋 纯设计/调研未实施 5 个
> **取代**：`plans-status-audit-2026-08.md`（2026-08-01/10 版本，多处结论已被本轮实测推翻，见 §4）

---

## 0. 总览

| 状态 | 数量 | 计划 |
|------|------|------|
| ✅ 已归档（本轮） | 8 | 308 / 317 / 364 / 399 / 404 / 407-a2vue / 409 / 410 |
| 🟡 部分完成（核心落地，有明确遗留） | 17 | 242 / 243 / 346 / 359 / 366 / 396 / 398 / 400 / 401 / 402 / 403 / 405 / 407-minesweeper / 408 / 411 / 412 / 413 |
| 📋 纯设计/调研（未实施） | 5 | 330 / 332 / 386 / 394 / 406 |

---

## 1. 🟡 部分完成（17 个）

### 1.1 有真实功能缺口 / workaround 的（按优先级）

| 计划 | 主题 | 已落地 | 剩余工作（§引用） | workaround / 遗漏 |
|---|---|---|---|---|
| **396** | a2r 改进（auto-ai 滚动聚合） | §5 蓝图完备 | **五条缺陷 0/5 落地**（§2.1–§2.5：借用推理 B/C/D/E + unit-variant）；流程 §3 第 3-6 步（golden 回归、删 sed、build 验证） | auto-ai 侧 sed workaround 全部仍在（`retranspile.sh:172-181` / ai-config:88 / client:99-100）；§2.1 症状可能已被 Plan 399 P11.4 `borrowed_iter_vars`（rust.rs:173/8767）覆盖，**待验证后删 sed** |
| **400** | api_gen a2r body 转译（399 路线A） | Phase 1+2 已合并（d2642943：`is_thin_delegation`/`try_transpile_body`/`AUTO_A2R_BODY` api_gen.rs:1117/1133/1463 + 单测） | Phase 3（多 `back/*.at` + `extern fn` 语言扩展）；Phase 4（auto-musk 全栈端到端验收）；§4.4 `use.rust` 收集 | `is_thin_delegation` 只 match `Stmt::If\|For`（api_gen.rs:1121-1128），**while/match 循环体会被误判为薄委托走模板路线B**——注释与实现不符的逻辑遗漏 |
| **403** | 011 计算器 MCP+grid+多模式 | 需求 1a/1b/1c/2/3 + VM List 基建 + Phase 403-F 浮点修复（9b9fec81）全落地，VM 求值全通过 | **需求 1a 承诺的 `tests/desktop_mcp.py` + acceptance 契约未交付**（`examples/ui/011-calculator/` 无 tests/ 目录） | 文档曾自相矛盾（正文称浮点损坏待办，与顶部 ✅ 冲突，已更正）；验收 checklist 已回填 |
| **405** | 023-realworld（Conduit） | 阶段1 8/8 + 阶段2 14/14 全绿（编辑器遗留已由 plan401/023-editor-fix 修复）；VM 多 store bug 已由 Plan 370 外部解决 | **真正的 token 认证未做且未移交任何计划**——`current_user()` 返回空 User 桩（db.at:56-59），端点无鉴权；store struct 字面量 null 已记 401 技术约定留后续 | 023 规避的 a2r 限制（双路径参数只提取第一个、slug 前端手输避开 String 借用）仍在 |
| **408** | view fn → Vue 组件合成 | P1–P12 全部合并（plan408_tests 17 测试 + golden 007-010）；**§6.3 auto-musk 试点已完成**（auto-musk 023：20/21 逃生舱原生化 + StreamingRenderer 经 028 升格平台实现） | §11 两项：**P5-2 🟡 `auto clean root` panic 未修**（target.rs:284 仍 `panic!`，from_str 无 "root" 分支）；P5-4 🟢 纯 module fn 文件不被 codegen（api.rs:456 报错） | P5-4 的 workaround = 塞进 widget/store 文件；新债流入：auto-musk 029 登记 shadcn Button 映射丢动态 class/title 为本仓债务 |
| **411** | VM 视觉对齐 vue（Home/Button） | P0-A/P0-B/P1-A/gap 废弃/pac.at 窗口 ✅；P1-B toast + P2-A 部分（copy icon/折叠钮）08-15 落地 | P1-C Inter 字体内嵌；P2-A① codeblock Prism 色板、P2-A④ 表格细节；**P2-B MCP 四项强化**（`Button.content` 子树仍未序列化进 vtree，vnode.rs:192 仍 `label: String`——R3 误判根源仍在） | §8.5 gap 兼容分支（vue.rs 3 处 + view_builder 8 处）保留未拆、validator 白名单未加（防 AI 再写 gap 属性） |
| **412** | Layout Gallery + VM 布局引擎 | 12 layout 页 + `rederive_layout` 全路径 + `grid_row_placements` 分配器 + plan412_tests（结构通道全绿） | **§6.2/§6.3 视觉+交互验证未执行**（实施会话无 GUI/MCP 环境）：全页双端并排截图 ≤1px、scroll/Overlay 抽验；§9.2/§9.3 验收未闭环 | 降级矩阵类（flex-wrap 单行、absolute 就近位等）渲染期一次性 eprintln 提示——计划内显式降级 |
| **413** | 跨平台代码编辑器（code_editor widget） | 代码交付物 100% 落地（三分支全合并 master `06cb1881`：cosmic-text ViEditor 三层结构 + View/DSL + VM natives + a2r + vue CodeMirror shell + MCP + LRU/warm-up + e2e） | **人工验收清单未实机闭环**：微软拼音 IME、150% DPI 行号、Linux(X11/Wayland) 复验、TESTING.md 交互（三击/Ctrl+词跳转/滚动条/vi 模式）；`@codemirror` 深化已明示另立计划 | `Box::leak` keyed 存储已在 §5.4 声明并配 LRU（cap=32）缓解。注意：master 近期 "PLAN-029" 提交属 **auto-musk 仓库** 029 计划，非 413 范围 |

### 1.2 纲领/追踪型（按本性长期维护）

| 计划 | 主题 | 已落地 | 剩余工作 |
|---|---|---|---|
| **242** | a2r 功能差距 tracker | 表格已回填：#1/#3/#5/#6 ✅（泛型约束/.to/ext/struct 解构，2026-08-20 核实）、#12 a2r 发射侧 ✅（Plan 355）；#14 ✅ | 未做：#2 HashMap::from、#8 闭包推断（根因：rust.rs:8342 `infer_type_from_expr` 缺 `Expr::Closure` 分支，靠 task-struct 特判绕过）、#9 平台文件、#10 Redis/SQLite、#11 所有权收尾、#13 parity 收尾、#15 GPUI、#16 自举 Phase 2/E、#17 dep cc+memmap2 |
| **243** | LSP/VSCode 现代化 | Phase 1–4 ✅（references/rename/code_action/signature_help/inlay_hint 真实实现 + grammar 已更新） | Phase 5：TS 迁移（仍 extension.js 单文件）、semantic tokens；Phase 6：CI 仅 `workflow_dispatch`（push 触发因常红禁用）、集成测试 70 行低覆盖、生成关键字/类型/函数列表、lsp-api-contract.md |
| **346** | Web Framework 差距调研 | 调研本体完结；核心差距已实施（query/表单/中间件/500/日志/通配符，http_server.rs 内 "Plan 346" 标注；session/SSR/OpenAPI 由 352 完成） | 未实施且**无后续计划认领**：3c 重定向、5a 服务端 multipart 上传、5e Rate Limit、#12 Request-ID |
| **359** | "Auto 作 Rust 脚本层"发布 | C1/C2/**C3（仪表盘已落地**，report.rs render_maturity_directory + parity-dashboard.html 241/241 + CI artifact，769bfeb8——08-04 自述"未做"已过时）/B1/B2/D1/D2 部分 | D2 generators 用例（parity/libs 无 generators）+ D3.3 双 demo；D3 http_client_sync 仍 blocked（DIV-HTTP-LANG-1）；**Phase E 五项 open**（DIV-TRAIT-VM-1/VM-2/LANG-1/HTTP-LANG-1/CHAR-AT-1）；A1/A2 落地页（V3 未达）；165 checkbox 未回填 |
| **366** | 跨平台 UI 测试 DSL | 366a 近端交付 3-4/5：acceptance.atd（T1-T13+T12-DARK）+ Playwright spec.ts | `auto test:ui` 一键命令未实现（实际由 Plan 371 run_autotest.py/.autotest 承担，可声明取代）；366b-f DSL 本体长期设计 |
| **401** | AutoUI 示例升级纲领 | 018/022（404）/023（405）✅；011 拆 403；024 由 409 以 `examples/widgets-gallery/` 完成 | 待办：019/020/021/025（均单文件）；021 双目录并存易混淆；旧 `examples/ui/024-*` 目录仅剩 README/gen 残留可清理 |
| **402** | 038 扫雷示例 | vue 完整；VM 核心流程可用（§13.8 修复 + §13.10 实机点击已修 7c09e371 + 右键/计时器落地） | 连锁展开/数字显示/胜负的**实机目视确认**未记录闭环（§13.8 🟡）；rust 版归 407 |
| **407-minesweeper** | 038 扫雷 rust 第三后端 | Phase 1–2 + R6 右键 + R9 grid 居中，均合并（f863be5e）——文档原"计划阶段"严重滞后，已更正 | R7 动态窗口 resize（difficulty→窗口尺寸）；Phase 4（三后端对比 + 015/011 回归）；生成快照 stale（早于 R6 修复，重生成即带） |
| **398** | VM expose/store sibling 修复 | 核心三修全在 master（§11 log/§12 parser `[][]T`/§2-§3 sibling rewrite）；§14.2 已被 b0434cff 顺带完成 | **§14.1 回归测试未做**（handler_codegen tests 仅 5 个、无 sibling 覆盖；parser 无 `[][]T` 用例）；M0.5/M1 为 auto-shell 侧下游任务 |

---

## 2. 📋 纯设计/调研未实施（5 个）

| 计划 | 主题 | 现状 | 备注 |
|---|---|---|---|
| **330** | Agent 友好调试工具链（`auto debug` CLI） | 4 Phase 全未实施（debug.rs/introspection.rs/AUTO_VM_TRACE 均 0 命中） | 自述准确。**建议先与 Plan 199 已交付的 `auto Debug --agent`（JSON 模式断点调试器，main.rs:504）做范围去重**——330 独有价值在 widget state dump / heap-objects / AUTO_VM_TRACE 静态诊断 |
| **332** | `#[derive(ToAtom)]` proc macro | Phase A–E 全未实施（auto-val/auto-lang-macros 无 ToAtom 命中） | Plan 381 serde Deserializer 落地后优先级已降；建议正式关闭或并入债务簿 |
| **386** | AutoUI RenderQueue 分离渲染（未来优化） | Stage 1–3 零实施；启动条件明示"≥3 个 COSMIC app 跑通 Host ② + 内存预算证明"未满足 | ⏸ 自述准确，保持暂缓 |
| **394** | AWAIT_FUTURE 通用 future 架构 | Phase A–D 零实施；§4 触发条件自评均未出现 | draft 自述准确；Plan 349 re-entry yield 为明示的务实替代 |
| **406** | VM 类型系统审计（nanbox 生产者-消费者配对） | 18 checkbox 全空；`docs/audit/vm-type-audit.md` 不存在；Phase 2 目标项 master 仍未修（GET_ELEM List\<bool\> 压 i32、JMP_IF pop_i32 魔数、EQ 无 is_bool arm） | 原始动机（038 VM 阻塞）已消失；但 GET_ELEM bool 编码 / JMP_IF tag 解码仍是**潜伏 bug**，建议缩小范围重开或登记风险 |

---

## 3. ✅ 本轮归档（8 个）

| 计划 | 主题 | 核验结论 |
|---|---|---|
| **308** | Godot demo 逆向翻译 | 4 demo / 6 fixture / 11 测试函数全在 master（2026-06-14 落地）。**文档状态行曾事实性错误（称未实施）**，已更正后归档；旧审计 ":87" 同源过时 |
| **317** | VM 真异步调度统一 | Phase 1-5/7/8/11 全链验证通过（actor/~await/lazy yield/serve_async/e2e CI）；Phase 6/9/10 决议不做（§11 论证）；serve_async 生命周期留 follow-up（登记债务簿） |
| **364** | a2r COSMIC 就绪 | W1–W7 + Phase 8 F1–F3 全部落地（代码+git 双证）。**旧审计"W1-W7 全 pending"完全过时**；Try 降级/F4 deferred 已登记债务簿 |
| **399** | AutoUI 示例 SSE/CRUD 扩展 | 全部声称验证属实（017-chat 9/9、Phase 11-13 落地）；路线A 移交 Plan 400；api_gen 5 类后处理兜底登记债务簿 |
| **404** | 022-kanban | 交付物全落地（6 tests + 拖拽 + row/col 属性穿透修复）；§1 端口缺口描述过时已附更正（e865566e 早已修复） |
| **407-a2vue** | a2vue icon/text 表达式 | parser + golden 005/006 + auto-musk 侧回流全部完成，零遗留 |
| **409** | Widgets Gallery 三模式一致性 | §1–§10 全 ✅；§9.6 CodeBlock/PreviewCard 纯 Auto 化暂缓（登记债务簿）；PUA 标记嵌图标为声明的结构性方案 |
| **410** | check_symbol 报错 span | Phase 1 全落地（err_span + 3 调用点 + 测试）；Phase 2 带触发条件合格移交；Expr::Dot 不查符号登记债务簿 |

**归档同步动作**：`docs/plan-indices/`（03/06/08/11 四个分类 + 状态计数）与 `docs/plan-reports/` 对应更新；06 索引顺带把 162/164/165/166 四行从 ⏳ 修正为 ✅（与 242 tracker 回填一致）；03 索引把游离的 191/208 行归位。复审发现的债务已登记 `KNOWN-DEBT-AND-RISKS.md`（308/317/364/399/409/410 共 7 条新条目）。

---

## 4. 关键发现

1. **文档滞后于代码是普遍模式**（8/30 计划状态行与 master 现实不符）：
   - 308/407-minesweeper：说"未实施/计划阶段"，实际早已合并；
   - 359-C3/396-§2.1/401-端口缺口/404-§1/402-§13.10：自述的"遗留/缺陷"实际已被后续计划（769bfeb8/399 P11.4/e865566e/7c09e371）修复但未回填；
   - 408-§6.3/405-VM：声称开放的项目实际已完成。
   建议：归档前必须做本轮这种代码级核验，不能信任计划自述。
2. **PLAN-029 归属甄别**：master 最近 5 个提交的 "PLAN-029 T13/T15/T17/T20" 属于 **auto-musk 仓库**的 `029-frontend-escape-hatch-elimination.md`（23/23 全勾），是 auto-musk 侧的语言仓配套修复，**不属于本仓 413 或任何本仓计划**，勿混淆。
3. **无认领的悬空项**（不在任何计划里，需要立项或显式放弃）：405 token 认证、408 P5-2（auto clean root panic）、346 的 redirect/multipart/rate-limit/request-id、242 #8 闭包推断根因、406 的 GET_ELEM bool/JMP_IF 潜伏 bug、024 旧目录清理。
4. **跨仓库移交健康**：408→auto-musk 023（已完成）、399→400（进行中）、398→auto-shell M0.5/M1、402→407 rust 版，链路均有据可查。

---

## 5. 补全任务清单（Backlog，2026-08-20 登记）

> 本节把 §1/§2 中**可在本仓落地**的功能遗漏与 workaround 转为可执行任务；每项用独立 worktree + 分支实施（`plan-fix/<id>`），完成即合并 master 并回填状态。
> 验证方式均为仓库内可跑的命令（cargo test / golden / e2e）；需 GUI 桌面会话或跨仓库协调的项目单列在 §5.3 暂缓。

### 5.1 本轮执行（代码级修复，按优先级）

| ID | 来源 | 任务 | 修复点 | 验证 | 分支 | 状态 |
|---|---|---|---|---|---|---|
| A1 | 408 §11 P5-2 | `auto clean` 读 `.am/pac.atom.at` 时 `root` 包裹节点触发 panic | pac.rs targets 循环：拍平 `root` 包裹层 + props 提升；未知 kind 告警跳过（不再 panic） | `cargo test -p auto-man` 214 全绿 + 回归测试 ×2 | plan-fix/408-clean-root | ✅ 已合并 `8f264dd8` |
| A2 | 400 | `is_thin_delegation` 控制流识别不全，含真实逻辑的 handler 体被误判薄委托走模板路线B | **审计初判修正**：Auto 无 while/match 语句（while 脱糖为 For 原本已覆盖）；真实缺口是 `Stmt::Is`/`Stmt::Try`/嵌套 `Block`——补齐 + 递归检查 | `cargo test -p auto-man` 218 全绿 + 测试 ×4 | plan-fix/400-thin-delegation | ✅ 已合并 `a9c64fad` |
| A3 | 398 §14.1 | parser `[][]T` + sibling-handler rewrite 无回归测试 | parser.rs 2 用例（Slice(Slice)/Slice(Tuple)）+ handler_codegen 3 用例（改写/带参/非变体不改写） | `--features ui-iced handler_codegen` 8/8 + 全量 lib 除已知环境失败 | plan-fix/398-regression-tests | ✅ 已合并 `c21eea16` |
| A4 | 406（潜伏 bug） | GET_ELEM 对 `List<bool>`/`List<Value>::Bool` 压裸 i32 1/0 而非 `encode_bool`，下游 `is_bool` 消费者（to_string/EQ 位比较）失效 | engine.rs GET_ELEM 两分支 + native.rs `push_value`（.get/.pop/.remove 路径）共 3 处改 `encode_bool` | e2e 测试 ×4（修复前 3 失败复现 bug） | plan-fix/406-getelem-bool | ✅ 已合并 `c1316a2c` |
| A5 | 346 3c | HTTP 重定向不可用 | **审计初判修正**：redirect shim（Plan 346 已写）从未真正可用——三段链调用解析不到（null）+ serve_async 不认 Response 句柄（200+句柄数字）。修复：`response_redirect` 声明+catalog 3108（2216 已被占用即冲突根因）+ serve_async 句柄识别（i32/i64）；**教训：DashMap 读守卫作用域内 remove 同分片会自死锁** | e2e `e2e_a_redirect_302_with_location`（302+Location+跟随）；全套串行除预存 flake | plan-fix/346-redirect | ✅ 已合并 `e282e70e` |
| A6 | 242 #8 | `infer_type_from_expr` 缺 `Expr::Closure` 分支（闭包推断落 Unknown，靠 task-struct 特判绕过） | rust.rs 补 Closure → `Type::Fn(params, ret)` + golden 004_closure_infer | a2r 套件失败集与 master **逐项一致**（48 个预存环境失败，零回归） | plan-fix/242-closure-infer | ✅ 已合并 `c2bd1d0c` |
| A7 | 366a-3 | `auto test:ui` 一键命令缺失 | crates/auto 新增 `test:ui` 子命令：发现 tests/ 四件套 → 首跑自动 install（bun>pnpm>npm）→ 走项目 test/report 脚本（--headed/--filter 透传，无脚本退回 exec playwright） | 冒烟 ×3（help/缺目录报错/委托执行 DELEGATED-OK） | plan-fix/366-test-ui | ✅ 已合并 `6e7f25f5` |
| A8 | 403 需求1a | 011-calculator 缺 `tests/desktop_mcp.py` + acceptance 契约 | 新增 desktop_mcp.py（经 autoui_press_sequence 真实按键驱动）+ acceptance.atd（T1-T7 契约）；**AUTOUI_MCP_PORT 自动挑空闲端口**（免疫僵尸进程占 9247） | **实机 14/14 全绿**（含 403-F 小数 e2e、科学模式括号） | plan-fix/403-desktop-mcp | ✅ 已合并 `d5ba7314` |

> **本批执行发现（2026-08-20）**：
> 1. **审计代理两处初判有误，已按实测修正**——(a) 400 的"while/match 误判"：Auto 无此二语句，真实缺口是 Is/Try/嵌套 Block；(b) 346 的"redirect 未实施"：shim 已存在但双重的不可用（调用解析 + 服务端识别），比"未实施"更隐蔽。
> 2. **native 编号冲突是隐形地雷**：catalog 中 2216 已被 `response.status_code` 占用，新函数必须用全局唯一新号（本批用 3108）。
> 3. **本机 a2r 套件环境注记**：需 `RUST_MIN_STACK=33554432`（否则栈溢出）；48 个 golden 因工作区内 stdlib 前导渲染差异（`from auto_lang crate` vs `from crate`）预存失败，master 与分支逐项一致。

### 5.2 待立项/需协调（本轮不做）

| ID | 来源 | 任务 | 暂缓原因 |
|---|---|---|---|
| B1 | 405 | 023-realworld 真 token 认证（current_user 空桩） | 应用级功能，需 api/db/playwright 联动改造，建议独立小计划 |
| B2 | 408 §11 P5-4 | 纯 module fn 文件不被 codegen（有 workaround：塞进 widget/store 文件） | 🟢 低优先，方案需先验证 codegen 入口扩展 |
| B3 | 396 | 五条 a2r 根因修复 + 删 auto-ai sed | **§2.1–§2.4 全部 ✅ 根治 + 跨仓闭环**（2026-08-20 三批，`plan-fix/b3-sed-verify` 合并 `efe84664`）：以"禁 sed 全量重转译 + cargo check"实证驱动——§2.2 ReadDir 借迭代（by_value_iter_bindings）、§2.3 read_to_string 借用（三处 dispatch）、§2.4 is_str_slice_var 补查 StrSlice、§2.1 补裸循环变量 clone；**顺带修复 Plan 405 前导回归**（裸 a2r_std 恢复 + api_gen qualify 后处理，48 golden 失败清零→339/339，A6 的"环境差异"误判纠正为 Plan 405 回归）。auto-ai 侧 B/C/D/E sed 段已验证 no-op 删除（auto-ai `64ba3b2`），错误 23→7。**§2.5 未动**（已定位症状：3 段限定单元变体模式 `auto_val.Value.Nil` 渲染为 `auto_val::Value.Nil`，变体段应 `::`；两次插桩未找到真实发射点——非 expr() 模块路径早退、非 is_stmt 默认臂、非 qualify_type_name 回退，待下次以断点级追踪；ai-config sed 保留） | 剩余 §2.5 |
| B4 | 406 剩余 | JMP_IF 魔数、EQ is_bool 臂 | ✅ 2026-08-20 二批完成（`plan-fix/b406-jmpif-bool` 合并 `e58fff15`）：新增 nv_truthy 统一 tag 优先解码，JMP_IF_Z/NZ、AND、OR、NOT 五处弃魔数；**结论：EQ/NE 的 bool==bool raw 位比较本已正确，无需 is_bool 臂**；真整数 -2147483647 与遗留哨兵不可区分为已知限制（注释注明）。测试 ×3 + 全量 3035 过 | 已关闭 |
| B5 | 242 剩余项 | #2 HashMap::from、#10 Redis/SQLite、#15 GPUI、#16 自举、#17 dep cc | 均为大件，独立立项 |
| B6 | 346 剩余 | 服务端 multipart 上传、Rate Limit、Request-ID | 中低优先级，无下游消费者 |
| B7 | 243 Phase 5/6 | VSCode TS 迁移、semantic tokens、CI 启用 | 需 VSCode 侧联调 |
| B8 | 359 | D2 generators 用例、D3 解锁、Phase E 五项、A1/A2 落地页 | 依赖语言特性（Phase E 各 DIV） |
| B9 | A8 副产品 | 013/015 端口硬编码 | ✅ 2026-08-20 二批完成（`plan-fix/b9-mcp-port` 合并 `f0ddb015`）：pick_free_port+AUTOUI_MCP_PORT 移植 + v2 快照 ID 兼容（vnode_）；实测 015 套件 8→10 pass、self-check 3/0/1→6/0/0 | 已关闭 |
| B11 | B3 副产品 | 两处 master 新回归 | ✅ 2026-08-20 四批全部根治（`plan-fix/b11-regressions` 合并 `3f3d0ec3`）：(a) len() 强转按比较对端类型定向（partner_len_cast，Eq/Neq/Lt/Le/Gt/Ge 六运算符）；(b) 实为**四个解析缺口**——`Map<K>` 单参硬 arity 错、`Err(Type.Variant { fields })` 嵌套模式（ResultCover+inner+绑定注册+a2r 发射）、顶层 `Type.Variant { fields }` 模式缺 Dot+`{` 识别与 StructPattern 绑定臂（Plan 165 回归）、枚举结构变体声明不收冒号字段——修复后 tool.at/error.at 历史首次全量转译。**auto-ai 全量重生成归零（23→0，rust/src 全绿提交 auto-ai `156c6c8`）**；解析回归测试 ×4 | 已关闭（衍生：BOM 文件致 parser.rs:272 panic 的健壮性问题待修） |
| B10 | B9 副产品 | (a) 015 autoui_state 把 List<ToolCallRec> 类 notes 渲染为句柄 int（`notes: 4000014`）而非数组——desktop_mcp 的计数断言失效（物化漂移）；(b) 013 的 desktop_mcp.py 是 015 逐字节复制、语义从未适配 013（测 "Notes"/dark_mode 等 013 没有的东西）| 新登记：前者疑为 VM 状态渲染缺口，后者需按 013 实际 app 定制或移除 |

### 5.3 需桌面会话（GUI/实机，无法 headless 补全）

| ID | 来源 | 任务 |
|---|---|---|
| G1 | 412 §6.2/§6.3 | Layout gallery 全页双端截图 ≤1px + scroll/Overlay 交互抽验（§9.2/§9.3 验收） |
| G2 | 413 | IME 实机输入、150% DPI、Linux 复验、TESTING.md 交互清单 |
| G3 | 402 §13.8 | 扫雷连锁展开/数字显示/胜负实机目视确认 |
| G4 | 411 | P1-C Inter 字体内嵌、P2-A① Prism 色板、P2-A④ 表格细节、P2-B MCP 四项（含 Button.content vtree 序列化） |

---

*本文档由 2026-08-20 plans 状态审计生成（6 代理代码级核验，基准 master `f21dc88f`）。实际代码进度以仓库为准。*
