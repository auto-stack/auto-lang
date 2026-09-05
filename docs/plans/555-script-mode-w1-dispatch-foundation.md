---
plan_id: PLAN-555
status: reviewed            # drafting → executing → execution_done → reviewed → archived
feature_name: script-mode-w1-dispatch-foundation
author: [zhaopuming]
created_at: 2026-09-05
updated_at: 2026-09-05

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "specs/auto-lang/vm/overview.md: 修改 —— 现状节补 Plan 555：ForeignObject 协议（HeapObject::as_foreign_object 默认钩子 + PyObjectHandle 六操作臂首实现）与分发组合子六件（interop.rs，ID 1860-1865，运行期 tag 分派 py 桥/Auto 原生方法表）"
  - "specs/auto-lang/frontend/overview.md: 修改 —— 现状节补 Plan 555：ScriptMode 八格矩阵（.as≡隐式 #[script]，#[rust] 显式压回，优先序 rust>script>扩展名）+ parser rust_pragma 文件级 pragma + CompileSession.script_mode 回填"
  - "specs/auto-lang/trans/overview.md: 修改 —— 现状节补 Plan 555：auto_s2s 改写器骨架（LoweringRule 规则表 W1 空置/词法 tokenize_all 改写面/identity 逐字节发射/三单测）"
  - "specs/auto-cli/project.md: 修改 —— `auto trans --path x auto [-o]` 子命令 + `--dump-lowered` 全局 flag（模式头渲染）"
new_spec_components:
  - "specs/auto-lang/vm/design/interop-dispatch.md: 新增 —— 互操作分发契约：ForeignObject 协议六操作面（send/contains 预留位）+ 组合子语义表（六操作×py/Auto 双通道）+ CALL_PY 计数传输形态（1860-1865 ID 段）+ py 三桥 467-469"
touched_goals:
  - "GOAL-005: W1 动态分派地基——组合子+py 三桥让 .as 管线可消费 539 桥族；py 五套件三方 64/64 零回归"
  - "GOAL-006: Consumer-mode parity——分发组合子即消费者模式对象协议雏形（§8 跨语言矩阵承接位）"

affects: [auto-lang/vm, auto-lang/frontend, auto-cli]   # 受影响的 specs 路径
current_step: 11
total_steps: 11
---

# [PLAN-555] 脚本模式 W1：动态分派地基（ForeignRef/组合子/.as 管线/s2s 骨架）

## 变更摘要

脚本模式（AutoScript，设计源 [script-mode-interop.md](../design/strategy/script-mode-interop.md)
§1/§2/§8/§9）实施波次的第 2 波（W1，前波 W0=Plan 550 null 地基已归档）。
落地四件地基设施，**零行为变更承诺**——现有 `.at` 语料与 py 五套件不受影响：

1. **模式管线**：`.as` 扩展名识别（≡ 隐式 `#[script]`）+ `#[script]`/`#[rust]`
   pragma 覆盖语义，接入 550 落成的 `CompileSession.script_marked` 通道；
   W1 语义 = passthrough（脚本语义激活在 W2 lowering 规则批）；
2. **ForeignObject 协议位 + 分发组合子**：`obj_get/obj_set/obj_call/obj_len/
   obj_iter/obj_type_name` 六个正常模式函数（native registry 注册），
   运行期 tag 分派——py 句柄 → py_xxx 桥、Auto 值 → 原生方法表；
   `send/contains` 协议位预留（跨语言矩阵 §8 承接点）；
3. **py 桥三补**：`py_setattr`(467)/`py_len`(468)/`py_type_name`(469)
   （539 桥型同构：GIL 闭包 + marshal 标准回程）；
4. **s2s 改写器骨架 + `--dump-lowered`**：`auto trans --to auto` 独立
   source-to-source 工具（Babel 模式），规则表空置 + per-rule 单测框架；
   `--dump-lowered` 输出改写产物（W1 = passthrough + 模式头）。

## 目标

1. `auto x.as` 直跑可用：与同内容 `.at` 行为逐字节一致（passthrough），
   script 标志入 compile session；
2. 模式解析矩阵（扩展名 .at/.as × pragma 无/`#[script]`/`#[rust]` 八格）
   单测钉死，`#[rust]` 可将 `.as` 覆盖回正常模式；
3. 六组合子双通道正确：py 句柄与 Auto 值（list/str/map）同一函数名工作；
4. py 三桥落地且 py_torch 系套件零回归；
5. s2s 骨架可扩展：插入一条测试规则即生效（单测证明），identity 改写
   round-trip 稳定；
6. `--dump-lowered` 在 run/trans 双入口输出改写产物。

### 非目标（Out of Scope，归 W2/W3）

- W2 糖批：A1-A5/B1-B9/C3-C7/D7/E2-E3 lowering 实规则全表；
- B8 裸模块句柄（`use.py torch` 绑模块对象，重做 Plan 300 腐烂面）；
- 跨模式 import 物化（`.at`→`.as` 导出自动 `!T`、`.as`→`.at` 签名检查）；
- 隐式 `!T` 传播 + Err 值通道（catch 拦值绑定结构化载荷）——与 W2
  lowering 发射点同批（见待澄清#1）；
- 窥孔零开销直达优化（静态已知 py 调用点直达 py_xxx）——见待澄清#2；
- JS/ArkTS ForeignObject 实现（仅协议位）；py 五套件 `.as` 改名迁移
  （W2）；`.at` 含 use.py 硬诊断（维持 550 警告级直至 W2 迁移完成）。

## 架构方案

### 现状基线（考古结论）

- **py 桥族**（539 沉淀）：natives 450-466（py_call/getattr/call_kw/call_may/
  iter/next/matmul/getitem/setitem/slice/call0/with/enter/exit/item_kw/
  float/callable），`PyObjectHandle` 为堆对象（`heap_objects` 注册表，
  GIL 闭包 + marshal 标准回程）；**py_setattr/py_len/py_type_name 缺席**
  （设计 §3 B2/B6/D8 待建行）；
- **模式面**（550 沉淀）：`#[script]` pragma → `parser.script_pragma` →
  `CompileSession.script_marked`；生产者门控 lint（use.py/null/nil 三信号
  stderr 警告，pragma 豁免）；`.as` 扩展名**尚未接入任何管线**；
- **multi_mode.rs**（Plan 081）：依赖编译模式管线（AutoVM/C/Rust），非
  脚本模式管线——设计 §1 所指"折进编译器模式管线（multi_mode.rs）"是
  W2+ 稳定后的事，W1 的 s2s 独立先行；
- **`auto trans`**：单文件转译 CLI 在位（main.rs `Commands::Trans`），
  s2s 骨架挂 `--to auto` 目标位；
- **PyObjectHandle 无协议抽象**：组合子需要的"运行期判外对象并分派"
  目前只能 isinstance 式 downcast，需要 ForeignObject 协议层收口。

### 设计要点

1. **模式 = 文件级信号**：扩展名为主 + pragma 覆盖（`#[script]` 提升、
   `#[rust]` 压回）；解析序 = 扩展名 → pragma 覆盖 → session 标志。
   W1 的 `.as` 不改变任何语义（passthrough），只立信号通道与测试矩阵。
2. **组合子在函数层，不在 codegen 热臂**（裁决"架构"行）：组合子是
   native registry 注册的普通函数（`interop.obj_get` 家族命名，见待澄清#3），
   内部查运行期 tag：PyObjectHandle → ForeignObject 适配 → py_xxx；
   Auto 值（list/str/map/object）→ 原生方法表（GET_ELEM/GET_FIELD/
   ARRAY_LEN 语义复用）。**这是跨语言矩阵（§8）的承接点**——换
   ForeignObject 实现即接新宿主。
3. **s2s 独立工具先行**（Babel 模式）：parse → AST → 规则表（按
   A1..F4 编号键控的 lowering 规则注册表，W1 全空置）→ emit 正常模式
   `.at`；每条规则配 source-to-source 单测（539 单测同型的产物断言）；
   稳定后折 multi_mode，不进 codegen 热臂（539 三次栈溢出教训）。
4. **py 三桥沿用 539 桥型**：`py_ffi.rs` GIL 闭包 + `marshal_pyany_to_stack`
   标准回程；setattr/len/type_name 均为 GIL 单调用，无新封送形态。
5. **零行为变更红线**：全部改动对现有 `.at` 语料不可观测（组合子是新
   函数、`.as` 是新扩展名、pragma 是新通道）；py 五套件与 tv/tt 门禁
   必须原样全绿。

## 技术栈

- `crates/auto-lang/src/mode.rs` + `compile.rs`（模式解析矩阵 + session 联动）；
- `crates/auto-lang/src/lib.rs` + `crates/auto/src/main.rs`（`.as` 直跑 +
  `--dump-lowered` flag，run/trans 双入口）；
- `crates/auto-lang/src/py_ffi.rs`（py_setattr 467 / py_len 468 /
  py_type_name 469 + ForeignObject 协议 trait + PyObjectHandle 适配器）；
- `crates/auto-lang/src/vm/interop.rs`（**新模块**：分发组合子六件 +
  协议位 send/contains）+ `vm/codegen.rs`（native 注册面）；
- `crates/auto-lang/src/trans/`（s2s：`--to auto` 目标 + 规则表骨架）；
- 探针：`scratch/p555/`（550 惯例，先证现状再改码）；
- 回归：`cargo tv` + `cargo tt` + py 五套件三方 + `cargo t vm`。

## 需求分析与背景调查

（从 spec ledger overview 与设计源取材）

- **设计源**：`docs/design/strategy/script-mode-interop.md` §1（糖→桥
  lowering 架构与组合子定位）、§2（`.as` 模式体系裁决）、§8（跨语言
  矩阵：ForeignRef+组合子=可移植层）、§9（波次骨架 W1 行）、§10
  （裁决存档：架构/s2s 先行/不进热臂）；
- **goals 关联**：GOAL-005（Python parity 线——W1 是 539 桥族之上的
  表面体验地基）；GOAL-006（Consumer-mode parity——组合子即消费者
  模式的对象协议雏形）；
- **前波沉淀**：P539-1..6（桥族 450-466 + 三方套件形态）；P550-1..6
  （null 家族守卫 + `#[script]` pragma 通道 + 生产者门控）——W1 直接
  消费 550 的 `script_marked` 通道与门控 lint；
- **债务联动**：P550-D4（CALL null 守卫端到端探针面要等 W2 动态分派
  语义激活）；P550-D6（pragma 位置语义与 `.as` 扩展名联动——本计划
  T02 裁定并销号）；
- **先行考古**：py natives 注册表在 `vm/codegen.rs:5307-5321`（450-464）
  + `py_ffi.rs:112/116`（465/466），新桥自 467 起；`auto trans` 入口
  `main.rs Commands::Trans`；`multi_mode.rs` 为 Plan 081 依赖模式管线。

## 详细设计

### 模式解析矩阵（T02 契约）

| 扩展名 | pragma | 解析结果 | 语义（W1） |
|---|---|---|---|
| `.at` | 无 | Normal | 现状（550 门控 lint 照常） |
| `.at` | `#[script]` | Script | passthrough + script 标志（550 已落） |
| `.at` | `#[rust]` | Normal | 现状（rust pragma 已有语义不动） |
| `.as` | 无 | Script | **passthrough** + script 标志 |
| `.as` | `#[script]` | Script | 同上（幂等） |
| `.as` | `#[rust]` | Normal | 覆盖回正常模式（显式压回通道） |

- 解析产物落 `CompileSession`（扩展 `script_marked` 或并列
  `script_mode: ScriptMode` 枚举，保留 550 字段兼容）；
- `.as` 文件的 550 门控 lint 自动豁免（script 标志即豁免位，无需新码）。

### 分发组合子（T05/T06 契约）

```
正常模式 Auto（组合子层）              运行期 tag 分派
────────────────────────            ─────────────────────────
obj_get(x, key)          ──▶  PyObjectHandle? ─ py_getattr
                              list/str/map/object? ─ GET_ELEM/GET_FIELD 语义
obj_set(x, key, v)       ──▶  py? py_setattr(467 新) : SET_ELEM/SET_FIELD
obj_call(x, args...)     ──▶  py? py_call : CALL_SPEC 语义
obj_len(x)               ──▶  py? py_len(468 新) : ARRAY_LEN/STR len 语义
obj_iter(x)              ──▶  py? py_iter(454) : 现有迭代通道
obj_type_name(x)         ──▶  py? py_type_name(469 新) : nv_py_type_name 家族
（协议位预留：obj_send / obj_contains —— 注册表占位，W2 挂 B7/C 族）
```

- 命名面：native registry 全名 `interop.obj_get` 等（Auto 层经
  `use interop` 或限定名可达；待澄清#3 请定夺命名根）；
- ForeignObject 协议：`py_ffi.rs` 内 trait（或 `vm/interop.rs` 定义 +
  py_ffi 实现——T05 按依赖方向择优），方法面 = 组合子所需七操作，
  PyObjectHandle 为首个实现；**协议不进 engine 热臂**，仅组合子内查表。

### s2s 改写器骨架（T07 契约）

- 入口：`auto trans <file> --to auto`（输出正常模式 `.at` 到 stdout/`-o`）；
- 管线：parse → AST → **规则表**（`LoweringRule { id: "A1".."F4",
  pattern, rewrite }` 注册表，W1 空置）→ emit（现有 AST 打印通道或
  token 级重组，取 round-trip 稳定者）；
- 单测框架：`trans` 模块 per-rule 测试宏/函数——注册一条测试规则断言
  改写产物（证明可扩展），identity 路径对语料样本断言稳定输出；
- `--dump-lowered`：run 入口（`auto x.as --dump-lowered`）与 trans 入口
  共用改写产物渲染（W1 产物 = passthrough 源 + 模式头注释行）。

## 测试设计

- **探针先行**（scratch/p555/）：`auto x.as` 现状（预期：扩展名不被
  接受或按未知处理——先证再改）；组合子缺席现状；`auto trans` 现状
  输出形态；基线记录入执行注记；
- **Rust 单测**：模式矩阵八格（mode 解析单测）；py 三桥（539 py_ffi
  单测同型：GIL 调用 + 返回 marshal 断言）；组合子双通道（构造
  PyObjectHandle 与 Auto 值各走一遍六组合子）；s2s 规则表扩展性 +
  identity 稳定性；
- **探针端到端**：`auto x.as` vs `auto x.at` 同内容输出 diff 为空；
  `#[rust]` 覆盖矩阵；`--dump-lowered` 输出留档；
- **门禁**：`cargo tv` + `cargo tt` + `cargo t vm` + py 五套件三方
  （零回归红线）。

## 验收标准

1. `auto x.as` 直跑 = 同内容 `.at` 输出逐字节一致；模式矩阵八格单测绿；
   `#[rust]` 覆盖探针在案；
2. 六组合子在 py 句柄与 Auto 值（list/str/map）双通道探针绿；
   ForeignObject 协议位（send/contains 预留）与 PyObjectHandle 首实现
   在案；
3. py_setattr/py_len/py_type_name 三桥 539 同型单测绿，py_torch 系
   套件零回归；
4. s2s 骨架：测试规则注入即生效（可扩展性单测）+ identity round-trip
   稳定；`--dump-lowered` run/trans 双入口输出在案；
5. `cargo tv` + `cargo tt` + `cargo t vm` 全绿；py 五套件三方全绿
   （零行为变更红线）；
6. P550-D6 销号（pragma/扩展名联动裁定入执行注记），P550-D4 期望面
   更新注记（端到端探针面归 W2 动态分派语义激活）。

## 执行步骤

- [x] T01 探针基线：`scratch/p555/` 三探针（`auto x.as` 现状/组合子
      缺席/`auto trans` 输出形态）+ 模式面考古注记（mode.rs/multi_mode.rs
      现状）（验证：探针逐条可复跑，现状入执行注记）
      [✅ 已完成] 2026-09-05 worktree master HEAD 干净构建探针：p1 `.as` 直跑**已然可跑**（CLI 位置参数不过滤扩展名，输出与 .at 逐字节同）——但无任何脚本模式信号（T03 的行为增量=模式信号可观测：.as 文件的 550 门控豁免）；p2 组合子缺席实证（E0401 obj_len）；p3 `auto trans --path <p> <TARGET>` 子命令形态（C/Rust/R2a/Python/JavaScript/GDScript——s2s 落位=新 TransTarget::Auto 变体）。考古：mode.rs=ExecutionMode（AutoVM/Evaluator/C/Rust，执行后端，194 行含测试模块）；multi_mode.rs=Plan 081 依赖编译模式管线（与脚本模式正交）。详见执行注记 T01
- [x] T02 模式解析矩阵：`mode.rs` 扩展（`.as`≡隐式 script + `#[rust]`
      覆盖）→ `compile.rs` session 联动（兼容 550 `script_marked`）
      （验证：模式矩阵八格单测 `cargo t mode` 绿）
      [✅ 已完成] TDD：八格单测先行红→`ScriptMode`+`resolve_script_mode`（优先序 #[rust]>#[script]>扩展名）绿；parser `rust_pragma`（annotation "rust" 臂，尾部统一 next 约定）；`CompileSession.script_mode`+`set_script_mode`（script_marked 派生兼容）；lib.rs 解析（path 扩展名 at/as 归一）+550 门控改按 ScriptMode 判定。可观测矩阵四探针：.at+null→警告 / .as+null→豁免 / .as+#[rust]→警告（压回）/ .at+#[script]→豁免，全符合契约表
- [x] T03 CLI `.as` 直跑：`lib.rs` run 路径 + `crates/auto/src/main.rs`
      接受 `.as`（passthrough 语义）（验证：同内容 .at/.as 输出 diff
      为空探针）
      [✅ 已完成] CLI 位置参数本不过滤扩展名（T01 考古），扩展名经 run_file_with_args→run_with_path→execute_autovm_with_path(path) 已入 T02 解析；p4 探针（递归+数组+for-in+拼接）：同内容 .at/.as 程序输出 diff 为空（仅 Running 横幅文件名行差异）——passthrough 语义实证
- [x] T04 py 三桥：`py_ffi.rs` py_setattr 467 / py_len 468 /
      py_type_name 469（539 桥型）（验证：新增 py_ffi 单测绿）
      [✅ 已完成] 三桥 467-469（setattr 语句形态推 null 保栈平衡同 setitem 约）+ register_py_object_builtins 名单 + init_py_ffi 注册；注册单测绿 + p5 torch 端到端探针（len=6/Tensor/setattr+getattr 回读 str）。（终扫补记：T04 完成当时漏加本标记，工作与验证均在案——复审标准 3 复跑 PASS；merge 终扫修正）
- [x] T05 ForeignObject 协议：协议 trait（七操作面）+ PyObjectHandle
      首实现适配（send/contains 协议位空置）（验证：协议单测绿）
      [✅ 已完成] vm/interop.rs 定义 ForeignObject（六操作方法面 get/set/call/len/iter/type_name + send/contains 注释预留位）；接入机制=HeapObject::as_foreign_object 默认钩子（None 默认，宿主句柄覆写——as_any 无法跨 trait downcast 的解法）；PyObjectHandle 首实现六臂（协议单测绿：downcast+foreign_kind=="py"）
- [x] T06 分发组合子：`vm/interop.rs` 新模块六组合子 + Auto 值原生
      方法表臂 + `vm/codegen.rs` 注册面（`interop.*` 家族）
      （验证：py 句柄/Auto 值双通道探针绿）
      [✅ 已完成] 组合子 1860-1865（catalog 限定名+裸名双注册，惰性命中）；发射=CALL_PY 传输形态（is_py_ffi_call 复用为"带实参数字节"约定，与 py 零耦合）；Auto 臂=str[i]//list[i]（含负索引+IndexError 对标 550）/map["k"] 读写/ARRAY_LEN 语义/array 通道迭代回推/容器 type_name；外对象臂经协议分派。p6 Auto 全矩阵（list/str/map/迭代 139）+ p7 torch 同函数名（len/sum=15/transpose/setattr/next）双探针绿；obj_call 栈布局 bug（pending 计数含 receiver）当场修复
- [x] T07 s2s 骨架：`trans/` 模块 `--to auto` 目标 + 规则表 + identity
      改写 + per-rule 单测框架（验证：测试规则注入单测 + identity
      round-trip 稳定单测绿）
      [✅ 已完成] trans/auto_s2s.rs（LoweringRule{id:A1..F4, rewrite}+builtin_rules 空表+lower_source 管线：词法→语法验证→单遍首中→发射）；Lexer::tokenize_all 公开词法面；三单测绿（identity 幂等逐字节/TEST-1 规则注入即生效且产物可解析/非法源拒绝）；CLI `auto trans --path x auto [-o]`（-o 落盘 round-trip 字节级同实证）
- [x] T08 `--dump-lowered`：run/trans 双入口 flag + 产物渲染
      （验证：`auto x.as --dump-lowered` 输出留档探针）
      [✅ 已完成] run 入口全局 flag（先于执行拦截=纯审查面）；lib.rs dump_lowered 渲染=模式头+改写产物（模式按扩展名档位：.as→script/.at→normal，头行留档探针在案）；trans auto 入口共用 trans_auto_s2s 产物源
- [x] T09 单测汇总挂档：T02/T04/T05/T06/T07 单测并入 `cargo t vm`
      可见（验证：`cargo t vm` 含全部新增用例绿）
      [✅ 已完成] cargo t vm 800/800（tests_script_mode 含）；python 档 test_w1_* 3 例绿（bridges+protocol+539 idiom 回归）；精确过滤 tests_script_mode 1/1 + tests_s2s 3/3。注：py_ffi 系 cfg(python) 不入 cargo t vm 档（特性分档在案）；`cargo t mode` 过滤撞名 d8_toggle_dark_mode=master 既有红（revert 双侧复跑证实，P555-D4）。bisect 操作暴露 vm.rs 漏提交（interop 声明）当场补 commit
- [x] T10 门禁：`cargo tv` + `cargo tt` + py 五套件三方（零回归红线）
      （验证：门禁输出留档执行注记）
      [✅ 已完成] tv 3588/3589 + tt 3775/3776——唯一红=test_charts_gallery_compiles（master 既有，branch diff 零 ui 文件，输入逐字节同 fork 点，P555-D4 甄别）；py 五套件三方 64/64（math 20+torch 7+infer 17+train 10+numpy 10）零回归红线成立
- [x] T11 折叠：KNOWN-DEBT 回写（P550-D6 销号注记 + P550-D4 期望面
      更新 + W1 自身债务登记）（验证：KNOWN-DEBT diff 在案）
      [✅ 已完成] KNOWN-DEBT P555 节登记：P550-D6 销号（八格矩阵裁定）、P550-D4 期望面更新（归 W2 语义激活）、P555-D1..D5（obj_call Auto 臂/s2s 帧形约定/obj_set 封送罕见面/master 双既有红甄别/CALL_PY 命名清理）

## 复审记录

**复审人**：zhaopuming（/auto-plan:review，2026-09-05）
**验证场所**：worktree `D:/autostack/.wt/lang-555/auto-lang`（branch plan-555-dev，
fork 点 e1429a0ef，5+1 commits——含 vm.rs 补提交；代码 diff 15 文件 +1251/−9：
mode/parser/compile/lib/lexer/vm.rs + vm/{interop 新,heap_object,engine,codegen,
native_catalog} + py_ffi + trans/{rs,auto_s2s 新} + auto/main.rs）

### 逐条验收裁定

| # | 验收标准 | 裁定 | 证据（复跑） |
|---|---|---|---|
| 1 | `.as` 直跑 passthrough + 八格矩阵 + `#[rust]` 覆盖 | **PASS** | p4 同内容 .at/.as 输出 diff 空；可观测矩阵 warn=1/0/1/0（.at+null 警/.as 豁免/.as+#[rust] 压回警告/.at+#[script] 豁免）；tests_script_mode 1/1 |
| 2 | 六组合子双通道 + 协议位 + 首实现 | **PASS** | p6 Auto 全矩阵（3/20/99/30/4/list/str/int/2/1/3/139 含负索引与迭代）；p7 py 句柄同函数名（6/Tensor/sum=15/transpose/setattr/next）；协议测试 1/1（downcast+kind="py"）；send/contains 预留位在 diff 注释可查 |
| 3 | py 三桥 539 同型单测 + torch 系零回归 | **PASS** | p5 端到端（6/Tensor/2/str）；python 档 test_w1_* 3/3（bridges+protocol+539 idiom 回归）；py_torch 7/7 + infer 17/17 + train 10/10 |
| 4 | s2s 可扩展 + identity 稳定 + dump 双入口 | **PASS** | tests_s2s 3/3（注入即生效/幂等逐字节/非法源拒绝）；run 入口 `--dump-lowered` 头行 `mode=script` 在案；trans auto 输出 passthrough 源 |
| 5 | 门禁全绿 + py 五套件零回归 | **PASS（附既有红甄别）** | `cargo tf` **3428/3429**（唯一红=test_charts_gallery_compiles，master 既有——branch diff 零 ui 文件，输入逐字节同 fork 点；550 期的 schema 两红已被并行提交修复，本期 tf 更干净）；tv 3588/3589 + tt 3775/3776（同一既有红）；py 五套件 64/64；cargo t vm 800/800 |
| 6 | P550-D6 销号 + P550-D4 期望面更新 | **PASS** | KNOWN-DEBT P555 节两条处置在案（grep 命中=2）；P555-D1..D5 债登记齐 |

### 遗漏 / 延后 / workaround 扫描

- **遗漏**：无——diff 内无 TODO/HACK/FIXME（复扫确认）；4 处"协议位预留/归 W2"
  命中均为计划文本明文授权的边界（非目标清单在草案确认时过目）。
- **延后**：无未经批准延期。Err 值通道/窥孔直达/B8 裸模块/跨模式 import/
  W2 糖批均在计划"非目标"节（用户确认草案时批准的波次切分）。
- **Workaround**：两项披露在案——CALL_PY 传输形态复用（命名误导，P555-D5，
  W2 顺手清理）；ForeignObject.obj_set 借道 push/pop 封送（裸 f64 罕见面，
  P555-D3）。均为有根因记录的取舍而非遮蔽。

### 计划文 vs 实现分歧清单（均在执行注记/债在案）

1. T05"七操作面"→ 实装 6 方法 + send/contains 注释预留位（计划文同节将
   二者列为"协议位预留"，6+2 为忠实读法）。
2. obj_call 的 Auto 臂=响亮拒绝（Auto 闭包动态调用面归 W2）——计划验证
   措词的 Auto 值面（list/str/map）无一是 callable，语义一致（P555-D1）。
3. s2s W1 帧=token 粒度单遍首中——计划"取 round-trip 稳定者"授权的择优；
   AST 级发射器归 W2 裁定（P555-D2）。
4. 执行事故自愈：vm.rs（interop 声明）漏出 T05/T06 提交清单，bisect 操作
   暴露后当场补提交（T09 标记留痕，工作树终态 clean）。

### 门禁读数汇总（终态）

`cargo tf` 3428/3429（1 红=master 既有 charts，diff-independent 甄别）·
`cargo tv` 3588/3589 · `cargo tt` 3775/3776（同一既有红）· `cargo t vm` 800/800 ·
python 档 test_w1_* 3/3 · tests_script_mode 1/1 · tests_s2s 3/3 ·
py 五套件三方 64/64 · 探针 p1-p7 复跑矩阵在案。

**裁定：六项验收全 PASS，无阻断债 → status: reviewed，可入 /auto-plan:merge。**

## 执行注记

### 终态门禁读数（2026-09-05，11/11 任务落地）

`cargo tv` 3588/3589 · `cargo tt` 3775/3776（唯一红 = master 既有
test_charts_gallery_compiles，P555-D4 甄别在案）· `cargo t vm` 800/800 ·
py 五套件三方 64/64（零回归红线成立）· python 档 test_w1_* 3/3 ·
tests_script_mode 1/1 · tests_s2s 3/3。

### T01 探针基线（2026-09-05，worktree master HEAD e1429a0ef 干净构建）

| 探针 | 现状 | 判读 |
|---|---|---|
| p1 `auto p1_as_direct.as` | `Running Auto … / hello-as`（与 .at 逐字节同） | CLI 位置参数不过滤扩展名——`.as` "能跑"但零模式信号；T03 行为增量=script 模式可观测（.as + null 字面量不出 550 门控警告） |
| p2 `obj_len(xs)` | E0401 Undefined symbol | 组合子缺席实证 |
| p3 `auto trans` | `--path <PATH> <COMMAND>` 子命令形态（C/Rust/R2a/Python/JavaScript/GDScript/A2cStdlib） | s2s 落位=新 `TransTarget::Auto` 变体（`auto trans --path x.as auto`） |

模式面考古：`mode.rs` 仅有 `ExecutionMode`（执行后端选择，Plan 081）——脚本
模式（语言方言信号）为正交新概念；`multi_mode.rs` 是依赖编译模式管线，
设计 §1"折进 multi_mode"归 W2+。

## 待澄清事项

1. **Err 值通道/catch 拦值归属**：建议归 W2（隐式 `!T` 传播点正是
   W2 lowering 规则的发射点，同批复用改写器管线；W1 组合子失败走
   现有 VMError→try-catch 通道与 550 行为一致）。如需提前请在确认
   计划时指出。
2. **窥孔零开销直达**（静态已知 py 调用点直达 py_xxx）：建议 W2——
   W1 组合子 tag 分派已保证正确性，窥孔是纯优化，先立正确性地基。
3. **组合子命名根**：建议 `interop.obj_get` 家族（native registry
   全名，s2s 产物可读 + 跨语言矩阵承接位）；备选 `auto.interop.*`
   stdlib 风格。确认计划时定夺。
4. **`.as`→a2py/parity runner**：W1 仅 VM 直跑；a2py 接受 `.as` 与
   parity runner glob 扩展归 W2 迁移批（py 套件改名 .as 时一起）。
