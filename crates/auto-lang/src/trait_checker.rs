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
                    // Plan 057: Unknown (generic params) is compatible with any concrete type
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
                            // Plan 057: Unknown is compatible with any concrete type (generic params)
                            | (Type::Int, Type::Unknown)
                            | (Type::Uint, Type::Unknown)
                            | (Type::Float, Type::Unknown)
                            | (Type::Double, Type::Unknown)
                            | (Type::Bool, Type::Unknown)
                            | (Type::Str(_), Type::Unknown)
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

    /// Plan 057: Check if a type implements all its declared generic specs
    ///
    /// # Arguments
    /// * `type_decl` - The type declaration to check
    /// * `get_spec` - A function to look up spec declarations by name
    ///
    /// # Returns
    /// * `Ok(())` if the type implements all its generic specs correctly
    /// * `Err(Vec<AutoError>)` with a list of conformance errors
    pub fn check_all_spec_impls<F>(
        type_decl: &TypeDecl,
        get_spec: F,
    ) -> Result<(), Vec<AutoError>>
    where
        F: Fn(&str) -> Option<Rc<SpecDecl>>,
    {
        let mut all_errors = Vec::new();

        for spec_impl in &type_decl.spec_impls {
            if let Some(spec_decl) = get_spec(spec_impl.spec_name.as_str()) {
                // Plan 057: Validate type argument count matches generic parameter count
                if spec_impl.type_args.len() != spec_decl.generic_params.len() {
                    all_errors.push(
                        SyntaxError::Generic {
                            message: format!(
                                "Type '{}' implements spec '{}' with {} type argument(s) but spec expects {}",
                                type_decl.name,
                                spec_impl.spec_name,
                                spec_impl.type_args.len(),
                                spec_decl.generic_params.len()
                            ),
                            span: Self::empty_span(),
                        }
                        .into(),
                    );
                    continue; // Skip further checks if type arg count doesn't match
                }

                // Check conformance (TODO: Substitute type parameters in future)
                if let Err(errors) = Self::check_conformance(type_decl, &spec_decl) {
                    all_errors.extend(errors);
                }
            } else {
                // Spec not found - this is an error but not a conformance error
                all_errors.push(
                    SyntaxError::Generic {
                        message: format!(
                            "Type '{}' declares generic spec '{}' but spec is not defined",
                            type_decl.name, spec_impl.spec_name
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
            generic_params: Vec::new(), // Plan 057
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
            spec_impls: Vec::new(), // Plan 057
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

    // Plan 057: Generic spec tests

    #[test]
    fn test_generic_spec_impl_correct_type_args() {
        use crate::ast::{GenericParam, TypeParam, SpecImpl};
        use std::rc::Rc;

        // Create a generic spec with one type parameter
        let spec = SpecDecl {
            name: Name::from("Storage"),
            generic_params: vec![GenericParam::Type(TypeParam {
                name: Name::from("T"),
                constraint: None,
            })],
            methods: vec![create_spec_method("get", vec![], Type::Unknown)],
        };

        // Create a type that implements the spec with correct type args
        let ty = TypeDecl {
            name: Name::from("Heap"),
            kind: crate::ast::TypeDeclKind::UserType,
            parent: None,
            has: Vec::new(),
            specs: Vec::new(),
            spec_impls: vec![SpecImpl {
                spec_name: Name::from("Storage"),
                type_args: vec![Type::Int], // Correct: 1 type arg for 1 param
            }],
            generic_params: Vec::new(),
            members: Vec::new(),
            delegations: Vec::new(),
            methods: vec![create_fn("get", vec![], Type::Int)],
        };

        // Mock spec lookup function
        let get_spec = |name: &str| -> Option<Rc<SpecDecl>> {
            if name == "Storage" {
                Some(Rc::new(spec.clone()))
            } else {
                None
            }
        };

        let result = TraitChecker::check_all_spec_impls(&ty, get_spec);
        if let Err(errors) = &result {
            eprintln!("Errors:");
            for e in errors {
                eprintln!("  {}", e);
            }
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_generic_spec_impl_wrong_type_arg_count() {
        use crate::ast::{GenericParam, TypeParam, SpecImpl};
        use std::rc::Rc;

        // Create a generic spec with two type parameters
        let spec = SpecDecl {
            name: Name::from("Map"),
            generic_params: vec![
                GenericParam::Type(TypeParam {
                    name: Name::from("K"),
                    constraint: None,
                }),
                GenericParam::Type(TypeParam {
                    name: Name::from("V"),
                    constraint: None,
                }),
            ],
            methods: vec![create_spec_method("get", vec![], Type::Unknown)],
        };

        // Create a type that implements the spec with wrong number of type args
        let ty = TypeDecl {
            name: Name::from("HashMap"),
            kind: crate::ast::TypeDeclKind::UserType,
            parent: None,
            has: Vec::new(),
            specs: Vec::new(),
            spec_impls: vec![SpecImpl {
                spec_name: Name::from("Map"),
                type_args: vec![Type::Int], // Wrong: 1 type arg for 2 params
            }],
            generic_params: Vec::new(),
            members: Vec::new(),
            delegations: Vec::new(),
            methods: vec![create_fn("get", vec![], Type::Int)],
        };

        // Mock spec lookup function
        let get_spec = |name: &str| -> Option<Rc<SpecDecl>> {
            if name == "Map" {
                Some(Rc::new(spec.clone()))
            } else {
                None
            }
        };

        let result = TraitChecker::check_all_spec_impls(&ty, get_spec);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0]
            .to_string()
            .contains("with 1 type argument(s) but spec expects 2"));
    }

    #[test]
    fn test_generic_spec_impl_missing_method() {
        use crate::ast::{GenericParam, TypeParam, SpecImpl};
        use std::rc::Rc;

        // Create a generic spec
        let spec = SpecDecl {
            name: Name::from("Storage"),
            generic_params: vec![GenericParam::Type(TypeParam {
                name: Name::from("T"),
                constraint: None,
            })],
            methods: vec![
                create_spec_method("get", vec![], Type::Unknown),
                create_spec_method("set", vec![], Type::Void),
            ],
        };

        // Create a type that implements the spec but is missing a method
        let ty = TypeDecl {
            name: Name::from("Heap"),
            kind: crate::ast::TypeDeclKind::UserType,
            parent: None,
            has: Vec::new(),
            specs: Vec::new(),
            spec_impls: vec![SpecImpl {
                spec_name: Name::from("Storage"),
                type_args: vec![Type::Int],
            }],
            generic_params: Vec::new(),
            members: Vec::new(),
            delegations: Vec::new(),
            methods: vec![create_fn("get", vec![], Type::Int)], // Missing 'set'
        };

        // Mock spec lookup function
        let get_spec = |name: &str| -> Option<Rc<SpecDecl>> {
            if name == "Storage" {
                Some(Rc::new(spec.clone()))
            } else {
                None
            }
        };

        let result = TraitChecker::check_all_spec_impls(&ty, get_spec);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].to_string().contains("does not implement required method"));
    }

    #[test]
    fn test_generic_spec_impl_undefined_spec() {
        use crate::ast::SpecImpl;
        use std::rc::Rc;

        // Create a type that implements a non-existent spec
        let ty = TypeDecl {
            name: Name::from("MyType"),
            kind: crate::ast::TypeDeclKind::UserType,
            parent: None,
            has: Vec::new(),
            specs: Vec::new(),
            spec_impls: vec![SpecImpl {
                spec_name: Name::from("NonExistentSpec"),
                type_args: vec![Type::Int],
            }],
            generic_params: Vec::new(),
            members: Vec::new(),
            delegations: Vec::new(),
            methods: vec![],
        };

        // Mock spec lookup function that returns None
        let get_spec = |_name: &str| -> Option<Rc<SpecDecl>> { None };

        let result = TraitChecker::check_all_spec_impls(&ty, get_spec);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].to_string().contains("but spec is not defined"));
    }
}
