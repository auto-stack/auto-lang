//! PLAN-596 T-01: 591 V2 能力 fixture。
//!
//! 设计约束(实现侧注意,勿"顺手修复"破坏用例意图):
//! - **trait impl 一律手写,禁 derive**——rustdoc JSON 对 derive 展开的 impl
//!   提取不可靠,classify(PLAN-596 T3)按显式 trait impl 归属提取白名单转发。
//! - `Temp` **不得**添加 inherent `to_string`(013 假象防线:to_string 必须只能
//!   经 trait 白名单转发到达,证明走的是 `<T as Display/ToString>` 路径)。
//! - 泛型方法采用**方法级泛型参数、接收者具体**形态(`pick_max<T: Ord>`)——
//!   591 原文 `Pair.max()`(泛型接收者)的等效实现:mono 提示来源与方法实参
//!   同源(调用点实参类型),与自由函数 `pick` 的回填机制统一。
//! - `apply` 的 `Box<dyn Fn(i64)->i64>` 形参在 v1 classify 被"含 Fn 的 Opaque
//!   参数"规则 skip——PLAN-596 T5 为其单开 Callback 参数类(callbacks 元数据
//!   + adapter),不要改回普通 Opaque。
//! - 方法签名满足 dep marshaller ABI 参数上限 3(含接收者):apply = p + f + x。

use std::fmt;

/// 仅 impl Display(无 inherent to_string)——T3 白名单转发的首证类型。
pub struct Temp {
    pub degree: i64,
}

impl fmt::Display for Temp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}deg", self.degree)
    }
}

/// V1×V2 组合:String 参数 + opaque 返回 + (调用侧)trait 转发与 pub 字段读。
pub fn make(tag: String) -> Temp {
    Temp { degree: tag.len() as i64 }
}

/// Clone 白名单转发 + 深拷贝独立性(副本字段写不影响原对象)。
#[derive(Debug)]
pub struct Tagged {
    pub tag: String,
    pub hits: i64,
}

impl Tagged {
    pub fn new(tag: String, hits: i64) -> Tagged {
        Tagged { tag, hits }
    }

    /// &mut 方法(既有能力,供独立性别用例观察原对象)
    pub fn bump(&mut self) {
        self.hits += 1;
    }
}

/// 手写 Clone(非 derive):rustdoc 可见的显式 trait impl。
impl Clone for Tagged {
    fn clone(&self) -> Self {
        Tagged { tag: self.tag.clone(), hits: self.hits }
    }
}

/// 泛型自由函数:调用点实参回填 mono 提示(pick::<i64>/pick::<String>)。
pub fn pick<T: Ord>(a: T, b: T) -> T {
    if a >= b { a } else { b }
}

/// 泛型方法宿主:方法级泛型 `pick_max<T: Ord>`(接收者具体)。
pub struct Pair {
    pub a: i64,
    pub b: i64,
}

impl Pair {
    pub fn new(a: i64, b: i64) -> Pair {
        Pair { a, b }
    }

    pub fn sum(&self) -> i64 {
        self.a + self.b
    }

    /// 泛型方法(591 原文 Pair.max() 的等效实现形态,见文件头注记)
    pub fn pick_max<T: Ord>(&self, x: T, y: T) -> T {
        if x >= y { x } else { y }
    }
}

/// 回调 adapter 原型宿主:Box<dyn Fn> 形参 → T5 callbacks 元数据 + adapter。
pub struct Invoker {
    _label: String,
}

impl Invoker {
    pub fn new() -> Invoker {
        Invoker { _label: "inv".to_string() }
    }

    /// f(x) 直传——591 V2-6 期望 apply(|x| x*2+1, 5) == 11(接收者不参与计算)。
    pub fn apply(&self, f: Box<dyn Fn(i64) -> i64>, x: i64) -> i64 {
        f(x)
    }
}
