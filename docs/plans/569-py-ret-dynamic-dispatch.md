---
plan_id: PLAN-569
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: py-ret-dynamic-dispatch
author: [zhaopuming]
created_at: 2026-09-06
updated_at: 2026-09-06

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm, auto-lang/trans]   # 受影响的 specs 路径
current_step: 0
total_steps: 13
---

# [PLAN-569] P539-D2 根治：py 返回值方法分派动态化——py-类型侧表 + 组合子双通道路由

## 变更摘要

清偿 [KNOWN-DEBT P539-D2](KNOWN-DEBT-AND-RISKS.md)（roadmap §7.4-③ 根治路线）：py 桥
调用返回值被 codegen **谎记为 String**，`.len()` 等方法调用静态路由到 `str.len`
原生，把句柄/列表堆 id 解码为字符串池索引读到垃圾（实测返回 "20"）。20 个 py 套件
以 `py_call(x,"__len__")`/for-in 计数/`x[0]` 三惯用法规避（8 套件 README 成文）。

根治方案（不撒大网、不动 ObjectType 枚举）：**codegen 侧 py-类型侧表** —— py-ffi
调用点/经用户函数返回传播的值携带 "py 可能" 语义；方法分派决策核
（`infer_object_type → "{type}.{method}" 限定名 peek` 三连）在静态路由前先查侧表，
py 可能接收者的方法调用改发 **obj_len / obj_call 分发组合子**（Plan 555 W1 已建、
PyObjectHandle 的 ForeignObject 适配器已实现、`.as` 管线已接线——地基全量在位，
缺的只是让谎言面不再把接收者送进 str.* 死路）。带手感的 shim 兜底加固
（str.len 探测到 PyObjectHandle → GIL len）随批。

## 目标

- py 桥返回值（use.py 导入项调用 / py_call 族 / 经用户 fn 返回传播）上的方法调用
  `.len()` 及任意 `.method()` 在 **.at 与 .as 两种模式**下都按运行期 tag 双通道
  分派：PyObjectHandle → GIL 通道，Auto 值 → 原生方法表——不再读到字符串池垃圾。
- `var t = arange(6)` 后 `t.len()` 输出 6（当前输出垃圾数字）；套件惯用法
  `py_call(x,"__len__")` 可以被 `.len()` 正式替换（py_list 套件示范去规避 ×4 处）。
- 规避正名不破坏三方一致：py_list 套件改用 `.len()` 后 a2py/oracle 逐字对齐。
- 零回归红线：Auto 值（str/List/struct）的 `.len()`/方法分派路径行为逐字节不变；
  全部 20 个 py 套件 + tv/tt/tf 门禁零新增红。

### 非目标

- 不引入 ObjectType::PyHandle 枚举变体（侵入全体 exhaustive match；侧表方案零枚举改动）。
- 不改 py_known 的 .as 分析面（codegen 层修复覆盖双模式后，s2s 层 py-known 门
  维持现状）；不改 GET_ELEM（已有 PyObjectHandle 臂，`x[0]` 规避本就正确）。
- 不处理 handle_rust_import 同族谎言（use.rs 未知签名 StrFixed——独立债面，顺登记）。

## 架构方案

沿用 555 W1 的组合子架构不动：**语义本体在 interop.rs 双通道组合子 + py_ffi 的
ForeignObject 适配器**；本计划只补"接收者何时算 py"的静态知识与其在分派决策核的
优先权：

```
方法调用 .m(args) 编译期决策（codegen.rs:7889-7944 决策核）
  ├─ ① 查 py-类型侧表（本计划新增）：接收者 py 可能？
  │     ├─ 是 + method==len → 发 obj_len（interop 1863，运行期 tag 双通道）
  │     └─ 是 + 其他方法   → 发 obj_call（interop 1860，外对象臂 call_method /
  │                          Auto 臂报 TypeError——与 W1 现状一致）
  ├─ ②（原路径）infer_object_type → "str.len" 等限定名 peek 静态路由
  └─ ③（原兜底）CALL_SPEC 动态分派
```

侧表的三种 py-可能来源（对齐谎言四个写入面中的三个运行期真实面）：
1. **py-ffi 调用点**：`is_py_ffi_call` 命中处（codegen.rs:8425-8437 py_native_map 臂）
   记 `last_expr_may_py = true`；`PyType::Auto | String → ObjectType::String` 的
   last_expr_type 谎言**保留**（extract_autovm_result 顶层结果格式化依赖它）。
2. **变量落盘**：Store 时 `last_expr_may_py` 传导进 `py_typed_vars: HashSet<String>`
   （与 var_types 平行，不替换）。
3. **用户函数返回传播**：编译 fn 体时尾表达式 may_py → `fn_may_py_returns:
   HashSet<String>`；调用点查此表传导（单层函数流型，分支不敏感保守正报——
   错报代价=obj_call 双通道对 Auto 值照常分派，安全；漏报代价=维持现状规避）。

## 技术栈

Rust（crates/auto-lang/src/vm/codegen.rs 分派决策核与侧表 / vm/interop.rs 组合子
（零改动，仅消费）/ vm/native.rs shim_str_len 兜底加固 / trans/python.rs a2py
`.len()` 反映射核对）；parity py_list 套件去规避示范。

## 需求分析与背景调查

（取材：KNOWN-DEBT P539-D2 条目、roadmap §7.4-③、2026-09-06 代码调研）

- **分派决策三连**：`codegen.rs:7889` `infer_object_type(obj)` → `:7900-7928` 映射
  type_name（String→"str"）→ `:7929` `native_name = "str.len"` → `:7932-7933`
  `BIGVM_NATIVES.peek_qualified` 命中即静态路由。`Array.len()` 特例直发 ARRAY_LEN
  （:7891-7898/:7937-7944）；解析失败落 CALL_SPEC 动态（:9399-9563）。
- **谎言四个写入面**：① 导入期 `handle_py_import` :5037
  `fn_return_types.insert(name, Type::StrFixed(0))`；② 调用点 :9087-9103——
  py_native_map 命中后 `PyType::Auto | String => ObjectType::String`（:9100-9101，
  注释自认 "Auto and String both use string pool"）；③ 变量落盘 :2093-2096——
  last_expr_type==String 且 ty==Unknown → var_types StrFixed；④ 用户 fn 显式
  `-> str` 经 :1495-1497 传播（build_fn_return_types :12969-13037 另有批量面）。
- **规避惯例分布**：`py_call(x,"__len__")` 8 套件（configparser/json/list×4/os/
  re×3/string/pandas/torch_train），README 映射表成文（py_list README:26 等）；
  for-in 计数（train.as:43,60,115 + 文件头 :17-18 明文注记 P539-D2）；`x[0]` 索引
  （GET_ELEM :5035-5068 有 PyObjectHandle 臂——规避为何可用的根因）。
- **组合子地基（interop.rs，575 行）**：ForeignObject 协议 :24-82；双通道判定
  `dispatch_foreign` :100-118（堆对象 downcast as_foreign_object）；obj_call :238-275
  （外对象臂 fo.obj_call / **Auto 臂现报 "not callable"** :270-274——Auto 方法面
  维持 W1 现状）；obj_len :278-339（外对象臂 GIL len / Auto 臂 str 字符数+ListData
  四型）；py 侧适配器 py_ffi.rs:52-223（obj_len=GIL len :119-131）。native id
  1860-1865（interop.rs:91-97），别名 native_catalog.rs:1728-1739。
- **shim 补丁式缓解现场**：`shim_str_len` native.rs:5332-5388 头部堆句柄探测
  （is_object/is_list → ListData 分派，注释自述病灶），但 PyObjectHandle 落
  :5366-5369 的 0 臂——本计划顺修为 GIL len（带手感兜底）。
- **历史试验**："RuntimeArray 谎言翻转试验无效已回退"仅存债务条目文字，代码零遗迹
  （git -S 无提交）——侧表方案与其正交（不动类型落盘，只加分派优先权）。
- **s2s 现状**：B6 `len(x)→obj_len(x)` 无条件改写（s2s_rules.rs:279-287，函数形态）；
  方法形态 `.len()` 在 .as 里因 py-known 门（recv_is_py）只对已知接收者改写——
  py_known 不跨函数边界，正是本计划 codegen 层补的洞。

## 详细设计

### A. codegen 侧表与决策核插队（核心）

**A1 侧表三件**（`Codegen` 结构体新字段，编译期纯静态、随会话生灭）：
`last_expr_may_py: bool`（表达式编译的粘性伴随位，随 last_expr_type 重置点同步）、
`py_typed_vars: HashSet<String>`（var x = <may_py expr>）、`fn_may_py_returns:
HashSet<String>`（fn 名 → 尾表达式 may_py）。写入点：py_native_map 命中臂
（:8425-8437）置位；Store 落盘（:2093-2096 邻域）传导；FnDecl 编译尾表达式后回填；
函数调用点（infer_object_type(Expr::Call) 消费 fn_return_types 的 :11535-11585
邻域）查表传导。重置纪律：凡 `last_expr_type = ...` 显式赋值处对照置 false（仅 py
臂置 true）——逐点审计 :9045-9103 与 :1252-1253（函数体首重置）。

**A2 决策核插队**（:7889 `infer_object_type` 调用之前）：接收者 may_py 判定
（Ident 查 py_typed_vars；Call 查 fn_may_py_returns；其余窄形态用 last_expr_may_py）
→ 命中且 method=="len" 发 obj_len（CALL_NAT_COUNTED, id 1863, 1 实参）；命中其他
方法发 obj_call（id 1860，接收者+方法名+实参——与 s2s A1 产物同构）；随后 return
不走 str.* 静态路由。Auto 值误入臂的语义：obj_len Auto 臂=ARRAY_LEN 语义（str/List
照常）；obj_call Auto 臂="not callable" TypeError——**与 550 callable 守卫一致**，
比静默垃圾响亮。保守正报可接受性由此成立。

**A3 导入期谎言不撤**：:5037 StrFixed 保留（顶层结果格式化依赖），侧表在调用点
覆盖分派行为——谎言只影响"打印成什么"，不再影响"路由到哪"。

### B. shim 兜底加固（防 .at 存量编译产物与漏报）

`shim_str_len`（native.rs:5332-5388）PyObjectHandle 臂：0 → `Python::attach` GIL
`len(handle)`（与 obj_len 外对象臂同语义）；注释更新为"根治归 PLAN-569，此臂为
存量兜底"。

### C. a2py 核对与套件去规避

- a2py `.len()`：核对 trans/python.rs 方法调用发射对 `.len()` 的现有映射（应为
  `len(x)`）；若缺则补 `.len()` 方法名特判臂（py 对象上无 `.len` 属性，直映会
  AttributeError）。
- py_list 套件示范：`py_call(lst,"__len__")` ×4 → `lst.len()`；README 映射表行
  更新（"`lst.len()` ↔ `len(lst)`（PLAN-569 起可用）"）；train.as 文件头注记更新。
  其余 7 套件去规避**不做**（规避仍正确，批量翻写属噪音批次——留待各套件自然触碰）。

### D. 债务核销

P539-D2 → ✅ 已清偿（本计划链接 + 套件实证）；use.rs 谎言面顺登记新债条目
（handle_rust_import :4924-4943 同族，独立偿还路径）。

## 测试设计

- **codegen 单测**：py-ffi 调用结果 `.len()` 编译产物含 obj_len id 1863（反汇编
  断言）；Auto str `.len()` 仍路由 str.len；经用户 fn 返回的 py 值 `.len()` 路由
  obj_len；Auto fn 返回 str `.len()` 不受影响（fn_may_py_returns 空）。
- **tv .as 语料**（新 99_py_dispatch 类目）：`var t = arange(6)` `t.len()` 输出 6；
  `var s = "abc"` `s.len()` 输出 3（双通道两臂各一例）。
- **engine 行为**：obj_call Auto 臂 TypeError 可 catch（.as catch 载荷断言，
  99_script_err 增例）。
- **parity**：py_list 8/8（去规避后三方逐字一致）；p5-p9 全 20 套件复跑零回归。
- **合入前**：`cargo tf` 一次（codegen 核心路径改动）+ `cargo tt`（a2py 面）。

## 验收标准

1. `var t = arange(6)` 后 `t.len() == 6`、`t.sum()` 等方法链照常（.at 直跑与 .as
   lowered 双形态）；`var s = "abc"` `s.len() == 3`（Auto 臂零变化）。
2. 用户 fn 返回 py 桥值（如 `fn get() -> str { return <py 调用> }`）的返回值
   `.len()` 路由 obj_len（跨函数流型传导实证）。
3. py_list 套件 `.len()` 去规避后 8/8 三方绿；p5-p9 五相位 20 套件零回归。
4. `cargo t` / `cargo tv` / `cargo tt` 零新增红（对照 master 基线；charts 既有红
   除外）；合入前 `cargo tf` 绿。
5. KNOWN-DEBT P539-D2 ✅ 已清偿（附本计划链接）；use.rs 谎言族新债登记在案；
   py_list/README 惯用法表更新。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [ ] **T01** codegen 侧表三字段 + 写入点：`crates/auto-lang/src/vm/codegen.rs`
  Codegen 结构体加 `last_expr_may_py/py_typed_vars/fn_may_py_returns`；py_native_map
  命中臂置位；last_expr_type 显式赋值处对照审计重置。
  验证：`cargo check -p auto-lang`（零新错误）。
- [ ] **T02** Store/fn 返回传导：Store 落盘邻域（:2093 附近）py_typed_vars 写入；
  FnDecl 尾表达式回填 fn_may_py_returns；调用点（:11535 邻域）查表传导。
  验证：`cargo check` + fn 流型传导单测。
- [ ] **T03** 决策核插队：方法分派决策（:7889 前）may_py → obj_len/obj_call 发射
  （len 特判 obj_len；其余 obj_call 与 s2s A1 产物同构）。
  验证：`cargo t` codegen 新单测 ×4（py-len/py-method/auto-str/auto-fn）。
- [ ] **T04** shim 兜底：`vm/native.rs` shim_str_len PyObjectHandle 臂 0 → GIL len。
  验证：`cargo t` str_len 相关既有测试绿。
- [ ] **T05** a2py `.len()` 反映射核对/补臂：`trans/python.rs` 方法调用发射核对
  `.len()→len(x)`；缺则补。验证：`cargo tt`。
- [ ] **T06** tv 语料：`crates/auto-lang/test/vm/99_py_dispatch/`（01_dual_len/
  02_auto_str_unchanged 两例 .as + expected.out + 测试函数）。
  验证：`cargo test -p auto-lang --lib --features test-vm-files,python test_99_py_dispatch`。
- [ ] **T07** obj_call Auto 臂可 catch 断言：99_script_err 增例（Auto 值误入
  obj_call → TypeError catch 载荷）。验证同 T06。
- [ ] **T08** py_list 去规避：`parity/libs/python/py_list/tests/auto/list.as`
  `py_call(lst,"__len__")` ×4 → `lst.len()`；README 映射表更新。
  验证：`cd parity && cargo run -- run py_list` 8/8。
- [ ] **T09** train.as 文件头注记更新（P539-D2 已清偿口径）+ 其余套件 README
  惯用法行统一注记（不改测试体）。验证：grep 注记在位。
- [ ] **T10** 全 parity 复跑：p5-p9 五相位零回归。验证：parity 报告全 100%。
- [ ] **T11** 门禁：`cargo t --no-fail-fast` 对照基线零新增 + `cargo tv` + `cargo tt`。
- [ ] **T12** 债务核销：KNOWN-DEBT P539-D2 ✅（附链接）；use.rs 谎言族新条目登记。
- [ ] **T13** 合入前 `cargo tf` 全量 + roadmap §7.4-③ 状态回写。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

1. **fn_may_py_returns 跨层深度**：首批只做单层（fn 尾表达式直呼 py-ffi），
   fnA→fnB→py 的两层链是否传导？推荐首批单层（保守漏报=维持规避，安全），
   多层流型按需求另批。
2. **obj_call Auto 臂语义**：Auto 值误入（保守正报）现报 TypeError "not callable"
   ——是否扩为"回落 CALL_SPEC 动态分派"的软臂？推荐维持硬臂（响亮优于静默，
   与 550 守卫一致），软臂留待实证误报频率后再议。
3. **行为变化公告**：`.len()` 等在误标 py 的 Auto str 上从"读垃圾"变"TypeError
   正报"（或正确分派）——推荐随计划提交信息公告即可，正报优于垃圾读。
