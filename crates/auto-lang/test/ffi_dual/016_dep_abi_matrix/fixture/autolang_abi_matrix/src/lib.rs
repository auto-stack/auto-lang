//! PLAN-592 T2: dep marshalling 全矩阵夹具。
//!
//! 覆盖面(对齐 shim-metadata 字母表 v/i/l/f/b/s/p):
//! - 整型宽槽收窄:u8/u16/u32/u64/usize/i8/i16/i32/i64 的参数+返回成对
//!   (wrapper 内显式 `as T` 收窄——VM 侧 i64 宽槽语义);
//! - f32 经 f64 槽往返 / f64 直达;
//! - bool(al 有效性,VM 侧 & 0xFF 掩码);
//! - &str(BorrowStr)/String(TakeStr) 参数,Unicode/空串返回;
//! - arity:v1 方法 ABI 参数上限 3 **含接收者**——`add2`(接收者+2)=上限内,
//!   `quad`(接收者+3)=上限外负面(marshaller RuntimeError "v1 supports ≤3");
//! - 边界值:i64::MIN/MAX(VM heap-aware 编码路径);
//! - 自由函数 plan-212 四类签名中的三类已覆盖(()→i64、(String)→String、
//!   (i64,i64)→i64)+ 一类未覆盖负面((f64,String)→i64,PLAN-592 T8 后断言
//!   VMError 而非静默 0);
//! - `Pt` pub 混宽字段(合成 getter + PLAN-592 `p.a` 字段语法桥接验收面)。
//!
//! **不入本 crate 的面**:超范围截断观测(`echo_u8(1000)`→232)与
//! `u64_max()`(u64::MAX 经 i64 槽不往返)——a2r 腿对超范围字面量编译不过,
//! 故只作为 VM 腿测试体内的特征化钉死,不进三腿共享语料(见 ffi_dual_016)。

pub struct Num {
    seed: i64,
}

impl Num {
    pub fn new() -> Num {
        Num { seed: 0 }
    }

    /// 整型 echo 族:参数 + 返回成对,宽槽收窄语义
    pub fn echo_i8(&self, x: i8) -> i8 {
        x
    }
    pub fn echo_i16(&self, x: i16) -> i16 {
        x
    }
    pub fn echo_i32(&self, x: i32) -> i32 {
        x
    }
    pub fn echo_i64(&self, x: i64) -> i64 {
        x
    }
    pub fn echo_u8(&self, x: u8) -> u8 {
        x
    }
    pub fn echo_u16(&self, x: u16) -> u16 {
        x
    }
    pub fn echo_u32(&self, x: u32) -> u32 {
        x
    }
    pub fn echo_u64(&self, x: u64) -> u64 {
        x
    }
    pub fn echo_usize(&self, x: usize) -> usize {
        x
    }
    /// f32 经 f64 槽(wrapper 内 as f32 收窄)
    pub fn echo_f32(&self, x: f32) -> f32 {
        x
    }
    pub fn echo_f64(&self, x: f64) -> f64 {
        x
    }
    /// bool:返回仅 al 有效(VM 侧掩码),参数经 i64 槽
    pub fn echo_bool(&self, x: bool) -> bool {
        x
    }

    /// arity 上限内(接收者 + 2 参 = 3 ABI 参数)
    pub fn add2(&self, a: i64, b: i64) -> i64 {
        a.wrapping_add(b).wrapping_add(self.seed)
    }

    /// arity 上限外负面(接收者 + 3 参 = 4 ABI 参数 → v1 RuntimeError)
    #[allow(dead_code)]
    pub fn quad(&self, a: i64, b: i64, c: i64) -> i64 {
        a.wrapping_add(b).wrapping_add(c).wrapping_add(self.seed)
    }

    /// 边界值(VM 48 位内联外 → heap-aware 编码路径)
    pub fn i64_min(&self) -> i64 {
        i64::MIN
    }
    pub fn i64_max(&self) -> i64 {
        i64::MAX
    }

    /// u64::MAX:经 i64 槽不往返(DIV-DEP-4 特征化观测面,不入三腿语料)
    pub fn u64_max(&self) -> u64 {
        u64::MAX
    }
}

pub struct Words {
    prefix: String,
}

impl Words {
    pub fn new() -> Words {
        Words { prefix: "hello".to_string() }
    }

    /// &str 参数(BorrowStr:CStr→String 借用)
    pub fn greet(&self, name: &str) -> String {
        format!("{}, {}!", self.prefix, name)
    }

    /// String 参数(TakeStr:所有权转移)
    pub fn shout(&self, s: String) -> String {
        format!("{}!!", s.to_uppercase())
    }

    /// Unicode 往返:CJK + emoji + 组合符
    pub fn unicode(&self) -> String {
        "汉字✓🎉e\u{301}".to_string()
    }

    /// 空串(CString 空串边界)
    pub fn empty(&self) -> String {
        String::new()
    }
}

/// pub 混宽字段:合成 getter(Pt.a/Pt.b/Pt.c)+ PLAN-592 `p.a` 字段语法桥接验收面
pub struct Pt {
    pub a: u8,
    pub b: String,
    pub c: i64,
}

impl Pt {
    /// 静态构造:3 参(静态无接收者,恰在 v1 上限内)
    pub fn new(a: u8, b: String, c: i64) -> Pt {
        Pt { a, b, c }
    }
}

/// 自由函数:plan-212 wrapper 已覆盖签名类
/// ()→i64
pub fn free_noop() -> i64 {
    42
}

/// (String)→String
pub fn free_s(s: String) -> String {
    format!("[{s}]")
}

/// (i64,i64)→i64
pub fn free_ll(a: i64, b: i64) -> i64 {
    a.wrapping_mul(10).wrapping_add(b)
}

/// 未覆盖签名类负面:(f64,String)→i64——PLAN-592 T8 后断言 VMError(而非静默 0)
pub fn free_uncovered(scale: f64, s: String) -> i64 {
    (scale * s.len() as f64) as i64
}
