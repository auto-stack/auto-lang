# Plan 426: setup 前导槽——per-instance setup 相位 + 生命周期语义定版

> **状态**: ✅ 执行完成(2026-08-23,worktree auto-musk;T1-T4 全落地,回归全绿——见 §1.5 设计定稿与 §7 执行记录)
> **前置**: Plan 408(已归档,composable kind 机制);auto-musk PLAN-037(use.web composable 现状与局限的一手实测)。
> **仓库**: **auto-lang**(parser / ui_gen/vue / ts_adapter);auto-musk composables 域为迁移验证方。
> **目标**: widget 获得通用的 **setup 相位语句槽**——每实例同步执行、先于首渲染,支持任意 setup 逻辑(不止无参 composable 调用)。`.Init`(onMounted,首渲染后)与 setup(同步,首渲染前)两相位语义文档定版;`use.web composable` kind 降级为糖(保留兼容)。

---

## 7. 执行记录(2026-08-23)

- **T1 设计定稿**(fb4597eb):§1.5——块关键字 `setup{}`(方案 A);refs
  采用**块级声明**(`refs <binding>: [f...]`;let 后缀方案与表达式语法
  冲突,实测改道,即 §1.2 第二选项);await MVP 拒绝;命名冲突编译错误。
- **T2 parser + codegen + 单测**(ee08af02):AST SetupBlock
  (WidgetDecl.setup/AuraWidget.setup,widget 与 component fn 兼容拼写均
  支持);parser 块级 refs 声明 + await 拒绝(表达式语句与 let 初始化器
  双检查);vue.rs 语句置于 script setup 顶层 state/computed 之前,绑定
  注册 facade locals(script/handler 访问无 .value、模板顶层 const 天然
  可见),refs 字段访问注入 .value(复用 facade_ref_fields);绑定与
  model/prop 冲突报错。单测 ×4(位置/refs/await 拒绝/冲突)全绿,
  3093+6 全量绿。
- **T3 musk 迁移对拍**(musk efee0e0):9 文件 composables 域迁 setup 块
  (useT→t ×5、useI18n→i18n+refs ×4、useGateRouter→gateRouter;use.web
  条目降普通导入仍指 composables 端口)。**产物对拍:import + 绑定位置
  不变即等价**(let/const 关键字与 handler 顺序为既有非确定差异);
  auto build + vue-tsc EXIT=0 + cargo test + vitest(2 存量基线)全绿。
- **复审补强(合并 master 后)**:①绑定发射改为 **`const`**(§1.2 文字
  要求;此前复用 transpile 的 `let` 偏离计划文字,`var` 绑定保持 let);
  ②setup 语句引用 model 变量/computed/prop 增加编译期拒绝(发射顺序上必
  TDZ,单测锁定);③refs 块级声明维持(设计定稿选项)。
- **T4 文档 + 收口**:三相位语义表(setup/.Init/.Destroy)写入
  scenario-dialect spec + docs/syntax.md UI 节;composable kind 标注
  "糖,推荐 setup 块";k2/k3/k4 canary + auto-lang 3093 + auto-man 6
  回归绿。后续登记:解释器侧每实例执行约定(auto-ui interpreter 联动)、
  async setup Suspense 支持。

---

## 1.5 设计定稿(2026-08-23,T1)

§1 两处决策按执行实测定版:

1. **语法形态:方案 A——块关键字 `setup { ... }`**(与 msg/model/on 同级,
   位置任意,语义固定 setup 相位)。widget 与 component fn(425 糖化后的
   兼容拼写)均支持。
2. **绑定语义:`let x = expr` → script setup 顶层语句**(ts_adapter 直出,
   `let`;绑定名注册为 facade local——script/handler 访问不注入 .value,
   模板经 script-setup 顶层 const 天然可见)。
   **refs 标注采用块级声明**:`refs <binding>: [f1, f2]`(setup 块内独立
   语句)。§1.2 的"let 后缀"方案(`let x = useI18n() refs: [...]`)实测
   与表达式语法冲突(表达式解析器把 `refs` 当中缀继续消费),改用块级
   声明——即 §1.2 给出的第二选项。标注字段 script 侧访问注入 `.value`
   (复用 composable kind 的 facade_ref_fields 机制)。
3. **await:MVP 明确报错**(parser 层拒绝,表达式语句与 let 初始化器均
   检查;错误信息指向 Suspense 限制)——按 §5 风险表决议,async setup
   另立任务。
4. **命名冲突**:setup 绑定与 model 变量/prop 同名 → 生成期编译错误
   (对齐 PLAN-037 T5 先例)。
5. **发射位置**:script setup 顶层,state/computed 定义**之前**(绑定
   先行);解释器侧每实例执行约定登记后续(auto-ui interpreter 联动)。


## 0. 背景与设计动机(2026-08-22 会话结论)

现状 `.Init → onMounted`(vue.rs:1721 注释),而真 composable(useI18n/useRouter 类)必须
**setup 期同步执行**,晚到 onMounted 坏三件事:内部 onMounted 注册失效、inject 无语境、
首渲染缺值。`use.web composable` kind 就是为此开的窄后门(自动调用 + 命名约定 + refs 标注),
但它只能表达"无脑调用一次",不能:传参、按条件调用、多语句初始化、调用结果二次加工。

关键设计修正(来自当时讨论):若把它实现为"widget 之前的模块级代码",在解释器(AutoUI
继承 AutoVM)里是**模块加载一次**——会单例化状态、丢 inject 语境、生命周期悬空。正确语义
是**每 widget 实例的 setup 前导**:

- a2vue:语句原样移植到 script setup 顶层(composable 需要的精确槽位);
- 解释器:每次 widget 实例化时执行,而非模块加载。

## 1. 设计决策(执行前定稿)

1. **语法形态**(二选一,倾向 A):
   - A. 块关键字:`setup { ... }`(与 msg/model/on 同级,位置任意,语义固定 setup 相位)——
     显式、无歧义、LSP 友好;
   - B. 体前导语句(widget 裸语句区,view 块之前)——更"脚本感",但与 425 的
     view 可选化(体即视图)存在消歧成本。
2. **绑定语义**:`let t = useT()` 声明的名字 = 组件局部绑定(script 顶层 const):
   - 模板自动解包(顶层 ref 免 .value);
   - script 内访问:ref 字段需 `.value`——沿用 composable kind 的 `refs: [...]`
     标注机制(解构时标注:`let { locale } = useI18n() refs: [locale]`,或块级
     `refs` 声明);
   - computed/model/on 中引用合法(parser 符号注册)。
3. **与 model 变量的关系**:setup 绑定 ≠ model 变量(不进 defineModel、不可被父绑定);
   需要双向时用 model 声明 + setup 内初始化(组合表达)。
4. **composable kind 降级**:现有 `use.web composable useT from ...` 改写为等价
   setup 块 + use.web 普通导入(糖转换在文档/迁移脚本层,语言层不动旧语法)。

## 2. MVP

1. parser:`setup {}` 块(方案 A),语句集 = 普通 handler 语句子集(let/表达式/await)。
2. codegen:语句置于 script setup 顶层(state/computed 定义之前——绑定先行);
   绑定名注册进 script 侧符号表(ts_adapter 的名字类别:非 state、非 prop、
   ref 标注者访问加 .value)。
3. await 支持:setup 含 await → 该组件 async setup(Suspense 边界,登记限制)。
4. 文档:`.Init`(onMounted)/ `.Destroy`(onUnmounted)/ setup(同步 setup)三相位
   语义表写入 syntax.md;composable kind 标注"糖,推荐 setup 块"。

## 3. auto-musk 迁移(验证面)

composables 域四件迁 setup 块(useT / useI18n refs:[locale] / gate_router / 424 未动的部分):
musk app.at 的 composable use.web 条目改写,产物对拍(import + const 位置不变即等价);
三测全绿。若 424 已落地 composable 端口转发,本计划替换其调用面为 setup 块直调。

## 4. 测试设计

- 单测:setup 块 → script 顶层语句位置;refs 标注 .value;绑定在 computed/on 可用;
  await → async setup。
- canary:k3 或 k5 放 setup 块调 useT 类(伪造 composable)端到端。
- 回归:musk 三测;composable kind 旧语法存量(plan408_tests/007 golden)不破。

## 5. 风险

| 风险 | 等级 | 对策 |
|---|---|---|
| 绑定符号与 state/prop 名类冲突 | 🟡 | 命名冲突 → 编译错误(对齐 PLAN-037 T5 同名错误的先例) |
| ref 解包语义双轨(模板/script) | 🟡 | refs 标注机制复用 + 文档明确;canary 实测 locale |
| async setup 的 Suspense 依赖 | 🟡 | MVP 拒绝 await(明确报错),Suspense 支持另立任务 |
| 解释器侧每实例执行约定 | 🟡 | 本计划只落 a2vue;解释器路径登记后续(auto-ui interpreter 联动) |

## 6. 执行步骤(草案)

1. T1 设计定稿(§1 两处决策)→ 更新本计划。
2. T2 parser + codegen + 单测。
3. T3 canary + musk composables 域迁移对拍。
4. T4 文档(三相位语义表)+ composable kind 糖标注 + 收口回归。
