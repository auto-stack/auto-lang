# 交接:Plan 417-E2 —— spec 关联类型(DIV-TRAIT-LANG-1)

> **给新会话**:本文自包含,读完即可开工。基准 master `595a79a8`(2026-08-22)。
> 姊妹项 E3(有界泛型)见 `docs/handoff-E3-bounded-generics.md`,两案互不依赖。
> 工作流/测试命令/环境陷阱与 E3 文档 **§0 完全相同**,此处不重复——先读那份的 §0。

## 1. 任务定义

**DIV-TRAIT-LANG-1**:Auto 的 spec 语法**没有关联类型**构造。

```auto
spec Container {
    type Item
    fn get(i int) Item
}
```
→ 当前解析失败:**"Expected method declaration in spec"**(还连锁出 ~20 个错误)。
入口:`crates/auto-lang/src/parser.rs` 的 spec 体解析(`spec_method` 在 :8855;
spec 体循环拒绝 `type` 开头的成员)。

这是**语言级特性**(语法→类型系统→VM→a2r 四层),Plan 243 时代即列为 roadmap;
trait_advanced 库 sub-scenario B 因此整个缺失(Rust oracle 也无对应测试,README
"### B. associated types" 段)。预估 2-3 天。

## 2. 四层实施路径

### P1 语法与 AST(半天-1 天)

- `ast/spec.rs:17 SpecDecl` 增字段:`associated_types: Vec<AssociatedType>`
  (新结构:`{ name: Name, bound: Option<Type> }`,bound 留扩展)。
- parser spec 体循环:识别 `type <Name>`(可选 `;`)成员 → 收进 associated_types。
- **决定点(建议)**:实现处的绑定语法。既有泛型 spec 实现用位置式
  `type Heap<T> as Storage<T>`(ast SpecImpl{spec_name,type_args})。关联类型建议
  **命名式**:`type Stack as Container<Item=int>`——清晰、与 Rust 习惯一致、
  可与位置式 type_args 共存(先 named、后 positional 兜底)。AST 上给
  TypeDecl.spec_impls 或新字段存绑定 map。
- 参考:Plan 364 W3 的 spec 泛型与 spec_impls 既有管线(generic_spec_tests.rs)。

### P2 类型检查(半天)

- trait_checker.rs:实现者若绑定关联类型,spec 方法签名中的 `Item` 需按绑定替换后
  再比对返回/参数类型(现有 check_conformance 的类型比对照用)。
- 未绑定时报错(明确错误信息,勿静默)。

### P3 VM 代码化(1 天)

- 关联类型在 VM 无运行时实体——方法编译时把签名/体内的 `Item` **文本替换为实现
  处的绑定类型**即可(合成 Type.method 时换参/换返回;参考 E4 已落的合成管线:
  vm/codegen.rs TypeDecl 编译臂 + spec_decls 经 TypeStore 查询,abc03b3a)。
- spec 本体无字节码(Stmt::SpecDecl 仅注册),所以 VM 侧基本是"合成时替换"。

### P4 a2r 发射(半天-1 天)

Rust 原生映射,手工对照样例:
```rust
trait Container { type Item; fn get(&self, i: i64) -> Self::Item; }
impl Container for Stack { type Item = i64; fn get(&self, i: i64) -> Self::Item { ... } }
```
- trait 发射:associated_types → `type Item;`;方法签名里的关联类型引用 →
  `Self::Item`(trans/rust.rs 的 spec→trait 发射臂,搜 `trait `+spec_decl)。
- impl 发射:绑定 → `type Item = i64;`(type as Spec 实现臂)。

### P5 parity 升级 + 三方(半天)

- `parity/libs/trait_advanced/`:sub-scenario B 从 L3 升 L1——auto/ 加带关联类型的
  spec+实现,tests/auto/ 加 TAP 用例,tests/rust/ 加同名 Rust oracle(Rust 原生,
  直接写)。
- 验收:`./target/debug/auto-parity.exe --auto-binary ../target/debug/auto.exe run
  trait_advanced` 全绿(现基线 10/10,升级后应为新增用例数)。
- golden:建议加 a2r 用例锁定 trait/impl 的关联类型发射形状。

## 3. 验收探针

```auto
spec Container {
    type Item
    fn get(i int) Item
    fn first() Item
}

type IntBox as Container<Item=int> {
    data List<int>
    fn get(i int) int {
        return self.data[i]
    }
    fn first() int {
        return self.data[0]
    }
}

fn main() {
    var b IntBox = IntBox { data: [7, 8, 9] }
    print(b.get(1))
    print(b.first())
}
```
基线:VM 输出 `8` `7`;a2r 产物独立编译输出同值;三方 TAP 全绿。

## 4. 与既有设施的关系(读一遍少踩坑)

- **E4 默认方法继承已落地**(abc03b3a):TypeDecl 编译臂会为未重声明的 spec 默认
  方法合成 `Type.method`——P3 的关联类型替换要**穿过这条合成管线**(合成方法体
  内若引用 Item 同样替换)。trait_vm_tests.rs 已有继承/覆盖/抽象三态用例可参照。
- **值位尾 if 已修**(6c454f19):`if a >= b { a } else { b }` 作函数尾免分号,
  探针里放心用。
- **a2r spec→trait 发射已有**:非泛型 spec 完整工作(trait_advanced 10/10 佐证);
  P4 是在其上加 associated_types 分支,不是新管线。
- **DIV-TRAIT-A2R-2**(`type T as Comparable<int>` 丢类型参数,E0107)是邻病但
  独立——若 P4 顺路修了就在分歧条一并翻转,否则留档。

## 5. 文档收尾(合并前)

- `parity/docs/known-divergences.md`:DIV-TRAIT-LANG-1 翻转 ✅;trait_advanced
  README sub-scenario B 段改写为 L1。
- `docs/plans/417-script-rollout-residuals.md`:E2 行打勾——**届时 Phase E 五项
  全关,417 可进入 finish-plan 评估**(165 checkbox 回填属 Plan 359 收尾,勿混)。
- 新发现登记 KNOWN-DEBT-AND-RISKS.md;会话记录追加 handoff-2026-08-22.md。

## 6. 风险提示

- 这是四层语言特性,**按 P1→P5 顺序小批提交**,每批全绿——切忌一次大改。
- `type` 关键字在 spec 体内与类型声明语句共用 token,注意 spec 体循环里的分派
  优先级(先判 `type` 成员再走 spec_method)。
- 绑定语法 `Container<Item=int>` 的解析与泛型 spec 实现的位置式 `Storage<T>`
  在同一位置出现——`<` 后既可能是类型参数也可能是命名绑定,需 peek `Name=`
  形态区分;拿不准就先只支持命名式。
