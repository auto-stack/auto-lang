use super::{Sink, ToStrError, Trans};
use crate::ast::*;
use crate::ast::{ArrayType, Type};
use crate::parser::Parser;
use crate::scope::Meta;
use crate::universe::Universe;
use crate::AutoResult;
use auto_val::AutoStr;
use auto_val::Op;
use auto_val::StrExt;
use auto_val::{shared, Shared};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;

pub enum OutKind {
    Header,
    Source,
    Both,
    None,
}

pub enum CStyle {
    Tradition,
    Modern,
}

pub struct CTrans {
    indent: usize,
    libs: HashSet<AutoStr>,
    pub header: Vec<u8>,
    uses_bool: bool,
    name: AutoStr,
    scope: Shared<Universe>,
    last_out: OutKind,
    style: CStyle,
}

impl CTrans {
    pub fn new(name: AutoStr) -> Self {
        Self {
            indent: 0,
            uses_bool: false,
            libs: HashSet::new(),
            header: Vec::new(),
            name,
            scope: shared(Universe::default()),
            last_out: OutKind::None,
            style: CStyle::Modern,
        }
    }

    pub fn set_scope(&mut self, scope: Shared<Universe>) {
        self.scope = scope;
    }

    pub fn set_stayle(&mut self, style: CStyle) {
        self.style = style;
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        self.indent -= 1;
    }

    fn print_indent(&self, out: &mut impl Write) -> AutoResult<()> {
        for _ in 0..self.indent {
            out.write(b"    ")?;
        }
        Ok(())
    }

    fn print_indent_header(&mut self) -> AutoResult<()> {
        for _ in 0..self.indent {
            self.header.write(b"    ")?;
        }
        Ok(())
    }

    fn print_with_indent(&mut self, out: &mut impl Write, text: &str) -> AutoResult<()> {
        for _ in 0..self.indent {
            out.write(b"    ").to()?;
        }
        out.write(text.as_bytes())?;
        Ok(())
    }

    fn header_guard_start(&self, header: &mut impl Write) -> AutoResult<()> {
        match self.style {
            CStyle::Tradition => {
                let upper = self.name.to_uppercase();
                let name_bytes = upper.as_bytes();
                header.write(b"#ifndef ")?;
                header.write(name_bytes)?;
                header.write(b"_H\n#define ")?;
                header.write(name_bytes)?;
                header.write(b"_H\n\n")?;
            }
            CStyle::Modern => {
                header.write(b"#pragma once\n\n")?;
            }
        }
        Ok(())
    }

    fn header_guard_end(&self, header: &mut impl Write) -> AutoResult<()> {
        match self.style {
            CStyle::Tradition => {
                header.write(b"\n#endif\n").to()?;
            }
            CStyle::Modern => {}
        }
        Ok(())
    }
}

impl CTrans {
    pub fn code(&mut self, code: Code, sink: &mut Sink) -> AutoResult<()> {
        for (i, stmt) in code.stmts.iter().enumerate() {
            if i > 0 {
                sink.body.write(b"\n")?;
            }
            self.stmt(stmt, sink)?;
        }
        if let Some(stmt) = code.stmts.last() {
            if !stmt.is_new_block() {
                sink.body.write(b"\n")?;
            }
        }
        Ok(())
    }

    fn eos(&mut self, out: &mut impl Write) -> AutoResult<()> {
        out.write(b";").to()
    }

    fn stmt(&mut self, stmt: &Stmt, sink: &mut Sink) -> AutoResult<bool> {
        let out = &mut sink.body;
        match stmt {
            Stmt::TypeDecl(type_decl) => {
                if matches!(type_decl.kind, TypeDeclKind::CType) {
                    return Ok(false);
                }
                self.type_decl(type_decl, sink)?;
            }
            Stmt::Expr(expr) => {
                self.expr(expr, out)?;
                self.eos(out)?;
            }
            Stmt::Store(store) => {
                if matches!(store.kind, StoreKind::CVar) {
                    return Ok(false);
                }
                self.store(store, out)?;
                self.eos(out)?;
            }
            Stmt::Fn(fn_decl) => {
                // No need to generate extern C function declarations
                if matches!(fn_decl.kind, FnKind::CFunction) {
                    return Ok(false);
                }
                self.fn_decl(fn_decl, sink)?;
            }
            Stmt::For(for_stmt) => {
                self.for_stmt(for_stmt, sink)?;
            }
            Stmt::If(if_) => {
                self.if_stmt(if_, sink)?;
            }
            Stmt::Is(is_stmt) => {
                self.is_stmt(is_stmt, sink)?;
            }
            Stmt::Use(use_stmt) => {
                self.use_stmt(use_stmt, out)?;
            }
            Stmt::EnumDecl(enum_decl) => {
                self.enum_decl(enum_decl, sink)?;
            }
            Stmt::Alias(alias) => {
                self.alias(alias, out)?;
            }
            Stmt::EmptyLine(n) => {
                self.empty_line(n, out)?;
            }
            Stmt::Union(union) => {
                self.union(union, sink)?;
            }
            Stmt::Tag(tag) => {
                self.tag(tag, sink)?;
            }
            Stmt::SpecDecl(spec_decl) => {
                self.spec_decl(spec_decl, sink)?;
            }
            Stmt::Break => {
                sink.body.write(b"break;")?;
            }
            Stmt::Ext(ext) => {
                // Plan 035 Phase 5.2: Handle ext statement
                // Generate C functions for each method
                self.ext_stmt(ext, sink)?;
            }
            Stmt::Node(_node) => {
                // CONFIG mode constructs - skip in C transpilation
                // These are only used for config evaluation, not for C code generation
                return Ok(false);
            }
            _ => {
                return Err(format!("C Transpiler: unsupported statement: {:?}", stmt).into());
            }
        }
        Ok(true)
    }

    fn tag(&mut self, tag: &Tag, sink: &mut Sink) -> AutoResult<()> {
        self.tag_enum(tag, sink)?;
        self.header.write(b"\n")?;
        self.tag_struct(tag, sink)?;

        // Generate method declarations and implementations for tag methods
        for method in &tag.methods {
            // Tag methods are declared as: ReturnType Tag_Method(Tag *self, args...)
            self.tag_method_decl(tag, method, sink)?;
        }

        // Collect method info to avoid double borrow issues
        let methods: Vec<_> = tag.methods.iter().cloned().collect();

        for (i, method) in methods.iter().enumerate() {
            self.tag_method_impl(tag, &method, sink)?;
        }

        Ok(())
    }

    fn tag_method_impl(&mut self, tag: &Tag, method: &Fn, sink: &mut Sink) -> AutoResult<()> {
        // Skip C functions
        if matches!(method.kind, FnKind::CFunction) {
            return Ok(());
        }

        // Pre-compute all strings before taking mutable borrow
        let ret_type_str = if !matches!(method.ret, Type::Unknown) {
            format!("{} ", self.c_type_name(&method.ret))
        } else {
            "void ".to_string()
        };

        let method_name_str = self.format_method_name(&tag.name, &method.name);

        let mut param_strs = Vec::new();
        for param in &method.params {
            let param_type = self.c_type_name(&param.ty);
            param_strs.push(format!("{} {}", param_type, param.name));
        }

        // Write function signature
        {
            let out = &mut sink.body;
            out.write(ret_type_str.as_bytes())?;
            out.write(method_name_str.as_bytes())?;
            out.write(b"(")?;
            out.write(format!("struct {}* self", tag.name).as_bytes())?;

            if !method.params.is_empty() {
                out.write(b", ")?;
                for (i, param_str) in param_strs.iter().enumerate() {
                    if i > 0 {
                        out.write(b", ")?;
                    }
                    out.write(param_str.as_bytes())?;
                }
            }

            out.write(b") {\n")?;
        }

        // Function body (drop the borrow before calling self.body)
        self.scope.borrow_mut().enter_fn(method.name.clone());
        self.body(&method.body, sink, &method.ret, "", &method.name)?;
        self.scope.borrow_mut().exit_fn();

        // Write closing brace
        {
            let out = &mut sink.body;
            out.write(b"\n}\n").to()?;
        }

        Ok(())
    }

    fn tag_method_decl(&mut self, tag: &Tag, method: &Fn, _sink: &mut Sink) -> AutoResult<()> {
        // Pre-compute all strings before taking mutable borrow
        let ret_type_str = if !matches!(method.ret, Type::Unknown) {
            format!("{} ", self.c_type_name(&method.ret))
        } else {
            "void ".to_string()
        };

        let method_name_str = self.format_method_name(&tag.name, &method.name);

        let mut param_strs = Vec::new();
        for param in &method.params {
            let param_type = self.c_type_name(&param.ty);
            param_strs.push(format!("{} {}", param_type, param.name));
        }

        // Now take mutable borrow and write
        let out = &mut self.header;
        out.write(ret_type_str.as_bytes())?;
        out.write(method_name_str.as_bytes())?;
        out.write(b"(")?;
        out.write(format!("struct {}* self", tag.name).as_bytes())?;

        if !method.params.is_empty() {
            out.write(b", ")?;
            for (i, param_str) in param_strs.iter().enumerate() {
                if i > 0 {
                    out.write(b", ")?;
                }
                out.write(param_str.as_bytes())?;
            }
        }

        out.write(b");\n").to()?;
        Ok(())
    }

    fn tag_enum(&mut self, tag: &Tag, sink: &mut Sink) -> AutoResult<()> {
        self.header.write(b"enum ")?;
        self.header.write(format!("{}Kind", tag.name).as_bytes())?;
        self.header.write(b" {\n")?;
        self.indent();
        for field in &tag.fields {
            let mut header = std::mem::take(&mut self.header);
            self.print_indent(&mut header)?;
            self.header = header;
            self.tag_field(tag, field, sink)?;
        }
        self.dedent();
        self.header.write(b"};\n")?;
        Ok(())
    }

    fn tag_field(&mut self, tag: &Tag, field: &TagField, _sink: &mut Sink) -> AutoResult<()> {
        let out = &mut self.header;
        out.write(format!("{}_{}", tag.name.to_uppercase(), field.name.to_uppercase()).as_bytes())?;
        out.write(b",\n")?;
        Ok(())
    }

    fn tag_struct(&mut self, tag: &Tag, sink: &mut Sink) -> AutoResult<()> {
        self.header.write(b"struct ")?;
        self.header.write(tag.name.as_bytes())?;
        self.header.write(b" {\n")?;
        self.indent();
        // enam tag
        self.print_indent_header()?;
        self.header.write(b"enum ")?;
        // Type is tagName + Kind
        self.header.write(format!("{}Kind", tag.name).as_bytes())?;
        self.header.write(b" tag;\n")?;

        // union data
        self.print_indent_header()?;
        self.header.write(b"union {\n")?;
        self.indent();

        for field in &tag.fields {
            self.print_indent_header()?;
            self.tag_struct_field(field, sink)?;
        }
        self.dedent();
        self.print_indent_header()?;
        self.header.write(b"} as;\n")?;
        self.dedent();
        self.header.write(b"};\n")?;
        Ok(())
    }

    fn tag_struct_field(&mut self, field: &TagField, _sink: &mut Sink) -> AutoResult<()> {
        let out = &mut self.header;
        out.write(field.ty.unique_name().as_bytes())?;
        out.write(b" ")?;
        out.write(field.name.as_bytes())?;
        out.write(b";\n")?;
        Ok(())
    }

    fn union(&mut self, union: &Union, sink: &mut Sink) -> AutoResult<()> {
        self.header.write(b"union ")?;
        self.header.write(union.name.as_bytes())?;
        self.header.write(b" {\n")?;
        self.indent();
        for field in &union.fields {
            let mut header = std::mem::take(&mut self.header);
            self.print_indent(&mut header)?;
            self.header = header;
            self.union_field(field, sink)?;
        }
        self.dedent();
        self.header.write(b"};\n")?;
        Ok(())
    }

    fn union_field(&mut self, field: &UnionField, _sink: &mut Sink) -> AutoResult<()> {
        let out = &mut self.header;
        out.write(field.ty.unique_name().as_bytes())?;
        out.write(b" ")?;
        out.write(field.name.as_bytes())?;
        out.write(b";\n")?;
        Ok(())
    }

    fn empty_line(&mut self, n: &usize, out: &mut impl Write) -> AutoResult<()> {
        // empty_line itself is a stmt, and we have a \n for one stme already
        for _ in 0..*n - 1 {
            match self.last_out {
                OutKind::Header => {
                    self.header.write(b"\n")?;
                }
                OutKind::Source => {
                    out.write(b"\n")?;
                }
                OutKind::Both => {
                    self.header.write(b"\n")?;
                    out.write(b"\n")?;
                }
                OutKind::None => {}
            }
        }
        Ok(())
    }

    fn alias(&mut self, alias: &Alias, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"#define ")?;
        out.write(alias.alias.as_bytes())?;
        out.write(b" ")?;
        out.write(alias.target.as_bytes())?;
        out.write(b"\n")?;
        Ok(())
    }

    fn enum_decl(&mut self, enum_decl: &EnumDecl, _sink: &mut Sink) -> AutoResult<()> {
        let mut out = std::mem::take(&mut self.header);
        out.write(b"enum ")?;
        out.write(enum_decl.name.as_bytes())?;
        out.write(b" {\n")?;
        for item in enum_decl.items.iter() {
            println!("Enum Item: {}", item.name);
            out.write(b"    ")?;
            out.write(format!("{}_", enum_decl.name.to_uppercase()).as_bytes())?;
            out.write(format!("{}", item.name).as_bytes())?;
            out.write(b" = ")?;
            out.write(format!("{}", item.value).as_bytes())?;
            out.write(b",\n")?;
        }
        out.write(b"};\n")?;
        self.header = out;

        self.last_out = OutKind::Header;
        Ok(())
    }

    fn format_method_name(&self, type_name: &str, method_name: &str) -> String {
        let camel = AutoStr::from(method_name).to_camel();
        format!("{}_{}", type_name, camel)
    }

    fn method_name(
        &mut self,
        type_name: &str,
        method_name: &str,
        out: &mut impl Write,
    ) -> AutoResult<()> {
        let camel = AutoStr::from(method_name).to_camel();
        out.write(format!("{}_{}", type_name, camel).as_bytes())?;
        Ok(())
    }

    fn type_decl(&mut self, type_decl: &TypeDecl, sink: &mut Sink) -> AutoResult<()> {
        let mut out = std::mem::take(&mut self.header);
        // write type body
        out.write(b"struct ")?;
        out.write(type_decl.name.as_bytes())?;
        out.write(b" {\n")?; // TODO: no newline for short decls
        for field in type_decl.members.iter() {
            out.write(b"    ")?;
            out.write(self.c_type_name(&field.ty).as_bytes())?;
            out.write(b" ")?;
            out.write(field.name.as_bytes())?;
            out.write(b";\n")?;
        }

        // Add delegation members
        for delegation in type_decl.delegations.iter() {
            out.write(b"    ")?;
            out.write(self.c_type_name(&delegation.member_type).as_bytes())?;
            out.write(b" ")?;
            out.write(delegation.member_name.as_bytes())?;
            out.write(b";\n")?;
        }

        out.write(b"};\n")?;

        // write methods
        if !type_decl.methods.is_empty() {
            out.write(b"\n")?;
        }
        for method in type_decl.methods.iter() {
            out.write(method.ret.unique_name().as_bytes())?;
            out.write(b" ")?;
            // Note add prefix to method name
            self.method_name(&type_decl.name, &method.name, &mut out)?;
            // out.write(method.name.as_bytes())?;
            out.write(b"(")?;
            // self
            out.write(b"struct ")?;
            out.write(type_decl.name.as_bytes())?;
            out.write(b" *self")?;
            if !method.params.is_empty() {
                out.write(b", ")?;
            }
            out.write(
                method
                    .params
                    .iter()
                    .map(|p| p.ty.unique_name())
                    .collect::<Vec<_>>()
                    .join(", ")
                    .as_bytes(),
            )?;
            out.write(b");\n")?;
        }

        // Generate delegation wrapper method declarations
        for delegation in type_decl.delegations.iter() {
            let spec_name = delegation.spec_name.clone();
            if let Some(meta) = self.scope.borrow().lookup_meta(spec_name.as_str()) {
                if let Meta::Spec(spec_decl) = meta.as_ref() {
                    for spec_method in spec_decl.methods.iter() {
                        // Return type
                        out.write(self.c_type_name(&spec_method.ret).as_bytes())?;
                        out.write(b" ")?;

                        // Method name
                        out.write(type_decl.name.as_bytes())?;
                        out.write(b"_")?;
                        out.write(spec_method.name.as_bytes())?;
                        out.write(b"(struct ")?;
                        out.write(type_decl.name.as_bytes())?;
                        out.write(b" *self")?;

                        // Parameters
                        for param in &spec_method.params {
                            out.write(b", ")?;
                            out.write(self.c_type_name(&param.ty).as_bytes())?;
                            out.write(b" ")?;
                            out.write(param.name.as_bytes())?;
                        }

                        out.write(b");\n")?;
                    }
                }
            }
        }

        self.header = out;

        for method in type_decl.methods.iter() {
            let out = &mut sink.body;
            out.write(method.ret.unique_name().as_bytes())?;
            out.write(b" ")?;
            self.method_name(&type_decl.name, &method.name, out)?;
            // out.write(method.name.as_bytes())?;
            out.write(b"(")?;
            // self
            out.write(b"struct ")?;
            out.write(type_decl.name.as_bytes())?;
            out.write(b" *self")?;
            if !method.params.is_empty() {
                out.write(b", ")?;
            }
            out.write(
                method
                    .params
                    .iter()
                    .map(|p| p.ty.unique_name())
                    .collect::<Vec<_>>()
                    .join(", ")
                    .as_bytes(),
            )?;
            out.write(b") ")?;
            // method body
            self.body(&method.body, sink, &method.ret, "", "")?;
            sink.body.write(b"\n")?;
        }

        // Generate delegation wrapper method implementations
        for delegation in type_decl.delegations.iter() {
            let spec_name = delegation.spec_name.clone();
            let member_type_name = delegation.member_type.unique_name();
            let member_name = delegation.member_name.clone();
            if let Some(meta) = self.scope.borrow().lookup_meta(spec_name.as_str()) {
                if let Meta::Spec(spec_decl) = meta.as_ref() {
                    for spec_method in spec_decl.methods.iter() {
                        let out = &mut sink.body;

                        // Return type
                        let ret_type_name = self.c_type_name(&spec_method.ret);
                        out.write(ret_type_name.as_bytes())?;
                        out.write(b" ")?;

                        // Method name
                        out.write(type_decl.name.as_bytes())?;
                        out.write(b"_")?;
                        out.write(spec_method.name.as_bytes())?;
                        out.write(b"(struct ")?;
                        out.write(type_decl.name.as_bytes())?;
                        out.write(b" *self")?;

                        // Parameters
                        for param in &spec_method.params {
                            out.write(b", ")?;
                            out.write(self.c_type_name(&param.ty).as_bytes())?;
                            out.write(b" ")?;
                            out.write(param.name.as_bytes())?;
                        }

                        out.write(b") {\n    ")?;

                        // Call the delegated member's method
                        if !matches!(spec_method.ret, Type::Void) {
                            out.write(b"return ")?;
                        }

                        out.write(member_type_name.as_bytes())?;
                        out.write(b"_")?;
                        out.write(spec_method.name.as_bytes())?;
                        out.write(b"(&self->")?;
                        out.write(member_name.as_bytes())?;

                        // Forward parameters
                        for param in &spec_method.params {
                            out.write(b", ")?;
                            out.write(param.name.as_bytes())?;
                        }

                        out.write(b");\n}\n")?;
                    }
                }
            }
        }

        // Generate vtable instances for each spec this type implements
        let spec_decls: Vec<_> = type_decl
            .specs
            .iter()
            .filter_map(|spec_name| {
                if let Some(meta) = self.scope.borrow().lookup_meta(spec_name.as_str()) {
                    if let Meta::Spec(spec_decl) = meta.as_ref() {
                        Some(spec_decl.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        for spec_decl in spec_decls {
            self.type_vtable_instance(type_decl, &spec_decl, sink)?;
        }

        if type_decl.members.len() > 0 || !type_decl.delegations.is_empty() {
            self.last_out = OutKind::Both;
        } else {
            self.last_out = OutKind::Header;
        }
        Ok(())
    }

    fn spec_decl(&mut self, spec_decl: &SpecDecl, _sink: &mut Sink) -> AutoResult<()> {
        // Generate vtable struct for the spec
        let mut header = std::mem::take(&mut self.header);

        // Write vtable struct definition
        header.write(b"typedef struct ")?;
        header.write(spec_decl.name.as_bytes())?;
        header.write(b"_vtable {\n")?;
        self.indent();

        for method in &spec_decl.methods {
            self.print_indent(&mut header)?;
            header.write(b"void (*")?;
            header.write(method.name.as_bytes())?;
            header.write(b")(")?;

            // First parameter is always self pointer
            header.write(b"void *self")?;

            // Add remaining parameters
            for param in method.params.iter() {
                header.write(b", ")?;
                header.write(self.c_type_name(&param.ty).as_bytes())?;
                header.write(b" ")?;
                header.write(param.name.as_bytes())?;
            }

            header.write(b");\n")?;
        }

        self.dedent();
        header.write(b"} ")?;
        header.write(spec_decl.name.as_bytes())?;
        header.write(b"_vtable;\n\n")?;

        self.header = header;
        self.last_out = OutKind::Header;
        Ok(())
    }

    fn type_vtable_instance(
        &mut self,
        type_decl: &TypeDecl,
        spec_decl: &SpecDecl,
        sink: &mut Sink,
    ) -> AutoResult<()> {
        // Generate vtable instance
        let out = &mut sink.body;
        out.write(spec_decl.name.as_bytes())?;
        out.write(b"_vtable ")?;
        out.write(type_decl.name.as_bytes())?;
        out.write(b"_")?;
        out.write(spec_decl.name.as_bytes())?;
        out.write(b"_vtable = {\n")?;
        self.indent();

        for method in spec_decl.methods.iter() {
            self.print_indent(out)?;
            out.write(b".")?;
            out.write(method.name.as_bytes())?;
            out.write(b" = ")?;

            // Function pointer to the type's method implementation
            // Use method_name() helper for consistent camelCase naming
            self.method_name(&type_decl.name, &method.name, out)?;

            // Add comma if not the last method
            // Note: we can't easily check if we're at the last item in a for loop
            // without collecting into a Vec first, so we'll always add a newline
            out.write(b"\n")?;
        }

        self.dedent();
        out.write(b"};\n\n")?;
        Ok(())
    }

    fn use_stmt(&mut self, use_stmt: &Use, _out: &mut impl Write) -> AutoResult<()> {
        match use_stmt.kind {
            UseKind::Auto => {
                let path = use_stmt.paths.join("/");
                self.libs.insert(format!("\"{}.h\"", path).into());
            }
            UseKind::C => {
                for path in use_stmt.paths.iter() {
                    if !self.libs.contains(path) {
                        self.libs.insert(path.clone());
                    }
                }
            }
            UseKind::Rust => {
                // do nothing
            }
        }
        Ok(())
    }

    fn float(&mut self, _f: &f64, txt: &str, out: &mut impl Write) -> AutoResult<()> {
        out.write_all(txt.as_bytes()).to()
    }

    fn dot(&mut self, lhs: &Expr, rhs: &Expr, out: &mut impl Write) -> AutoResult<()> {
        match lhs {
            Expr::Ident(ident) => {
                if ident == "self" {
                    out.write(b"self->")?;
                    self.expr(rhs, out)?;
                    return Ok(());
                }
                let ty = self.lookup_type(ident);
                match ty {
                    Type::Enum(_) => match rhs {
                        Expr::Ident(rid) => {
                            out.write(
                                format!("{}_{}", ident.to_uppercase(), rid.to_uppercase())
                                    .as_bytes(),
                            )?;
                            return Ok(());
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            _ => {}
        }
        // if rhs is ptr or tgt
        match rhs {
            Expr::Ident(id) => match id.as_str() {
                "ptr" => {
                    out.write(b"&").to()?;
                    self.expr(lhs, out)?;
                }
                "tgt" => {
                    out.write(b"*").to()?;
                    self.expr(lhs, out)?;
                }
                _ => {
                    self.expr(lhs, out)?;
                    out.write(format!(".").as_bytes()).to()?;
                    self.expr(rhs, out)?;
                }
            },
            _ => {
                println!("got {:?}", rhs);
                self.expr(lhs, out)?;
                out.write(format!(".").as_bytes()).to()?;
                self.expr(rhs, out)?;
            }
        }
        Ok(())
    }

    fn expr(&mut self, expr: &Expr, out: &mut impl Write) -> AutoResult<()> {
        match expr {
            Expr::Int(i) => out.write_all(i.to_string().as_bytes()).to(),
            Expr::Bina(lhs, op, rhs) => {
                match op {
                    Op::Range => self.range("i", lhs, rhs, out)?,
                    _ => match op {
                        Op::Dot => {
                            self.dot(lhs, rhs, out)?;
                        }
                        _ => {
                            self.expr(lhs, out)?;
                            _ = out.write(format!(" {} ", op.op()).as_bytes()).to()?;
                            self.expr(rhs, out)?
                        }
                    },
                }
                Ok(())
            }
            Expr::Unary(op, expr) => {
                // Plan 052: Unary operators - handle address-of and dereference
                let op_str = match op {
                    Op::Add => "&",  // Unary & for address-of
                    Op::Mul => "*",  // Unary * for dereference
                    _ => op.op(),
                };
                out.write(format!("{}", op_str).as_bytes()).to()?;
                self.expr(expr, out)?;
                Ok(())
            }
            Expr::Ident(name) => self.ident(name, out),
            Expr::GenName(name) => out.write(name.as_bytes()).to(),
            Expr::Str(s) => out.write_all(format!("\"{}\"", s).as_bytes()).to(),
            Expr::CStr(s) => out.write_all(format!("\"{}\"", s).as_bytes()).to(),
            Expr::FStr(fs) => self.fstr(fs, out),
            Expr::Bool(b) => {
                self.uses_bool = true;
                out.write_all(if *b { b"true" } else { b"false" }).to()
            }
            Expr::Char(ch) => self.char(ch, out),
            Expr::Call(call) => self.call(call, out),
            Expr::Array(array) => self.array(array, out),
            Expr::Float(f, t) => self.float(f, t, out),
            Expr::Double(d, t) => self.float(d, t, out),
            Expr::Index(arr, idx) => self.index(arr, idx, out),
            Expr::Node(nd) => self.node(nd, out),
            Expr::Pair(pair) => self.pair(pair, out),
            Expr::Cover(cover) => self.cover(cover, out),
            Expr::Null => self.null(out),
            Expr::Nil => self.nil(out),
            // Borrow expressions (Phase 3)
            // C doesn't have borrow checking, so we generate pointer references
            Expr::View(e) => {
                // Immutable borrow: generate pointer (&)
                out.write_all(b"&(").to()?;
                self.expr(e, out)?;
                out.write_all(b")").to()
            }
            Expr::Mut(e) => {
                // Mutable borrow: generate pointer (&)
                // Note: C doesn't distinguish const from mut at the pointer level
                // The borrow checker (AutoLang compiler) ensures safety
                out.write_all(b"&(").to()?;
                self.expr(e, out)?;
                out.write_all(b")").to()
            }
            Expr::Take(e) => {
                // Move semantics: in C, this is just the value itself
                // The borrow checker ensures the source isn't used again
                self.expr(e, out)
            }
            // May type operators (Phase 1b.3)
            Expr::NullCoalesce(left, right) => {
                // Null-coalescing operator: left ?? right
                // In C, we use ternary operator: (left_is_some ? left_value : right)
                // For May types: (_tmp.tag == May_Val ? _tmp.data.val : right)
                self.expr(left, out)?;
                out.write_all(b" != NULL ? ")?;
                self.expr(left, out)?;
                out.write_all(b" : ")?;
                self.expr(right, out)
            }
            Expr::ErrorPropagate(expr) => {
                // Error propagation operator: expression.?
                // For May types, this unwraps the value if present
                // In C: (_tmp.tag == May_Val ? _tmp.data.val : return early)
                // For now, just emit the expression (TODO: implement proper early return)
                self.expr(expr, out)
            }
            // Plan 056: Dot expression for field access
            Expr::Dot(object, field) => {
                // Check if this is an enum access: Enum.Value -> ENUM_VALUE
                if let Expr::Ident(type_name) = object.as_ref() {
                    // Check if type_name is an enum
                    if self.is_enum_type(type_name) {
                        // Generate C enum syntax: COLOR_BLUE instead of Color.BLUE
                        let enum_constant = format!("{}_{}", type_name, field).to_uppercase();
                        out.write_all(enum_constant.as_bytes())?;
                        return Ok(());
                    }

                    // Special case for self: use self-> instead of self.
                    if type_name == "self" {
                        out.write_all(b"self->")?;
                        out.write_all(field.as_bytes())?;
                        return Ok(());
                    }

                    // Special cases for pointer operations: ptr and tgt
                    match field.as_str() {
                        "ptr" => {
                            // x.ptr -> &x
                            out.write_all(b"&")?;
                            self.expr(object, out)?;
                            return Ok(());
                        }
                        "tgt" => {
                            // y.tgt -> *y
                            out.write_all(b"*")?;
                            self.expr(object, out)?;
                            return Ok(());
                        }
                        _ => {}
                    }
                }

                // Regular field access: object.field
                self.expr(object, out)?;
                out.write_all(b".")?;
                out.write_all(field.as_bytes())?;
                Ok(())
            }
            _ => Err(format!("C Transpiler: unsupported expression: {}", expr).into()),
        }
    }

    fn char(&mut self, ch: &char, out: &mut impl Write) -> AutoResult<()> {
        if *ch == '\n' {
            out.write_all(b"'\\n'").to()
        } else if *ch == '\t' {
            out.write_all(b"'\\t'").to()
        } else {
            out.write_all(format!("'{}'", ch).as_bytes()).to()
        }
    }

    fn fstr(&mut self, fs: &FStr, out: &mut impl Write) -> AutoResult<()> {
        for p in &fs.parts {
            match p {
                Expr::Str(s) => {
                    out.write_all(format!("\"{}\"", s.replace("\"", "\\\"")).as_bytes())?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn null(&mut self, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"NULL")?;
        Ok(())
    }

    fn nil(&mut self, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"NULL")?;
        Ok(())
    }

    fn ident(&mut self, name: &AutoStr, out: &mut impl Write) -> AutoResult<()> {
        // if ident is Uncover
        let meta = self.lookup_meta(name);
        let Some(meta) = meta else {
            // TODO: check all names, include B in A.B
            out.write(name.as_bytes())?;
            return Ok(());
        };
        match meta.as_ref() {
            Meta::Store(store) => match &store.expr {
                Expr::Uncover(un) => {
                    out.write(format!("{}.as.{}", un.src, un.cover.tag).as_bytes())?;
                    return Ok(());
                }
                _ => {}
            },
            _ => {}
        }

        out.write(name.as_bytes())?;
        Ok(())
    }

    fn cover(&mut self, cover: &Cover, out: &mut impl Write) -> AutoResult<()> {
        let Cover::Tag(c) = cover;
        let typ = self.lookup_type(&c.kind);
        let Type::Tag(t) = typ else {
            return Err(format!("C Transpiler: unsupported cover type: {}", typ).into());
        };
        let enum_name = t.borrow().enum_name(&c.tag);
        out.write(enum_name.as_bytes())?;
        Ok(())
    }

    fn key(&mut self, key: &Key, out: &mut impl Write) -> AutoResult<()> {
        out.write(format!(".{}", key.to_astr()).as_bytes())?;
        Ok(())
    }

    fn pair(&mut self, pair: &Pair, out: &mut impl Write) -> AutoResult<()> {
        self.key(&pair.key, out)?;
        out.write(b" = ")?;
        self.expr(&pair.value, out)?;
        Ok(())
    }

    fn node_arg(
        &mut self,
        typ: &Type,
        arg: &Arg,
        idx: usize,
        out: &mut impl Write,
    ) -> AutoResult<()> {
        let Type::User(type_decl) = typ else {
            return Err(format!("Type is not a user type for node: {}", typ).into());
        };
        match arg {
            Arg::Pos(expr) => {
                if let Some(f) = type_decl.members.get(idx) {
                    out.write(b".")?;
                    out.write(f.name.as_bytes())?;
                    out.write(b" = ")?;
                    self.expr(expr, out)?;
                } else {
                    return Err(
                        format!("Field [{}] not found for type: {}", idx, type_decl.name).into(),
                    );
                };
            }
            Arg::Name(n) => {
                let Some(f) = type_decl.find_member(n) else {
                    return Err(
                        format!("Field {} not found for type: {}", n, type_decl.name).into(),
                    );
                };
                // named arg is actually an identifier
                out.write(b".")?;
                out.write(f.name.as_bytes())?;
                out.write(b" = ")?;
                let ident = Expr::Ident(n.clone());
                self.expr(&ident, out)?;
            }
            Arg::Pair(k, v) => {
                let Some(f) = type_decl.find_member(k) else {
                    return Err(
                        format!("Field {} not found for type: {}", k, type_decl.name).into(),
                    );
                };
                out.write(b".")?;
                out.write(f.name.as_bytes())?;
                out.write(b" = ")?;
                self.expr(v, out)?;
            }
        }
        Ok(())
    }

    fn node(&mut self, node: &Node, out: &mut impl Write) -> AutoResult<()> {
        // lookup type meta and find field name for each arg
        let Some(typ) = self.scope.borrow().lookup_ident_type(&node.name) else {
            return Err(format!("Type not found for node: {}", node.name).into());
        };

        // Type validation for struct initialization
        if let Type::User(type_decl) = &typ {
            // Validate args (named arguments)
            for arg in &node.args.args {
                if let Arg::Pair(key, value_expr) = arg {
                    // Find the field declaration
                    let field = type_decl
                        .members
                        .iter()
                        .find(|m| &m.name == key)
                        .ok_or_else(|| {
                            format!("Field '{}' not found in type '{}'", key, type_decl.name)
                        })?;

                    // Get the expected type from the field declaration
                    let expected_type = &field.ty;

                    // Infer the type of the value expression
                    let value_type = self.infer_literal_type(value_expr);

                    // Check if types match
                    if !self.types_compatible(&value_type, expected_type) {
                        return Err(format!(
                            "Type mismatch: field '{}' declared as '{}' but initialized with '{}' value",
                            key, expected_type, value_type
                        ).into());
                    }
                }
            }

            // Validate body (field: value pairs in object literal)
            for stmt in &node.body.stmts {
                if let Stmt::Expr(expr) = stmt {
                    if let Expr::Pair(pair) = expr {
                        let field_name = pair.key.to_astr();
                        // Find the field declaration
                        let field = type_decl
                            .members
                            .iter()
                            .find(|m| &m.name == &field_name)
                            .ok_or_else(|| {
                                format!(
                                    "Field '{}' not found in type '{}'",
                                    field_name, type_decl.name
                                )
                            })?;

                        // Get the expected type from the field declaration
                        let expected_type = &field.ty;

                        // Infer the type of the value expression
                        let value_type = self.infer_literal_type(&pair.value);

                        // Check if types match
                        if !self.types_compatible(&value_type, expected_type) {
                            return Err(format!(
                                "Type mismatch: field '{}' declared as '{}' but initialized with '{}' value",
                                field_name, expected_type, value_type
                            ).into());
                        }
                    }
                }
            }
        }

        out.write(b"{")?;
        // translate args to pairs in body
        for (i, arg) in node.args.args.iter().enumerate() {
            if i > 0 {
                out.write(b", ")?;
            }
            self.node_arg(&typ, arg, i, out)?;
        }
        // out.write(node.name.as_bytes())?;
        for (i, stmt) in node.body.stmts.iter().enumerate() {
            if i > 0 {
                out.write(b", ")?;
            }
            match stmt {
                Stmt::Expr(expr) => self.expr(expr, out)?,
                _ => {
                    return Err(format!(
                        "C Transpiler: unsupported statement in node body: {}",
                        stmt
                    )
                    .into())
                }
            }
        }
        out.write(b"}")?;
        Ok(())
    }

    fn index(&mut self, arr: &Box<Expr>, idx: &Box<Expr>, out: &mut impl Write) -> AutoResult<()> {
        // Check if this is a slice operation (range index)
        if let Expr::Range(ref range) = **idx {
            return self.slice(arr, range, out);
        }

        // Regular index operation
        self.expr(arr, out)?;
        out.write(b"[")?;
        self.expr(idx, out)?;
        out.write(b"]")?;
        Ok(())
    }

    /// Generate C code for slice operations
    ///
    /// For now, generates a call to a helper function that performs the slice
    /// TODO: Generate inline slice code for better performance
    fn slice(&mut self, arr: &Box<Expr>, range: &Range, out: &mut impl Write) -> AutoResult<()> {
        // Write array expression
        self.expr(arr, out)?;

        // Write slice notation as comment (C doesn't have native slice syntax)
        out.write(b"/* [")?;
        self.expr(&range.start, out)?;
        if range.eq {
            out.write(b"..=")?;
        } else {
            out.write(b"..")?;
        }
        self.expr(&range.end, out)?;
        out.write(b"] */")?;

        // For now, just generate the array expression
        // TODO: Implement actual slice code generation
        Ok(())
    }

    /// Plan 035 Phase 5.2: Generate C functions for ext statement
    /// C doesn't have extension methods, so we generate regular functions
    /// with names like "TypeName_method_name"
    fn ext_stmt(&mut self, ext: &Ext, sink: &mut Sink) -> AutoResult<()> {
        for method in &ext.methods {
            // Create a modified Fn for C generation
            let mut c_method = method.clone();

            // Change function name to "TypeName_method_name" format
            c_method.name = format!("{}_{}", ext.target, method.name).into();

            // For instance methods, add self as first parameter
            if !method.is_static {
                // Convert type name to Type enum
                let self_type = self.name_to_type(&ext.target);

                let self_param: Param = Param::new("self".into(), self_type, None);
                c_method.params.insert(0, self_param);
            }

            // Generate the function declaration
            self.fn_decl(&c_method, sink)?;
        }

        Ok(())
    }

    /// Convert type name to Type enum for built-in types
    fn name_to_type(&self, name: &AutoStr) -> Type {
        match name.as_str() {
            "int" => Type::Int,
            "uint" => Type::Uint,
            "byte" => Type::Byte,
            "float" => Type::Float,
            "double" => Type::Double,
            "bool" => Type::Bool,
            "char" => Type::Char,
            "str" => Type::Str(0), // Size unknown at compile time
            "cstr" => Type::CStr,
            // For user-defined types, we'd need to lookup TypeDecl
            // For now, use Unknown as fallback
            _ => Type::Unknown,
        }
    }

    fn fn_decl(&mut self, fn_decl: &Fn, sink: &mut Sink) -> AutoResult<()> {
        let out = &mut sink.body;
        // header
        let mut header = Vec::new();
        self.fn_sig(&fn_decl, &mut header)?;
        if fn_decl.name != "main" {
            self.header.extend(header);
            self.header.write(b";\n").to()?;
        }

        // source
        if matches!(fn_decl.kind, FnKind::CFunction) {
            // add "extern"
            out.write(b"extern ")?;
        }

        // function signature
        self.fn_sig(&fn_decl, out)?;

        // function body
        match fn_decl.kind {
            // C Functin Decl has no body
            FnKind::CFunction => {
                sink.body.write(b";")?;
            }
            _ => {
                out.write(b" ").to()?;
                self.scope.borrow_mut().enter_fn(fn_decl.name.clone());
                if fn_decl.name == "main" {
                    self.body(&fn_decl.body, sink, &Type::Int, "", &fn_decl.name)?;
                } else {
                    self.body(&fn_decl.body, sink, &fn_decl.ret, "", &fn_decl.name)?;
                }
                self.scope.borrow_mut().exit_fn();
            }
        }

        sink.body.write(b"\n")?;
        Ok(())
    }

    fn fn_sig(&mut self, fn_decl: &Fn, out: &mut impl Write) -> AutoResult<()> {
        // special: main
        // TODO: main with args
        if fn_decl.name == "main" {
            out.write(b"int main(void)").to()?;
            return Ok(());
        }
        // return type
        // Check if return type is array or slice - C can't return them by value
        let ret_is_array = matches!(fn_decl.ret, Type::Array(_) | Type::Slice(_));

        if !matches!(fn_decl.ret, Type::Unknown) {
            if ret_is_array {
                // For array/slice returns, return pointer to element type instead
                match &fn_decl.ret {
                    Type::Array(array_type) => {
                        let elem_type = self.c_type_name(&array_type.elem);
                        out.write(format!("{}* ", elem_type).as_bytes()).to()?;
                    }
                    Type::Slice(slice_type) => {
                        let elem_type = self.c_type_name(&slice_type.elem);
                        out.write(format!("{}* ", elem_type).as_bytes()).to()?;
                    }
                    _ => {}
                }
            } else {
                out.write(format!("{} ", self.c_type_name(&fn_decl.ret)).as_bytes())
                    .to()?;
            }
        } else {
            out.write(b"void ").to()?;
        }
        // name
        let name = fn_decl.name.clone();
        out.write(name.as_bytes()).to()?;
        // params
        out.write(b"(").to()?;

        // Build parameter list
        let mut params_vec = Vec::new();

        // Add output size parameter for array returns
        if ret_is_array {
            params_vec.push("int* out_size".to_string());
        }

        // Add existing parameters
        if !fn_decl.params.is_empty() {
            let params = fn_decl
                .params
                .iter()
                .map(|p| format!("{} {}", self.c_type_name(&p.ty), p.name))
                .collect::<Vec<_>>();
            params_vec.extend(params);
        }

        // Write parameters
        if params_vec.is_empty() {
            out.write(b"void").to()?;
        } else {
            out.write(params_vec.join(", ").as_bytes()).to()?;
        }

        for p in fn_decl.params.iter() {
            if matches!(p.ty, Type::Bool) {
                self.uses_bool = true;
            }
        }
        out.write(b")").to()?;

        Ok(())
    }

    fn body(
        &mut self,
        body: &Body,
        sink: &mut Sink,
        ret_type: &Type,
        insert: &str,
        fn_name: &str,
    ) -> AutoResult<()> {
        let has_return = !matches!(ret_type, Type::Void | Type::Unknown { .. });
        let ret_is_array = matches!(ret_type, Type::Array(_) | Type::Slice(_));

        self.scope.borrow_mut().enter_scope();
        sink.body.write(b"{\n")?;
        self.indent();
        if !insert.is_empty() {
            self.print_indent(&mut sink.body)?;
            sink.body.write(insert.as_bytes())?;
        }
        for (i, stmt) in body.stmts.iter().enumerate() {
            if !matches!(stmt, Stmt::EmptyLine(_)) {
                self.print_indent(&mut sink.body)?;
            }
            if i < body.stmts.len() - 1 {
                self.stmt(stmt, sink)?;
                sink.body.write(b"\n")?;
            } else {
                // last stmt
                if has_return {
                    if self.is_returnable(stmt) {
                        // Check if this is an array/slice literal return
                        if ret_is_array {
                            if let Stmt::Expr(Expr::Array(arr)) = stmt {
                                // Generate static array and return pointer
                                match ret_type {
                                    Type::Array(array_type) => {
                                        let elem_type = self.c_type_name(&array_type.elem);
                                        // Use actual array length if type says 0, otherwise use type's length
                                        let len = if array_type.len == 0 {
                                            arr.len()
                                        } else {
                                            array_type.len
                                        };
                                        let temp_name = format!("_static_{}", fn_name);

                                        // Declare static array
                                        self.print_indent(&mut sink.body)?;
                                        sink.body
                                            .write(
                                                format!(
                                                    "static {} {}[] = {{",
                                                    elem_type, temp_name
                                                )
                                                .as_bytes(),
                                            )
                                            .to()?;

                                        // Write array elements
                                        for (j, elem) in arr.iter().enumerate() {
                                            if j > 0 {
                                                sink.body.write(b", ").to()?;
                                            }
                                            self.expr(elem, &mut sink.body)?;
                                        }
                                        sink.body.write(b"};\n").to()?;

                                        // Set out_size and return pointer
                                        self.print_indent(&mut sink.body)?;
                                        sink.body
                                            .write(format!("*out_size = {};\n", len).as_bytes())
                                            .to()?;
                                        self.print_indent(&mut sink.body)?;
                                        sink.body
                                            .write(format!("return {};\n", temp_name).as_bytes())
                                            .to()?;
                                    }
                                    Type::Slice(slice_type) => {
                                        let elem_type = self.c_type_name(&slice_type.elem);
                                        let len = arr.len();
                                        let temp_name = format!("_static_{}", fn_name);

                                        // Declare static array
                                        self.print_indent(&mut sink.body)?;
                                        sink.body
                                            .write(
                                                format!(
                                                    "static {} {}[] = {{",
                                                    elem_type, temp_name
                                                )
                                                .as_bytes(),
                                            )
                                            .to()?;

                                        // Write array elements
                                        for (j, elem) in arr.iter().enumerate() {
                                            if j > 0 {
                                                sink.body.write(b", ").to()?;
                                            }
                                            self.expr(elem, &mut sink.body)?;
                                        }
                                        sink.body.write(b"};\n").to()?;

                                        // Set out_size and return pointer
                                        self.print_indent(&mut sink.body)?;
                                        sink.body
                                            .write(format!("*out_size = {};\n", len).as_bytes())
                                            .to()?;
                                        self.print_indent(&mut sink.body)?;
                                        sink.body
                                            .write(format!("return {};\n", temp_name).as_bytes())
                                            .to()?;
                                    }
                                    _ => {
                                        sink.body.write(b"return ")?;
                                    }
                                }
                            } else {
                                sink.body.write(b"return ")?;
                            }
                        } else {
                            sink.body.write(b"return ")?;
                        }
                    }
                }

                // Skip the statement if we already handled the array/slice return above
                if !(ret_is_array && matches!(stmt, Stmt::Expr(Expr::Array(_)))) {
                    self.stmt(stmt, sink)?;
                    sink.body.write(b"\n")?;
                }

                if has_return && !self.is_returnable(stmt) && !ret_is_array {
                    match ret_type {
                        Type::Void | Type::Unknown { .. } => {}
                        _ => {
                            self.print_indent(&mut sink.body)?;
                            sink.body.write(
                                format!("return {};\n", ret_type.default_value()).as_bytes(),
                            )?;
                        }
                    }
                }
            }
        }
        self.dedent();
        self.print_indent(&mut sink.body)?;
        sink.body.write(b"}")?;
        self.scope.borrow_mut().exit_scope();
        Ok(())
    }

    fn c_type_name(&self, ty: &Type) -> String {
        match ty {
            Type::Int => "int".to_string(),
            Type::Uint => "unsigned int".to_string(),
            Type::Float => "float".to_string(),
            Type::Double => "double".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Str(_) => "char*".to_string(),
            Type::CStr => "char*".to_string(),
            Type::Array(array_type) => {
                let elem_type = &array_type.elem;
                let len = array_type.len;
                format!("{}[{}]", self.c_type_name(elem_type), len)
            }
            Type::RuntimeArray(rta) => {
                // Plan 052: Runtime arrays transpile to pointers in C
                // Since size is determined at runtime, we use pointer syntax
                // E.g., [size]int -> int* (allocated at runtime)
                let elem_type = self.c_type_name(&rta.elem);
                format!("{}*", elem_type)
            }
            Type::List(elem) => {
                // List<T> transpiles to list_T* (wrapper around dynamic array)
                let elem_type = self.c_type_name(elem);
                format!("list_{}*", elem_type)
            }
            Type::User(usr_type) => format!("struct {}", usr_type.name),
            Type::Ptr(ptr) => {
                format!("{}*", self.c_type_name(&ptr.of.borrow()))
            }
            Type::Enum(en) => {
                format!("enum {}", en.borrow().name)
            }
            Type::Union(u) => {
                format!("union {}", u.name)
            }
            Type::Tag(t) => {
                format!("struct {}", t.borrow().name)
            }
            Type::Spec(_spec_decl) => {
                // Spec 类型在 C 中使用 void* 表示（多态）
                "void*".to_string()
            }
            Type::Slice(slice) => {
                // []T transpiles to slice_T struct in C
                // For now, use pointer representation (similar to how arrays work)
                let elem_type = self.c_type_name(&slice.elem);
                format!("struct slice_{}", elem_type)
            }
            Type::Unknown => "unknown".to_string(),
            Type::CStruct(decl) => format!("{}", decl.name),
            Type::Char => "char".to_string(),
            Type::Void => "void".to_string(),
            Type::GenericInstance(inst) => {
                // Generic instances: MyType<int> -> my_type_int
                let args: Vec<String> = inst.args.iter()
                    .map(|t| self.c_type_name(t))
                    .collect();
                format!("{}_{}",
                    inst.base_name.to_lowercase(),
                    args.join("_")
                )
            }
            Type::Storage(storage) => {
                // Storage types are marker types, transpile to void (Plan 055)
                // They don't have runtime representation
                match &storage.kind {
                    crate::ast::StorageKind::Dynamic => "/* Dynamic storage */ void".to_string(),
                    crate::ast::StorageKind::Fixed { capacity } => {
                        format!("/* Fixed storage capacity: {} */ void", capacity)
                    }
                }
            }
            _ => {
                println!("Unsupported type for C transpiler: {}", ty);
                panic!("Unsupported type for C transpiler: {}", ty);
            }
        }
    }

    fn store(&mut self, store: &Store, out: &mut impl Write) -> AutoResult<()> {
        if matches!(store.kind, StoreKind::CVar) {
            // skip C variables declaration
            return Ok(());
        }
        // StoreKind::Var is now supported (treated as mutable variable)

        // Check if the expression is a function call that returns an array or slice
        let expr_is_array_call = if let Expr::Call(call) = &store.expr {
            if let Expr::Ident(fn_name) = &call.name.as_ref() {
                if let Some(meta) = self.lookup_meta(fn_name) {
                    if let Meta::Fn(fn_decl) = meta.as_ref() {
                        matches!(fn_decl.ret, Type::Array(_) | Type::Slice(_))
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        // Special handling for array-returning function calls
        // This must come before the type checking below, so it takes priority
        if expr_is_array_call {
            // Get the array/slice return type from the function declaration
            let array_type = if let Expr::Call(call) = &store.expr {
                if let Expr::Ident(fn_name) = &call.name.as_ref() {
                    if let Some(meta) = self.lookup_meta(fn_name) {
                        if let Meta::Fn(fn_decl) = meta.as_ref() {
                            match &fn_decl.ret {
                                Type::Array(arr) => Some(arr.clone()),
                                Type::Slice(slice) => Some(ArrayType {
                                    elem: slice.elem.clone(),
                                    len: 0, // Slices don't have a fixed length known at compile time
                                }),
                                _ => None,
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(array_type) = array_type {
                let elem_type = self.c_type_name(&array_type.elem);
                let size_var = format!("_size_{}", store.name);

                // Declare size variable first (before the variable declaration)
                out.write(format!("int {};\n    ", size_var).as_bytes())
                    .to()?;

                // Declare pointer variable
                out.write(format!("{}* {} = ", elem_type, store.name).as_bytes())
                    .to()?;
            } else {
                // Fallback: couldn't get array type, use store type
                let type_name = self.c_type_name(&store.ty);
                out.write(format!("{} {} = ", type_name, store.name).as_bytes())
                    .to()?;
            }
        } else if matches!(store.ty, Type::Unknown) {
            if let Some(inferred_type) = self.infer_expr_type(&store.expr) {
                // Update the scope with the inferred type for future lookups
                self.scope
                    .borrow_mut()
                    .update_store_type(&store.name, inferred_type.clone());

                let type_name = self.c_type_name(&inferred_type);
                out.write(format!("{} {} = ", type_name, store.name).as_bytes())
                    .to()?;
            } else {
                let type_name = self.c_type_name(&store.ty);
                out.write(format!("{} {} = ", type_name, store.name).as_bytes())
                    .to()?;
            }
        } else {
            match &store.ty {
                Type::Array(array_type) => {
                    let elem_type = &array_type.elem;
                    let len = array_type.len;
                    let elem_type_name = self.c_type_name(elem_type);
                    out.write(format!("{} {}[{}] = ", elem_type_name, store.name, len).as_bytes())
                        .to()?;
                }
                Type::RuntimeArray(rta) => {
                    // Plan 052: Runtime array allocation (using heap allocation for simplicity)
                    let elem_type = self.c_type_name(&rta.elem);

                    // Always use heap allocation (malloc) for runtime arrays
                    // This avoids scope issues with VLAs and ensures the array is accessible after declaration
                    out.write(format!("{}* {} = malloc(sizeof({}) * (", elem_type, store.name, elem_type).as_bytes()).to()?;
                    self.expr(&rta.size_expr, out)?;
                    out.write(b"))").to()?;  // Close the malloc call with parenthesized size expression

                    // Initialize array if expression provided
                    if !matches!(store.expr, Expr::Nil) {
                        out.write(b";\n    ").to()?;  // End malloc statement
                        // For now, just zero-initialize
                        // TODO: Add proper initialization based on store.expr
                        out.write(format!("memset({}, 0, sizeof({}) * ", store.name, elem_type).as_bytes()).to()?;
                        self.expr(&rta.size_expr, out)?;
                        out.write(b")").to()?;  // Close memset call
                        // Note: eos() will add the final semicolon
                    } else {
                        // No initialization, eos() will add semicolon
                    }

                    return Ok(());  // Early return since we've handled everything
                }
                Type::Slice(slice_type) => {
                    // For slices, we need to determine the size from the initializer expression
                    // Slices of spec types transpile to void* arrays
                    let elem_type = &slice_type.elem;
                    let is_spec_slice = matches!(elem_type.as_ref(), Type::Spec(_));
                    let elem_type_name = if is_spec_slice {
                        "void*".to_string()
                    } else {
                        self.c_type_name(elem_type)
                    };

                    // Try to get the size from the array literal expression
                    let len = if let Expr::Array(arr) = &store.expr {
                        arr.len()
                    } else {
                        0 // Unknown size, will be determined at runtime
                    };

                    out.write(format!("{} {}[{}] = ", elem_type_name, store.name, len).as_bytes())
                        .to()?;

                    // For spec slices, we need to take addresses of struct elements
                    if is_spec_slice {
                        if let Expr::Array(arr) = &store.expr {
                            out.write(b"{").to()?;
                            for (i, elem) in arr.iter().enumerate() {
                                out.write(b"&").to()?;
                                self.expr(elem, out)?;
                                if i < arr.len() - 1 {
                                    out.write(b", ").to()?;
                                }
                            }
                            out.write(b"}").to()?;
                            return Ok(());
                        }
                    }
                }
                _ => {
                    let type_name = self.c_type_name(&store.ty);
                    out.write(format!("{} {} = ", type_name, store.name).as_bytes())
                        .to()?;
                }
            }
        }

        // For array-returning calls, we need to handle the call specially
        if expr_is_array_call {
            if let Expr::Call(call) = &store.expr {
                if let Expr::Ident(_fn_name) = &call.name.as_ref() {
                    let size_var = format!("_size_{}", store.name);

                    // Write the function call
                    self.expr(&call.name, out)?;
                    out.write(b"(").to()?;

                    // Write existing arguments
                    for (i, arg) in call.args.args.iter().enumerate() {
                        self.arg(arg, out)?;
                        if i < call.args.args.len() - 1 {
                            out.write(b", ").to()?;
                        }
                    }

                    // Add size parameter
                    if !call.args.args.is_empty() {
                        out.write(b", ").to()?;
                    }
                    out.write(b"&").to()?;
                    out.write(size_var.as_bytes()).to()?;
                    out.write(b")").to()?;
                } else {
                    self.expr(&store.expr, out)?;
                }
            } else {
                self.expr(&store.expr, out)?;
            }
        } else {
            self.expr(&store.expr, out)?;
        }
        Ok(())
    }

    fn is_stmt_target(&mut self, target: &Expr, sink: &mut Sink) -> AutoResult<()> {
        match target {
            Expr::Ident(name) => {
                // lookup name's meta
                let meta = self.lookup_meta(name);
                let Some(meta) = meta else {
                    return Err(format!("is-stmt target not found {}", name).into());
                };
                match meta.as_ref() {
                    Meta::Store(store) => match &store.ty {
                        Type::Tag(_) => {
                            sink.body.write(format!("{}.tag", name).as_bytes())?;
                            return Ok(());
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            _ => {}
        }
        self.expr(target, &mut sink.body)?;
        Ok(())
    }

    fn is_stmt(&mut self, is_stmt: &Is, sink: &mut Sink) -> AutoResult<()> {
        sink.body.write(b"switch (")?;
        self.is_stmt_target(&is_stmt.target, sink)?;
        // self.expr(&is_stmt.target, &mut sink.body)?;
        sink.body.write(b") {\n")?;

        // Infer return type from the first branch that has a body with a single expression
        let return_type = self.infer_is_return_type(is_stmt);

        for case in &is_stmt.branches {
            self.print_indent(&mut sink.body)?;
            match case {
                IsBranch::EqBranch(expr, body) => {
                    sink.body.write(b"case ")?;
                    self.expr(expr, &mut sink.body)?;
                    sink.body.write(b":\n")?;
                    self.indent();
                    self.print_indent(&mut sink.body)?;
                    self.body(body, sink, &return_type, "", "")?;
                    sink.body.write(b"\n")?;
                    self.print_with_indent(&mut sink.body, "break;\n")?;
                    self.dedent();
                }
                IsBranch::IfBranch(expr, body) => {
                    sink.body.write(b"case ")?;
                    self.expr(expr, &mut sink.body)?;
                    sink.body.write(b": \n")?;
                    self.indent();
                    self.print_indent(&mut sink.body)?;
                    self.body(body, sink, &return_type, "", "")?;
                    sink.body.write(b"\n")?;
                    self.print_with_indent(&mut sink.body, "break;\n")?;
                    self.dedent();
                }
                IsBranch::ElseBranch(body) => {
                    sink.body.write(b"default:\n")?;
                    self.indent();
                    self.print_indent(&mut sink.body)?;
                    self.body(body, sink, &return_type, "", "")?;
                    sink.body.write(b"\n")?;
                    self.print_with_indent(&mut sink.body, "break;\n")?;
                    self.dedent();
                }
            }
        }
        self.print_indent(&mut sink.body)?;
        sink.body.write(b"}")?;
        Ok(())
    }

    fn infer_is_return_type(&mut self, is_stmt: &Is) -> Type {
        // Check if all branches have a single expression of the same type
        // Collect all non-panic, non-Unknown types and use the first one found
        for branch in &is_stmt.branches {
            match branch {
                IsBranch::EqBranch(_, body)
                | IsBranch::IfBranch(_, body)
                | IsBranch::ElseBranch(body) => {
                    if body.stmts.len() == 1 {
                        if let Stmt::Expr(expr) = &body.stmts[0] {
                            // Skip panic calls - they don't determine the return type
                            if let Expr::Call(call) = expr {
                                if let Expr::Ident(name) = &call.name.as_ref() {
                                    if name == "panic" {
                                        continue;
                                    }
                                }
                            }
                            let ty = self.infer_expr_type(expr);
                            if let Some(t) = ty {
                                if !matches!(t, Type::Unknown) {
                                    return t;
                                }
                            }
                        }
                    }
                }
            }
        }
        Type::Void
    }

    fn get_type_of(&mut self, name: &AutoStr) -> Option<Type> {
        if let Some(m) = self.lookup_meta(name) {
            match m.as_ref() {
                Meta::Store(store) => Some(store.ty.clone()),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Infer the return type of an expression if possible
    fn infer_expr_type(&mut self, expr: &Expr) -> Option<Type> {
        match expr {
            // For method calls like file.read_text()
            Expr::Call(call) => {
                // First check if it's a direct function call
                if let Expr::Ident(fn_name) = &call.name.as_ref() {
                    if let Some(meta) = self.lookup_meta(fn_name) {
                        if let Meta::Fn(fn_decl) = meta.as_ref() {
                            return Some(fn_decl.ret.clone());
                        }
                    }
                }

                // Then check if it's a method call (obj.method())
                if let Expr::Bina(lhs, Op::Dot, rhs) = call.name.as_ref() {
                    if let Expr::Ident(obj_name) = lhs.as_ref() {
                        if let Expr::Ident(method_name) = rhs.as_ref() {
                            // Check if this is a tag type method call (tag construction)
                            if let Some(meta) = self.lookup_meta(obj_name) {
                                match meta.as_ref() {
                                    Meta::Type(Type::Tag(tag)) => {
                                        // This is tag construction: Tag.Variant(args)
                                        // Return the tag type (clone the Shared<Tag>)
                                        return Some(Type::Tag(tag.clone()));
                                    }
                                    Meta::Store(store) => {
                                        // This is a regular method call on an instance
                                        if let Type::User(decl) = &store.ty {
                                            // Find the method and return its return type
                                            for method in &decl.methods {
                                                if method.name == *method_name {
                                                    return Some(method.ret.clone());
                                                }
                                            }
                                        }
                                        // Also check if it's a tag type
                                        if let Type::Tag(tag) = &store.ty {
                                            // Find the method in the tag
                                            let tag_ref = tag.borrow();
                                            for method in &tag_ref.methods {
                                                if method.name == *method_name {
                                                    return Some(method.ret.clone());
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                None
            }
            // For direct identifier lookups
            Expr::Ident(name) => self.get_type_of(name),
            // Literal expressions
            Expr::Bool(_) => Some(Type::Bool),
            Expr::Int(_) => Some(Type::Int),
            Expr::I8(_) => Some(Type::Int),
            Expr::Uint(_) | Expr::Byte(_) | Expr::U8(_) => Some(Type::Uint),
            Expr::Float(_, _) => Some(Type::Float),
            Expr::Double(_, _) => Some(Type::Double),
            Expr::Char(_) => Some(Type::Char),
            Expr::Str(_) => Some(Type::Str(0)),
            Expr::CStr(_) => Some(Type::CStr),
            _ => None,
        }
    }

    fn for_stmt(&mut self, for_stmt: &For, sink: &mut Sink) -> AutoResult<()> {
        let mut iter_var = String::new();
        match &for_stmt.iter {
            Iter::Call(_) => {
                sink.body.write(b"while (").to()?;
                self.iter(&for_stmt.iter, &mut sink.body)?;
            }
            Iter::Ever => {
                sink.body.write(b"while (1").to()?;
            }
            Iter::Named(n) => {
                sink.body.write(b"for (")?;
                // iter elem's type
                match &for_stmt.range {
                    Expr::Range(r) => {
                        self.range(n.as_str(), &r.start, &r.end, &mut sink.body)?;
                    }
                    Expr::Ident(range_name) => {
                        let range_type = self.get_type_of(range_name);
                        if let Some(range_type) = range_type {
                            match &range_type {
                                Type::Array(arr) => {
                                    let elem_type = &*arr.elem;
                                    let elem_size = arr.len;
                                    let elem_type_name = self.c_type_name(elem_type);
                                    iter_var = format!(
                                        "{} {} = {}[{}];\n",
                                        elem_type_name, n, range_name, "i"
                                    );
                                    self.range(
                                        "i",
                                        &Expr::Int(0),
                                        &Expr::Int(elem_size as i32),
                                        &mut sink.body,
                                    )?;
                                }
                                Type::Slice(slice) => {
                                    // For slices, we need to get the size from the store metadata
                                    // Try to get the size from the store's type info
                                    let elem_type = &*slice.elem;
                                    let elem_type_name = if matches!(elem_type, Type::Spec(_)) {
                                        "void*".to_string()
                                    } else {
                                        self.c_type_name(elem_type)
                                    };

                                    // Get the size from the store's initialization
                                    let size = if let Some(meta) = self.lookup_meta(range_name) {
                                        if let Meta::Store(store) = meta.as_ref() {
                                            if let Expr::Array(arr) = &store.expr {
                                                arr.len()
                                            } else {
                                                0
                                            }
                                        } else {
                                            0
                                        }
                                    } else {
                                        0
                                    };

                                    iter_var = format!(
                                        "{} {} = {}[{}];\n",
                                        elem_type_name, n, range_name, "i"
                                    );
                                    self.range(
                                        "i",
                                        &Expr::Int(0),
                                        &Expr::Int(size as i32),
                                        &mut sink.body,
                                    )?;
                                }
                                // Type::Str => {
                                // let elem_type = Type::Char;
                                // let elem_size =
                                //
                                // }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
                // if typ.is_indexable() {
                // let elem_type = typ.get_elem_type();
                // sink.body.write(format!("{} ", elem_type).as_bytes())?;
                // } else {
                // sink.body.write(b"int ")?;
                // }
                // iter elem

                // let idx = "i";
                // sink.body.write(format!("size_t {} = {}[0]; ", idx, &for_stmt.range.repr()).as_bytes())?;
                // sink.body.write(format!("{} < {}.size; ", idx, &for_stmt.range.repr()).as_bytes())?;
                // sink.body.write(format!("{}++", idx).as_bytes())?;
            }
            Iter::Cond => {
                sink.body.write(b"for (").to()?;
                // Handle init statement if present: for init; condition { ... }
                if let Some(init_stmt) = &for_stmt.init {
                    self.stmt(init_stmt, sink)?;
                    sink.body.write(b"; ").to()?;
                }
                self.expr(&for_stmt.range, &mut sink.body)?;
            }
            Iter::Indexed(_, _) => {
                sink.body.write(b"for (").to()?;
                // Handle init statement if present
                if let Some(init_stmt) = &for_stmt.init {
                    self.stmt(init_stmt, sink)?;
                    sink.body.write(b"; ").to()?;
                }
                self.expr(&for_stmt.range, &mut sink.body)?;
            }
        }
        sink.body.write(b") ").to()?;
        self.body(&for_stmt.body, sink, &Type::Void, iter_var.as_str(), "")?;
        Ok(())
    }

    fn iter(&mut self, iter: &Iter, out: &mut impl Write) -> AutoResult<()> {
        match iter {
            Iter::Indexed(_i, _iter) => {}
            Iter::Named(_) => {}
            Iter::Ever => {}
            Iter::Cond => {}
            Iter::Call(call) => {
                self.call(call, out)?;
            }
        }
        Ok(())
    }

    fn range(
        &mut self,
        iter: &str,
        start: &Expr,
        end: &Expr,
        out: &mut impl Write,
    ) -> AutoResult<()> {
        // TODO: check index name for deep loops
        out.write(format!("int {} = ", iter).as_bytes())?;
        self.expr(start, out)?;
        out.write(format!("; {} < ", iter).as_bytes())?;
        self.expr(end, out)?;
        out.write(format!("; {}++", iter).as_bytes())?;
        Ok(())
    }

    fn if_stmt(&mut self, if_: &If, sink: &mut Sink) -> AutoResult<()> {
        sink.body.write(b"if ").to()?;
        for (i, branch) in if_.branches.iter().enumerate() {
            sink.body.write(b"(").to()?;
            self.expr(&branch.cond, &mut sink.body)?;
            sink.body.write(b") ").to()?;
            self.body(&branch.body, sink, &Type::Void, "", "")?;
            if i < if_.branches.len() - 1 {
                sink.body.write(b" else ")?;
            }
        }
        if let Some(body) = &if_.else_ {
            sink.body.write(b" else ").to()?;
            self.body(body, sink, &Type::Void, "", "")?;
        }
        Ok(())
    }

    fn lookup_meta(&self, ident: &AutoStr) -> Option<Rc<Meta>> {
        self.scope.borrow().lookup_meta(ident)
    }

    fn lookup_type(&self, ident: &AutoStr) -> Type {
        self.scope.borrow().lookup_type(ident)
    }

    fn is_enum_type(&self, type_name: &AutoStr) -> bool {
        match self.lookup_type(type_name) {
            Type::Enum(_) => true,
            _ => false,
        }
    }

    fn print_slice(&mut self, arr: &Expr, r: &Range, out: &mut impl Write) -> AutoResult<()> {
        let Some(arr_type) = self.get_type_of(&arr.repr()) else {
            return Err(format!("Wrong array type: {}", arr).into());
        };
        match &arr_type {
            Type::Array(_a) => {
                // for (int i = 0; i < size; i++) { print()}
                out.write(b"for (")?;
                self.range("i", &r.start, &r.end, out)?;
                out.write(b") {\n")?;

                self.indent();
                self.print_indent(out)?;
                out.write(format!("printf(\"%d\", {}[{}]);\n", arr.repr(), "i").as_bytes())?;
                self.dedent();

                self.print_indent(out)?;
                out.write(b"}\n")?;
                self.print_indent(out)?;
                out.write(b"printf(\"\\n\")")?;
            }
            Type::Str(_size) => {
                out.write(b"for (")?;
                self.range("i", &r.start, &r.end, out)?;
                out.write(b") {\n")?;
                self.indent();
                self.print_indent(out)?;
                out.write(format!("printf(\"%c\", {}[{}]);\n", arr.repr(), "i").as_bytes())?;
                self.dedent();
                self.print_indent(out)?;
                out.write(b"}\n")?;
                self.print_indent(out)?;
                out.write(b"printf(\"\\n\")")?;
            }
            _ => {
                return Err(format!("Wrong slice type {}", arr_type).into());
            }
        }
        Ok(())
    }

    /// Get printf format specifier for an identifier expression
    fn get_ident_format_specifier(&mut self, ident: &AutoStr) -> &'static str {
        let meta = self.lookup_meta(ident);
        if let Some(meta) = meta {
            if let Meta::Store(st) = meta.as_ref() {
                return match &st.ty {
                    Type::Str(_) | Type::CStr => "%s",
                    Type::Float => "%f",
                    Type::Char => "%c",
                    Type::Ptr(ptr) => {
                        if matches!(*ptr.of.borrow(), Type::Char) {
                            "%s"
                        } else {
                            "%d"
                        }
                    }
                    Type::Array(arr) => {
                        if matches!(*arr.elem, Type::Char) {
                            "%s"
                        } else {
                            "%d"
                        }
                    }
                    _ => "%d",
                };
            }
        }
        "%d"
    }

    /// Get printf format specifier for an index expression
    fn get_index_format_specifier(&mut self, arr: &Expr, idx: &Expr) -> Option<&'static str> {
        // Check if this is a slice operation (index with range)
        if let Expr::Range(_) = idx {
            return None; // Handled separately by print_slice
        }

        // Check array type
        if let Expr::Ident(n) = arr {
            if let Some(m) = self.lookup_meta(&n) {
                if let Meta::Store(s) = m.as_ref() {
                    if matches!(s.ty, Type::Str(_)) {
                        return Some("%c");
                    }
                }
            }
        }
        Some("%d")
    }

    /// Check if any argument has a custom print method and generate that call instead
    fn try_custom_print(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<bool> {
        for arg in call.args.args.iter() {
            if let Arg::Pos(expr) = arg {
                if let Expr::Ident(ident) = expr {
                    let meta = self.lookup_meta(ident);
                    if let Some(meta) = meta {
                        if let Meta::Store(st) = meta.as_ref() {
                            if let Type::User(typ) = &st.ty {
                                if typ.has_method("print") {
                                    out.write(format!("{}_Print(", typ.name).as_bytes())?;
                                    for (i, arg) in call.args.args.iter().enumerate() {
                                        self.arg(arg, out)?;
                                        if i < call.args.args.len() - 1 {
                                            out.write(b", ").to()?;
                                        }
                                    }
                                    out.write(b")")?;
                                    return Ok(true);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(false)
    }

    fn process_print(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
        // TODO: check type of the args and format accordingly
        self.libs.insert("<stdio.h>".into());

        // Check if any arg has a custom print method
        if self.try_custom_print(call, out)? {
            return Ok(());
        }

        // get number and type of args
        let mut arg_types = Vec::new();
        for arg in call.args.args.iter() {
            match arg {
                Arg::Pos(expr) => {
                    match expr {
                        Expr::Int(_) => arg_types.push("%d"),
                        Expr::Str(_) => arg_types.push("%s"),
                        Expr::CStr(_) => arg_types.push("%s"),
                        Expr::Float(_, _) => arg_types.push("%f"),
                        Expr::Char(_) => arg_types.push("%c"),
                        // TODO: check the actual type of the identifier
                        Expr::Ident(ident) => {
                            arg_types.push(self.get_ident_format_specifier(ident));
                        }
                        Expr::Index(arr, idx) => {
                            match &**idx {
                                Expr::Range(r) => {
                                    return self.print_slice(&**arr, r, out);
                                }
                                _ => {
                                    // Use helper to get format specifier
                                    if let Some(spec) = self.get_index_format_specifier(arr, idx) {
                                        arg_types.push(spec);
                                    } else {
                                        arg_types.push("%d");
                                    }
                                }
                            }
                        }
                        _ => {
                            // other types are now viewed as ints
                            arg_types.push("%d");
                        }
                    }
                }
                _ => {
                    // TODO: implement identifier args and named args
                }
            }
        }
        let fmt = format!("printf(\"{}\\n\", ", arg_types.join(" "));
        out.write(fmt.as_bytes())?;
        for (i, arg) in call.args.args.iter().enumerate() {
            self.arg(arg, out)?;
            if i < call.args.args.len() - 1 {
                out.write(b", ").to()?;
            }
        }
        out.write(b")")?;
        Ok(())
    }

    /// Handle method calls on Tag types (e.g., Tag.Class(data))
    fn handle_tag_method(
        &mut self,
        tag: &Tag,
        lname: &str,
        rname: &AutoStr,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<bool> {
        let ftype = tag.get_field_type(rname);
        if let Type::Unknown = ftype {
            return Ok(false);
        }

        let mut rtext: Vec<u8> = Vec::new();
        // Handle optional argument for tag constructors (e.g., MayInt.Nil())
        if let Some(Arg::Pos(expr)) = call.args.args.first() {
            self.expr(expr, &mut rtext)?;
        } else {
            // No argument provided, use default value 0
            rtext.write(b"0")?;
        }

        // transform this method call into a node creation
        let node = Node {
            name: lname.into(),
            id: lname.into(),
            num_args: 0,
            args: Args::new(),
            body: Body {
                stmts: vec![
                    // kind
                    Stmt::Expr(Expr::Pair(Pair {
                        key: Key::NamedKey("tag".into()),
                        value: Box::new(Expr::GenName(tag.enum_name(rname))),
                    })),
                    // value
                    Stmt::Expr(Expr::Pair(Pair {
                        key: Key::NamedKey(format!("as.{}", rname).into()),
                        value: Box::new(Expr::GenName(String::from_utf8(rtext).unwrap().into())),
                    })),
                ],
                has_new_line: true,
            },
            typ: shared(Type::Tag(shared(tag.clone()))),
        };
        self.node(&node, out)?;
        Ok(true)
    }

    /// Handle method calls on Store/UserType instances
    fn handle_store_method(
        &mut self,
        decl: &TypeDecl,
        lname: &str,
        method_name: &str,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<bool> {
        let type_name = &decl.name;

        // First check if the type has this method directly
        for m in decl.methods.iter() {
            if m.name == *method_name {
                // write the method call as Type_MethodName(&s, args...)
                // Note: add type prefix as Type_MethodName(...)
                self.method_name(type_name, method_name, out)?;
                out.write(b"(")?;
                out.write(b"&")?;
                out.write(lname.as_bytes())?;
                if !call.args.is_empty() {
                    out.write(b", ")?;
                    for (i, arg) in call.args.args.iter().enumerate() {
                        if i > 0 {
                            out.write(b", ")?;
                        }
                        self.expr(&arg.get_expr(), out)?;
                    }
                }
                out.write(b")").to()?;
                return Ok(true);
            }
        }

        // Check delegations - look for a delegation that implements this method
        // Collect delegation info first to avoid borrow issues
        let mut delegation_impl: Option<(AutoStr, AutoStr)> = None;
        for delegation in decl.delegations.iter() {
            let spec_name = delegation.spec_name.clone();
            if let Some(meta) = self.scope.borrow().lookup_meta(spec_name.as_str()) {
                if let Meta::Spec(spec_decl) = meta.as_ref() {
                    for spec_method in spec_decl.methods.iter() {
                        if spec_method.name == *method_name {
                            delegation_impl = Some((
                                delegation.member_name.clone(),
                                delegation.member_type.unique_name(),
                            ));
                            break;
                        }
                    }
                }
            }
            if delegation_impl.is_some() {
                break;
            }
        }

        if let Some((_member_name, _member_type_name)) = delegation_impl {
            // Use the delegation wrapper method
            out.write(type_name.as_bytes())?;
            out.write(b"_")?;
            out.write(method_name.as_bytes())?;
            out.write(b"(&")?;
            out.write(lname.as_bytes())?;
            if !call.args.is_empty() {
                out.write(b", ")?;
                for (i, arg) in call.args.args.iter().enumerate() {
                    if i > 0 {
                        out.write(b", ")?;
                    }
                    self.expr(&arg.get_expr(), out)?;
                }
            }
            out.write(b")").to()?;
            return Ok(true);
        }

        Ok(false)
    }

    fn method_call(
        &mut self,
        lhs: &Box<Expr>,
        rhs: &Box<Expr>,
        call: &Call,
        out: &mut impl Write,
    ) -> AutoResult<bool> {
        // Plan 035 Phase 5.2: Handle ext methods for built-in types
        // Check if lhs is a built-in type or variable
        let lhs_type = self.get_expr_type(lhs);

        // If lhs has a known type, check if it's a built-in type (ext method)
        // or user-defined type (regular method)
        if !matches!(lhs_type, Type::Unknown) {
            let Expr::Ident(method_name) = rhs.as_ref() else {
                return Ok(false);
            };

            // Check if it's a built-in type (ext method) or user-defined type
            match &lhs_type {
                Type::Int
                | Type::Uint
                | Type::Byte
                | Type::Float
                | Type::Double
                | Type::Bool
                | Type::Char
                | Type::Str(_)
                | Type::CStr => {
                    // Built-in type: ext method, pass by value
                    let type_name = self.type_to_name(&lhs_type);
                    let c_function_name = format!("{}_{}", type_name, method_name);

                    // Write the function call
                    out.write_all(c_function_name.as_bytes()).to()?;
                    out.write(b"(").to()?;

                    // Write self as first argument (by value)
                    self.expr(lhs, out)?;

                    // Write remaining arguments
                    for (_i, arg) in call.args.args.iter().enumerate() {
                        out.write(b", ").to()?;
                        self.arg(arg, out)?;
                    }

                    out.write(b")").to()?;
                    return Ok(true);
                }
                Type::User(_decl) => {
                    // User-defined type: regular method, pass by pointer
                    // Fall through to original logic below
                }
                _ => {
                    // Other types: try original logic
                }
            }
        }

        // Original logic for Tag and Store methods
        let Expr::Ident(lname) = lhs.as_ref() else {
            return Ok(false);
        };
        let Some(meta) = self.lookup_meta(lname) else {
            return Ok(false);
        };
        match meta.as_ref() {
            // Tag.Class(data)
            Meta::Type(typ) => match typ {
                Type::Tag(tag) => {
                    let Expr::Ident(rname) = rhs.as_ref() else {
                        return Ok(false);
                    };
                    Ok(self.handle_tag_method(&*tag.borrow(), lname, rname, call, out)?)
                }
                _ => Ok(false),
            },
            // instance.method_name(&s, args...)
            Meta::Store(store) => {
                // check rhs is a method call
                let Expr::Ident(method_name) = rhs.as_ref() else {
                    return Ok(false);
                };

                // Handle tag type methods
                if let Type::Tag(tag) = &store.ty {
                    let tag_ref = tag.borrow();
                    // Check if this is a tag method (not a variant constructor)
                    for method in &tag_ref.methods {
                        if method.name == *method_name {
                            // Generate: Tag_Method(&instance, args...)
                            self.method_name(&tag_ref.name, method_name, out)?;
                            out.write(b"(")?;
                            out.write(b"&")?;
                            out.write(lname.as_bytes())?;
                            if !call.args.is_empty() {
                                out.write(b", ")?;
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    if i > 0 {
                                        out.write(b", ")?;
                                    }
                                    self.expr(&arg.get_expr(), out)?;
                                }
                            }
                            out.write(b")").to()?;
                            return Ok(true);
                        }
                    }
                    // Not a method, might be a variant constructor - fall through
                    return Ok(false);
                }

                // Handle user-defined type methods
                let Type::User(decl) = &store.ty else {
                    return Ok(false);
                };
                Ok(self.handle_store_method(decl, lname, method_name, call, out)?)
            }
            _ => Ok(false),
        }
    }

    /// Get the type of an expression (Plan 035 Phase 5.2 helper)
    fn get_expr_type(&self, expr: &Expr) -> Type {
        match expr {
            Expr::Ident(name) => {
                // Try to lookup variable type
                if let Some(meta) = self.lookup_meta(name) {
                    match meta.as_ref() {
                        Meta::Store(store) => store.ty.clone(),
                        _ => Type::Unknown,
                    }
                } else {
                    // Check if it's a built-in type name
                    self.name_to_type(name)
                }
            }
            _ => Type::Unknown,
        }
    }

    /// Convert Type to its name string (Plan 035 Phase 5.2 helper)
    fn type_to_name(&self, ty: &Type) -> String {
        match ty {
            Type::Int => "int".to_string(),
            Type::Uint => "uint".to_string(),
            Type::Byte => "byte".to_string(),
            Type::Float => "float".to_string(),
            Type::Double => "double".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Char => "char".to_string(),
            Type::Str(_) => "str".to_string(),
            Type::CStr => "cstr".to_string(),
            Type::User(decl) => decl.name.to_string(),
            _ => "unknown".to_string(),
        }
    }

    /// Validate struct initialization arguments against type declaration
    fn validate_struct_init(&mut self, type_decl: &TypeDecl, args: &Args) -> AutoResult<()> {
        for arg in &args.args {
            if let Arg::Pair(key, value_expr) = arg {
                // Find the field declaration
                let field = type_decl
                    .members
                    .iter()
                    .find(|m| &m.name == key)
                    .ok_or_else(|| {
                        format!("Field '{}' not found in type '{}'", key, type_decl.name)
                    })?;

                // Get the expected type from the field declaration
                let expected_type = &field.ty;

                // Infer the type of the value expression
                let value_type = self.infer_literal_type(value_expr);

                // Check if types match
                if !self.types_compatible(&value_type, expected_type) {
                    return Err(format!(
                        "Type mismatch: field '{}' declared as '{}' but initialized with '{}' value",
                        key, expected_type, value_type
                    ).into());
                }
            }
        }
        Ok(())
    }

    /// Infer the type of a literal expression (for type checking)
    fn infer_literal_type(&self, expr: &Expr) -> Type {
        match expr {
            Expr::I8(_) | Expr::Int(_) => Type::Int,
            Expr::Uint(_) | Expr::Byte(_) | Expr::U8(_) => Type::Uint,
            Expr::Float(_, _) => Type::Float,
            Expr::Double(_, _) => Type::Double,
            Expr::Bool(_) => Type::Bool,
            Expr::Char(_) => Type::Char,
            Expr::Str(s) => Type::Str(s.len()),
            Expr::CStr(_) => Type::CStr,
            Expr::Array(elems) => {
                if elems.is_empty() {
                    Type::Array(ArrayType {
                        elem: Box::new(Type::Unknown),
                        len: 0,
                    })
                } else {
                    let elem_type = self.infer_literal_type(&elems[0]);
                    Type::Array(ArrayType {
                        elem: Box::new(elem_type),
                        len: elems.len(),
                    })
                }
            }
            Expr::Ident(name) => self.lookup_type(name),
            _ => Type::Unknown,
        }
    }

    /// Check if two types are compatible (for type checking)
    fn types_compatible(&self, actual: &Type, expected: &Type) -> bool {
        match (actual, expected) {
            // Exact match
            (a, e) if std::mem::discriminant(a) == std::mem::discriminant(e) => true,
            // String types are compatible
            (Type::Str(_), Type::Str(_)) => true,
            // Numeric types: allow some conversions
            (Type::Int, Type::Uint) | (Type::Uint, Type::Int) => true,
            (Type::Float, Type::Double) | (Type::Double, Type::Float) => true,
            // Unknown matches anything (for error recovery)
            (Type::Unknown, _) | (_, Type::Unknown) => true,
            _ => false,
        }
    }

    fn call(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
        // method call
        // Support both old syntax (Expr::Bina with Op::Dot) and new syntax (Expr::Dot)
        if let Expr::Bina(lhs, op, rhs) = call.name.as_ref() {
            if matches!(op, Op::Dot) {
                if self.method_call(lhs, rhs, call, out)? {
                    return Ok(());
                }
            }
        } else if let Expr::Dot(object, method) = call.name.as_ref() {
            // Plan 056: Method call using Expr::Dot(object, method)
            // Convert object and method to boxed references for method_call
            if self.method_call(&Box::new(object.as_ref().clone()), &Box::new(Expr::Ident(method.clone())), call, out)? {
                return Ok(());
            }
        }

        // normal call
        if let Expr::Ident(name) = &call.name.as_ref() {
            if name == "print" {
                return self.process_print(call, out);
            } else {
                // Check if this is a struct initialization (user-defined type)
                if let Some(meta) = self.lookup_meta(name) {
                    if let Meta::Type(Type::User(type_decl)) = meta.as_ref() {
                        // Validate struct initialization arguments
                        self.validate_struct_init(&type_decl, &call.args)?;
                    }
                }
                self.expr(&call.name, out)?;
                out.write(b"(").to()?;
            }
        } else {
            self.expr(&call.name, out)?;
            out.write(b"(").to()?;
        }
        for (i, arg) in call.args.args.iter().enumerate() {
            self.arg(arg, out)?;
            if i < call.args.args.len() - 1 {
                out.write(b", ").to()?;
            }
        }
        // TODO: support named args in C
        // Find where a named arg is positioned, and insert default arg values in between
        // // // for (name, expr) in &call.args.map {
        // //     self.expr(expr, out)?;
        // }
        out.write(b")").to()?;
        Ok(())
    }

    fn arg(&mut self, arg: &Arg, out: &mut impl Write) -> AutoResult<()> {
        match arg {
            Arg::Name(name) => self.str(name.as_str(), out),
            Arg::Pair(_, expr) => self.expr(expr, out),
            Arg::Pos(expr) => self.expr(expr, out),
        }
    }

    fn str(&mut self, s: &str, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"\"").to()?;
        out.write(s.as_bytes()).to()?;
        out.write(b"\"").to()?;
        Ok(())
    }

    fn array(&mut self, array: &Vec<Expr>, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"{").to()?;
        for (i, expr) in array.iter().enumerate() {
            self.expr(expr, out)?;
            if i < array.len() - 1 {
                out.write(b", ").to()?;
            }
        }
        out.write(b"}").to()?;
        Ok(())
    }

    fn is_returnable(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(expr) => match expr {
                Expr::Call(call) => {
                    if let Expr::Ident(name) = &call.name.as_ref() {
                        if name == "print" {
                            return false;
                        }
                    }
                    // check return type of call
                    match &call.ret {
                        Type::Void | Type::Unknown => {
                            return false;
                        }
                        _ => {}
                    }
                    true
                }
                _ => true,
            },
            _ => false,
        }
    }
}

fn cmp_include_name(a: &AutoStr, b: &AutoStr) -> Ordering {
    let sa = a.as_bytes()[0];
    let sb = b.as_bytes()[0];
    match (sa, sb) {
        (b'<', b'"') => {
            return std::cmp::Ordering::Less;
        }
        (b'"', b'<') => {
            return std::cmp::Ordering::Greater;
        }
        _ => {
            return a.to_string().cmp(&b.to_string());
        }
    }
}

impl Trans for CTrans {
    fn trans(&mut self, ast: Code, sink: &mut Sink) -> AutoResult<()> {
        // Split stmts into decls and main
        // TODO: handle potential includes when needed
        let mut decls: Vec<Stmt> = Vec::new();
        let mut main: Vec<Stmt> = Vec::new();

        // preprocess
        for stmt in ast.stmts.into_iter() {
            if stmt.is_decl() {
                decls.push(stmt);
            } else {
                match stmt {
                    Stmt::For(_) => main.push(stmt),
                    Stmt::If(_) => main.push(stmt),
                    Stmt::Expr(ref expr) => {
                        match expr {
                            Expr::Call(call) => {
                                if let Expr::Ident(name) = &call.name.as_ref() {
                                    if name == "print" {
                                        self.libs.insert("<stdio.h>".into());
                                    }
                                }
                            }
                            _ => {}
                        }
                        main.push(stmt);
                    }
                    Stmt::Use(use_stmt) => self.use_stmt(&use_stmt, &mut sink.body)?,
                    _ => {}
                }
            }
        }

        // Decls
        for (i, decl) in decls.iter().enumerate() {
            let generated = self.stmt(decl, sink)?;
            if i < decls.len() - 1 && generated {
                sink.body.write(b"\n").to()?;
            }
        }

        // Transpile substituted generic tags from scope
        // These are tags created during parsing when generic instances are used (e.g., May<int> → May_int)
        let scope = self.scope.clone();
        let scope_borrowed = scope.borrow();
        // Iterate through all scopes to find substituted tags
        for (_sid, scope_data) in scope_borrowed.scopes.iter() {
            for (name, meta) in scope_data.symbols.iter() {
                if let Meta::Type(Type::Tag(tag)) = &**meta {
                    // Check if this is a substituted tag (contains underscore and has no type params)
                    let tag_borrowed = tag.borrow();
                    if tag_borrowed.generic_params.is_empty() && name.contains('_') {
                        // This is likely a substituted tag - transpile it
                        drop(tag_borrowed);
                        self.tag(&tag.borrow(), sink)?;
                    }
                }
            }
        }
        drop(scope_borrowed);

        // Main
        // TODO: check wether auto code already has a main function
        if !main.is_empty() {
            if !decls.is_empty() {
                sink.body.write(b"\n").to()?;
            }

            sink.body.write(b"int main(void) {\n").to()?;
            self.indent();
            for (i, stmt) in main.iter().enumerate() {
                self.print_indent(&mut sink.body)?;
                if i < main.len() - 1 {
                    self.stmt(stmt, sink)?;
                    sink.body.write(b"\n").to()?;
                } else {
                    if self.is_returnable(stmt) {
                        sink.body.write(b"return ").to()?;
                        self.stmt(stmt, sink)?;
                        sink.body.write(b"\n").to()?;
                    } else {
                        self.stmt(stmt, sink)?;
                        sink.body.write(b"\n").to()?;
                        self.print_indent(&mut sink.body)?;
                        sink.body.write(b"return 0;\n").to()?;
                    }
                }
            }
            self.dedent();
            sink.body.write(b"}\n").to()?;
        }

        // write header if header content is not empty
        if !self.header.is_empty() || !self.libs.is_empty() {
            // write header guards
            self.header_guard_start(&mut sink.header)?;
            // includes
            let libs_set = std::mem::take(&mut self.libs);
            let mut libs = libs_set.into_iter().collect::<Vec<_>>();
            libs.sort_by(cmp_include_name);

            for path in libs.iter() {
                sink.header.write(b"#include ")?;
                sink.header.write(path.as_bytes())?;
                sink.header.write(b"\n")?;
            }

            if self.uses_bool {
                sink.header.write(b"#include <stdbool.h>\n")?;
            }

            if !libs.is_empty() && !self.header.is_empty() {
                sink.header.write(b"\n")?;
            }

            sink.header.write_all(&self.header)?;
            // header guard end
            self.header_guard_end(&mut sink.header)?;
        }

        Ok(())
    }
}

pub fn transpile_part(code: &str) -> AutoResult<AutoStr> {
    let mut transpiler = CTrans::new("part".into());
    let scope = Rc::new(RefCell::new(Universe::new()));
    let mut parser = Parser::new(code, scope);
    let ast = parser.parse().map_err(|e| e.to_string())?;
    let mut out = Sink::new(AutoStr::from(""));
    transpiler.code(ast, &mut out)?;
    Ok(String::from_utf8(out.body).unwrap().into())
}

pub struct CCode {
    pub includes: Vec<u8>,
    pub source: Vec<u8>,
    pub header: Vec<u8>,
}

// Transpile the code into a whole C program
pub fn transpile_c(name: impl Into<AutoStr>, code: &str) -> AutoResult<(Sink, Shared<Universe>)> {
    let name = name.into();
    let scope = Rc::new(RefCell::new(Universe::new()));
    let mut parser = Parser::new(code, scope);
    parser.set_dest(crate::parser::CompileDest::TransC);
    let ast = parser.parse().map_err(|e| e.to_string())?;
    let mut out = Sink::new(name.clone());
    let mut transpiler = CTrans::new(name);
    transpiler.scope = parser.scope.clone();
    transpiler.trans(ast, &mut out)?;

    let uni = parser.scope.clone();
    let paks = std::mem::take(&mut parser.scope.borrow_mut().code_paks);
    // let paks = parser.scope.borrow().code_paks.clone();
    for (sid, pak) in paks.iter() {
        let name = sid.name();
        let mut out = Sink::new(name.clone());
        let mut transpiler = CTrans::new(sid.name().into());
        uni.borrow_mut().set_spot(sid.clone());
        transpiler.scope = uni.clone();
        transpiler.trans(pak.ast.clone(), &mut out)?;

        let src = out.done()?.clone();

        let str = String::from_utf8(src).unwrap();
        let file = pak.file.replace(".at", ".c");
        std::fs::write(Path::new(file.as_str()), str)?;

        let header = out.header;
        if header.is_empty() {
            continue;
        }
        let header_file = &pak.header;
        std::fs::write(Path::new(header_file.as_str()), header)?;
    }
    parser.scope.borrow_mut().code_paks = paks;
    Ok((out, parser.scope.clone()))
}
