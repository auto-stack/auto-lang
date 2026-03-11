//! **DEPRECATED**: This module is deprecated.
//!
//! Use `types::TypeStore` instead, which is the single source of truth
//! for type information. This module will be removed in a future version.
//!
//! ## Migration Guide
//!
//! | Old API | New API |
//! |---------|---------|
//! | `TypeRegistry::register_type_decl(decl)` | `TypeStore::register_type_decl(&decl)` |
//! | `TypeRegistry::lookup_type_decl(name)` | `TypeStore::lookup_type_decl(name)` or `lookup_type_decl_str(name)` |
//! | `TypeRegistry::get_template(name)` | `TypeStore::get_template(name)` or use `vm::generic_registry::GenericRegistry` |
//!
//! ## Context
//!
// Plan 089: Type Registry - Unified Type Declaration Storage
//
// This module provides a centralized type declaration management system
// for the infer module, replacing the scattered storage across codegen and Database.

use crate::ast::TypeDecl;
use crate::vm::generic_registry::{ClassTemplate, FieldDef};
use auto_val::AutoStr;
use std::collections::HashMap;

/// 类型注册表 - 统一管理所有类型声明
///
/// Stores complete type declaration information including fields,
/// replacing the scattered storage in codegen.types, codegen.generic_registry, and Database.type_info_store.
#[derive(Clone)]
pub struct TypeRegistry {
    /// 类型声明映射：类型名 -> TypeDecl
    ///
    /// Stores the complete type declaration with member types,
    /// unlike Database.type_info_store.TypeInfo which only stores method names.
    pub type_decls: HashMap<AutoStr, TypeDecl>,

    /// 泛型模板：类型名 -> ClassTemplate
    ///
    /// Stores generic type templates with field definitions,
    /// compatible with vm/generic_registry format.
    pub generic_templates: HashMap<String, ClassTemplate>,
}

impl TypeRegistry {
    /// 创建新的空类型注册表
    pub fn new() -> Self {
        Self {
            type_decls: HashMap::new(),
            generic_templates: HashMap::new(),
        }
    }

    /// 注册类型声明
    ///
    /// Creates a ClassTemplate from the TypeDecl and stores both the
    /// original declaration and the generic template.
    pub fn register_type_decl(&mut self, type_decl: TypeDecl) {
        let name = type_decl.name.clone();

        // 创建 ClassTemplate（兼容 vm/generic_registry 格式）
        let template = ClassTemplate {
            name: name.as_ref().to_string(),
            generic_params: type_decl.generic_params.clone(),
            fields: type_decl.members.iter().map(|m| FieldDef {
                name: m.name.as_ref().to_string(),
                field_type: m.ty.clone(),
            }).collect(),
            methods: HashMap::new(), // TODO: 处理方法
        };

        self.type_decls.insert(name.clone(), type_decl);
        self.generic_templates.insert(name.as_ref().to_string(), template);
    }

    /// 查找类型声明
    ///
    /// Returns the TypeDecl by name if registered.
    pub fn lookup_type_decl(&self, name: &AutoStr) -> Option<&TypeDecl> {
        self.type_decls.get(name)
    }

    /// 获取泛型模板
    ///
    /// Returns the generic ClassTemplate by name if registered.
    pub fn get_template(&self, name: &AutoStr) -> Option<&ClassTemplate> {
        self.generic_templates.get(name.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Member, Name, Type};

    #[test]
    fn test_type_registry_new() {
        let registry = TypeRegistry::new();
        assert!(registry.type_decls.is_empty());
        assert!(registry.generic_templates.is_empty());
    }

    #[test]
    fn test_type_registry_register_and_lookup() {
        let mut registry = TypeRegistry::new();

        let type_decl = TypeDecl {
            name: Name::from("Point"),
            kind: crate::ast::TypeDeclKind::UserType,
            parent: None,
            has: vec![],
            specs: vec![],
            generic_params: vec![],
            members: vec![
                Member {
                    name: Name::from("x"),
                    ty: Type::Int,
                    value: None,
                },
                Member {
                    name: Name::from("y"),
                    ty: Type::Str(0),
                    value: None,
                },
            ],
            delegations: vec![],
            methods: vec![],
            spec_impls: vec![],
        };

        registry.register_type_decl(type_decl);

        // 查找类型声明
        let found = registry.lookup_type_decl(&AutoStr::from("Point"));
        assert!(found.is_some());

        let found_decl = found.unwrap();
        assert_eq!(found_decl.name, Name::from("Point"));
        assert_eq!(found_decl.members.len(), 2);
        assert!(matches!(found_decl.members[0].ty, Type::Int));
        assert!(matches!(found_decl.members[1].ty, Type::Str(0)));
    }

    #[test]
    fn test_type_registry_get_template() {
        let mut registry = TypeRegistry::new();

        let type_decl = TypeDecl {
            name: Name::from("Pair"),
            kind: crate::ast::TypeDeclKind::UserType,
            parent: None,
            has: vec![],
            specs: vec![],
            generic_params: vec![],
            members: vec![],
            delegations: vec![],
            methods: vec![],
            spec_impls: vec![],
        };

        registry.register_type_decl(type_decl);

        // 查找泛型模板
        let template = registry.get_template(&AutoStr::from("Pair"));
        assert!(template.is_some());

        let found_template = template.unwrap();
        assert_eq!(found_template.name, "Pair");
        assert_eq!(found_template.fields.len(), 0); // Pair 没有字段（泛型）
        assert!(found_template.generic_params.is_empty()); // Pair 没有类型参数
    }
}
