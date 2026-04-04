use auto_val::AutoStr;

#[derive(Debug, Clone)]
pub enum VarKind {
    Int,
    Float,
    Str,
    Bool,
    Select,
    Unknown(AutoStr),
}
