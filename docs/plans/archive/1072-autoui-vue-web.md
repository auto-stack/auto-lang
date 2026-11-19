---
plan_id: PLAN-1072
status: archived
feature_name: AutoUI Vue 与生产 TS/JS 接管
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19
plan_revision: 1
current_step: 4
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

取材 [AutoLang 全局总览](../../specs/overview.md) 的 UI/Web 生态，以及 [980 原合同](980-ace-m2-autoui.md#0-变更摘要)。原技术合同、输入、测试分母和负控按归属表接收。

## 5. 详细设计

**前置**。消费 980 DIALECT/NATIVE 实际使用的已交接口、981 EVENT-B 对应项、976/978。010 的列表字段按 1071 所交精确语义接入；场景不使用该字段时可独立推进。U0 联合收据 `2c9c5f228` 按原身份接收。

**接手范围**。接 980 B-WEB、完整生产 TS/JS 的 Vue 子面和 PHASE-LAMBDA-IF。包括 005、010、AutoNotes 跨文件件、quick_switcher/use、组件包和实际工程的旧新映射；已封存内容按原来源复用。

**PHASE-LAMBDA-IF 交付状态注记（2026-11-19 跨会话协调）**：该刀主体已与拆分裁定**同日在 980 名下交付落主**——lang `347540437` + auto-ui `05ed07c3`（T-LIF-1..6 全交、AC-LIF-01..06 闭、规范沉淀 `m2-autoui.md` §13.12；收据=980 §9「lambda-if 刀交付」）。**F-R21-2 亦已闭**（981 r21 指派项，同日 980 名下：ace_package_sfc 脚本重放落 `handler_stmts` 消费门=具名拒 `source.age.sfc-handler-stmt`，auto-ui `7b8456e5`；收据=980 §9「F-R21-2 收口」——981 T-BASE-ONIF 可据此复核）。1072 承接的**残余**：AC-LIF-07 的 1072 侧记账、原 §10-1 owned-wrap 统一决策、分支内调用形→通道调用承载（#6p 谱系）、005 整例 use/style/design 残墙、`n_assigns` 平铺计数的 census 漂移记账——不重做已交主体。

每组开工确认精确输入源、对应原用例、接口 owner 及身份。共享语义和 ABI 由原 owner 串行写入，变化时重验实际受影响格。已有阶段收据保留其验收边界。

### 规范增量

- **规范目标**：`docs/specs/ace-migration/m2-autoui.md`
  - 更新顶部 `> Status:` 摘要，记录 PLAN-1072 完成 Vue 与生产 TS/JS 接管交付（明确 `--features ace-vue` + `AUTO_UI_ACE_VUE=1` 开关作用域及双轨边界）。
  - 新增 `## 15. Vue 与生产 TS/JS 接管（PLAN-1072，2026-11-19）`：沉淀组件面 `style` 惰性替换路径、`.name`/裸名并容的 `expose` 语法对齐、`view-for` + 仅视图数据字段（`ComponentViewDataField`）+ 条件样式值（与 `Expr` 共享 `replace_style_in_entry_expr` 对导入/未声明/调用形的 fail-closed 检查）、`marshal_str_boundary`（含引号字面量保护）统一 str 边界、冷生成 `write_ace_vue_rt` 修复、`setup/watch/expose/ext_imports/unmount` 与错误保旧恢复边界、旧新映射表及残余记账；并明确以下**实现边界**（不超宣称）：
    1. **开关与路径边界**：仅在 `auto-ui-tools --features ace-vue` + `AUTO_UI_ACE_VUE=1` 下对 `app.at`（`vue.rs:4173, 7684`）与 `components/*.at`（`vue.rs:4495`）启用 ACE lane 产物替换；默认构建、`front_dir` 兄弟文件（`vue.rs:7398`）、`pages/`（`vue.rs:4350`）、`widgets/`（`vue.rs:7757`）与 `stores/` 仍走旧 `VueGenerator` / `ts_adapter::TypeScriptTrans`。
    2. **双轨拼接边界**：`generate_widget_with_lane`（`ace_vue_script.rs:796-884`）以 `VueGenerator::new().generate_sfc(&widget)` 为底稿，仅将受检 `on` / `__evt_*` handler 体拼接替换；`computed`、`watch`、`setup`、`ext_imports`、`.Init`/`.Destroy` 与 `absent` 回退处理器仍由 `VueGenerator` 发射（如 `Widget B` 含 `setup`+`use{}` 双轨 fail-closed 保留 `absent=[".Bump"]`）。
    3. **产品验证夹具边界**：1072 实证入口为 Rust 库 `gen_vue_project` / `incremental_compile_changed` + `@vue/compiler-sfc` + `esbuild`（`plan1072_vue_error_recovery.rs`）与本地 Node `http` Playwright（`test_005_login.mjs`、`test_010_form.mjs` 带 `@/components/ui/*` stub、`test_p1072_lifecycle_watch_expose.mjs` 带 `BOOT_EXT_STUB`）；`plan980_010_form_short.rs` 浏览器腿仍挂旧 `App.vue`，而 `plan1072_vue_web.rs::p1072_010_product_browser_mount` 挂 lane `App.sfc` 实测三字段 `v-model` + `Submit`（`SetTheme`/`SetAccent` 在 lane 单测及 `test_p1072` 之外未另做 010 页面点击）；公开 CLI `auto gen`/`auto build`/`auto run` 派发及完整 `pnpm`/Vite 打包运行归 **PLAN-982** 联合门。
- **债务账本目标**：`docs/plans/KNOWN-DEBT-AND-RISKS.md`
  - 追加两项残余债务：(a) `#6p` 分支内调用形（具名缺席 `.__evt_onclick_1`）与 `ace_package_sfc` 多分支控制流重放边界（`sfc-handler-stmt`、`sfc-view-for`）；(b) 1072 移交的残余 `V-TS-CONSUMER-UI` / `B-WEB` / 982 联合义务（`pages/`/`widgets/`/`stores/` lane 接管、`watch`/`computed`/`setup` 去 `TypeScriptTrans` 下沉、`--features ace-vue` + `AUTO_UI_ACE_VUE=1` 默认开启切换、完整 `pnpm`/Vite/`auto run` CLI 闭环）。

## 6. 测试设计

与 982 联合验一个原样 Vue UI 工程的 `auto gen`（Vue/TS 与运行时输出）、`auto build`（实际构建）、`auto run`（真实浏览器、事件与状态）。**分工边界**：1072 交 Rust 库入口（`gen_vue_project` / `incremental_compile_changed` / `generate_sfcs_with_lane_product`）、官方 `a2x_rt.mjs` 运行时、`@vue/compiler-sfc` + `esbuild` 产物编译与 Playwright Node `http` 浏览器行为依据；982 验公开 CLI（`auto run --render vue` / `auto build`）与完整 `pnpm`/Vite 构建运行闭环。U0 只作为既有 Run 子面按原身份复用。

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

- [x] **T-03｜work｜验收 Vue 产品链**。验收 V-B-WEB 及适用产品生成入口（`gen_vue_project` / `incremental_compile_changed` + `@vue/compiler-sfc` / `esbuild` + Playwright Node `http` 浏览器实操事件、状态、watch/expose/setup/unmount、错误保旧恢复）；完整 `auto run --render vue` CLI + `pnpm`/Vite 打包循环按 §6 与 PLAN-982 联验交付。

- [x] **T-04｜review｜独立复审**。逐项核 AC-01..03、原任务和实际反例，修复发现后复验。

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

2026-11-19 stage: review（T-04 独立终审，`review_scope: final`，1 轮修复闭环）| plan_id: PLAN-1072 | plan_revision: r1 | outcome: pass | reviewer: subagent `a3954308-7f96-4c71-a25c-9bb6c65634b1`（`research` 独立审计 + 冻结同提交重放） | reviewed_commits: lang `4bccd505548aed0bbba761fa237d32e88f5095f3`（修复 `48738ff32`）, ui `c1be21e45d238c9eabd64879b4d5a23d5d953a64`（修复 `8210ef5bc`）, plan doc `25741b96cf83bf1d7e54a563d2e9a4f76f10fce4` | siblings: auto-down `72b097006f34db3a871449f10e213820d51abef6`, auto-man `f461960f5db9bc99267f93541dc7c5bdeedfa048`, auto-web `3d328a3561935ac390916cfa356603bc30135e03`。

**Round-0 独立复审 `needs_fix`（四项阻塞发现）→ Repair Cycle 1 全量闭环。**
1. **Finding 1（已修，`4bccd5055`）**：`plan1072_component_style.rs:122` 原写 `let _ = compile(src);` → 改为 `compile(src).expect("`style` remains legal as a plain identifier");`，19/19 通过且 `--exact` 实测 `0.02s..0.03s`。
2. **Finding 2（已修，`4bccd5055`）**：`frontdoor.rs:2987-3098` 抽取共享 `replace_style_in_entry_expr` 覆盖 `ComponentEntryValue::If`（`then_value` & `else_value`）与 `ComponentEntryValue::Expr`，统一对 `imports` / `reject_unknown && entry_key == "style"` / `ExprKind::Call` 执行 `source.age.style-call` fail-closed；`plan1072_component_style.rs:450-478` 补齐 3 项条件样式负控（`p1072_conditional_style_unknown_ident_is_named` / `p1072_conditional_style_imported_ident_is_named` / `p1072_conditional_style_call_form_is_named`，各 `0.02s`）。
3. **Finding 3（已修，`c1be21e45` + Option A）**：`rustfmt --edition 2021` 对齐 `plan1072_vue_web.rs`、`plan1072_vue_error_recovery.rs`、`plan980_ace_vue_tool_wiring.rs`（Python token 等价性 `True`，两仓 PLAN-1072 变更行 `rustfmt --check` 全过）。`Cargo.lock` 按 Option A 不入 `plan-1072-dev`（+6 行为兄弟仓 `../auto-web` @ `3d328a35` 依赖解析产物，且 `auto-ui` 主检出已有未提交 `M Cargo.lock` 避免合并冲突），`auto-ui` 测试以 `--offline` 运行后 `git checkout -- Cargo.lock` 还原干净。
4. **Finding 4（已修，`25741b96c`）**：在 `1072-autoui-vue-web.md` §5 规范增量 / §6 / §8 精确限定开关作用域（`--features ace-vue` + `AUTO_UI_ACE_VUE=1`，仅 `app.at` + `components/*.at`）、`VueGenerator`/`TypeScriptTrans` 双轨并存边界（`Widget B` `absent=[".Bump"]`、`computed`/`watch`/`setup`/`pages/`/`widgets/`/`stores/`）、以及 `1072`（库/SFC/`esbuild`/Playwright Node `http`）与 `PLAN-982`（CLI/`pnpm`/Vite 打包）联合门划分。

**终态冻结提交实跑门禁汇总（`4bccd5055` + `c1be21e45`）。**
- 机器占用退让记录：在 `lang-1098`/`lang-1078` 并发 `--release`/nextest 期间退让检查（约 4m+ 累计多段退让，复验前确认 0 `cargo/rustc/nextest/link` 占用）。
- `auto-lang` (`4bccd5055`)：
  - `cargo t ace-source` (`run-5r7lq5ca`): **PASS**, `923` executed / `923` ok / `0` failed / `0` ignored, wall `582.599s` (reviewer log `581.65s`), `gate_passed: true`。
  - `cargo t ace-av` (`run-29xmmz07`): `1428` executed, **1426 passed / 2 failed / 22 skipped** (`Summary [89.305s]`, wall `125.335s`)——仅预存 `host_console::av_console_entry_selection_cannot_bypass_main_policy` (`0.078s`) 与 `host_console::av_console_owned_front_door_enforces_console_entry_only` (`0.055s`) 两条红，零并发负载抖动（`v02_d06_spawn_with_output_av` 与 `kv_zip_1064` 本轮全绿）。
  - `ace-core --test conformance`: `plan1072_component_style` **19/19** (`0.04s` / `5.538s` 含重编；复审重跑 `0.05s` / `1.369s`；4 新增/修负控 `--exact` 均为 `0.02s..0.03s`)、`plan980_style_surface` **9/9** (`0.04s` / `0.325s`)、`plan981_grammar_scope` **4/4** (`0.30s..0.32s`)、`plan980_lambda_if` **5/5** (`0.02s..0.03s`)、`plan981_members_slot` **18/18** (`0.06s..0.09s`)。
- `auto-ui` (`c1be21e45`, `--offline --test-threads=1`, `AUTO_UI_ACE_WEB_E2E=1`)：**全部 10 套件零失败**（`plan980_010_form_short` 前次因 OS 临时端口撞 Chromium 安全港限制 6669 `ERR_UNSAFE_PORT` 重放立即通过）：
  - `plan1072_vue_web` 8/8 (`5.52s`；批跑复审 `5.59s`/`5.776s`)；本轮终态 `c1be21e45` 全部 8 条 `--exact` 实测 `0.16s..1.45s` test / `0.894s..2.241s` wall（均为 `<= 10.000s`；注：复审子代理在冷启动 Chromium 竞争下观察到 `p1072_005_product_browser_mount` 单条冷启方差 `10.40s` / `25.88s`，批量同套件 8 例总耗时 `5.59s`，须保持 E2E 浏览器套件 `--test-threads=1` 串行）。
  - `plan1072_vue_error_recovery` 1/1 (`2.50s` / `74.209s` 含冷编；复审 `2.37s` / `2.622s`)、`plan980_ace_vue_tool_wiring` 1/1 (`2.70s` / `21.464s`)、`ace_ts_provider` 27/27 (`9.37s` / `32.408s`；复审 `7.47s`)、`plan980_010_form_short` 1/1 (`1.84s` / `2.717s`)、`plan980_u0_002_product_sfc` 2/2 (`1.48s` / `24.108s`)、`plan980_package_sfc` 8/8 (`2.26s` / `23.073s`)、`plan980_bweb_jg_watch` 2/2 (`2.20s` / `34.902s`)、`plan980_notes_fp_editor` 1/1 (`1.39s` / `2.751s`)、`plan980_u0_002_sfc` 2/2 (`1.42s` / `25.813s`)；另有 Round-0 实跑未改动 crate 基线：`vue_capabilities` 77+5（5 例均为 PLAN-643 预存 chart）、`auto-ui-tools --lib vue::tests` 60/60、`auto-ui-dialect --lib` 105/105、`ui_domain_validation` 5/5。
- 脏工作区安全核对：两仓主检出未提交 WIP 与本计划修改文件及目标文档零重叠。

next_responsible_role: merge（T-05：规范沉淀 `m2-autoui.md` §15、债务入 `KNOWN-DEBT-AND-RISKS.md`、rebase + `range-diff` + `--ff-only` 合入双仓、归档至 `docs/plans/archive/`、`wt-guard` + 五树清理 + `plan-artifacts/plan-1072` 清删与保留登记表更新、批回归到期移交）

`stage: goal | plan_id: PLAN-1072 | plan_revision: r1 | round: 3 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1072/{auto-lang,auto-ui} @ plan-1072-dev | commits: lang 4bccd505548aed0bbba761fa237d32e88f5095f3, ui c1be21e45d238c9eabd64879b4d5a23d5d953a64, plan 25741b96cf83bf1d7e54a563d2e9a4f76f10fce4 | 证据锚点: T-04 两轮独立复审 pass（ace-source 923/923 PASS, ace-av 1426/1428 仅预存 host_console×2, UI 10 套件全绿, 四发现全修） | next: merge`

2026-11-19 stage: merge（checkpoints: `prepared` → `landed` → `canonical_verified` → `archived`, `completion_kind: delivered`）| plan_id: PLAN-1072 | outcome: pass | landed_master: auto-lang `9142f94bb787ff01ba48d5176e8ef205cfd9c6b8` (`--ff-only` over `55016bf97`), auto-ui `67893861c959788b2770b00b8d24bb9a31eb8418` (`--ff-only` over `e7aeda14`)。
- **`prepared` & `range-diff` 证明**：`auto-lang` 代码变基 `e81c27e9f..4bccd50554 → 55016bf97..307055b19`（`git range-diff` 4/4 `=`：`5492b3ca7=5d059ac4d`、`041148cc1=389ce787f`、`48738ff32=8eeed9cfa`、`4bccd5055=307055b19`；`experimental/ace/{crates/core,grammars}` 在旧新基线间零变更）；`auto-ui` 变基 `d2e7d7a4..c1be21e45 → e7aeda14..67893861`（`git range-diff` 5/5 `=`：`ec7342c1=4ce227b1`、`1489d0cc=2d393650`、`b9417486=f234ce84`、`8210ef5b=09f307ee`、`c1be21e4=67893861`）。五 worktree `wt-guard.ps1` 全部输出 `clean`；双仓主检出既有脏 WIP 与合入文件零重叠、未触动。
- **`canonical_verified`**：规范仅文档尾提交 `9142f94bb` 仅改 `docs/specs/ace-migration/m2-autoui.md`（更新顶部 `> Status:` 与新增 [§15 Vue 与生产 TS/JS 接管](../../specs/ace-migration/m2-autoui.md#15-vue-与生产-tsjs-接管plan-10722026-11-19)）；落主 SHA-256 = `857CEB6A33EC4DAE353C499C402E66A1234D61808B3E3721A120519577D1C5E7`。两项边界债务 `D-1072-01`、`D-1072-02` 已登记入 [KNOWN-DEBT-AND-RISKS.md](../KNOWN-DEBT-AND-RISKS.md)。
- **`archived`**：已 `git mv docs/plans/1072-autoui-vue-web.md docs/plans/archive/1072-autoui-vue-web.md`，相对链接已校对，frontmatter `status: archived`；接下来执行 `wt-guard` 清理 5 worktree、删分支与 `plan-artifacts/plan-1072/`，刷新 `.retention/policy.json` 并落最终 `cleaned` 收据。

## 10. 待澄清事项

T-01 已核（§9 收据）；②已于 T-03 残余记账闭环（`n_assigns`、AC-LIF-07、§10-1 `owned-wrap`/`marshal_str_boundary`、`#6p` 分支内调用形与 `ace_package_sfc` 控制流边界）。保留观察：①`auto-ui` / `auto-lang` 主检出既有未提交 WIP（`examples/ui/stylekit`、`packages/themes`、`packages/blueprints`、`docs/design` 等）与 PLAN-1072 改件零重叠——禁止碰触，T-05 仅以 `--ff-only` 线性落地本计划显式文件。
