---
plan_id: PLAN-514
status: reviewed               # drafting → executing → execution_done → reviewed → archived
feature_name: aavm-idiom2-methods
author: [zhaopuming]
created_at: 2026-09-01
updated_at: 2026-09-02

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/aavm/design/divergences.md: 增 514 节（D-ext-trait/D-pipe-first/D-methodized）"
  - "docs/specs/aavm/design/divergence-rules.md: §4b 方法化写法规范（第 10-14 条）"
  - "docs/specs/aavm/design/idiom-upgrade-prereqs.md: W5/W3/W6 状态注记"
  - "docs/specs/aavm/project.md: lib 六文件 514 能力注记"
  - "auto/lib/README.md: 514 段（方法化 γ4/管道/塔顶）"
  - ".github/workflows/vm-files-ci.yml: 第一层去 --include-ignored（P511-2 口径收窄）"
new_spec_components:
  - "docs/specs/aavm/design/method-emission-spec.md: 新增（W0 步骤 1，方法发射 live 对齐基准）"
touched_goals: [GOAL-017]     # 自举：用 Auto 写 Auto 编译器（aavm）——塔顶方法化自举达成

affects: [aavm]
current_step: 24
total_steps: 25
---

# [PLAN-514] aavm 风格二期：AA2R 方法发射（W5）+ 宿主加固 + lib 方法化（γ4）

## 变更摘要

风格二期承接 Plan 447 三部曲的"可选二期"余项（idiom-upgrade-prereqs.md
§4-W5）与 511 执行期债务，四个波次：

- **W1 宿主两侧加固**：修复五方矩阵②腿编译期红（P511-5：主 a2r 对 lib
  `File::read_text` 的独立二进制转译缺导入）；探针全形态实测 VM 方法路径
  与主 a2r 方法发射；顺收 447-旁支两条债务（`struct` 误用报 E0201、
  嵌套 fn 静默失效）。
- **W2 AA2R W5**：a2r.at 学会发射 `type` 体内方法 / `ext` 块 / static fn /
  接收者 `.field` 简写（与主 a2r live 逐字符 + rustc 零错）；同批清偿
  P511-1 七件 AA2R 预存转译债 → `--include-ignored` 37/37 全绿 →
  AA2R 腿去 ignore 纳入常规门禁（P511-2 口径收窄清偿）。
- **W3 lib 方法化（γ4）**：六文件按"自有类型方法进 `type` 体、跨类型
  扩展用 `ext`"转换，行为不变判据（闸门+矩阵+99_unit 全绿贯穿），
  a2r.at 自身最后（自举塔顶：自己转译自己）。
- **W4 447 尾巴收账**：C3（cg_expr/cg_stmt p_kind 链 is 化）、
  `t_array_elem` Type 载荷化（D23/D27）、11.2b f-string 批量改写重试、
  Phase 11 收账余项。

全程 TDD：既有九族闸门为保护网（每文件转换后必须绿，红=行为变化=回退），
新能力语料/探针先行落盘转红再实现。

## 目标

1. AA2R 能发射方法族语法，与主 a2r `transpile_rust` 输出 live 逐字符一致，
   产物 rustc 实编译零错（corpus_a2r g 系新件为判据）。
2. `test_aavm2 --include-ignored` 37/37 全绿（P511-1 七件债清偿），AA2R
   compile 腿去 ignore 并纳入 vm-files-ci 常规门禁。
3. 五方矩阵恢复全程绿（P511-5 修复），并作为 W3 行为不变判据贯穿。
4. lib 六文件方法化完成：`fn cg_xxx(c CG, ...)` 形态的状态类型自有函数
   转为类型方法（映射细则见待澄清①），a2r.at 自身可被自己转译且 rustc
   零错（自举塔顶验证）。
5. 447 尾巴四件逐一处置（完成或显式登记回退裁定）。
6. 447-旁支两条宿主债务清偿（`struct` 误用改报语法错；嵌套 fn 静默失效
   修复或显式裁定）。

### 非目标（Out of Scope）

- OOP 批内容：aavm **目标语言**的方法/impl/trait 语义（is-struct 模式、
  pub type 跨模块共享等）——独立后续计划。
- `ext T for Trait`（trait 实现）、泛型 ext（`impl<T>`）、关联 const、
  注解宏位（#[zbus] 族）：AA2R 发射明确不做，divergence 登记。
- W6 闭包/迭代器链还原（prereqs §4-W6）：不做。
- 嵌套 fn/闭包在 lib 写法上解禁（本计划只修宿主静默失效 bug；lib 仍
  顶层 fn 规范）。
- 行为层 DIVERGE 翻案（D12 游标/D16 char 计长/D28 布局等）：不动。
- aavm VBool parity 债（P474-旁支）：不扩面不清偿。

## 架构方案

自举塔约束（prereqs §4 头注）：**lib 用什么语法，AA2R 必须先能发射什么**。
故波次顺序不可倒置：宿主稳（W1）→ AA2R 会发（W2）→ lib 才能用（W3）。
W2 的 a2r.at 扩展本身仍用现有自由函数风格书写（塔式爬升，447 先例），
W3 最后才轮到 a2r.at 自身改写。

```text
W1 宿主加固          W2 AA2R W5                W3 lib 方法化 γ4        W4 尾巴
────────────        ─────────────────        ─────────────────       ────────
矩阵②腿修复    →    corpus_a2r g09+ 先红  →   token.at 试点       →   C3 is 化
99_idiom2 探针  →    ar_prescan/_emit_ext  →   lexer/typeinfo/parser    t_array_elem
E0201/嵌套fn    →    接收器合成/方法调用位  →   codegen/engine           11.2b 重试
主 a2r 发射洞   →    P511-1 七件债清偿     →   a2r.at 自身(塔顶)        Phase11 余项
```

**关键设计约束（全程）**：

- **宿主为规范**：AA2R 发射文本 = 主 a2r 现行输出（live 对齐，无落盘
  golden）；主 a2r 自身有洞先修主 a2r（447 先例 H4/H5）。
- **行为不变判据**：W3 每文件转换 = 一次转换一个提交，九族闸门 + 矩阵 +
  99_unit 全绿才进下一个文件；任何红即回退该文件转换。
- **方法化映射原则**（缺省，细则待澄清①）：状态类型（P/CG/Ar/Val 操作
  族）自有函数 → `type` 体内方法，调用点 `cg_emit(c, idx)` → `c.emit(idx)`；
  跨类型/无状态纯函数保留自由函数；不强求 100% 函数消失，以一对一可读性
  为准。
- **矩阵全程保持绿**：P511-5 修复后，W2/W3 每折叠点必跑（它是 W3 唯一
  能看见"lib 行为不变"的全链判据）。

## 需求分析与背景调查

（取材：docs/specs/aavm/design/idiom-upgrade-prereqs.md、Plan 447/511 归档、
KNOWN-DEBT P447-①/P511 节、trans/rust.rs ext_decl/fn_decl 实读）

### 基线（2026-09-01 实测，master 1f7313e93）

| 门禁 | 结果 | 状态 |
|---|---|---|
| tv 标准门禁（`-- test_aavm2`，非 ignored） | 16 过 / 0 挂 / 3 ignored | ✅ 绿 |
| `--include-ignored` 全量 | 18 过 / 1 挂 | 🔴 test_aavm2_compile_corpus：P511-1 七件（b13/b14×2/b19/b27/b29/b30） |
| 99_unit（`auto test -d .../99_unit`） | 13 过 / 0 挂 | ✅ 绿 |
| 五方矩阵（parity/ `cargo run ... aavm`） | 编译期红 | 🔴 P511-5：②腿 aavm2_bin 转译产物 `File::read_text` 缺导入，E0433 ×3 |

结论：**当前 AAVM 代码通过标准门禁与 Auto 侧单测**（用户前置条件满足）；
两处红均为已登记/本次登记的预存债，且都在本计划清偿范围内——W2 清
P511-1/2，W1 清 P511-5。

### 宿主方法族语义（已考古）

- 声明：`type Counter { value int; fn get() int { return .value } }`
  （体内方法 + `.field` 接收者简写）；`static fn new(v int)`（`Counter.new(10)`
  → Rust `Counter::new(10)`）；`ext Counter { fn double() ... }`（外部扩展）；
  构造 `Counter(v)`（examples/playground-demo/13-methods.at）。
- 主 a2r 发射：`ext_decl`（trans/rust.rs:16052）发 `impl T {`（或
  `impl Trait for T`）；方法体走 `fn_decl`（:12288），`fn.parent.is_some()`
  判定方法身份（:64/:74），合成接收者 `&self`/`&mut self`（突变经
  scan_mutated_bindings），static 免；`.field` 简写 → `self.field`
  （:11500 位）；type 体内方法与 ext 共用发射族（:12330）。
- VM 执行：`Stmt::Ext`（vm/codegen.rs:2402）+ type 体内方法同路径。
- **已知洞**：主 a2r 独立二进制转译对 lib `File::read_text` 缺 `File`
  导入（P511-5）；447-旁支：`struct` 误用报 E0201 名字解析错、嵌套 fn
  静默失效。

### AA2R 现状（a2r.at，3097 行快照）

- `ar_prescan_type`（:655）：遇 `Fn`/`Static`/`Hash` 的 type 体内成员直接
  "v2 unsupported"——方法表零起点。
- `ar_run`（:3014）顶层分派仅 Fn/Type/Enum，无 Ext 分支。
- 可复用杠杆：`ar_emit_fn2`（:2851，方法体复用）、`ar_scan_mutations`
  （:784，&mut 判定）、`ar_vpush`（:347，self 入作用域）、`ar_method_call`
  （实例方法解析位）。

### P511-1 两根因（KNOWN-DEBT 在案）

① `.at` lexer 的 `char_at` 码点语义经 a2r 转译后变字节序——注释内 CJK
多字节字符产生 Unknown token（b13/b14/b19）；② `arr_flag` 接收者跟踪在
转译侧失效——`a.len()` 报 "receiver is not an array"（b27/b29/b30）。

### 风险与对策

| 风险 | 对策 |
|---|---|
| 主 a2r 方法发射/自动引用路径有未爆洞（探针暴露） | W1 探针先行暴露；小修顺收，大洞按待澄清③裁定插修复波 |
| AA2R 扩展连带暴露主 a2r/宿主缺口（447 先例 7+ 处） | W2 工期按 1:1 预留连带修复缓冲 |
| lib 方法化改变行为（接收者按值/按引用语义差异） | 每文件一提交+全闸门绿的硬闸；红即回退该文件 |
| a2r.at 自身转换后自举塔断（自己转译不了自己） | 塔顶验证前置到 token.at 试点（步骤 12）先验证发射面足够 |
| 11.2b f-string 重试再触 D40 三缺口 | 待澄清②：登记回退，不阻塞 |

## 详细设计

### W1 宿主两侧加固

1. **矩阵②腿修复（P511-5）**：主 a2r 对 `File.read_text`（及同族
   `auto.file.*`）在独立二进制模式下的导入注入或 shim 映射——方案在
   W0 考古定案（import 注入 vs std 直映）；修复后 parity 全绿复跑留档，
   此后每折叠点必跑。
2. **99_idiom2 探针族**（红先行落盘 `test/vm/99_idiom2/`）：方法读字段/
   写字段（&mut 判定）、方法内调方法、static 构造 + 方法链、`Counter(v)`
   位置构造、`List<T>` 元素方法调用、方法值传参/返回、ext 扩展方法、
   `.field` 简写与 `self.x` 双形态。VM 侧跑（vm_file_tests 显式 fn 常驻）
   + 主 a2r 产物 rustc 实编译。
3. **447-旁支两债**：`struct` 非关键字误用改报语法错（parser.rs
   check_symbol 位）；嵌套 fn 静默失效修复（parser/vm/codegen 嵌套路径；
   type 体内方法编译路径若同踩必验）。
4. 主 a2r 方法发射洞（探针 rustc 暴露的自动引用/解引用/构造器形态）顺修。

### W2 AA2R W5（a2r.at 扩展，自由函数风格书写）

1. **语料先行（红）**：corpus_a2r 增 g09_ext_basic / g10_type_inline_method /
   g11_static_new / g12_mut_field / g13_method_chain / g14_ctor_call /
   g15_self_dot_forms——落盘即红（AA2R 遇 type 方法/ext 报 unsupported）。
2. `ar_prescan_type` 摘除 Fn/Static unsupported：体内方法解析进 `ArTy`
   方法表（名/参数/返回/static 位/可变位）；`ar_run` 加 `Ext` 分支 +
   `ar_prescan_ext`（ext T 块 → 同一方法表 + 待发 impl 块标记）。
3. 新 `ar_emit_ext`：发 `impl T {`，逐方法复用 `ar_emit_fn2` + 接收器位
   合成（查方法表定方法身份；`&mut` 经 `ar_scan_mutations` 字段写检测；
   static 免）；`self` 经 `ar_vpush` 以目标类型入作用域；`.field` 简写 →
   `self.field`（镜像主 a2r :11500 位语义）。
4. 方法调用位：`ar_method_call` 增查方法表——实例方法直发同名；static
   调用 `Type.new(x)` → `Type::new(x)`；`T(v)` 构造对齐主 a2r 构造发射。
   type 发射改"字段 struct + 方法 impl 块"分离（主 a2r 同款）。
5. **P511-1 清偿**：根因① char_at 码点语义在转译侧的字节序偏差（a2r.at
   自身 char 处理对齐宿主词法语料语义）；根因② arr_flag 接收者跟踪的
   转译侧保持——目标 `test_aavm2_compile_corpus` 37/37。
6. AA2R compile 腿去 ignore；vm-files-ci.yml 第一层命令口径更新（P511-2
   收窄清偿）；折叠点②（矩阵+CI 绿后合入）。

### W3 lib 方法化（γ4）

逐文件顺序（依赖序，每文件一提交）：token.at（试点，含塔顶样板验证）→
lexer.at → typeinfo.at + parser.at → codegen.at + engine.at → a2r.at
（最后，自举塔顶）。转换要点：

- 状态类型方法化：`P`（游标+作用域）、`CG`、`Ar`、lexer 状态等的自有
  操作函数进 `type` 体；跨类型辅助（如 `ev_vi` 值构造族、纯词法表函数）
  保留自由函数或入 ext——逐文件映射表在对应步骤展开时定（待澄清①）。
- 调用点机械翻转：`cg_emit_store(c, idx)` → `c.emit_store(idx)`。
- **行为不变硬闸**：每文件转换后 tv 标准门禁 + include-ignored（W2 后
  应全绿）+ 99_unit + 矩阵全绿才进下一文件；红即回退。
- token.at 试点附加验证：AA2R 转译方法化 token.at → rustc 零错 → 产物
  与转换前行为一致（塔顶可行性确认，失败则 W3 暂停回 W2 补发射面）。
- a2r.at 自身转换后：AA2R 转译自己 → rustc 零错 → 全闸门绿（自举塔顶
  终局；即 447 系列 G2 自举演示的方法化版）。

### W4 447 尾巴收账

1. C3：`cg_expr`/`cg_stmt` 的 p_kind 链 is 化（方法化后在新方法体内
   顺路完成；镜像 Rust match 臂形态）。
2. `t_array_elem` 的 "(array-type ...)" 字符串形状解析 → Type 载荷化
   （D23/D27 收账）。
3. 11.2b f-string 批量改写重试：前置清偿 D40 三缺口；仍卡则按待澄清②
   登记回退。
4. Phase 11 收账余项核对（447 归档清单：②AA2R 单语句块臂不内联对齐主
   a2r、③List 实参克隆 parity、④臂值位赋值表达式支持——逐项处置或
   显式登记）。

### 规格与登记（贯穿）

- W0/W1 考古结论落盘 `docs/specs/aavm/design/`（方法发射规格章）；
  99_idiom2 探针常驻。
- divergences.md：W5 相关新分叉（ext for Trait/泛型 ext/关联 const 不做）、
  方法风格写法规范入 divergence-rules.md §4。
- idiom-upgrade-prereqs.md 状态注记（W5 完成清账）；project.md 模块清单
  /能力矩阵回写；KNOWN-DEBT P511-1/2/5、P447-①两债核销。

## 测试设计（TDD：保护网 + 红先行）

### 保护网（既有九族闸门，全程不得破绿）

沿用 511 的存量资产清单：corpus_m1~m4 / corpus_a2r / 里程碑 fib /
99_idiom_probe 16 件 / AA2R 探针冒烟 / 001-002 金样 / repro_242 /
五方矩阵 / vm-files-ci 三层。W3 每文件转换后全绿（红=行为变化=回退）。

### 红先行（新能力与债务清偿）

| 红灯 | 载体 | 转绿点 |
|---|---|---|
| corpus_a2r g09–g15（AA2R 遇方法语法 unsupported） | 步骤 6 落盘 | W2 实现（步骤 7–9） |
| test_aavm2_compile_corpus 30/37（P511-1 七件） | 基线已在红 | 步骤 10 |
| 99_idiom2 探针族（VM/主 a2r 两侧洞暴露） | 步骤 2 落盘 | W1 修复（步骤 3–4） |
| 矩阵②腿编译红（P511-5） | 基线已在红 | 步骤 3 |
| 447-旁支 E0201/嵌套 fn 复现件 | 步骤 2 附带落盘 | 步骤 4 |

### 命令

- 标准门禁 / 全量：`cargo test -p auto-lang --lib --features test-vm-files
  -- test_aavm2 [--include-ignored] --test-threads=1`
- Auto 侧：`./target/debug/auto.exe test -d crates/auto-lang/test/vm/aavm2/99_unit`
- 矩阵：`cd parity && cargo run -- --root . --auto-binary ../target/debug/auto.exe aavm`
- rustc 实编译：探针/corpus_a2r 新件经主 a2r 与 AA2R 双侧产物 `rustc
  --edition 2021` 零错
- 分级门禁 Category B/C：日常 `cargo t`/`cargo tv`，折叠点 `cargo tf`

## 验收标准

1. corpus_a2r g 系新件与主 a2r live 逐字符一致 + 双侧产物 rustc 零错。
2. `--include-ignored` 37/37 全绿；AA2R compile 腿去 ignore 入 CI 常规
   门禁（P511-1/2 清偿）。
3. 五方矩阵全程绿（P511-5 清偿后每折叠点留档）。
4. lib 六文件方法化完成且行为不变（闸门+矩阵+99_unit 贯穿绿）；a2r.at
   自身可被自己转译且 rustc 零错（塔顶验证留档）。
5. 447 尾巴四件逐一处置（完成或显式回退登记）；447-旁支两债清偿或裁定。
6. 99_idiom2 探针族常驻回归；divergences/写法规范/文档回写完成。
7. `cargo tf` 绿零新增警告；无静默丢弃（延后项显式登记）。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

> 约定：W0 在 master 直接做（纯文档+探针件）；W1 起在 worktree
> `.worktrees/plan-514-dev` 内；三折叠点（步骤 5/11/16）matrix+CI 绿后
> 合入 master。

### W0 基线与规格（master）

1. [✅ 已完成] 规格章落盘 `docs/specs/aavm/design/method-emission-spec.md`：live 对齐表（17 形态，13-methods+arch1/2/11 实测）、P511-5 定案 std 直映（三方语义一致+merge 零依赖）、洞 B（`set`→`insert` 遮蔽，Plan 393 E1 先例）、债①②精确复现（arch7/arch10）。2026-09-01。
2. [✅ 已完成] 99_idiom2 12 件落盘（m01–m09 绿件 + d01 plain 嵌套 fn 绿守卫 +
   d01b 捕获位复现件 #[ignore] 红 + d02 struct 误用 .expected.error 守卫）；
   vm_file_tests 接线 12 fn；实跑 11 过/1 ignore。考古修正：债① plain 形态现已
   可工作，真实静默失效在**捕获位**（嵌套 fn 引用外层局部 → 静默 0）；m06 避
   VM 保留名 Box。2026-09-01。

### W1 宿主两侧加固（worktree）

3. [✅ 已完成] 矩阵②腿修复：主 a2r maybe_module_method 加 "File" 臂（std 直映：
   read_text→`std::fs::read_to_string(&x).unwrap_or_default()` 等，定案依据=
   三方语义一致+merge 零依赖，spec 见 method-emission-spec §3 洞 A）。连带⑤腿
   伴生缺口同批清偿：a2r.at `mod` 保留字标识符（ar_rust_ident r#mod 镜像主 a2r）、
   AA2R File.* 直映、两处 E0382（owned-str push 借用克隆；按值非 Copy 实参无条件
   克隆对齐主 a2r struct_flags——ar_lu_after 看不见循环携带复用）。实跑：②腿
   aavm2_bin 构建通过并运行，矩阵全表可跑（此前②编译红挡全表）；余 DIFF=
   b13/b14/b19（根因① CJK）+b27+（根因② arr_flag）＝P511-1 登记债，W2-10
   清偿后折叠点②复跑全绿。2026-09-01。
4. [✅ 已完成] 宿主修复批：债② struct 误用改报 E0007 语法错（parse_stmt_inner
   顶部拦截+check_symbol Ident 臂，裸/带名形态覆盖，d02 守卫绿）；债① 嵌套 fn
   捕获外层局部改报 E0201（infer context fn_scope_idxs 边界栈+no-capture 查找+
   check_symbol Bina 臂，d01b 转绿解除 ignore；已知限：嵌套 fn 引用全局运行期
   静默 0=预存 VM 行为，master 同态非回归）。探针暴露洞顺修：洞 B `set`→`insert`
   三处接收者守卫+静态构造类型预扫（Type.new→User，Dot/Bina 双形态）+`List(T)`
   圆括号形态对齐 Type::List→Vec&lt;T&gt;+is_owned_list_arg 扩 Array。验证：
   99_idiom2 12/12 绿（m01–m09 主 a2r 产物 rustc 零错）；aavm2 标准门禁 16 绿；
   99_unit 13 绿；g01–g08 逐字符对拍保持绿。2026-09-01。
5. [✅ 已完成] 折叠点①：cargo tf 全绿（3350 过/0 败/96 skip=含 P511-1 预存
   ignore）+ CI 二三层绿（vm goldens+ffi_dual 56、conformance 36）→ master
   合入 996f98572，worktree 已回同步。CI 第一层（--include-ignored）余
   P511-1 预存红，按计划 W2-10/W2-11 清偿后转绿。2026-09-01。

### W2 AA2R W5 + P511-1（worktree 续）

6. [✅ 已完成] g09_ext_basic/g10_type_inline_method/g11_static_new/g12_mut_field/
   g13_method_chain/g14_ctor_call/g15_self_dot_forms 落盘；红证：
   `PARSE-ERROR:v2 unsupported: type body member <Static>`（AA2R 首失配即停）。
7. [✅ 已完成] ArMethod 方法表（is_static/mutates/pos）+ar_prescan_type 体内
   fn/static 解析+ar_scan_method_writes（`.x =`/`self.x =` 字段写 token 扫描→
   &mut 判定）+ar_prescan_ext（方法并入目标 type 表）+ar_run Ext 分支。
8. [✅ 已完成] ar_emit_type2 struct/impl 分离（体内方法内联+ext 方法按预扫位
   跳转，全部并入同一 impl 块=主 a2r 合并形）+ar_emit_method2（接收者合成
   &self/&mut self/static 免+self 经 ar_vpush 入作用域）+原子 leading-Dot 臂
   （`.field`→`self.field`、`.method(...)`→self 调用）。迭代修复：单行方法体
   专用 ar_skip_method_body（ar_skip_decl 越界吞外层 RBrace）、ext 发射位空行
   抑制、static 双关键字跳层。
9. [✅ 已完成] Type.new(x)→`Type::new(x)` 静态类型路径+零参 get() 用户方法
   直通（避开数组索引臂）+构造器（ar_call_tail 既有位置构造展开复用）。
   验证：corpus_a2r 全绿（g01–g15 逐字符对拍主 a2r）+g09–g15 主 a2r 产物
   rustc --emit=metadata 零错+标准门禁 16 绿+99_unit 13 绿。2026-09-01。
10. [✅ 已完成] P511-1 双根因清偿——根因①：lexer.at tokenize 主循环加
    cur_char 哨兵（VM 侧 len/char_at 同为码点自洽；转译侧 len()=字节>
    码点，主循环越过真实末尾产 Unknown——b13/b14/b19）；根因②（真因与
    登记不同）：主 a2r 通用 Bina 臂缺优先级括号——`(a||b)&&c` 发射为
    `a||b&&c` 语义反转，codegen.at 全局注册条件误命中→var 走全局路径→
    arr_flag 链塌缩（b27/b29/b30）；补 auto_op_prec 表（镜像 a2r.at
    infix_l）+lhs/rhs 括号判定。⑤腿追加：a2r.at slice 臂对齐主 a2r
    chars().take(b).skip(a) 码点形（字节切片 CJK 内 panic）。验证：
    test_aavm2_compile_corpus 37/37 绿；--include-ignored 19/19 绿。
    2026-09-01。
11. [✅ 已完成] test_aavm2_compile_corpus 去 ignore 入常规门禁（标准
    `-- test_aavm2` 17 过/2 ignore[按需 rustc 探针]）；vm-files-ci 第一层
    去 --include-ignored（P511-2 口径收窄清偿）；折叠点②：矩阵五腿
    46/46 全绿（matrix_run4 留档）+cargo tf 3350 绿→master 合入
    515370ae4，worktree 回同步。2026-09-01。

### W3 lib 方法化 γ4（worktree 续；每文件一提交全绿才进）

12. [✅ 已完成] token.at 映射表裁定：无状态类型/纯表函数
    （keyword_kind/kind_name）→ 按 γ4 缺省**保留自由函数**（零转换）；
    塔顶样板验证移至首个真实状态类型 P。**P 方法化落地（2026-09-02，
    commit 8aef313ca）**：14 方法入 type 体（p_err→fail 避字段撞名）+
    全库调用点翻转；语句位前导点仅块首豁免（W3-1 清偿后非块首句首
    `.method()` 仍硬语法错——skip_empty_lines 一处 self.next()）。W3-2
    补丁续修实际清偿量远超登记的"仅剩 decl_lookup 位"：主 a2r 8 处
    （存档补丁 a–d + e 真因=未注解局部注册 Type::Unknown 而非缺失 +
    字面量 .as_str() 误加 E0658 + 用户方法返回类型 qualified 键推断
    （"P.text"，含 merge 流程 global_fn_ret_types 补扫 TypeDecl/Ext）+
    expr_mutates_self 裸 self 漏判 + 跨方法 &mut 传递闭包
    compute_type_mut_methods + 两实参环 infer 兜底）+ AA2R 镜像 3 组
    （ar_method_call 方法表实参强转/返回类型、ar_scan_method_writes
    变异调用扫描 ar_call_rooted_self、ar_fixpoint_mutates 传递闭包）。
    验证全绿：cc 编译红逐位清零（主 a2r merge 产物 rustc 零错）+
    corpus g01–g16 逐字符 + aavm2 17/2 + include-ignored 19/19 +
    99_unit 13 + **⑤腿 AA2R 转译方法化 lib → rustc 零错（塔顶样板
    验证通过）** + 矩阵五腿 46/46（matrix_w3_12.log 留档 scratch/p514/）。
    2026-09-02。
13. [✅ 已完成] lexer.at 转换=**零转换裁定**（无状态类型：纯字符谓词
    is_digit/is_alpha/... 与 (source,start)→Tok 扫描器族；γ4 缺省保留
    自由函数，镜像 token.at 步骤 12 先例）。验证：W3-12 提交态闸门
    全绿即覆盖（文件未动）。2026-09-02。
14. [✅ 已完成] typeinfo.at + parser.at 转换（commit 9d2b96cf1）：
    parser.at 的 P 已于步骤 12 完成，语法产生式族按步骤 12 裁定保留
    自由函数；typeinfo.at t_* 族是 (P+累加器) 行走器（非纯 P 操作）
    保留自由函数；纯 P 操作 p_text_at/p_peek_text（typeinfo/codegen
    跨文件逐字节重复）合并入 type P 作 peek_text(n) 方法（codegen 侧
    副本随 W3-15 清）。验证：cc 零红 + corpus g01–g16 逐字符 +
    include-ignored 19/19 + 99_unit 13 + 矩阵五腿 46/46
    （matrix_w3_14.log；首跑⑤腿构建环境性 flake，复跑+独立复现双绿）。
    2026-09-02。
15. [✅ 已完成] codegen.at + engine.at 转换（commit 0e4d10df9）：CG
    c-only 操作族 28 方法入 type 体（cg_err→fail、cg_new→static new；
    产生式族与纯表函数保留自由函数）+ 全库翻转；engine.at ev_field_idx
    并入 CG.field_idx（其余 ev_* 无状态类型，零转换）。主 a2r 配套：
    方法环补 `.str()` 按接收者类型发射臂 + 后处理
    fix_str_to_string_assignments 改按函数作用域（原全文件 &str 名字池
    让短名撞跨函数局部误加 .to_string()）；AA2R 镜像：静态方法臂按
    方法表解析返回类型（`var m = CG.new()` 此前恒 Unknown）。corpus
    g17 探针常驻。验证：cc 零红 + corpus g01–g17 逐字符 +
    include-ignored 19/19 + 99_unit 13 + ⑤腿零红 + 矩阵 46/46
    （matrix_w3_15.log）。2026-09-02。
16. [✅ 已完成] a2r.at 自身转换（塔顶终局，commit aaa6778b5）：Ar
    自有操作族 23 方法入 type 体（ar_err→fail、ar_new→static new；
    产生式/预扫带 p 族保留自由函数）+全库翻转。主 a2r 配套
    （expr_mutates_self 索引位赋值+.set 变异名）+ AA2R 镜像
    （ar_scan_method_writes 方括号回跳+set、ar_scan_mutations 方法表
    mutates 查表+prescan 参数临时作用域）。**塔顶终局验证**：方法化的
    AA2R 自己转译自己（⑤腿 aa2r_transpile_merge 全 lib）→ rustc 零错。
    折叠点③：corpus g01–g17 逐字符 + include-ignored 19/19 + 99_unit
    13 + 矩阵五腿 46/46（matrix_w3_16.log）+ cargo tf 3350/3350 →
    合入 master。2026-09-02。

### W4 447 尾巴（worktree 续）

17. [✅ 已完成] C3：cg_expr/cg_stmt p_kind 链 is 化（worktree commit）：
    cg_expr 梯子→is 11 臂+else（原子）；cg_stmt 简单 kind 臂→is 12 臂，
    复合条件（pub/下标赋值/字段赋值）与表达式语句尾入 else。验证：cc
    零红 + corpus g01–g17 逐字符 + include-ignored 19/19 + 99_unit 13 +
    ⑤腿零红 + 矩阵 46/46（matrix_w4_17.log）。2026-09-02。
18. [✅ 已处置-显式登记] t_array_elem Type 载荷化：按 divergences.md
    447-③ 时点既定裁定**保留**——Type Display 文本与 "(array-type ...)"
    形状是 M2/M3 dump 口径的组成部分，载荷化将漂移判据 golden；与 D20
    （E 载体）同一原则，判据层重构另立计划时一并处理（引用
    divergences.md §D23/D27 条目原文）。tv-aavm2 现行绿即验证。
19. [✅ 已处置-按待澄清②缺省登记回退] 11.2b f-string 批量改写重试：
    D40 三缺口（FStrPart 直通发射）仍在册（divergences.md D40 续），
    前置清偿超限；按待澄清②缺省登记回退，不阻塞。矩阵现行 46/46 绿
    （行为不变判据现行成立）。
20. [✅ 已处置-逐项显式登记] Phase 11 收账余项（447 归档登记原文
    447-aavm-prerequisites.md §10.4）：②AA2R 单语句块臂不内联（主 a2r
    write_match_arm_body 内联）——语料无该形态暴露，对齐属 dump 判据
    层重构（与 D23/D27 同则另立）；③List 实参克隆 parity（主 a2r
    is_owned_list_arg 无条件 clone vs AA2R last-use）——b27+ 修复后矩阵
    46/46 行为一致，文本形状差异随 ② 同批处置；④臂值位赋值表达式
    aavm cg 不支持——写法规范（值位须纯表达式）保留，b32/g06 以绕开
    形态落盘。三项 KNOWN-DEBT 登记 P514-20。

### W5 管道算子（2026-09-02 用户裁定新增；设计稿 docs/design/pipe-operator.md，讨论稿已定稿口径：首版方法形+字段投影形，函数形二期）

21. [✅ 已完成] 宿主侧：token.rs/lexer.rs 新增 PipeGt 两字符 token（贪婪，
   镜像 ?? 先例）；expr_pratt_with_left_inner 循环顶专用臂脱糖（方法形
   `lhs.m(args)`/字段投影形 `lhs.f`；PREC_PIPE=infix_prec(2)；换行后行首
   `|>` 显式续行——save/restore 越行探测，命中才吞换行，空行计数无损）；
   链糖报错文案更新指向 `|>`。验证：pipe1 探针 VM 四形态（同链 3/同线
   管道 3/多行管道 18/投影 18）+主 a2r 转译与链式产物逐字符一致。
22. [✅ 已完成] lib 镜像：token.at 枚举+kind_name("PipeGt")、lexer.at
   '|'+peek '>' 符号臂、a2r.at ar_expr_tail 换行探测+脱糖臂（复用
   ar_method_call 类型/实参全套机制）；corpus_a2r 新件 g16_pipe_basic
   （四形态）。验证：corpus_a2r g01–g16 逐字符全绿。
23. [✅ 已完成] 99_idiom2 增 m10_pipe 常驻（13 绿）；aavm2 标准门禁 17/2、
   99_unit 13、cargo tf 3350/0、矩阵五腿复跑 46/46（matrix_w5 留档）→
   折叠点④合入 master。设计稿未决项按定稿口径落实：方法形+投影形入，
   函数形二期、is-臂内不支持。2026-09-02。

### 收尾

24. [✅ 已完成] 文档回写（2026-09-02）：divergences.md 增 514 节
    （D-ext-trait/D-pipe-first/D-methodized）；divergence-rules.md §4b
    方法化写法规范 5 条；idiom-upgrade-prereqs.md 状态注记（W5✅/W3
    裁定/W6 不做）；project.md 能力矩阵 514 注记；auto/lib/README 514 段；
    KNOWN-DEBT 核销 P511-1/2/5 + 447-①两债 + P514-W3-2，新登 P514-20。
25. [ ] 复审（/auto-plan:review：验收逐条对代码、遗漏扫描、健康检查、
    spec-impact 元数据）→ `cargo tf` → status: reviewed。

## 复审记录

**复审人**：zhaopuming（AI 代理，/auto-plan:review）**时间**：2026-09-02
**worktree**：.worktrees/plan-514-dev（提交链 8aef313ca→ff86babf4，master 侧
簿记 33421e8c8/a8e7744fb）

### 验收逐条（verify, don't trust——全部在 worktree 重跑或对照实物）

| # | 判定 | 证据 |
|---|---|---|
| 1 | ✅ PASS | tv 3513/3513（含 test_aavm2_a2r_is_corpus g01–g17 逐字符）；cc=主 a2r merge 产物 rustc 零错（仅 E0601）；⑤腿 AA2R 产物 rustc 零红（corpus_a2r 17 件+全 lib 双侧） |
| 2 | ✅ PASS | include-ignored 19/19（tv 全档）；vm-files-ci.yml:48-51 无 --include-ignored；"37/37"为计划时点口径，现行 corpus 增至 17 件仍全绿 |
| 3 | ✅ PASS | 五份留档 scratch/p514/matrix_w3_12/14/15/16+w4_17.log 各 46/46（含一次环境性 flake 的复跑双证） |
| 4 | ✅ PASS | 六文件裁定实物核验（P14+CG28+Ar23 方法在 type 体；lexer 零转换；peek_text/field_idx 合并）；塔顶=⑤腿 aa2r_transpile_merge 全 lib（含方法化 a2r.at 自身）产物 rustc 零错，复现脚本+矩阵双证 |
| 5 | ✅ PASS | C3 完成（W4-17 矩阵留档）；W4-18 引用 D23/D27 既定保留裁定（divergences.md 原文在案）；W4-19 按待澄清②缺省登记；W4-20 三项登记 P514-20；447-①两债核销（E0207→语法错/E0007 证据步骤 4） |
| 6 | ✅ PASS | 99_idiom2 13 件+13 测试 fn 实物核验（test/vm/99_idiom2/）；divergences 514 节/§4b/prereqs/project/README 落盘（a783b0fb9） |
| 7 | ✅ PASS | 复审重跑 cargo tf 3355/3355+96 skip；cargo check 警告 160 条全部预存文件（trans/rust.rs 零警告定位）；延后项全部显式登记（P514-20/P514-R1/D40 续） |

### 全量门禁（review gate 独跑）

- `cargo tf`：3355 passed / 0 failed / 96 skipped ✅
- `cargo tv`：3513 passed / 0 failed ✅
- `cargo tt`：28 failed——**对照基线 87dda951b（独立 worktree 实测）基线即
  同 28 件**（444 期 golden 债族，KNOWN-DEBT 444 条目已更新实测现状）；
  514 净引入 **0**（唯一 2 件新 mismatch 实为 514 顺带根治 `.field` 读位
  误发 `.field()` 缺陷、golden 为缺陷期非法 Rust 捕获，已重生成 ff86babf4
  并复验 28=28）。

### 遗漏/延后/workaround 扫描

- 遗漏：无（1–24 步证据链逐条可溯；翻转脚本/矩阵留档/塔顶产物齐全）。
- 延后：W4-18/19/20 全部走计划预设"显式登记"路径（待澄清①②缺省），
  非静默；W6 不做（待澄清⑤缺省）。
- Workaround：g17 探针收窄为对齐子集（`var list = List.new()` 两侧注解
  预存分歧 → 登记为 P514-R1，语义侧⑤腿 rustc 门覆盖）；方法体非块首
  语句位 self. 约定入 divergence-rules §4b 规范化。
- 复审新发现并已处置：tt 陈旧 golden 族（444 条目更新+2 件重生成）。

### 结论

**PASS**——`status: reviewed`。ready for /auto-plan:merge。

## 待澄清事项

6. **~~W3 P 方法化阻塞~~ ✅ 已结案清偿（2026-09-02，W3-1 语义裁定+W3-2
   补丁续修双落地，commit 8aef313ca；P 方法化全绿详见步骤 12 证据）**。
   原始定性存档：
   调研收官推翻"宿主 VM 可达闭包洞"初判——真实根因两条：①宿主流式链糖
   与方法体语句位前导点冲突（parser.rs parse_body_inner :6846 句首 `.`
   合并；修复=lib 约定：语句位用显式 self.，已验证绿）；②主 a2r 方法体
   习语实参发射缺口 5+ 处级联（Vec.get 借用/usize/Option 字段/self 类型
   注册/str 参 as_str×2，修至仅剩 decl_lookup 位=另一实参环）。按"超出
   小修量级"裁定：lib+rust.rs 回退至 W2 折叠态（corpus 37/37+g 逐字符+
   99_unit 全绿复验）；补丁 scratch/p514_w3_maina2r_methodfixes.patch、
   翻转脚本 scratch/p514_p_methodize.py 存档；重启清单见
   KNOWN-DEBT P514-2。
1. **方法化映射细则**（阻塞 W3 各步骤展开，不阻塞 W1/W2）：状态类型方法
   进 `type` 体 vs `ext` 的边界（缺省：自有类型一体、跨类型扩展 ext）；
   转换覆盖率目标（缺省：不设 100%，以"一对一 Rust 对译可读性"为准，
   逐文件映射表在步骤 12–16 展开时定）。
2. **11.2b 重试若 D40 三缺口修复超限**：登记回退，不阻塞（缺省）。
3. **主 a2r/VM 方法路径若探针暴露大洞**（超出小修量级）：W3 前插修复波、
   W3 顺延（缺省）；是否允许 W3 部分文件先行待裁定。
4. **矩阵②腿修复方案**（阻塞步骤 3）：import 注入 vs std 直映，W0 考古
   定案（缺省倾向 shim 映射，与 429-B1 shim 体系同构）。
5. W6 闭包发射确认不做（缺省，divergence 登记）。
