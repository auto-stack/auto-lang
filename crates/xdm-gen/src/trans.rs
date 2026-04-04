use crate::var::VarKind;
use auto_lang::ast::{Arg, Code, Expr, Key, Name, Node, Stmt};
use auto_lang::trans::Sink;
use auto_lang::types::TypeStore;
use auto_val::{AutoResult, AutoStr};
use chrono;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

// ============================================================
// AST Helper Functions (free functions)
// Replace Evaler.eval_node() + auto_val::Node property access
// ============================================================

/// Extract a named property from AST Node's args (Arg::Pair) or body (Stmt::Store).
fn get_node_prop(node: &Node, prop_name: &str) -> Option<Expr> {
    // First check args for named pair: foo: bar
    for arg in &node.args.args {
        if let Arg::Pair(name, expr) = arg {
            if name.as_str() == prop_name {
                return Some(expr.clone());
            }
        }
    }
    // Then check body for store stmts: kind = bool
    for stmt in &node.body.stmts {
        match stmt {
            Stmt::Store(store) => {
                if store.name.as_str() == prop_name {
                    return Some(store.expr.clone());
                }
            }
            Stmt::Expr(Expr::Pair(pair)) => {
                // Also match Pair expressions: key: value
                if let Key::NamedKey(name) = &pair.key {
                    if name.as_str() == prop_name {
                        return Some(*pair.value.clone());
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// Get positional arg by index.
fn get_pos_arg(node: &Node, index: usize) -> Option<Expr> {
    let mut pos_idx = 0;
    for arg in &node.args.args {
        if let Arg::Pos(expr) = arg {
            if pos_idx == index {
                return Some(expr.clone());
            }
            pos_idx += 1;
        }
    }
    None
}

/// Convert simple Expr to string.
fn expr_to_str(expr: &Expr) -> Option<AutoStr> {
    match expr {
        Expr::Bool(b) => Some(b.to_string().into()),
        Expr::Int(i) => Some(i.to_string().into()),
        Expr::Uint(u) => Some(u.to_string().into()),
        Expr::Float(f, _) => Some(f.to_string().into()),
        Expr::Str(s) => Some(s.clone()),
        Expr::Ident(name) => Some(name.clone()),
        _ => None,
    }
}

// ============================================================
// XdmTrans struct
// ============================================================

pub struct XdmTrans {
    type_store: Arc<RwLock<TypeStore>>,
    sink: Sink,
    indent: usize,
}

impl XdmTrans {
    pub fn new(type_store: Arc<RwLock<TypeStore>>, sink: Sink) -> Self {
        Self {
            type_store,
            sink,
            indent: 0,
        }
    }

    pub fn trans(&mut self, code: Code) -> AutoResult<()> {
        println!("Start tranlating to XDM");
        self.start()?;
        for stmt in code.stmts {
            self.stmt(&stmt)?;
        }
        Ok(())
    }

    pub fn start(&mut self) -> AutoResult<()> {
        self.println(
            r#"<?xml version="1.0"?>
<datamodel version="3.0" xmlns="http://www.tresos.de/_projects/DataModel2/08/root.xsd"
                         xmlns:a="http://www.tresos.de/_projects/DataModel2/08/attribute.xsd"
                         xmlns:v="http://www.tresos.de/_projects/DataModel2/06/schema.xsd"
                         xmlns:d="http://www.tresos.de/_projects/DataModel2/06/data.xsd">
<!--
*   @file    CanTrcv.xdm
*   @version 1.0.0
*
*   @brief   AUTOSAR CanTrcv - Tresos Studio plugin schema file
*   @details This file contains the schema configuration for and CanTrcv Tresos Studio plugin.
-->
<d:ctr type="AUTOSAR" factory="autosar"
         xmlns:ad="http://www.tresos.de/_projects/DataModel2/08/admindata.xsd"
         xmlns:icc="http://www.tresos.de/_projects/DataModel2/08/implconfigclass.xsd"
         xmlns:mt="http://www.tresos.de/_projects/DataModel2/11/multitest.xsd" >
    <d:lst type="TOP-LEVEL-PACKAGES">"#,
        )?;
        self.indent();
        self.indent();
        Ok(())
    }

    pub fn finish(mut self) -> AutoResult<Sink> {
        self.dedent();
        self.dedent();
        self.println(
            r#"
        </d:lst>
    </d:ctr>
</datamodel>"#,
        )?;
        Ok(self.sink)
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        self.indent -= 1;
    }

    fn dent(&mut self) -> AutoResult<()> {
        for _ in 0..self.indent {
            self.sink.print(b"    ")?;
        }
        Ok(())
    }

    fn stmt(&mut self, stmt: &Stmt) -> AutoResult<()> {
        match stmt {
            Stmt::Node(n) => self.node(n)?,
            Stmt::Expr(e) => self.expr(e)?,
            _ => {}
        }
        Ok(())
    }

    fn expr(&mut self, expr: &Expr) -> AutoResult<()> {
        match expr {
            Expr::Ident(ident) => {
                self.use_var(ident)?;
            }
            Expr::Node(nd) => {
                self.node(nd)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn use_var(&mut self, _ident: &Name) -> AutoResult<()> {
        // TODO: Add HashMap<Name, Node> if top-level var resolution is needed
        Ok(())
    }

    fn appendln(&mut self, text: impl Into<AutoStr>) -> AutoResult<()> {
        self.sink.println(text.into().as_bytes())?;
        Ok(())
    }

    fn append(&mut self, text: impl Into<AutoStr>) -> AutoResult<()> {
        self.sink.print(text.into().as_bytes())?;
        Ok(())
    }

    fn print(&mut self, text: impl Into<AutoStr>) -> AutoResult<()> {
        self.dent()?;
        self.sink.print(text.into().as_bytes())?;
        Ok(())
    }

    fn println(&mut self, text: impl Into<AutoStr>) -> AutoResult<()> {
        self.dent()?;
        self.sink.println(text.into().as_bytes())?;
        Ok(())
    }

    fn newline(&mut self) -> AutoResult<()> {
        self.sink.println(b"\n")?;
        Ok(())
    }

    fn xml_tag_name(&self, name: &str) -> &str {
        let tname = match name {
            "module" => "d:ctr",
            "moddef" => "d:chc",
            "v" => "v:var",
            "ctr" => "v:ctr",
            "chc" => "v:chc",
            "lst" => "v:lst",
            "ref" => "v:ref",
            _ => "v:UNKNOWN",
        };
        tname
    }

    fn node(&mut self, node: &Node) -> AutoResult<()> {
        let name = &node.name;
        match name.as_str() {
            "module" => {
                self.module(node)?;
            }
            "moddef" => {
                self.moddef(node)?;
            }
            "v" => {
                self.variable(node)?;
            }
            "ctr" => {
                self.container(node)?;
            }
            "chc" => {
                self.choice(node)?;
            }
            "ref" => {
                self.reference(node)?;
            }
            "lst" => {
                self.list(node)?;
            }
            _ => {
                return Err(format!("Unknown node! {}", name).into());
            }
        }
        Ok(())
    }

    fn tag_header(&mut self, node: &Node) -> AutoResult<()> {
        self.print(format!("<{}", self.xml_tag_name(&node.name)))?;
        self.append(format!(" name=\"{}\"", node.id))?;
        Ok(())
    }

    fn module(&mut self, node: &Node) -> AutoResult<()> {
        self.tag_header(node)?;
        self.append(" type=\"AR-PACKAGE\"")?;
        self.append(">\n")?;

        self.indent();
        self.uuid(node)?;

        self.println("<d:lst type=\"ELEMENTS\">")?;
        self.indent();

        for kid in &node.body.stmts {
            self.stmt(kid)?;
        }
        self.dedent();
        self.println("</d:lst>")?;
        self.dedent();

        self.println(format!("</{}>", self.xml_tag_name(&node.name)))?;
        Ok(())
    }

    fn admin_data(&mut self) -> AutoResult<()> {
        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let data = format!(
            r#"<a:a name="ADMIN-DATA" type="ADMIN-DATA">
    <ad:ADMIN-DATA>
        <ad:DOC-REVISIONS>
            <ad:DOC-REVISION>
                <ad:REVISION-LABEL>4.4.0</ad:REVISION-LABEL>
                <ad:ISSUED-BY>AUTOSAR</ad:ISSUED-BY>
                <ad:DATE>{}</ad:DATE>
            </ad:DOC-REVISION>
        </ad:DOC-REVISIONS>
    </ad:ADMIN-DATA>
</a:a>"#,
            date
        );
        // add indents to each line's head
        let cur_indent = "    ".repeat(self.indent);
        let indented_data = data
            .lines()
            .map(|line| format!("{}{}", cur_indent, line))
            .collect::<Vec<String>>()
            .join("\n");
        self.appendln(indented_data)?;
        Ok(())
    }

    fn moddef(&mut self, node: &Node) -> AutoResult<()> {
        self.tag_header(node)?;
        self.appendln(r#" type="AR-ELEMENT" value="MODULE-DEF">"#)?;
        self.indent();

        self.println(r#"<v:ctr type="MODULE-DEF">"#)?;
        self.indent();
        self.admin_data()?;

        self.println(r#"<a:a name="LOWER-MULTIPLICITY" value="0"/>"#)?;
        self.println(r#"<a:a name="RELEASE" value="asc:4.4"/>"#)?;
        self.println(r#"<a:a name="UPPER-MULTIPLICITY" value="*"/>"#)?;
        self.println(r#"<a:a name="POSTBUILDVARIANTSUPPORT" value="true"/>"#)?;

        self.uuid(node)?;

        self.newline()?;

        for kid in &node.body.stmts {
            self.stmt(kid)?;
        }
        self.dedent();
        self.println(r#"</v:ctr>"#)?;
        self.dedent();
        self.println(format!("</{}>", self.xml_tag_name(&node.name)))?;
        Ok(())
    }

    fn print_options(&mut self, exprs: &[Expr]) -> AutoResult<()> {
        self.println("<a:da name=\"RANGE\">")?;

        self.indent();
        for opt in exprs {
            let val = expr_to_str(opt).unwrap_or_default();
            self.println(format!("<a:v>{}</a:v>", val))?;
        }
        self.dedent();
        self.println("</a:da>")
    }

    fn default_text(&self, kind: &VarKind) -> AutoStr {
        match kind {
            VarKind::Int => "0",
            VarKind::Bool => "false",
            VarKind::Float => "0.0",
            VarKind::Str => "",
            VarKind::Select => "[]",
            VarKind::Unknown(_) => "unknown",
        }
        .into()
    }

    // ============================================================
    // Attribute methods - now take &Node (AST) instead of &auto_val::Node
    // ============================================================

    fn attr_editable(&mut self, node: &Node) -> AutoResult<()> {
        if let Some(expr) = get_node_prop(node, "editable") {
            match &expr {
                Expr::Bool(b) => {
                    self.println(format!("<a:da name=\"EDITABLE\" value=\"{}\"/>", b))?;
                }
                Expr::Str(s) => {
                    self.println(format!(
                        "<a:da name=\"EDITABLE\" type=\"XPath\" value=\"{}\"/>",
                        s
                    ))?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn attr_readonly(&mut self, node: &Node) -> AutoResult<()> {
        if let Some(expr) = get_node_prop(node, "readonly") {
            match &expr {
                Expr::Bool(b) => {
                    self.println(format!("<a:da name=\"READONLY\" value=\"{}\"/>", b))?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn attr_enable(&mut self, node: &Node) -> AutoResult<()> {
        if let Some(expr) = get_node_prop(node, "enable") {
            match &expr {
                Expr::Bool(b) => {
                    self.println(format!("<a:da name=\"ENABLE\" value=\"{}\"/>", b))?;
                }
                Expr::Str(s) => {
                    self.println(format!(
                        "<a:da name=\"ENABLE\" type=\"XPath\" value=\"{}\"/>",
                        s
                    ))?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn attr_optional(&mut self, node: &Node) -> AutoResult<()> {
        if let Some(expr) = get_node_prop(node, "optional") {
            let b = match &expr {
                Expr::Bool(b) => *b,
                _ => false,
            };
            self.println(format!("<a:a name=\"OPTIONAL\" value=\"{}\"/>", b))?;
        }
        Ok(())
    }

    fn attr_ref(&mut self, node: &Node) -> AutoResult<()> {
        if let Some(expr) = get_node_prop(node, "ref") {
            let ref_str = expr_to_str(&expr).unwrap_or_default();
            self.println("<a:da name=\"REF\">")?;
            self.indent();
            self.println(format!("<a:v>{}</a:v>", ref_str))?;
            self.dedent();
            self.println("</a:da>")?;
        }
        Ok(())
    }

    fn attr_origin(&mut self, node: &Node) -> AutoResult<()> {
        let origin = if let Some(expr) = get_node_prop(node, "origin") {
            expr_to_str(&expr).unwrap_or_default()
        } else {
            // Try TypeStore for type default value
            let type_store = self.type_store.read().unwrap();
            if let Some(decl) = type_store.lookup_type_decl_str(&node.name) {
                decl.members
                    .iter()
                    .find(|m| m.name.as_str() == "origin")
                    .and_then(|m| m.value.as_ref())
                    .and_then(|v| expr_to_str(v))
                    .unwrap_or_else(|| "AUTOSAR_ECUC".into())
            } else {
                AutoStr::from("AUTOSAR_ECUC")
            }
        };
        if !origin.is_empty() {
            self.println(format!("<a:a name=\"ORIGIN\" value=\"{}\" />", origin))?;
        }
        Ok(())
    }

    fn attr_range(&mut self, node: &Node) -> AutoResult<()> {
        if let Some(expr) = get_node_prop(node, "range") {
            match &expr {
                Expr::Int(v) => {
                    self.println("<a:da name=\"INVALID\" type=\"Range\">")?;
                    self.indent();
                    self.println(format!("<a:tst expr=\"&gt;={}\"/>", v))?;
                    self.println(format!("<a:tst expr=\"&lt;={}\"/>", v))?;
                    self.dedent();
                    self.println("</a:da>")?;
                }
                Expr::Array(exprs) if exprs.len() == 2 => {
                    let min = expr_to_str(&exprs[0]).unwrap_or_default();
                    let max = expr_to_str(&exprs[1]).unwrap_or_default();
                    self.println("<a:da name=\"INVALID\" type=\"Range\">")?;
                    self.indent();
                    self.println(format!("<a:tst expr=\"&gt;={}\"/>", min))?;
                    self.println(format!("<a:tst expr=\"&lt;={}\"/>", max))?;
                    self.dedent();
                    self.println("</a:da>")?;
                }
                Expr::Range(range) => {
                    let start_str = expr_to_str(&range.start).unwrap_or_default();
                    let end_str = expr_to_str(&range.end).unwrap_or_default();
                    self.println("<a:da name=\"INVALID\" type=\"Range\">")?;
                    self.indent();
                    self.println(format!("<a:tst expr=\"&gt;={}\"/>", start_str))?;
                    if range.eq {
                        self.println(format!("<a:tst expr=\"&lt;={}\"/>", end_str))?;
                    } else {
                        self.println(format!("<a:tst expr=\"&lt;{}\"/>", end_str))?;
                    }
                    self.dedent();
                    self.println("</a:da>")?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn variable(&mut self, node: &Node) -> AutoResult<()> {
        self.tag_header(node)?;

        // Determine kind: try named prop first, then positional arg
        let kind_expr = get_node_prop(node, "kind").or_else(|| get_pos_arg(node, 0));
        let kind_str = kind_expr
            .as_ref()
            .and_then(|e| expr_to_str(e))
            .unwrap_or_default();
        let kind = match kind_str.as_str() {
            "bool" => VarKind::Bool,
            "int" => VarKind::Int,
            "float" => VarKind::Float,
            "str" => VarKind::Str,
            "select" => VarKind::Select,
            _ => VarKind::Unknown(kind_str.clone()),
        };
        // print kind
        let kind_text = match &kind {
            VarKind::Int => "INTEGER",
            VarKind::Bool => "BOOLEAN",
            VarKind::Float => "FLOAT",
            VarKind::Str => "STRING",
            VarKind::Select => "ENUMERATION",
            VarKind::Unknown(_) => {
                return Err(format!("Unknown variable kind {:?} for {}", kind, node.id).into());
            }
        };
        self.append(format!(" type=\"{}\"", kind_text))?;
        self.append(">\n")?;

        self.indent();
        self.uuid(node)?;

        // attr origin
        self.attr_origin(node)?;

        // attr editable
        self.attr_editable(node)?;
        // attr readonly
        self.attr_readonly(node)?;

        // attr enable
        self.attr_enable(node)?;
        // attr optional
        self.attr_optional(node)?;
        // attr ref
        self.attr_ref(node)?;

        // attr invalid
        self.attr_invalid(node)?;

        // default value
        let default_text = if let Some(expr) = get_node_prop(node, "default") {
            expr_to_str(&expr).unwrap_or_default()
        } else {
            self.default_text(&kind)
        };
        if !default_text.is_empty() {
            self.println(format!(
                "<a:da name=\"DEFAULT\" value=\"{}\" />",
                default_text
            ))?;
        }

        // range
        self.attr_range(node)?;
        // options for select
        match kind {
            VarKind::Select => {
                // check options
                if let Some(expr) = get_node_prop(node, "options") {
                    match &expr {
                        Expr::Array(exprs) => {
                            self.print_options(exprs)?;
                        }
                        _ => {
                            return Err(
                                format!("Missing options for select variable {}", node.id).into()
                            );
                        }
                    }
                } else {
                    return Err(format!("Missing options for select variable {}", node.id).into());
                }
            }
            _ => {}
        }
        for kid in &node.body.stmts {
            if let Stmt::Node(_) = kid {
                self.stmt(kid)?;
            }
        }
        self.dedent();
        self.println(format!("</{}>", self.xml_tag_name(&node.name)))?;
        Ok(())
    }

    fn container(&mut self, node: &Node) -> AutoResult<()> {
        self.tag_header(node)?;
        self.append(" type=\"IDENTIFIABLE\"")?;
        self.append(">\n")?;
        self.indent();
        self.uuid(node)?;

        // attr enable
        self.attr_enable(node)?;
        // attr optional
        self.attr_optional(node)?;

        for kid in &node.body.stmts {
            self.stmt(kid)?;
        }
        self.dedent();
        self.println(format!("</{}>", self.xml_tag_name(&node.name)))?;
        Ok(())
    }

    fn choice(&mut self, node: &Node) -> AutoResult<()> {
        self.tag_header(node)?;
        self.append(">\n")?;
        self.indent();
        self.uuid(node)?;
        for kid in &node.body.stmts {
            self.stmt(kid)?;
        }
        self.dedent();
        self.println(format!("</{}>", self.xml_tag_name(&node.name)))?;
        Ok(())
    }

    fn list(&mut self, node: &Node) -> AutoResult<()> {
        self.tag_header(node)?;
        self.append(" type=\"MAP\">\n")?;
        self.indent();
        self.uuid(node)?;

        self.attr_invalid(node)?;
        self.attr_editable(node)?;
        for kid in &node.body.stmts {
            self.stmt(kid)?;
        }
        self.dedent();
        self.println(format!("</{}>", self.xml_tag_name(&node.name)))?;
        Ok(())
    }

    fn reference(&mut self, node: &Node) -> AutoResult<()> {
        self.tag_header(node)?;
        self.append(" type=\"REFERENCE\"")?;
        self.append(">\n")?;
        self.indent();
        self.uuid(node)?;

        // origin
        self.attr_origin(node)?;

        // enable
        self.attr_enable(node)?;
        // optional
        self.attr_optional(node)?;
        // ref
        self.attr_ref(node)?;
        // invalid
        self.attr_invalid(node)?;

        for kid in &node.body.stmts {
            self.stmt(kid)?;
        }
        self.dedent();
        self.println(format!("</{}>", self.xml_tag_name(&node.name)))?;
        Ok(())
    }

    fn uuid(&mut self, _node: &Node) -> AutoResult<()> {
        let uuid = Uuid::new_v4();
        self.println(format!("<a:a name=\"UUID\" value=\"ECUC:{}\"/>", uuid))?;
        Ok(())
    }

    fn escape_xml(&self, text: &AutoStr) -> AutoStr {
        text.replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("\"", "&quot;")
            .replace("'", "&apos;")
    }

    fn attr_invalid(&mut self, node: &Node) -> AutoResult<()> {
        if let Some(expr) = get_node_prop(node, "invalid") {
            match &expr {
                Expr::Array(items) => {
                    self.println("<a:da name=\"INVALID\" type=\"XPath\">")?;
                    self.indent();
                    for item in items {
                        let (state, obj_pairs) = match item {
                            Expr::Pair(pair) => {
                                let key_str = match &pair.key {
                                    auto_lang::ast::Key::NamedKey(name) => name.as_str(),
                                    auto_lang::ast::Key::StrKey(s) => s.as_str(),
                                    _ => "",
                                };
                                match key_str {
                                    "test_true" => (true, &*pair.value),
                                    "test_false" => (false, &*pair.value),
                                    _ => {
                                        return Err(
                                            "'invalid' value should be an array with 'test_true' or 'test_false' pairs".into()
                                        );
                                    }
                                }
                            }
                            _ => {
                                return Err(
                                    "'invalid' value should be an array with 'test_true' or 'test_false' pairs".into()
                                );
                            }
                        };
                        // Extract expr and msg from the object
                        let (expr_val, msg_val) = match obj_pairs {
                            Expr::Object(pairs) => {
                                let mut ev = AutoStr::new();
                                let mut mv = AutoStr::new();
                                for p in pairs {
                                    let k = match &p.key {
                                        auto_lang::ast::Key::NamedKey(n) => n.as_str(),
                                        _ => "",
                                    };
                                    if k == "expr" {
                                        ev = expr_to_str(&p.value).unwrap_or_default();
                                    } else if k == "msg" {
                                        mv = expr_to_str(&p.value).unwrap_or_default();
                                    }
                                }
                                (ev, mv)
                            }
                            _ => {
                                return Err(
                                    "invalid pair value should be an object with expr and msg"
                                        .into(),
                                );
                            }
                        };
                        let expr_escaped = self.escape_xml(&expr_val);
                        self.println(format!(
                            "<a:tst expr=\"{}\" {}=\"{}\"/>",
                            expr_escaped, state, msg_val
                        ))?;
                    }
                    self.dedent();
                    self.println("</a:da>")?;
                }
                _ => {
                    return Err(
                        "'invalid' value should be an array with 'test_true' or 'test_false' pairs"
                            .into(),
                    );
                }
            }
        }
        Ok(())
    }
}
