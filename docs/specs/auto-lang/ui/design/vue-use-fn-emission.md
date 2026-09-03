# use 导入 helper fn 发射进 vue SFC

> Plan 522（2026-09-02）。关闭 437 §0.6.E-3 缺口：`use` 导入的模块级
> helper fn 此前不发射进生成的 vue SFC——computed/handler 体调用它们在
> Vue 侧无函数可调（vue-tsc TS2304），语料被迫内联重算或直调 `math.*`
> 绕过。VM 侧无此问题（import_stmts 编进同一 VM 模块 + PLAN-051 C3 的
> import_aliases 裸名解析）——本机制补齐 Vue 侧，双端语义对齐。

## 范围

`ui_gen/vue.rs` 的 SFC script 生成（`emit_use_module_fns`）、
`ui_gen/api.rs` 的池收集（`collect_use_module_fns`）、auto-man
components/ 包通道接线、VM 装载的包组件 use 依赖收集
（`lib.rs build_dynamic_component_inner` + `plan370_test_support` 镜像）。

## 机制

1. **池收集**（api.rs）：`use_scanner` 扫宿主文件 use 语句 →
   `resolve_use_module` 解析模块文件 → **全模块** fn 入池（镜像 VM
   loader 全模块加载：`build_month_grid → weekday_of` 这类未被 use items
   列名的兄弟 fn 也要在池内）；不可读/不可解析模块静默跳过（与
   `collect_use_module_actions` 同策略）。跨模块同名先到先得（use 顺序）。
2. **入口门控**：widget 体引用的裸名必须 ∈ use items 导入名集合——与
   VM `import_aliases` 同口径（未导入的池内 fn 不拉；池内闭包拉取不受
   此门限）。
3. **引用收集**：computed 表达式 + handler 体 + **lifecycle 体**
   （`.Init` 提取进 onMounted 而非 handlers 表——漏扫则 donut 类
   Init-调用 helper 的形态 TS2304）。walker 覆盖复合表达式；方法调用
   （`obj.foo()`）不计——helper 只以裸名形态被引用。
4. **闭包拉取**：被拉 fn 体内引用的池内其他 fn 一并拉入，迭代到不动
   点；同名只发一份（池序稳定输出）。
5. **三态闸**（发射前过滤）：
   - ext_imports 同名 → 静默跳过（Plan 051 手写 TS 逃逸口优先）；
   - 命名冲突（state/computed/props/handler/facade/同文件 module fn）→
     R013 警告 + 跳过；
   - 转译边界预扫描（`use_fn_body_unsupported`：`is`/cover 族模式匹配、
     借用语义表达式）→ R013 警告 + 回退现状（不发射），不阻塞生成。
     v1 保证面：纯算术/字符串/列表 fn。
6. **发射**：script 尾部（function 声明提升保证上方 computed 段可解析，
   调用点零改写）；body 转译用 `transpile_body_as_return`（Plan 448 H1
   机制——尾表达式体必须 `return` 化，否则 `is_leap` 类谓词在 Vue 运行时
   拿到 undefined）。
7. **接线面**：主通道（`generate_component_from_file` 两处 gen 构造）；
   auto-man components/ 包通道（裸生成器重生成时挂回池，
   `with_use_module_fns`）；VM 侧包装载对包组件文件自身的 use 依赖做
   collect_module_imports + 裸名别名（与根文件 use 同规则）。

## 边界与债

- 同文件 module_fns 与 store composable 路径仍用 `transpile_handler_body`
  （尾表达式丢 return，与 use-fn 路径不一致）——债 P522-1；
- auto-man components/ 通道 module_fns 未挂——债 P522-2；
- 方向 B（共享 utils 模块文件）演进预留：同一 fn 被 ≥2 个 SFC 引用时
  产物重复，阈值触发再立项。

## 语料与验证锚

016-calendar（computed 化迁移：`month_label`/`days` 调
`month_name`/`build_month_grid`，store `.Rebuild` ×4 重算链删除）+
024-charts（donut `dc`/`ds` helper 形态回正，`chart_geom.at`）。
测试锁：vue 单测 11（发射/不发/闭包/块体/handler/门控/去重/ext 抑制/
冲突/pattern 回退/尾表达式 return）+ api 集成 3（文件系统端到端 + 016/024
语料锁）+ vm_bridge 2（016 生产路径 grid、024 dc/ds VM 别名求值）。
