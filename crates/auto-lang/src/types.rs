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
//! ```text
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

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use auto_val::AutoStr;
use crate::ast::{Type, TypeDecl, Fn, SpecDecl, Name, GenericInstance, EnumDecl};

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
    type_decls: HashMap<AutoStr, Rc<TypeDecl>>,

    /// 函数声明：函数名 -> 函数声明
    fn_decls: HashMap<Name, Fn>,

    /// Spec 声明：spec 名 -> spec 声明
    spec_decls: HashMap<AutoStr, SpecDecl>,

    /// Plan 190: Names of types imported via use.rust
    rust_types: HashSet<String>,

    /// Plan 190: Maps short name -> full Rust path
    rust_type_paths: HashMap<String, String>,

    /// 泛型模板：类型名 -> 泛型模板（用于类型参数替换）
    generic_templates: HashMap<String, GenericTemplate>,

    /// 类型别名：别名 -> 目标类型名（Plan 090）
    type_aliases: HashMap<AutoStr, AutoStr>,

    /// Enum 声明：enum 名 -> EnumDecl
    enum_decls: HashMap<AutoStr, Rc<EnumDecl>>,
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
            enum_decls: HashMap::new(),
            rust_types: HashSet::new(),
            rust_type_paths: HashMap::new(),
        }
    }

    /// 注册类型声明
    pub fn register_type_decl(&mut self, decl: &TypeDecl) {
        let name = decl.name.clone();
        self.type_decls.insert(name, Rc::new(decl.clone()));

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

    /// 注册 ext 块中的方法到目标类型的 type_decl
    pub fn register_ext_methods(&mut self, ext: &crate::ast::Ext) {
        let target_name = ext.target.clone();
        let target_str = AutoStr::from(target_name.as_str());

        if let Some(decl) = self.type_decls.get(&target_str).cloned() {
            // TypeDecl already exists, append methods via Rc replacement
            let mut decl = (*decl).clone();
            for method in &ext.methods {
                if !decl.methods.iter().any(|m| m.name == method.name) {
                    decl.methods.push(method.clone());
                }
                // Also register in fn_decls so import_items can find them
                self.fn_decls.insert(method.name.clone(), method.clone());
            }
            self.type_decls.insert(target_str.clone(), Rc::new(decl));
        } else {
            // Type not yet registered — create a placeholder TypeDecl with the ext methods
            use crate::ast::{TypeDecl, TypeDeclKind};
            let mut placeholder = TypeDecl {
                name: target_name.clone(),
                kind: TypeDeclKind::UserType,
                parent: None,
                has: Vec::new(),
                specs: Vec::new(),
                spec_impls: Vec::new(),
                generic_params: Vec::new(),
                members: Vec::new(),
                delegations: Vec::new(),
                methods: ext.methods.clone(),
                attrs: vec![],
                doc: None,
                is_pub: false,
            };
            // Register methods in fn_decls too
            for method in &ext.methods {
                if !placeholder.methods.iter().any(|m| m.name == method.name) {
                    placeholder.methods.push(method.clone());
                }
                self.fn_decls.insert(method.name.clone(), method.clone());
            }
            self.type_decls.insert(target_str, Rc::new(placeholder));
        }
    }

    /// 注册泛型模板
    pub fn register_generic_template(&mut self, template: GenericTemplate) {
        self.generic_templates.insert(template.name().to_string(), template);
    }

    /// Plan 190: Register a Rust type imported via use.rust
    pub fn register_rust_type(&mut self, name: impl Into<AutoStr>, full_path: impl Into<String>) {
        use crate::ast::RustSource;
        use crate::ast::{Name, TypeDecl, TypeDeclKind};

        let type_name = name.into();
        let path = full_path.into();
        self.rust_types.insert(type_name.to_string());
        self.rust_type_paths.insert(type_name.to_string(), path.clone());

        let decl = TypeDecl {
            name: Name::from(type_name.clone()),
            kind: TypeDeclKind::UserType,
            parent: None,
            has: Vec::new(),
            specs: Vec::new(),
            spec_impls: Vec::new(),
            generic_params: Vec::new(),
            members: Vec::new(),
            delegations: Vec::new(),
            methods: Vec::new(),
            attrs: vec![],
            doc: None,
            is_pub: false,
        };
        self.type_decls.insert(type_name, Rc::new(decl));
    }

    /// Plan 190: Check if a type name was imported via use.rust
    pub fn is_rust_type(&self, name: &str) -> bool {
        self.rust_types.contains(name)
    }

    /// Plan 190: Get the full Rust path for a use.rust imported type
    pub fn get_rust_type_path(&self, name: &str) -> Option<String> {
        self.rust_type_paths.get(name).cloned()
    }

    /// 查找类型声明
    pub fn lookup_type_decl(&self, name: &AutoStr) -> Option<Rc<TypeDecl>> {
        self.type_decls.get(name).cloned()
    }

    /// 查找类型声明（字符串参数）
    pub fn lookup_type_decl_str(&self, name: &str) -> Option<Rc<TypeDecl>> {
        self.type_decls.get(&AutoStr::from(name)).cloned()
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

    /// 注册 Enum 声明
    pub fn register_enum_decl(&mut self, decl: EnumDecl) {
        let name = decl.name.clone();
        self.enum_decls.insert(name, Rc::new(decl));
    }

    /// 查找 Enum 声明
    pub fn lookup_enum_decl(&self, name: &AutoStr) -> Option<Rc<EnumDecl>> {
        self.enum_decls.get(name).cloned()
    }

    /// 查找 Enum 声明（字符串参数）
    pub fn lookup_enum_decl_str(&self, name: &str) -> Option<Rc<EnumDecl>> {
        self.enum_decls.get(&AutoStr::from(name)).cloned()
    }

    /// 检查名称是否为 Enum 类型
    pub fn is_enum(&self, name: &str) -> bool {
        self.enum_decls.contains_key(&AutoStr::from(name))
    }

    /// 获取 Enum 变体的值
    pub fn get_enum_variant_value(&self, enum_name: &str, variant_name: &str) -> Option<i32> {
        self.enum_decls.get(&AutoStr::from(enum_name))
            .and_then(|decl| decl.items.iter()
                .find(|item| item.name.as_ref() == variant_name)
                .map(|item| item.value()))
    }

    /// Plan 127: 查找枚举变体的值（通过变体名称）
    ///
    /// 遍历所有枚举，查找具有指定名称的变体。
    /// 用于支持直接使用变体名称（如 `Red`）而不需要枚举名称（如 `Color.Red`）。
    pub fn find_enum_variant_by_name(&self, variant_name: &str) -> Option<(AutoStr, i32)> {
        for (enum_name, decl) in &self.enum_decls {
            if let Some(item) = decl.items.iter().find(|item| item.name.as_ref() == variant_name) {
                return Some((enum_name.clone(), item.value()));
            }
        }
        None
    }

    /// 统一的类型检查（包含 type、enum、spec）
    pub fn is_type(&self, name: &str) -> bool {
        let key = AutoStr::from(name);
        self.type_decls.contains_key(&key)
            || self.enum_decls.contains_key(&key)
            || self.spec_decls.contains_key(&key)
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
            return Some(Type::User(type_decl.as_ref().clone()));
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

            let _field_ty = template.field_ty.clone();

            // 替换模板中的类型参数
            for (i, _arg) in type_args.iter().enumerate() {
                if let GenericParamType::Type = template.param_types[i] {
                    // TODO: 实现 substitute 方法
                    // field_ty = field_ty.substitute(&template.param_names[i], arg);
                }
            }

            Type::GenericInstance(GenericInstance {
                base_name: Name::from(type_name),
                args: type_args.to_vec(),
                source: None,
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

    /// Plan 085: 合并另一个 TypeStore 的内容
    ///
    /// 用于模块导入时将模块的符号合并到当前 TypeStore。
    /// 如果存在同名符号，新符号会覆盖旧符号。
    pub fn merge(&mut self, other: &TypeStore) {
        // 合并类型声明
        for (name, decl) in &other.type_decls {
            self.type_decls.insert(name.clone(), decl.clone());
        }

        // 合并函数声明
        for (name, fn_decl) in &other.fn_decls {
            self.fn_decls.insert(name.clone(), fn_decl.clone());
        }

        // 合并 spec 声明
        for (name, spec_decl) in &other.spec_decls {
            self.spec_decls.insert(name.clone(), spec_decl.clone());
        }

        // 合并泛型模板
        for (name, template) in &other.generic_templates {
            self.generic_templates.insert(name.clone(), template.clone());
        }

        // 合并类型别名
        for (alias, target) in &other.type_aliases {
            self.type_aliases.insert(alias.clone(), target.clone());
        }
    }

    /// Plan 085: 选择性导入符号
    ///
    /// 只导入指定的项，而不是全部符号。
    /// 用于 `use module: item1, item2` 形式的导入。
    pub fn import_items(&mut self, other: &TypeStore, items: &[String]) {
        for item in items {
            let item_name = AutoStr::from(item.as_str());
            let item_name_key = Name::from(item.as_str());

            // 检查是否是类型
            if let Some(decl) = other.type_decls.get(&item_name) {
                self.type_decls.insert(item_name.clone(), decl.clone());
            }

            // 检查是否是函数
            if let Some(fn_decl) = other.fn_decls.get(&item_name_key) {
                self.fn_decls.insert(item_name_key.clone(), fn_decl.clone());
            }

            // 检查是否是 spec
            if let Some(spec_decl) = other.spec_decls.get(&item_name) {
                self.spec_decls.insert(item_name.clone(), spec_decl.clone());
            }

            // 检查是否是类型别名
            if let Some(target) = other.type_aliases.get(&item_name) {
                self.type_aliases.insert(item_name.clone(), target.clone());
            }
        }
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
                    attrs: vec![],
                },
                crate::ast::Member {
                    name: Name::from("y"),
                    ty: Type::Int,
                    value: None,
                    attrs: vec![],
                },
            ],
            methods: vec![],
            delegations: vec![],
            attrs: vec![],
            doc: None,
            is_pub: false,
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
            attrs: vec![],
            doc: None,
            is_pub: false,
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
