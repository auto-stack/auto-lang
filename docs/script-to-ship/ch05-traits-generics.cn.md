# 第 5 章 —— Trait 与泛型

Auto 里 `trait` 这个词叫 `spec`。声明它，用 `type T as Spec { ... }` 在类型上实现它，再通过 spec 类型的参数分发——a2r 产出 Rust 的 `trait` / `impl` / `Box<dyn>`。

<Listing file="script-to-ship/ch05-traits-generics/05_shapes.at" view="scriptship" caption="一个 spec 带两个实现，动态分发" />

## 当前可用（L1）

示例展示了已验证的子集：一个带必需方法的 spec、两个实现、以及一个接受 spec 类型并动态分发的函数 `area_of(s Shape)`。a2r 把它转成 Rust 的 `trait Shape`、两个 `impl Shape for ...` 块、以及一个 `Box<dyn Shape>` 参数。这是真正的动态分发，在 parity 套件（`parity/libs/trait_advanced/`，L1 10/10）里与原生 Rust 三向验证。

## 诚实的边界（L3）

Auto 的 spec 系统比 Rust 的 trait 系统年轻，有些高级形式尚未在两个后端上都支持。这些在 `parity/docs/known-divergences.md` §"trait_advanced (D2)" 里作为开放缺口公开记录，不是隐瞒：

- **关联类型** —— Auto 的 spec 语法没有 `type Item;` 构造（语言缺口；L3）。
- **返回值的默认方法体** —— a2r 包裹方法体导致返回类型不匹配（a2r 缺口；void 默认方法可用）。
- **泛型 spec 实现** —— a2r 在 `impl Comparable<i32> for T` 上丢弃具体类型参数（a2r 缺口）。
- **有界泛型函数**（`fn max<T has Comparable>`）—— bound 语法被拒绝，VM 无法通过类型参数分发。

公开列出这些的意义：Auto 不会在没做好的地方假装做好了。L1 基线（上面的示例）是已验证的；L3 项在路线图上。当一章用到某个特性时，它会告诉你该特性处于哪一档。

下一章：[发布：上线 →](ch06-ship-release)
