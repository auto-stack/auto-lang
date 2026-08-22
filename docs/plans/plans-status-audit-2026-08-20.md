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
| **396** | a2r 改进（auto-ai 滚动聚合） | §5 蓝图完备 | ✅ **已归档（2026-08-21 finish-plan 复核）**——§2.1–§2.4（B3 批）+ §2.5（1ae3b33c：无括号三段 unit-variant 模式剥模块段，golden 06/009）+ §2.6（283990bc：a2r_std time i64 对齐 stdlib）六条全根治；§2 范围 sed 全部毕业（agent 64ba3b2 / ai-config 31a5304）；auto-ai 三转译 crate 首次同时全绿（client 归零 e05e48d）；a2r golden 340/340 | a2r-std 手抄 stdlib 漂移风险 + tier/SOUL 两 sed（Plan 020/016 归属）登记债务簿 |
| **400** | api_gen a2r body 转译（399 路线A） | Phase 1+2 已合并（d2642943：`is_thin_delegation`/`try_transpile_body`/`AUTO_A2R_BODY` api_gen.rs:1117/1133/1463 + 单测） | Phase 3（多 `back/*.at` + `extern fn` 语言扩展）；Phase 4（auto-musk 全栈端到端验收）；§4.4 `use.rust` 收集 | `is_thin_delegation` 只 match `Stmt::If\|For`（api_gen.rs:1121-1128），**while/match 循环体会被误判为薄委托走模板路线B**——注释与实现不符的逻辑遗漏 |
| **403** | 011 计算器 MCP+grid+多模式 | 需求 1a/1b/1c/2/3 + VM List 基建 + Phase 403-F 浮点修复（9b9fec81）全落地，VM 求值全通过 | **需求 1a 承诺的 `tests/desktop_mcp.py` + acceptance 契约未交付**（`examples/ui/011-calculator/` 无 tests/ 目录） | 文档曾自相矛盾（正文称浮点损坏待办，与顶部 ✅ 冲突，已更正）；验收 checklist 已回填 |
| **405** | 023-realworld（Conduit） | 阶段1 8/8 + 阶段2 14/14 全绿（编辑器遗留已由 plan401/023-editor-fix 修复）；VM 多 store bug 已由 Plan 370 外部解决 | **真正的 token 认证未做且未移交任何计划**——`current_user()` 返回空 User 桩（db.at:56-59），端点无鉴权；store struct 字面量 null 已记 401 技术约定留后续 | 023 规避的 a2r 限制（双路径参数只提取第一个、slug 前端手输避开 String 借用）仍在 |
| **408** | view fn → Vue 组件合成 | ✅ **已归档（2026-08-20 finish-plan 复核：验证重跑全绿）**——P1–P12 + §6.3 auto-musk 试点 + P5-2（audit-A1 修复）全部完成 | P5-4（🟢 纯 module fn 不被 codegen）延期登记债务簿 | 新债流入：auto-musk 029 登记 shadcn Button 映射丢动态 class/title 为本仓债务 |
| **411** | VM 视觉对齐 vue（Home/Button） | P0-A/P0-B/P1-A/gap 废弃/pac.at 窗口 ✅；P1-B toast + P2-A 部分（copy icon/折叠钮）08-15 落地；**P1-C Inter 字体 + P2-A① Prism 色板 + P2-A④ 表格细节 ✅ 2026-08-22 落地**（`plan-fix/g4a-visual` 535b291d/d2748a24，合并 7cf2f4ce）：①色板对齐 vue 实际加载的 prism-tomorrow 主题 + 补 function/operator/boolean 三类别（highlight_code 重写，测试 ×2）；②表头 font-medium+text-muted-foreground 递归注入、行 border-b（1px zinc 分隔条）、单元格 px-4/py-3（Table into_iced 臂改造）；③Inter 三字重 OFL 内嵌 + 5 入口 default_font + Medium 字重正身（测试：ui-iced 3485/8 基线、038/013 实机保持） | 剩余仅 §8.5 gap 兼容分支保留未拆（声明的结构决议）、validator 白名单未加；**P1-C 视觉并排截图核对（vue vs VM 字形）可后补** | §8.5 gap 兼容分支（vue.rs 3 处 + view_builder 8 处）保留未拆、validator 白名单未加（防 AI 再写 gap 属性） |
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
| **398** | VM expose/store sibling 修复 | ✅ **已归档（2026-08-20 finish-plan 复核：5/5 测试重跑绿）**——核心三修 + §14.1 回归测试（audit-A3）+ §14.2（b0434cff）全部完成 | M0.5/M1 auto-shell 下游任务 🟢 延期登记债务簿 |

---

## 2. 📋 纯设计/调研未实施（5 个）

| 计划 | 主题 | 现状 | 备注 |
|---|---|---|---|
| **330** | Agent 友好调试工具链（`auto debug` CLI） | 4 Phase 全未实施（debug.rs/introspection.rs/AUTO_VM_TRACE 均 0 命中） | 自述准确。**建议先与 Plan 199 已交付的 `auto Debug --agent`（JSON 模式断点调试器，main.rs:504）做范围去重**——330 独有价值在 widget state dump / heap-objects / AUTO_VM_TRACE 静态诊断 |
| **332** | `#[derive(ToAtom)]` proc macro | Phase A–E 全未实施（auto-val/auto-lang-macros 无 ToAtom 命中） | Plan 381 serde Deserializer 落地后优先级已降；建议正式关闭或并入债务簿 |
| **386** | AutoUI RenderQueue 分离渲染（未来优化） | Stage 1–3 零实施；启动条件明示"≥3 个 COSMIC app 跑通 Host ② + 内存预算证明"未满足 | ⏸ 自述准确，保持暂缓 |
| **394** | AWAIT_FUTURE 通用 future 架构 | Phase A–D 零实施；§4 触发条件自评均未出现 | draft 自述准确；Plan 349 re-entry yield 为明示的务实替代 |
| **406** | VM 类型系统审计（nanbox 生产者-消费者配对） | ✅ **已关闭（2026-08-20 四批后 finish-plan 复审）**：Phase 2 目标 bug 全部根治——GET_ELEM bool（audit-A4 `c1316a2c`）、JMP_IF 魔数（audit-B4 `e58fff15` nv_truthy）、EQ 复核无需修复、bug 5/6/7 早前顺带解决；Phase 1 全量审计矩阵未产出（驱动已消，🟢 延期登记）。归档 |

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
| B1 | 405 | 023-realworld 真 token 认证（current_user 空桩） | **核心 ✅ 2026-08-22 落地**（`plan-fix/b1-auth` 7f2faaa2）：db 层 Cred 凭证存储+密码校验+递增唯一令牌（登录重铸失效旧 token）+current_user(token) 真解析；api 层 bearer_token（meta JSON/裸 Bearer 双形态）+json_str 提取器+create_article 作者由 token 推导（封冒名）；前端 store 进程内传会话。e2e 六段全绿（真实 023 源码源码手术载入 VM-http）；转译 25/23 fragments 零错。**新发现**：(a) ✅ str-parity 已根治（`plan-fix/str-parity` 9a01ae64）：.substr 对齐 a2r len 语义 + .slice/.sub 独立端语义 shim（catalog 新 id 1524）+ .find 锁定，测试 ×4（含 shim↔a2r_std 对照表）；(b) ✅ **api_gen rust 路径 header 注入已落地（2026-08-22，`plan-fix/b1c-headers` 711ec24d，合并 73e80207）**：`meta str` 参数按 VM push_meta 同约定识别（query/body 双排除，不再误当字段），消费方（db 委托/a2r body 路线）挂 HeaderMap 提取器 + meta_json 绑定（JSON 格式与 VM 逐字节对齐），测试 ×2；(c) playwright UI 登录流与 401 语义化（现 flat 空 User）留后续 | 核心已关闭；401/playwright 留后续 |
| B2 | 408 §11 P5-4 | 纯 module fn 文件不被 codegen（有 workaround：塞进 widget/store 文件） | 🟢 低优先，方案需先验证 codegen 入口扩展 |
| B3 | 396 | 五条 a2r 根因修复 + 删 auto-ai sed | **§2.1–§2.6 全部 ✅ 根治 + 跨仓闭环（2026-08-20 三批 + 2026-08-21 收官）**：§2.2 ReadDir 借迭代（by_value_iter_bindings）、§2.3 read_to_string 借用（三处 dispatch）、§2.4 is_str_slice_var 补查 StrSlice、§2.1 补裸循环变量 clone（efe84664）；§2.5 三段限定 unit-variant 模式（1ae3b33c：`auto_val.Value.Nil` 的 AST 实为 `Dot(TagCover{auto_val,Value}, Nil)`——use.rust 模块名走 tag_cover 两段吞并、`.Nil` 残留外层 Dot，a2r 由 Cover 裸分支+字段访问拼出非法 `auto_val::Value.Nil`；两次插桩扑空因值位 `::` 分支要求 Ident/Bina/Dot 基座；修在 parser `is_branch_cond_expr_inner` 补无括号三段转换剥模块段，golden 06/009）；§2.6 新登（283990bc）：a2r_std time i32 手抄滞后 vs stdlib i64 声明，恢复 i64 后 client 归零。**auto-ai 三转译 crate 首次同时全绿**；§2 范围 sed 全部毕业（agent B/C/D/E：64ba3b2；ai-config unit-variant：31a5304）；计划已归档（未竟项登记债务簿：a2r-std 手抄漂移风险、tier/SOUL sed 属 Plan 020/016） | 已关闭 |
| B4 | 406 剩余 | JMP_IF 魔数、EQ is_bool 臂 | ✅ 2026-08-20 二批完成（`plan-fix/b406-jmpif-bool` 合并 `e58fff15`）：新增 nv_truthy 统一 tag 优先解码，JMP_IF_Z/NZ、AND、OR、NOT 五处弃魔数；**结论：EQ/NE 的 bool==bool raw 位比较本已正确，无需 is_bool 臂**；真整数 -2147483647 与遗留哨兵不可区分为已知限制（注释注明）。测试 ×3 + 全量 3035 过 | 已关闭 |
| B5 | 242 剩余项 | #2 HashMap::from、#10 Redis/SQLite、#15 GPUI、#16 自举、#17 dep cc | **✅ 2026-08-22 拆粒度立项完成**：Plan 415（`docs/plans/415-a2r-remaining-big-items.md`）——五子项 A-E 各自独立 worktree/验收矩阵，GPUI/自举前置 spike 决策点已注明；#8 闭包推断已由 audit-A6 修复（c2bd1d0c）移出范围 | 已立项（415） |
| B6 | 346 剩余 | 服务端 multipart 上传、Rate Limit、Request-ID | **全案 ✅ 2026-08-22 关闭**：Request-ID + Rate Limit（前批合并，e2e ×3）；**multipart 上传 ✅**（`plan-fix/b6-multipart` 7f26c50a）：RFC 2046 简化解析（二进制安全）+ Content-Length 字节续读（突破 8KB 初读，legacy 字符串路径零改动）+ AUTO_UPLOAD_DIR 落盘（basename 清洗+防同名）+ handler body 槽接 multipart JSON（fields/files 元数据）；拦路虎=content_type 头小写化毁边界大小写（保留原始副本）；e2e：20KB 二进制（0xFF/近边界序列/>8KB 续读）字段+元数据+落盘字节全断言。**346 生产化阶段三件套（5a/5e/#12）全部落地** | 已关闭 |
| B7 | 243 Phase 5/6 | VSCode TS 迁移、semantic tokens、CI 启用 | **✅ 2026-08-22 拆粒度立项完成**：Plan 416（`docs/plans/416-lsp-vscode-phase5-6.md`）——5-A TS 纯迁移/5-B semantic tokens（唯一 VSCode 实机点，最低配置=本机 F5）/5-C 补全数据源 + 6-A CI 解红/6-B 集成测试扩容；顺序编排为"无联调项先行" | 已立项（416） |
| B8 | 359 | D2 generators 用例、D3 解锁、Phase E 五项、A1/A2 落地页 | **✅ 2026-08-22 拆粒度立项完成**：Plan 417（`docs/plans/417-script-rollout-residuals.md`）——Phase E 五 DIV 按依赖排序（E1 CHAR-AT 最小 → E5 HTTP-LANG 同时解锁 D3）；D2 generators 不依赖 Phase E 可先行；165 checkbox 回填并入 finish-plan 收尾 | 已立项（417） |
| B9 | A8 副产品 | 013/015 端口硬编码 | ✅ 2026-08-20 二批完成（`plan-fix/b9-mcp-port` 合并 `f0ddb015`）：pick_free_port+AUTOUI_MCP_PORT 移植 + v2 快照 ID 兼容（vnode_）；实测 015 套件 8→10 pass、self-check 3/0/1→6/0/0 | 已关闭 |
| B11 | B3 副产品 | 两处 master 新回归 | ✅ 2026-08-20 四批全部根治（`plan-fix/b11-regressions` 合并 `3f3d0ec3`）：(a) len() 强转按比较对端类型定向（partner_len_cast，Eq/Neq/Lt/Le/Gt/Ge 六运算符）；(b) 实为**四个解析缺口**——`Map<K>` 单参硬 arity 错、`Err(Type.Variant { fields })` 嵌套模式（ResultCover+inner+绑定注册+a2r 发射）、顶层 `Type.Variant { fields }` 模式缺 Dot+`{` 识别与 StructPattern 绑定臂（Plan 165 回归）、枚举结构变体声明不收冒号字段——修复后 tool.at/error.at 历史首次全量转译。**auto-ai 全量重生成归零（23→0，rust/src 全绿提交 auto-ai `156c6c8`）**；解析回归测试 ×4 | 已关闭（衍生 BOM panic 已修：1fb96eac——lexer 跳过 U+FEFF，带 BOM 的 .at 不再在 parser.rs:272 崩溃，2026-08-21） |
| B12 | B10(b) 副产品 | 013-todo VM 缺口（2026-08-22 四轮全收官）：**(i) ✅ 幻影实参帧错位**（695b4736：handler 参数数门控）；**(ii) ✅ renderer 内嵌史前 Todo 应用双重执行**（41c9ac39：legacy 臂 `!has_handler` 门控，backtrace 定案）；**(a) ✅ Init 计数双缺陷**（`plan-fix/b12a-count` a7fe6e9d）：(a1) GET_FIELD 的 GenericInstanceData 分支 bool 压裸 i32 而 ObjectData 分支早已 encode_bool（Plan 402 §13.10；A4/GET_ELEM 同族）→ `== false` 永假；(a2) iced boot 在 fire_init 后无条件写 active_count/todo_count=0 碾碎 Init 结果（史前机制遗留，撤销）。**实机 013 desktop_mcp 12→16 过/0 败**：T2 active_count is 3 首次 PASS、T6 添加流三连、T7 toggle-all 真语义。**(iii) ✅ 子组件事件剥离已根治（2026-08-22，`plan-fix/dgap4-ctrack` 0f6a3eb7，合并 1fd16ea7）**：三连根因——①render_child_widget 走无跟踪 plain build（子树样式/事件不进 BuildProbe→快照全盲，而 handler 实际可派发）；②`text f"..."` 的 Expr::FStr 无求值臂（空串→节点按视觉空过滤→兄弟左移错位，App 层同样复现）；③tracked row 条件拼接把多节点体记同一路径。修复后 **013 实机 22 过/0 败/0 skip**（T3-T5 真点击+真实状态断言） | (i)(ii)(iii) 全关闭 |
| B10 | B9 副产品 | (a) ✅ **2026-08-21 已修**（`plan-fix/b10a-state-list` 合并见 git log）：`read_all_state_materialized` 只物化 `Value::VmRef`，漏了编译后 `List<T>.new`/`[...]` 字面量存的裸 `Value::Int(句柄)`（Plan 289 路径）——Int 经 `vmref_to_vec` 同一探测路径物化（真 Int 不受影响）；015 实机 desktop_mcp **13 过/0 败/1 跳过**（notes 计数断言复活，B9 时代 10 pass）；单测锁定（ui-iced）。(b) ✅ **2026-08-21 已修**（`plan-fix/b10b-todo-mcp` 合并见 git log）：013 desktop_mcp.py 从 015 字节副本重写为 TodoMVC 真实语义（T1 结构/T2 种子/T6 type+submit 添加/T7 toggle-all，断言全部实机校准），**12 过/0 败/6 跳过** + self-check 6/0/0；SKIP 均带根因注记，并实证两处新缺口登记 B12 | 已关闭 |

### 5.3 需桌面会话（GUI/实机，无法 headless 补全）

| ID | 来源 | 任务 |
|---|---|---|
| G1 | 412 §6.2/§6.3 | Layout gallery 全页双端截图 ≤1px + scroll/Overlay 交互抽验（§9.2/§9.3 验收） |
| G2 | 413 | IME 实机输入、150% DPI、Linux 复验、TESTING.md 交互清单 |
| G3 | 402 §13.8 | **✅ 2026-08-22 已关闭**（`plan-fix/g3-mines` e8eb3624）：实机探针首跑即崩——renderer highlight_code 的 `code[i..i+1]` 按字节切多字节标签（"⏱ 0s"）→ char boundary panic，038 VM 模式从未能启动；修复后 desktop_mcp 21 过/0 败：洪水填充（29 揭开>1 判别）/16 数字格显示/踩雷→lost+全雷 💣/Reset 复位；LCG 确定性布局两运行逐位一致（Python 复刻第 6 雷处分叉留档）；附带实证 App 自有 for 循环的 81 参数化事件全存活（D-GAP-4 仅子组件） |
| G4 | 411 | **P2-B Button.content vtree ✅ 2026-08-22 已修**（`plan-fix/g4-vtree` f7658c45）：extract_children/_ref 补 Button content 臂（两变体子序锁定），R3 "卡片丢 desc" 误判根源关闭；测试：4 节点子树序列化断言。**P2-B 其余三项 ✅ 2026-08-22 已修**（`plan-fix/g4b-mcp` 6fbcbe40）：autoui_check 对齐（render_support 表按 builder 实际分支重写，60 假阳性清零）+ 快照过滤 Empty 占位 + 截图零尺寸守卫盘点（已有，renderer.rs:6003）。剩余：P1-C Inter 字体内嵌、P2-A① Prism 色板、P2-A④ 表格细节、P2-B #1 vtree layout 回填——**2026-08-22 专项调查完成、实施撤回**（探针实证，worktree 已清）：①机制链本已存在（Plan 282/314：MCP 激活→needs_bounds→LayoutCollector→backfill_bounds→快照），但实收恒 2 key——根因=builder 记 AuraNode 结构路径而 conversion 走 for 展开 AbstractView，首循环后路径全漂移，容器 aura id 几乎全 miss；②试验路径哈希 id（vnode_&lt;id_from_path&gt;，与 vtree 同源，backfill 直解析免 map）覆盖率 1→97/178 节点，机制可行；③**卡点=iced operation 嵌套坐标语义**：根容器得绝对值 (0,0,370,506) ✓，但嵌套容器/按钮位出负坐标半尺寸值（-16,-8,32,16）——父相对 vs 绝对未定，需祖先链累加设计+逐 kind 验证，不达标不合并（全绿纪律）；④按钮位出现 85 bbox 值来源待查（wrap_debug mouse_area 或内联解析误归）。**2026-08-22 二轮突破（坐标语义卡点已解，实施仍撤回）**：①发现 vtree 里的 97 个 bbox 其实来自**样式估算器**（非 collector）——此前"负坐标半尺寸"疑云系估算值混入，非 iced 真值；②LayoutCollector 的 aura_id_str 只认 aura_ 前缀 → vnode id 全被丢弃（0 key），补双前缀识别后 collector 交出**真值且为绝对坐标**：root (0,0,370,506)、棋盘容器 288×288（=9×32）在 1280 宽窗居中 x=496 ✓——**坐标语义判定=绝对直用，无需祖先累加**；③残余缺口收窄为**转换路径覆盖**：8930 次 container 访问中仅 2 节点带 id（root+grid）——中间容器未走 render_dynamic_view 的带 path 臂（或 trait 路径分流），需查 Column/Row 臂实际命中情况或给 IntoIcedElement 路径补 thread-local path；④**✅ 2026-08-22 三轮收官落地**（`plan-fix/g4e-bbox` 30fc2d08）：配方全实施——vnode 路径哈希 id ×5 位点 + **apply_column/row_style plain 路径丢 id 补全**（无样式分支直接 col.padding(pd).into() 丢弃 widget_id——真正缺口；透明 container 包裹零布局扰动，038/013 实机 21/0、16/0 实证）+ 双前缀 collector + inspector 直解析。实机：容器骨架真实绝对坐标全出（root→主列→信息行→难度行→棋盘 288×288 居中，层层合理）；负坐标疑云定位为 box prop 声明盒合成（与真实 bbox 两个 prop）；叶子 bbox 按 §13.10 保持 F12 门控（点击 bug 设计决议）。**视觉像素三项 ✅ 2026-08-22 收官（`plan-fix/g4a-visual` 535b291d+d2748a24，合并 7cf2f4ce）**：P1-C Inter 三字重（v4.1 静态，OFL 许可随附）include_bytes 内嵌 + 5 个 iced 入口 default_font + font_weight_to_iced family 钉住（Medium 字重恢复正身）；P2-A① highlight_code 色板换 prism-tomorrow（vue main.ts 实际加载主题）+ function/operator/boolean 三类别补齐；P2-A④ 表头 medium/muted 递归注入 + 行 border-b 分隔条 + 单元格 px-4/py-3；038/013 实机回归保持（emoji/中文走 cosmic-text 回退正常）。剩余：Inter 与 vue 并排字形截图人工核对可后补（非阻塞） |

---

*本文档由 2026-08-20 plans 状态审计生成（6 代理代码级核验，基准 master `f21dc88f`）。实际代码进度以仓库为准。*
