//! PLAN-960 完整模块包绑定：包闭包身份、use/dep、别名/重导出、公共类型/
//! 方法/常量与模块 store。先建声明图（use/重导出边需无环），值初始化
//! 循环具名诊断；函数互递归合法。绑定只重写名字与全局读写为
//! `module.global.get/set:<sym>` 内在调用形态，不做类型合成或 Checked
//! 构造。与 950 `modules::binding` 完全分立，旧面行为不变。
use super::*;
use crate::source::ast::*;
use std::collections::{BTreeMap, BTreeSet};

/// 包闭包的一个输入单元：包名、版本（锁身份）、模块 id 与独立源文本。
/// 包发现由宿主（Man 包锁）提供闭包；本层不搜盘。
#[derive(Clone, Debug)]
pub struct PackageModuleInput {
    pub package: String,
    pub version: String,
    pub id: String,
    pub source: String,
}

#[derive(Clone, Debug)]
pub struct PackageDiagnostic {
    /// `pkg::mod` 显示键；诊断 span 已归位到该模块源内。
    pub module: String,
    pub diagnostic: Diagnostic,
}

#[derive(Clone, Debug)]
pub struct PackageSegment {
    pub package: String,
    pub module: String,
    pub source: String,
    pub start: usize,
}

/// 一个模块级全局（const 或 var）。const 不可变；var 为模块私有 store。
#[derive(Clone, Debug)]
pub struct GlobalDecl {
    pub package: String,
    pub module: String,
    pub name: String,
    pub ty: TypeAnn,
    pub mutable: bool,
    /// 本模块源内的名字 span。
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct BoundPackages {
    /// `pkg::mod` 显示键。
    pub entry: String,
    pub capabilities: Vec<String>,
    /// 扁平化程序：fn/type 项（含合成 init 函数）与 const/store 声明项，
    /// 名字已 mangle；声明项仅作 `module.global.get/set:` 访问器伪签名的
    /// 类型载体（其值体由 init 函数求值，不在该层解析——见 `globals`）。
    pub program: Program,
    pub units: BTreeMap<String, PackageSyntax>,
    pub segments: Vec<PackageSegment>,
    /// fn 符号 -> (package, module, 源函数名/合成名)。
    pub origins: BTreeMap<String, (String, String, String)>,
    /// type 符号 -> (package, module, 源类型名)。
    pub type_origins: BTreeMap<String, (String, String, String)>,
    /// global 符号 -> 声明（const/store）。
    pub globals: BTreeMap<String, GlobalDecl>,
    /// 实际直接调用边（fn 符号 -> fn 符号集；init 函数与全局伪 owner 亦在列）。
    pub references: BTreeMap<String, BTreeSet<String>>,
    /// 符号 -> 直接读取的全局符号集（fn 体 / 全局初始化表达式）。
    pub global_reads: BTreeMap<String, BTreeSet<String>>,
    /// 值初始化总序（全局符号；跨模块 topo，模块内亦按依赖序）。
    pub init_order: Vec<String>,
    /// 模块初始化序（`pkg::mod` 键；diamond 去重后每模块恰一次）。
    pub module_init_order: Vec<String>,
    /// 每个含全局的模块的 init 函数符号。
    pub module_init_fns: BTreeMap<String, String>,
    pub binding_walks: BTreeSet<String>,
    pub input_hash: String,
    pub grammar_id: String,
}

impl BoundPackages {
    pub fn source_len(&self) -> usize {
        self.segments.last().map_or(0, |s| s.start + s.source.len())
    }
    pub fn is_char_boundary(&self, pos: usize) -> bool {
        self.segments
            .iter()
            .any(|s| pos >= s.start && s.source.is_char_boundary(pos - s.start))
    }
    pub fn locate(&self, mut diagnostic: Diagnostic) -> PackageDiagnostic {
        let segment = self
            .segments
            .iter()
            .rev()
            .find(|s| diagnostic.span.start >= s.start);
        let module = segment
            .map(|s| unit_key(&s.package, &s.module))
            .unwrap_or_default();
        if let Some(s) = segment {
            diagnostic.span.start = diagnostic.span.start.saturating_sub(s.start);
            diagnostic.span.end = diagnostic.span.end.saturating_sub(s.start);
        }
        PackageDiagnostic { module, diagnostic }
    }
    /// 模块键的段起点（扁平坐标）。
    pub fn segment_start(&self, key: &str) -> Option<usize> {
        self.segments
            .iter()
            .find(|s| unit_key(&s.package, &s.module) == key)
            .map(|s| s.start)
    }
}

pub const GLOBAL_GET_PREFIX: &str = "module.global.get:";
pub const GLOBAL_SET_PREFIX: &str = "module.global.set:";

fn fail(
    module_key: &str,
    code: &'static str,
    message: impl Into<String>,
    span: Span,
) -> PackageDiagnostic {
    PackageDiagnostic {
        module: module_key.into(),
        diagnostic: Diagnostic::new(Stage::Bind, code, message, span),
    }
}

fn valid_id(id: &str) -> bool {
    let mut b = id.bytes();
    matches!(b.next(), Some(b'a'..=b'z' | b'A'..=b'Z' | b'_'))
        && b.all(|b| b.is_ascii_alphanumeric() || b == b'_')
}
fn valid_package(pkg: &str) -> bool {
    let mut b = pkg.bytes();
    matches!(b.next(), Some(b'a'..=b'z'))
        && b.all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
fn valid_version(v: &str) -> bool {
    !v.is_empty()
        && v.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'+' | b'_'))
}

pub fn unit_key(package: &str, module: &str) -> String {
    format!("{package}::{module}")
}
fn split_key(key: &str) -> (&str, &str) {
    key.split_once("::").unwrap_or((key, ""))
}
/// 包名含 `-` 时在符号中以 `_` 表示（符号须为合法标识符）；长度前缀保证无歧义。
fn sym_part(s: &str) -> String {
    s.replace('-', "_")
}

/// fn 符号：entry 模块的 `main` 保持 `main`；其余
/// `p{包长}_{包}m{模块长}_{模块}_{名}`（长度前缀保证无歧义）。
pub fn fn_symbol(package: &str, module: &str, name: &str, entry: &str) -> String {
    if unit_key(package, module) == entry && name == "main" {
        "main".into()
    } else {
        format!(
            "p{}_{}m{}_{}_{}",
            package.len(),
            sym_part(package),
            module.len(),
            module,
            name
        )
    }
}
/// type 符号：`t{包长}_{包}m{模块长}_{模块}_{名}`。
pub fn type_symbol(package: &str, module: &str, name: &str) -> String {
    format!(
        "t{}_{}m{}_{}_{}",
        package.len(),
        sym_part(package),
        module.len(),
        module,
        name
    )
}
/// global（const/store）符号：`g{包长}_{包}m{模块长}_{模块}_{名}`。
pub fn global_symbol(package: &str, module: &str, name: &str) -> String {
    format!(
        "g{}_{}m{}_{}_{}",
        package.len(),
        sym_part(package),
        module.len(),
        module,
        name
    )
}
/// 模块 init 函数符号：`i{包长}_{包}m{模块长}_{模块}`。
pub fn init_symbol(package: &str, module: &str) -> String {
    format!(
        "i{}_{}m{}_{}",
        package.len(),
        sym_part(package),
        module.len(),
        module
    )
}

struct UnitDecls {
    fns: BTreeMap<String, Span>,
    types: BTreeMap<String, Span>,
    consts: BTreeMap<String, Span>,
    stores: BTreeMap<String, Span>,
}

enum Target {
    Fn(String),
    Type(String),
    Const(String),
}

struct BoundImport {
    symbol: String,
    is_type: bool,
    is_const: bool,
}

/// 绑定宿主提供的包闭包。无文件系统发现、源拼接、parser 回退、类型
/// 合成或 Checked 构造；可见性与初始化序在此裁定。
pub fn bind_packages(
    inputs: &[PackageModuleInput],
    entry: &str,
    caps: &[String],
    budget: FrontBudget,
) -> Result<BoundPackages, PackageDiagnostic> {
    let zero = Span { start: 0, end: 0 };
    let mut units = BTreeMap::new();
    let mut total = 0usize;
    for input in inputs {
        let key = unit_key(&input.package, &input.id);
        total = total
            .checked_add(input.source.len())
            .ok_or_else(|| fail(&key, "source.resource", "module input byte budget", zero))?;
        if total > budget.work_bytes {
            return Err(fail(
                &key,
                "source.resource",
                "module input byte budget",
                zero,
            ));
        }
        if !valid_package(&input.package)
            || !valid_version(&input.version)
            || !valid_id(&input.id)
            || units.contains_key(&key)
        {
            return Err(fail(
                &key,
                "source.module-identity",
                "invalid or duplicate package/module identity",
                zero,
            ));
        }
        let syntax = parse_package_source(&input.source, caps, budget).map_err(|mut ds| {
            PackageDiagnostic {
                module: key.clone(),
                diagnostic: ds.remove(0),
            }
        })?;
        units.insert(key, syntax);
    }
    bind_package_syntax(inputs, entry, caps, budget, units)
}

/// 冷/恢复共用的绑定：源/语法/能力身份先重验，再绑名字。
pub fn bind_package_syntax(
    inputs: &[PackageModuleInput],
    entry: &str,
    caps: &[String],
    budget: FrontBudget,
    units: BTreeMap<String, PackageSyntax>,
) -> Result<BoundPackages, PackageDiagnostic> {
    let zero = Span { start: 0, end: 0 };
    if !caps.iter().any(|c| c == PACKAGES_CAPABILITY)
        || caps
            .iter()
            .any(|c| c == crate::source::modules::MODULES_CAPABILITY)
    {
        return Err(fail(
            entry,
            "source.module-capability",
            "package module capability mismatch",
            zero,
        ));
    }
    let grammar = packages_grammar().map_err(|e| fail(entry, "source.infra", e, zero))?;
    let grammar_id = format!("{}", grammar.program().identity.grammar_id.0);
    let mut sources = BTreeMap::new();
    let mut versions = BTreeMap::new();
    let mut bytes = 0usize;
    for input in inputs {
        let key = unit_key(&input.package, &input.id);
        bytes = bytes
            .checked_add(input.source.len())
            .ok_or_else(|| fail(&key, "source.resource", "module input byte budget", zero))?;
        if bytes > budget.work_bytes {
            return Err(fail(
                &key,
                "source.resource",
                "module input byte budget",
                zero,
            ));
        }
        if !valid_package(&input.package)
            || !valid_version(&input.version)
            || !valid_id(&input.id)
            || sources.contains_key(&key)
        {
            return Err(fail(
                &key,
                "source.module-identity",
                "invalid or duplicate package/module identity",
                zero,
            ));
        }
        let u = units
            .get(&key)
            .ok_or_else(|| fail(&key, "source.module-missing", "source unit absent", zero))?;
        if u.source_hash != crate::runtime_contract::sha256_hex(input.source.as_bytes())
            || u.grammar_id != grammar_id
        {
            return Err(fail(
                &key,
                "source.module-stale",
                "source/grammar identity mismatch",
                zero,
            ));
        }
        sources.insert(key.clone(), input.source.clone());
        versions.insert(key, input.version.clone());
    }
    if units.len() != sources.len() {
        return Err(fail(
            entry,
            "source.module-identity",
            "extraneous projected module",
            zero,
        ));
    }
    if !units.contains_key(entry) {
        return Err(fail(
            entry,
            "source.module-missing",
            "entry module absent",
            zero,
        ));
    }

    // ---- 声明清单与重名拒绝（fn/type/const/store 共享模块命名空间）----
    let mut decls: BTreeMap<String, UnitDecls> = BTreeMap::new();
    for (key, unit) in &units {
        let mut d = UnitDecls {
            fns: BTreeMap::new(),
            types: BTreeMap::new(),
            consts: BTreeMap::new(),
            stores: BTreeMap::new(),
        };
        let mut all: BTreeMap<String, Span> = BTreeMap::new();
        for item in &unit.program.items {
            let (name, span) = match item {
                Item::Fn(f) => (f.name.clone(), f.name.span),
                Item::Type(t) => (t.name.clone(), t.name.span),
                Item::Const(c) => (c.name.clone(), c.name.span),
                Item::Store(v) => (v.name.clone(), v.name.span),
                Item::Enum(en) => {
                    return Err(fail(
                        key,
                        "source.module-surface",
                        "enum declarations belong to the collections surface (PLAN-962)",
                        en.name.span,
                    ))
                }
            };
            if all.insert(name.name.clone(), span).is_some()
                || unit.imports.iter().any(|i| i.bound_name() == name.name)
            {
                return Err(fail(
                    key,
                    "source.module-duplicate-declaration",
                    format!("duplicate declaration `{}`", name.name),
                    span,
                ));
            }
            match item {
                Item::Fn(_) => d.fns.insert(name.name, span),
                Item::Type(_) => d.types.insert(name.name, span),
                Item::Const(_) => d.consts.insert(name.name, span),
                Item::Store(_) => d.stores.insert(name.name, span),
                Item::Enum(_) => unreachable!(),
            };
        }
        decls.insert(key.clone(), d);
    }

    // ---- 路径解析与公共可见性（含重导出链；V-02 私有重导出拒绝）----
    let resolve_path =
        |owner: &str, path: &[String], span: Span| -> Result<(String, String), PackageDiagnostic> {
            let (owner_pkg, _) = split_key(owner);
            let (target_key, item) = match path.len() {
                2 => (unit_key(owner_pkg, &path[0]), path[1].clone()),
                3 => (unit_key(&path[0], &path[1]), path[2].clone()),
                _ => {
                    return Err(fail(
                        owner,
                        "source.module-path",
                        "module paths are `mod.item` or `pkg.mod.item`",
                        span,
                    ))
                }
            };
            if !units.contains_key(&target_key) {
                return Err(fail(
                    owner,
                    "source.module-missing",
                    format!("unknown module `{}`", path[..path.len() - 1].join(".")),
                    span,
                ));
            }
            Ok((target_key, item))
        };
    let public_target = |owner: &str,
                         target_key: &str,
                         item: &str,
                         span: Span,
                         reexport: bool|
     -> Result<Target, PackageDiagnostic> {
        let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
        let mut cursor = (target_key.to_string(), item.to_string());
        loop {
            if !seen.insert(cursor.clone()) {
                return Err(fail(
                    owner,
                    "source.module-reexport-cycle",
                    format!("re-export cycle through `{}.{}`", cursor.0, cursor.1),
                    span,
                ));
            }
            let unit = &units[&cursor.0];
            let d = &decls[&cursor.0];
            let (pkg, module) = split_key(&cursor.0);
            let exported = unit.exports.contains(&cursor.1);
            let private_code: &'static str = if reexport {
                "source.module-reexport-private"
            } else {
                "source.module-private"
            };
            if d.fns.contains_key(&cursor.1) {
                return if exported {
                    Ok(Target::Fn(fn_symbol(pkg, module, &cursor.1, entry)))
                } else {
                    Err(fail(
                        owner,
                        private_code,
                        format!("private function `{}.{}`", cursor.0, cursor.1),
                        span,
                    ))
                };
            }
            if d.types.contains_key(&cursor.1) {
                return if exported {
                    Ok(Target::Type(type_symbol(pkg, module, &cursor.1)))
                } else {
                    Err(fail(
                        owner,
                        private_code,
                        format!("private type `{}.{}`", cursor.0, cursor.1),
                        span,
                    ))
                };
            }
            if d.consts.contains_key(&cursor.1) {
                return if exported {
                    Ok(Target::Const(global_symbol(pkg, module, &cursor.1)))
                } else {
                    Err(fail(
                        owner,
                        private_code,
                        format!("private constant `{}.{}`", cursor.0, cursor.1),
                        span,
                    ))
                };
            }
            if d.stores.contains_key(&cursor.1) {
                return Err(fail(
                    owner,
                    private_code,
                    format!("module store `{}.{}` is private", cursor.0, cursor.1),
                    span,
                ));
            }
            if let Some(re) = unit
                .imports
                .iter()
                .find(|i| i.reexport && i.bound_name() == cursor.1)
            {
                cursor = resolve_path(&cursor.0, &re.path, re.span)?;
                continue;
            }
            return Err(fail(
                owner,
                "source.module-symbol",
                format!("unknown declaration `{}.{}`", cursor.0, cursor.1),
                span,
            ));
        }
    };

    // ---- 导入绑定 + 模块边（仅 use/reexport；调用不构成模块边）----
    let mut binds_by_unit: BTreeMap<String, BTreeMap<String, BoundImport>> = BTreeMap::new();
    let mut edges: BTreeMap<String, BTreeSet<String>> =
        units.keys().map(|k| (k.clone(), BTreeSet::new())).collect();
    for (key, unit) in &units {
        let mut binds = BTreeMap::new();
        for i in &unit.imports {
            let (target_key, item) = resolve_path(key, &i.path, i.span)?;
            if target_key == *key {
                return Err(fail(
                    key,
                    "source.module-path",
                    "a module cannot import from itself",
                    i.span,
                ));
            }
            let target = public_target(key, &target_key, &item, i.span, i.reexport)?;
            let b = match target {
                Target::Fn(s) => BoundImport {
                    symbol: s,
                    is_type: false,
                    is_const: false,
                },
                Target::Type(s) => BoundImport {
                    symbol: s,
                    is_type: true,
                    is_const: false,
                },
                Target::Const(s) => BoundImport {
                    symbol: s,
                    is_type: false,
                    is_const: true,
                },
            };
            binds.insert(i.bound_name().to_string(), b);
            edges.get_mut(key).unwrap().insert(target_key);
        }
        binds_by_unit.insert(key.clone(), binds);
    }
    {
        let mut pending = edges;
        while !pending.is_empty() {
            let leaves: BTreeSet<_> = pending
                .iter()
                .filter(|(_, deps)| deps.is_empty())
                .map(|(k, _)| k.clone())
                .collect();
            if leaves.is_empty() {
                return Err(fail(
                    entry,
                    "source.module-cycle",
                    "cyclic module dependency",
                    zero,
                ));
            }
            pending.retain(|k, _| !leaves.contains(k));
            for deps in pending.values_mut() {
                deps.retain(|k| !leaves.contains(k));
            }
        }
    }

    // ---- 全局表（所有模块先登记，再改写体）----
    let mut globals: BTreeMap<String, GlobalDecl> = BTreeMap::new();
    for (key, unit) in &units {
        let (pkg, module) = split_key(key);
        for item in &unit.program.items {
            let (name, ty, mutable, span) = match item {
                Item::Const(c) => (&c.name, &c.ty, false, c.name.span),
                Item::Store(v) => (&v.name, &v.ty, true, v.name.span),
                _ => continue,
            };
            globals.insert(
                global_symbol(pkg, module, &name.name),
                GlobalDecl {
                    package: pkg.into(),
                    module: module.into(),
                    name: name.name.clone(),
                    ty: ty.clone(),
                    mutable,
                    span,
                },
            );
        }
    }

    // ---- 体改写（名字 mangle、限定调用/常量读取、全局 get/set）----
    let mut program_items: Vec<Item> = Vec::new();
    let mut segments: Vec<PackageSegment> = Vec::new();
    let mut origins: BTreeMap<String, (String, String, String)> = BTreeMap::new();
    let mut type_origins: BTreeMap<String, (String, String, String)> = BTreeMap::new();
    let mut references: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut global_reads: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut binding_walks: BTreeSet<String> = BTreeSet::new();
    // 已改写的初始化表达式（按模块、声明序）。
    let mut initializers: BTreeMap<String, Vec<(String, Expr)>> = BTreeMap::new();
    let mut offset = 0usize;
    let mut identity = format!("{PACKAGES_BINDING_VERSION}:{}:{entry}\n", entry.len());
    let mut result_grammar_id = String::new();
    for (key, unit) in &units {
        let (pkg, module) = split_key(key);
        result_grammar_id = unit.grammar_id.clone();
        identity.push_str(&format!(
            "{}:{}:{}:{}:{}:{}:{}\n",
            pkg.len(),
            pkg,
            versions[key].len(),
            versions[key],
            module.len(),
            module,
            crate::runtime_contract::sha256_hex(sources[key].as_bytes())
        ));
        let mut walker = Walker {
            key,
            entry,
            units: &units,
            decls: &decls,
            binds: &binds_by_unit[key],
            globals: &globals,
            current_owner: String::new(),
            refs: &mut references,
            global_reads: &mut global_reads,
            work: 0,
            depth: 0,
            set_counter: 0,
        };
        let mut inits: Vec<(String, Expr)> = Vec::new();
        for item in &unit.program.items {
            match item {
                Item::Fn(original) => {
                    if let Some(tp) = original.type_params.first() {
                        return Err(fail(
                            key,
                            "source.module-templates",
                            "template functions in package modules close in PLAN-963",
                            tp.span,
                        ));
                    }
                    let owner = fn_symbol(pkg, module, &original.name.name, entry);
                    let mut f = original.clone();
                    walker.current_owner = owner.clone();
                    walker.walk_fn(&mut f)?;
                    binding_walks.insert(owner.clone());
                    origins.insert(
                        owner.clone(),
                        (pkg.into(), module.into(), f.name.name.clone()),
                    );
                    f.name.name = owner;
                    shift_fn(&mut f, offset);
                    program_items.push(Item::Fn(f));
                }
                Item::Type(original) => {
                    let owner = type_symbol(pkg, module, &original.name.name);
                    let mut t = original.clone();
                    for fd in &mut t.fields {
                        walker.walk_ann(&mut fd.ty)?;
                    }
                    for m in &mut t.methods {
                        if let Some(tp) = m.type_params.first() {
                            return Err(fail(
                                key,
                                "source.module-templates",
                                "template methods in package modules close in PLAN-963",
                                tp.span,
                            ));
                        }
                        let m_owner = format!("{owner}.{}", m.name.name);
                        walker.current_owner = m_owner.clone();
                        walker.walk_fn(m)?;
                        binding_walks.insert(m_owner.clone());
                        origins.insert(
                            m_owner,
                            (
                                pkg.into(),
                                module.into(),
                                format!("{}.{}", original.name.name, m.name.name),
                            ),
                        );
                        m.owner = Some(Ident {
                            name: owner.clone(),
                            span: t.name.span,
                        });
                    }
                    type_origins.insert(
                        owner.clone(),
                        (pkg.into(), module.into(), t.name.name.clone()),
                    );
                    t.name.name = owner;
                    shift_type(&mut t, offset);
                    program_items.push(Item::Type(t));
                }
                Item::Const(c) => {
                    let sym = global_symbol(pkg, module, &c.name.name);
                    let mut item = c.clone();
                    walker.current_owner = sym.clone();
                    walker.walk_ann(&mut item.ty)?;
                    walker.walk_expr(&mut item.value, &BTreeSet::new())?;
                    inits.push((sym.clone(), item.value.clone()));
                    // 声明项保留在扁平程序中：resolve 据其注册
                    // `module.global.get/set:` 伪签名（值体不在该层解析——
                    // 初始化已移入 init 函数）。
                    item.name.name = sym;
                    shift_ann(&mut item.ty, offset);
                    shift_expr(&mut item.value, offset);
                    item.span.start += offset;
                    item.span.end += offset;
                    program_items.push(Item::Const(item));
                }
                Item::Store(v) => {
                    let sym = global_symbol(pkg, module, &v.name.name);
                    let mut item = v.clone();
                    walker.current_owner = sym.clone();
                    walker.walk_ann(&mut item.ty)?;
                    walker.walk_expr(&mut item.value, &BTreeSet::new())?;
                    inits.push((sym.clone(), item.value.clone()));
                    item.name.name = sym;
                    shift_ann(&mut item.ty, offset);
                    shift_expr(&mut item.value, offset);
                    item.span.start += offset;
                    item.span.end += offset;
                    program_items.push(Item::Store(item));
                }
                Item::Enum(_) => unreachable!("enum rejected above"),
            }
        }
        drop(walker);
        if !inits.is_empty() {
            initializers.insert(key.clone(), inits);
        }
        segments.push(PackageSegment {
            package: pkg.into(),
            module: module.into(),
            source: sources[key].clone(),
            start: offset,
        });
        offset += sources[key].len() + 1;
    }

    // ---- 值初始化图：g 依赖 h 当 g 的初始化（经被调函数传递）读 h ----
    let reads_closure = |start: &str| -> BTreeSet<String> {
        let mut out: BTreeSet<String> = BTreeSet::new();
        let mut seen: BTreeSet<String> = BTreeSet::new();
        let mut stack = vec![start.to_string()];
        while let Some(n) = stack.pop() {
            if !seen.insert(n.clone()) {
                continue;
            }
            if let Some(r) = global_reads.get(&n) {
                out.extend(r.iter().cloned());
            }
            if let Some(calls) = references.get(&n) {
                for c in calls {
                    stack.push(c.clone());
                }
            }
        }
        out
    };
    let mut deps: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for g in globals.keys() {
        let mut d = reads_closure(g);
        d.remove(g);
        d.retain(|h| globals.contains_key(h));
        deps.insert(g.clone(), d);
    }
    let mut init_order: Vec<String> = Vec::new();
    {
        let mut pending = deps;
        while !pending.is_empty() {
            let ready: Vec<String> = pending
                .iter()
                .filter(|(_, d)| d.is_empty())
                .map(|(g, _)| g.clone())
                .collect();
            if ready.is_empty() {
                // 环成员具名（稳定序）；span 指向首成员声明。
                let members: Vec<String> = pending.keys().cloned().collect();
                let names: Vec<String> = members
                    .iter()
                    .map(|s| {
                        globals
                            .get(s)
                            .map(|g| format!("{}::{}.{}", g.package, g.module, g.name))
                            .unwrap_or_else(|| s.clone())
                    })
                    .collect();
                let first = globals.get(&members[0]).expect("pending global");
                return Err(fail(
                    &unit_key(&first.package, &first.module),
                    "source.module-init-cycle",
                    format!("value initialization cycle: {}", names.join(" -> ")),
                    first.span,
                ));
            }
            for g in &ready {
                init_order.push(g.clone());
                pending.remove(g);
            }
            for d in pending.values_mut() {
                for g in &ready {
                    d.remove(g);
                }
            }
        }
    }

    // ---- 模块初始化序与 init 函数合成（set 调用按初始化总序）----
    let mut module_init_order: Vec<String> = Vec::new();
    for g in &init_order {
        let info = &globals[g];
        let key = unit_key(&info.package, &info.module);
        if !module_init_order.contains(&key) {
            module_init_order.push(key);
        }
    }
    // 跨模块依赖要求：若模块 A 的某全局依赖模块 B 的全局，B 的 init 函数先
    // 跑；但 A 的 init 函数内的 set 序按 init_order——仍需保证 A 内所有依赖
    // B 的全局在 B init 之后——由 module_init_order（按首次出现）保证：
    // 首次出现的模块其首个 ready 全局的依赖均已就绪，但该模块后续全局若依赖
    // 更后模块，则 init 函数整体序会错。因此当模块交错时按"全局总序分段"：
    // 每个模块的 init 函数包含其全部全局，模块序取其**最晚**依赖模块之后。
    // 实现：按 init_order 迭代，模块的秩 = 其全局在 init_order 中的最大位置。
    {
        let mut rank: BTreeMap<String, usize> = BTreeMap::new();
        for (pos, g) in init_order.iter().enumerate() {
            let info = &globals[g];
            let key = unit_key(&info.package, &info.module);
            rank.insert(key, pos);
        }
        module_init_order.sort_by_key(|k| rank[k]);
        // 一致性核：按该模块序逐全局重放，任一全局的依赖须已初始化；若违反
        // （模块内交错依赖无法用单 init 函数满足），具名拒绝。
        let mut done: BTreeSet<String> = BTreeSet::new();
        for key in &module_init_order {
            let (pkg, module) = split_key(key);
            let mine: Vec<&String> = init_order
                .iter()
                .filter(|g| {
                    let i = &globals[*g];
                    i.package == pkg && i.module == module
                })
                .collect();
            for g in &mine {
                let mut d = reads_closure(g);
                d.remove(*g);
                d.retain(|h| globals.contains_key(h));
                for h in &d {
                    if !done.contains(h) && !mine.contains(&h) {
                        let gi = &globals[*g];
                        return Err(fail(
                            key,
                            "source.module-init-order",
                            format!(
                                "module `{key}` cannot initialize as one unit: `{}` needs `{}` from a module initialized later",
                                gi.name,
                                globals.get(h).map(|x| format!("{}::{}.{}", x.package, x.module, x.name)).unwrap_or_default()
                            ),
                            gi.span,
                        ));
                    }
                }
            }
            for g in mine {
                done.insert(g.clone());
            }
        }
    }
    let mut module_init_fns: BTreeMap<String, String> = BTreeMap::new();
    for key in &module_init_order {
        let (pkg, module) = split_key(key);
        let Some(inits) = initializers.remove(key) else {
            continue;
        };
        let start = segments
            .iter()
            .find(|s| unit_key(&s.package, &s.module) == *key)
            .map(|s| s.start)
            .unwrap_or(0);
        let owner = init_symbol(pkg, module);
        let mut by_sym: BTreeMap<String, Expr> = inits.into_iter().collect();
        let mut stmts: Vec<Stmt> = Vec::new();
        let mut init_span: Option<Span> = None;
        for g in &init_order {
            let Some(mut value) = by_sym.remove(g) else {
                continue;
            };
            let decl = &globals[g];
            let span = Span {
                start: decl.span.start + start,
                end: decl.span.end + start,
            };
            init_span = Some(init_span.map_or(span, |s| s.join(span)));
            shift_expr(&mut value, start);
            stmts.push(Stmt::Let {
                kw: span,
                name: Ident {
                    name: format!("__init{}", stmts.len()),
                    span,
                },
                ann: None,
                mutable: false,
                value: Expr {
                    kind: ExprKind::Call {
                        callee: Ident {
                            name: format!("{GLOBAL_SET_PREFIX}{g}"),
                            span,
                        },
                        args: vec![Arg::Pos(value)],
                    },
                    span,
                },
                span,
            });
            // init 函数的调用/读取边 = 各初始化伪 owner 的并集（供 query 事实）。
            let calls = references.get(g).cloned().unwrap_or_default();
            references.entry(owner.clone()).or_default().extend(calls);
            let reads = global_reads.get(g).cloned().unwrap_or_default();
            global_reads.entry(owner.clone()).or_default().extend(reads);
        }
        let span = init_span.unwrap_or(Span { start, end: start });
        stmts.push(Stmt::Return {
            value: Expr {
                kind: ExprKind::IntLit(0),
                span,
            },
            span,
        });
        references.entry(owner.clone()).or_default();
        global_reads.entry(owner.clone()).or_default();
        origins.insert(owner.clone(), (pkg.into(), module.into(), "__init".into()));
        binding_walks.insert(owner.clone());
        program_items.push(Item::Fn(FnItem {
            name: Ident {
                name: owner.clone(),
                span,
            },
            type_params: Vec::new(),
            params: Vec::new(),
            ret: TypeAnn {
                kind: TypeRef::Builtin(TypeWord::Int),
                span,
            },
            body: Block { stmts, span },
            span,
            receiver: Receiver::Free,
            owner: None,
            script: false,
        }));
        module_init_fns.insert(key.clone(), owner);
    }

    Ok(BoundPackages {
        entry: entry.into(),
        capabilities: caps
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        program: Program {
            items: program_items,
        },
        units,
        segments,
        origins,
        type_origins,
        globals,
        references,
        global_reads,
        init_order,
        module_init_order,
        module_init_fns,
        binding_walks,
        input_hash: crate::runtime_contract::sha256_hex(identity.as_bytes()),
        grammar_id: result_grammar_id,
    })
}

// ---- span 平移（扁平坐标）----
fn sh(span: &mut Span, offset: usize) {
    span.start += offset;
    span.end += offset;
}
fn shift_ann(ann: &mut TypeAnn, offset: usize) {
    sh(&mut ann.span, offset);
    match &mut ann.kind {
        TypeRef::Builtin(_) => {}
        TypeRef::TyVar(i) | TypeRef::Record(i) => sh(&mut i.span, offset),
        TypeRef::Optional(inner) => shift_ann(inner, offset),
        // PLAN-962：tuple 类型界平移。
        TypeRef::Tuple(args) => {
            for a in args {
                shift_ann(a, offset);
            }
        }
        TypeRef::Generic { name, args } => {
            sh(&mut name.span, offset);
            for a in args {
                shift_ann(a, offset);
            }
        }
        TypeRef::Function { params, result } => {
            for p in params {
                shift_ann(p, offset);
            }
            shift_ann(result, offset);
        }
    }
}
pub(crate) fn shift_fn(f: &mut FnItem, offset: usize) {
    sh(&mut f.name.span, offset);
    sh(&mut f.span, offset);
    for tp in &mut f.type_params {
        sh(&mut tp.span, offset);
    }
    for p in &mut f.params {
        sh(&mut p.name.span, offset);
        shift_ann(&mut p.ty, offset);
        sh(&mut p.span, offset);
    }
    shift_ann(&mut f.ret, offset);
    if let Some(o) = &mut f.owner {
        sh(&mut o.span, offset);
    }
    shift_block(&mut f.body, offset);
}
fn shift_type(t: &mut TypeItem, offset: usize) {
    sh(&mut t.name.span, offset);
    sh(&mut t.span, offset);
    for fd in &mut t.fields {
        sh(&mut fd.name.span, offset);
        shift_ann(&mut fd.ty, offset);
        sh(&mut fd.span, offset);
    }
    for m in &mut t.methods {
        shift_fn(m, offset);
    }
}
fn shift_block(b: &mut Block, offset: usize) {
    sh(&mut b.span, offset);
    for s in &mut b.stmts {
        shift_stmt(s, offset);
    }
}
fn shift_stmt(s: &mut Stmt, offset: usize) {
    match s {
        // PLAN-962：索引赋值 span/内部平移。
        Stmt::IndexAssign(ia) => {
            shift_expr(&mut ia.index, offset);
            shift_expr(&mut ia.value, offset);
            sh(&mut ia.span, offset);
        }
        // PLAN-962：tuple 解构平移。
        Stmt::LetDestruct { names, value, span } => {
            for n in names {
                sh(&mut n.span, offset);
            }
            shift_expr(value, offset);
            sh(span, offset);
        }
        // PLAN-962 (T-08)：hold 平移。
        Stmt::Hold {
            name,
            bind,
            body,
            span,
        } => {
            sh(&mut name.span, offset);
            sh(&mut bind.span, offset);
            shift_block(body, offset);
            sh(span, offset);
        }
        Stmt::ExprStmt { value, span } | Stmt::Return { value, span } => {
            shift_expr(value, offset);
            sh(span, offset);
        }
        Stmt::Let {
            kw,
            name,
            ann,
            value,
            span,
            ..
        } => {
            sh(kw, offset);
            sh(&mut name.span, offset);
            if let Some(a) = ann {
                shift_ann(a, offset);
            }
            shift_expr(value, offset);
            sh(span, offset);
        }
        Stmt::Assign {
            target,
            value,
            span,
        } => {
            sh(&mut target.base.span, offset);
            for p in &mut target.path {
                sh(&mut p.span, offset);
            }
            shift_expr(value, offset);
            sh(span, offset);
        }
        Stmt::If {
            cond,
            then,
            els,
            span,
        } => {
            shift_expr(cond, offset);
            shift_block(then, offset);
            if let Some(e) = els {
                shift_block(e, offset);
            }
            sh(span, offset);
        }
        Stmt::While { cond, body, span } => {
            shift_expr(cond, offset);
            shift_block(body, offset);
            sh(span, offset);
        }
        Stmt::Loop { body, span } => {
            shift_block(body, offset);
            sh(span, offset);
        }
        Stmt::Break { span } | Stmt::Continue { span } => sh(span, offset),
        Stmt::IsMatch {
            scrutinee,
            arms,
            span,
        } => {
            sh(&mut scrutinee.span, offset);
            for a in arms {
                shift_block(&mut a.body, offset);
                sh(&mut a.span, offset);
            }
            sh(span, offset);
        }
        Stmt::ForIn {
            var,
            iter,
            body,
            span,
        } => {
            sh(&mut var.span, offset);
            shift_expr(iter, offset);
            shift_block(body, offset);
            sh(span, offset);
        }
    }
}
fn shift_arg(a: &mut Arg, offset: usize) {
    match a {
        Arg::Pos(e) | Arg::MovePos(e) => shift_expr(e, offset),
        Arg::Named { name, value, span } => {
            sh(&mut name.span, offset);
            shift_expr(value, offset);
            sh(span, offset);
        }
    }
}
fn shift_expr(e: &mut Expr, offset: usize) {
    sh(&mut e.span, offset);
    match &mut e.kind {
        // PLAN-962：范围界同步平移。
        ExprKind::Range { start, end, .. } => {
            shift_expr(start, offset);
            shift_expr(end, offset);
        }
        // PLAN-962：数组字面量/索引同步平移。
        ExprKind::ArrayLit(elems) => {
            for e in elems {
                shift_expr(e, offset);
            }
        }
        // PLAN-962：tuple 字面量同步平移。
        ExprKind::TupleLit(elems) => {
            for e in elems {
                shift_expr(e, offset);
            }
        }
        // PLAN-962 (T-08)：grid 单元平移。
        ExprKind::GridLit { cols, rows } => {
            for c in cols {
                sh(&mut c.span, offset);
            }
            for row in rows {
                for e in row {
                    shift_expr(e, offset);
                }
            }
        }
        ExprKind::Index { base, index } => {
            shift_expr(base, offset);
            shift_expr(index, offset);
        }
        ExprKind::IntLit(_)
        | ExprKind::BoolLit(_)
        | ExprKind::StrLit(_)
        | ExprKind::CharLit(_)
        | ExprKind::TypeWordArg(_)
        // PLAN-961: a typed literal is a leaf.
        | ExprKind::TypedLit { .. } => {}
        ExprKind::Var(i) | ExprKind::MutArg { name: i } | ExprKind::ViewArg { name: i } => {
            sh(&mut i.span, offset)
        }
        ExprKind::Call { callee, args } => {
            sh(&mut callee.span, offset);
            for a in args {
                shift_arg(a, offset);
            }
        }
        ExprKind::CallValue { callee, args } => {
            shift_expr(callee, offset);
            for a in args {
                shift_arg(a, offset);
            }
        }
        ExprKind::Binary { lhs, rhs, .. } => {
            shift_expr(lhs, offset);
            shift_expr(rhs, offset);
        }
        ExprKind::Unary { operand, .. } => shift_expr(operand, offset),
        ExprKind::Method { recv, name, args } | ExprKind::MethodCall { recv, name, args } => {
            shift_expr(recv, offset);
            sh(&mut name.span, offset);
            for a in args {
                shift_arg(a, offset);
            }
        }
        ExprKind::Field { base, name } => {
            shift_expr(base, offset);
            sh(&mut name.span, offset);
        }
        ExprKind::CtorNamed { ty, fields, .. } => {
            sh(&mut ty.span, offset);
            for f in fields {
                sh(&mut f.name.span, offset);
                shift_expr(&mut f.value, offset);
                sh(&mut f.span, offset);
            }
        }
        ExprKind::GenericType { name, args } => {
            sh(&mut name.span, offset);
            for a in args {
                shift_ann(a, offset);
            }
        }
        ExprKind::HashExpr { expr } => shift_expr(expr, offset),
        ExprKind::Closure { params, body, .. } => {
            for p in params {
                sh(&mut p.name.span, offset);
                shift_ann(&mut p.ty, offset);
                sh(&mut p.span, offset);
            }
            match body.as_mut() {
                ClosureBody::Expr(e) => shift_expr(e, offset),
                ClosureBody::Block(b) => shift_block(b, offset),
            }
        }
    }
}

/// 体改写器：局部/导入/类型/模块可见集决定名字解析；全局读写改写为
/// `module.global.get/set:<sym>` 调用形态。语句走显式帧，表达式递归带
/// 显式深度预算（具名 `source.resource`）。
struct Walker<'a> {
    key: &'a str,
    entry: &'a str,
    units: &'a BTreeMap<String, PackageSyntax>,
    decls: &'a BTreeMap<String, UnitDecls>,
    binds: &'a BTreeMap<String, BoundImport>,
    globals: &'a BTreeMap<String, GlobalDecl>,
    /// 当前改写项的符号（fn / `type.method` / 全局伪 owner）。
    current_owner: String,
    refs: &'a mut BTreeMap<String, BTreeSet<String>>,
    global_reads: &'a mut BTreeMap<String, BTreeSet<String>>,
    work: usize,
    depth: usize,
    /// 合成 `let __setN` 名计数（每模块单调）。
    set_counter: usize,
}

const WALK_WORK_BUDGET: usize = 1 << 22;
const WALK_DEPTH_BUDGET: usize = 2_000;

impl<'a> Walker<'a> {
    fn pkg(&self) -> &'a str {
        split_key(self.key).0
    }
    fn module(&self) -> &'a str {
        split_key(self.key).1
    }
    fn my_decls(&self) -> &'a UnitDecls {
        &self.decls[self.key]
    }

    fn budget(&mut self, span: Span) -> Result<(), PackageDiagnostic> {
        self.work += 1;
        if self.work > WALK_WORK_BUDGET || self.depth > WALK_DEPTH_BUDGET {
            return Err(fail(
                self.key,
                "source.resource",
                "module binding work budget",
                span,
            ));
        }
        Ok(())
    }
    fn record_call(&mut self, target: &str) {
        self.refs
            .entry(self.current_owner.clone())
            .or_default()
            .insert(target.to_string());
    }
    fn record_read(&mut self, sym: &str) {
        self.global_reads
            .entry(self.current_owner.clone())
            .or_default()
            .insert(sym.to_string());
    }

    /// 同包裸模块名 -> 模块键（跨包形以 `pkg.mod` 两段在调用处识别）。
    fn same_package_module(&self, name: &str) -> Option<String> {
        let k = unit_key(self.pkg(), name);
        self.units.contains_key(&k).then_some(k)
    }
    fn is_package_name(&self, name: &str) -> bool {
        self.units.keys().any(|k| split_key(k).0 == name)
    }

    fn walk_fn(&mut self, f: &mut FnItem) -> Result<(), PackageDiagnostic> {
        self.refs.entry(self.current_owner.clone()).or_default();
        self.global_reads
            .entry(self.current_owner.clone())
            .or_default();
        self.walk_ann(&mut f.ret)?;
        for p in &mut f.params {
            self.walk_ann(&mut p.ty)?;
        }
        let mut locals: BTreeSet<String> = f.params.iter().map(|p| p.name.name.clone()).collect();
        if f.receiver == Receiver::View {
            locals.insert("self".into());
        }
        self.walk_block(&mut f.body, locals)
    }

    fn walk_ann(&mut self, ann: &mut TypeAnn) -> Result<(), PackageDiagnostic> {
        match &mut ann.kind {
            // PLAN-962：tuple 类型界递归。
            TypeRef::Tuple(args) => {
                for a in args {
                    self.walk_ann(a)?;
                }
                Ok(())
            }
            TypeRef::Builtin(_) => Ok(()),
            TypeRef::Record(name) => {
                let mangled = self.type_name(name)?;
                name.name = mangled;
                Ok(())
            }
            TypeRef::TyVar(_) => Err(fail(
                self.key,
                "source.module-templates",
                "type variables in package modules close in PLAN-963",
                ann.span,
            )),
            TypeRef::Optional(_) | TypeRef::Generic { .. } | TypeRef::Function { .. } => Err(fail(
                self.key,
                "source.module-surface",
                "optional/generic/function types belong to later packages (PLAN-962..965)",
                ann.span,
            )),
        }
    }

    /// 类型名解析：本模块类型 > 导入类型。
    fn type_name(&self, name: &Ident) -> Result<String, PackageDiagnostic> {
        if self.my_decls().types.contains_key(&name.name) {
            return Ok(type_symbol(self.pkg(), self.module(), &name.name));
        }
        if let Some(b) = self.binds.get(&name.name) {
            return if b.is_type {
                Ok(b.symbol.clone())
            } else {
                Err(fail(
                    self.key,
                    "source.module-symbol",
                    format!("`{}` is not a type", name.name),
                    name.span,
                ))
            };
        }
        Err(fail(
            self.key,
            "source.module-symbol",
            format!("unknown type `{}`", name.name),
            name.span,
        ))
    }
    fn is_type_name(&self, name: &str) -> bool {
        self.my_decls().types.contains_key(name) || self.binds.get(name).is_some_and(|b| b.is_type)
    }
    /// 裸全局名 -> 全局符号（本模块 const/store，或导入常量）。
    fn global_of(&self, name: &str) -> Option<String> {
        let d = self.my_decls();
        if d.consts.contains_key(name) || d.stores.contains_key(name) {
            return Some(global_symbol(self.pkg(), self.module(), name));
        }
        self.binds
            .get(name)
            .filter(|b| b.is_const)
            .map(|b| b.symbol.clone())
    }

    fn walk_block(
        &mut self,
        b: &mut Block,
        locals: BTreeSet<String>,
    ) -> Result<(), PackageDiagnostic> {
        // 显式帧（块与其后续语句共享扩展后的局部集）。
        struct Frame<'b> {
            stmts: &'b mut [Stmt],
            locals: BTreeSet<String>,
        }
        let mut stack: Vec<Frame> = vec![Frame {
            stmts: b.stmts.as_mut_slice(),
            locals,
        }];
        while let Some(frame) = stack.pop() {
            let Frame { stmts, locals } = frame;
            let Some((s, rest)) = stmts.split_first_mut() else {
                continue;
            };
            self.budget(stmt_span(s))?;
            let mut next_locals = locals.clone();
            match s {
                // PLAN-962：索引赋值走读（base 是既有局部）。
                Stmt::IndexAssign(ia) => {
                    self.walk_expr(&mut ia.index, &locals)?;
                    self.walk_expr(&mut ia.value, &locals)?;
                }
                // PLAN-962：tuple 解构走读（新绑定名入后续局部集）。
                Stmt::LetDestruct { names, value, .. } => {
                    self.walk_expr(value, &locals)?;
                    for n in names {
                        next_locals.insert(n.name.clone());
                    }
                }
                // PLAN-962 (T-08)：hold——体内走读（绑定名块内可见；体由
                // 外层帧栈继续走读，别名在此仅记账）。
                Stmt::Hold { bind, .. } => {
                    next_locals.insert(bind.name.clone());
                }
                Stmt::Let {
                    name, ann, value, ..
                } => {
                    if let Some(a) = ann {
                        self.walk_ann(a)?;
                    }
                    self.walk_expr(value, &locals)?;
                    next_locals.insert(name.name.clone());
                }
                Stmt::Assign {
                    target,
                    value,
                    span,
                } => {
                    if !locals.contains(&target.base.name) {
                        if let Some(sym) = self.global_of(&target.base.name) {
                            if !target.path.is_empty() {
                                return Err(fail(
                                    self.key,
                                    "source.module-store-path",
                                    "path assignment to a module store is not part of the packages surface",
                                    target.base.span,
                                ));
                            }
                            if !self.globals.get(&sym).is_some_and(|g| g.mutable) {
                                return Err(fail(
                                    self.key,
                                    "source.module-const-assign",
                                    format!("cannot assign to constant `{}`", target.base.name),
                                    target.base.span,
                                ));
                            }
                            let mut v = std::mem::replace(
                                value,
                                Expr {
                                    kind: ExprKind::IntLit(0),
                                    span: *span,
                                },
                            );
                            self.walk_expr(&mut v, &locals)?;
                            let span = *span;
                            let name_span = target.base.span;
                            // 存储写绑定为合成 let（set 产 0）：所有域都有 let 语句形，
                            // 不依赖 effect-statement 形式。
                            self.set_counter += 1;
                            *s = Stmt::Let {
                                kw: name_span,
                                name: Ident {
                                    name: format!("__set{}", self.set_counter),
                                    span: name_span,
                                },
                                ann: None,
                                mutable: false,
                                value: Expr {
                                    kind: ExprKind::Call {
                                        callee: Ident {
                                            name: format!("{GLOBAL_SET_PREFIX}{sym}"),
                                            span: name_span,
                                        },
                                        args: vec![Arg::Pos(v)],
                                    },
                                    span,
                                },
                                span,
                            };
                            stack.push(Frame {
                                stmts: rest,
                                locals: next_locals,
                            });
                            continue;
                        }
                        // 非局部、非全局：保留给 resolver（其未知名诊断）。
                    }
                    self.walk_expr(value, &locals)?;
                }
                Stmt::ExprStmt { value, .. } | Stmt::Return { value, .. } => {
                    self.walk_expr(value, &locals)?;
                }
                Stmt::If {
                    cond, then, els, ..
                } => {
                    self.walk_expr(cond, &locals)?;
                    stack.push(Frame {
                        stmts: rest,
                        locals: next_locals,
                    });
                    if let Some(e) = els {
                        stack.push(Frame {
                            stmts: e.stmts.as_mut_slice(),
                            locals: locals.clone(),
                        });
                    }
                    stack.push(Frame {
                        stmts: then.stmts.as_mut_slice(),
                        locals,
                    });
                    continue;
                }
                Stmt::While { cond, body, .. } => {
                    self.walk_expr(cond, &locals)?;
                    stack.push(Frame {
                        stmts: rest,
                        locals: next_locals,
                    });
                    stack.push(Frame {
                        stmts: body.stmts.as_mut_slice(),
                        locals,
                    });
                    continue;
                }
                Stmt::Loop { body, .. } => {
                    stack.push(Frame {
                        stmts: rest,
                        locals: next_locals,
                    });
                    stack.push(Frame {
                        stmts: body.stmts.as_mut_slice(),
                        locals,
                    });
                    continue;
                }
                Stmt::Break { .. } | Stmt::Continue { .. } => {}
                Stmt::IsMatch { span, .. } | Stmt::ForIn { span, .. } => {
                    return Err(fail(
                        self.key,
                        "source.module-surface",
                        "`is`/`for in` belong to the collections surface (PLAN-962)",
                        *span,
                    ))
                }
            }
            stack.push(Frame {
                stmts: rest,
                locals: next_locals,
            });
        }
        Ok(())
    }

    fn walk_arg(
        &mut self,
        a: &mut Arg,
        locals: &BTreeSet<String>,
    ) -> Result<(), PackageDiagnostic> {
        match a {
            Arg::Pos(v) | Arg::MovePos(v) | Arg::Named { value: v, .. } => {
                self.walk_expr(v, locals)
            }
        }
    }
    fn walk_args(
        &mut self,
        args: &mut [Arg],
        locals: &BTreeSet<String>,
    ) -> Result<(), PackageDiagnostic> {
        for a in args {
            self.walk_arg(a, locals)?;
        }
        Ok(())
    }

    fn walk_expr(
        &mut self,
        e: &mut Expr,
        locals: &BTreeSet<String>,
    ) -> Result<(), PackageDiagnostic> {
        self.depth += 1;
        self.budget(e.span)?;
        let r = self.walk_expr_inner(e, locals);
        self.depth -= 1;
        r
    }

    fn walk_expr_inner(
        &mut self,
        e: &mut Expr,
        locals: &BTreeSet<String>,
    ) -> Result<(), PackageDiagnostic> {
        let span = e.span;
        match &mut e.kind {
            // PLAN-962：范围界递归走读。
            ExprKind::Range { start, end, .. } => {
                self.walk_expr_inner(start, locals)?;
                self.walk_expr_inner(end, locals)
            }
            // PLAN-962：数组字面量/索引递归走读。
            ExprKind::ArrayLit(elems) => {
                for e in elems {
                    self.walk_expr_inner(e, locals)?;
                }
                Ok(())
            }
            // PLAN-962：tuple 字面量递归走读。
            ExprKind::TupleLit(elems) => {
                for e in elems {
                    self.walk_expr_inner(e, locals)?;
                }
                Ok(())
            }
            // PLAN-962 (T-08)：grid 单元递归走读。
            ExprKind::GridLit { rows, .. } => {
                for row in rows {
                    for e in row {
                        self.walk_expr_inner(e, locals)?;
                    }
                }
                Ok(())
            }
            ExprKind::Index { base, index } => {
                self.walk_expr_inner(base, locals)?;
                self.walk_expr_inner(index, locals)
            }
            ExprKind::IntLit(_)
            | ExprKind::BoolLit(_)
            | ExprKind::StrLit(_)
            | ExprKind::CharLit(_)
            | ExprKind::TypeWordArg(_)
            // PLAN-961: a typed literal is a leaf.
            | ExprKind::TypedLit { .. } => Ok(()),
            ExprKind::Var(v) => {
                if locals.contains(&v.name) {
                    return Ok(());
                }
                if let Some(sym) = self.global_of(&v.name) {
                    self.record_read(&sym);
                    let vspan = v.span;
                    e.kind = ExprKind::Call {
                        callee: Ident {
                            name: format!("{GLOBAL_GET_PREFIX}{sym}"),
                            span: vspan,
                        },
                        args: Vec::new(),
                    };
                    return Ok(());
                }
                if self.my_decls().fns.contains_key(&v.name) {
                    return Err(fail(
                        self.key,
                        "source.module-symbol",
                        format!(
                            "function `{}` is not a value (function values close in PLAN-964)",
                            v.name
                        ),
                        v.span,
                    ));
                }
                if let Some(b) = self.binds.get(&v.name) {
                    if !b.is_type {
                        return Err(fail(
                            self.key,
                            "source.module-symbol",
                            format!("imported function `{}` is not a value", v.name),
                            v.span,
                        ));
                    }
                }
                // 类型名作值、未知名：交 resolver 给出其诊断。
                Ok(())
            }
            ExprKind::Call { callee, args } => {
                if !locals.contains(&callee.name) {
                    if self.my_decls().fns.contains_key(&callee.name) {
                        let sym = fn_symbol(self.pkg(), self.module(), &callee.name, self.entry);
                        self.record_call(&sym);
                        callee.name = sym;
                    } else if let Some(b) = self.binds.get(&callee.name) {
                        if b.is_const {
                            return Err(fail(
                                self.key,
                                "source.module-symbol",
                                format!("constant `{}` is not callable", callee.name),
                                callee.span,
                            ));
                        }
                        if b.is_type {
                            return Err(fail(
                                self.key,
                                "source.module-symbol",
                                format!(
                                    "type `{}` is not callable; construct it with `{{ ... }}`",
                                    callee.name
                                ),
                                callee.span,
                            ));
                        }
                        let sym = b.symbol.clone();
                        self.record_call(&sym);
                        callee.name = sym;
                    } else if self.global_of(&callee.name).is_some() {
                        return Err(fail(
                            self.key,
                            "source.module-symbol",
                            format!("`{}` is a module global, not a function", callee.name),
                            callee.span,
                        ));
                    } else if self.is_type_name(&callee.name) {
                        // 位置构造 `Pair(1, true)`：resolver 路由为 record 构造。
                        let sym = self.type_name(&Ident {
                            name: callee.name.clone(),
                            span: callee.span,
                        })?;
                        callee.name = sym;
                    } else {
                        return Err(fail(
                            self.key,
                            "source.module-symbol",
                            format!("unknown function `{}`", callee.name),
                            callee.span,
                        ));
                    }
                }
                self.walk_args(args, locals)
            }
            ExprKind::CallValue { .. } => Err(fail(
                self.key,
                "source.module-surface",
                "indirect calls belong to the closures surface (PLAN-964)",
                span,
            )),
            ExprKind::Binary { lhs, rhs, .. } => {
                self.walk_expr(lhs, locals)?;
                self.walk_expr(rhs, locals)
            }
            ExprKind::Unary { operand, .. } => self.walk_expr(operand, locals),
            ExprKind::Method { recv, args, .. } => {
                // 文本内建方法：接收者改写、方法名由 resolver 按接收者类型绑定。
                self.walk_expr(recv, locals)?;
                self.walk_args(args, locals)
            }
            ExprKind::MethodCall { recv, name, args } => {
                // `mod.item(..)` / `pkg.mod.item(..)` -> 裸调用；类型静态调用 mangle；
                // 其余为实例方法。
                if let Some((target_key, head_span)) = self.module_prefix(recv, locals) {
                    let sym = self.cross_call(&target_key, &name.name, name.span)?;
                    self.record_call(&sym);
                    let mut taken = std::mem::take(args);
                    self.walk_args(&mut taken, locals)?;
                    e.kind = ExprKind::Call {
                        callee: Ident {
                            name: sym,
                            span: head_span.join(name.span),
                        },
                        args: taken,
                    };
                    return Ok(());
                }
                if let ExprKind::Var(base) = &mut recv.kind {
                    if !locals.contains(&base.name) && self.is_type_name(&base.name) {
                        let mangled = self.type_name(base)?;
                        base.name = mangled;
                        return self.walk_args(args, locals);
                    }
                }
                self.walk_expr(recv, locals)?;
                self.walk_args(args, locals)
            }
            ExprKind::Field { base, name } => {
                // `mod.CONST` / `pkg.mod.CONST` 常量读取；局部记录字段保留。
                if let Some((target_key, head_span)) = self.module_prefix(base, locals) {
                    let sym = self.cross_const(&target_key, &name.name, name.span)?;
                    self.record_read(&sym);
                    e.kind = ExprKind::Call {
                        callee: Ident {
                            name: format!("{GLOBAL_GET_PREFIX}{sym}"),
                            span: head_span.join(name.span),
                        },
                        args: Vec::new(),
                    };
                    return Ok(());
                }
                self.walk_expr(base, locals)
            }
            ExprKind::CtorNamed { ty, fields, .. } => {
                let mangled = self.type_name(ty)?;
                ty.name = mangled;
                for f in fields {
                    self.walk_expr(&mut f.value, locals)?;
                }
                Ok(())
            }
            ExprKind::MutArg { .. } | ExprKind::ViewArg { .. } => Err(fail(
                self.key,
                "source.module-surface",
                "`.mut` arguments are outside the packages surface (PLAN-962)",
                span,
            )),
            ExprKind::GenericType { .. } => Err(fail(
                self.key,
                "source.module-surface",
                "parameterized types belong to the collections surface (PLAN-962)",
                span,
            )),
            ExprKind::HashExpr { .. } => Err(fail(
                self.key,
                "source.module-surface",
                "compile-time expressions belong to the CT surface (PLAN-966)",
                span,
            )),
            ExprKind::Closure { .. } => Err(fail(
                self.key,
                "source.module-surface",
                "closures belong to the closures surface (PLAN-964)",
                span,
            )),
        }
    }

    /// 识别模块前缀形状：`Var(mod)`（同包）或 `Field{Var(pkg), mod}`（跨包）。
    /// 局部名、类型名优先于模块名（显式声明的名字遮蔽隐式模块名）。
    fn module_prefix(&self, recv: &Expr, locals: &BTreeSet<String>) -> Option<(String, Span)> {
        match &recv.kind {
            ExprKind::Var(m) => {
                if locals.contains(&m.name)
                    || self.is_type_name(&m.name)
                    || self.global_of(&m.name).is_some()
                {
                    return None;
                }
                self.same_package_module(&m.name).map(|k| (k, m.span))
            }
            ExprKind::Field { base, name: mid } => {
                let ExprKind::Var(pkg) = &base.kind else {
                    return None;
                };
                if locals.contains(&pkg.name)
                    || self.is_type_name(&pkg.name)
                    || self.global_of(&pkg.name).is_some()
                {
                    return None;
                }
                if !self.is_package_name(&pkg.name) {
                    return None;
                }
                let k = unit_key(&pkg.name, &mid.name);
                self.units.contains_key(&k).then_some((k, pkg.span))
            }
            _ => None,
        }
    }

    fn cross_call(
        &self,
        target: &str,
        item: &str,
        span: Span,
    ) -> Result<String, PackageDiagnostic> {
        let d = &self.decls[target];
        let (pkg, module) = split_key(target);
        if d.fns.contains_key(item) {
            if self.units[target].exports.contains(item) || target == self.key {
                return Ok(fn_symbol(pkg, module, item, self.entry));
            }
            return Err(fail(
                self.key,
                "source.module-private",
                format!("private function `{target}.{item}`"),
                span,
            ));
        }
        if d.consts.contains_key(item) || d.stores.contains_key(item) {
            return Err(fail(
                self.key,
                "source.module-symbol",
                format!("`{target}.{item}` is a module global, not a function"),
                span,
            ));
        }
        if d.types.contains_key(item) {
            return Err(fail(
                self.key,
                "source.module-symbol",
                format!("type `{target}.{item}` is not callable"),
                span,
            ));
        }
        // 重导出项经链解析。
        if let Some(re) = self.units[target]
            .imports
            .iter()
            .find(|i| i.reexport && i.bound_name() == item)
        {
            let _ = re;
            return Err(fail(
                self.key,
                "source.module-symbol",
                format!("qualified access to re-export `{target}.{item}` is not supported; import it with `use`"),
                span,
            ));
        }
        Err(fail(
            self.key,
            "source.module-symbol",
            format!("unknown function `{target}.{item}`"),
            span,
        ))
    }

    fn cross_const(
        &self,
        target: &str,
        item: &str,
        span: Span,
    ) -> Result<String, PackageDiagnostic> {
        let d = &self.decls[target];
        let (pkg, module) = split_key(target);
        if d.consts.contains_key(item) {
            if self.units[target].exports.contains(item) || target == self.key {
                return Ok(global_symbol(pkg, module, item));
            }
            return Err(fail(
                self.key,
                "source.module-private",
                format!("private constant `{target}.{item}`"),
                span,
            ));
        }
        if d.stores.contains_key(item) {
            return Err(fail(
                self.key,
                "source.module-private",
                format!("module store `{target}.{item}` is private"),
                span,
            ));
        }
        Err(fail(
            self.key,
            "source.module-symbol",
            format!("unknown constant `{target}.{item}`"),
            span,
        ))
    }
}

fn stmt_span(s: &Stmt) -> Span {
    match s {
        // PLAN-962 (T-08)：hold。
        Stmt::Hold { span, .. } => *span,
        Stmt::ExprStmt { span, .. }
        | Stmt::Let { span, .. }
        | Stmt::Assign { span, .. }
        | Stmt::Return { span, .. }
        | Stmt::If { span, .. }
        | Stmt::While { span, .. }
        | Stmt::Loop { span, .. }
        | Stmt::Break { span }
        | Stmt::Continue { span }
        | Stmt::IsMatch { span, .. }
        | Stmt::ForIn { span, .. } => *span,
        // PLAN-962：索引赋值。
        Stmt::IndexAssign(ia) => ia.span,
        // PLAN-962：tuple 解构。
        Stmt::LetDestruct { span, .. } => *span,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(package: &str, id: &str, source: &str) -> PackageModuleInput {
        PackageModuleInput {
            package: package.into(),
            version: "1.0.0".into(),
            id: id.into(),
            source: source.into(),
        }
    }
    fn bind(
        inputs: &[PackageModuleInput],
        entry: &str,
    ) -> Result<BoundPackages, PackageDiagnostic> {
        bind_packages(
            inputs,
            entry,
            &[PACKAGES_CAPABILITY.into()],
            FrontBudget::WORK_OWNED_64MIB,
        )
    }

    #[test]
    fn packages_bind_aliases_reexports_types_consts_and_stores() {
        let b = bind(
            &[
                input("lib", "geo", "pub type Point { x int\n y int\n fn sum() int { return .x + .y } }\npub const ORIGIN int = 0\npub fn make(x int, y int) Point { return Point { x: x, y: y } }"),
                input("lib", "facade", "pub use lib.geo.make as mk\npub use lib.geo.Point\nfn hidden() int { return 1 }"),
                input("app", "main", "use lib.facade.mk\nuse lib.facade.Point\nvar count int = 0\nconst BASE int = lib.geo.ORIGIN + 1\npub fn bump() int { count = count + 1\n return count }\nfn main() int { let p = mk(1, 2)\n let n = bump()\n return p.sum() + BASE + count + n }"),
            ],
            "app::main",
        )
        .unwrap();
        // fn/type symbols are package+module qualified; entry main stays `main`.
        assert!(b.origins.contains_key("main"));
        assert!(b.origins.contains_key("p3_libm3_geo_make"));
        assert!(b.type_origins.contains_key("t3_libm3_geo_Point"));
        // globals: two in app::main, one in lib::geo; BASE depends on ORIGIN.
        assert_eq!(b.globals.len(), 3);
        let pos = |s: &str| b.init_order.iter().position(|g| g == s).unwrap();
        assert!(pos("g3_libm3_geo_ORIGIN") < pos("g3_appm4_main_BASE"));
        assert_eq!(
            b.module_init_order,
            vec!["lib::geo".to_string(), "app::main".to_string()]
        );
        assert_eq!(b.module_init_fns["app::main"], "i3_appm4_main");
        // store write rewritten to a set call; reads to get calls.
        let bump = b
            .program
            .items
            .iter()
            .find_map(|i| match i {
                Item::Fn(f) if f.name.name == "p3_appm4_main_bump" => Some(f),
                _ => None,
            })
            .unwrap();
        let Stmt::Let { value, .. } = &bump.body.stmts[0] else {
            panic!("set stmt")
        };
        let ExprKind::Call { callee, .. } = &value.kind else {
            panic!("set call")
        };
        assert_eq!(callee.name, "module.global.set:g3_appm4_main_count");
        assert_eq!(
            b.global_reads["p3_appm4_main_bump"],
            BTreeSet::from(["g3_appm4_main_count".to_string()])
        );
        // main references the aliased cross-package maker.
        assert!(b.references["main"].contains("p3_libm3_geo_make"));
        // every bound symbol relocates to a real source span
        for item in &b.program.items {
            if let Item::Fn(f) = item {
                assert!(b.is_char_boundary(f.name.span.start), "{}", f.name.name);
            }
        }
        // input order / host relocation does not change identity (V-05)
        let b2 = bind(
            &[
                input("app", "main", "use lib.facade.mk\nuse lib.facade.Point\nvar count int = 0\nconst BASE int = lib.geo.ORIGIN + 1\npub fn bump() int { count = count + 1\n return count }\nfn main() int { let p = mk(1, 2)\n let n = bump()\n return p.sum() + BASE + count + n }"),
                input("lib", "facade", "pub use lib.geo.make as mk\npub use lib.geo.Point\nfn hidden() int { return 1 }"),
                input("lib", "geo", "pub type Point { x int\n y int\n fn sum() int { return .x + .y } }\npub const ORIGIN int = 0\npub fn make(x int, y int) Point { return Point { x: x, y: y } }"),
            ],
            "app::main",
        )
        .unwrap();
        assert_eq!(b.input_hash, b2.input_hash);
        // a different package version is a different identity
        let mut v2 = input("lib", "geo", "pub type Point { x int\n y int\n fn sum() int { return .x + .y } }\npub const ORIGIN int = 0\npub fn make(x int, y int) Point { return Point { x: x, y: y } }");
        v2.version = "1.0.1".into();
        let b3 = bind(
            &[
                v2,
                input("lib", "facade", "pub use lib.geo.make as mk\npub use lib.geo.Point\nfn hidden() int { return 1 }"),
                input("app", "main", "use lib.facade.mk\nuse lib.facade.Point\nvar count int = 0\nconst BASE int = lib.geo.ORIGIN + 1\npub fn bump() int { count = count + 1\n return count }\nfn main() int { let p = mk(1, 2)\n let n = bump()\n return p.sum() + BASE + count + n }"),
            ],
            "app::main",
        )
        .unwrap();
        assert_ne!(b.input_hash, b3.input_hash);
    }

    #[test]
    fn packages_reject_private_reexport_init_cycles_and_keep_recursion() {
        // V-02: re-exporting a private type is rejected by name.
        let e = bind(
            &[
                input(
                    "lib",
                    "geo",
                    "type Secret { x int }\npub fn f() int { return 1 }",
                ),
                input(
                    "app",
                    "main",
                    "pub use lib.geo.Secret\nfn main() int { return 1 }",
                ),
            ],
            "app::main",
        )
        .unwrap_err();
        assert_eq!(e.diagnostic.code, "source.module-reexport-private", "{e:?}");
        // V-06: value initialization cycle is named with its members.
        let e = bind(
            &[input(
                "app",
                "main",
                "const A int = B + 1\nconst B int = A + 1\nfn main() int { return A }",
            )],
            "app::main",
        )
        .unwrap_err();
        assert_eq!(e.diagnostic.code, "source.module-init-cycle");
        assert!(
            e.diagnostic.message.contains("app::main.A"),
            "{}",
            e.diagnostic.message
        );
        // indirect cycle through a function is also a cycle
        let e = bind(
            &[input(
                "app",
                "main",
                "const A int = f()\nconst B int = A\nfn f() int { return B }\nfn main() int { return A }",
            )],
            "app::main",
        )
        .unwrap_err();
        assert_eq!(e.diagnostic.code, "source.module-init-cycle");
        // function mutual recursion across modules is legal (not a module cycle)
        let b = bind(
            &[
                input("app", "a", "use app.b.even\npub fn odd(n int) int { if n == 0 { return 0 }\n return even(n - 1) }"),
                input("app", "b", "pub fn even(n int) int { if n == 0 { return 1 }\n return app.a.odd(n - 1) }"),
                input("app", "main", "use app.a.odd\nfn main() int { return odd(5) }"),
            ],
            "app::main",
        );
        // note: `use app.b.even` from a and `app.a.odd` from b is a USE edge only from a -> b;
        // b's qualified call does not add a module edge, so the graph stays acyclic.
        assert!(b.is_ok(), "{b:?}");
        // but a use-edge cycle is still rejected
        let e = bind(
            &[
                input(
                    "app",
                    "a",
                    "use app.b.even\npub fn odd() int { return even() }",
                ),
                input(
                    "app",
                    "b",
                    "use app.a.odd\npub fn even() int { return odd() }",
                ),
                input(
                    "app",
                    "main",
                    "use app.a.odd\nfn main() int { return odd() }",
                ),
            ],
            "app::main",
        )
        .unwrap_err();
        assert_eq!(e.diagnostic.code, "source.module-cycle");
    }

    #[test]
    fn packages_reject_private_access_const_assign_store_export_and_templates() {
        for (lib, app, code) in [
            (
                "fn hidden() int { return 1 }",
                "fn main() int { return lib.x.hidden() }",
                "source.module-private",
            ),
            (
                "pub const K int = 1",
                "fn main() int { return lib.x.K() }",
                "source.module-symbol",
            ),
            (
                "var s int = 1\npub fn f() int { return s }",
                "use lib.x.s\nfn main() int { return s }",
                "source.module-private",
            ),
            (
                "pub const K int = 1",
                "use lib.x.K\nfn main() int { K = 2\n return K }",
                "source.module-const-assign",
            ),
            (
                "pub fn f<T>(x T) T { return x }",
                "fn main() int { return 1 }",
                "source.module-templates",
            ),
            (
                "pub fn f() int { return 1 }",
                "use lib.x.f\nfn main() int { return f }",
                "source.module-symbol",
            ),
            (
                "pub fn f() int { return 1 }",
                "use lib.x.f.g\nfn main() int { return 1 }",
                "source.module-path",
            ),
            (
                "pub fn f() int { return 1 }",
                "use lib.x.f\nfn f() int { return 2 }\nfn main() int { return f() }",
                "source.module-duplicate-declaration",
            ),
        ] {
            let e = bind(
                &[input("lib", "x", lib), input("app", "main", app)],
                "app::main",
            )
            .unwrap_err();
            assert_eq!(e.diagnostic.code, code, "{lib} / {app}: {e:?}");
        }
        // same-name private symbols in two packages stay isolated (V-01)
        let b = bind(
            &[
                input(
                    "one",
                    "m",
                    "fn helper() int { return 1 }\npub fn get() int { return helper() }",
                ),
                input(
                    "two",
                    "m",
                    "fn helper() int { return 2 }\npub fn get() int { return helper() }",
                ),
                input(
                    "app",
                    "main",
                    "fn main() int { return one.m.get() + two.m.get() }",
                ),
            ],
            "app::main",
        )
        .unwrap();
        assert!(b.origins.contains_key("p3_onem1_m_helper"));
        assert!(b.origins.contains_key("p3_twom1_m_helper"));
        assert_eq!(
            b.references["p3_onem1_m_get"],
            BTreeSet::from(["p3_onem1_m_helper".to_string()])
        );
        assert_eq!(
            b.references["p3_twom1_m_get"],
            BTreeSet::from(["p3_twom1_m_helper".to_string()])
        );
    }
}
