//! PLAN-591: V1 布局/语义 fixture。
//! 设计约束(430 v1 管线):
//! - 静态构造器 ABI 参数 ≤3(含接收者前导的方法面同理);
//! - Option 返回用 &str/自类型(591 T2 仅 s/p 槽,None→null 无哨兵歧义);
//! - Result Err 文案固定前缀 `Err:bad-input:`,供 VM 腿负面断言锚定;
//! - 字段全 pub(合成 getter 面 + 591 T1 offset 直读双通道);
//! - 嵌套字段类型 derive(Clone)(合成 getter 对不透明字段走 clone)。

#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Circle,
    Square,
    Triangle,
}

#[derive(Clone)]
pub struct Inner {
    pub n: i64,
    pub label: String,
}

impl Inner {
    pub fn new(n: i64, label: &str) -> Self {
        Self {
            n,
            label: label.to_string(),
        }
    }
}

pub struct Outer {
    pub tag: String,
    pub inner: Inner,
}

impl Outer {
    pub fn new(n: i64, label: &str) -> Self {
        Self {
            tag: "outer".to_string(),
            inner: Inner::new(n, label),
        }
    }
}

/// 乱序交错字段声明(u8/String/u8/i64/bool):repr(Rust) 重排由编译器定,
/// 真实偏移经 591 T1 探针制备——乱序声明放大偏移错误的可观测性。
pub struct Messy {
    pub a: u8,
    pub tag: String,
    pub b: u8,
    pub total: i64,
    pub flag: bool,
}

impl Messy {
    pub fn new(total: i64, tag: &str, flag: bool) -> Self {
        Self {
            a: 7,
            tag: tag.to_string(),
            b: 11,
            total,
            flag,
        }
    }

    /// Option<&str> 返回(591 T2 nullable 面):tag 命中 → Some,否则 None。
    /// 方法名避开发射器内建劫持名(find 会撞 a2r_std::str_find)。
    pub fn lookup(&self, key: &str) -> Option<&str> {
        if key == self.tag.as_str() {
            Some(self.tag.as_str())
        } else {
            None
        }
    }

    /// &mut 变异(591 V1-2 字段写断言的 Rust 侧读回通道)。
    pub fn add(&mut self, delta: i64) {
        self.total += delta;
    }

    pub fn total_snapshot(&self) -> i64 {
        self.total
    }
}

pub struct Point {
    pub x: i64,
    pub y: i64,
}

impl Point {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    /// Result<Point, String> 返回(591 T2 fallible 面):
    /// "x,y" 形态合法;其余 Err(固定前缀文案)。
    pub fn parse(s: &str) -> Result<Point, String> {
        let bad = format!("Err:bad-input:{s}");
        match s.split_once(',') {
            Some((xs, ys)) => match (xs.parse::<i64>(), ys.parse::<i64>()) {
                (Ok(x), Ok(y)) => Ok(Point { x, y }),
                _ => Err(bad),
            },
            None => Err(bad),
        }
    }
}

pub struct Shape {
    pub kind: Kind,
    pub radius: i64,
}

impl Shape {
    pub fn new(kind_idx: i64, radius: i64) -> Self {
        let kind = match kind_idx {
            0 => Kind::Circle,
            1 => Kind::Square,
            _ => Kind::Triangle,
        };
        Self { kind, radius }
    }

    /// enum 判别(591 V1-7):判别值与 match 语义一致,经 bool 探针三轨可断言。
    pub fn is_circle(&self) -> bool {
        matches!(self.kind, Kind::Circle)
    }

    pub fn is_square(&self) -> bool {
        matches!(self.kind, Kind::Square)
    }

    pub fn is_triangle(&self) -> bool {
        matches!(self.kind, Kind::Triangle)
    }
}

/// Display(591 D2 DIV-DEP-8 print 面三轨对齐):rustdoc 据此合成 to_string
/// 进 shim 包,VM print/.to(str) 路由该面;a2r 发射 `println!("{}", m)` 同为
/// Display 形态;oracle 逐字镜像。
impl std::fmt::Display for Messy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Messy({}, {}, {}, {}, {})",
            self.a, self.tag, self.b, self.total, self.flag
        )
    }
}

/// features 变体(对抗②):width 字段型随 features 组合切换——
/// 同 crate 异 features → 字段型/布局不同 → 指纹必变 → 重建;
/// 宽窄字段对同一构造实参呈现不同值(陈旧复用当场暴露)。
pub struct FeatCfg {
    #[cfg(not(feature = "wide"))]
    pub width: i32,
    #[cfg(feature = "wide")]
    pub width: i64,
    pub note: String,
}

impl FeatCfg {
    pub fn new(total: i64, note: &str) -> Self {
        Self {
            width: total as _,
            note: note.to_string(),
        }
    }
}
