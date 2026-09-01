---
plan_id: PLAN-514
status: executing               # drafting → executing → execution_done → reviewed → archived
feature_name: aavm-idiom2-methods
author: [zhaopuming]
created_at: 2026-09-01
updated_at: 2026-09-01

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-017]     # 自举：用 Auto 写 Auto 编译器（aavm）

affects: [aavm]
current_step: 2
total_steps: 22
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

3. [ ] 矩阵②腿修复（主 a2r `File::read_text` 族独立转译导入/shim）→
   parity 全绿复跑留档。验证：矩阵命令全绿（P511-5 清偿）。
4. [ ] 宿主修复批：E0201→语法错；嵌套 fn 静默失效；探针暴露的主 a2r
   方法发射洞顺修。验证：99_idiom2 全绿 + rustc 零错 + `cargo t` 零回归。
5. [ ] 折叠点①：CI 绿 + master 合入（`fix(aavm-host): Plan 514 W1 矩阵②腿
   +方法路径加固 (Plan 514)`）。

### W2 AA2R W5 + P511-1（worktree 续）

6. [ ] corpus_a2r g09–g15 语料先行落盘。验证：AA2R 侧红清单（unsupported
   报错即红证）。
7. [ ] `auto/lib/a2r.at`：ar_prescan_type 方法表 + ar_prescan_ext +
   ar_run Ext 分支。验证：g09–g10 预扫层转绿（发射层可仍红）。
8. [ ] ar_emit_ext + ar_emit_fn2 接收器合成 + self 入作用域 + .field→
   self.field。验证：g11–g13 live 对拍绿。
9. [ ] 方法调用位（ar_method_call 查表/static Type::/构造器）+ type 发射
   分离。验证：g09–g15 全绿 + rustc 零错。
10. [ ] P511-1 双根因修复（CJK 注释词法 + arr_flag 接收者跟踪）。
    验证：`--include-ignored` 37/37 绿。
11. [ ] AA2R 腿去 ignore + CI 口径更新 + 折叠点②（矩阵+CI 绿合入）。

### W3 lib 方法化 γ4（worktree 续；每文件一提交全绿才进）

12. [ ] token.at 试点方法化 + 塔顶样板验证（AA2R 转译方法化 token.at →
    rustc 零错 → 行为一致）。验证：全闸门 + 矩阵绿。
13. [ ] lexer.at 转换。验证：同上。
14. [ ] typeinfo.at + parser.at 转换。验证：同上。
15. [ ] codegen.at + engine.at 转换。验证：同上。
16. [ ] a2r.at 自身转换（塔顶终局）+ 折叠点③（矩阵①–⑤全绿合入）。

### W4 447 尾巴（worktree 续）

17. [ ] C3：cg_expr/cg_stmt p_kind 链 is 化（方法体内顺路）。
    验证：tv-aavm2 绿 + 矩阵绿。
18. [ ] t_array_elem Type 载荷化（D23/D27）。验证：tv-aavm2 绿。
19. [ ] 11.2b f-string 批量改写重试（D40 前置清偿；卡则按待澄清②登记
    回退）。验证：矩阵 ③④⑤ 绿（行为不变）。
20. [ ] Phase 11 收账余项核对处置（447 归档清单②③④逐项）。
    验证：逐项证据或显式登记。

### 收尾

21. [ ] 文档回写：divergences.md（W5 分叉+方法风格规范入 divergence-rules
    §4）、idiom-upgrade-prereqs.md 状态注记、project.md、auto/lib/README、
    KNOWN-DEBT 核销（P511-1/2/5、P447-①两债）。
22. [ ] 复审（/auto-plan:review：验收逐条对代码、遗漏扫描、健康检查、
    spec-impact 元数据）→ `cargo tf` → status: reviewed。

## 复审记录

## 待澄清事项

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
