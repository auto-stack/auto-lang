//! Plan 173: r2a — Rust to AutoLang Transpiler
//!
//! Converts Rust source code into AutoLang (.at) code.
//! Uses the `syn` crate for Rust parsing.

use crate::AutoResult;
use std::fmt::Write;

// ──────────────────────────────────────────────────────────────
// Public API
// ──────────────────────────────────────────────────────────────

/// Transpile Rust source code to AutoLang source code.
pub fn transpile_r2a(_name: &str, rust_code: &str) -> AutoResult<String> {
    let file = syn::parse_file(rust_code).map_err(|e| format!("Rust parse error: {}", e))?;
    let mut trans = R2aTrans::new();
    trans.convert_file(&file);
    let output = trans.into_output();
    Ok(output.trim_end().to_string() + "\n")
}

// ──────────────────────────────────────────────────────────────
// Type Mapping: syn::Type → AutoLang type string
// ──────────────────────────────────────────────────────────────

fn map_type(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(tp) => map_path_type(&tp.path),
        syn::Type::Reference(r) => map_type(&r.elem),
        syn::Type::Ptr(p) => format!("*{}", map_type(&p.elem)),
        syn::Type::Array(a) => {
            let elem = map_type(&a.elem);
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(lit_int),
                ..
            }) = &a.len
            {
                format!("[{}]{}", lit_int.base10_digits(), elem)
            } else {
                format!("[]{}", elem)
            }
        }
        syn::Type::Slice(s) => format!("[]{}", map_type(&s.elem)),
        syn::Type::Tuple(t) => {
            if t.elems.is_empty() {
                "void".into()
            } else {
                let types: Vec<String> = t.elems.iter().map(map_type).collect();
                format!("/* ({}) */", types.join(", "))
            }
        }
        syn::Type::Never(_) => "void".into(),
        syn::Type::Group(g) => map_type(&g.elem),
        syn::Type::Paren(p) => map_type(&p.elem),
        syn::Type::TraitObject(t) => {
            // dyn Trait → comment (AutoLang has no dynamic dispatch)
            let trait_name = t.bounds.iter().filter_map(|b| match b {
                syn::TypeParamBound::Trait(t) => {
                    t.path.segments.last().map(|s| s.ident.to_string())
                }
                _ => None,
            }).collect::<Vec<_>>().join(" + ");
            format!("/* dyn {} */", trait_name)
        }
        syn::Type::ImplTrait(it) => {
            // impl Trait → same as dyn, extract trait name
            let trait_name = it.bounds.iter().filter_map(|b| match b {
                syn::TypeParamBound::Trait(t) => {
                    t.path.segments.last().map(|s| s.ident.to_string())
                }
                _ => None,
            }).collect::<Vec<_>>().join(" + ");
            format!("/* impl {} */", trait_name)
        }
        _ => "/* unknown */".into(),
    }
}

fn map_path_type(path: &syn::Path) -> String {
    if path.segments.is_empty() {
        return "void".into();
    }
    let last = path.segments.last().unwrap();
    let ident = last.ident.to_string();

    match ident.as_str() {
        "i8" => "i8".into(),
        "i16" | "i32" | "isize" => "int".into(),
        "i64" | "i128" => "i64".into(),
        "u8" => "byte".into(),
        "u16" | "u32" | "usize" => "uint".into(),
        "u64" | "u128" => "u64".into(),
        "f32" => "f32".into(),
        "f64" => "float".into(),
        "bool" => "bool".into(),
        "char" => "char".into(),
        "String" => "str".into(),
        "str" => "cstr".into(),  // primitive str (usually behind &)
        _ => map_generic_type(&ident, &last.arguments),
    }
}

fn map_generic_type(name: &str, args: &syn::PathArguments) -> String {
    let type_args: Vec<String> = match args {
        syn::PathArguments::AngleBracketed(ab) => ab
            .args
            .iter()
            .filter_map(|a| match a {
                syn::GenericArgument::Type(t) => Some(map_type(t)),
                _ => None,
            })
            .collect(),
        _ => return name.into(),
    };

    match name {
        "Vec" if type_args.len() == 1 => format!("List<{}>", type_args[0]),
        "Option" if type_args.len() == 1 => format!("may {}", type_args[0]),
        "Result" if type_args.len() == 2 => format!("result {}, {}", type_args[0], type_args[1]),
        "HashMap" | "BTreeMap" if type_args.len() == 2 => {
            format!("Map<{}, {}>", type_args[0], type_args[1])
        }
        "Box" | "Arc" | "Rc" if type_args.len() == 1 => type_args.into_iter().next().unwrap(),
        _ => {
            if type_args.is_empty() {
                name.into()
            } else {
                format!("{}<{}>", name, type_args.join(", "))
            }
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Operator Mapping
// ──────────────────────────────────────────────────────────────

fn map_binop(op: &syn::BinOp) -> &'static str {
    match op {
        syn::BinOp::Add(_) => "+",
        syn::BinOp::Sub(_) => "-",
        syn::BinOp::Mul(_) => "*",
        syn::BinOp::Div(_) => "/",
        syn::BinOp::Rem(_) => "%",
        syn::BinOp::And(_) => "and",
        syn::BinOp::Or(_) => "or",
        syn::BinOp::BitAnd(_) => "&",
        syn::BinOp::BitOr(_) => "|",
        syn::BinOp::BitXor(_) => "^",
        syn::BinOp::Shl(_) => "<<",
        syn::BinOp::Shr(_) => ">>",
        syn::BinOp::Eq(_) => "==",
        syn::BinOp::Ne(_) => "!=",
        syn::BinOp::Lt(_) => "<",
        syn::BinOp::Le(_) => "<=",
        syn::BinOp::Gt(_) => ">",
        syn::BinOp::Ge(_) => ">=",
        syn::BinOp::AddAssign(_) => "+=",
        syn::BinOp::SubAssign(_) => "-=",
        syn::BinOp::MulAssign(_) => "*=",
        syn::BinOp::DivAssign(_) => "/=",
        syn::BinOp::RemAssign(_) => "%=",
        syn::BinOp::BitAndAssign(_) => "&=",
        syn::BinOp::BitOrAssign(_) => "|=",
        syn::BinOp::BitXorAssign(_) => "^=",
        syn::BinOp::ShlAssign(_) => "<<=",
        syn::BinOp::ShrAssign(_) => ">>=",
        _ => "/* unknown op */",
    }
}

// ──────────────────────────────────────────────────────────────
// Method Name Mapping
// ──────────────────────────────────────────────────────────────

fn map_method_name(method: &str) -> &str {
    match method {
        "to_lowercase" => "to_lower",
        "to_uppercase" => "to_upper",
        "len" => "length",
        "push_str" => "append",
        "push" => "push",
        "pop" => "pop",
        "is_empty" => "is_empty",
        "contains" => "has",
        "trim" => "trim",
        "split" => "split",
        "sort" => "sort",
        "to_string" => "to_str",
        "as_str" => "to_cstr",
        "unwrap" => "unwrap",
        "unwrap_or" => "unwrap_or",
        "expect" => "expect",
        _ => method,
    }
}

// ──────────────────────────────────────────────────────────────
// R2aTrans — Main Converter
// ──────────────────────────────────────────────────────────────

/// How `self` appears in an impl method signature.
enum SelfKind {
    Self_,    // &self
    MutSelf,  // &mut self
}

struct R2aTrans {
    output: String,
    indent: usize,
}

impl R2aTrans {
    fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
        }
    }

    fn into_output(self) -> String {
        self.output
    }

    // ── Output helpers ──

    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
    }

    fn write_line(&mut self, s: &str) {
        self.write_indent();
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn write_empty_line(&mut self) {
        self.output.push('\n');
    }

    /// Emit generic parameters with optional trait bounds, e.g. `<T>` or `<T: Clone, U>`
    fn emit_generic_params(&mut self, generics: &syn::Generics) {
        if generics.params.is_empty() {
            return;
        }
        let params: Vec<String> = generics
            .params
            .iter()
            .filter_map(|p| match p {
                syn::GenericParam::Type(tp) => {
                    let ident = tp.ident.to_string();
                    let bounds: Vec<String> = tp
                        .bounds
                        .iter()
                        .filter_map(|b| match b {
                            syn::TypeParamBound::Trait(t) => {
                                t.path.segments.last().map(|s| s.ident.to_string())
                            }
                            syn::TypeParamBound::Lifetime(_) => None,
                            _ => None,
                        })
                        .collect();
                    if bounds.is_empty() {
                        Some(ident)
                    } else {
                        Some(format!("{}: {}", ident, bounds.join("+ ")))
                    }
                }
                syn::GenericParam::Lifetime(_) => None,
                syn::GenericParam::Const(cp) => Some(cp.ident.to_string()),
            })
            .collect();
        if !params.is_empty() {
            self.write("<");
            self.write(&params.join(", "));
            self.write(">");
        }
    }

    // ── File ──

    fn convert_file(&mut self, file: &syn::File) {
        for (i, item) in file.items.iter().enumerate() {
            if i > 0 {
                self.write_empty_line();
            }
            self.convert_item(item);
        }
    }

    // ── Items ──

    fn convert_item(&mut self, item: &syn::Item) {
        match item {
            syn::Item::Fn(f) => self.convert_fn(f),
            syn::Item::Struct(s) => self.convert_struct(s),
            syn::Item::Enum(e) => self.convert_enum(e),
            syn::Item::Use(u) => self.convert_use(u),
            syn::Item::Const(c) => self.convert_const(c),
            syn::Item::Static(s) => self.convert_static(s),
            syn::Item::Impl(i) => self.convert_impl(i),
            syn::Item::Trait(t) => self.convert_trait(t),
            syn::Item::Mod(m) => self.convert_mod(m),
            syn::Item::Type(t) => self.convert_type_alias(t),
            syn::Item::Union(u) => self.convert_union(u),
            _ => {}
        }
    }

    // ── Functions ──

    fn convert_fn(&mut self, item_fn: &syn::ItemFn) {
        let is_pub = !matches!(item_fn.vis, syn::Visibility::Inherited);
        let is_async = item_fn.sig.asyncness.is_some();

        self.emit_doc_attrs(&item_fn.attrs);
        self.emit_non_doc_attrs(&item_fn.attrs);

        self.write_indent();
        if is_pub {
            self.write("pub ");
        }

        if is_async {
            self.write("// async\n");
            self.write_indent();
        }

        self.write("fn ");
        self.write(&item_fn.sig.ident.to_string());

        // Generic type params
        self.emit_generic_params(&item_fn.sig.generics);

        // Parameters
        self.write("(");
        let inputs: Vec<String> = item_fn
            .sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Typed(pt) => {
                    let name = pat_to_string(&pt.pat);
                    if name == "self" {
                        return None;
                    }
                    let ty = map_type(&pt.ty);
                    Some(format!("{} {}", name, ty))
                }
                syn::FnArg::Receiver(_) => None,
            })
            .collect();
        self.write(&inputs.join(", "));
        self.write(")");

        // Return type
        if let syn::ReturnType::Type(_, ty) = &item_fn.sig.output {
            let ret = map_type(ty);
            if ret != "void" {
                let _ = write!(self.output, " {}", ret);
            }
        }

        self.write(" ");
        self.convert_block(&item_fn.block);
        self.write("\n");
    }

    // ── Structs → type ──

    fn convert_struct(&mut self, s: &syn::ItemStruct) {
        let is_pub = !matches!(s.vis, syn::Visibility::Inherited);

        self.emit_doc_attrs(&s.attrs);
        self.emit_non_doc_attrs(&s.attrs);
        self.write_indent();
        if is_pub {
            self.write("pub ");
        }
        self.write("type ");
        self.write(&s.ident.to_string());

        // Generic type params
        self.emit_generic_params(&s.generics);

        match &s.fields {
            syn::Fields::Named(fields) => {
                self.write(" {\n");
                self.indent += 1;
                for field in &fields.named {
                    // Emit field-level attributes as comments
                    self.emit_non_doc_attrs(&field.attrs);
                    let name = field.ident.as_ref().unwrap().to_string();
                    let ty = map_type(&field.ty);
                    self.write_line(&format!("{} {}", name, ty));
                }
                self.indent -= 1;
                self.write_line("}");
            }
            syn::Fields::Unnamed(fields) => {
                let types: Vec<String> = fields.unnamed.iter().map(|f| map_type(&f.ty)).collect();
                let _ = writeln!(self.output, "({})", types.join(", "));
            }
            syn::Fields::Unit => {
                self.write("\n");
            }
        }
    }

    // ── Enums ──

    fn convert_enum(&mut self, e: &syn::ItemEnum) {
        let is_pub = !matches!(e.vis, syn::Visibility::Inherited);

        self.emit_doc_attrs(&e.attrs);
        self.emit_non_doc_attrs(&e.attrs);
        self.write_indent();
        if is_pub {
            self.write("pub ");
        }
        self.write("enum ");
        self.write(&e.ident.to_string());

        // Generic type params
        self.emit_generic_params(&e.generics);

        self.write(" {\n");

        self.indent += 1;
        for variant in &e.variants {
            self.write_indent();
            self.write(&variant.ident.to_string());

            match &variant.fields {
                syn::Fields::Named(fields) => {
                    self.write(" {\n");
                    self.indent += 1;
                    for field in &fields.named {
                        let name = field.ident.as_ref().unwrap().to_string();
                        let ty = map_type(&field.ty);
                        self.write_line(&format!("{} {}", name, ty));
                    }
                    self.indent -= 1;
                    self.write_line("}");
                }
                syn::Fields::Unnamed(fields) => {
                    let types: Vec<String> =
                        fields.unnamed.iter().map(|f| map_type(&f.ty)).collect();
                    self.write(&format!("({})", types.join(", ")));
                    // Emit discriminant if present (after tuple fields)
                    if let Some((_, expr)) = &variant.discriminant {
                        let val = self.expr_to_string(expr);
                        let _ = write!(self.output, " = {}", val);
                    }
                    self.write("\n");
                }
                syn::Fields::Unit => {
                    // Emit discriminant if present
                    if let Some((_, expr)) = &variant.discriminant {
                        let val = self.expr_to_string(expr);
                        let _ = write!(self.output, " = {}", val);
                    }
                    self.write("\n");
                }
            }
        }
        self.indent -= 1;
        self.write_line("}");
    }

    // ── Use ──

    fn convert_use(&mut self, u: &syn::ItemUse) {
        let is_pub = !matches!(u.vis, syn::Visibility::Inherited);

        self.write_indent();
        if is_pub {
            self.write("pub ");
        }
        self.write("use ");
        self.write(&use_tree_to_string(&u.tree));
        self.write("\n");
    }

    // ── Const ──

    fn convert_const(&mut self, c: &syn::ItemConst) {
        let is_pub = !matches!(c.vis, syn::Visibility::Inherited);

        self.write_indent();
        if is_pub {
            self.write("pub ");
        }
        self.write("const ");
        self.write(&c.ident.to_string());
        let ty = map_type(&c.ty);
        let _ = write!(self.output, " {}", ty);
        let val = self.expr_to_string(&c.expr);
        let _ = write!(self.output, " = {}", val);
        self.write("\n");
    }

    // ── Static ──

    fn convert_static(&mut self, s: &syn::ItemStatic) {
        self.write_indent();
        if !matches!(s.vis, syn::Visibility::Inherited) {
            self.write("pub ");
        }
        let ty = map_type(&s.ty);
        let val = self.expr_to_string(&s.expr);
        let _ = writeln!(self.output, "let {} {} = {}", s.ident, ty, val);
    }

    // ── Impl → ext ──

    fn convert_impl(&mut self, impl_block: &syn::ItemImpl) {
        let trait_name = impl_block.trait_.as_ref().map(|(_, path, _)| {
            path.segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default()
        });

        let self_ty = match impl_block.self_ty.as_ref() {
            syn::Type::Path(tp) => tp
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default(),
            _ => "/* unknown self type */".into(),
        };

        self.write_indent();
        self.write("ext ");
        self.write(&self_ty);

        if let Some(trait_n) = &trait_name {
            let _ = write!(self.output, " for {}", trait_n);
        }

        self.write(" {\n");
        self.indent += 1;

        for item in &impl_block.items {
            match item {
                syn::ImplItem::Fn(impl_fn) => {
                    self.convert_impl_fn(impl_fn);
                }
                syn::ImplItem::Const(impl_const) => {
                    self.write_indent();
                    let ty = map_type(&impl_const.ty);
                    let _ = write!(self.output, "const {} {}", impl_const.ident, ty);
                    let val = self.expr_to_string(&impl_const.expr);
                    let _ = write!(self.output, " = {}", val);
                    self.write("\n");
                }
                syn::ImplItem::Type(impl_type) => {
                    let ty = map_type(&impl_type.ty);
                    self.write_line(&format!("// type {} = {}", impl_type.ident, ty));
                }
                _ => {}
            }
        }

        self.indent -= 1;
        self.write_line("}");
        self.write_empty_line();
    }

    fn convert_impl_fn(&mut self, impl_fn: &syn::ImplItemFn) {
        let is_pub = !matches!(impl_fn.vis, syn::Visibility::Inherited);

        // Determine self kind: &self, &mut self, or none (static)
        let self_kind = impl_fn.sig.inputs.first().and_then(|arg| match arg {
            syn::FnArg::Receiver(recv) => {
                if recv.mutability.is_some() {
                    Some(SelfKind::MutSelf)
                } else {
                    Some(SelfKind::Self_)
                }
            }
            _ => None,
        });

        self.write_indent();
        if is_pub {
            self.write("pub ");
        }
        match &self_kind {
            None => self.write("static "),
            Some(SelfKind::MutSelf) => self.write("mut "),
            Some(SelfKind::Self_) => {}
        }
        self.write("fn ");
        self.write(&impl_fn.sig.ident.to_string());

        self.write("(");
        let params: Vec<String> = impl_fn
            .sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Typed(pt) => {
                    let name = pat_to_string(&pt.pat);
                    let ty = map_type(&pt.ty);
                    Some(format!("{} {}", name, ty))
                }
                syn::FnArg::Receiver(_) => None,
            })
            .collect();
        self.write(&params.join(", "));
        self.write(")");

        if let syn::ReturnType::Type(_, ty) = &impl_fn.sig.output {
            let ret = map_type(ty);
            if ret != "void" {
                let _ = write!(self.output, " {}", ret);
            }
        }

        self.write(" ");
        self.convert_block(&impl_fn.block);
        self.write("\n");
    }

    // ── Trait → spec ──

    fn convert_trait(&mut self, trait_def: &syn::ItemTrait) {
        let is_pub = !matches!(trait_def.vis, syn::Visibility::Inherited);

        self.write_indent();
        if is_pub {
            self.write("pub ");
        }
        self.write("spec ");
        self.write(&trait_def.ident.to_string());
        self.write(" {\n");

        self.indent += 1;
        for item in &trait_def.items {
            match item {
                syn::TraitItem::Fn(fn_item) => {
                    self.write_indent();
                    self.write("fn ");
                    self.write(&fn_item.sig.ident.to_string());

                    self.write("(");
                    let params: Vec<String> = fn_item
                        .sig
                        .inputs
                        .iter()
                        .filter_map(|arg| match arg {
                            syn::FnArg::Typed(pt) => {
                                let name = pat_to_string(&pt.pat);
                                if name == "self" {
                                    return None;
                                }
                                let ty = map_type(&pt.ty);
                                Some(format!("{} {}", name, ty))
                            }
                            syn::FnArg::Receiver(_) => None,
                        })
                        .collect();
                    self.write(&params.join(", "));
                    self.write(")");

                    if let syn::ReturnType::Type(_, ty) = &fn_item.sig.output {
                        let ret = map_type(ty);
                        if ret != "void" {
                            let _ = write!(self.output, " {}", ret);
                        }
                    }

                    if let Some(default) = &fn_item.default {
                        self.write(" ");
                        self.convert_block(default);
                    }

                    self.write("\n");
                }
                syn::TraitItem::Const(c) => {
                    let ty = map_type(&c.ty);
                    self.write_line(&format!("const {} {}", c.ident, ty));
                }
                syn::TraitItem::Type(t) => {
                    self.write_line(&format!("// type {}", t.ident));
                }
                _ => {}
            }
        }
        self.indent -= 1;
        self.write_line("}");
        self.write_empty_line();
    }

    // ── Mod → use ──

    fn convert_mod(&mut self, m: &syn::ItemMod) {
        if let Some((_, items)) = &m.content {
            for item in items {
                self.convert_item(item);
            }
        } else {
            self.write_indent();
            if !matches!(m.vis, syn::Visibility::Inherited) {
                self.write("pub ");
            }
            let _ = writeln!(self.output, "use {}", m.ident);
        }
    }

    // ── Type alias ──

    fn convert_type_alias(&mut self, t: &syn::ItemType) {
        let ty = map_type(&t.ty);
        let generics = if !t.generics.params.is_empty() {
            let mut s = String::new();
            // Build generic params string for inline use
            let params: Vec<String> = t
                .generics
                .params
                .iter()
                .filter_map(|p| match p {
                    syn::GenericParam::Type(tp) => {
                        let ident = tp.ident.to_string();
                        let bounds: Vec<String> = tp
                            .bounds
                            .iter()
                            .filter_map(|b| match b {
                                syn::TypeParamBound::Trait(t) => {
                                    t.path.segments.last().map(|s| s.ident.to_string())
                                }
                                syn::TypeParamBound::Lifetime(_) => None,
                                _ => None,
                            })
                            .collect();
                        if bounds.is_empty() {
                            Some(ident)
                        } else {
                            Some(format!("{}: {}", ident, bounds.join("+ ")))
                        }
                    }
                    syn::GenericParam::Lifetime(_) => None,
                    syn::GenericParam::Const(cp) => Some(cp.ident.to_string()),
                })
                .collect();
            if !params.is_empty() {
                s = format!("<{}>", params.join(", "));
            }
            s
        } else {
            String::new()
        };
        self.write_line(&format!("type {}{} = {}", t.ident, generics, ty));
    }

    // ── Union ──

    fn convert_union(&mut self, u: &syn::ItemUnion) {
        self.write_indent();
        let _ = write!(self.output, "union {} ", u.ident);
        self.write("{\n");
        self.indent += 1;
        for field in &u.fields.named {
            let name = field.ident.as_ref().unwrap().to_string();
            let ty = map_type(&field.ty);
            self.write_line(&format!("{} {}", name, ty));
        }
        self.indent -= 1;
        self.write_line("}");
    }

    // ── Block ──

    fn convert_block(&mut self, block: &syn::Block) {
        self.write("{\n");
        self.indent += 1;
        for stmt in &block.stmts {
            self.convert_stmt(stmt);
        }
        self.indent -= 1;
        self.write_indent();
        self.write("}");
    }

    // ── Statements ──

    fn convert_stmt(&mut self, stmt: &syn::Stmt) {
        match stmt {
            syn::Stmt::Local(local) => {
                let s = self.local_to_string(local);
                self.write_line(&s);
            }
            syn::Stmt::Item(item) => {
                self.convert_item(item);
            }
            syn::Stmt::Expr(expr, _semi) => {
                let s = self.expr_to_string(expr);
                self.write_line(&s);
            }
            syn::Stmt::Macro(stmt_macro) => {
                // Macro in statement position (e.g., println!("hello"))
                let s = self.stmt_macro_to_string(stmt_macro);
                self.write_line(&s);
            }
        }
    }

    fn stmt_macro_to_string(&mut self, mac: &syn::StmtMacro) -> String {
        let name = mac
            .mac
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();

        let token_str = mac.mac.tokens.to_string();

        match name.as_str() {
            "println" | "print" => self.format_macro_to_print(&token_str),
            "format" => self.format_macro_to_fstring(&token_str),
            "vec" => self.vec_macro_to_array(&token_str),
            _ => format!("/* {}!({}) */", name, token_str),
        }
    }

    fn local_to_string(&mut self, local: &syn::Local) -> String {
        let (name, is_mut, ty_annotation) = match &local.pat {
            syn::Pat::Ident(pat_ident) => {
                (pat_ident.ident.to_string(), pat_ident.mutability.is_some(), None)
            }
            syn::Pat::Type(pat_type) => {
                let inner_name = pat_to_string(&pat_type.pat);
                let ty = map_type(&pat_type.ty);
                let is_mut = if let syn::Pat::Ident(pi) = &*pat_type.pat {
                    pi.mutability.is_some()
                } else {
                    false
                };
                (inner_name, is_mut, Some(format!(" {}", ty)))
            }
            _ => (pat_to_string(&local.pat), false, None),
        };

        let keyword = if is_mut { "var" } else { "let" };

        if let Some(init) = &local.init {
            let expr = self.expr_to_string(&init.expr);
            match ty_annotation {
                Some(ty) => format!("{} {}{} = {}", keyword, name, ty, expr),
                None => format!("{} {} = {}", keyword, name, expr),
            }
        } else {
            match ty_annotation {
                Some(ty) => format!("{} {}{}", keyword, name, ty),
                None => format!("{} {}", keyword, name),
            }
        }
    }

    // ── Expressions ──

    fn expr_to_string(&mut self, expr: &syn::Expr) -> String {
        match expr {
            syn::Expr::Lit(lit) => lit_to_string(&lit.lit),

            syn::Expr::Binary(bin) => {
                let left = self.expr_to_string(&bin.left);
                let right = self.expr_to_string(&bin.right);
                let op = map_binop(&bin.op);
                format!("{} {} {}", left, op, right)
            }

            syn::Expr::Unary(un) => {
                let inner = self.expr_to_string(&un.expr);
                match &un.op {
                    syn::UnOp::Not(_) => format!("not {}", inner),
                    syn::UnOp::Neg(_) => format!("-{}", inner),
                    syn::UnOp::Deref(_) => format!("{}.*", inner),
                    _ => inner,
                }
            }

            syn::Expr::Call(call) => {
                let args: Vec<String> = call.args.iter().map(|a| self.expr_to_string(a)).collect();

                // Check for special constructors
                if let syn::Expr::Path(path) = &*call.func {
                    let full_path = path_expr_to_string(&path.path);
                    let full_ident = path
                        .path
                        .segments
                        .iter()
                        .map(|s| s.ident.to_string())
                        .collect::<Vec<_>>()
                        .join("::");

                    match full_ident.as_str() {
                        "String::from" | "String::new" | "Box::new" | "Arc::new"
                        | "Rc::new" | "Vec::new" => {
                            return handle_constructor(&full_ident, &args);
                        }
                        "Some" | "Ok" | "Err" => {
                            return handle_constructor(&full_ident, &args);
                        }
                        _ => {}
                    }

                    return format!("{}({})", full_path, args.join(", "));
                }

                let func = self.expr_to_string(&call.func);
                format!("{}({})", func, args.join(", "))
            }

            syn::Expr::MethodCall(mc) => {
                let receiver = self.expr_to_string(&mc.receiver);
                let method = mc.method.to_string();
                let args: Vec<String> = mc.args.iter().map(|a| self.expr_to_string(a)).collect();
                let mapped = map_method_name(&method);

                // Ownership simplifications
                match method.as_str() {
                    "clone" | "into" => return receiver,
                    "to_string" => return format!("{}.to_str()", receiver),
                    _ => {}
                }

                if args.is_empty() {
                    format!("{}.{}()", receiver, mapped)
                } else {
                    format!("{}.{}({})", receiver, mapped, args.join(", "))
                }
            }

            syn::Expr::Path(path) => path_expr_to_string(&path.path),

            syn::Expr::If(expr_if) => self.if_to_string(expr_if),

            syn::Expr::ForLoop(for_loop) => self.for_loop_to_string(for_loop),

            syn::Expr::While(while_loop) => self.while_to_string(while_loop),

            syn::Expr::Loop(loop_expr) => self.loop_to_string(loop_expr),

            syn::Expr::Match(match_expr) => self.match_to_string(match_expr),

            syn::Expr::Block(block) => self.block_expr_to_string(&block.block),

            syn::Expr::Assign(assign) => {
                let left = self.expr_to_string(&assign.left);
                let right = self.expr_to_string(&assign.right);
                format!("{} = {}", left, right)
            }

            syn::Expr::Return(ret) => {
                if let Some(expr) = &ret.expr {
                    format!("return {}", self.expr_to_string(expr))
                } else {
                    "return".into()
                }
            }

            syn::Expr::Break(_) => "break".into(),
            syn::Expr::Continue(_) => "continue".into(),

            syn::Expr::Macro(macro_expr) => self.macro_to_string(macro_expr),

            syn::Expr::Array(array) => {
                let elems: Vec<String> = array.elems.iter().map(|e| self.expr_to_string(e)).collect();
                format!("[{}]", elems.join(", "))
            }

            syn::Expr::Index(index) => {
                let arr = self.expr_to_string(&index.expr);
                let idx = self.expr_to_string(&index.index);
                format!("{}[{}]", arr, idx)
            }

            syn::Expr::Field(field) => {
                let obj = self.expr_to_string(&field.base);
                let name = match &field.member {
                    syn::Member::Named(ident) => ident.to_string(),
                    syn::Member::Unnamed(idx) => idx.index.to_string(),
                };
                format!("{}.{}", obj, name)
            }

            syn::Expr::Reference(ref_expr) => {
                let inner = self.expr_to_string(&ref_expr.expr);
                if ref_expr.mutability.is_some() {
                    format!("{}.mut", inner)
                } else {
                    format!("{}.view", inner)
                }
            }

            syn::Expr::Paren(paren) => format!("({})", self.expr_to_string(&paren.expr)),

            syn::Expr::Group(group) => self.expr_to_string(&group.expr),

            syn::Expr::Range(range) => self.range_to_string(range),

            syn::Expr::Struct(struct_expr) => self.struct_expr_to_string(struct_expr),

            syn::Expr::Cast(cast) => {
                let expr = self.expr_to_string(&cast.expr);
                let ty = map_type(&cast.ty);
                format!("{}.as({})", expr, ty)
            }

            syn::Expr::Try(try_expr) => {
                let expr = self.expr_to_string(&try_expr.expr);
                format!("{}.?", expr)
            }

            syn::Expr::Tuple(tuple) => {
                let elems: Vec<String> = tuple.elems.iter().map(|e| self.expr_to_string(e)).collect();
                format!("({})", elems.join(", "))
            }

            syn::Expr::Closure(closure) => {
                let body = self.expr_to_string(&closure.body);
                let params: Vec<String> = closure.inputs.iter().map(|p| pat_to_string(p)).collect();
                format!("/* |{}| {} */", params.join(", "), body)
            }

            syn::Expr::Async(_) => "/* async block */".into(),
            syn::Expr::Await(await_expr) => {
                let base = self.expr_to_string(&await_expr.base);
                format!("{} /* .await */", base)
            }

            syn::Expr::Repeat(repeat) => {
                let expr = self.expr_to_string(&repeat.expr);
                let len = self.expr_to_string(&repeat.len);
                format!("[{}; {}]", expr, len)
            }

            syn::Expr::Let(expr_let) => {
                let pat = pat_to_string(&expr_let.pat);
                let expr = self.expr_to_string(&expr_let.expr);
                format!("let {} = {}", pat, expr)
            }

            _ => "/* unsupported expr */".into(),
        }
    }

    // ── Control flow ──

    fn if_to_string(&mut self, expr_if: &syn::ExprIf) -> String {
        let cond = self.expr_to_string(&expr_if.cond);
        let body = self.inline_block(&expr_if.then_branch);
        let mut result = format!("if {} {}", cond, body);

        if let Some((_, else_branch)) = &expr_if.else_branch {
            match &**else_branch {
                syn::Expr::If(nested_if) => {
                    result.push_str(" else ");
                    result.push_str(&self.if_to_string(nested_if));
                }
                syn::Expr::Block(block) => {
                    let else_body = self.inline_block(&block.block);
                    result.push_str(&format!(" else {}", else_body));
                }
                _ => {
                    let else_expr = self.expr_to_string(else_branch);
                    result.push_str(&format!(" else {}", else_expr));
                }
            }
        }

        result
    }

    fn for_loop_to_string(&mut self, for_loop: &syn::ExprForLoop) -> String {
        let pat = pat_to_string(&for_loop.pat);
        let iter = self.expr_to_string(&for_loop.expr);
        let body = self.inline_block(&for_loop.body);
        format!("for {} in {} {}", pat, iter, body)
    }

    fn while_to_string(&mut self, while_loop: &syn::ExprWhile) -> String {
        let cond = self.expr_to_string(&while_loop.cond);
        let body = self.inline_block(&while_loop.body);
        format!("for {} {}", cond, body)
    }

    fn loop_to_string(&mut self, loop_expr: &syn::ExprLoop) -> String {
        let body = self.inline_block(&loop_expr.body);
        format!("for ever {}", body)
    }

    fn match_to_string(&mut self, match_expr: &syn::ExprMatch) -> String {
        let target = self.expr_to_string(&match_expr.expr);
        let mut result = format!("is {} {{\n", target);

        let saved_indent = self.indent;
        self.indent += 1;
        for arm in &match_expr.arms {
            let pat = pat_expr_to_string(&arm.pat);
            let body = if let syn::Expr::Block(block) = arm.body.as_ref() {
                if block.block.stmts.len() == 1 {
                    self.stmt_to_inline_string(&block.block.stmts[0])
                } else {
                    self.inline_block(&block.block)
                }
            } else {
                self.expr_to_string(&arm.body)
            };
            let indent_str = "    ".repeat(self.indent);
            result.push_str(&format!("{}{} -> {}\n", indent_str, pat, body));
        }
        self.indent = saved_indent;

        let indent_str = "    ".repeat(self.indent);
        result.push_str(&format!("{}}}", indent_str));
        result
    }

    // ── Block formatting ──

    fn inline_block(&mut self, block: &syn::Block) -> String {
        if block.stmts.is_empty() {
            return "{}".into();
        }

        let mut result = "{\n".to_string();
        let saved_indent = self.indent;
        self.indent += 1;
        for stmt in &block.stmts {
            let s = self.stmt_to_inline_string(stmt);
            let indent_str = "    ".repeat(self.indent);
            result.push_str(&indent_str);
            result.push_str(&s);
            result.push('\n');
        }
        self.indent = saved_indent;
        let indent_str = "    ".repeat(self.indent);
        result.push_str(&indent_str);
        result.push('}');
        result
    }

    fn block_expr_to_string(&mut self, block: &syn::Block) -> String {
        self.inline_block(block)
    }

    fn stmt_to_inline_string(&mut self, stmt: &syn::Stmt) -> String {
        match stmt {
            syn::Stmt::Local(local) => self.local_to_string(local),
            syn::Stmt::Item(_) => "/* nested item */".into(),
            syn::Stmt::Expr(expr, _semi) => self.expr_to_string(expr),
            syn::Stmt::Macro(_) => "/* macro stmt */".into(),
        }
    }

    // ── Ranges ──

    fn range_to_string(&mut self, range: &syn::ExprRange) -> String {
        let start = range
            .start
            .as_ref()
            .map(|e| self.expr_to_string(e))
            .unwrap_or_default();
        let end = range
            .end
            .as_ref()
            .map(|e| self.expr_to_string(e))
            .unwrap_or_default();

        match (&range.limits, range.end.is_some()) {
            (syn::RangeLimits::HalfOpen(_), true) => format!("{}..{}", start, end),
            (syn::RangeLimits::HalfOpen(_), false) => format!("{}..", start),
            (syn::RangeLimits::Closed(_), _) => format!("{}..={}", start, end),
        }
    }

    // ── Struct expression ──

    fn struct_expr_to_string(&mut self, struct_expr: &syn::ExprStruct) -> String {
        let name = struct_expr
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();
        let fields: Vec<String> = struct_expr
            .fields
            .iter()
            .map(|f| {
                let field_name = match &f.member {
                    syn::Member::Named(ident) => ident.to_string(),
                    syn::Member::Unnamed(idx) => idx.index.to_string(),
                };
                let value = self.expr_to_string(&f.expr);
                format!("{}: {}", field_name, value)
            })
            .collect();
        if fields.is_empty() {
            format!("{}()", name)
        } else {
            format!("{}({})", name, fields.join(", "))
        }
    }

    // ── Macros ──

    fn macro_to_string(&mut self, mac: &syn::ExprMacro) -> String {
        let name = mac
            .mac
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();

        let token_str = mac.mac.tokens.to_string();

        match name.as_str() {
            "println" | "print" => self.format_macro_to_print(&token_str),
            "format" => self.format_macro_to_fstring(&token_str),
            "vec" => self.vec_macro_to_array(&token_str),
            "panic" => format!("panic({})", token_str),
            "assert" | "assert_eq" | "assert_ne" => format!("{}({})", name, token_str),
            "unimplemented" => "/* unimplemented!() */".into(),
            "todo" => "/* todo!() */".into(),
            "unreachable" => "/* unreachable!() */".into(),
            _ => format!("/* {}!({}) */", name, token_str),
        }
    }

    /// Convert println!("format string", args) to print(...)
    fn format_macro_to_print(&self, token_str: &str) -> String {
        let token_str = token_str.trim();

        if token_str.is_empty() {
            return "print()".into();
        }

        // Find the format string (first quoted string)
        if !token_str.starts_with('"') {
            return format!("print({})", token_str);
        }

        // Extract the format string
        let (format_str, rest) = extract_string_literal(token_str);

        // Collect arguments (after the format string)
        let args = split_macro_args(&rest);

        // No placeholders: print("hello")
        if !format_str.contains("{}") {
            if args.is_empty() {
                return format!("print(\"{}\")", format_str);
            }
            let mut all = vec![format!("\"{}\"", format_str)];
            all.extend(args);
            return format!("print({})", all.join(", "));
        }

        // println!("{}", x) → print(x)
        if format_str == "{}" && args.len() == 1 {
            return format!("print({})", args[0]);
        }

        // Convert to f-string: print(f"...")
        let fstr = build_fstring(&format_str, &args);
        format!("print(f\"{}\")", fstr)
    }

    /// Convert format!("hello {}", name) to f"hello $name"
    fn format_macro_to_fstring(&self, token_str: &str) -> String {
        let result = self.format_macro_to_print(token_str);
        // Strip the print() wrapper
        if let Some(inner) = result.strip_prefix("print(") {
            if let Some(inner) = inner.strip_suffix(')') {
                return inner.into();
            }
        }
        result
    }

    /// Convert vec![1, 2, 3] → [1, 2, 3]
    fn vec_macro_to_array(&self, token_str: &str) -> String {
        // Normalize spacing from token stream
        let items: Vec<&str> = token_str.split(',').map(|s| s.trim()).collect();
        format!("[{}]", items.join(", "))
    }

    // ── Doc comments ──

    fn emit_doc_attrs(&mut self, attrs: &[syn::Attribute]) {
        for attr in attrs {
            if attr.path().is_ident("doc") {
                if let syn::Meta::NameValue(nv) = &attr.meta {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = &nv.value
                    {
                        let doc = s.value();
                        let doc = doc.trim_end();
                        self.write_line(&format!("//{}", doc));
                    }
                }
            }
        }
    }

    /// Emit non-doc attributes as comments (derive, serde, cfg, test, etc.)
    fn emit_non_doc_attrs(&mut self, attrs: &[syn::Attribute]) {
        for attr in attrs {
            // Skip doc attrs (handled by emit_doc_attrs)
            if attr.path().is_ident("doc") {
                continue;
            }
            // Skip allow/warn attributes (lint control, not semantic)
            if attr.path().is_ident("allow") || attr.path().is_ident("warn") {
                continue;
            }
            // Build attribute string from meta
            let attr_str = meta_to_string(&attr.meta);
            self.write_line(&format!("// #[{}]", attr_str));
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Free helper functions (no &mut self needed)
// ──────────────────────────────────────────────────────────────

/// Convert syn::Meta to a readable attribute string without using quote/ToTokens.
fn meta_to_string(meta: &syn::Meta) -> String {
    match meta {
        syn::Meta::Path(path) => path_expr_to_string(path),
        syn::Meta::List(list) => {
            let path = path_expr_to_string(&list.path);
            let tokens = list.tokens.to_string();
            format!("{}({})", path, tokens)
        }
        syn::Meta::NameValue(nv) => {
            let path = path_expr_to_string(&nv.path);
            let val = expr_meta_value_to_string(&nv.value);
            format!("{} = {}", path, val)
        }
    }
}

fn expr_meta_value_to_string(expr: &syn::Expr) -> String {
    match expr {
        syn::Expr::Lit(lit) => lit_to_string(&lit.lit),
        _ => "/* expr */".into(),
    }
}

fn use_tree_to_string(tree: &syn::UseTree) -> String {
    match tree {
        syn::UseTree::Path(p) => {
            let prefix = p.ident.to_string();
            let rest = use_tree_to_string(&p.tree);
            match prefix.as_str() {
                "crate" | "self" => rest,
                "super" => format!("super.{}", rest),
                _ => format!("{}.{}", prefix, rest),
            }
        }
        syn::UseTree::Name(n) => n.ident.to_string(),
        syn::UseTree::Rename(r) => r.ident.to_string(),
        syn::UseTree::Glob(_) => "*".into(),
        syn::UseTree::Group(g) => {
            let items: Vec<String> = g.items.iter().map(use_tree_to_string).collect();
            items.join(", ")
        }
    }
}

fn handle_constructor(name: &str, args: &[String]) -> String {
    match name {
        "String::from" | "Box::new" | "Arc::new" | "Rc::new" => {
            if args.len() == 1 {
                args[0].clone()
            } else {
                format!("{}({})", name, args.join(", "))
            }
        }
        "String::new" => "\"\"".into(),
        "Vec::new" => "[]".into(),
        "Some" if args.len() == 1 => format!("Some({})", args[0]),
        "Ok" if args.len() == 1 => format!("Ok({})", args[0]),
        "Err" if args.len() == 1 => format!("Err({})", args[0]),
        _ => format!("{}({})", name, args.join(", ")),
    }
}

fn pat_to_string(pat: &syn::Pat) -> String {
    match pat {
        syn::Pat::Ident(ident) => ident.ident.to_string(),
        syn::Pat::Type(pt) => pat_to_string(&pt.pat),
        syn::Pat::Wild(_) => "_".into(),
        syn::Pat::Tuple(tuple) => {
            let elems: Vec<String> = tuple.elems.iter().map(pat_to_string).collect();
            format!("({})", elems.join(", "))
        }
        syn::Pat::Paren(p) => pat_to_string(&p.pat),
        syn::Pat::Reference(r) => pat_to_string(&r.pat),
        syn::Pat::Rest(_) => "..".into(),
        _ => "/* unknown pat */".into(),
    }
}

fn path_expr_to_string(path: &syn::Path) -> String {
    if path.segments.is_empty() {
        return "/* empty path */".into();
    }

    let segs: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();

    if segs.len() >= 2 && segs[0] == "crate" {
        return segs[1..].join(".");
    }
    if segs.len() >= 2 && segs[0] == "self" {
        return segs[1..].join(".");
    }
    if segs.len() >= 2 && segs[0] == "super" {
        return format!("super.{}", segs[1..].join("."));
    }
    if segs.len() == 2 && segs[0] == "Option" && segs[1] == "None" {
        return "None".into();
    }
    if segs.len() == 1 && segs[0] == "None" {
        return "None".into();
    }
    segs.last().unwrap().clone()
}

fn lit_to_string(lit: &syn::Lit) -> String {
    match lit {
        syn::Lit::Int(i) => i.base10_digits().into(),
        syn::Lit::Float(f) => f.base10_digits().into(),
        syn::Lit::Bool(b) => b.value.to_string(),
        syn::Lit::Char(c) => format!("'{}'", c.value()),
        syn::Lit::Str(s) => format!("\"{}\"", s.value()),
        syn::Lit::ByteStr(bs) => {
            let bytes: Vec<String> = bs.value().iter().map(|b| b.to_string()).collect();
            format!("[{}]", bytes.join(", "))
        }
        syn::Lit::Byte(b) => b.value().to_string(),
        syn::Lit::Verbatim(v) => v.to_string(),
        _ => "/* unknown lit */".into(),
    }
}

fn pat_expr_to_string(pat: &syn::Pat) -> String {
    match pat {
        syn::Pat::Wild(_) => "_".into(),
        syn::Pat::Ident(ident) => {
            if ident.mutability.is_some() {
                format!("var {}", ident.ident)
            } else {
                ident.ident.to_string()
            }
        }
        syn::Pat::Lit(lit) => lit_to_string(&lit.lit),
        syn::Pat::Path(path) => path_expr_to_string(&path.path),
        syn::Pat::Tuple(tuple) => {
            let elems: Vec<String> = tuple.elems.iter().map(pat_expr_to_string).collect();
            format!("({})", elems.join(", "))
        }
        syn::Pat::Struct(struct_pat) => {
            let name = struct_pat
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default();
            let fields: Vec<String> = struct_pat
                .fields
                .iter()
                .map(|f| pat_expr_to_string(&f.pat))
                .collect();
            if fields.is_empty() {
                format!("{}()", name)
            } else {
                format!("{}({})", name, fields.join(", "))
            }
        }
        syn::Pat::TupleStruct(ts) => {
            let name = ts
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default();
            let fields: Vec<String> = ts.elems.iter().map(pat_expr_to_string).collect();
            if fields.is_empty() {
                name
            } else {
                format!("{}({})", name, fields.join(", "))
            }
        }
        syn::Pat::Range(range) => {
            let start = range
                .start
                .as_ref()
                .map(|e| {
                    if let syn::Expr::Lit(lit) = e.as_ref() {
                        lit_to_string(&lit.lit)
                    } else {
                        "/* ? */".into()
                    }
                })
                .unwrap_or_default();
            let end = range
                .end
                .as_ref()
                .map(|e| {
                    if let syn::Expr::Lit(lit) = e.as_ref() {
                        lit_to_string(&lit.lit)
                    } else {
                        "/* ? */".into()
                    }
                })
                .unwrap_or_default();
            match &range.limits {
                syn::RangeLimits::HalfOpen(_) => format!("{}..{}", start, end),
                syn::RangeLimits::Closed(_) => format!("{}..={}", start, end),
            }
        }
        syn::Pat::Slice(slice) => {
            let elems: Vec<String> = slice.elems.iter().map(pat_expr_to_string).collect();
            format!("[{}]", elems.join(", "))
        }
        syn::Pat::Or(or) => {
            let cases: Vec<String> = or.cases.iter().map(pat_expr_to_string).collect();
            cases.join(" | ")
        }
        syn::Pat::Type(pt) => pat_expr_to_string(&pt.pat),
        syn::Pat::Paren(p) => format!("({})", pat_expr_to_string(&p.pat)),
        syn::Pat::Rest(_) => "..".into(),
        _ => "/* unknown pattern */".into(),
    }
}

// ──────────────────────────────────────────────────────────────
// String parsing helpers for macro conversion
// ──────────────────────────────────────────────────────────────

/// Extract a string literal from the beginning of a token stream string.
/// Returns (content, remaining_tokens).
fn extract_string_literal(s: &str) -> (String, String) {
    let s = s.trim();
    if !s.starts_with('"') {
        return (s.to_string(), String::new());
    }

    let mut content = String::new();
    let mut chars = s.chars();
    chars.next(); // skip opening quote

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                match next {
                    '"' => content.push('"'),
                    '\\' => content.push('\\'),
                    'n' => content.push('\n'),
                    't' => content.push('\t'),
                    'r' => content.push('\r'),
                    '0' => content.push('\0'),
                    _ => {
                        content.push('\\');
                        content.push(next);
                    }
                }
            }
        } else if c == '"' {
            break;
        } else {
            content.push(c);
        }
    }

    let remaining: String = chars.collect();
    (content, remaining.trim().trim_start_matches(',').trim().to_string())
}

/// Split macro arguments (after the format string) into individual argument strings.
fn split_macro_args(s: &str) -> Vec<String> {
    if s.is_empty() {
        return Vec::new();
    }

    let mut args = Vec::new();
    let mut current = String::new();
    let mut depth = 0;

    for c in s.chars() {
        match c {
            '(' | '[' | '{' => {
                depth += 1;
                current.push(c);
            }
            ')' | ']' | '}' => {
                depth -= 1;
                current.push(c);
            }
            ',' if depth == 0 => {
                let arg = current.trim().to_string();
                if !arg.is_empty() {
                    args.push(arg);
                }
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }

    let last = current.trim().to_string();
    if !last.is_empty() {
        args.push(last);
    }
    args
}

/// Build an f-string from a Rust format string and its arguments.
fn build_fstring(format_str: &str, args: &[String]) -> String {
    let mut result = String::new();
    let mut arg_idx = 0;
    let mut chars = format_str.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'}') {
            chars.next(); // consume '}'
            if arg_idx < args.len() {
                let arg = &args[arg_idx];
                if is_simple_ident(arg) {
                    result.push('$');
                    result.push_str(arg);
                } else {
                    result.push_str("${");
                    result.push_str(arg);
                    result.push('}');
                }
                arg_idx += 1;
            }
        } else if c == '"' {
            result.push('\\');
            result.push('"');
        } else if c == '\\' {
            if let Some(next) = chars.next() {
                match next {
                    'n' => result.push_str("\\n"),
                    't' => result.push_str("\\t"),
                    'r' => result.push_str("\\r"),
                    '"' | '\\' => {
                        result.push('\\');
                        result.push(next);
                    }
                    _ => {
                        result.push('\\');
                        result.push(next);
                    }
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

fn is_simple_ident(s: &str) -> bool {
    !s.is_empty()
        && s.chars().next().unwrap().is_alphabetic()
        && s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

// ──────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_mapping() {
        assert_eq!(map_type(&syn::parse_quote!(i32)), "int");
        assert_eq!(map_type(&syn::parse_quote!(u8)), "byte");
        assert_eq!(map_type(&syn::parse_quote!(f64)), "float");
        assert_eq!(map_type(&syn::parse_quote!(String)), "str");
        assert_eq!(map_type(&syn::parse_quote!(Vec<i32>)), "List<int>");
        assert_eq!(map_type(&syn::parse_quote!(Option<i32>)), "may int");
        assert_eq!(map_type(&syn::parse_quote!(&str)), "cstr");
        assert_eq!(map_type(&syn::parse_quote!(())), "void");
        assert_eq!(map_type(&syn::parse_quote!(*mut i32)), "*int");
        assert_eq!(map_type(&syn::parse_quote!([i32; 10])), "[10]int");
        assert_eq!(map_type(&syn::parse_quote!(&[i32])), "[]int");
        assert_eq!(map_type(&syn::parse_quote!(Box<i32>)), "int");
    }

    #[test]
    fn test_hello_world() {
        let rust_code = r#"fn main() {
    println!("hello, world!");
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("fn main()"), "missing fn main() in: {}", result);
        assert!(result.contains("print"), "missing print in: {}", result);
        assert!(result.contains("hello, world!"), "missing hello in: {}", result);
    }

    #[test]
    fn test_fn_with_params() {
        let rust_code = r#"fn add(a: i32, b: i32) -> i32 {
    a + b
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("fn add(a int, b int) int"));
    }

    #[test]
    fn test_let_var() {
        let rust_code = r#"fn main() {
    let x: i32 = 42;
    let mut count = 0;
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("let x int = 42"));
        assert!(result.contains("var count = 0"));
    }

    #[test]
    fn test_struct() {
        let rust_code = r#"struct Point {
    x: i32,
    y: i32,
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("type Point"));
        assert!(result.contains("x int"));
        assert!(result.contains("y int"));
    }

    #[test]
    fn test_enum() {
        let rust_code = r#"enum Color {
    Red,
    Green,
    Blue,
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("enum Color"));
        assert!(result.contains("Red"));
        assert!(result.contains("Green"));
        assert!(result.contains("Blue"));
    }

    #[test]
    fn test_for_loop() {
        let rust_code = r#"fn main() {
    for i in 0..10 {
        println!("{}", i);
    }
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("for i in 0..10"));
    }

    #[test]
    fn test_while_loop() {
        let rust_code = r#"fn main() {
    while x > 0 {
        x = x - 1;
    }
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("for x > 0"));
    }

    #[test]
    fn test_match() {
        let rust_code = r#"fn main() {
    let x = 1;
    match x {
        0 => println!("zero"),
        1 => println!("one"),
        _ => println!("other"),
    }
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("is x"));
        assert!(result.contains("0 ->"));
        assert!(result.contains("1 ->"));
        assert!(result.contains("_ ->"));
    }

    #[test]
    fn test_impl() {
        let rust_code = r#"struct Point {
    x: i32,
    y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    fn distance(&self) -> f64 {
        0.0
    }
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("ext Point"));
        assert!(result.contains("static fn new"));
    }

    #[test]
    fn test_trait() {
        let rust_code = r#"trait Drawable {
    fn draw(&self);
    fn area(&self) -> f64;
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("spec Drawable"));
        assert!(result.contains("fn draw()"));
        assert!(result.contains("fn area() float"));
    }

    #[test]
    fn test_const() {
        let rust_code = r#"const MAX_SIZE: i32 = 100;
pub const VERSION: &str = "1.0";"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("const MAX_SIZE int = 100"));
    }

    #[test]
    fn test_operators() {
        let rust_code = r#"fn main() {
    let a = true && false;
    let b = true || false;
    let c = !true;
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("and"));
        assert!(result.contains("or"));
        assert!(result.contains("not"));
    }

    #[test]
    fn test_loop() {
        let rust_code = r#"fn main() {
    loop {
        break;
    }
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("for ever"));
        assert!(result.contains("break"));
    }

    #[test]
    fn test_ownership_simplifications() {
        let rust_code = r#"fn main() {
    let s = String::from("hello");
    let b = Box::new(42);
    let c = s.clone();
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains(r#"let s = "hello""#), "missing String::from simplification in: {}", result);
        assert!(result.contains("let b = 42"), "missing Box::new simplification in: {}", result);
    }

    #[test]
    fn test_println_format() {
        let rust_code = r#"fn main() {
    println!("hello");
    println!("{}", x);
    println!("x = {}", x);
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains(r#"print("hello")"#));
    }

    #[test]
    fn test_vec_macro() {
        let rust_code = r#"fn main() {
    let v = vec![1, 2, 3];
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("[1, 2, 3]"), "missing array in: {}", result);
    }

    #[test]
    fn test_if_else() {
        let rust_code = r#"fn main() {
    if x > 1 {
        println!("big");
    } else {
        println!("small");
    }
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("if x > 1"));
        assert!(result.contains("else"));
    }

    #[test]
    fn test_result_type() {
        assert_eq!(
            map_type(&syn::parse_quote!(Result<i32, String>)),
            "result int, str"
        );
    }

    #[test]
    fn test_hashmap_type() {
        assert_eq!(
            map_type(&syn::parse_quote!(HashMap<String, i32>)),
            "Map<str, int>"
        );
    }

    // ── File-based tests ──

    fn test_r2a_file(case: &str) {
        let base = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test")
            .join("r2a");
        let input_path = base.join(case).with_extension("rs");
        let expected_path = base.join(case).with_extension("expected.at");
        let wrong_path = base.join(case).with_extension("wrong.at");

        let rust_code = std::fs::read_to_string(&input_path)
            .unwrap_or_else(|e| panic!("Cannot read {}: {}", input_path.display(), e));
        let expected = std::fs::read_to_string(&expected_path)
            .unwrap_or_else(|e| panic!("Cannot read {}: {}", expected_path.display(), e));

        let actual = transpile_r2a(case, &rust_code).unwrap();

        if actual.trim() != expected.trim() {
            std::fs::write(&wrong_path, &actual).ok();
            panic!(
                "r2a output mismatch for {}\nExpected:\n{}\nGot:\n{}\nWrong output written to {}",
                case,
                expected.trim(),
                actual.trim(),
                wrong_path.display()
            );
        }

        // Clean up any previous .wrong.at file on success
        if wrong_path.exists() {
            std::fs::remove_file(&wrong_path).ok();
        }
    }

    #[test]
    fn test_r2a_001_hello() {
        test_r2a_file("01_basics/001_hello");
    }

    #[test]
    fn test_r2a_002_func() {
        test_r2a_file("01_basics/002_func");
    }

    #[test]
    fn test_r2a_003_struct() {
        test_r2a_file("02_types/001_struct");
    }

    #[test]
    fn test_r2a_004_enum() {
        test_r2a_file("02_types/002_enum");
    }

    #[test]
    fn test_r2a_005_if_basic() {
        test_r2a_file("03_control/001_if_basic");
    }

    #[test]
    fn test_r2a_006_for_range() {
        test_r2a_file("03_control/002_for_range");
    }

    #[test]
    fn test_r2a_007_match() {
        test_r2a_file("03_control/003_match");
    }

    #[test]
    fn test_r2a_008_while() {
        test_r2a_file("03_control/004_while");
    }

    #[test]
    fn test_r2a_009_loop() {
        test_r2a_file("03_control/005_loop");
    }

    // ── Round-trip tests: a2r .expected.rs → r2a → structural checks ──

    /// Read an a2r .expected.rs file, transpile with r2a, and verify structural properties.
    fn roundtrip_a2r(case: &str) {
        let base = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test")
            .join("a2r");
        let dir = base.join(case);
        let rs_path = dir.read_dir().ok().and_then(|mut entries| {
            entries.find_map(|e| {
                let e = e.ok()?;
                let p = e.path();
                if p.extension().map(|e| e == "rs").unwrap_or(false) {
                    Some(p)
                } else {
                    None
                }
            })
        });

        let rs_path = rs_path.unwrap_or_else(|| panic!("No .rs file in {}", dir.display()));
        let rust_code =
            std::fs::read_to_string(&rs_path).unwrap_or_else(|e| panic!("Read error: {}", e));

        let result = transpile_r2a(case, &rust_code).unwrap();

        // Structural checks: must not be empty, must not panic
        assert!(!result.trim().is_empty(), "r2a produced empty output for {}", case);
        // Must not contain unresolved Rust syntax fragments
        assert!(
            !result.contains("/* unknown */"),
            "r2a produced 'unknown' markers for {}: {}",
            case,
            result
        );
    }

    #[test]
    fn rt_01_basics_hello() {
        roundtrip_a2r("01_basics/001_hello");
    }

    #[test]
    fn rt_01_basics_func() {
        roundtrip_a2r("01_basics/003_func");
    }

    #[test]
    fn rt_02_types_struct() {
        roundtrip_a2r("02_types/001_struct");
    }

    #[test]
    fn rt_02_types_enum() {
        roundtrip_a2r("02_types/002_enum");
    }

    #[test]
    fn rt_03_control_if_basic() {
        roundtrip_a2r("03_control_flow/001_if_basic");
    }

    #[test]
    fn rt_03_control_if_nested() {
        roundtrip_a2r("03_control_flow/002_if_nested");
    }

    #[test]
    fn rt_03_control_for_range() {
        roundtrip_a2r("03_control_flow/005_for_range");
    }

    #[test]
    fn rt_03_control_while() {
        roundtrip_a2r("03_control_flow/007_while_loop");
    }

    #[test]
    fn rt_03_control_match() {
        roundtrip_a2r("03_control_flow/008_is_match");
    }

    #[test]
    fn rt_05_expr_arithmetic() {
        roundtrip_a2r("05_expressions/001_arithmetic");
    }

    #[test]
    fn rt_11_methods() {
        roundtrip_a2r("11_methods/001_method");
    }

    #[test]
    fn rt_11_struct_methods() {
        roundtrip_a2r("11_methods/002_struct_methods");
    }

    #[test]
    fn rt_11_ext_for() {
        roundtrip_a2r("11_methods/006_ext_for");
    }

    #[test]
    fn rt_12_specs() {
        roundtrip_a2r("12_specs/001_basic_spec");
    }

    #[test]
    fn rt_12_spec() {
        roundtrip_a2r("12_specs/002_spec");
    }

    #[test]
    fn rt_12_spec_delegation() {
        roundtrip_a2r("12_specs/003_spec_delegation");
    }

    #[test]
    fn rt_13_single() {
        roundtrip_a2r("13_delegation/001_single");
    }

    #[test]
    fn rt_13_multi_spec() {
        roundtrip_a2r("13_delegation/002_multi_spec");
    }

    #[test]
    fn rt_13_multi_delegation() {
        roundtrip_a2r("13_delegation/003_multi_delegation");
    }

    // ── Phase 2: impl methods with self ──

    #[test]
    fn test_mut_self() {
        let rust_code = r#"
struct Counter {
    count: i32,
}

impl Counter {
    fn new() -> Counter {
        Counter { count: 0 }
    }
    fn increment(&mut self) {
        self.count = self.count + 1;
    }
    fn get_count(&self) -> i32 {
        self.count
    }
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("mut fn increment"), "missing mut fn in: {}", result);
        assert!(result.contains("fn get_count() int"), "missing get_count in: {}", result);
        assert!(result.contains("static fn new"), "missing static fn in: {}", result);
    }

    #[test]
    fn test_dyn_trait() {
        let rust_code = r#"
fn main() {
    let v: Vec<Box<dyn Flyer>> = vec![];
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("dyn Flyer"), "missing dyn trait comment in: {}", result);
    }

    #[test]
    fn test_impl_for() {
        let rust_code = r#"
trait Flyer {
    fn fly(&self);
}

struct Pigeon {}

impl Flyer for Pigeon {
    fn fly(&self) {
        println!("Flap Flap");
    }
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("spec Flyer"), "missing spec Flyer in: {}", result);
        assert!(result.contains("ext Pigeon for Flyer"), "missing ext for in: {}", result);
        assert!(result.contains("print"), "missing print in: {}", result);
    }

    // ── Phase 2: file-based tests ──

    #[test]
    fn test_r2a_010_mut_self() {
        test_r2a_file("04_methods/001_mut_self");
    }

    #[test]
    fn test_r2a_011_impl_for() {
        test_r2a_file("05_traits/001_impl_for");
    }

    #[test]
    fn test_r2a_012_struct_methods() {
        test_r2a_file("05_traits/002_struct_methods");
    }

    #[test]
    fn test_r2a_013_dyn_trait() {
        test_r2a_file("05_traits/003_dyn_trait");
    }

    #[test]
    fn test_r2a_014_union() {
        test_r2a_file("05_traits/004_union");
    }

    #[test]
    fn test_r2a_015_raw_pointer() {
        test_r2a_file("05_traits/005_raw_pointer");
    }

    // ── Phase 3: pattern matching, generics, Option/Result ──

    #[test]
    fn test_r2a_016_hetero_enum() {
        test_r2a_file("06_pattern_matching/001_hetero_enum");
    }

    #[test]
    fn test_r2a_017_struct_destructure() {
        test_r2a_file("06_pattern_matching/002_struct_destructure");
    }

    #[test]
    fn test_r2a_018_discriminant() {
        test_r2a_file("06_pattern_matching/003_discriminant");
    }

    #[test]
    fn test_r2a_019_generic_enum() {
        test_r2a_file("06_pattern_matching/004_generic_enum");
    }

    #[test]
    fn test_r2a_020_type_alias() {
        test_r2a_file("08_generics/001_type_alias");
    }

    #[test]
    fn test_r2a_021_generic_struct() {
        test_r2a_file("08_generics/002_generic_struct");
    }

    #[test]
    fn test_r2a_022_generic_fn() {
        test_r2a_file("08_generics/003_generic_fn");
    }

    #[test]
    fn test_r2a_023_map_type() {
        test_r2a_file("08_generics/004_map_type");
    }

    #[test]
    fn test_r2a_024_option_construct() {
        test_r2a_file("09_option_result/001_option_construct");
    }

    #[test]
    fn test_r2a_025_try_operator() {
        test_r2a_file("09_option_result/002_try_operator");
    }

    #[test]
    fn test_r2a_026_unwrap_or() {
        test_r2a_file("09_option_result/003_unwrap_or");
    }

    // ── Phase 4: collections, modules, type conversion, interop ──

    #[test]
    fn test_r2a_027_array() {
        test_r2a_file("10_collections/001_array");
    }

    #[test]
    fn test_r2a_028_method_chain() {
        test_r2a_file("10_collections/002_method_chain");
    }

    #[test]
    fn test_r2a_029_use() {
        test_r2a_file("14_modules/001_use");
    }

    #[test]
    fn test_r2a_030_pub_visibility() {
        test_r2a_file("14_modules/002_pub_visibility");
    }

    #[test]
    fn test_r2a_031_const_decl() {
        test_r2a_file("14_modules/003_const_decl");
    }

    #[test]
    fn test_r2a_032_derive_attr() {
        test_r2a_file("14_modules/004_derive_attr");
    }

    #[test]
    fn test_r2a_033_type_cast() {
        test_r2a_file("15_type_conversion/001_type_cast");
    }

    #[test]
    fn test_r2a_034_box_arc() {
        test_r2a_file("15_type_conversion/002_box_arc");
    }

    #[test]
    fn test_r2a_035_string_methods() {
        test_r2a_file("15_type_conversion/003_string_methods");
    }

    #[test]
    fn test_r2a_036_async_fn() {
        test_r2a_file("16_interop/001_async_fn");
    }

    #[test]
    fn test_r2a_037_field_attrs() {
        test_r2a_file("16_interop/002_field_attrs");
    }

    // ── Phase 3: unit tests ──

    #[test]
    fn test_generic_struct() {
        let rust_code = r#"
struct Box<T> {
    value: T,
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("type Box<T>"), "missing generic struct in: {}", result);
        assert!(result.contains("value T"), "missing generic field in: {}", result);
    }

    #[test]
    fn test_generic_enum() {
        let rust_code = r#"
enum May<T> {
    val(T),
    nil,
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("enum May<T>"), "missing generic enum in: {}", result);
        assert!(result.contains("val(T)"), "missing tuple variant in: {}", result);
    }

    #[test]
    fn test_generic_fn_with_bounds() {
        let rust_code = r#"
fn duplicate<T: Clone>(x: T) -> T {
    x.clone()
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("fn duplicate<T: Clone>"), "missing bounded generic in: {}", result);
    }

    #[test]
    fn test_enum_discriminant() {
        let rust_code = r#"
enum Coin {
    Penny = 0,
    Nickel = 1,
    Dime = 2,
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("Penny = 0"), "missing discriminant in: {}", result);
        assert!(result.contains("Nickel = 1"), "missing discriminant in: {}", result);
        assert!(result.contains("Dime = 2"), "missing discriminant in: {}", result);
    }

    #[test]
    fn test_type_alias_generic() {
        let rust_code = r#"
type IntList<T> = Vec<T>;"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("type IntList<T>"), "missing generic alias in: {}", result);
        assert!(result.contains("List<T>"), "missing Vec->List mapping in: {}", result);
    }

    // ── Phase 3: round-trip tests for groups 06, 08, 09 ──

    #[test]
    fn rt_06_enum_pattern() {
        roundtrip_a2r("06_pattern_matching/001_enum_pattern");
    }

    #[test]
    fn rt_06_struct_destructure() {
        roundtrip_a2r("06_pattern_matching/002_struct_destructure");
    }

    #[test]
    fn rt_06_empty_variant_match() {
        roundtrip_a2r("06_pattern_matching/003_empty_variant_match");
    }

    #[test]
    fn rt_06_hetero_enum() {
        roundtrip_a2r("06_pattern_matching/004_hetero_enum");
    }

    #[test]
    fn rt_06_generic_hetero_enum() {
        roundtrip_a2r("06_pattern_matching/005_generic_hetero_enum");
    }

    #[test]
    fn rt_06_enum_fn_param() {
        roundtrip_a2r("06_pattern_matching/006_enum_fn_param");
    }

    #[test]
    fn rt_06_is_in_ext() {
        roundtrip_a2r("06_pattern_matching/007_is_in_ext");
    }

    #[test]
    fn rt_08_type_alias() {
        roundtrip_a2r("08_generics/001_type_alias");
    }

    #[test]
    fn rt_08_const_generics() {
        roundtrip_a2r("08_generics/002_const_generics");
    }

    #[test]
    fn rt_08_generic_field() {
        roundtrip_a2r("08_generics/003_generic_field");
    }

    #[test]
    fn rt_08_generic_ptr_field() {
        roundtrip_a2r("08_generics/004_generic_ptr_field");
    }

    #[test]
    fn rt_08_with_constraint() {
        roundtrip_a2r("08_generics/005_with_constraint");
    }

    // Skipped: a2r generates invalid Rust (bare identifiers in struct literal with hyphens)
    #[test]
    #[ignore]
    fn rt_08_map_type() {
        roundtrip_a2r("08_generics/006_map_type");
    }

    #[test]
    fn rt_08_no_tuple_generic() {
        roundtrip_a2r("08_generics/007_no_tuple_generic");
    }

    #[test]
    fn rt_09_option() {
        roundtrip_a2r("09_option_result/001_option");
    }

    // Skipped: a2r generates Option</* unknown */> with comments inside generics (invalid Rust)
    #[test]
    #[ignore]
    fn rt_09_option_construct() {
        roundtrip_a2r("09_option_result/002_option_construct");
    }

    #[test]
    fn rt_09_null_coalesce() {
        roundtrip_a2r("09_option_result/003_null_coalesce");
    }

    #[test]
    fn rt_09_error_propagate() {
        roundtrip_a2r("09_option_result/004_error_propagate");
    }

    #[test]
    fn rt_09_question_uint() {
        roundtrip_a2r("09_option_result/005_question_uint");
    }

    #[test]
    fn rt_09_question_return_int() {
        roundtrip_a2r("09_option_result/008_question_return_int");
    }

    // ── Phase 4: round-trip tests for groups 10, 14-16 ──

    #[test]
    fn rt_10_array() {
        roundtrip_a2r("10_collections/001_array");
    }

    // Skipped: a2r generates invalid Rust (List<int, Heap>.new() is not valid syntax)
    #[test]
    #[ignore]
    fn rt_10_list_storage() {
        roundtrip_a2r("10_collections/002_list_storage");
    }

    #[test]
    fn rt_10_method_chain() {
        roundtrip_a2r("10_collections/005_method_chain");
    }

    #[test]
    fn rt_14_rust_use() {
        roundtrip_a2r("14_modules/001_rust_use");
    }

    #[test]
    fn rt_14_pub_visibility() {
        roundtrip_a2r("14_modules/003_pub_visibility");
    }

    #[test]
    fn rt_14_const_decl() {
        roundtrip_a2r("14_modules/006_const_decl");
    }

    #[test]
    fn rt_14_derive_attr() {
        roundtrip_a2r("14_modules/008_derive_attr");
    }

    // ── Plan 190: Rust stdlib use.rust tests ──

    #[test]
    fn rt_15_rust_collections() {
        roundtrip_a2r("15_rust_std/001_collections");
    }

    #[test]
    fn rt_15_rust_fs() {
        roundtrip_a2r("15_rust_std/002_fs");
    }

    #[test]
    fn rt_15_rust_sync() {
        roundtrip_a2r("15_rust_std/003_sync");
    }

    #[test]
    fn rt_15_rust_time() {
        roundtrip_a2r("15_rust_std/004_time");
    }

    #[test]
    fn rt_15_rust_path() {
        roundtrip_a2r("15_rust_std/005_path");
    }

    #[test]
    fn rt_15_rust_box_cell() {
        roundtrip_a2r("15_rust_std/006_box_cell");
    }

    #[test]
    fn rt_15_rust_env_process() {
        roundtrip_a2r("15_rust_std/007_env_process");
    }

    #[test]
    fn rt_15_rust_thread() {
        roundtrip_a2r("15_rust_std/008_thread");
    }

    #[test]
    fn rt_15_rust_serde_json() {
        roundtrip_a2r("15_rust_std/009_serde_json");
    }

    #[test]
    fn rt_15_rust_regex() {
        roundtrip_a2r("15_rust_std/010_regex");
    }

    #[test]
    fn rt_15_type_cast() {
        roundtrip_a2r("15_type_conversion/001_type_cast");
    }

    #[test]
    fn rt_15_box_arc() {
        roundtrip_a2r("15_type_conversion/004_box_arc");
    }

    #[test]
    fn rt_16_async_fn() {
        roundtrip_a2r("16_interop/001_async_fn");
    }

    // ── Phase 4: unit tests ──

    #[test]
    fn test_async_fn() {
        let rust_code = r#"
async fn fetch_data(url: &str) -> String {
    String::from("data")
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("// async"), "missing async comment in: {}", result);
        assert!(result.contains("fn fetch_data"), "missing fn in: {}", result);
    }

    #[test]
    fn test_derive_attr() {
        let rust_code = r#"
#[derive(Debug, Clone)]
struct Point {
    x: i32,
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("derive"), "missing derive comment in: {}", result);
        assert!(result.contains("type Point"), "missing struct in: {}", result);
    }

    #[test]
    fn test_field_attr() {
        let rust_code = r#"
struct User {
    #[serde(rename = "role_id")]
    role: i32,
    name: String,
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("serde"), "missing serde comment in: {}", result);
        assert!(result.contains("role int"), "missing field in: {}", result);
    }

    #[test]
    fn test_type_cast() {
        let rust_code = r#"
fn main() {
    let x: i32 = 42;
    let y: u32 = x as u32;
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains(".as("), "missing type cast in: {}", result);
    }

    #[test]
    fn test_box_arc_rc_new() {
        let rust_code = r#"
fn main() {
    let a = Box::new(42);
    let b = Arc::new("hello");
    let c = Rc::new(42);
}"#;
        let result = transpile_r2a("test", rust_code).unwrap();
        assert!(result.contains("let a = 42"), "missing Box simplification in: {}", result);
        assert!(result.contains("let b = \"hello\""), "missing Arc simplification in: {}", result);
        assert!(result.contains("let c = 42"), "missing Rc simplification in: {}", result);
    }
}
