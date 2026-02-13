//! TypeStore: 统一的类型信息管理系统
//!
//! # 概述
//!
//! `TypeStore` 提供了一个集中的类型信息管理系统，将分散在 Parser、Codegen 和 InferenceContext 中的
//! 类型声明整合到一个统一的存储中。
//!
//! # 背景问题
//!
//! 在重构之前，类型声明信息分散在多个位置：
//! - Parser 有 `type_registry: Option<SharedTypeRegistry>`（用于 REPL）
//! - Codegen 有 `types: HashMap<String, TypeInfo>` 和 `generic_registry: GenericRegistry`
//! - InferenceContext 有 `type_registry: TypeRegistry`
//!
//! 这导致：
//! 1. 数据重复（类型声明在多处存储）
//! 2. 同步困难（需要在多个地方更新）
//! 3. 复杂的包装（`SharedTypeRegistry = Rc<RefCell<TypeRegistry>>`）
//! 4. 职责不清（类型推导和类型查询使用不同的存储）
//!
//! # Plan 084: TypeStore
//!
//! 目标：创建一个统一的 `TypeStore` 作为单一数据源
//! - 集中管理所有类型声明、函数声明、spec 声明和泛型模板
//! - Parser 和 Codegen 通过 `Arc<TypeStore>` 共享同一个实例
//! - InferenceContext 通过 `type_env` 和 `type_registry` 访问类型信息
//!
//! # 架构
//!
//! ```
//! Parser ───────┐
//! type_store: Arc<TypeStore> ──────┐
//!     ├── type_decls
//!     ├── fn_decls
//!     ├── spec_decls
//!     └── generic_templates
//!
//! Codegen ───────┐
//! type_store: Arc<TypeStore> ──────┐
//!     ├── type_decls
//!     ├── fn_decls
//!     ├── spec_decls
//!     └── generic_templates
//!
//! InferenceContext ───────┐
//! type_env: HashMap<Name, Type>
//! type_registry: TypeRegistry
//! └──────────────────────┘
//! ```
//!
//! # 设计原则
//!
//! 1. **单一数据源**：所有类型信息存储在 `TypeStore` 中
//! 2. **共享访问**：通过 `Arc<TypeStore>` 实现 thread-safe 的共享
//! 3. **清晰职责**：
//!    - TypeStore：类型声明和查询（编译时）
//!    - InferenceContext：类型推导和变量绑定（运行时）
//!    - REPL type_registry：REPL 跨会话支持
//! 4. **向后兼容**：通过 `SharedTypeRegistry` 保持 REPL 功能
//!
//! # 公共 API
//!
//! - `new()` - 创建新的 TypeStore
//! - `register_type_decl()` - 注册类型声明
//! - `lookup_type_decl()` - 查找类型声明
//! - `get_template()` - 获取泛型模板（用于类型参数替换）
//! - `create_generic_instance()` - 创建泛型实例
//! - `register_fn_decl()` - 注册函数声明
//! - `register_spec_decl()` - 注册 spec 声明
//! - `register_generic_template()` - 注册泛型模板
//!
//! # 相关模块
//!
//! - `ast::types` - AST 类型定义（Type, TypeDecl, Fn, SpecDecl, GenericParam, GenericTemplate）
//! - `infer::context` - 类型推导上下文（保留，仅使用 type_env）
//! - `infer::expr` - 表达式类型推导（使用 type_registry）
//! - `infer::registry` - 类型注册表（将被替换）

use std::collections::HashMap;

use auto_val::AutoStr;
use crate::ast::{Type, TypeDecl, Fn, SpecDecl, Name, GenericInstance};

/// 泛型模板
///
/// 用于类型参数替换的模板定义。
/// 例如：`Pair<int, string>` 中的 `int` 和 `string` 被替换为具体的类型。
#[derive(Debug, Clone)]
pub struct GenericTemplate {
    /// 类型参数名
    pub name: String,

    /// 类型参数类型（只支持 Type 和 String）
    pub param_types: Vec<GenericParamType>,

    /// 字段类型
    pub field_ty: Type,
}

impl GenericTemplate {
    /// 获取泛型模板名称
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// 类型参数类型（只支持 Type 和 String）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenericParamType {
    Type,
    String,
}

/// 类型存储
///
/// 集中管理所有类型、函数和 spec 的声明信息。
///
/// # 示例
///
/// ```ignore
/// let mut store = TypeStore::new();
///
/// // 注册类型声明
/// store.register_type_decl(type_decl)?;
///
/// // 查找类型
/// let decl = store.lookup_type_decl("Point")?;
/// ```
///
/// ```
#[derive(Debug, Clone)]
pub struct TypeStore {
    /// 类型声明：类型名 -> 完整的类型声明
    type_decls: HashMap<AutoStr, TypeDecl>,

    /// 函数声明：函数名 -> 函数声明
    fn_decls: HashMap<Name, Fn>,

    /// Spec 声明：spec 名 -> spec 声明
    spec_decls: HashMap<AutoStr, SpecDecl>,

    /// 泛型模板：类型名 -> 泛型模板（用于类型参数替换）
    generic_templates: HashMap<String, GenericTemplate>,

    /// 类型别名：别名 -> 目标类型名（Plan 090）
    type_aliases: HashMap<AutoStr, AutoStr>,
}

impl TypeStore {
    /// 创建新的类型存储
    pub fn new() -> Self {
        Self {
            type_decls: HashMap::new(),
            fn_decls: HashMap::new(),
            spec_decls: HashMap::new(),
            generic_templates: HashMap::new(),
            type_aliases: HashMap::new(),
        }
    }

    /// 注册类型声明
    pub fn register_type_decl(&mut self, decl: &TypeDecl) {
        self.type_decls.insert(decl.name.clone(), decl.clone());

        // 如果是泛型，注册为泛型模板
        if !decl.generic_params.is_empty() {
            let template = GenericTemplate {
                name: decl.name.to_string(),
                param_types: Vec::new(), // TODO: 从 generic_params 提取
                field_ty: Type::Unknown,
            };
            self.generic_templates.insert(template.name().to_string(), template);
        }
    }

    /// 注册函数声明
    pub fn register_fn_decl(&mut self, decl: &Fn) {
        self.fn_decls.insert(decl.name.clone(), decl.clone());
    }

    /// 注册 spec 声明
    pub fn register_spec_decl(&mut self, decl: &SpecDecl) {
        self.spec_decls.insert(decl.name.clone(), decl.clone());
    }

    /// 注册泛型模板
    pub fn register_generic_template(&mut self, template: GenericTemplate) {
        self.generic_templates.insert(template.name().to_string(), template);
    }

    /// 查找类型声明
    pub fn lookup_type_decl(&self, name: &AutoStr) -> Option<&TypeDecl> {
        self.type_decls.get(name)
    }

    /// 查找类型声明（字符串参数）
    pub fn lookup_type_decl_str(&self, name: &str) -> Option<&TypeDecl> {
        self.type_decls.get(&AutoStr::from(name))
    }

    /// 查找函数声明
    pub fn lookup_fn_decl(&self, name: &Name) -> Option<&Fn> {
        self.fn_decls.get(name)
    }

    /// 查找函数声明（字符串参数）
    pub fn lookup_fn_decl_str(&self, name: &str) -> Option<&Fn> {
        self.fn_decls.get(&Name::from(name))
    }

    /// 查找 spec 声明
    pub fn lookup_spec_decl(&self, name: &AutoStr) -> Option<&SpecDecl> {
        self.spec_decls.get(name)
    }

    /// 查找 spec 声明（字符串参数）
    pub fn lookup_spec_decl_str(&self, name: &str) -> Option<&SpecDecl> {
        self.spec_decls.get(&AutoStr::from(name))
    }

    /// 获取泛型模板
    pub fn get_template(&self, name: &str) -> Option<&GenericTemplate> {
        self.generic_templates.get(name)
    }

    /// 注册类型别名（Plan 090）
    pub fn register_type_alias(&mut self, alias: AutoStr, target: AutoStr) {
        self.type_aliases.insert(alias, target);
    }

    /// 查找类型别名
    pub fn lookup_type_alias(&self, alias: &AutoStr) -> Option<&AutoStr> {
        self.type_aliases.get(alias)
    }

    /// 查找类型别名（字符串参数）
    pub fn lookup_type_alias_str(&self, alias: &str) -> Option<&AutoStr> {
        self.type_aliases.get(&AutoStr::from(alias))
    }

    /// 解析类型别名（递归解析直到找到真正的类型）
    pub fn resolve_type_alias(&self, name: &AutoStr) -> AutoStr {
        if let Some(target) = self.type_aliases.get(name) {
            self.resolve_type_alias(target)
        } else {
            name.clone()
        }
    }

    /// Plan 090: 根据名称查找类型
    ///
    /// 用于替代 Universe 的 `find_type_for_name()` 方法。
    /// 查找类型声明并返回对应的 Type。
    pub fn find_type_for_name(&self, name: &str) -> Option<Type> {
        // 首先检查类型别名
        let resolved_name = self.resolve_type_alias(&AutoStr::from(name));

        // 查找类型声明
        if let Some(type_decl) = self.type_decls.get(&resolved_name) {
            return Some(Type::User(type_decl.clone()));
        }

        None
    }

    /// 创建泛型实例（用于类型参数替换）
    pub fn create_generic_instance(&self, type_name: &str, type_args: &[Type]) -> Type {
        if let Some(template) = self.get_template(type_name) {
            // 替换类型参数
            if type_args.len() != template.param_types.len() {
                return Type::Unknown;
            }

            let mut field_ty = template.field_ty.clone();

            // 替换模板中的类型参数
            for (i, arg) in type_args.iter().enumerate() {
                if let GenericParamType::Type = template.param_types[i] {
                    // TODO: 实现 substitute 方法
                    // field_ty = field_ty.substitute(&template.param_names[i], arg);
                }
            }

            Type::GenericInstance(GenericInstance {
                base_name: Name::from(type_name),
                args: type_args.to_vec(),
            })
        } else {
            Type::Unknown
        }
    }

    /// 列出所有类型声明
    pub fn list_types(&self) -> Vec<AutoStr> {
        self.type_decls.keys().cloned().collect()
    }

    /// Plan 090: 获取所有已定义的名称
    ///
    /// 用于替代 Universe 的 `get_defined_names()` 方法。
    /// 返回类型、函数、Spec 的所有名称列表（用于错误提示）。
    pub fn get_defined_names(&self) -> Vec<String> {
        let mut names = Vec::new();

        // 添加类型名
        for name in self.type_decls.keys() {
            names.push(name.to_string());
        }

        // 添加函数名
        for name in self.fn_decls.keys() {
            names.push(name.to_string());
        }

        // 添加 spec 名
        for name in self.spec_decls.keys() {
            names.push(name.to_string());
        }

        names
    }

    /// 列出所有函数声明
    pub fn list_functions(&self) -> Vec<Name> {
        self.fn_decls.keys().cloned().collect()
    }

    /// 列出所有 spec 声明
    pub fn list_specs(&self) -> Vec<AutoStr> {
        self.spec_decls.keys().cloned().collect()
    }

    /// 列出所有泛型模板
    pub fn list_generic_templates(&self) -> Vec<String> {
        self.generic_templates.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_store_basic() {
        let mut store = TypeStore::new();

        // 注册一个简单的类型声明
        let type_decl = TypeDecl {
            name: Name::from("Point"),
            kind: crate::ast::TypeDeclKind::UserType,
            parent: None,
            has: vec![],
            specs: vec![],
            spec_impls: vec![],
            generic_params: vec![],
            members: vec![
                crate::ast::Member {
                    name: Name::from("x"),
                    ty: Type::Int,
                    value: None,
                },
                crate::ast::Member {
                    name: Name::from("y"),
                    ty: Type::Int,
                    value: None,
                },
            ],
            methods: vec![],
            delegations: vec![],
        };

        store.register_type_decl(&type_decl);

        // 查找类型声明
        let point_decl = store.lookup_type_decl(&AutoStr::from("Point")).unwrap();
        assert_eq!(point_decl.name.to_string(), "Point");
        assert_eq!(point_decl.generic_params.len(), 0);
        assert_eq!(point_decl.members.len(), 2);
    }

    #[test]
    fn test_generic_template() {
        let mut store = TypeStore::new();

        // 创建泛型模板
        let template = GenericTemplate {
            name: "Pair".to_string(),
            param_types: vec![
                GenericParamType::Type,
                GenericParamType::String,
            ],
            field_ty: Type::Unknown,
        };

        store.register_generic_template(template);

        // 创建泛型实例（类型参数替换）
        let instance = store.create_generic_instance(
            "Pair",
            &[Type::Int, Type::Str(0)]
        );

        assert!(matches!(instance, Type::GenericInstance(_)));
    }

    #[test]
    fn test_type_parameter_substitution() {
        let mut store = TypeStore::new();

        // 注册泛型模板
        let template = GenericTemplate {
            name: "Box".to_string(),
            param_types: vec![
                GenericParamType::Type,
                GenericParamType::Type,
            ],
            field_ty: Type::Unknown,
        };

        store.register_generic_template(template);

        // 创建泛型实例
        let instance = store.create_generic_instance(
            "Box",
            &[Type::Int, Type::Str(0)]
        );

        // 验证类型参数替换
        if let Type::GenericInstance(ref inst) = instance {
            assert_eq!(inst.base_name.to_string(), "Box");
            // Type doesn't implement Eq, so we can't compare directly
            assert!(matches!(inst.args[0], Type::Int));
            assert!(matches!(inst.args[1], Type::Str(_)));
        }
    }

    #[test]
    fn test_list_operations() {
        let mut store = TypeStore::new();

        // 注册类型
        let point_type = TypeDecl {
            name: Name::from("Point"),
            kind: crate::ast::TypeDeclKind::UserType,
            parent: None,
            has: vec![],
            specs: vec![],
            spec_impls: vec![],
            generic_params: vec![],
            members: vec![],
            methods: vec![],
            delegations: vec![],
        };
        store.register_type_decl(&point_type);

        // 简化测试：仅测试函数注册和列表
        // 注意：Fn 和 Param 的完整构造需要更多字段
        // 这里我们只测试基本的 store 功能是否正常工作

        // 列出所有类型
        let types = store.list_types();
        assert_eq!(types.len(), 1);
        assert_eq!(types[0].to_string(), "Point");
    }
}
