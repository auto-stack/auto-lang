//! 类型推导上下文和环境管理
//!
//! # 概述
//!
//! `InferenceContext` 负责管理类型推导过程中的所有状态，包括：
//! - 类型环境（变量到类型的映射）
//! - 类型约束集合
//! - 作用域链（支持变量遮蔽）
//! - 当前函数返回类型
//! - 错误和警告收集
//!
//! # 示例
//!
//! ```rust
//! use auto_lang::infer::InferenceContext;
//! use auto_lang::ast::{Type, Name};
//!
//! let mut ctx = InferenceContext::new();
//!
//! // 添加变量绑定
//! let name = Name::from("x");
//! ctx.bind_var(name.clone(), Type::Int);
//!
//! // 查找变量类型
//! let ty = ctx.lookup_type(&name);
//! assert!(matches!(ty, Some(Type::Int)));
//! ```

use crate::ast::{Fn, Name, SpecDecl, Store, StoreKind, Type};
use crate::database::Database;
use crate::error::{AutoError, TypeError, Warning};
use crate::types;  // Plan 084 Phase 4: TypeStore integration
use miette::SourceSpan;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

/// 类型推导上下文
///
/// 维护类型推导过程中的所有状态，包括类型环境、约束、作用域等。
#[derive(Clone)]
pub struct InferenceContext {
    /// 类型环境：变量 -> Type
    pub type_env: HashMap<Name, Type>,

    /// 推导期间收集的约束
    pub constraints: Vec<super::TypeConstraint>,

    /// 用于变量遮蔽的作用域链
    /// 最内层作用域在最后
    pub scopes: Vec<HashMap<Name, Type>>,

    /// 当前函数返回类型（用于检查返回语句）
    pub current_ret: Option<Type>,

    /// Database 引用（用于符号查找）
    /// Phase 070: Migrated from Universe to Database for compile-time data
    pub database: std::sync::Arc<std::sync::RwLock<Database>>,

    /// 错误累加器
    pub errors: Vec<AutoError>,

    /// 警告累加器
    pub warnings: Vec<Warning>,

    /// Plan 084: 统一的 TypeStore
    ///
    /// 使用 RwLock 包装以支持共享可变性。
    /// 始终有值 - 如果独立使用则创建内部 TypeStore，如果与 Parser/Codegen 共享则使用传入的实例。
    /// 通过 write() 进行注册，read() 进行查询。
    ///
    /// 替代了之前的 type_registry, fn_registry, spec_registry。
    pub type_store: Arc<std::sync::RwLock<types::TypeStore>>,
}

impl InferenceContext {
    /// 创建新的类型推导上下文
    ///
    /// Plan 084: 始终创建内部的 TypeStore
    pub fn new() -> Self {
        Self {
            type_env: HashMap::new(),
            constraints: Vec::new(),
            scopes: Vec::new(),
            current_ret: None,
            database: std::sync::Arc::new(std::sync::RwLock::new(Database::new())),
            errors: Vec::new(),
            warnings: Vec::new(),
            type_store: Arc::new(std::sync::RwLock::new(types::TypeStore::new())),
        }
    }

    /// 使用现有的 Database 创建上下文
    ///
    /// Phase 070: Migrated from with_universe to with_database
    /// Plan 084: 始终创建内部的 TypeStore
    pub fn with_database(database: std::sync::Arc<std::sync::RwLock<Database>>) -> Self {
        Self {
            type_env: HashMap::new(),
            constraints: Vec::new(),
            scopes: Vec::new(),
            current_ret: None,
            database,
            errors: Vec::new(),
            warnings: Vec::new(),
            type_store: Arc::new(std::sync::RwLock::new(types::TypeStore::new())),
        }
    }

    /// Plan 084: 使用共享的 TypeStore 创建上下文
    ///
    /// 允许 InferenceContext 与 Parser/Codegen 共享类型存储。
    /// 所有类型查询和注册都通过这个共享的 TypeStore 进行。
    pub fn with_type_store(type_store: Arc<std::sync::RwLock<types::TypeStore>>) -> Self {
        Self {
            type_env: HashMap::new(),
            constraints: Vec::new(),
            scopes: Vec::new(),
            current_ret: None,
            database: std::sync::Arc::new(std::sync::RwLock::new(Database::new())),
            errors: Vec::new(),
            warnings: Vec::new(),
            type_store,
        }
    }

    /// Plan 084: 设置/替换 TypeStore 引用
    ///
    /// 用于在创建上下文后切换到共享的 TypeStore。
    pub fn set_type_store(&mut self, type_store: Arc<std::sync::RwLock<types::TypeStore>>) {
        self.type_store = type_store;
    }

    /// 查找变量的类型
    ///
    /// 首先从内到外查找作用域链，最后在全局类型环境中查找。
    ///
    /// # 参数
    ///
    /// * `name` - 变量名
    ///
    /// # 返回
    ///
    /// 如果找到变量则返回其类型，否则返回 `None`
    pub fn lookup_type(&self, name: &Name) -> Option<Type> {
        // 从内到外查找作用域链
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }

        // 最后在全局类型环境中查找
        if let Some(ty) = self.type_env.get(name) {
            return Some(ty.clone());
        }

        None
    }

    /// 添加变量绑定到类型环境
    ///
    /// 如果有活动的作用域，绑定到最内层作用域；否则绑定到全局类型环境。
    ///
    /// # 参数
    ///
    /// * `name` - 变量名
    /// * `ty` - 变量类型
    pub fn bind_var(&mut self, name: Name, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ty);
        } else {
            self.type_env.insert(name, ty);
        }
    }

    /// 添加类型约束
    ///
    /// # 参数
    ///
    /// * `constraint` - 要添加的约束
    pub fn add_constraint(&mut self, constraint: super::TypeConstraint) {
        self.constraints.push(constraint);
    }

    /// 推入新的作用域
    ///
    /// 用于处理变量遮蔽和块级作用域
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Plan 084: 注册类型声明到 TypeStore
    pub fn register_type_decl(&mut self, type_decl: crate::ast::TypeDecl) {
        if let Ok(mut store) = self.type_store.write() {
            store.register_type_decl(&type_decl);
        }
    }

    /// Phase 089/084: 查找类型声明
    ///
    /// Plan 084: 只从 TypeStore 查找类型声明
    pub fn lookup_type_decl(&self, name: &auto_val::AutoStr) -> Option<crate::ast::TypeDecl> {
        if let Ok(store) = self.type_store.read() {
            if let Some(decl) = store.lookup_type_decl(name) {
                return Some(decl.as_ref().clone());
            }
        }
        None
    }

    /// 注册函数声明
    ///
    /// Plan 084: 只存储到 TypeStore
    pub fn register_fn(&mut self, fn_decl: Fn) {
        if let Ok(mut store) = self.type_store.write() {
            store.register_fn_decl(&fn_decl);
        }
    }

    /// 注册 spec 声明
    ///
    /// Plan 084: 只存储到 TypeStore
    pub fn register_spec(&mut self, spec_decl: SpecDecl) {
        if let Ok(mut store) = self.type_store.write() {
            store.register_spec_decl(&spec_decl);
        }
    }

    /// 查找元数据（替代 Universe 的 lookup_meta）
    ///
    /// Plan 084: 只从 TypeStore 查找
    pub fn lookup_meta(&self, name: &str) -> Option<Rc<crate::scope::Meta>> {
        use crate::scope::Meta;

        if let Ok(store) = self.type_store.read() {
            // 查找函数声明
            if let Some(fn_decl) = store.lookup_fn_decl_str(name) {
                return Some(Rc::new(Meta::Fn(fn_decl.clone())));
            }

            // 查找 spec 声明
            if let Some(spec_decl) = store.lookup_spec_decl_str(name) {
                return Some(Rc::new(Meta::Spec(spec_decl.clone())));
            }

            // 查找类型声明
            if let Some(type_decl) = store.lookup_type_decl_str(name) {
                return Some(Rc::new(Meta::Type(Type::User(type_decl.as_ref().clone()))));
            }
        }

        // 查找变量绑定（从 type_env）
        // 注意：type_env 中只存储类型信息，不包含 Store 的完整元数据
        // 这是与 Universe 的主要区别，因为 Store 还包含 expr 等信息
        if let Some(ty) = self.lookup_type(&Name::from(name)) {
            // 构造一个简单的 Store，只包含类型信息
            // TODO: 如果需要完整的 Store 信息（包括 expr），可能需要额外的存储
            return Some(Rc::new(Meta::Store(Store {
                name: crate::ast::Name::from(name),
                ty,
                // Store kind: 默认为 Var（Plan 091: 修复 var 赋值检查）
                kind: StoreKind::Var,
                // expr 使用默认值（空表达式）
                expr: crate::ast::Expr::Nil,
            })));
        }

        None
    }

    /// 弹出当前作用域
    ///
    /// # 注意
    ///
    /// 调用此函数前应该确保有作用域可弹出，否则会 panic。
    pub fn pop_scope(&mut self) {
        self.scopes.pop().expect("No scope to pop");
    }

    /// Plan 090: 进入函数作用域
    ///
    /// 与 `push_scope()` 功能相同，但语义上表示进入函数。
    /// 用于替代 Universe 的 `enter_fn()` 方法。
    pub fn enter_fn(&mut self, _name: &str) {
        self.push_scope();
    }

    /// Plan 090: 退出函数作用域
    ///
    /// 与 `pop_scope()` 功能相同，但语义上表示退出函数。
    pub fn exit_fn(&mut self) {
        self.pop_scope();
    }

    /// Plan 090: 获取所有已定义的变量名
    ///
    /// 用于错误提示中的候选名称列表。
    pub fn get_defined_var_names(&self) -> Vec<String> {
        let mut names = Vec::new();

        // 从作用域栈中收集
        for scope in &self.scopes {
            for name in scope.keys() {
                names.push(name.to_string());
            }
        }

        // 从全局类型环境中收集
        for name in self.type_env.keys() {
            names.push(name.to_string());
        }

        names
    }

    /// 设置当前函数的返回类型
    ///
    /// # 参数
    ///
    /// * `ret` - 返回类型
    pub fn set_return_type(&mut self, ret: Type) {
        self.current_ret = Some(ret);
    }

    /// 获取当前函数的返回类型
    ///
    /// # 返回
    ///
    /// 如果在函数中则返回返回类型，否则返回 `None`
    pub fn get_return_type(&self) -> Option<Type> {
        self.current_ret.clone()
    }

    /// 添加错误
    ///
    /// # 参数
    ///
    /// * `error` - 类型错误
    pub fn add_error(&mut self, error: AutoError) {
        self.errors.push(error);
    }

    /// 添加警告
    ///
    /// # 参数
    ///
    /// * `warning` - 类型警告
    pub fn add_warning(&mut self, warning: Warning) {
        self.warnings.push(warning);
    }

    /// 检查是否有错误
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// 检查是否有警告
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// 统一两个类型
    ///
    /// 这是类型统一算法的入口点，将被 `unification.rs` 模块完整实现。
    ///
    /// # 参数
    ///
    /// * `ty1` - 第一个类型
    /// * `ty2` - 第二个类型
    ///
    /// # 返回
    ///
    /// 统一后的类型，如果无法统一则返回错误
    pub fn unify(&mut self, ty1: Type, ty2: Type) -> Result<Type, TypeError> {
        match (ty1, ty2) {
            // Unknown 类型是通配符，可以与任何类型统一
            (Type::Unknown, ty) | (ty, Type::Unknown) => Ok(ty),

            // 相同的基础类型
            (Type::Byte, Type::Byte) => Ok(Type::Byte),
            (Type::Int, Type::Int) => Ok(Type::Int),
            (Type::Uint, Type::Uint) => Ok(Type::Uint),
            (Type::USize, Type::USize) => Ok(Type::USize),
            (Type::Float, Type::Float) => Ok(Type::Float),
            (Type::Double, Type::Double) => Ok(Type::Double),
            (Type::Bool, Type::Bool) => Ok(Type::Bool),
            (Type::Char, Type::Char) => Ok(Type::Char),
            (Type::Void, Type::Void) => Ok(Type::Void),

            // 字符串类型：允许长度统一（未知长度）
            (Type::Str(n1), Type::Str(n2)) => {
                if n1 == n2 {
                    Ok(Type::Str(n1))
                } else {
                    Ok(Type::Str(0)) // 未知长度
                }
            }
            // Str literal → StrSlice 隐式转换
            (Type::Str(_), Type::StrSlice) => Ok(Type::StrSlice),
            (Type::StrSlice, Type::Str(_)) => Ok(Type::StrSlice),
            (Type::StrSlice, Type::StrSlice) => Ok(Type::StrSlice),
            // String ↔ StrSlice 隐式转换
            (Type::String, Type::StrSlice) | (Type::StrSlice, Type::String) => Ok(Type::StrSlice),
            (Type::Str(_) | Type::String, Type::String) | (Type::String, Type::Str(_)) => {
                Ok(Type::String)
            }
            (Type::CStr, Type::CStr) => Ok(Type::CStr),

            // 数组类型：统一元素类型和长度
            (Type::Array(arr1), Type::Array(arr2)) => {
                let elem_ty = self.unify(*arr1.elem.clone(), *arr2.elem.clone())?;
                if arr1.len == arr2.len {
                    Ok(Type::Array(crate::ast::ArrayType {
                        elem: Box::new(elem_ty),
                        len: arr1.len,
                    }))
                } else {
                    Err(TypeError::Mismatch {
                        expected: format!("[{}; {}]", elem_ty, arr1.len),
                        found: format!("[{}; {}]", elem_ty, arr2.len),
                        span: SourceSpan::new(0.into(), 0),
                    })
                }
            }

            // 强制转换：int <-> uint（带警告）
            (Type::Int, Type::Uint) | (Type::Uint, Type::Int) => {
                self.warnings.push(Warning::ImplicitTypeConversion {
                    from: "int".into(),
                    to: "uint".into(),
                    span: SourceSpan::new(0.into(), 0),
                });
                Ok(Type::Uint)
            }

            // 强制转换：float <-> double（带警告）
            (Type::Float, Type::Double) | (Type::Double, Type::Float) => {
                self.warnings.push(Warning::ImplicitTypeConversion {
                    from: "float".into(),
                    to: "double".into(),
                    span: SourceSpan::new(0.into(), 0),
                });
                Ok(Type::Double)
            }

            // 用户类型：必须名称匹配
            (Type::User(decl1), Type::User(decl2)) => {
                if decl1.name == decl2.name {
                    Ok(Type::User(decl1))
                } else {
                    Err(TypeError::Mismatch {
                        expected: decl1.name.to_string(),
                        found: decl2.name.to_string(),
                        span: SourceSpan::new(0.into(), 0),
                    })
                }
            }

            // 其他组合：类型不匹配
            (ty1, ty2) => Err(TypeError::Mismatch {
                expected: ty1.to_string(),
                found: ty2.to_string(),
                span: SourceSpan::new(0.into(), 0),
            }),
        }
    }

    /// 尝试统一两个类型，带类型强制转换支持
    ///
    /// 某些类型之间可以自动转换（如 int <-> uint, float <-> double）。
    ///
    /// # 参数
    ///
    /// * `ty1` - 第一个类型
    /// * `ty2` - 第二个类型
    /// * `span` - 源代码位置（用于警告）
    ///
    /// # 返回
    ///
    /// 统一后的类型，如果无法统一则返回错误
    pub fn unify_with_coercion(
        &mut self,
        ty1: Type,
        ty2: Type,
        span: SourceSpan,
    ) -> Result<Type, TypeError> {
        use crate::infer::unification::unify_with_coercion as inner_unify;
        inner_unify(&ty1, &ty2, &mut self.warnings, span).map_err(|e| e.into())
    }

    /// 清空上下文状态
    ///
    /// 用于重新使用上下文进行新的推导
    pub fn clear(&mut self) {
        self.type_env.clear();
        self.constraints.clear();
        self.scopes.clear();
        self.current_ret = None;
        self.errors.clear();
        self.warnings.clear();
    }
}

impl Default for InferenceContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = InferenceContext::new();
        assert!(!ctx.has_errors());
        assert!(!ctx.has_warnings());
    }

    #[test]
    fn test_bind_and_lookup() {
        let mut ctx = InferenceContext::new();
        let name = Name::from("x");

        ctx.bind_var(name.clone(), Type::Int);
        let ty = ctx.lookup_type(&name);

        assert!(matches!(ty, Some(Type::Int)));
    }

    #[test]
    fn test_scope_stack() {
        let mut ctx = InferenceContext::new();
        let name = Name::from("x");

        // 外层作用域
        ctx.bind_var(name.clone(), Type::Int);
        assert!(matches!(ctx.lookup_type(&name), Some(Type::Int)));

        // 内层作用域
        ctx.push_scope();
        ctx.bind_var(name.clone(), Type::Float);
        assert!(matches!(ctx.lookup_type(&name), Some(Type::Float)));

        // 弹出内层作用域
        ctx.pop_scope();
        assert!(matches!(ctx.lookup_type(&name), Some(Type::Int)));
    }

    #[test]
    fn test_unify_same_types() {
        let mut ctx = InferenceContext::new();
        assert!(matches!(ctx.unify(Type::Int, Type::Int), Ok(Type::Int)));
        assert!(matches!(
            ctx.unify(Type::Float, Type::Float),
            Ok(Type::Float)
        ));
        assert!(matches!(ctx.unify(Type::Bool, Type::Bool), Ok(Type::Bool)));
    }

    #[test]
    fn test_unify_different_types_fails() {
        let mut ctx = InferenceContext::new();
        assert!(matches!(
            ctx.unify(Type::Int, Type::Bool),
            Err(TypeError::Mismatch { .. })
        ));
    }

    #[test]
    fn test_unify_with_unknown() {
        let mut ctx = InferenceContext::new();
        assert!(matches!(ctx.unify(Type::Unknown, Type::Int), Ok(Type::Int)));
        assert!(matches!(ctx.unify(Type::Int, Type::Unknown), Ok(Type::Int)));
    }

    #[test]
    fn test_unify_arrays() {
        let mut ctx = InferenceContext::new();
        let arr1 = Type::Array(crate::ast::ArrayType {
            elem: Box::new(Type::Int),
            len: 3,
        });
        let arr2 = Type::Array(crate::ast::ArrayType {
            elem: Box::new(Type::Int),
            len: 3,
        });

        assert!(matches!(ctx.unify(arr1.clone(), arr2), Ok(Type::Array(..))));
    }

    #[test]
    fn test_unify_arrays_different_length_fails() {
        let mut ctx = InferenceContext::new();
        let arr1 = Type::Array(crate::ast::ArrayType {
            elem: Box::new(Type::Int),
            len: 3,
        });
        let arr2 = Type::Array(crate::ast::ArrayType {
            elem: Box::new(Type::Int),
            len: 4,
        });

        assert!(matches!(
            ctx.unify(arr1, arr2),
            Err(TypeError::Mismatch { .. })
        ));
    }
}
