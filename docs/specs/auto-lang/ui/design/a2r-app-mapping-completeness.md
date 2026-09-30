# a2r 应用映射完备性契约（host-call / try-catch / envelope 投影三面）

> PLAN-710 供①（2026-09-30）。auto-edit M4 供料包供①的承接件：三类
> 生成缺口清偿后 `auto build -r rust` 对真实工程 corpus（auto-edit
> main@70c5c60）**exit 0**（普查对照 133→0，docs/reports/p710-census.md）。
> 本册为 a2r 域的应用映射完备性真源——shell-a2r-seams.md（seam 面）的
> 完备性姊妹册。**语义同源纪律（frozen）**：a2r 发射形与 VM 解释轨逐形
> 同语义——G-C 直调 core 实现（不绕 VM shim）、G-A/G-B 语义对拍以
> VM 值语义（`auto_val::Value::is_true` 等）为锚。

## 1. G-C：宿主调用表（`vm_builtin_host_call`）delta 臂

- **单源面**：`ui_gen/rust.rs` `vm_builtin_host_call` 表 = a2r 内建
  host-call 的唯一映射点；新内建入表即得双轨覆盖（671 T-06/674 §2
  红线沿用）。
- **delta 臂形**（PLAN-710）：
  `{ let __key = ({a0}).clone(); auto_lang::ui::code_editor::code_editor_delta(&__key).unwrap_or_else(|| panic!("code_editor_delta: no editor registered for key {:?}", __key.as_str())) }`
  ——直调 core 实现（`code_editor::code_editor_delta`，Option<String>
  destructive read）；键值块内预绑一次（参数表达式单次求值，与 VM
  pop 一次对齐）。
- **Option→String 同源映射**：VM shim（native.rs `shim_code_editor_delta`）
  未注册键 = `VMError::RuntimeError("code_editor_delta: no editor
  registered for key {key:?}")`——a2r 形 `unwrap_or_else(panic!)` 保同
  消息（`{:?}` 引号形一致）；**非** `unwrap_or_default()`（空串形会静默
  偏离——错误即错误）。
- **五面注册核**（703 家族）：catalog/bigvm/intrinsics/shim/静态表 delta
  自 673 在位；本册增 ui_gen 臂后五面齐。

## 2. G-B：try/catch 语句臂（catch_unwind 形）

- **发射形**：
  `match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { <try 体> })) { Ok(_) => {}, Err(__p) => { <catch 体> } }`
  （back_proxy.rs 先例）。try 体照 join_stmt_block 规则直发——**成功路径
  零语义扰动**。
- **变量逃逸形**：try 外声明、try 内赋值（corpus `fsize` 株）——闭包
  `AssertUnwindSafe` 捕 `&mut` 回写；外层 `let mut`（Var 发射形）成立。
- **catch 绑定形**（`catch (e)`）：`let e = auto_lang::a2r_std::panic_message(&__p)`
  ——载荷消息串。VM 侧 catch 帧推入错误消息字符串（vm/codegen.rs
  `Stmt::Try` STORE_LOCAL 形）——两侧同源。无绑定形（corpus 实例集
  全形态）载荷不消费。
- **finally**：跟发于 match 之后；**两侧同形偏差**——catch 体内错误不
  触发 finally（VM 无嵌套 handler 帧；a2r catch 块 panic 直接传播）。
- **边界（SD 注记，实例集外不扩）**：try 体闭包内 `return`/`break`/
  `continue` 不出闭包（语义改变）——corpus 零实例；出现即生成器已知
  边界，需显式扩展（242 tracker 归口）。非 Dot/Index 链的 NullCoalesce
  同为认知面外（落占位显式拦截）。

## 3. G-A：envelope 成员访问投影（NullCoalesce 族）

- **发射形**：左链 Dot/Index 展平 → `a2r_std::json` 路径切片/组合：
  - 成员链 `v.f` / 嵌套 `v.a.b` → `get_str_or(&recv, &["f"...], default)`
    族单调用（路径切片）；
  - 索引混合 `rows[i].f` → `get_owned`/`at_owned` 逐段组合；Vec<Value>
    态基座首索引走原生下标；
  - 接收者按 envelope 契约视作 `serde_json::Value`（value_local/Value
    循环变量/未知名一视同仁——corpus 防御式 envelope 风格的成员读全部
    coalesce 门控）。
- **类型投影按 ?? 右值四族**：

| 右值形 | helper | Rust 型 |
|---|---|---|
| `"lit"` | `get_str_or(&v, keys, "lit")` | String |
| int 字面量（含 `-1`——Unary(Sub) 入臂） | `(get_int_or(&v, keys, d) as i32)` | i32 |
| `true`/`false` | `get_bool_or(&v, keys, d)` | bool |
| `[]` | `get_array_or(&v, keys)` | Vec<Value> |
| 调用等非字面量 | `get_str_or_with(&v, keys, || expr)` | String |

- **缺省语义同源**：路径缺段/字段缺席/类型不符 → ?? 右值（VM 侧缺字段
  → Null → coalesce 右值的防御性收敛——envelope 契约面通用件）。
- **条件位投影（D-6）**：`if r.field`（Value 族裸读条件）→
  `a2r_std::json::truthy(&(r["field"]))`——`Value::is_true`（auto_val
  value.rs:682：Bool 直读/数值>0/串非空/其余 false）同源。挂点：
  convert_condition（view 条件串重写）、cond_expr_rust（stmt If/While、
  if-expr 条件）；逻辑组合逐原子递归，比较位维持既有 Eq/Neq 降串归一。

## 4. corpus 三重判定面（机读判据）

`auto build -r rust`（工程副本，工具链判据前核构建源）：
1. **exit 0**；
2. 生成物 grep 零 `/* expr */` / 零 `/* unhandled stmt */` / 零裸
   `code_editor_delta`；
3. 生成 workspace `cargo check` 过。
三重缺一不可。实机留痕：docs/reports/p710-census.md（133→0 对照表 +
回补面 D-4..D-8 归因——scroll_to 臂/scroll-pane controller 臂/条件位
投影/merged GET-POST 桩契约收口/char_at·substr·to_int 臂）。

## 5. 对拍锚

`crates/auto-lang/src/tests/plan710_supply_probes.rs`（11 测试）：G-C
envelope/watermark/错误消息、G-B 两臂+绑定、G-A 五段投影+truthy 逐格
（VM 轨 vs a2r helper 同 fixture）+ 发射形七断言。

## 6. 已知边界与下游件

- merged back-api 桩面为**记录+缺省**形态（GET/POST 按契约签名收口，
  运行期真值由 merged back 服务端供）——route A（api_impl 真实现委派）
  的 fsys.at 转译链启用属后续件（census §3 D-7 注记）。
- 下游消费件（auto-edit `perf.py a2r` exit 0 复验 + L2 锚点补跑 + 装载
  链/diff 视图矩阵回归）= 669 模式下游件，非本册边界。
