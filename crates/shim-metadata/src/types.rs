//! ShimMethod / Ty / MarshalPlan —— 元信息中间表示。

/// rustdoc 类型表示的投影(足够分类器用)。
#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    Bool,
    /// &str / String(参数语义由规则层决定借用还是克隆)
    Str,
    /// 外来类型(装箱为 RustStdlibObject)
    Opaque(String),
    /// 泛型参数 T —— 默认导致"不可调用",除非例外表给 mono 提示
    Generic(String),
    SelfTy,
    Void,
}

impl Ty {
    pub fn is_scalar(&self) -> bool {
        matches!(
            self,
            Ty::I32 | Ty::U32 | Ty::I64 | Ty::U64 | Ty::F32 | Ty::F64 | Ty::Bool
        )
    }
    pub fn rust_name(&self) -> String {
        match self {
            Ty::I32 => "i32".into(),
            Ty::U32 => "u32".into(),
            Ty::I64 => "i64".into(),
            Ty::U64 => "u64".into(),
            Ty::F32 => "f32".into(),
            Ty::F64 => "f64".into(),
            Ty::Bool => "bool".into(),
            Ty::Str => "String".into(),
            Ty::Opaque(n) | Ty::Generic(n) => n.clone(),
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
}

/// 一条方法元信息(签名层)。
#[derive(Debug, Clone)]
pub struct ShimMethod {
    pub type_name: String,
    pub method: String,
    pub self_kind: SelfKind,
    pub params: Vec<Ty>,
    pub ret: Ty,
    /// 签名含未解决泛型(rustdoc generics.params 非空且方法级)
    pub generic: bool,
}

/// 分类结果(规律层输出,430 §背景 6 条规则)。
#[derive(Debug, Clone, PartialEq)]
pub enum RetPlan {
    ScalarI32,
    ScalarI64,
    ScalarF64,
    ScalarBool,
    ScalarStr,
    /// 装箱 RustStdlibObject 压句柄
    Opaque(String),
    /// 链式:原地修改后压回接收者句柄(签名 ret == Self)
    ChainSelf,
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
}

/// 分类失败原因(→ 例外表/跳过清单)。
#[derive(Debug, Clone)]
pub struct Skip {
    pub type_name: String,
    pub method: String,
    pub reason: String,
}
