# Plan 426: setup 前导槽——per-instance setup 相位 + 生命周期语义定版

> **状态**: 🟢 立项待执行(draft;含设计决策,执行前需按 §1 定稿)
> **前置**: Plan 408(已归档,composable kind 机制);auto-musk PLAN-037(use.web composable 现状与局限的一手实测)。
> **仓库**: **auto-lang**(parser / ui_gen/vue / ts_adapter);auto-musk composables 域为迁移验证方。
> **目标**: widget 获得通用的 **setup 相位语句槽**——每实例同步执行、先于首渲染,支持任意 setup 逻辑(不止无参 composable 调用)。`.Init`(onMounted,首渲染后)与 setup(同步,首渲染前)两相位语义文档定版;`use.web composable` kind 降级为糖(保留兼容)。

---

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
