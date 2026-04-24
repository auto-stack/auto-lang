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
    /// Runtime auto-detection of return type (Python is dynamically typed)
    Auto,
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

    /// Default string→string signature (backward compat with Plan 214)
    pub fn default_string_string() -> Self {
        Self::new().param(PyType::String).returns(PyType::String)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
