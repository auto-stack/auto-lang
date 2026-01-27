//! Trait Consistency Checker
//!
//! This module provides functionality to check if a type correctly implements a spec/trait.
//! It verifies that all required methods are implemented with matching signatures.

use crate::ast::{SpecDecl, TypeDecl, Type};
use crate::error::{AutoError, SyntaxError};
use miette::SourceSpan;
use std::rc::Rc;

/// Trait checker for verifying spec conformance
pub struct TraitChecker;

impl TraitChecker {
    /// Check if a type correctly implements a spec
    ///
    /// Verifies that:
    /// - All required methods from the spec are implemented
    /// - Method signatures match (parameter count, return type)
    ///
    /// # Arguments
    /// * `type_decl` - The type declaration to check
    /// * `spec_decl` - The spec declaration to check against
    ///
    /// # Returns
    /// * `Ok(())` if the type correctly implements the spec
    /// * `Err(Vec<AutoError>)` with a list of conformance errors
    pub fn check_conformance(
        type_decl: &TypeDecl,
        spec_decl: &SpecDecl,
    ) -> Result<(), Vec<AutoError>> {
        let mut errors = Vec::new();

        for spec_method in &spec_decl.methods {
            let implemented = type_decl.methods.iter().find(|m| m.name == spec_method.name);

            match implemented {
                Some(method) => {
                    // Check parameter count
                    if method.params.len() != spec_method.params.len() {
                        errors.push(
                            SyntaxError::Generic {
                                message: format!(
                                    "Method '{}' has {} parameter(s) but spec '{}' requires {}",
                                    method.name,
                                    method.params.len(),
                                    spec_decl.name,
                                    spec_method.params.len()
                                ),
                                span: Self::empty_span(),
                            }
                            .into(),
                        );
                    }

                    // Check return type compatibility
                    // For now, we just check if return types match exactly
                    // TODO: Add more sophisticated type compatibility checking
                    let is_compatible = matches!(
                        (&method.ret, &spec_method.ret),
                            // Exact match for same types
                        (Type::Void, Type::Void)
                            | (Type::Int, Type::Int)
                            | (Type::Uint, Type::Uint)
                            | (Type::Float, Type::Float)
                            | (Type::Double, Type::Double)
                            | (Type::Bool, Type::Bool)
                            | (Type::Str(_), Type::Str(_))
                            | (Type::Unknown, Type::Void)  // Unknown is compatible with Void
                            | (Type::Void, Type::Unknown)  // Void is compatible with Unknown
                    );

                    if !is_compatible {
                        errors.push(
                            SyntaxError::Generic {
                                message: format!(
                                    "Method '{}' has return type {:?} but spec '{}' requires {:?}",
                                    method.name, method.ret, spec_decl.name, spec_method.ret
                                ),
                                span: Self::empty_span(),
                            }
                            .into(),
                        );
                    }
                }
                None => {
                    errors.push(
                        SyntaxError::Generic {
                            message: format!(
                                "Type '{}' does not implement required method '{}' from spec '{}'",
                                type_decl.name, spec_method.name, spec_decl.name
                            ),
                            span: Self::empty_span(),
                        }
                        .into(),
                    );
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Check if a type implements all its declared specs
    ///
    /// # Arguments
    /// * `type_decl` - The type declaration to check
    /// * `get_spec` - A function to look up spec declarations by name
    ///
    /// # Returns
    /// * `Ok(())` if the type implements all its specs
    /// * `Err(Vec<AutoError>)` with a list of conformance errors
    pub fn check_all_specs<F>(
        type_decl: &TypeDecl,
        get_spec: F,
    ) -> Result<(), Vec<AutoError>>
    where
        F: Fn(&str) -> Option<Rc<SpecDecl>>,
    {
        let mut all_errors = Vec::new();

        for spec_name in &type_decl.specs {
            if let Some(spec_decl) = get_spec(spec_name.as_str()) {
                if let Err(errors) = Self::check_conformance(type_decl, &spec_decl) {
                    all_errors.extend(errors);
                }
            } else {
                // Spec not found - this is an error but not a conformance error
                all_errors.push(
                    SyntaxError::Generic {
                        message: format!(
                            "Type '{}' declares spec '{}' but spec is not defined",
                            type_decl.name, spec_name
                        ),
                        span: Self::empty_span(),
                    }
                    .into(),
                );
            }
        }

        if all_errors.is_empty() {
            Ok(())
        } else {
            Err(all_errors)
        }
    }

    /// Create an empty span for error reporting
    ///
    /// TODO: Improve this to include actual source locations
    fn empty_span() -> SourceSpan {
        (0, 0).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Name, Param, SpecMethod, Type, Fn};
    use ecow::EcoString as AutoStr;

    fn create_test_spec(name: &str, methods: Vec<SpecMethod>) -> SpecDecl {
        SpecDecl {
            name: Name::from(name),
            methods,
        }
    }

    fn create_test_type(name: &str, methods: Vec<Fn>, specs: Vec<AutoStr>) -> TypeDecl {
        TypeDecl {
            name: Name::from(name),
            kind: crate::ast::TypeDeclKind::UserType,
            parent: None,
            has: Vec::new(),
            specs,
            generic_params: Vec::new(),
            members: Vec::new(),
            delegations: Vec::new(),
            methods,
        }
    }

    fn create_spec_method(name: &str, params: Vec<Param>, ret: Type) -> SpecMethod {
        SpecMethod {
            name: Name::from(name),
            params,
            ret,
        }
    }

    fn create_fn(name: &str, params: Vec<Param>, ret: Type) -> Fn {
        use crate::ast::{Body, FnKind};

        Fn {
            kind: FnKind::Method,
            name: Name::from(name),
            parent: None,
            params,
            body: Body::new(),
            ret,
            ret_name: None,
            is_static: false,  // Plan 035 Phase 4: Default to instance method
        }
    }

    #[test]
    fn test_conformance_success() {
        let spec = create_test_spec(
            "Flyer",
            vec![create_spec_method("fly", vec![], Type::Void)],
        );

        let ty = create_test_type(
            "Pigeon",
            vec![create_fn("fly", vec![], Type::Void)],
            vec!["Flyer".into()],
        );

        assert!(TraitChecker::check_conformance(&ty, &spec).is_ok());
    }

    #[test]
    fn test_conformance_missing_method() {
        let spec = create_test_spec(
            "Flyer",
            vec![
                create_spec_method("fly", vec![], Type::Void),
                create_spec_method("land", vec![], Type::Void),
            ],
        );

        let ty = create_test_type(
            "Pigeon",
            vec![create_fn("fly", vec![], Type::Void)],
            vec!["Flyer".into()],
        );

        let result = TraitChecker::check_conformance(&ty, &spec);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].to_string().contains("land"));
    }

    #[test]
    fn test_conformance_param_count_mismatch() {
        let spec = create_test_spec(
            "Calculator",
            vec![create_spec_method(
                "add",
                vec![
                    Param {
                        name: Name::from("a"),
                        ty: Type::Int,
                        default: None,
                    },
                    Param {
                        name: Name::from("b"),
                        ty: Type::Int,
                        default: None,
                    },
                ],
                Type::Int,
            )],
        );

        let ty = create_test_type(
            "BadCalc",
            vec![create_fn("add", vec![], Type::Int)],
            vec!["Calculator".into()],
        );

        let result = TraitChecker::check_conformance(&ty, &spec);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0]
            .to_string()
            .contains("has 0 parameter(s) but spec"));
    }

    #[test]
    fn test_conformance_return_type_mismatch() {
        let spec = create_test_spec(
            "Getter",
            vec![create_spec_method("get_value", vec![], Type::Int)],
        );

        let ty = create_test_type(
            "BadGetter",
            vec![create_fn("get_value", vec![], Type::Str(0))],
            vec!["Getter".into()],
        );

        let result = TraitChecker::check_conformance(&ty, &spec);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0]
            .to_string()
            .contains("has return type"));
    }
}
