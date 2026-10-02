# VM 模块级 fn 调用语义（store handler 域 / 视图 computed）

## 范围

VM 渲染目标（`--render=vm`）下，.at 源内对**模块级 fn** 的两类调用域的语义契约与已知边界：
store handler 域（含 timer/lifecycle 派发）与视图 computed 域（表达式 computed，含
`use`/`use.web.fn` 导入形态）。契约锚定：PLAN-733（2026-10-02）；矩阵实证 = 进程内
`plan733_fn_call_semantics_tests`（全参型 × 同文件/跨模块 × 消费原语）+ 二进制级探针
（`D:/autostack/.demo/lang-733/`，真机 `auto run --render=vm` + MCP 快照直读）。

对应代码：`ui/vm_bridge.rs`（`call_vm_fn` / `call_handler_for`）、`ui/handler_codegen.rs`
（handler/computed 合成）、`ui/aura_view_builder.rs`（视图侧表达式求值器）、
`vm/engine.rs`（栈编码约定）。

## 原则

1. **调用返回真实值**：fn 调用（任意参位数 0..N、任意参型含列表/obj/str/int/bool，
   同文件内联与跨模块 import 等同）返回被调 fn 的真实计算结果；参数逐位绑定。
2. **失败显式**：符号缺失/执行错误以错误形态抛出（handler 域可 try/catch 捕获），
   不静默降级为空列表/空串。
3. **H2 统一栈编码**（ADR-06/PLAN-733 重申）：Rust 桥推入 VM 栈的一切堆引用一律
   `encode_object`（TAG_OBJECT）；`encode_list`（TAG_LIST）不是合法栈编码——engine
   数值操作臂（ARRAY_LEN/GET_ELEM/for-in 头部）只识别 TAG_OBJECT/裸 i32。

## 语义矩阵（实证锚）

| 调用域 | 值形态 | 语义 | 证据 |
|---|---|---|---|
| store handler 域（含 timer Poll/Tick 跨轮次） | 同文件 fn，0/1/2/3/6 参，列表首参 | ✅ 真值返回 | p733 矩阵测试 `f_zero/f_one/f_two/f_three/f_six` |
| store handler 域 | 跨模块 import fn（2 参/6 参列表首参） | ✅ 真值返回 | `f_twox/f_sixx` + 探针 `fixedx=2` |
| store handler 域 | fn 结果消费：`.length` / `== []` / for-in / 元素成员读 | ✅ 真值 | 消费探针格 `c_*` |
| 跨 handler 轮次 | 列表字段写（param/literal/局部构建/fn 结果）→ 下轮 for-in/`.length` | ✅ 存活（VmRef 持有约定 + rc 记账） | `cross_*`/`w_*`/`tick_*` 格 |
| 视图 computed（表达式） | 纯表达式 | ✅ 真值渲染 | `plain => "P"+n` |
| 视图 computed（表达式） | fn 调用，**标量实参** | ✅ 真值渲染 | `cc`（含对象字面量实参，PLAN-733 补 `Expr::Object` 臂后） |
| 视图 computed（表达式） | fn 调用，**列表实参（状态字段读/prop 读）** | ✅ 真值渲染（PLAN-733 修复：`call_vm_fn` 栈编码 encode_list→encode_object） | `cc2`；修复前恒空（musk 附页①"computed 内函数调用❌恒空"运行时根因） |
| store handler 域 | 同文件 fn 裸调用，**跨模块同名歧义**（同名 fn 另存于其他模块） | ✅ own-module 绑定（PLAN-733 修复：store_scope_module——合成 store handler/computed 隐藏 fn 时设 codegen 绑定域，resolve_call_symbol 步骤 2.5 路径） | `f_dup` 格 + musk 实链（修复前 `Undefined symbol: <bare>` 整链接失败，canvasProgressRows 内联实验实测） |
| 视图（模板/行文本） | `t(<动态键>)` i18n 查表（`t(r.label_key)` / `t("prefix_" + r.state)`） | ✅ 真值渲染（PLAN-733 修复：t 臂动态键解析——此前非字面量键落空→行文本空被过滤） | musk 实链收据「启动预览 已可见」（t 动态键两形态全真值） |
| 块体 computed | 隐藏 VM fn（`__computed_<W>_<p>`，Plan 448 H2） | ✅ 真值（`call_computed_fn` 路径） | 既有 plan448 面 |

## 已知边界（登记，非本契约承诺面）

1. **懒挂载时序**（PLAN-711 pending demand）：子件经 prop 通道推送的数据在第 2 帧
   后才就位——首拍 timer 读到空列表是**时序**不是语义缺陷。消费方模式：每拍重算
   （无一次性门/has 冻结）自愈于下一拍。实证：探针 `frb=0 → rb=3`。
2. **Rust 外部派发的堆引用实参**：`push_value`（vm_bridge）对 VmRef/Array 落
   `push_i32(0)` 占位（自述"not passed as scalar args"）——从 Rust 侧带列表实参直接
   派发 handler（`call_widget_handler` 直发形态）时 handler 收到 0。既有已知约定；
   主链（视图动作/MCP action）只传标量。需要时应走 `call_vm_fn`（正确编码）。
3. **预存红族 musk_vm_track p053_1/p053_4/p053_6/p054**（master 3053f1fdf 基线即红，
   PLAN-733 修复前后零行为变化）：computed 链式 helper 的深层腿（子件 override 态
   `.store.X` 解析链等）未随本案绿；其断言面（musk chats 消息列表 computed 形态）
   作为后续计划指针。
4. **视图 `.x.length` 模板成员链**（附页①矩阵⑤表现层）：engine GET_FIELD 的
   ListData `.length` 臂已存在（plan-022），视图侧 Rust 求值器 `materialize_obj_ref`
   对 ListData 引用原样透传（只物化对象）——模板位 `.length` 求值链未全覆盖，
   独立于 fn 调用语义，分层归因保留。

## 维护

- 矩阵回归锚：`crates/auto-lang/src/tests/plan733_fn_call_semantics_tests.rs`
  （日常档 `cargo t` 面，`ui-iced` feature）。
- 改 `call_vm_fn` 实参编码/新增堆引用实参形态时，先对照本矩阵补格，再动实现。
