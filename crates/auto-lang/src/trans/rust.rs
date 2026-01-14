use super::{Sink, Trans};
use crate::ast::*;
use crate::parser::Parser;
use crate::universe::Universe;
use crate::AutoResult;
use auto_val::{shared, Shared};
use auto_val::{AutoStr, Op};
use std::collections::HashSet;
use std::io::Write;

pub enum RustEdition {
    E2021,
    E2024,
}

pub struct RustTrans {
    indent: usize,
    uses: HashSet<AutoStr>,
    scope: Shared<crate::universe::Universe>,
    edition: RustEdition,
}

impl RustTrans {
    pub fn new(_name: AutoStr) -> Self {
        Self {
            indent: 0,
            uses: HashSet::new(),
            scope: shared(crate::universe::Universe::default()),
            edition: RustEdition::E2021,
        }
    }

    pub fn set_scope(&mut self, scope: Shared<crate::universe::Universe>) {
        self.scope = scope;
    }

    pub fn set_edition(&mut self, edition: RustEdition) {
        self.edition = edition;
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

    fn rust_type_name(&self, ty: &Type) -> String {
        match ty {
            Type::Byte => "u8".to_string(),
            Type::Int => "i32".to_string(),
            Type::Uint => "u32".to_string(),
            Type::USize => "usize".to_string(),
            Type::Float | Type::Double => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Char => "char".to_string(),
            Type::Str(_) => "String".to_string(),
            Type::CStr => "&str".to_string(),
            Type::Array(arr) => {
                format!("[{}; {}]", self.rust_type_name(&arr.elem), arr.len)
            }
            Type::Ptr(ptr) => {
                // Check if we need reference or Box
                match &*ptr.of.borrow() {
                    Type::User(_) => format!("Box<{}>", self.rust_type_name(&*ptr.of.borrow())),
                    _ => format!("&{}", self.rust_type_name(&*ptr.of.borrow())),
                }
            }
            Type::User(usr) => usr.name.to_string(),
            Type::Enum(en) => en.borrow().name.to_string(),
            Type::Spec(spec) => format!("dyn {}", spec.borrow().name),  // Spec 作为 trait object
            Type::Union(u) => u.name.to_string(),
            Type::Tag(t) => t.borrow().name.to_string(),
            Type::Void => "()".to_string(),
            Type::Unknown => "/* unknown */".to_string(),
            Type::CStruct(decl) => decl.name.to_string(),
            Type::Linear(inner) => {
                // Linear types unwrap to their inner type for transpilation
                // The move semantics are enforced by AutoLang's ownership system
                self.rust_type_name(inner)
            }
        }
    }

    fn expr(&mut self, expr: &Expr, out: &mut impl Write) -> AutoResult<()> {
        match expr {
            // Literals
            Expr::Int(i) => write!(out, "{}", i).map_err(Into::into),
            Expr::Uint(u) => write!(out, "{}", u).map_err(Into::into),
            Expr::I8(i) => write!(out, "{}", i).map_err(Into::into),
            Expr::U8(u) => write!(out, "{}", u).map_err(Into::into),
            Expr::I64(i) => write!(out, "{}", i).map_err(Into::into),
            Expr::Byte(b) => write!(out, "{}", b).map_err(Into::into),
            Expr::Float(f, _) => write!(out, "{}", f).map_err(Into::into),
            Expr::Double(d, _) => write!(out, "{}", d).map_err(Into::into),
            Expr::Bool(b) => write!(out, "{}", b).map_err(Into::into),
            Expr::Char(c) => if *c == '\n' {
                write!(out, "'\\n'")
            } else if *c == '\t' {
                write!(out, "'\\t'")
            } else if *c == '\\' {
                write!(out, "'\\\\'")
            } else {
                write!(out, "'{}'", c)
            }
            .map_err(Into::into),
            Expr::Str(s) => write!(out, "\"{}\"", s).map_err(Into::into),
            Expr::CStr(s) => write!(out, "\"{}\"", s).map_err(Into::into),
            Expr::Ident(name) => write!(out, "{}", name).map_err(Into::into),
            Expr::GenName(name) => write!(out, "{}", name).map_err(Into::into),
            Expr::Nil => write!(out, "None").map_err(Into::into),
            Expr::Null => write!(out, "None").map_err(Into::into),

            // Operators
            Expr::Bina(lhs, op, rhs) => {
                match op {
                    Op::Dot => {
                        // Member access: expr.field or .field (shorthand for self.field)
                        match lhs.as_ref() {
                            Expr::Nil | Expr::Null => {
                                // .field -> self.field
                                write!(out, "self.")?;
                                self.expr(rhs, out)?;
                            }
                            _ => {
                                // Check if this is enum variant access: Type::Variant
                                // Use :: if rhs is an identifier starting with uppercase (enum variant convention)
                                let is_enum_variant = if let Expr::Ident(rhs_name) = rhs.as_ref() {
                                    rhs_name
                                        .chars()
                                        .next()
                                        .map(|c| c.is_uppercase())
                                        .unwrap_or(false)
                                } else {
                                    false
                                };

                                if matches!(lhs.as_ref(), Expr::Ident(_)) && is_enum_variant {
                                    self.expr(lhs, out)?;
                                    write!(out, "::")?;
                                    self.expr(rhs, out)?;
                                } else {
                                    // expr.field or expr.method()
                                    self.expr(lhs, out)?;
                                    write!(out, ".")?;
                                    self.expr(rhs, out)?;
                                }
                            }
                        }
                    }
                    Op::Range => {
                        // Range: start..end
                        self.expr(lhs, out)?;
                        write!(out, "..")?;
                        self.expr(rhs, out)?;
                    }
                    Op::RangeEq => {
                        // Inclusive range: start..=end
                        self.expr(lhs, out)?;
                        write!(out, "..=")?;
                        self.expr(rhs, out)?;
                    }
                    _ => {
                        // Binary operators: lhs OP rhs
                        self.expr(lhs, out)?;
                        write!(out, " {} ", op.op())?;
                        self.expr(rhs, out)?;
                    }
                }
                Ok(())
            }

            Expr::Unary(op, expr) => {
                write!(out, "{}", op.op())?;
                self.expr(expr, out)?;
                Ok(())
            }

            // Collections
            Expr::Array(arr) => {
                write!(out, "[")?;
                for (i, elem) in arr.iter().enumerate() {
                    self.expr(elem, out)?;
                    if i < arr.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, "]").map_err(Into::into)
            }

            Expr::Index(arr, idx) => {
                self.expr(arr, out)?;
                write!(out, "[")?;
                self.expr(idx, out)?;
                write!(out, "]").map_err(Into::into)
            }

            Expr::Range(range) => {
                self.expr(&range.start, out)?;
                if range.eq {
                    write!(out, "..=")?;
                } else {
                    write!(out, "..")?;
                }
                self.expr(&range.end, out).map_err(Into::into)
            }

            Expr::Pair(pair) => {
                // Pair expression: key: value
                let key = match &pair.key {
                    crate::ast::Key::NamedKey(name) => name.clone(),
                    crate::ast::Key::IntKey(n) => format!("{}", n).into(),
                    crate::ast::Key::BoolKey(b) => format!("{}", b).into(),
                    crate::ast::Key::StrKey(s) => s.clone(),
                };
                write!(out, "{}: ", key)?;
                self.expr(&pair.value, out).map_err(Into::into)
            }

            Expr::Object(pairs) => {
                // Object literal: {key1: value1, key2: value2}
                write!(out, "{{")?;
                for (i, pair) in pairs.iter().enumerate() {
                    self.expr(&Expr::Pair(pair.clone()), out)?;
                    if i < pairs.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, "}}").map_err(Into::into)
            }

            Expr::Grid(grid) => {
                // Grid expression: 2D array
                // Convert to nested vec: vec![vec![...], ...]
                write!(out, "vec![")?;
                for (i, row) in grid.data.iter().enumerate() {
                    write!(out, "vec![")?;
                    for (j, cell) in row.iter().enumerate() {
                        self.expr(cell, out)?;
                        if j < row.len() - 1 {
                            write!(out, ", ")?;
                        }
                    }
                    write!(out, "]")?;
                    if i < grid.data.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, "]").map_err(Into::into)
            }

            Expr::Cover(cover) => {
                // Cover expression for tagged unions
                match cover {
                    crate::ast::Cover::Tag(tag_cover) => {
                        write!(out, "/* TagCover: {} {} {} */",
                               tag_cover.kind, tag_cover.tag, tag_cover.elem).map_err(Into::into)
                    }
                }
            }

            Expr::Uncover(uncover) => {
                // Uncover expression for pattern matching
                write!(out, "/* TagUncover: {} */", uncover.src).map_err(Into::into)
            }

            Expr::Ref(name) => {
                // Reference expression: &name
                write!(out, "&{}", name).map_err(Into::into)
            }

            // Struct construction: Point(1, 2) -> Point { x: 1, y: 2 }
            // Special case: loop { body } -> loop { body }
            Expr::Node(node) => {
                // Check if this is a loop expression
                if node.name == "loop" {
                    write!(out, "loop {{")?;
                    if !node.body.stmts.is_empty() {
                        write!(out, "\n")?;
                        self.indent();

                        for stmt in &node.body.stmts {
                            self.print_indent(out)?;
                            match stmt {
                                Stmt::Expr(expr) => {
                                    self.expr(expr, out)?;
                                    out.write(b";\n")?;
                                }
                                Stmt::Store(store) => {
                                    self.store(store, out)?;
                                    out.write(b";\n")?;
                                }
                                Stmt::Break => {
                                    out.write(b"break;\n")?;
                                }
                                _ => {
                                    // For other statement types, format inline
                                    match stmt {
                                        Stmt::If(if_) => {
                                            // Inline if statement
                                            write!(out, "if ")?;
                                            for (i, branch) in if_.branches.iter().enumerate() {
                                                if i == 0 {
                                                } else {
                                                    write!(out, " else if ")?;
                                                }
                                                self.expr(&branch.cond, out)?;
                                                write!(out, " {{ ")?;
                                                // Single statement body
                                                if let Some(stmt) = branch.body.stmts.first() {
                                                    match stmt {
                                                        Stmt::Expr(expr) => {
                                                            self.expr(expr, out)?;
                                                        }
                                                        Stmt::Break => {
                                                            write!(out, "break")?;
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                                write!(out, " }}; ")?;
                                            }
                                            write!(out, "}};\n")?;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }

                        self.dedent();
                        self.print_indent(out)?;
                    }
                    write!(out, "}}")
                } else {
                    // Regular struct construction
                    write!(out, "{} {{", node.name)?;
                    if !node.args.args.is_empty() || !node.body.stmts.is_empty() {
                        write!(out, " ")?;
                    }

                    // Try to get type declaration to map positional args to field names
                    let type_decl = self.scope.borrow().lookup_type(&node.name);

                    for (i, arg) in node.args.args.iter().enumerate() {
                        match arg {
                            Arg::Pos(expr) => {
                                // Positional arg - map to actual field name from type definition
                                let field_name = if let Type::User(decl) = &type_decl {
                                    if i < decl.members.len() {
                                        decl.members[i].name.clone()
                                    } else {
                                        format!("field{}", i).into()
                                    }
                                } else {
                                    format!("field{}", i).into()
                                };
                                write!(out, "{}: ", field_name)?;
                                self.expr(expr, out)?;
                            }
                            Arg::Name(name) => {
                                // Named arg without value
                                write!(out, "{}: ", name)?;
                            }
                            Arg::Pair(key, expr) => {
                                // Named argument: field: value
                                write!(out, "{}: ", key)?;
                                self.expr(expr, out)?;
                            }
                        }
                        if i < node.args.args.len() - 1 || !node.body.stmts.is_empty() {
                            write!(out, ", ")?;
                        }
                    }

                    // Handle body statements (field initializers)
                    for (i, stmt) in node.body.stmts.iter().enumerate() {
                        match stmt {
                            Stmt::Store(store) => {
                                write!(out, "{}: ", store.name)?;
                                self.expr(&store.expr, out)?;
                            }
                            Stmt::Expr(Expr::Pair(pair)) => {
                                // Named field initializer: x: 3
                                let field_name = match &pair.key {
                                    crate::ast::Key::NamedKey(name) => name.clone(),
                                    crate::ast::Key::IntKey(n) => format!("{}", n).into(),
                                    crate::ast::Key::BoolKey(b) => format!("{}", b).into(),
                                    crate::ast::Key::StrKey(s) => s.clone(),
                                };
                                write!(out, "{}: ", field_name)?;
                                self.expr(&pair.value, out)?;
                            }
                            _ => {}
                        }
                        if i < node.body.stmts.len() - 1 {
                            write!(out, ", ")?;
                        }
                    }

                    if !node.args.args.is_empty() || !node.body.stmts.is_empty() {
                        write!(out, " ")?;
                    }
                    write!(out, "}}")
                }.map_err(Into::into)
            }

            // Function calls
            Expr::Call(call) => self.call(call, out),

            // F-strings: f"hello $name" -> format!("hello {}", name)
            Expr::FStr(fstr) => {
                write!(out, "format!(\"")?;
                let mut _arg_count = 0;
                for part in &fstr.parts {
                    match part {
                        Expr::Str(s) | Expr::CStr(s) => {
                            write!(out, "{}", s.replace("\"", r##"\""##))?;
                        }
                        Expr::Char(c) => {
                            write!(out, "{}", c)?;
                        }
                        _ => {
                            // Expression placeholder
                            write!(out, "{{}}")?;
                            _arg_count += 1;
                        }
                    }
                }
                write!(out, "\"")?;

                // Add arguments after format string
                for part in &fstr.parts {
                    match part {
                        Expr::Str(_) | Expr::CStr(_) | Expr::Char(_) => {}
                        _ => {
                            write!(out, ", ")?;
                            self.expr(part, out)?;
                        }
                    }
                }

                write!(out, ")").map_err(Into::into)
            }

            // Control flow (stub for now)
            Expr::If(if_) => {
                // Transpile if/else if/else chains
                for (i, branch) in if_.branches.iter().enumerate() {
                    if i == 0 {
                        write!(out, "if ")?;
                    } else {
                        write!(out, " else if ")?;
                    }

                    // Condition
                    self.expr(&branch.cond, out)?;
                    write!(out, " {{")?;

                    // Body
                    if !branch.body.stmts.is_empty() {
                        write!(out, "\n")?;
                        self.indent();
                        let stmt_count = branch.body.stmts.len();
                        for (i, stmt) in branch.body.stmts.iter().enumerate() {
                            self.print_indent(out)?;
                            // Handle different statement types
                            let is_last = i == stmt_count - 1;
                            match stmt {
                                Stmt::Expr(Expr::If(inner_if)) => {
                                    // If expression - don't add semicolon
                                    self.expr(&Expr::If(inner_if.clone()), out)?;
                                    out.write(b"\n")?;
                                }
                                Stmt::Expr(expr) => {
                                    self.expr(expr, out)?;
                                    // Check if this expression needs a semicolon
                                    let needs_semicolon = if !is_last {
                                        true
                                    } else {
                                        // Check if it's a call to print or other void functions
                                        match expr {
                                            Expr::Call(call) => {
                                                if let Expr::Ident(name) = call.name.as_ref() {
                                                    name == "print" || name == "println"
                                                } else {
                                                    false
                                                }
                                            }
                                            _ => false,
                                        }
                                    };

                                    if needs_semicolon {
                                        out.write(b";\n")?;
                                    } else {
                                        out.write(b"\n")?;
                                    }
                                }
                                Stmt::If(inner_if) => {
                                    // Nested if statement - handle as expression
                                    self.expr(&Expr::If(inner_if.clone()), out)?;
                                    out.write(b"\n")?;
                                }
                                Stmt::Store(store) => {
                                    self.store(store, out)?;
                                    out.write(b";\n")?;
                                }
                                _ => {
                                    // Other statement types - write directly
                                    write!(out, "/* unsupported statement in if body */\n")?;
                                }
                            }
                        }
                        self.dedent();
                        self.print_indent(out)?;
                    }
                    write!(out, "}}")?;
                }

                // Else clause
                if let Some(else_body) = &if_.else_ {
                    write!(out, "\nelse {{")?;
                    if !else_body.stmts.is_empty() {
                        write!(out, "\n")?;
                        self.indent();
                        let stmt_count = else_body.stmts.len();
                        for (i, stmt) in else_body.stmts.iter().enumerate() {
                            self.print_indent(out)?;
                            let is_last = i == stmt_count - 1;
                            match stmt {
                                Stmt::Expr(Expr::If(inner_if)) => {
                                    // Nested if expression in else
                                    self.expr(&Expr::If(inner_if.clone()), out)?;
                                    out.write(b"\n")?;
                                }
                                Stmt::Expr(expr) => {
                                    self.expr(expr, out)?;
                                    // Check if this expression needs a semicolon
                                    let needs_semicolon = if !is_last {
                                        true
                                    } else {
                                        // Check if it's a call to print or other void functions
                                        match expr {
                                            Expr::Call(call) => {
                                                if let Expr::Ident(name) = call.name.as_ref() {
                                                    name == "print" || name == "println"
                                                } else {
                                                    false
                                                }
                                            }
                                            _ => false,
                                        }
                                    };

                                    if needs_semicolon {
                                        out.write(b";\n")?;
                                    } else {
                                        out.write(b"\n")?;
                                    }
                                }
                                Stmt::If(inner_if) => {
                                    // Nested if statement - handle as expression
                                    self.expr(&Expr::If(inner_if.clone()), out)?;
                                    out.write(b"\n")?;
                                }
                                Stmt::Store(store) => {
                                    self.store(store, out)?;
                                    out.write(b";\n")?;
                                }
                                _ => {
                                    write!(out, "/* unsupported statement in else body */\n")?;
                                }
                            }
                        }
                        self.dedent();
                        self.print_indent(out)?;
                    }
                    write!(out, "}}\n")?;
                } else {
                    // No else clause - add newline after closing brace
                    write!(out, "\n")?;
                }

                Ok(())
            }

            // Lambda/closure: |params| body
            Expr::Lambda(lambda) => {
                write!(out, "|")?;
                for (i, param) in lambda.params.iter().enumerate() {
                    write!(out, "{}", self.rust_type_name(&param.ty))?;
                    write!(out, " {}", param.name)?;
                    if i < lambda.params.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, "| ")?;

                // Lambda body - if it's a single expression, write it directly
                if lambda.body.stmts.len() == 1 {
                    match &lambda.body.stmts[0] {
                        Stmt::Expr(expr) => {
                            self.expr(expr, out)?;
                        }
                        Stmt::Store(store) => {
                            self.store(store, out)?;
                        }
                        _ => {
                            write!(out, "{{ /* unsupported lambda body */ }}")?;
                        }
                    }
                } else {
                    // Multiple statements - use block
                    write!(out, "{{ ")?;
                    for (i, stmt) in lambda.body.stmts.iter().enumerate() {
                        match stmt {
                            Stmt::Expr(expr) => {
                                self.expr(expr, out)?;
                                if i < lambda.body.stmts.len() - 1 {
                                    write!(out, "; ")?;
                                }
                            }
                            Stmt::Store(store) => {
                                self.store(store, out)?;
                                write!(out, "; ")?;
                            }
                            _ => {
                                write!(out, "/* unsupported statement */ ")?;
                            }
                        }
                    }
                    write!(out, "}}")?;
                }
                Ok(())
            }

            _ => Err(format!("Rust Transpiler: unsupported expression: {}", expr).into()),
        }
    }

    fn call(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
        // Special case for print function
        if let Expr::Ident(name) = call.name.as_ref() {
            if name == "print" {
                return self.print_call(call, out);
            }
        }

        // Normal call
        self.expr(&call.name, out)?;
        write!(out, "(")?;
        for (i, arg) in call.args.args.iter().enumerate() {
            self.arg(arg, out)?;
            if i < call.args.args.len() - 1 {
                write!(out, ", ")?;
            }
        }
        write!(out, ")").map_err(Into::into)
    }

    fn arg(&mut self, arg: &Arg, out: &mut impl Write) -> AutoResult<()> {
        match arg {
            Arg::Pos(expr) => self.expr(expr, out),
            Arg::Name(name) => write!(out, "{}", name).map_err(Into::into),
            Arg::Pair(_, expr) => self.expr(expr, out),
        }
    }

    fn print_call(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
        // print("hello") -> println!("hello")
        // print(value) -> println!("{}", value)
        // print(f"...") -> println!("...", args)
        // print("text:", value) -> println!("text: {}", value)

        if call.args.args.is_empty() {
            write!(out, "println!()")?;
            return Ok(());
        }

        // Check if first argument is an f-string
        if let Arg::Pos(first_arg) = &call.args.args[0] {
            if let Expr::FStr(fstr) = first_arg {
                // Generate println! with f-string format
                write!(out, "println!(\"")?;

                // Build format string from f-string parts
                for part in &fstr.parts {
                    match part {
                        Expr::Str(s) | Expr::CStr(s) => {
                            write!(out, "{}", s.replace("\"", r##"\""##))?;
                        }
                        Expr::Char(c) => {
                            write!(out, "{}", c)?;
                        }
                        _ => {
                            // Expression placeholder
                            write!(out, "{{}}")?;
                        }
                    }
                }
                write!(out, "\"")?;

                // Add f-string arguments
                for part in &fstr.parts {
                    match part {
                        Expr::Str(_) | Expr::CStr(_) | Expr::Char(_) => {}
                        _ => {
                            write!(out, ", ")?;
                            self.expr(part, out)?;
                        }
                    }
                }

                // Add additional arguments (after f-string)
                for arg in call.args.args.iter().skip(1) {
                    write!(out, ", ")?;
                    self.arg(arg, out)?;
                }

                write!(out, ")")?;
                return Ok(());
            }
        }

        if call.args.args.len() == 1 {
            if let Arg::Pos(expr) = &call.args.args[0] {
                match expr {
                    Expr::Str(s) | Expr::CStr(s) => {
                        write!(out, "println!(\"{}\")", s)?;
                        return Ok(());
                    }
                    _ => {
                        // Single non-string argument: use format string
                        write!(out, "println!(\"{{}}\", ")?;
                        self.expr(expr, out)?;
                        write!(out, ")")?;
                        return Ok(());
                    }
                }
            }
        }

        // Multiple arguments: check if first is a string literal
        if let Arg::Pos(first_arg) = &call.args.args[0] {
            if let Expr::Str(s) | Expr::CStr(s) = first_arg {
                // First arg is a string - use it as format prefix
                let mut format_string = s.replace("\"", r##"\""##);
                let arg_count = call.args.args.len() - 1;

                // Add placeholders for remaining args
                for _ in 0..arg_count {
                    format_string.push_str(" {}");
                }

                write!(out, "println!(\"{}\"", format_string)?;

                // Add remaining args
                for arg in call.args.args.iter().skip(1) {
                    write!(out, ", ")?;
                    self.arg(arg, out)?;
                }
                write!(out, ")")?;
                return Ok(());
            }
        }

        // Fallback: generic format string with placeholders
        write!(out, "println!(\"")?;
        for (i, _arg) in call.args.args.iter().enumerate() {
            if i > 0 {
                write!(out, " ")?;
            }
            write!(out, "{{}}")?;
        }
        write!(out, "\"")?;
        for arg in &call.args.args {
            write!(out, ", ")?;
            self.arg(arg, out)?;
        }
        write!(out, ")").map_err(Into::into)
    }

    fn stmt(&mut self, stmt: &Stmt, sink: &mut Sink) -> AutoResult<bool> {
        match stmt {
            Stmt::Expr(expr) => {
                self.expr(expr, &mut sink.body)?;
                // No semicolon for expressions in expression position
                // (handled by body() method)
                Ok(true)
            }

            Stmt::Store(store) => {
                self.store(store, &mut sink.body)?;
                sink.body.write(b";")?;
                Ok(true)
            }

            Stmt::Fn(fn_decl) => {
                self.fn_decl(fn_decl, sink)?;
                Ok(true)
            }

            Stmt::For(for_stmt) => {
                self.for_stmt(for_stmt, sink)?;
                Ok(true)
            }

            Stmt::If(if_) => {
                self.if_stmt(if_, sink)?;
                Ok(true)
            }

            Stmt::Is(is_stmt) => {
                self.is_stmt(is_stmt, sink)?;
                Ok(true)
            }

            Stmt::Use(use_stmt) => {
                self.use_stmt(use_stmt, &mut sink.body)?;
                Ok(true)
            }

            Stmt::TypeDecl(type_decl) => {
                self.type_decl(type_decl, sink)?;
                Ok(true)
            }

            Stmt::SpecDecl(spec_decl) => {
                self.spec_decl(spec_decl, sink)?;
                Ok(true)
            }

            Stmt::EnumDecl(enum_decl) => {
                self.enum_decl(enum_decl, sink)?;
                Ok(true)
            }

            Stmt::EmptyLine(n) => {
                for _ in 0..*n {
                    sink.body.write(b"\n")?;
                }
                Ok(true)
            }

            Stmt::Break => {
                sink.body.write(b"break;")?;
                Ok(true)
            }

            Stmt::Node(node) => {
                // Handle loop and other control flow nodes
                self.expr(&Expr::Node(node.clone()), &mut sink.body)?;
                sink.body.write(b";")?;
                Ok(true)
            }

            _ => Err(format!("Rust Transpiler: unsupported statement: {:?}", stmt).into()),
        }
    }

    // Variable declaration
    fn store(&mut self, store: &Store, out: &mut impl Write) -> AutoResult<()> {
        // Handle C variables and struct fields (should not be generated)
        match store.kind {
            StoreKind::CVar | StoreKind::Field => {
                return Ok(());
            }
            _ => {}
        }

        // Type inference for Unknown types
        if matches!(store.ty, Type::Unknown) {
            match store.kind {
                StoreKind::Let => {
                    write!(out, "let {} = ", store.name)?;
                }
                StoreKind::Mut => {
                    write!(out, "let mut {} = ", store.name)?;
                }
                StoreKind::Var => {
                    // Var is for dynamic/script mode, treat as let
                    write!(out, "let {} = ", store.name)?;
                }
                _ => {
                    write!(out, "let {} = ", store.name)?;
                }
            }
        } else {
            // Explicit type annotation
            match store.kind {
                StoreKind::Let => {
                    write!(
                        out,
                        "let {}: {} = ",
                        store.name,
                        self.rust_type_name(&store.ty)
                    )?;
                }
                StoreKind::Mut => {
                    write!(
                        out,
                        "let mut {}: {} = ",
                        store.name,
                        self.rust_type_name(&store.ty)
                    )?;
                }
                StoreKind::Var => {
                    write!(
                        out,
                        "let {}: {} = ",
                        store.name,
                        self.rust_type_name(&store.ty)
                    )?;
                }
                _ => {
                    write!(
                        out,
                        "let {}: {} = ",
                        store.name,
                        self.rust_type_name(&store.ty)
                    )?;
                }
            }
        }

        self.expr(&store.expr, out)?;
        Ok(())
    }

    // Function declaration
    fn fn_decl(&mut self, fn_decl: &Fn, sink: &mut Sink) -> AutoResult<()> {
        // Skip C function declarations
        if matches!(fn_decl.kind, FnKind::CFunction | FnKind::VmFunction) {
            return Ok(());
        }

        // Check if this is a method (has parent)
        let is_method = fn_decl.parent.is_some();

        // Print indent for methods (inside impl block)
        if is_method {
            self.print_indent(&mut sink.body)?;
        }

        // Function signature
        write!(sink.body, "fn {}", fn_decl.name)?;

        // Parameters
        write!(sink.body, "(")?;

        // Add &self as first parameter for methods
        if is_method {
            write!(sink.body, "&self")?;
            if !fn_decl.params.is_empty() {
                write!(sink.body, ", ")?;
            }
        }

        for (i, param) in fn_decl.params.iter().enumerate() {
            write!(
                sink.body,
                "{}: {}",
                param.name,
                self.rust_type_name(&param.ty)
            )?;
            if i < fn_decl.params.len() - 1 {
                write!(sink.body, ", ")?;
            }
        }
        write!(sink.body, ")")?;

        // Return type
        if !matches!(fn_decl.ret, Type::Void) {
            write!(sink.body, " -> {}", self.rust_type_name(&fn_decl.ret))?;
        }

        // Function body
        write!(sink.body, " ")?;
        self.scope.borrow_mut().enter_fn(fn_decl.name.clone());
        self.body(&fn_decl.body, sink, &fn_decl.ret, "")?;
        self.scope.borrow_mut().exit_fn();

        Ok(())
    }

    // For loop
    fn for_stmt(&mut self, for_stmt: &For, sink: &mut Sink) -> AutoResult<()> {
        match &for_stmt.iter {
            Iter::Named(name) => {
                sink.body.write(b"for ")?;
                sink.body.write(name.as_bytes())?;
                sink.body.write(b" in ")?;

                // Check if it's a range or array iteration
                if let Expr::Range(range) = &for_stmt.range {
                    // Range iteration: for x in start..end
                    self.expr(&range.start, &mut sink.body)?;
                    sink.body.write(b"..")?;
                    self.expr(&range.end, &mut sink.body)?;
                    sink.body.write(b" {\n")?;

                    // Body
                    self.indent();
                    for stmt in &for_stmt.body.stmts {
                        self.print_indent(&mut sink.body)?;
                        match stmt {
                            Stmt::Expr(expr) => {
                                self.expr(expr, &mut sink.body)?;
                                sink.body.write(b";\n")?;
                            }
                            Stmt::Store(store) => {
                                self.store(store, &mut sink.body)?;
                                sink.body.write(b";\n")?;
                            }
                            _ => {}
                        }
                    }
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    sink.body.write(b"}")?;
                } else {
                    // Array iteration: for x in arr
                    self.expr(&for_stmt.range, &mut sink.body)?;
                    sink.body.write(b" {\n")?;

                    // Body
                    self.indent();
                    for stmt in &for_stmt.body.stmts {
                        self.print_indent(&mut sink.body)?;
                        match stmt {
                            Stmt::Expr(expr) => {
                                self.expr(expr, &mut sink.body)?;
                                sink.body.write(b";\n")?;
                            }
                            Stmt::Store(store) => {
                                self.store(store, &mut sink.body)?;
                                sink.body.write(b";\n")?;
                            }
                            _ => {}
                        }
                    }
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    sink.body.write(b"}")?;
                }
            }
            Iter::Ever => {
                // Infinite loop: loop { body }
                sink.body.write(b"loop {\n")?;
                self.indent();
                for stmt in &for_stmt.body.stmts {
                    self.print_indent(&mut sink.body)?;
                    match stmt {
                        Stmt::Expr(expr) => {
                            self.expr(expr, &mut sink.body)?;
                            sink.body.write(b";\n")?;
                        }
                        Stmt::Store(store) => {
                            self.store(store, &mut sink.body)?;
                            sink.body.write(b";\n")?;
                        }
                        _ => {}
                    }
                }
                self.dedent();
                self.print_indent(&mut sink.body)?;
                sink.body.write(b"}")?;
            }
            Iter::Cond => {
                // Conditional loop: while condition { ... }
                // Check if there's an init statement
                if let Some(init_stmt) = &for_stmt.init {
                    // Emit init statement before the loop
                    match &**init_stmt {
                        Stmt::Store(store) => {
                            self.store(store, &mut sink.body)?;
                            sink.body.write(b";\n")?;
                        }
                        _ => {
                            self.stmt(&**init_stmt, sink)?;
                            sink.body.write(b"\n")?;
                        }
                    }
                }

                sink.body.write(b"while ")?;
                self.expr(&for_stmt.range, &mut sink.body)?;
                sink.body.write(b" {\n")?;

                self.indent();
                for stmt in &for_stmt.body.stmts {
                    self.print_indent(&mut sink.body)?;
                    match stmt {
                        Stmt::Expr(expr) => {
                            self.expr(expr, &mut sink.body)?;
                            sink.body.write(b";\n")?;
                        }
                        Stmt::Store(store) => {
                            self.store(store, &mut sink.body)?;
                            sink.body.write(b";\n")?;
                        }
                        _ => {}
                    }
                }
                self.dedent();
                self.print_indent(&mut sink.body)?;
                sink.body.write(b"}")?;
            }
            _ => {}
        }
        Ok(())
    }

    // If statement
    fn if_stmt(&mut self, if_: &If, sink: &mut Sink) -> AutoResult<()> {
        for (i, branch) in if_.branches.iter().enumerate() {
            if i == 0 {
                sink.body.write(b"if ")?;
            } else {
                sink.body.write(b" else if ")?;
            }

            self.expr(&branch.cond, &mut sink.body)?;
            sink.body.write(b" ")?;

            // Process branch body - use body() method for proper formatting
            sink.body.write(b"{\n")?;
            self.indent();
            let stmt_count = branch.body.stmts.len();
            for (i, stmt) in branch.body.stmts.iter().enumerate() {
                self.print_indent(&mut sink.body)?;
                let is_last = i == stmt_count - 1;
                match stmt {
                    Stmt::Expr(Expr::If(inner_if)) => {
                        // Nested if expression - handle recursively
                        self.expr(&Expr::If(inner_if.clone()), &mut sink.body)?;
                        sink.body.write(b"\n")?;
                    }
                    Stmt::Expr(expr) => {
                        self.expr(expr, &mut sink.body)?;
                        // Check if this expression needs a semicolon
                        // Add semicolon if: not last OR it's a void function call (like print)
                        let needs_semicolon = if !is_last {
                            true
                        } else {
                            // Check if it's a call to print or other void functions
                            match expr {
                                Expr::Call(call) => {
                                    if let Expr::Ident(name) = call.name.as_ref() {
                                        name == "print" || name == "println"
                                    } else {
                                        false
                                    }
                                }
                                _ => false,
                            }
                        };

                        if needs_semicolon {
                            sink.body.write(b";\n")?;
                        } else {
                            sink.body.write(b"\n")?;
                        }
                    }
                    Stmt::If(inner_if) => {
                        // Nested if statement - handle recursively
                        self.if_stmt(inner_if, sink)?;
                    }
                    Stmt::Store(store) => {
                        self.store(store, &mut sink.body)?;
                        sink.body.write(b";\n")?;
                    }
                    _ => {}
                }
            }
            self.dedent();
            self.print_indent(&mut sink.body)?;
            sink.body.write(b"}")?;
        }

        if let Some(else_body) = &if_.else_ {
            sink.body.write(b" else ")?;
            sink.body.write(b"{\n")?;
            self.indent();
            let stmt_count = else_body.stmts.len();
            for (i, stmt) in else_body.stmts.iter().enumerate() {
                self.print_indent(&mut sink.body)?;
                let is_last = i == stmt_count - 1;
                match stmt {
                    Stmt::Expr(Expr::If(inner_if)) => {
                        // Nested if expression in else
                        self.expr(&Expr::If(inner_if.clone()), &mut sink.body)?;
                        sink.body.write(b"\n")?;
                    }
                    Stmt::Expr(expr) => {
                        self.expr(expr, &mut sink.body)?;
                        // Check if this expression needs a semicolon
                        let needs_semicolon = if !is_last {
                            true
                        } else {
                            // Check if it's a call to print or other void functions
                            match expr {
                                Expr::Call(call) => {
                                    if let Expr::Ident(name) = call.name.as_ref() {
                                        name == "print" || name == "println"
                                    } else {
                                        false
                                    }
                                }
                                _ => false,
                            }
                        };

                        if needs_semicolon {
                            sink.body.write(b";\n")?;
                        } else {
                            sink.body.write(b"\n")?;
                        }
                    }
                    Stmt::If(inner_if) => {
                        // Nested if statement in else
                        self.if_stmt(inner_if, sink)?;
                    }
                    Stmt::Store(store) => {
                        self.store(store, &mut sink.body)?;
                        sink.body.write(b";\n")?;
                    }
                    _ => {}
                }
            }
            self.dedent();
            self.print_indent(&mut sink.body)?;
            sink.body.write(b"}\n")?;
        }

        Ok(())
    }

    // Is statement (pattern matching)
    fn is_stmt(&mut self, is_stmt: &Is, sink: &mut Sink) -> AutoResult<()> {
        sink.body.write(b"match ")?;
        self.expr(&is_stmt.target, &mut sink.body)?;
        sink.body.write(b" {\n")?;
        self.indent();

        for branch in &is_stmt.branches {
            self.print_indent(&mut sink.body)?;

            match branch {
                IsBranch::EqBranch(expr, body) => {
                    self.expr(expr, &mut sink.body)?;
                    sink.body.write(b" => ")?;
                    // Simple body processing
                    if let Some(stmt) = body.stmts.first() {
                        match stmt {
                            Stmt::Expr(expr) => self.expr(expr, &mut sink.body)?,
                            _ => sink.body.write_all(b"/* TODO */")?,
                        }
                    }
                    sink.body.write(b",\n")?;
                }
                IsBranch::IfBranch(expr, body) => {
                    self.expr(expr, &mut sink.body)?;
                    sink.body.write(b" if true => ")?;
                    if let Some(stmt) = body.stmts.first() {
                        match stmt {
                            Stmt::Expr(expr) => self.expr(expr, &mut sink.body)?,
                            _ => sink.body.write_all(b"/* TODO */")?,
                        }
                    }
                    sink.body.write(b",\n")?;
                }
                IsBranch::ElseBranch(body) => {
                    sink.body.write(b"_ => ")?;
                    if let Some(stmt) = body.stmts.first() {
                        match stmt {
                            Stmt::Expr(expr) => self.expr(expr, &mut sink.body)?,
                            _ => sink.body.write_all(b"/* TODO */")?,
                        }
                    }
                    sink.body.write(b",\n")?;
                }
            }
        }

        self.dedent();
        self.print_indent(&mut sink.body)?;
        sink.body.write(b"}")?;
        Ok(())
    }

    // Use statement
    fn use_stmt(&mut self, use_stmt: &Use, out: &mut impl Write) -> AutoResult<()> {
        match use_stmt.kind {
            UseKind::Auto => {
                // Map Auto stdlib to Rust modules
                for path in &use_stmt.paths {
                    let rust_path = path.replace("auto.", "crate::");
                    write!(out, "use {};", rust_path)?;
                    self.uses.insert(path.clone());
                }
            }
            UseKind::C => {
                // Ignore C imports for Rust transpiler
            }
            UseKind::Rust => {
                // Direct Rust imports
                for path in &use_stmt.paths {
                    write!(out, "use {};", path)?;
                    self.uses.insert(path.clone());
                }
            }
        }
        Ok(())
    }

    // Type declaration (struct)
    fn type_decl(&mut self, type_decl: &TypeDecl, sink: &mut Sink) -> AutoResult<()> {
        // Generate traits for composed types
        for has_type in &type_decl.has {
            if let Type::User(has_decl) = has_type {
                // Check if this type is already defined (has members or methods)
                let is_trait_only = has_decl.members.is_empty() && has_decl.methods.is_empty();

                // Generate trait definition
                write!(sink.body, "trait {} {{\n", has_decl.name)?;
                self.indent();

                for method in &has_decl.methods {
                    // Generate method signature with &self
                    self.print_indent(&mut sink.body)?;
                    write!(sink.body, "fn {}(&self", method.name)?;

                    // Parameters (skip self which is already added)
                    for (i, param) in method.params.iter().enumerate() {
                        write!(sink.body, ", {}: {}", param.name, self.rust_type_name(&param.ty))?;
                        if i < method.params.len() - 1 {
                            write!(sink.body, ", ")?;
                        }
                    }

                    // Return type
                    if !matches!(method.ret, Type::Void) {
                        write!(sink.body, ") -> {}", self.rust_type_name(&method.ret))?;
                    } else {
                        write!(sink.body, ")")?;
                    }

                    write!(sink.body, ";\n")?;
                }

                self.dedent();
                write!(sink.body, "}}\n\n")?;

                // If this is a trait-only type (no struct definition), also generate a default impl
                if is_trait_only && !has_decl.methods.is_empty() {
                    write!(sink.body, "impl {} for {} {{\n", has_decl.name, has_decl.name)?;
                    self.indent();

                    for method in &has_decl.methods {
                        self.print_indent(&mut sink.body)?;
                        write!(sink.body, "fn {}(&self", method.name)?;

                        // Parameters
                        for (i, param) in method.params.iter().enumerate() {
                            write!(sink.body, ", {}: {}", param.name, self.rust_type_name(&param.ty))?;
                            if i < method.params.len() - 1 {
                                write!(sink.body, ", ")?;
                            }
                        }

                        // Return type
                        if !matches!(method.ret, Type::Void) {
                            write!(sink.body, ") -> {}", self.rust_type_name(&method.ret))?;
                        } else {
                            write!(sink.body, ")")?;
                        }

                        write!(sink.body, " {{\n")?;
                        self.indent();
                        self.print_indent(&mut sink.body)?;
                        write!(sink.body, "// Method implementation for {}\n", has_decl.name)?;
                        self.dedent();
                        self.print_indent(&mut sink.body)?;
                        write!(sink.body, "}}\n")?;
                    }

                    self.dedent();
                    write!(sink.body, "}}\n\n")?;
                }
            }
        }

        // Struct definition
        write!(sink.body, "struct {} {{", type_decl.name)?;

        // Collect all members (including from parent and composed types)
        // Use a set to avoid duplicates
        let mut all_members = Vec::new();
        let mut seen_fields = std::collections::HashSet::new();

        // First add members from parent type (inheritance)
        if let Some(ref parent_type) = type_decl.parent {
            if let Type::User(parent_decl) = parent_type.as_ref() {
                for member in &parent_decl.members {
                    if seen_fields.insert(member.name.clone()) {
                        all_members.push(member);
                    }
                }
            }
        }

        // Then add members from composed types
        for has_type in &type_decl.has {
            if let Type::User(has_decl) = has_type {
                for member in &has_decl.members {
                    if seen_fields.insert(member.name.clone()) {
                        all_members.push(member);
                    }
                }
            }
        }

        // Then add own members (can override inherited and composed ones)
        for member in &type_decl.members {
            if seen_fields.insert(member.name.clone()) {
                all_members.push(member);
            }
        }

        // Add delegation members to seen_fields and generate them separately
        for delegation in &type_decl.delegations {
            seen_fields.insert(delegation.member_name.clone());
        }

        if !all_members.is_empty() || !type_decl.delegations.is_empty() {
            sink.body.write(b"\n")?;
            self.indent();

            // First, write regular members
            for member in all_members {
                self.print_indent(&mut sink.body)?;
                write!(
                    sink.body,
                    "{}: {},",
                    member.name,
                    self.rust_type_name(&member.ty)
                )?;
                sink.body.write(b"\n")?;
            }

            // Then, write delegation members
            for delegation in &type_decl.delegations {
                self.print_indent(&mut sink.body)?;
                write!(
                    sink.body,
                    "{}: {},",
                    delegation.member_name,
                    self.rust_type_name(&delegation.member_type)
                )?;
                sink.body.write(b"\n")?;
            }

            self.dedent();
            self.print_indent(&mut sink.body)?;
        }

        sink.body.write(b"}\n")?;

        // Implement traits for composed types
        for has_type in &type_decl.has {
            if let Type::User(has_decl) = has_type {
                write!(sink.body, "\nimpl {} for {} {{\n", has_decl.name, type_decl.name)?;
                self.indent();

                for method in &has_decl.methods {
                    self.print_indent(&mut sink.body)?;
                    write!(sink.body, "fn {}(&self", method.name)?;

                    // Parameters
                    for (i, param) in method.params.iter().enumerate() {
                        write!(sink.body, ", {}: {}", param.name, self.rust_type_name(&param.ty))?;
                        if i < method.params.len() - 1 {
                            write!(sink.body, ", ")?;
                        }
                    }

                    // Return type
                    if !matches!(method.ret, Type::Void) {
                        write!(sink.body, ") -> {}", self.rust_type_name(&method.ret))?;
                    } else {
                        write!(sink.body, ")")?;
                    }

                    write!(sink.body, " {{\n")?;
                    self.indent();
                    self.print_indent(&mut sink.body)?;
                    write!(sink.body, "// TODO: Implement {} method body from {}\n", method.name, has_decl.name)?;
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    write!(sink.body, "}}\n")?;
                }

                self.dedent();
                write!(sink.body, "}}\n")?;
            }
        }

        // Generate trait implementations for delegations
        for delegation in &type_decl.delegations {
            let spec_name = delegation.spec_name.clone();
            let member_name = delegation.member_name.clone();

            // Get the spec declaration - clone to avoid borrow issues
            let spec_decl_clone = if let Some(meta) = self.scope.borrow().lookup_meta(spec_name.as_str()) {
                if let crate::scope::Meta::Spec(spec_decl) = meta.as_ref() {
                    Some(spec_decl.clone())
                } else {
                    None
                }
            } else {
                None
            };

            // Now use the cloned spec_decl
            if let Some(spec_decl) = spec_decl_clone {
                write!(sink.body, "\nimpl {} for {} {{\n", spec_decl.name, type_decl.name)?;
                self.indent();

                // Generate methods that delegate to the member
                for spec_method in &spec_decl.methods {
                    self.print_indent(&mut sink.body)?;
                    write!(sink.body, "fn {}(&self", spec_method.name)?;

                    // Parameters
                    for param in &spec_method.params {
                        write!(sink.body, ", {}: {}", param.name, self.rust_type_name(&param.ty))?;
                    }

                    // Return type
                    if !matches!(spec_method.ret, Type::Void) {
                        write!(sink.body, ") -> {}", self.rust_type_name(&spec_method.ret))?;
                    } else {
                        write!(sink.body, ")")?;
                    }

                    write!(sink.body, " {{\n")?;
                    self.indent();
                    self.print_indent(&mut sink.body)?;
                    write!(sink.body, "self.{}.{}(", member_name, spec_method.name)?;

                    // Forward parameters
                    for (i, param) in spec_method.params.iter().enumerate() {
                        if i > 0 {
                            write!(sink.body, ", ")?;
                        }
                        write!(sink.body, "{}", param.name)?;
                    }

                    write!(sink.body, ")\n")?;
                    self.dedent();
                    self.print_indent(&mut sink.body)?;
                    write!(sink.body, "}}\n")?;
                }

                self.dedent();
                write!(sink.body, "}}\n")?;
            }
        }

        // Generate impl block with own methods
        if !type_decl.methods.is_empty() {
            sink.body.write(b"\n")?;
            write!(sink.body, "impl {} {{", type_decl.name)?;
            sink.body.write(b"\n")?;
            self.indent();

            for method in &type_decl.methods {
                self.fn_decl(method, sink)?;
                sink.body.write(b"\n")?;
            }

            self.dedent();
            self.print_indent(&mut sink.body)?;
            sink.body.write(b"}\n")?;
        }

        // Generate trait implementations for specs
        if !type_decl.specs.is_empty() {
            // Collect spec declarations from scope
            let spec_decls: Vec<_> = type_decl.specs.iter().filter_map(|spec_name| {
                if let Some(meta) = self.scope.borrow().lookup_meta(spec_name.as_str()) {
                    if let crate::scope::Meta::Spec(spec_decl) = meta.as_ref() {
                        Some(spec_decl.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }).collect();

            // Generate impl block for each spec
            for spec_decl in spec_decls {
                sink.body.write(b"\n")?;
                writeln!(sink.body, "impl {} for {} {{", spec_decl.name, type_decl.name)?;
                self.indent();

                // Find methods in type_decl that match spec methods
                for spec_method in &spec_decl.methods {
                    // Find the implementation in type_decl
                    if let Some(method) = type_decl.methods.iter().find(|m| m.name == spec_method.name) {
                        self.print_indent(&mut sink.body)?;

                        // Method signature
                        write!(sink.body, "fn {}(&self", method.name)?;

                        // Parameters
                        for param in &method.params {
                            write!(sink.body, ", {}: {}", param.name, self.rust_type_name(&param.ty))?;
                        }

                        // Return type
                        if !matches!(method.ret, Type::Void) {
                            write!(sink.body, ") -> {}", self.rust_type_name(&method.ret))?;
                        } else {
                            write!(sink.body, ")")?;
                        }

                        writeln!(sink.body, " {{")?;
                        self.indent();

                        // Generate method body
                        self.body(&method.body, sink, &method.ret, "")?;

                        self.dedent();
                        self.print_indent(&mut sink.body)?;
                        writeln!(sink.body, "}}")?;
                    }
                }

                self.dedent();
                writeln!(sink.body, "}}")?;
            }
        }

        Ok(())
    }

    // Enum declaration
    fn enum_decl(&mut self, enum_decl: &EnumDecl, sink: &mut Sink) -> AutoResult<()> {
        // Generate enum definition
        sink.body.write(b"enum ")?;
        sink.body.write(enum_decl.name.as_bytes())?;
        sink.body.write(b" {\n")?;
        self.indent();

        for (_i, item) in enum_decl.items.iter().enumerate() {
            self.print_indent(&mut sink.body)?;
            sink.body
                .write(format!("{} = {},", item.name, item.value).as_bytes())?;
            sink.body.write(b"\n")?;
        }

        self.dedent();
        self.print_indent(&mut sink.body)?;
        sink.body.write(b"}\n")?;

        // Generate Display trait implementation
        sink.body.write(b"\n")?;
        writeln!(
            sink.body,
            "impl std::fmt::Display for {} {{",
            enum_decl.name
        )?;
        self.indent();
        self.print_indent(&mut sink.body)?;
        writeln!(
            sink.body,
            "fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {{"
        )?;
        self.indent();
        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "match self {{")?;
        self.indent();

        for item in &enum_decl.items {
            self.print_indent(&mut sink.body)?;
            writeln!(
                sink.body,
                "{}::{} => write!(f, \"{}\"),",
                enum_decl.name, item.name, item.name
            )?;
        }

        self.dedent();
        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "}}")?;
        self.dedent();
        self.print_indent(&mut sink.body)?;
        writeln!(sink.body, "}}")?;
        self.dedent();
        writeln!(sink.body, "}}")?;

        Ok(())
    }

    // Spec/trait declaration
    fn spec_decl(&mut self, spec_decl: &SpecDecl, sink: &mut Sink) -> AutoResult<()> {
        // Generate trait definition
        writeln!(sink.body, "trait {} {{", spec_decl.name)?;
        self.indent();

        for method in &spec_decl.methods {
            self.print_indent(&mut sink.body)?;
            write!(sink.body, "fn {}(&self", method.name)?;

            // Parameters (skip self which is already added as &self)
            for param in &method.params {
                write!(sink.body, ", {}: {}", param.name, self.rust_type_name(&param.ty))?;
            }

            // Return type
            if !matches!(method.ret, Type::Void) {
                write!(sink.body, ") -> {}", self.rust_type_name(&method.ret))?;
            } else {
                write!(sink.body, ")")?;
            }

            writeln!(sink.body, ";")?;
        }

        self.dedent();
        writeln!(sink.body, "}}\n")?;

        Ok(())
    }

    // Body and block management
    fn body(
        &mut self,
        body: &Body,
        sink: &mut Sink,
        ret_type: &Type,
        _insert: &str,
    ) -> AutoResult<()> {
        let has_return = !matches!(ret_type, Type::Void);

        sink.body.write(b"{\n")?;
        self.indent();

        // Process statements
        for (i, stmt) in body.stmts.iter().enumerate() {
            if !matches!(stmt, Stmt::EmptyLine(_)) {
                self.print_indent(&mut sink.body)?;
            }

            let is_last = i == body.stmts.len() - 1;

            if is_last && has_return && self.is_returnable(stmt) {
                // Last statement in a non-void function: expression position (no semicolon)
                match stmt {
                    Stmt::Expr(expr) => {
                        self.expr(expr, &mut sink.body)?;
                        sink.body.write(b"\n")?;
                    }
                    _ => {
                        self.stmt(stmt, sink)?;
                        sink.body.write(b"\n")?;
                    }
                }
            } else {
                // Regular statement: add semicolon if needed
                match stmt {
                    Stmt::Expr(expr) => {
                        self.expr(expr, &mut sink.body)?;
                        sink.body.write(b";\n")?;
                    }
                    Stmt::Store(store) => {
                        self.store(store, &mut sink.body)?;
                        sink.body.write(b";\n")?;
                    }
                    Stmt::EmptyLine(n) => {
                        for _ in 0..*n {
                            sink.body.write(b"\n")?;
                        }
                    }
                    Stmt::Break => {
                        sink.body.write(b"break;\n")?;
                    }
                    _ => {
                        // For other statement types that handle their own formatting
                        self.stmt(stmt, sink)?;
                        sink.body.write(b"\n")?;
                    }
                }
            }
        }

        self.dedent();
        self.print_indent(&mut sink.body)?;
        sink.body.write(b"}")?;
        Ok(())
    }

    fn is_returnable(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(expr) => match expr {
                Expr::Call(call) => {
                    // print() is not returnable (returns unit type)
                    if let Expr::Ident(name) = call.name.as_ref() {
                        if name == "print" {
                            return false;
                        }
                    }
                    true
                }
                Expr::If(_) => true,
                Expr::Block(_) => true,
                Expr::Bina(_, _, _) => true,
                Expr::Ident(_) => true,
                Expr::Array(_) => true,
                Expr::Index(_, _) => true,
                _ => false,
            },
            _ => false,
        }
    }
}

impl Trans for RustTrans {
    fn trans(&mut self, ast: Code, sink: &mut Sink) -> AutoResult<()> {
        // Phase 1: Emit file header
        // sink.body.write(b"//! Auto-generated Rust code\n\n")?;

        // Phase 2: Split into declarations and main
        let mut decls: Vec<Stmt> = Vec::new();
        let mut main: Vec<Stmt> = Vec::new();

        for stmt in ast.stmts.into_iter() {
            if stmt.is_decl() {
                decls.push(stmt);
            } else {
                match stmt {
                    Stmt::For(_) => main.push(stmt),
                    Stmt::If(_) => main.push(stmt),
                    Stmt::Expr(_) => main.push(stmt),
                    Stmt::Store(_) => main.push(stmt),
                    Stmt::Break => main.push(stmt),
                    Stmt::Use(use_stmt) => {
                        self.use_stmt(&use_stmt, &mut sink.body)?;
                        sink.body.write(b"\n")?;
                    }
                    _ => {}
                }
            }
        }

        // Phase 3: Generate declarations
        for (i, decl) in decls.iter().enumerate() {
            self.stmt(decl, sink)?;
            if i < decls.len() - 1 {
                // Add blank line between declarations
                // Check if we already end with a newline
                if sink.body.ends_with(b"\n") {
                    sink.body.write(b"\n")?;
                } else {
                    sink.body.write(b"\n\n")?;
                }
            }
        }

        // Phase 4: Generate main function if needed
        if !main.is_empty() {
            if !decls.is_empty() {
                // Add blank line before main
                if sink.body.ends_with(b"\n") {
                    sink.body.write(b"\n")?;
                } else {
                    sink.body.write(b"\n\n")?;
                }
            }

            // Check if main should return a value
            let has_return = main.iter().any(|s| self.is_returnable(s));

            sink.body.write(b"fn main()")?;
            if has_return {
                sink.body.write(b" -> i32")?;
            }
            sink.body.write(b" {\n")?;
            self.indent();

            for (i, stmt) in main.iter().enumerate() {
                self.print_indent(&mut sink.body)?;

                let is_last = i == main.len() - 1;
                if is_last && has_return && self.is_returnable(stmt) {
                    // Last expression: no semicolon (expression position)
                    match stmt {
                        Stmt::Expr(expr) => {
                            self.expr(expr, &mut sink.body)?;
                            sink.body.write(b"\n")?;
                        }
                        _ => {
                            self.stmt(stmt, sink)?;
                            sink.body.write(b"\n")?;
                        }
                    }
                } else {
                    self.stmt(stmt, sink)?;
                    // Only add semicolon for simple statements (expr, store)
                    // Compound statements (if, for, etc.) handle their own formatting
                    match stmt {
                        Stmt::Expr(_) | Stmt::Store(_) => {
                            sink.body.write(b";\n")?;
                        }
                        _ => {
                            // Compound statements already have proper formatting
                        }
                    }
                }
            }

            self.dedent();
            sink.body.write(b"}\n")?;
        }

        // Add final newline only if not already ending with one
        if !sink.body.is_empty() && !sink.body.ends_with(b"\n") {
            sink.body.write(b"\n")?;
        }

        Ok(())
    }
}

/// Transpile AutoLang code to Rust
pub fn transpile_rust(
    name: impl Into<AutoStr>,
    code: &str,
) -> AutoResult<(Sink, Shared<Universe>)> {
    let name = name.into();
    let scope = shared(crate::universe::Universe::default());
    let mut parser = Parser::new(code, scope);
    parser.set_dest(crate::parser::CompileDest::TransRust);
    let ast = parser.parse().map_err(|e| e.to_string())?;

    let mut out = Sink::new(name.clone());
    let mut transpiler = RustTrans::new(name);
    transpiler.scope = parser.scope.clone();
    transpiler.trans(ast, &mut out)?;

    Ok((out, parser.scope.clone()))
}

/// Transpile code fragment for testing
pub fn transpile_part(code: &str) -> AutoResult<AutoStr> {
    let scope = shared(crate::universe::Universe::default());
    let mut parser = Parser::new(code, scope);
    let ast = parser.parse().map_err(|e| e.to_string())?;
    let mut out = Sink::new(AutoStr::from(""));
    let mut transpiler = RustTrans::new("part".into());
    transpiler.trans(ast, &mut out)?;
    let src = out.done()?.clone();
    Ok(String::from_utf8(src).unwrap().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read_to_string;
    use std::path::PathBuf;

    fn test_a2r(case: &str) -> AutoResult<()> {
        // Parse test case name: "000_hello" -> "hello"
        let parts: Vec<&str> = case.split("_").collect();
        let name = parts[1..].join("_");

        let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let src_path = format!("test/a2r/{}/{}.at", case, name);
        let src_path = d.join(src_path);
        let src = read_to_string(src_path.as_path())?;

        let exp_path = format!("test/a2r/{}/{}.expected.rs", case, name);
        let exp_path = d.join(exp_path);
        let expected = if !exp_path.is_file() {
            "".to_string()
        } else {
            read_to_string(exp_path.as_path())?
        };

        let (mut rcode, _) = transpile_rust(&name, &src)?;
        let rs_code = rcode.done()?;

        if rs_code != expected.as_bytes() {
            // Generate .wrong.rs for comparison
            let gen_path = format!("test/a2r/{}/{}.wrong.rs", case, name);
            let gen_path = d.join(gen_path);
            std::fs::write(&gen_path, rs_code)?;
        }

        assert_eq!(String::from_utf8_lossy(rs_code), expected);
        Ok(())
    }

    #[test]
    fn test_000_hello() {
        test_a2r("000_hello").unwrap();
    }

    #[test]
    fn test_001_sqrt() {
        test_a2r("001_sqrt").unwrap();
    }

    #[test]
    fn test_002_array() {
        test_a2r("002_array").unwrap();
    }

    #[test]
    fn test_003_func() {
        test_a2r("003_func").unwrap();
    }

    #[test]
    fn test_006_struct() {
        test_a2r("006_struct").unwrap();
    }

    #[test]
    fn test_007_enum() {
        test_a2r("007_enum").unwrap();
    }

    #[test]
    fn test_008_method() {
        test_a2r("008_method").unwrap();
    }

    #[test]
    fn test_010_if() {
        test_a2r("010_if").unwrap();
    }

    #[test]
    fn test_011_for() {
        test_a2r("011_for").unwrap();
    }

    #[test]
    fn test_012_is() {
        test_a2r("012_is").unwrap();
    }

    #[test]
    fn test_013_while() {
        test_a2r("013_while").unwrap();
    }

    #[test]
    fn test_014_closure() {
        test_a2r("014_closure").unwrap();
    }

    #[test]
    fn test_015_nested_if() {
        test_a2r("015_nested_if").unwrap();
    }

    #[test]
    fn test_016_complex() {
        test_a2r("016_complex").unwrap();
    }

    #[test]
    fn test_017_struct_methods() {
        test_a2r("017_struct_methods").unwrap();
    }

    #[test]
    fn test_018_enum_pattern() {
        test_a2r("018_enum_pattern").unwrap();
    }

    #[test]
    fn test_019_blocks() {
        test_a2r("019_blocks").unwrap();
    }

    #[test]
    fn test_020_comprehensive() {
        test_a2r("020_comprehensive").unwrap();
    }

    #[test]
    fn test_021_indexing() {
        test_a2r("021_indexing").unwrap();
    }

    #[test]
    fn test_022_unary() {
        test_a2r("022_unary").unwrap();
    }

    #[test]
    fn test_023_arithmetic() {
        test_a2r("023_arithmetic").unwrap();
    }

    #[test]
    fn test_024_fstring() {
        test_a2r("024_fstring").unwrap();
    }

    #[test]
    fn test_025_fstring_edge() {
        test_a2r("025_fstring_edge").unwrap();
    }

    #[test]
    fn test_026_ref_expr() {
        test_a2r("026_ref_expr").unwrap();
    }

    #[test]
    fn test_027_range_expr() {
        test_a2r("027_range_expr").unwrap();
    }

    #[test]
    fn test_028_object() {
        test_a2r("028_object").unwrap();
    }

    #[test]
    fn test_029_composition() {
        test_a2r("029_composition").unwrap();
    }

    #[test]
    fn test_030_field_composition() {
        test_a2r("030_field_composition").unwrap();
    }

    #[test]
    fn test_031_spec() {
        test_a2r("031_spec").unwrap();
    }

    #[test]
    fn test_032_delegation() {
        test_a2r("032_delegation").unwrap();
    }

    #[test]
    fn test_033_multi_delegation() {
        test_a2r("033_multi_delegation").unwrap();
    }

    #[test]
    fn test_034_delegation_params() {
        test_a2r("034_delegation_params").unwrap();
    }

    #[test]
    fn test_035_inheritance() {
        test_a2r("035_inheritance").unwrap();
    }
}
