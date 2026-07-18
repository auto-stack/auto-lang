use crate::ast::*;
use crate::AutoResult;
use auto_val::{Op, AutoStr};
use std::io::Write;
use super::{Sink, TypeScriptTrans, ToStrError};
use super::super::escape_str;

#[allow(unused_variables)]
impl TypeScriptTrans {
    pub fn expr(&mut self, expr: &Expr, sink: &mut Sink) -> AutoResult<()> {
        let out = &mut sink.body;
        match expr {
            // Literals
            Expr::Int(i) => write!(&mut sink.body, "{}", i).map_err(Into::into),
            Expr::Uint(u) => write!(&mut sink.body, "{}", u).map_err(Into::into),
            Expr::Float(f, _) => write!(&mut sink.body, "{}", f).map_err(Into::into),
            Expr::Double(d, _) => write!(&mut sink.body, "{}", d).map_err(Into::into),
            Expr::Bool(b) => write!(&mut sink.body, "{}", b).map_err(Into::into),
            Expr::Char(c) => write!(&mut sink.body, "'{}'", c).map_err(Into::into),
            Expr::Str(s) => write!(&mut sink.body, "\"{}\"", escape_str(s)).map_err(Into::into),
            Expr::CStr(s) => write!(&mut sink.body, "\"{}\"", s).map_err(Into::into),

            // F-strings → Template literals (perfect match!)
            Expr::FStr(fstr) => self.fstr(fstr, sink),

            // Identifier
            Expr::Ident(name) => {
                if name == "self" {
                    // In Vue <script setup>, self is not needed
                    // Skip output — field access will provide the name
                } else {
                    if name == "print" {
                        self.needs_print = true;
                    }
                    sink.body.write_all(name.as_bytes())?;
                }
                Ok(())
            } // Binary operations
            Expr::Bina(lhs, op, rhs) => {
                match op {
                    Op::Dot => self.dot(lhs, rhs, sink),
                    _ => {
                        self.expr(lhs, sink)?;
                        sink.body.write(format!(" {} ", op.op()).as_bytes()).to()?;
                        self.expr(rhs, sink)
                    }
                }
            }

            // Plan 056: Dot expression for field access
            Expr::Dot(object, field) => {
                // Handle self.field -> field (Vue <script setup>)
                if let Expr::Ident(name) = object.as_ref() {
                    if name.as_str() == "self" {
                        sink.body.write_all(field.as_bytes())?;
                        return Ok(());
                    }
                }
                // TypeScript uses . for all field access
                self.expr(object, sink)?;
                sink.body.write_all(b".")?;
                sink.body.write_all(field.as_bytes())?;
                Ok(())
            }

            // Unary operations
            Expr::Unary(op, expr) => {
                sink.body.write(format!("{}", op.op()).as_bytes()).to()?;
                self.expr(expr, sink)
            }

            // Function calls
            Expr::Call(call) => self.call(call, sink),

            // Arrays
            Expr::Array(elems) => self.array(elems, sink),

            // Index
            Expr::Index(arr, idx) => self.index(arr, idx, sink),

            // Range (Phase 1)
            Expr::Range(range) => {
                self.needs_range = true;
                sink.body.write(b"range(")?;
                self.expr(&range.start, sink)?;
                sink.body.write(b", ")?;
                self.expr(&range.end, sink)?;
                if range.eq {
                    sink.body.write(b", true")?;
                }
                sink.body.write(b")")?;
                Ok(())
            }

            // Node expression (Phase 1 loops, Phase 3 object construction)
            Expr::Node(node) => {
                if node.name == "loop" {
                    sink.body.write(b"while (true)")?;
                    self.if_body(&node.body, sink)?;
                } else {
                    // Object instantiation: e.g. Point(1, 2) -> new Point(1, 2)
                    sink.body.write(b"new ")?;
                    sink.body.write_all(node.name.as_bytes())?;
                    sink.body.write(b"(")?;
                    for (i, arg) in node.args.args.iter().enumerate() {
                        if i > 0 { sink.body.write(b", ")?; }
                        match arg {
                            Arg::Pos(expr) => self.expr(expr, sink)?,
                            Arg::Name(name) => sink.body.write_all(name.as_bytes())?,
                            Arg::Pair(_key, expr) => self.expr(expr, sink)?, // TypeScript constructors don't have named args like this, pass as positional
                        }
                    }
                    sink.body.write(b")")?;
                }
                Ok(())
            }

            // Pair expression (for object literals)
            Expr::Pair(pair) => {
                match &pair.key {
                    Key::NamedKey(name) => {
                        sink.body.write_all(name.as_bytes())?;
                        sink.body.write(b": ")?;
                    }
                    Key::IntKey(n) => {
                        write!(&mut sink.body, "{}: ", n).to()?;
                    }
                    Key::BoolKey(b) => {
                        write!(&mut sink.body, "{}: ", b).to()?;
                    }
                    Key::StrKey(s) => {
                        write!(&mut sink.body, "\"{}\": ", s).to()?;
                    }
                }
                self.expr(&pair.value, sink)?;
                Ok(())
            }

            // Object literal expression
            Expr::Object(pairs) => {
                sink.body.write(b"{ ")?;
                for (i, pair) in pairs.iter().enumerate() {
                    if i > 0 {
                        sink.body.write(b", ")?;
                    }
                    self.expr(&Expr::Pair(pair.clone()), sink)?;
                }
                sink.body.write(b" }")?;
                Ok(())
            }


            // Lambda expression
            Expr::Lambda(lambda) => {
                sink.body.write(b"(")?;
                for (i, param) in lambda.params.iter().enumerate() {
                    if i > 0 {
                        sink.body.write(b", ")?;
                    }
                    sink.body.write_all(param.name.as_bytes())?;
                    if !matches!(param.ty, Type::Unknown) {
                        sink.body.write(b": ")?;
                        sink.body.write_all(Self::type_to_ts(&param.ty).as_bytes())?;
                    }
                }
                sink.body.write(b")")?;

                if !matches!(lambda.ret, Type::Unknown | Type::Void) {
                    sink.body.write(b": ")?;
                    sink.body.write_all(Self::type_to_ts(&lambda.ret).as_bytes())?;
                } else if matches!(lambda.ret, Type::Void) {
                    sink.body.write(b": void")?;
                }

                sink.body.write(b" => ")?;
                self.body(&lambda.body, sink)?;
                Ok(())
            }

            // Closure expression
            Expr::Closure(closure) => {
                sink.body.write(b"(")?;
                for (i, param) in closure.params.iter().enumerate() {
                    if i > 0 {
                        sink.body.write(b", ")?;
                    }
                    sink.body.write_all(param.name.as_bytes())?;
                    if let Some(ref ty) = param.ty {
                        sink.body.write(b": ")?;
                        sink.body.write_all(Self::type_to_ts(ty).as_bytes())?;
                    }
                }
                sink.body.write(b") => ")?;
                self.expr(&closure.body, sink)?;
                Ok(())
            }

            // Block
            Expr::Block(block) => {
                sink.body.write(b"{")?;
                for stmt in &block.stmts {
                    self.stmt(stmt, sink)?; // Will be defined in ts_stmt
                }
                sink.body.write(b"}")?;
                Ok(())
            }

            // Cover expression (tag construction: Atom.Int(11))
            Expr::Cover(cover) => {
                match cover {
                    crate::ast::Cover::Tag(tag_cover) => {
                        sink.body.write_all(tag_cover.kind.as_bytes())?;
                        sink.body.write(b".")?;
                        sink.body.write_all(tag_cover.tag.as_bytes())?;

                        let real_bindings: Vec<&AutoStr> = tag_cover.bindings.iter()
                            .filter(|b| b.as_str() != "_")
                            .collect();

                        // Scalar enum members are used directly; payload enums use factory functions.
                        let is_scalar = self.scalar_enums.contains(&tag_cover.kind) && real_bindings.is_empty();
                        if !is_scalar {
                            sink.body.write(b"(")?;
                            if real_bindings.len() == 1 {
                                sink.body.write_all(real_bindings[0].as_bytes())?;
                            } else if real_bindings.len() > 1 {
                                for (i, b) in real_bindings.iter().enumerate() {
                                    if i > 0 { sink.body.write(b", ")?; }
                                    sink.body.write_all(b.as_bytes())?;
                                }
                            }
                            sink.body.write(b")")?;
                        }
                    }
                }
                Ok(())
            }

            // Uncover expression (tag destructuring binding)
            Expr::Uncover(uncover) => {
                // Access the value field of a tagged union object
                sink.body.write_all(uncover.src.as_bytes())?;
                sink.body.write(b".value")?;
                Ok(())
            }

            // Box/Arc smart pointer (hardcoded in parser for Rust backend)
            // In TS context, treat as regular constructor calls
            Expr::BoxExpr(inner) => {
                sink.body.write(b"new Box(")?;
                self.expr(inner, sink)?;
                sink.body.write(b")")?;
                Ok(())
            }
            Expr::ArcExpr(inner) => {
                sink.body.write(b"new Arc(")?;
                self.expr(inner, sink)?;
                sink.body.write(b")")?;
                Ok(())
            }

            // Generic name expression (e.g. Pair<int, str> in expression position)
            // Strip generic args — TypeScript infers them at call sites
            Expr::GenName(name) => {
                let base = name.split('<').next().unwrap_or(name);
                sink.body.write_all(base.as_bytes()).map_err(Into::into)
            }

            // Type cast / conversion
            Expr::Cast { expr, target_type } | Expr::To { expr, target_type } => {
                match target_type {
                    Type::Int | Type::Uint | Type::USize
                    | Type::I64 | Type::U64 | Type::Byte
                    | Type::Float | Type::Double => {
                        write!(&mut sink.body, "Number(")?;
                        self.expr(expr, sink)?;
                        sink.body.write(b")")?;
                    }
                    Type::StrFixed(_) | Type::StrOwned | Type::StrSlice | Type::CStrLit => {
                        write!(&mut sink.body, "String(")?;
                        self.expr(expr, sink)?;
                        sink.body.write(b")")?;
                    }
                    _ => {
                        sink.body.write(b"(")?;
                        self.expr(expr, sink)?;
                        sink.body.write(b")")?;
                    }
                }
                Ok(())
            }

            // Null / Nil
            Expr::Null | Expr::Nil => {
                sink.body.write(b"null")?;
                Ok(())
            }

            // Plan 120: Option constructors
            Expr::Some(inner) => {
                // TypeScript: Some(x) → x (value is just the value, null means absent)
                self.expr(inner, sink)
            }
            Expr::None => {
                sink.body.write(b"null")?;
                Ok(())
            }

            // Plan 120: Result constructors
            Expr::Ok(inner) => {
                // TypeScript: Ok(x) → x
                self.expr(inner, sink)
            }
            Expr::Err(msg) => {
                // TypeScript: Err(msg) → new Error(msg)
                sink.body.write(b"new Error(")?;
                self.expr(msg, sink)?;
                sink.body.write(b")")?;
                Ok(())
            }

            // Plan 120: Null coalescing (??)
            Expr::NullCoalesce(left, right) => {
                self.expr(left, sink)?;
                sink.body.write(b" ?? ")?;
                self.expr(right, sink)?;
                Ok(())
            }

            // Plan 120: Error propagate (?.)
            Expr::ErrorPropagate(inner) => {
                self.expr(inner, sink)?;
                Ok(())
            }

            // Plan 124: Async block
            Expr::AsyncBlock { body, return_type } => {
                sink.body.write(b"(async ()")?;
                if let Some(ret) = return_type {
                    sink.body.write(b": Promise<")?;
                    sink.body.write_all(Self::type_to_ts(ret).as_bytes())?;
                    sink.body.write(b">")?;
                }
                sink.body.write(b"")?;
                self.body(body, sink)?;
                sink.body.write(b")()")?;
                Ok(())
            }

            // Plan 120: Option/Result patterns in is statements
            Expr::OptionPattern(cover) => {
                match cover.variant {
                    crate::ast::cover::OptionVariant::Some => {
                        if let Some(ref binding) = cover.binding {
                            // TypeScript: destructuring check
                            write!(&mut sink.body, "{{ _tag: \"Some\", value: {} }}", binding)?;
                        } else {
                            sink.body.write(b"{ _tag: \"Some\" }")?;
                        }
                    }
                    crate::ast::cover::OptionVariant::None => {
                        sink.body.write(b"null")?;
                    }
                }
                Ok(())
            }
            Expr::ResultPattern(cover) => {
                match cover.variant {
                    crate::ast::cover::ResultVariant::Ok => {
                        if let Some(ref binding) = cover.binding {
                            write!(&mut sink.body, "{{ _tag: \"Ok\", value: {} }}", binding)?;
                        } else {
                            sink.body.write(b"{ _tag: \"Ok\" }")?;
                        }
                    }
                    crate::ast::cover::ResultVariant::Err => {
                        if let Some(ref binding) = cover.binding {
                            write!(&mut sink.body, "{{ _tag: \"Err\", value: {} }}", binding)?;
                        } else {
                            sink.body.write(b"{ _tag: \"Err\" }")?;
                        }
                    }
                }
                Ok(())
            }
            Expr::OptionUncover(uncover) => {
                // Access value from Some pattern
                sink.body.write_all(uncover.src.as_bytes())?;
                sink.body.write(b".value")?;
                Ok(())
            }
            Expr::ResultUncover(uncover) => {
                // Access value from Ok/Err pattern
                sink.body.write_all(uncover.src.as_bytes())?;
                sink.body.write(b".value")?;
                Ok(())
            }
            Expr::StructPattern(_sc) => {
                // Struct destructuring - output as a comment/placeholder
                sink.body.write(b"/* struct pattern */")?;
                Ok(())
            }

            // Plan 124: Await expression
            Expr::Await { expr } => {
                sink.body.write(b"(await ")?;
                self.expr(expr, sink)?;
                sink.body.write(b")")?;
                Ok(())
            }

            // Plan 200: Tuple expression
            Expr::Tuple(elems) => {
                sink.body.write(b"[")?;
                for (i, elem) in elems.iter().enumerate() {
                    if i > 0 { sink.body.write(b", ")?; }
                    self.expr(elem, sink)?;
                }
                sink.body.write(b"]")?;
                Ok(())
            }

            // Unsupported expressions
            _ => Err(format!("TypeScript Transpiler: unsupported expression: {}", expr).into()),
        }
    }

    pub fn fstr(&mut self, fstr: &FStr, sink: &mut Sink) -> AutoResult<()> {

        let out = &mut sink.body;
        sink.body.write(b"`")?;
        for part in &fstr.parts {
            match part {
                Expr::Str(s) => {
                    let escaped = s.replace("`", "\\`").replace("${", "\\${");
                    sink.body.write_all(escaped.as_bytes())?;
                }
                Expr::Char(c) => {
                    sink.body.write_all(c.to_string().as_bytes())?;
                }
                _ => {
                    sink.body.write(b"${")?;
                    self.expr(part, sink)?;
                    sink.body.write(b"}")?;
                }
            }
        }
        sink.body.write(b"`")?;
        Ok(())
    }

    pub fn call(&mut self, call: &Call, sink: &mut Sink) -> AutoResult<()> {

        let out = &mut sink.body;
        // Check if this is a print call and convert to console.log
        let is_print = matches!(&*call.name, Expr::Ident(name) if name == "print");

        if is_print {
            sink.body.write(b"console.log")?;
        } else {
            self.expr(&call.name, sink)?;
        }

        sink.body.write(b"(")?;

        for (i, arg) in call.args.args.iter().enumerate() {
            if i > 0 {
                sink.body.write(b", ")?;
            }
            self.arg(arg, sink)?;
        }

        sink.body.write(b")")?;
        Ok(())
    }

    pub fn arg(&mut self, arg: &Arg, sink: &mut Sink) -> AutoResult<()> {

        let out = &mut sink.body;
        match arg {
            Arg::Pos(expr) => self.expr(expr, sink),
            Arg::Name(name) => sink.body.write_all(name.as_bytes()).map_err(Into::into),
            Arg::Pair(key, expr) => {
                sink.body.write_all(key.as_bytes())?;
                sink.body.write(b": ")?;
                self.expr(expr, sink)?;
                Ok(())
            }
        }
    }

    pub fn array(&mut self, elems: &[Expr], sink: &mut Sink) -> AutoResult<()> {

        let out = &mut sink.body;
        sink.body.write(b"[")?;

        for (i, elem) in elems.iter().enumerate() {
            if i > 0 {
                sink.body.write(b", ")?;
            }
            self.expr(elem, sink)?;
        }

        sink.body.write(b"]")?;
        Ok(())
    }

    pub fn index(&mut self, arr: &Box<Expr>, idx: &Box<Expr>, sink: &mut Sink) -> AutoResult<()> {

        let out = &mut sink.body;
        self.expr(arr, sink)?;
        sink.body.write(b"[")?;
        self.expr(idx, sink)?;
        sink.body.write(b"]")?;
        Ok(())
    }

    pub fn dot(&mut self, lhs: &Expr, rhs: &Expr, sink: &mut Sink) -> AutoResult<()> {

        let out = &mut sink.body;
        self.expr(lhs, sink)?;
        sink.body.write(b".")?;
        self.expr(rhs, sink)?;
        Ok(())
    }
}
