//! PLAN-596 复审 F-2: mono 重建对抗专用 fixture——单泛型自由函数最小面。

/// 泛型自由函数:调用点实参类型词法推导 mono 提示(i64/String 实例)。
pub fn pick<T: Ord>(a: T, b: T) -> T {
    if a >= b { a } else { b }
}
