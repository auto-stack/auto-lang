---
plan_id: PLAN-1072
status: execution_done
feature_name: AutoUI Vue 与生产 TS/JS 接管
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19
plan_revision: 1
current_step: 3
total_steps: 5
depends_on: [PLAN-980, PLAN-981, PLAN-976, PLAN-978]
supersedes_spec_components: []
new_spec_components: [docs/specs/ace-migration/m2-autoui.md#15]
touched_goals: [GOAL-002]
---

# [PLAN-1072] AutoUI Vue 与生产 TS/JS 接管

## 0. 变更摘要

2026-11-19，用户批准按交付拆分 980/981。本计划接手原义务及可复用输入，按交付来源身份匹配后计验收。

## 1. 目标

Vue 生产模式正式收据，含完整真实旧语料迁移、合法 TS/JS 模块闭包、浏览器行为和错误恢复。可提前给 Tauri 交已复核的 Vue 生成接口。

## 2. 架构方案

auto-ui 主责，auto-lang 与 AutoMan 按接口供受检语义、provider 和真 CLI。复用 ACE 受检前门、共享 Schema、现有宿主和生成入口。

## 3. 技术栈

沿原合同冻结 Windows 工具链、SDK、资源路径及产物身份。真实窗口、GPU、浏览器、外仓或离线消费按实际用例运行。

## 4. 需求分析与背景调查

取材 [AutoLang 全局总览](../specs/overview.md) 的 UI/Web 生态，以及 [980 原合同](980-ace-m2-autoui.md#0-变更摘要)。原技术合同、输入、测试分母和负控按归属表接收。

## 5. 详细设计

**前置**。消费 980 DIALECT/NATIVE 实际使用的已交接口、981 EVENT-B 对应项、976/978。010 的列表字段按 1071 所交精确语义接入；场景不使用该字段时可独立推进。U0 联合收据 `2c9c5f228` 按原身份接收。

**接手范围**。接 980 B-WEB、完整生产 TS/JS 的 Vue 子面和 PHASE-LAMBDA-IF。包括 005、010、AutoNotes 跨文件件、quick_switcher/use、组件包和实际工程的旧新映射；已封存内容按原来源复用。

**PHASE-LAMBDA-IF 交付状态注记（2026-11-19 跨会话协调）**：该刀主体已与拆分裁定**同日在 980 名下交付落主**——lang `347540437` + auto-ui `05ed07c3`（T-LIF-1..6 全交、AC-LIF-01..06 闭、规范沉淀 `m2-autoui.md` §13.12；收据=980 §9「lambda-if 刀交付」）。**F-R21-2 亦已闭**（981 r21 指派项，同日 980 名下：ace_package_sfc 脚本重放落 `handler_stmts` 消费门=具名拒 `source.age.sfc-handler-stmt`，auto-ui `7b8456e5`；收据=980 §9「F-R21-2 收口」——981 T-BASE-ONIF 可据此复核）。1072 承接的**残余**：AC-LIF-07 的 1072 侧记账、原 §10-1 owned-wrap 统一决策、分支内调用形→通道调用承载（#6p 谱系）、005 整例 use/style/design 残墙、`n_assigns` 平铺计数的 census 漂移记账——不重做已交主体。

每组开工确认精确输入源、对应原用例、接口 owner 及身份。共享语义和 ABI 由原 owner 串行写入，变化时重验实际受影响格。已有阶段收据保留其验收边界。

### 规范增量

- **规范目标**：`docs/specs/ace-migration/m2-autoui.md`
  - 更新顶部 `> Status:` 摘要，记录 PLAN-1072 完成 Vue 与生产 TS/JS 接管交付。
  - 新增 `## 15. Vue 与生产 TS/JS 接管（PLAN-1072，2026-11-19）`：沉淀组件面 `style` 惰性替换路径、`.name`/裸名并容的 `expose` 语法对齐、`view-for` + 仅视图数据字段（`ComponentViewDataField`）+ 条件样式值、`marshal_str_boundary`（含引号字面量保护）统一 str 边界、冷生成 `write_ace_vue_rt` 修复、`setup/watch/expose/ext_imports/unmount` 与错误保旧恢复边界、旧新映射表及残余记账。
- **债务账本目标**：`docs/plans/KNOWN-DEBT-AND-RISKS.md`
  - 追加 `#6p` 分支内调用形（具名缺席）与 `ace_package_sfc` 多分支控制流重放边界的残余债务条（由后续包级控制流计划接手）。

## 6. 测试设计

与 982 联合验一个原样 Vue UI 工程的 `auto gen`（Vue/TS 与运行时输出）、`auto build`（实际构建）、`auto run`（真实浏览器、事件与状态）。1072 交生成/运行时及行为依据，982 验公开 CLI 与工具派发。U0 只作为既有 Run 子面按原身份复用。

原 V-B-WEB、V-EVT-WEB-CONSUME、V-TS-CONSUMER-UI Vue 格、AC/V-LIF 全部及正式语料 census。核实际 watch/setup/expose/ext_imports/style、绑定、更新/卸载、错误保旧、a2x_rt/TS 与真浏览器结果。旧发射或只观测生成文本不计通过。

沿原入口记录源码/目标/SDK、命令、子终态、独立 oracle、资源、条件排除与恢复。材料归本计划的统一目录。

## 7. 验收标准

| ID | 标准 |
|---|---|
| AC-01 | §5 对应原分母齐，输入、原独立预期、负控及实际依赖可追溯。 |
| AC-02 | §6 真产品入口及全部适用原检查通过，故障、恢复、资源和来源身份对证。 |
| AC-03 | 同树冻结独立复审，正式交付 source/SDK/ABI 与验收证据，消费者可按约定接口接收。 |

## 8. 执行步骤

- [x] **T-01｜work｜冻结语料**。复用 `auto-ui/crates/auto-ui-runtime/tests/ace_ts_provider.rs` 与 980 真实语料 census，定位旧源及独立期望。

- [x] **T-02｜work｜完成受检生成**。在既有 `ace_ts_provider.rs`、SFC 和 tools 产品入口补完 005、010 及分母内的缺口，复用 980 的生产 SFC 与 `a2x_rt.mjs`。

- [x] **T-03｜work｜验收 Vue 产品链**。跑原 V-B-WEB 及适用产品用例，从 `auto run --render vue` 到 pnpm/Vite/真实浏览器，对照页面事件、退出与清理。

- [ ] **T-04｜review｜独立复审**。逐项核 AC-01..03、原任务和实际反例，修复发现后复验。

- [ ] **T-05｜merge｜正式交付**。按 owner 合入、沉淀规范，记录可消费版本，按项目规约归档及清理。

## 9. 复审记录

2026-11-19 stage: new，规划与交接基面 78b375bf4，实施与复审待执行。

2026-11-19 stage: goal（/goal 完成计划1072；授权链=work→review→bounded repair→merge 全链，§4 首记）| plan_id: PLAN-1072 | plan_revision: r1 | round: 1 | 建组：`D:/autostack/.wt/lang-1072/{auto-lang,auto-ui}` 双 `plan-1072-dev`（基面 lang `e81c27e9f` / ui `d2e7d7a4`，均=各仓 master）；`plan-artifacts/plan-1072/` 已建。主检出预检：**auto-ui master 带未提交 WIP**（docs/design 修改 + examples/ui/stylekit、packages/themes `.at` 删除，与 auto-lang 主检出 packages/blueprints 删除同型——疑似某迁移会话遗留）；不阻塞 worktree 实施，merge 落地前需 owner 路由，已面呈用户。

2026-11-19 stage: work（T-01 冻结语料）| plan_id: PLAN-1072 | plan_revision: r1 | outcome: pass | worktree: lang-1072 双仓 | task_ids: T-01 | evidence: 静态勘定（探针零运行），收据如下。

**分母与输入身份（冻结）**。①永久用例面：`auto-ui/crates/auto-ui-runtime/tests/ace_ts_provider.rs` 27 例（census v2 冻结表 9 行 + 全量双轨普查 83 语料聚合 (75,8,8,22) + param/channel/lambda-if/browser 档）；产品接线面 `auto-ui-tools/tests/plan980_ace_vue_tool_wiring.rs`、010 档 `plan980_010_form_short.rs`（现状=0/6 具名拒）、jg `plan980_bweb_jg_watch.rs`（use 块 7/7 缺席=981 合同）、notes `plan980_notes_fp_editor.rs`、包面 `plan980_package_sfc.rs` 8/8。②真实语料：`examples/ui/005-login/src/front/app.at`（980 census sha `446da5dc1bb68b6a`·134 行）与 `010-contact-form/src/front/app.at`（`15a632b4cda27cd2`·177 行）；AutoNotes 跨文件件=`auto-notes/src/front/components/{fp_editor.at(106 行),package.at(fpack),pac.at(notesfp)}`+宿主 `app.at:84 use { package: official }`；quick_switcher=`auto-notes/vendor/auto-down/jade-garden/front/auto/src/front/quick_switcher.at`（219 行，`use {` TS-ext 组合式块=981 合同锁组件面具名拒）。③规范：`docs/specs/ace-migration/m2-autoui.md` §13.12/13.13（缺席 taxonomy ①-⑦）/13.15。

**005/010 墙勘定（对照 002 实证隔离）**。002-counter（同含 `use themes.themes:` 点分 use + design if-else）整源过 lane=U0 收据实证 ⇒ 该两形态**非墙**（980 lambda-if 收据"005 仍压 use/style/design 三墙"的 use/design 归因据此订正）。005 实墙=**文件级 `style` 声明 ×3（全字面量）+view 内 `style:` ident 引用 ×6**；010 实墙=**①文件级 `style field_col`（字面量）②`var faqs = [{q,a}×4]` 无类型列表字段（组件面 model 文法+field_type 域双拒）③view `for i, f in .faqs`（age-view-item 无 for 规则）**；010 handler 面（Submit if-else/SetTheme if 体/SetAccent）on-if 刀已就绪。1071 T-02 已交 010 边界裁定：**列表字段=仅视图数据源（v-for 渲染、非可变 state 通道、不进 AceStateValue）**，AGE 列表语义归 collections owner。

**路由决策（依 981 r18 会签原文，非新裁定）**。981 会签将组件面 style 锁在 const 路径三条件后（组件面无 const-item/resolve 拒裸 const/无 ModuleGlobalsPlan），但**同会签 (c)3 已裁定投影期字面量替换为两面通用语义**（"改为投影期把 style 名替换为字符串字面量"）且明示"若能端到端再单提一步开 age-component"。1072 取**替换路径而非 const 路径**：组件面 style-item=惰性视图数据（不产 Item::Const/Fn、不进 resolve/全局计划——三条件不触发、无"能解析跑不了"半面）；参数化/默认值/跨模块/带参调用沿包面同名拒（style-defaults/style-call）。grammar-id 重基线=age-component 一值（style-item 规则 profile 增 age-component；包面规则集不变）；13 邻居逐字节保持，冻结面同提交更新注明前值（沿 #6m/#6o/view-if/on-if 先例）。UI 侧 `generate_sfcs_with_lane_product` 现以裸 VueGenerator 丢文件级 style（`style:` ident 落 `:style="field_label"`=运行时 undefined）——修法=生成前经 `design_tokens::recipe` 公共 API 注册本文件配方（auto-ui-runtime 已依赖 auto-ui-compiler；产品道四调用点现为线程局部余温隐式生效，独立入口（auto run CLI）为空注册表），与旧管线同源展开。

**T-02 执行序**：刀A=组件面 style（lang grammar/投影惰性表+替换/UI 配方注册显式化+005 产品翻转+浏览器档）；刀B=010（lang view-for 结构承载+无类型列表字段视图数据面/UI v-for 对账+010 产品翻转）；残余记账（n_assigns census、AC-LIF-07、owned-wrap 决策、#6p 分支内调用形谱系归档）。quick_switcher/AutoNotes app.at 整源/`use { package }` 按既名墙保留（981 合同/域决策），旧新映射表随 T-02 收据落档。

next_responsible_role: work（T-02 刀A 起）

2026-11-19 stage: work（T-02 刀A：组件面 style + type 键——005 整例翻转）| plan_id: PLAN-1072 | plan_revision: r1 | outcome: pass（切片交付）| worktree: lang-1072 双仓 @ plan-1072-dev | task_ids: T-02（刀A）| code: lang `plan-1072-dev`（待提交）+ ui `plan-1072-dev`（待提交）。

**交付。** ①文法：style-head/style-item 开 age-component（981 r18 会签 (c)3 投影期字面量替换路径——**非** const 面，组件面三锁条件不触发；params 步骤包面门控=文法层边界）+ age-event-key 增 type-kw 键臂（005 `type: "password"`——type 是全局 token；红相=修复前错误经恢复链归因到 λ 收尾 `}`）。②投影：`apply_component_face_styles`（惰性视图数据表+`replace_style_in_items` 复用+reject_unknown 限 style 键——空表也走查=未声明名/导入名引用 fail-closed 具名拒）；重名沿 (c)1 同码、带参调用/不可折叠值沿 style-call 族。③UI：`generate_sfc_with_lane(_product)` 两入口 prepare→parse→replay 显式注册 style 配方（修复前独立入口空注册表——`style: NAME` 落 `:style="NAME"`=运行时 undefined；产品道四调用点靠线程局部余温隐式生效）。④**005 整例翻转**：6 handler 全派生（Submit 双 if-else 结构化/UseAuto/UseSolarized/ToggleMode/λ-if×2）、absent=[]；SFC 配方展开为字面量 class（field_label→"text-sm font-medium text-foreground mt-6"）。

**验证。** lang 永久用例 10（plan1072_component_style：字面量/复制/重名/导入拒/调用拒/未知拒/params 包面/普通标识符/type 键）；grammar-id：age-component `61799f12…→4755265c…→c8a7479d…`（style+type 两刀）、age-package `93cd6814…→c337685c…`（type 刀）、13 邻居逐字节不变。UI 永久用例 3（plan1072_vue_web）。门禁：ace-source **PASS**；ace-av **1425/1428**——3 红全归因（host_console×2=预存 stash 实证在案 982 谱系、kv_zip=隔离重放绿并发归因）；UI：ace_ts_provider 串行 **27/27**（census 冻结聚合 (75,8,8,22) 稳定）、package_sfc **8/8**、U0 002 **2/2**、notes fp **1/1**、vue_capabilities 77+5=主检出基线同红（chart 族预存）。

2026-11-19 stage: work（T-02 刀B：view-for + 仅视图数据字段 + 条件样式值——010 整例翻转）| plan_id: PLAN-1072 | plan_revision: r1 | outcome: pass（切片交付）| worktree: lang-1072 双仓 @ plan-1072-dev | task_ids: T-02（刀B）。

**交付。** ①文法：`age-view-for`（双名形 `for i, f in .expr`，两 AGE 面）+ `age-view-data-lit` 惰性数据文法族（模型无类型 alt 专属——记录/标量条目+尾随逗号；**不开受检表达式面**，数组/对象字面量属 aggregates 域）+ `age-entry-value` 增 if-expr 臂（条件样式值）。②结构：`ComponentViewItem::For`（结构承载惰性语义——循环体文本叶**不**进 texts：循环变量无绑定域；`view_contains_for`=渲染腿判据）+ `ComponentViewDataField`（无类型列表字段=**仅视图数据**，1071 边界：原始字面量文本承载、不进 fields/state/scenario；handler 写它=未声明字段具名拒）+ `ComponentEntryValue::If`（design 三段同形）。③遍历九处臂（lambda 收集/绑定验证/style 替换/manifest/事件绑定/计数/观察——各按其语义递归或惰性）。④渲染腿 fail-closed：native/desktop 装载点 `view-for-unsupported` 具名拒（镜像 view-if）；包面 SFC `sfc-view-for` 具名拒（for+视图数据=组件面形，包面保持标量边界）。⑤UI：`__bind_*` 方言合成件出 lane 规格集（值绑定由模板 `:value` 承载——红相实证：进 ops=整 lane 编译失败连带真 handler 全体缺席）。**010 整例翻转**：3 handler 全派生（Submit/SetTheme 双 bare-if/SetAccent 实读参）、absent=[]；产物 v-for over faqs + field_col 展开。

**验证。** lang 永久用例 +5（⑪view-data 字面量承载 ⑫for 结构惰性（含 texts 不平铺判据）⑬条件样式值 ⑭view-data 写具名拒 ⑮无类型标量=文法层拒）；grammar-id：age-component `c8a7479d…→42afe2cb…`、age-package `c337685c…→91c53cde…`（第三次重基线，13 邻居逐字节不变）。UI：plan1072 **3/3**（010 翻转态断言）、010 工具档键下生成成功（a2x_str_eq/a2x_rt 标记+v-for/faqs 产物断言——历史 0/6 具名拒口径收档）、census **27/27** 稳定、package_sfc **8/8**、U0 002 **2/2**+product **2/2**、notes fp **1/1**。

**残余（T-02 记账项）。** quick_switcher/AutoNotes app.at 整源按既名墙保留（981 合同 use 块/`use { package }` 域决策）；n_assigns census 漂移记账、AC-LIF-07 记账、owned-wrap 统一决策、#6p 分支内调用形谱系归档——随 T-03/收尾轮落档。

2026-11-19 stage: work（刀C：str 边界统一——原 980 §10-1 owned-wrap 统一决策**收口** + T-03 浏览器档）| plan_id: PLAN-1072 | plan_revision: r1 | outcome: pass | worktree: lang-1072 双仓 | task_ids: T-02（owned-wrap 决策）/T-03（浏览器档）。

**裁定（§10-1 收口，证据驱动）。** 宿主（Vue ref/SFC 形参）以**普通 JS 字符串**承载 str 状态与形参；rt 文本值（Uint8Array/A2xOwnedStr）只存在于表达式求值内部。宿主映射层 `marshal_str_boundary`：读=`a2x_str_lit` 包裹、写=解码 `a2x_str_utf8`+释放 owned 副本、owned-swap 快照行与旧值释放块删（宿主 GC 拥有字符串）。红相（005 产品浏览器档实证）：`a2x_str_ne(email.value,…)` trap "text value is not UTF-8 bytes"、`a2x_str_free(旧串)` trap "cannot free a non-text value"——lang 侧 rt 语义发射对 rt 世界正确，差异只在宿主边界（本层承载）。与 handler 面的差异归档：状态函数道/检查函数道/λ 道三道统一经本层（此前 981 检查函数道在宿主的 str 形参同样未执行实证——005 首次真执行）。

**T-03 浏览器档（V-B-WEB 真浏览器/事件/绑定 vs 源 expected）。** 新 rig `test_005_login.mjs`（七判据：初始无错/空提交双必填/输入 email 清一错/输入密码全清/主题三按钮/已填提交零错/零 page error）+ 005/010 产品浏览器档 driver（lane 产物原样落盘零注入）：**005 七判据全绿、010 同 legacy 行为 oracle（三字段输入绑定+提交回执+零 page error）在受检 lane 产物全绿**。五处既有断言具名修订（marshal 包裹形：param_environment/on_if/wiring×3）。

**门禁。** plan1072 **5/5**（含浏览器档）、ace_ts_provider 串行 **27/27**、package_sfc **8/8**、U0 002 **2/2**、notes fp **1/1**、tool wiring **1/1**、010 form **1/1**。

2026-11-19 stage: work（T-03 剩余闭环 + 旧新映射表 + 残余四项记账）| plan_id: PLAN-1072 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/lang-1072/{auto-lang,auto-ui} @ plan-1072-dev | task_ids: T-02, T-03 | commits: lang `48738ff32a60fa56578da7ecee073d9bf8384917`（基于 `5492b3ca7`, `041148cc1`）, ui `8210ef5bc0b1096d1184503373094d49d46ae9e1`（基于 `ec7342c1`, `1489d0cc`, `b9417486`） | siblings: auto-down `72b097006f34db3a871449f10e213820d51abef6`, auto-man `f461960f5db9bc99267f93541dc7c5bdeedfa048`, auto-web `3d328a3561935ac390916cfa356603bc30135e03`。

**T-03 剩余修复与新增覆盖。** ①`auto-lang`（`48738ff32`）：`experimental/ace/grammars/auto-source.at` 对齐 `age-expose-member` / `age-expose-item` / `age-expose-name-tail`（`profiles: [age-component, age-package]`）并容 `.name` 与裸名及可选逗号；`AgeComponent` `42afe2cb…→fad57c5f81b4ee3c…`、`AgePackage` `91c53cde…→06809c843671b91f…`，13 邻居 grammar-id 逐字节不变（`plan981_grammar_scope` 4/4）。②`auto-ui`（`8210ef5b`）：`crates/auto-ui-tools/src/vue.rs` 在 `from_workspace_reads` `components/*.at` 循环（L4496）继承 `ace_vue_rt` 并在 `VueProject::generate`（L5336）补调 `self.write_ace_vue_rt()?;`（修复 `--gen-only` 冷生成漏写 `src/a2x_rt.mjs`）；`ace_vue_script.rs::wrap_host_reads`（L229）跟踪引号与反斜杠转义避免误包裹 `"..."`/`'...'` 内文本；修订 `plan980_u0_002_product_sfc.rs` 补入刀 C 引入的 `a2x_str_utf8`；修订 `ace_ts_provider.rs::p980_web_param_browser_mount`（E2E 档原预刀C手写 `rt_inline` 假桩缺 `a2x_str_lit`/`a2x_str_utf8`）改用官方 `ace_a2x_ts::script_runtime_body(ScriptLang::JavaScript, &[])` 实跑跨边界读写；新增 `test_p1072_lifecycle_watch_expose.mjs`、`plan1072_vue_web.rs` 三例及 `plan1072_vue_error_recovery.rs` 验证 `setup`（严格单次执行 + `ext_imports`）/ `watch`（触发与卸载停止监听）/ `expose`（`vm.Inc()` 与 dot 前缀）/ `style` 展开 / 冷生成 `@vue/compiler-sfc`+`esbuild` / 增量 non-strict 与 strict 及 lane 拒绝的错误保旧与恢复。

**旧新映射表（真实语料与基准用例对账）。**

| 语料/入口 | 旧源/基准 | 新受检路径与产物 | 验收结果与边界 |
|---|---|---|---|
| `002-counter` (U0) | `examples/ui/002-counter` | `ace_vue_script` lane 全派生 SFC + 官方 `a2x_rt.mjs` | `plan980_u0_002_sfc` 2/2、`plan980_u0_002_product_sfc` 2/2（含 `a2x_str_utf8`）通过 |
| `005-login` | `446da5dc1bb68b6a`（134L） | ACE lane 6/6 handlers 派生（`absent=[]`）+ 配方 `style` 展开 + `test_005_login.mjs` + `gen_vue_project` | `plan1072_vue_web` + `plan1072_vue_error_recovery`：7 判据实浏览器、冷生成 SFC+esbuild、保旧恢复全通过 |
| `010-contact-form` | `15a632b4cda27cd2`（177L） | `ComponentViewDataField` + `ComponentViewItem::For` + `__bind_*` 出规，3/3 handlers（`absent=[]`） | `plan1072_vue_web` + `plan980_010_form_short` + `plan1072_vue_error_recovery` 全绿；`faqs` 维持 PLAN-1071 仅视图数据边界 |
| `AutoNotes fp_editor` vs `app.at` | `fp_editor.at`（106L） / `app.at:84` | `fp_editor.at` 全受检 lane；`app.at:84 use { package: official }` 锁包域 | `plan980_notes_fp_editor` 1/1 通过；宿主 `use { package }` 按 981 合同锁域边界 |
| `Jade Garden quick_switcher` | `quick_switcher.at`（219L） | 双轨 `watch` + ext imports (`use { ... }`) | `plan980_bweb_jg_watch` 2/2 + `test_jg_watch_mount.mjs` 通过；组件面 `use` 块按 981 合同具名拒 |
| 包面组件 (`ace_package_sfc`) | `package.at` 组件包语料 | `ace_package_sfc` 发射；`sfc-handler-stmt`（F-R21-2）与 `sfc-view-for`（刀B）具名拒 | `plan980_package_sfc` 8/8 通过；多分支控制流重放见残余记账④ |
| `vue_capabilities` 83 语料 | `tests/vue_capabilities.rs` | `ace_ts_provider::p980_web_full_dual_track_census` | 27/27 通过，聚合 census `(75,8,8,22)` 稳定零漂移 |
| 生命周期/样式通道 | `setup/watch/expose/ext_imports/style/unmount` | 双轨 + ACE lane 全派生 + `expose` dot 前缀语法对齐 | `plan1072_vue_web` + `test_p1072_lifecycle_watch_expose.mjs` 实浏览器通过；`setup {}` 沿 Vue host compiler 顶层生成 |
| 错误保旧与恢复 | `incremental_compile_changed` / `gen_vue_project` | 冷生成脚手架 + `a2x_rt.mjs`；错误时 `App.vue`/`a2x_rt.mjs` 逐字节不变 | `plan1072_vue_error_recovery`：non-strict `Ok(0)` 保旧、strict `Err` 保旧、lane 拒保旧、修复恢复全绿 |

**残余记账四项（全部落位）。**
1. `n_assigns` census 漂移记账：`ace_ts_provider.rs`（L315-320, L392-397）与 `ace_vue_script.rs`（L490-496）记账 `n_assigns` 来自语言侧 `handler_assigns`（981 r21 起由 `flat_handler_assigns(&stmts)` 跨分支深平铺）；唯一消费者 `needs_fn = n_assigns > 0 || rows.iter().any(|r| !r.arg_spans.is_empty())`，含分支赋值的 handler 保守且语义正确（分支体进函数），零执行漂移。
2. `AC-LIF-01..07` 1072 侧记账：AC-LIF-01..06 主体已随 980 `lambda-if` 刀（lang `347540437` + ui `05ed07c3`, `m2-autoui.md` §13.12）交齐；1072 侧经 `plan1072_vue_web` + `test_p1072_lifecycle_watch_expose.mjs` 验证 AC-LIF-07（`setup` 单次执行 + `ext_imports` + `watch` 更新与 `unmount` 停听 + `expose` 调用与 `.Inc` 对齐语法）。
3. 原 980 §10-1 `owned-wrap` 统一决策：Knife C（ui `b9417486`）`marshal_str_boundary` 与 `wrap_host_reads` 引号字面量保护及 `p980_web_param_browser_mount` 官方 `script_runtime_body` 迁移已统一三道（状态函数/检查函数/λ 道）str 边界。
4. `#6p` 分支内调用形与 `ace_package_sfc` 多分支控制流重放边界：`ace_ts_provider.rs:1619` 锁死分支内调用形具名缺席（`.__evt_onclick_1`），`ace_package_sfc.rs` 保留 `sfc-handler-stmt` 与 `sfc-view-for` 具名拒，并迁入 `KNOWN-DEBT-AND-RISKS.md`。

**本轮测试耗时表（依 `test-discipline.md`）。**

*表 1：模块与作用域分组耗时*

| 仓 | 命令/测试集 | 结果（pass / fail） | 测试体耗时 | 总命令耗时 | 提交/状态说明 |
|---|---|---|---|---|---|
| `auto-lang` | `ace-core --test conformance plan1072_component_style` | 16 / 0 | 0.12s | 0.437s | 冻结 `48738ff32` |
| `auto-lang` | `ace-core --test conformance plan981_grammar_scope` | 4 / 0 | 0.31s | 0.472s | 本会话同 SHA `48738ff32` 实测 |
| `auto-lang` | `ace-core --test conformance plan981_members_slot` | 18 / 0 | 0.08s | 0.229s | 本会话同 SHA `48738ff32` 实测 |
| `auto-lang` | `ace-core --test conformance plan980_lambda_if` | 5 / 0 | 0.03s | 0.183s | 本会话同 SHA `48738ff32` 实测 |
| `auto-ui` | `plan980_u0_002_product_sfc` (E2E=1) | 2 / 0 | 2.66s | 20.015s | `tests/ace_ts_provider.rs` 修改前、源码与产品测试已同现态 |
| `auto-ui` | `plan1072_vue_web` (全 8 例, E2E=1) | 8 / 0 | 5.15s | 28.282s | 另见表 2 在终态 `8210ef5bc` 同内容下 `--exact` 全绿 |
| `auto-ui` | `plan1072_vue_error_recovery` | 1 / 0 | 4.38s | 35.910s | `tests/ace_ts_provider.rs` 修改前、tools 与 recovery 测试已同现态 |
| `auto-ui` | `plan980_u0_002_sfc` (E2E=1) | 2 / 0 | 18.47s | 50.623s | 仅 `tests/ace_ts_provider.rs` 在后；src 一致（T-04 冻结同 SHA 重放） |
| `auto-ui` | `plan980_bweb_jg_watch` (E2E=1) | 2 / 0 | 1.22s | 21.020s | 同上 |
| `auto-ui` | `plan980_notes_fp_editor` | 1 / 0 | 1.06s | 2.155s | 同上 |
| `auto-ui` | `ace_ts_provider` (E2E=1) | 27 / 0 | 34.27s | 74.012s | 终态 `8210ef5bc` 实测 |
| `auto-ui` | `plan980_package_sfc` | 8 / 0 | 2.31s | 13.820s | 终态 `8210ef5bc` 实测 |
| `auto-ui` | `plan980_ace_vue_tool_wiring` | 1 / 0 | 2.60s | 26.532s | 终态 `8210ef5bc` 实测 |
| `auto-ui` | `plan980_010_form_short` (E2E=1) | 1 / 0 | 1.71s | 31.588s | 终态 `8210ef5bc` 实测 |

*表 2：PLAN-1072 新增用例逐条耗时（全部 `<= 10.000s` 达标）*

| 仓/测试文件 | 用例名 | 结果 | 测试体耗时 | 命令总耗时 |
|---|---|---|---|---|
| `auto-lang` `plan1072_component_style` | 16 个新增用例全集（`--exact` 逐条实测，`p1072_component_style_literal_compiles` .. `p1072_expose_dot_prefixed_names_project_and_compile`） | 16 / 0 | 0.02s..0.07s | 0.172s..0.291s |
| `auto-ui` `plan1072_vue_web` | `p1072_005_lane_whole_source_derives` | 1 / 0 | 0.30s | 1.564s |
| `auto-ui` `plan1072_vue_web` | `p1072_005_style_expansion_is_literal_not_binding` | 1 / 0 | 0.19s | 0.826s |
| `auto-ui` `plan1072_vue_web` | `p1072_010_lane_whole_source_derives` | 1 / 0 | 0.19s | 0.804s |
| `auto-ui` `plan1072_vue_web` | `p1072_005_product_browser_mount` | 1 / 0 | 1.34s | 1.945s |
| `auto-ui` `plan1072_vue_web` | `p1072_010_product_browser_mount` | 1 / 0 | 1.26s | 1.876s |
| `auto-ui` `plan1072_vue_web` | `p1072_marshal_str_boundary_preserves_string_literals` | 1 / 0 | 0.16s | 0.767s |
| `auto-ui` `plan1072_vue_web` | `p1072_watch_expose_style_lane_and_setup_boundary` | 1 / 0 | 0.21s | 0.822s |
| `auto-ui` `plan1072_vue_web` | `p1072_watch_expose_setup_unmount_browser_mount` | 1 / 0 | 1.34s | 1.962s |
| `auto-ui` `plan1072_vue_error_recovery` | `p1072_vue_cold_gen_error_preserve_and_recovery` | 1 / 0 | 4.38s | 35.910s（含冷编；test 体 4.38s） |

next_responsible_role: review（T-04 独立复审，含 `cargo t ace-source`、`cargo t ace-av` 与 `auto-ui` 同冻结提交复验）

`stage: goal | plan_id: PLAN-1072 | plan_revision: r1 | round: 2 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1072/{auto-lang,auto-ui} @ plan-1072-dev | commits: lang 48738ff32a60fa56578da7ecee073d9bf8384917, ui 8210ef5bc0b1096d1184503373094d49d46ae9e1 | 证据锚点: T-03 剩余闭环（冷生成 a2x_rt/marshal 引号保护/expose 点前缀/.mjs 生命周期/错误保旧恢复）+ 旧新映射 9 项 + 残余 4 项记账 + UI 10 套件及 25 新增单测全 <=10s | next: review`

## 10. 待澄清事项

T-01 已核（§9 收据）；②已于 T-03 残余记账闭环（`n_assigns`、AC-LIF-07、§10-1 `owned-wrap`/`marshal_str_boundary`、`#6p` 分支内调用形与 `ace_package_sfc` 控制流边界）。保留观察：①`auto-ui` / `auto-lang` 主检出既有未提交 WIP（`examples/ui/stylekit`、`packages/themes`、`packages/blueprints`、`docs/design` 等）与 PLAN-1072 改件零重叠——禁止碰触，T-05 仅以 `--ff-only` 线性落地本计划显式文件。
