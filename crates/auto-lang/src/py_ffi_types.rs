//! Plan 222: Python FFI type definitions — no pyo3 dependency
//!
//! Pure data types for Python FFI marshalling. Used by both codegen (no python feature)
//! and py_ffi.rs (with python feature) to describe function signatures.

/// Supported Python types for FFI marshalling.
#[derive(Debug, Clone, PartialEq)]
pub enum PyType {
    None,
    Bool,
    Int,
    Float,
    String,
    List,
    /// `T | None` / `Optional[T]`（Plan 567 T16，W3 注解预言机）——
    /// nullability 知识：封送时 None → null 值，内型走 D4 强制/动态。
    Nullable(Box<PyType>),
    /// Runtime auto-detection of return type (Python is dynamically typed)
    Auto,
}

// ============================================================================
// Plan 567 T16/T19（W3 注解预言机）: 返回注解知识表 + nullable lint 收集器
// ============================================================================

/// use.py 导入项名 → 返回注解 PyType。py_ffi 注册期（python feature）
/// 填充；codegen 消费（py_return_types 灌注 + nullable lint）。无 python
/// feature 构建时恒空（查询零命中，行为与 567 前一致）。
pub static PY_RETURN_ANNOTATIONS: std::sync::OnceLock<
    std::sync::RwLock<std::collections::HashMap<String, PyType>>,
> = std::sync::OnceLock::new();

fn annotations_table(
) -> &'static std::sync::RwLock<std::collections::HashMap<String, PyType>> {
    PY_RETURN_ANNOTATIONS.get_or_init(|| std::sync::RwLock::new(std::collections::HashMap::new()))
}

/// 注册期记录一条返回注解知识。
pub fn record_return_annotation(name: &str, t: PyType) {
    if let Ok(mut m) = annotations_table().write() {
        m.insert(name.to_string(), t);
    }
}

/// 查询导入项的返回注解（无知识 = None）。
pub fn lookup_return_annotation(name: &str) -> Option<PyType> {
    annotations_table().read().ok().and_then(|m| m.get(name).cloned())
}

/// nullable lint 收集器：已知 `T | None` 返回的调用点未判空直用
///（Store/语句位裸消费）时记录（Plan 567 T19——W 级，不阻断）。
pub static PY_NULLABLE_LINT_HITS: std::sync::OnceLock<
    std::sync::RwLock<Vec<String>>,
> = std::sync::OnceLock::new();

pub fn record_nullable_lint(site: String) {
    let hits = PY_NULLABLE_LINT_HITS
        .get_or_init(|| std::sync::RwLock::new(Vec::new()));
    if let Ok(mut v) = hits.write() {
        // 同名站点去重（W 级提示，一次即可）。
        if !v.iter().any(|s| s == &site) {
            log::warn!(
                "W0010 py-nullable-unguarded: {} 的返回注解为 `T | None`，未判空直接使用（null 是值）",
                site
            );
            v.push(site);
        }
    }
}

/// 测试通道：清空两个表（仅 cfg(test) 使用）。
#[cfg(test)]
pub fn reset_oracle_tables_for_test() {
    if let Ok(mut m) = annotations_table().write() {
        m.clear();
    }
    if let Some(hits) = PY_NULLABLE_LINT_HITS.get() {
        if let Ok(mut v) = hits.write() {
            v.clear();
        }
    }
}

/// Describes the signature of a Python function for FFI marshalling.
#[derive(Debug, Clone)]
pub struct PySignature {
    pub params: Vec<PyType>,
    pub returns: PyType,
}

impl PySignature {
    pub fn new() -> Self {
        Self {
            params: Vec::new(),
            returns: PyType::Auto,
        }
    }

    pub fn param(mut self, t: PyType) -> Self {
        self.params.push(t);
        self
    }

    pub fn returns(mut self, t: PyType) -> Self {
        self.returns = t;
        self
    }

    /// All-auto signature: runtime NanoValue tag detection for all params and return.
    /// Each param's type is auto-detected from the VM stack's NanoValue tag at call time.
    /// Return type uses `py_auto_marshal_return` for dynamic Python→VM conversion.
    /// Plan 300: Replaces hardcoded `default_string_string()` as the default registration.
    pub fn all_auto(param_count: usize) -> Self {
        Self {
            params: vec![PyType::Auto; param_count],
            returns: PyType::Auto,
        }
    }

    /// Default string→string signature (backward compat with Plan 214)
    pub fn default_string_string() -> Self {
        Self::new().param(PyType::String).returns(PyType::String)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plan 567 T19: nullable lint 收集与同名站点去重。
    #[test]
    fn test_nullable_lint_recording_dedup() {
        crate::py_ffi_types::reset_oracle_tables_for_test();
        record_nullable_lint("var = get(...)".to_string());
        record_nullable_lint("var = get(...)".to_string());
        record_nullable_lint("var = find(...)".to_string());
        if let Some(hits) = PY_NULLABLE_LINT_HITS.get() {
            if let Ok(v) = hits.read() {
                assert_eq!(v.len(), 2, "dedup by site: {:?}", v);
            }
        }
    }

    #[test]
    fn test_py_type_equality() {
        assert_eq!(PyType::Int, PyType::Int);
        assert_ne!(PyType::Int, PyType::Float);
    }

    #[test]
    fn test_py_signature_builder() {
        let sig = PySignature::new()
            .param(PyType::Float)
            .returns(PyType::Float);
        assert_eq!(sig.params.len(), 1);
        assert_eq!(sig.params[0], PyType::Float);
        assert_eq!(sig.returns, PyType::Float);
    }

    #[test]
    fn test_default_string_string() {
        let sig = PySignature::default_string_string();
        assert_eq!(sig.params.len(), 1);
        assert_eq!(sig.params[0], PyType::String);
        assert_eq!(sig.returns, PyType::String);
    }

    #[test]
    fn test_multi_param_signature() {
        let sig = PySignature::new()
            .param(PyType::String)
            .param(PyType::Int)
            .returns(PyType::Auto);
        assert_eq!(sig.params.len(), 2);
        assert_eq!(sig.returns, PyType::Auto);
    }

    #[test]
    fn test_all_auto_signature() {
        let sig = PySignature::all_auto(2);
        assert_eq!(sig.params.len(), 2);
        assert_eq!(sig.params[0], PyType::Auto);
        assert_eq!(sig.params[1], PyType::Auto);
        assert_eq!(sig.returns, PyType::Auto);
    }

    #[test]
    fn test_all_auto_zero_params() {
        let sig = PySignature::all_auto(0);
        assert!(sig.params.is_empty());
        assert_eq!(sig.returns, PyType::Auto);
    }
}
