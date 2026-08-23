//! ShimMethod / Ty / MarshalPlan —— 元信息中间表示。

/// rustdoc 类型表示的投影(足够分类器用)。
#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    I8,
    I16,
    I32,
    U8,
    U16,
    U32,
    I64,
    U64,
    Usize,
    F32,
    F64,
    Bool,
    /// &str / &String(参数按借用传)
    Str,
    /// 按值 String(参数转移所有权;返回值直接拥有)
    StrOwned,
    /// 借用的外来类型(&T 参数 → 传句柄指针;跨 ABI 安全)
    Opaque(String),
    /// 拥有的外来类型(按值 T 参数 → v1 跳过:VM 侧无法构造)
    OpaqueOwned(String),
    /// 泛型参数 T —— 默认导致"不可调用",除非例外表给 mono 提示
    Generic(String),
    SelfTy,
    Void,
}

impl Ty {
    pub fn is_scalar(&self) -> bool {
        matches!(
            self,
            Ty::I8
                | Ty::I16
                | Ty::I32
                | Ty::U8
                | Ty::U16
                | Ty::U32
                | Ty::I64
                | Ty::U64
                | Ty::Usize
                | Ty::F32
                | Ty::F64
                | Ty::Bool
        )
    }
    pub fn rust_name(&self) -> String {
        match self {
            Ty::I8 => "i8".into(),
            Ty::I16 => "i16".into(),
            Ty::I32 => "i32".into(),
            Ty::U8 => "u8".into(),
            Ty::U16 => "u16".into(),
            Ty::U32 => "u32".into(),
            Ty::Usize => "usize".into(),
            Ty::I64 => "i64".into(),
            Ty::U64 => "u64".into(),
            Ty::F32 => "f32".into(),
            Ty::F64 => "f64".into(),
            Ty::Bool => "bool".into(),
            Ty::Str | Ty::StrOwned => "String".into(),
            Ty::Opaque(n) | Ty::OpaqueOwned(n) | Ty::Generic(n) => n.clone(),
            Ty::SelfTy => "Self".into(),
            Ty::Void => "()".into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelfKind {
    /// 关联函数(无 self)
    Static,
    /// &self
    Read,
    /// &mut self
    Write,
    /// 按值 self(wrapper 侧 Box::from_raw 重构后按值调用)
    Move,
}

/// 一条方法元信息(签名层)。
#[derive(Debug, Clone)]
pub struct ShimMethod {
    pub type_name: String,
    pub method: String,
    pub self_kind: SelfKind,
    pub params: Vec<Ty>,
    /// 返回位置的有效类型(Result<T,E> 已解包为 T,见 fallible)
    pub ret: Ty,
    /// 签名含未解决泛型(rustdoc generics.params 非空且方法级)
    pub generic: bool,
    /// 原返回是 Result<T, E>:unwrap_ok 策略——wrapper 解 Ok,
    /// Err 经 cdylib 错误通道传出,VM 侧转 VMError(430-F unwrap 策略)
    pub fallible: bool,
}

/// 分类结果(规律层输出,430 §背景 6 条规则)。
#[derive(Debug, Clone, PartialEq)]
pub enum RetPlan {
    ScalarI32,
    ScalarI64,
    ScalarF64,
    ScalarBool,
    ScalarStr,
    /// 装箱压句柄(std 路径:RustStdlibObject;三方路径:cdylib 裸指针)
    Opaque(String),
    /// 链式(返回 Self,新值):装箱新对象压新句柄(签名 ret == Self)
    ChainSelf,
    /// 链式(返回 &Self/&mut Self,同一对象):原地修改后压回**原**句柄
    /// (builder 模式 `fn has_headers(&mut self, b: bool) -> &mut Self`)
    ChainInPlace,
    Void,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgPlan {
    /// &str:从字符串池弹、借用
    BorrowStr,
    /// String:弹出并转移
    TakeStr,
    ScalarI32,
    ScalarI64,
    /// usize 型参数(索引/长度):按 i64 弹栈,传参时 as usize
    ScalarUsize,
    ScalarF64,
    ScalarBool,
    /// 接收者堆句柄
    SelfHandle,
    /// 外来对象句柄(参数是别的 Opaque 类型)
    OpaqueHandle,
}

#[derive(Debug, Clone)]
pub struct MarshalPlan {
    pub method: ShimMethod,
    pub ret: RetPlan,
    pub args: Vec<ArgPlan>,
    /// 返回 Option<&T>(借用引用)→ 生成 .copied() 装箱
    pub copy_result: bool,
    /// 原 Result 返回(wrapper 解 Ok;Err 走错误通道 → VMError)
    pub fallible: bool,
}

/// 分类失败原因(→ 例外表/跳过清单)。
#[derive(Debug, Clone)]
pub struct Skip {
    pub type_name: String,
    pub method: String,
    pub reason: String,
}
