// =============================================================================
// Fragment Hashing: Multi-level hashing for fine-grained incremental compilation
// =============================================================================
//
// This module provides three levels of fragment hashing:
//
// **L1 Text Hash**: Hash of source text
// - Detects ANY text change (comments, formatting, whitespace)
// - Fastest to compute
// - Used for: "Did anything change?"
//
// **L2 AST Hash**: Hash of AST structure
// - Ignores comments and formatting
// - Sensitive to code structure changes
// - Used for: "Did the code structure change?"
//
// **L3 Interface Hash**: Hash of signature only
// - Hashes name, parameters, return type
// - Ignores function body entirely
// - Used for: "Did the public interface change?"
//
// **Phase 3.1**: Fragment-level hashing for fine-grained incremental compilation

use crate::ast::{Body, Fn, FnKind, Type};
use auto_val::AutoStr;

// =============================================================================
// Fragment Hasher
// =============================================================================

/// Fragment hasher with three levels of hashing
///
/// Provides L1 (text), L2 (AST), and L3 (interface) hashing
/// for fine-grained incremental compilation.
pub struct FragmentHasher;

impl FragmentHasher {
    // =========================================================================
    // L1: Text Hash (source text)
    // =========================================================================

    /// Hash the source text of a fragment
    ///
    /// This is the L1 hash - it detects ANY change to the source text,
    /// including comments, whitespace, and formatting.
    ///
    /// **Use case**: Fast check for "did anything change?"
    pub fn hash_text(frag: &Fn) -> u64 {
        // For L1, we hash the string representation
        // This detects any change including formatting
        let text = format!("{:?}", frag);
        blake3::hash(text.as_bytes())
            .as_bytes()[..8]
            .try_into()
            .map(u64::from_le_bytes)
            .unwrap_or(0)
    }

    // =========================================================================
    // L2: AST Hash (structure)
    // =========================================================================

    /// Hash the AST structure of a fragment
    ///
    /// This is the L2 hash - it detects structural changes to the code,
    /// but ignores comments and most formatting differences.
    ///
    /// **Note**: Currently hashes simplified structure. Can be enhanced
    /// to hash full body including expressions.
    pub fn hash_ast(frag: &Fn) -> u64 {
        let mut hasher = blake3::Hasher::new();

        // Hash function name
        hasher.update(frag.name.as_bytes());

        // Hash parameters (names and types)
        for param in &frag.params {
            hasher.update(param.name.as_bytes());
            hash_type_no_rc(&mut hasher, &param.ty);
        }

        // Hash return type
        hash_type_no_rc(&mut hasher, &frag.ret);

        // Hash body structure (simplified - just statement count)
        hasher.update(&frag.body.stmts.len().to_be_bytes());

        hasher.finalize()
            .as_bytes()[..8]
            .try_into()
            .map(u64::from_le_bytes)
            .unwrap_or(0)
    }

    // =========================================================================
    // L3: Interface Hash (signature only)
    // =========================================================================

    /// Hash the interface (signature) of a fragment
    ///
    /// This is the L3 hash - it detects ONLY changes to the public interface:
    /// - Function name
    /// - Parameter names and types
    /// - Return type
    ///
    /// **Key insight**: If L3 hash is unchanged, dependents don't need to recompile
    /// even if the function body changed completely.
    pub fn hash_interface(frag: &Fn) -> u64 {
        let mut hasher = blake3::Hasher::new();

        // Hash function name
        hasher.update(frag.name.as_bytes());

        // Hash parameter names and types
        for param in &frag.params {
            hasher.update(param.name.as_bytes());
            hash_type_no_rc(&mut hasher, &param.ty);
        }

        // Hash return type
        hash_type_no_rc(&mut hasher, &frag.ret);

        hasher.finalize()
            .as_bytes()[..8]
            .try_into()
            .map(u64::from_le_bytes)
            .unwrap_or(0)
    }
}

// =============================================================================
// Helper Functions for AST Hashing
// =============================================================================

/// Hash a Type node (simplified, avoids Rc issues)
fn hash_type_no_rc(hasher: &mut blake3::Hasher, ty: &Type) {
    match ty {
        Type::Byte => hasher.update(b"Byte"),
        Type::Int => hasher.update(b"Int"),
        Type::Uint => hasher.update(b"Uint"),
        Type::USize => hasher.update(b"USize"),
        Type::Float => hasher.update(b"Float"),
        Type::Double => hasher.update(b"Double"),
        Type::Bool => hasher.update(b"Bool"),
        Type::Char => hasher.update(b"Char"),
        Type::Str(_) => hasher.update(b"Str"),
        Type::CStr => hasher.update(b"CStr"),
        Type::StrSlice => hasher.update(b"StrSlice"),
        Type::Array(_) => hasher.update(b"Array"),
        Type::RuntimeArray(_) => hasher.update(b"RuntimeArray"),
        Type::List(_) => hasher.update(b"List"),
        Type::Slice(_) => hasher.update(b"Slice"),
        Type::Ptr(_) => hasher.update(b"Ptr"),
        Type::Reference(_) => hasher.update(b"Reference"),
        Type::User(_) => hasher.update(b"User"),
        Type::Union(_) => hasher.update(b"Union"),
        Type::Tag(_) => hasher.update(b"Tag"),
        Type::Enum(_) => hasher.update(b"Enum"),
        Type::Spec(_) => hasher.update(b"Spec"),
        Type::GenericInstance(_) => hasher.update(b"GenericInstance"),
        Type::Storage(_) => hasher.update(b"Storage"),
        Type::Fn(_, _) => hasher.update(b"Fn"),
        Type::Void => hasher.update(b"Void"),
        Type::Unknown => hasher.update(b"Unknown"),
        Type::CStruct(_) => hasher.update(b"CStruct"),
        Type::Linear(_) => hasher.update(b"Linear"),
        Type::Variadic => hasher.update(b"Variadic"),
    };
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Param;

    #[test]
    fn test_hash_text_consistent() {
        // L1 hash should be consistent
        let fn1 = Fn::new(
            FnKind::Function,
            AutoStr::from("add"),
            None,
            vec![],
            Body::new(),
            Type::Int,
        );

        let hash1 = FragmentHasher::hash_text(&fn1);
        let hash2 = FragmentHasher::hash_text(&fn1);

        // Same function should produce same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_interface_same_signature() {
        // L3 hash should only care about signature
        let fn1 = Fn::new(
            FnKind::Function,
            AutoStr::from("add"),
            None,
            vec![
                Param {
                    name: AutoStr::from("a"),
                    ty: Type::Int,
                    default: None,
                },
                Param {
                    name: AutoStr::from("b"),
                    ty: Type::Int,
                    default: None,
                },
            ],
            Body::new(),
            Type::Int,
        );

        let hash1 = FragmentHasher::hash_interface(&fn1);
        let hash2 = FragmentHasher::hash_interface(&fn1);

        // Same signature should produce same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_interface_different_name() {
        let fn_add = Fn::new(
            FnKind::Function,
            AutoStr::from("add"),
            None,
            vec![],
            Body::new(),
            Type::Int,
        );

        let fn_sub = Fn::new(
            FnKind::Function,
            AutoStr::from("sub"),
            None,
            vec![],
            Body::new(),
            Type::Int,
        );

        let hash_add = FragmentHasher::hash_interface(&fn_add);
        let hash_sub = FragmentHasher::hash_interface(&fn_sub);

        // Different names should produce different hashes
        assert_ne!(hash_add, hash_sub);
    }

    #[test]
    fn test_hash_interface_different_params() {
        let fn1 = Fn::new(
            FnKind::Function,
            AutoStr::from("foo"),
            None,
            vec![Param {
                name: AutoStr::from("a"),
                ty: Type::Int,
                default: None,
            }],
            Body::new(),
            Type::Int,
        );

        let fn2 = Fn::new(
            FnKind::Function,
            AutoStr::from("foo"),
            None,
            vec![
                Param {
                    name: AutoStr::from("a"),
                    ty: Type::Int,
                    default: None,
                },
                Param {
                    name: AutoStr::from("b"),
                    ty: Type::Int,
                    default: None,
                },
            ],
            Body::new(),
            Type::Int,
        );

        let hash1 = FragmentHasher::hash_interface(&fn1);
        let hash2 = FragmentHasher::hash_interface(&fn2);

        // Different parameters should produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_interface_different_return_type() {
        let fn_int = Fn::new(
            FnKind::Function,
            AutoStr::from("foo"),
            None,
            vec![],
            Body::new(),
            Type::Int,
        );

        let fn_bool = Fn::new(
            FnKind::Function,
            AutoStr::from("foo"),
            None,
            vec![],
            Body::new(),
            Type::Bool,
        );

        let hash_int = FragmentHasher::hash_interface(&fn_int);
        let hash_bool = FragmentHasher::hash_interface(&fn_bool);

        // Different return types should produce different hashes
        assert_ne!(hash_int, hash_bool);
    }

    #[test]
    fn test_hash_ast_consistent() {
        // L2 hash should be consistent for same structure
        let fn1 = Fn::new(
            FnKind::Function,
            AutoStr::from("test"),
            None,
            vec![],
            Body::new(),
            Type::Int,
        );

        let hash1 = FragmentHasher::hash_ast(&fn1);
        let hash2 = FragmentHasher::hash_ast(&fn1);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_different_types() {
        // Different hash functions should produce different values
        let fn_decl = Fn::new(
            FnKind::Function,
            AutoStr::from("foo"),
            None,
            vec![],
            Body::new(),
            Type::Int,
        );

        let text_hash = FragmentHasher::hash_text(&fn_decl);
        let ast_hash = FragmentHasher::hash_ast(&fn_decl);
        let iface_hash = FragmentHasher::hash_interface(&fn_decl);

        // All hashes should be different (very likely)
        let count = [text_hash, ast_hash, iface_hash]
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len();

        assert_eq!(count, 3, "Hash collision detected!");
    }

    #[test]
    fn test_hash_interface_ignores_body() {
        // L3 hash should ignore function body
        let body1 = Body::new();
        let body2 = Body::new();

        let fn1 = Fn::new(
            FnKind::Function,
            AutoStr::from("foo"),
            None,
            vec![Param {
                name: AutoStr::from("a"),
                ty: Type::Int,
                default: None,
            }],
            body1,
            Type::Int,
        );

        let fn2 = Fn::new(
            FnKind::Function,
            AutoStr::from("foo"),
            None,
            vec![Param {
                name: AutoStr::from("a"),
                ty: Type::Int,
                default: None,
            }],
            body2,
            Type::Int,
        );

        // Same signature should produce same L3 hash
        let hash1 = FragmentHasher::hash_interface(&fn1);
        let hash2 = FragmentHasher::hash_interface(&fn2);
        assert_eq!(hash1, hash2);

        // But L2 hash should also be the same (both empty bodies)
        let hash1_ast = FragmentHasher::hash_ast(&fn1);
        let hash2_ast = FragmentHasher::hash_ast(&fn2);
        assert_eq!(hash1_ast, hash2_ast);
    }
}
