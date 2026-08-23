# Plan 436: setup 前导槽——解释器与 a2r 落地(每实例执行约定)

> **状态**: ✅ 完成(2026-08-23,worktree plan-436 执行;§1 决策定稿:**1-A 显式报错**(T0 探针:1-B 需将 setup 绑定接入 struct 字段与视图/处理器标识符解析,面大,留待需要时立项)+ **解释器 L1 落 bridge 单实例层**。残留边界见 §6)
> **前置**: Plan 426(已归档,setup{} 块语法 + a2vue 语义)、Plan 425(已归档,widget 单轨)。
> **仓库**: **auto-lang**(`crates/auto-lang/src/ui/interpreter/` + `src/interpreter/` + `src/trans/rust.rs`)。旧 ../auto-ui 仓已废弃,AutoUI 运行时全部在本仓(债务簿 426 条目已修正指向)。
> **目标**: `setup {}` 相位语义在 **a2r(Rust 转译)与动态解释器**两条路径落地/守门,消除"a2vue 独有"的后端不对称;解释器侧建立组件实例化约定(短期单实例语义 + 实例键边界文档化)。

## 6. 执行结果(2026-08-23 回填)

- **T1 a2r 止血(决策 1-A)**:`ui_gen/rust.rs generate_rust` 入口守卫——带 setup 块的 widget 显式 `GenError::UnsupportedStmt`(PLAN-037 T7 哲学,同 use.web 门控);`trans/rust.rs trans()` 逻辑路径对 `Stmt::WidgetDecl(setup 有)` 同款守卫(此前 wildcard `_ => {}` 静默消失)。测试 `test_setup_block_rejected_on_rust_target`。
- **T2 解释器 L1**:`InterpreterBridge::interpret` 改为 **UI 场景解析**(关键发现:VM 默认解析器拒绝 widget 语法,bridge 此前根本无法加载 widget 源)→ `AutoInterpreter::eval_ast`(新增;`VmInterpreter::run` 抽出 `run_ast` 供 AST 直入)→ `run_setup_preambles`:setup 语句 + 尾随绑定名数组表达式在**独立 VM run** 中执行一次(每次 run 新 VM,程序级作用域不延续——绑定须为字面量表达式),值经栈顶结果提取入 `WidgetState.fields`(type 键单实例),先于任何视图求值;`widget_state()` getter。非 UI 场景脚本走原 VM 解析回退(行为不变)。测试 ×3(绑定入 fields/错误显式上浮带 widget 上下文/无 setup 不建状态 + 普通脚本回归)。
- **T3 文档**:`ui/interpreter/mod.rs` 头注释重写(移除不存在的 SymbolTable/WidgetMetadata/ComponentInstance/InterpreterRuntime 引用,写实架构 + 边界);`docs/syntax.md` 三相位表扩为 **×三后端矩阵**(a2vue/解释器/a2r + 边界注记);债务簿 426「setup 解释器侧」条目改 ✅ 并记录残留。
- **T4 收口**:全量回归 auto-lang 默认 3128/3128、ui-iced 3606/3606(唯一失败为环境敏感的 Plan 077 纳秒基准,单跑恒绿)、auto-man 226/226。musk 三测零影响(musk 走 a2vue,本计划未触 vue 轨生成路径——按计划预期)。
- **残留边界(登记于 syntax.md + 债务簿)**:①解释器 setup 前导不延续程序级作用域;②`.Init`/`.Destroy` 事件路由未实现(仅 setup 相位落地);③a2r 真 setup 支持(1-B)未做;④L2 多实例不可及(type 键单例)。

---

## 0. 背景与现状调研(2026-08-23 实测)

Plan 426 落地了 `setup {}` 的语法与 a2vue 语义(script setup 顶层,每组件实例同步执行,
先于首渲染),但另两条后端路径未跟进。实测三个关键事实:

1. **a2r 静默忽略 setup**:`trans/rust.rs` 对 `Stmt::Setup`/WidgetDecl.setup 零处理——
   setup 语句经 wildcard 分支**无声消失**(与 use.web 门控前"错误后端静默丢弃"同款坏味道,
   PLAN-037 T7 已确立显式报错的哲学)。
2. **动态解释器无组件实例化机制**:`ui/interpreter/bridge.rs`(230 行)的
   `InterpreterBridge` 状态是 `HashMap<widget_name, WidgetState>`——**类型键单例**,
   无实例概念;`WidgetState.fields` 只存值映射。VM 侧(`interpreter/vm_interpreter.rs`,
   239 行)完全没有 widget 语义,`get_main_view` 只是 eval `main()` 期望 Node。
3. **文档腐烂**:`ui/interpreter/mod.rs` 头注释引用的 `SymbolTable`(widget 元数据版)/
   `WidgetMetadata`/`ComponentInstance`/`InterpreterRuntime` **并不存在**(scope.rs 的
   SymbolTable 是通用符号表,非 widget 元数据)。

结论:解释器路径连 model/on 的组件执行都未完整承载,"setup 每实例执行"在该路径的
诚实表述是**先定义边界、再落最小语义**。

## 1. 设计决策(执行前定稿)

1. **a2r 的 setup 语义**(二选一,倾向 A):
   - A. MVP 显式报错:Rust 目标遇 setup 块 → "setup requires vue render 或待 a2r 支持
      (登记)"——与 use.web 门控同哲学,**先止血静默丢失**;
   - B. 直接实现:setup 语句生成进 Rust widget 的构造/首视图之前(a2r 的 widget 是
      Elm 式 struct+update,等价槽位 = `new()` 后、首次 `view()` 前;语句经现有
      stmt→Rust 转译,绑定进 struct 字段或局部)。工作量中等,可作 P2。
2. **解释器的 setup 语义分层**:
   - L1(单实例对齐):`WidgetState` 建立时执行一次 setup 语句(经 AutoInterpreter eval),
     绑定写入 `fields`——对根 widget/单实例组件,与 a2vue 的"每实例"等价;
   - L2(真每实例):依赖子组件实例化机制(实例键状态)——**本计划只调研并文档化边界,
     不实现**(见 §3 T0)。
3. **相位顺序**:解释器侧约定 setup 先于首次视图求值(视图脏标记机制的求值入口前插
   setup 执行);`.Init` 在解释器现状不明——T0 调研后按 a2vue 语义(挂载后)对齐或登记。
4. **绑定可见性**:setup 绑定进 `WidgetState.fields` 后,视图/事件路径按普通字段消费
   (refs 标注字段的 `.value` 语义在解释器中不存在——Value 即值,文档化差异即可)。

## 2. MVP

1. a2r 按决策 1-A 报错(止血);若 T0 调研显示 B 便宜则直接 B。
2. 解释器 L1:`InterpreterBridge` 在 WidgetState 初始化时执行 setup 块(语句 eval +
   绑定入 fields);setup 先于首视图。
3. mod.rs 文档腐烂修复(头注释与实际 API 对齐)。
4. 三相位语义表(setup/.Init/.Destroy)在解释器侧的对应关系写入 syntax.md 附录或
   scenario-dialect spec 的解释器章节。

## 3. 执行步骤(草案)

1. **T0 调研**(定稿输入):①解释器路径 .Init/.Destroy 现状(事件路由是否触发生命周期);
   ②子组件在解释器/vnode 路径是否存在多实例(决定 L2 是否可及);③a2r setup 生成
   (1-B)的真实成本探针。产出:§1 决策定稿 + 边界文档。
2. **T1** a2r 止血(或 1-B 实现)+ 单测。
3. **T2** 解释器 L1(setup 执行 + 绑定入 fields + 相位顺序)+ 单测/头测。
4. **T3** 文档(mod.rs 头、syntax.md 三相位×后端矩阵)+ 债务簿 426 条目更新。
5. **T4** 收口:全量回归 + musk 三测(应零影响——musk 走 a2vue)+ 归档登记。

## 4. 测试设计

- a2r:setup 块 → 报错文案断言(或 1-B 产物断言)。
- 解释器:headless(ui-headless feature)e2e——含 setup 块的 .at 经 bridge 加载,
  断言 fields 含 setup 绑定且首视图求值可见其产物。
- 回归:a2vue 侧 426 既有单测不动;k2/k3 canary;musk 三测。

## 5. 风险

| 风险 | 等级 | 对策 |
|---|---|---|
| 解释器路径本身组件语义不完整(model/on 载荷缺位),setup 无处安放 | 🟡 | T0 先行;若机制缺位则 L1 落在"bridge 单实例"层并文档化,a2r 承担主路径语义 |
| AutoInterpreter eval 的语句级执行与 .at 源耦合(热重载语义) | 🟡 | setup 语句随 reload 重放(与 fields 重建同生命周期) |
| a2r 1-B 生成位置与 Elm 生命周期错位(new vs view) | 🟡 | MVP 报错路径兜底,B 方案探针验证后再启 |
| 实例键改造(L2)波及 vnode/gpui 渲染 | 🔴 | 明确出界,仅调研记录 |
