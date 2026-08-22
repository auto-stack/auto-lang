# 交接:Plan 417-E3 —— 有界泛型函数(DIV-TRAIT-VM-1)

> **给新会话**:本文自包含,读完即可开工。基准 master `595a79a8`(2026-08-22)。
> 姊妹项 E2(关联类型)见 `docs/handoff-E2-associated-types.md`,两案互不依赖。

## 0. 工作流(沿用既有模式,勿偏离)

- **worktree 流程**:`git worktree add .worktrees/plan-fix-417e3 -b plan-fix/417e3-bounds master` → 分阶段实施+测试全绿 → `git merge --no-ff plan-fix/417e3-bounds` 回 master → 删 worktree → 推 **origin+gitee 双远端**。
- **全绿纪律**:不达标不合并。宁可缩小单批范围。
- **提交前检查**:`git status` 里若出现 `examples/rust-workspace/`(并行会话 WIP)或 `examples/capability-tests/`(并行 k3 WIP,**未跟踪**),**绝不能 add**——用精确路径 add,禁止 `git add -A`(上一会话曾误扫,靠 `git rm --cached` 挽回)。
- **测试命令**:
  - a2r golden:`RUST_MIN_STACK=33554432 cargo test -p auto-lang --lib --features test-trans a2r_tests`(基线 **344/344**)
  - 全量 lib:`RUST_MIN_STACK=33554432 cargo test -p auto-lang --lib`(基线 3051 过/1 败 = route::discovery test_exists 环境项)
  - parity 三方:`cd parity && cargo build -p auto-parity && ./target/debug/auto-parity.exe --auto-binary <worktree>/target/debug/auto.exe run <lib>`
  - 实机回归(改 VM 时必跑):038(21/0)/013(22/0/0)/015(13/0/1),`cd examples/ui/XXX/tests && AUTO_BIN=<exe> python desktop_mcp.py`
- **环境陷阱**:python 补丁必须二进制安全(`open(p,'rb')`/`'wb'`);renderer.rs 是 CRLF、rust.rs 是 LF,多行锚点先查行尾;Git Bash heredoc 吃转义——复杂补丁优先用 Edit 工具;D 盘易满(每 worktree target ~10GB),合并后立即 `git worktree remove --force`。

## 1. 任务定义

**DIV-TRAIT-VM-1**(parity/docs/known-divergences.md,trait_advanced 库):有界泛型函数
`fn max_of<T has Comparable>(a T, b T) T` 在 VM 不可用。2026-08-22 探针实证**双断点**:

### 断点①语法:泛型参数表拒 `has`

```
spec Comparable { fn compare(other int) int }
fn max_of<T has Comparable>(a T, b T) T { ... }
→ error: Expected '>' or ',' in generic parameter list, got has
```

- 入口:`crates/auto-lang/src/parser.rs` `parse_generic_param`(≈:9868)/`parse_generic_params`(≈:9905)。
- **关键既有设施**:`ast/types.rs:370 TypeParam.constraint: Vec<Type>` 已存在!Plan 364 W3 的属性式界约束 `#[with(T as A + B)]` 已收集到该字段且 a2r 已发射 `T: A + B`。所以断点①只是**给 parse_generic_param 加 `has <SpecName>` 分支写入 constraint**——不需要新字段。
- 注意 `parse_generic_params` 同时服务 `<T>` 与 `[T]`(Plan 368 FU-1,stdlib 用 `[T]`);`has` 只在有界场景出现,两种括号都要接受。

### 断点②分派:泛型参数上的方法调用静态解析 → 链接错

```
fn max_of<T>(a T, b T) T { if a.compare(0) >= b.compare(0) { a } else { b } }
→ link error: Undefined symbol: T.compare in module <main>
```

- codegen 把 `a.compare(...)` 按接收者**静态类型名**拼 `T.compare` 发 CALL reloc。
- 解析点:`crates/auto-lang/src/vm/codegen.rs` 方法调用编译臂(`expr_to_name(obj)` → `format!("{}.{}", ...)` ≈:1763/1772 一带)。
- **关键既有设施(修复路径的支点)**:VM 已有 **CALL_SPEC 运行时动态分派**——`crates/auto-lang/src/vm/engine.rs:5058` 起:从栈上 receiver 的堆对象读**运行时类型名**(GenericInstanceData.mono_name / type_tag),拼 `"<真实类型>.<方法>"` 查 exports。E4 修复时实测过该路径("CALL_SPEC: no function 'Robot.announce'"报错即出自这里)。
- **修复方向**:当接收者的静态类型是泛型参数(查 fn 的 generic_params / var_types 中 Type::Unknown+参数名匹配)时,改发 CALL_SPEC(动态)而非静态 CALL。先写最小探针确认 CALL_SPEC 路径对用户类型对象确实解析到 `Score.compare`。

### a2r 侧(断点③,转译器现状也是坏的)

裸 `<T>` 的 `fn max_of<T>(a T, b T) T` 当前发射为:
```rust
fn max_of(a: T, b: T) -> impl T { ... }   // 双错:泛型声明丢失 + impl T 误用
```
- 需要发射 `fn max_of<T: Comparable>(a: T, b: T) -> T`;无 bound 时 `fn max_of<T>(...) -> T`。
- 界约束发射可参照 Plan 364 W3 的 constraint→`T: A + B` 既有路径(搜 `constraint` in trans/rust.rs)。
- 返回类型 `impl T` 的误发射:查 generic 返回类型的 rust_return_type_name 分支。

## 2. 实施顺序建议(每阶段独立可验证)

1. **P1 语法**(半天):parse_generic_param 接受 `T has Spec`(单 spec 即可,多 bound `has A + B` 可选)。验证:解析探针不再报错 + 既有 golden 零回归。
2. **P2 VM 动态分派**(1-2 天):泛型接收者方法调用改 CALL_SPEC。验证:下方完整探针 VM 跑通输出 9;trait_vm_tests(已有 `crates/auto-lang/src/tests/trait_vm_tests.rs`)加有界泛型用例;实机三例回归。
3. **P3 a2r 发射**(半天-1 天):`<T: Spec>`/`<T>` 正确发射。验证:golden 新用例(建议 `03_control_flow/014` 或 `08_generators` 邻位;**golden 流程**:建 `<name>.at` + 空 `.expected.rs` → 跑测试生成 `.wrong.rs` → 人工核验是合法 Rust(独立 cargo build)→ 提升为 expected)。
4. **P4 trait_checker 界校验**(可选,小):调用点实参类型须满足 bound——查 `crates/auto-lang/src/trait_checker.rs` 现有 check_conformance 模式。
5. **P5 parity 升级**(半天):`parity/libs/trait_advanced/` README 的 sub-scenario C 从 L3 升 L1——在 `tests/auto/` 加有界泛型测试 + `tests/rust/trait_advanced.rs` 加同名 oracle(Rust 原生 trait bound);phase map 已注册 d2。三方全绿后翻转 DIV-TRAIT-VM-1 并在 417 计划 E3 行打勾。

### 完整验收探针(基线:VM 输出 9;a2r 产物独立编译输出 9)

```auto
spec Comparable {
    fn compare(other int) int
}

fn max_of<T has Comparable>(a T, b T) T {
    if a.compare(0) >= b.compare(0) { a } else { b }
}

type Score as Comparable {
    val int
    fn compare(other int) int {
        return self.val - other
    }
}

fn main() {
    var x Score = Score { val: 3 }
    var y Score = Score { val: 9 }
    print(max_of(x, y).val)
}
```
(值位尾 if 已由 6c454f19 修好,a2r 侧此形态免分号 ✓)

## 3. 相邻事项(明确不在本批范围)

- **DIV-TRAIT-A2R-2**:`type T as Comparable<int>` 泛型 spec 实现丢类型参数(E0107)——独立分歧,若 P3 顺手可修,否则留档。
- E4 的默认方法继承(已修,abc03b3a):泛型 spec 的默认方法合成**未覆盖**——若 P2 中 TypeDecl 合成路径(spec_decls 经 TypeStore)遇泛型 spec,按需扩展。

## 4. 文档收尾(合并前)

- `parity/docs/known-divergences.md`:DIV-TRAIT-VM-1 详条翻转 ✅(含修复描述与测试锚点)。
- `docs/plans/417-script-rollout-residuals.md`:E3 行打勾、顺序行更新(届时 Phase E 仅剩 E2)。
- 若有新发现登记 `docs/plans/KNOWN-DEBT-AND-RISKS.md`(格式照旧)。
- 完成后可在 `docs/handoff-2026-08-22.md` 追加一段会话记录。

## 5. 本族已修项(上下文,勿重做)

- E1 char_at 推断 Int(aff61954)、E5 http 解析验证翻转+D3 5/5(f599dde2)、E4 spec 默认方法继承(abc03b3a:trait_checker 放行 + codegen 经 TypeStore 合成 Type.method,重声明=覆盖)、a2r 值位尾 if 免分号(6c454f19,trait_advanced 三方 10/10)。
- parity 四库全绿基线:string_utils 22/22、generators 6/6、http_client_sync 5/5、trait_advanced 10/10。
