use crate::ast::*;
use crate::AutoResult;
use auto_val::Op;
use std::io::Write;
use super::{TypeScriptTrans, ToStrError};

impl TypeScriptTrans {
    pub fn expr(&mut self, expr: &Expr, out: &mut impl Write) -> AutoResult<()> {
        match expr {
            // Literals
            Expr::Int(i) => write!(out, "{}", i).map_err(Into::into),
            Expr::Uint(u) => write!(out, "{}", u).map_err(Into::into),
            Expr::Float(f, _) => write!(out, "{}", f).map_err(Into::into),
            Expr::Double(d, _) => write!(out, "{}", d).map_err(Into::into),
            Expr::Bool(b) => write!(out, "{}", b).map_err(Into::into),
            Expr::Char(c) => write!(out, "'{}'", c).map_err(Into::into),
            Expr::Str(s) => write!(out, "\"{}\"", s).map_err(Into::into),
            Expr::CStr(s) => write!(out, "\"{}\"", s).map_err(Into::into),

            // F-strings → Template literals (perfect match!)
            Expr::FStr(fstr) => self.fstr(fstr, out),

            // Identifier
            Expr::Ident(name) => {
                if name == "self" {
                    out.write(b"this")?;
                } else {
                    if name == "print" {
                        self.needs_print = true;
                    }
                    out.write_all(name.as_bytes())?;
                }
                Ok(())
            } // Binary operations
            Expr::Bina(lhs, op, rhs) => {
                match op {
                    Op::Dot => self.dot(lhs, rhs, out),
                    _ => {
                        self.expr(lhs, out)?;
                        out.write(format!(" {} ", op.op()).as_bytes()).to()?;
                        self.expr(rhs, out)
                    }
                }
            }

            // Plan 056: Dot expression for field access
            Expr::Dot(object, field) => {
                // TypeScript uses . for all field access
                self.expr(object, out)?;
                out.write_all(b".")?;
                out.write_all(field.as_bytes())?;
                Ok(())
            }

            // Unary operations
            Expr::Unary(op, expr) => {
                out.write(format!("{}", op.op()).as_bytes()).to()?;
                self.expr(expr, out)
            }

            // Function calls
            Expr::Call(call) => self.call(call, out),

            // Arrays
            Expr::Array(elems) => self.array(elems, out),

            // Index
            Expr::Index(arr, idx) => self.index(arr, idx, out),

            // Range (Phase 1)
            Expr::Range(range) => {
                self.needs_range = true;
                out.write(b"range(")?;
                self.expr(&range.start, out)?;
                out.write(b", ")?;
                self.expr(&range.end, out)?;
                if range.eq {
                    out.write(b", true")?;
                }
                out.write(b")")?;
                Ok(())
            }

            // Node expression (Phase 1 loops, Phase 3 object construction)
            Expr::Node(node) => {
                if node.name == "loop" {
                    out.write(b"while (true)")?;
                    self.if_body(&node.body, out)?;
                } else {
                    // Object instantiation: e.g. Point(1, 2) -> new Point(1, 2)
                    out.write(b"new ")?;
                    out.write_all(node.name.as_bytes())?;
                    out.write(b"(")?;
                    for (i, arg) in node.args.args.iter().enumerate() {
                        if i > 0 { out.write(b", ")?; }
                        match arg {
                            Arg::Pos(expr) => self.expr(expr, out)?,
                            Arg::Name(name) => out.write_all(name.as_bytes())?,
                            Arg::Pair(_key, expr) => self.expr(expr, out)?, // TypeScript constructors don't have named args like this, pass as positional
                        }
                    }
                    out.write(b")")?;
                }
                Ok(())
            }

            // Pair expression (for object literals)
            Expr::Pair(pair) => {
                match &pair.key {
                    Key::NamedKey(name) => {
                        out.write_all(name.as_bytes())?;
                        out.write(b": ")?;
                    }
                    Key::IntKey(n) => {
                        write!(out, "{}: ", n).to()?;
                    }
                    Key::BoolKey(b) => {
                        write!(out, "{}: ", b).to()?;
                    }
                    Key::StrKey(s) => {
                        write!(out, "\"{}\": ", s).to()?;
                    }
                }
                self.expr(&pair.value, out)?;
                Ok(())
            }

            // Object literal expression
            Expr::Object(pairs) => {
                out.write(b"{ ")?;
                for (i, pair) in pairs.iter().enumerate() {
                    if i > 0 {
                        out.write(b", ")?;
                    }
                    self.expr(&Expr::Pair(pair.clone()), out)?;
                }
                out.write(b" }")?;
                Ok(())
            }


            // Lambda expression
            Expr::Lambda(lambda) => {
                out.write(b"(")?;
                for (i, param) in lambda.params.iter().enumerate() {
                    if i > 0 {
                        out.write(b", ")?;
                    }
                    out.write_all(param.name.as_bytes())?;
                    if !matches!(param.ty, Type::Unknown) {
                        out.write(b": ")?;
                        out.write_all(Self::type_to_ts(&param.ty).as_bytes())?;
                    }
                }
                out.write(b")")?;

                if !matches!(lambda.ret, Type::Unknown | Type::Void) {
                    out.write(b": ")?;
                    out.write_all(Self::type_to_ts(&lambda.ret).as_bytes())?;
                } else if matches!(lambda.ret, Type::Void) {
                    out.write(b": void")?;
                }

                out.write(b" => ")?;
                self.body(&lambda.body, out)?;
                Ok(())
            }

            // Closure expression
            Expr::Closure(closure) => {
                out.write(b"(")?;
                for (i, param) in closure.params.iter().enumerate() {
                    if i > 0 {
                        out.write(b", ")?;
                    }
                    out.write_all(param.name.as_bytes())?;
                    if let Some(ref ty) = param.ty {
                        out.write(b": ")?;
                        out.write_all(Self::type_to_ts(ty).as_bytes())?;
                    }
                }
                out.write(b") => ")?;
                self.expr(&closure.body, out)?;
                Ok(())
            }

            // Block
            Expr::Block(block) => {
                out.write(b"{")?;
                for stmt in &block.stmts {
                    self.stmt(stmt, out)?; // Will be defined in ts_stmt
                }
                out.write(b"}")?;
                Ok(())
            }

            // Cover expression (tag construction: Atom.Int(11))
            Expr::Cover(cover) => {
                match cover {
                    crate::ast::Cover::Tag(tag_cover) => {
                        // Atom.Int(11) -> Atom.Int(11) where Atom is a factory const
                        out.write_all(tag_cover.kind.as_bytes())?;
                        out.write(b".")?;
                        out.write_all(tag_cover.tag.as_bytes())?;
                        out.write(b"(")?;
                        out.write_all(tag_cover.elem.as_bytes())?;
                        out.write(b")")?;
                    }
                }
                Ok(())
            }

            // Uncover expression (tag destructuring binding)
            Expr::Uncover(uncover) => {
                // Access the value field of a tagged union object
                out.write_all(uncover.src.as_bytes())?;
                out.write(b".value")?;
                Ok(())
            }

            // Box/Arc smart pointer (hardcoded in parser for Rust backend)
            // In TS context, treat as regular constructor calls
            Expr::BoxExpr(inner) => {
                out.write(b"new Box(")?;
                self.expr(inner, out)?;
                out.write(b")")?;
                Ok(())
            }
            Expr::ArcExpr(inner) => {
                out.write(b"new Arc(")?;
                self.expr(inner, out)?;
                out.write(b")")?;
                Ok(())
            }

            // Generic name expression (e.g. Pair<int, str> in expression position)
            // Strip generic args — TypeScript infers them at call sites
            Expr::GenName(name) => {
                let base = name.split('<').next().unwrap_or(name);
                out.write_all(base.as_bytes()).map_err(Into::into)
            }

            // Unsupported expressions
            _ => Err(format!("TypeScript Transpiler: unsupported expression: {}", expr).into()),
        }
    }

    pub fn fstr(&mut self, fstr: &FStr, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"`")?;
        for part in &fstr.parts {
            match part {
                Expr::Str(s) => {
                    let escaped = s.replace("`", "\\`").replace("${", "\\${");
                    out.write_all(escaped.as_bytes())?;
                }
                Expr::Char(c) => {
                    out.write_all(c.to_string().as_bytes())?;
                }
                _ => {
                    out.write(b"${")?;
                    self.expr(part, out)?;
                    out.write(b"}")?;
                }
            }
        }
        out.write(b"`")?;
        Ok(())
    }

    pub fn call(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
        // Check if this is a print call and convert to console.log
        let is_print = matches!(&*call.name, Expr::Ident(name) if name == "print");

        if is_print {
            out.write(b"console.log")?;
        } else {
            self.expr(&call.name, out)?;
        }

        out.write(b"(")?;

        for (i, arg) in call.args.args.iter().enumerate() {
            if i > 0 {
                out.write(b", ")?;
            }
            self.arg(arg, out)?;
        }

        out.write(b")")?;
        Ok(())
    }

    pub fn arg(&mut self, arg: &Arg, out: &mut impl Write) -> AutoResult<()> {
        match arg {
            Arg::Pos(expr) => self.expr(expr, out),
            Arg::Name(name) => out.write_all(name.as_bytes()).map_err(Into::into),
            Arg::Pair(key, expr) => {
                out.write_all(key.as_bytes())?;
                out.write(b": ")?;
                self.expr(expr, out)?;
                Ok(())
            }
        }
    }

    pub fn array(&mut self, elems: &[Expr], out: &mut impl Write) -> AutoResult<()> {
        out.write(b"[")?;

        for (i, elem) in elems.iter().enumerate() {
            if i > 0 {
                out.write(b", ")?;
            }
            self.expr(elem, out)?;
        }

        out.write(b"]")?;
        Ok(())
    }

    pub fn index(&mut self, arr: &Box<Expr>, idx: &Box<Expr>, out: &mut impl Write) -> AutoResult<()> {
        self.expr(arr, out)?;
        out.write(b"[")?;
        self.expr(idx, out)?;
        out.write(b"]")?;
        Ok(())
    }

    pub fn dot(&mut self, lhs: &Expr, rhs: &Expr, out: &mut impl Write) -> AutoResult<()> {
        self.expr(lhs, out)?;
        out.write(b".")?;
        self.expr(rhs, out)?;
        Ok(())
    }
}
