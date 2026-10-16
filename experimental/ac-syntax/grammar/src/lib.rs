//! ac-syntax-grammar：Atom 语法配置 → `ac_syntax_rt::GrammarProgram` 编译器。
//!
//! 配置格式（grammar-contract v1，静态 Atom 文档）：
//! ```atom
//! grammar(name: "demo", revision: 1, newline: false, stmt-end: [";"], sync: [";"]) {
//!     tokens {
//!         ws(trivia: true) { charset: " \t\r\n" }
//!         line-comment(trivia: true) { line-comment: "//" }
//!         block-comment(trivia: true) { block-comment: "/*", close: "*/", nested: true }
//!         fn { literal: "fn" }                    // 关键字/标点按字面量
//!         ident { charset: "a-zA-Z_" }            // 受限字符集（非任意 regex）
//!         string { quote: "\"", escape: true }
//!     }
//!     rules {
//!         program { list(decl) }                  // list = ListPlain（项间 stmt-end 分隔）
//!         decl(recover: true) { alt { api-decl } alt { note-decl } }
//!         api-decl { tok(api, field: "kw") tok(ident, field: "name") ... }
//!     }
//!     pratt(expr) {
//!         primary: "atom-expr"
//!         op(binding: 1, assoc: "left") { of: "<" }
//!         group: ["(", ")"]
//!         call { args: "args", same-line: true }
//!     }
//!     outline { decl-rule: "decl", name-field: "name", params-field: "params" }
//! }
//! ```
//! 生成期校验（拒绝并定位，不静默改语法）：未知 token/rule 引用、重复定义、
//! 左递归（经可空前缀的环）、可空 repeat、非前进 scanner 规则、Pratt binding
//! 越界。步骤简写：`lit(of)`/`tok(of)`/`ref(of)`/`pratt(of)`，或直写规则名/
//! token 名；`list`/`list-nl` 子节点包装列表步。

use ac_syntax_rt::program::*;
use ac_syntax_rt::{
    identity, GRAMMAR_SCHEMA_VERSION_STR, PARSER_ABI_VERSION, RUNTIME_VERSION, SCANNER_ABI_VERSION,
};
use auto_atom::{Atom as AtomDoc, AtomParser};

#[derive(Clone, Debug, PartialEq)]
pub struct CompileDiag {
    pub message: String,
    /// 配置中的节点路径（Atom 无字节位置，用路径标注定位）。
    pub path: String,
}

/// 编译入口：Atom 文本 → 可执行语法程序（身份已绑定）。
pub fn compile(source: &str) -> Result<GrammarProgram, Vec<CompileDiag>> {
    let atom = AtomParser::parse(source).map_err(|e| {
        vec![CompileDiag {
            message: format!("atom parse error: {e:?}"),
            path: "<document>".into(),
        }]
    })?;
    let mut c = Ctx { diags: Vec::new() };
    let program = c.build(atom)?;
    if !c.diags.is_empty() {
        return Err(c.diags);
    }
    Ok(program)
}

struct Ctx {
    diags: Vec<CompileDiag>,
}

impl Ctx {
    fn err(&mut self, path: &str, msg: String) {
        self.diags.push(CompileDiag {
            message: msg,
            path: path.to_string(),
        });
    }

    fn build(&mut self, atom: AtomDoc) -> Result<GrammarProgram, Vec<CompileDiag>> {
        let AtomDoc::Node(root) = atom else {
            return Err(vec![CompileDiag {
                message: "grammar document must be a `grammar(...)` node".into(),
                path: "<document>".into(),
            }]);
        };
        if root.name.as_str() != "grammar" {
            return Err(vec![CompileDiag {
                message: format!("root node must be `grammar`, found `{}`", root.name),
                path: "<document>".into(),
            }]);
        }
        let name = pstr(&root, "name").unwrap_or_else(|| {
            self.err("grammar", "missing `name`".into());
            "unnamed".into()
        });
        let revision = pint(&root, "revision").unwrap_or(1) as u64;
        let newline_significant = pbool(&root, "newline").unwrap_or(false);
        let stmt_end: Vec<String> = plist(&root, "stmt-end").unwrap_or_default();
        let sync: Vec<String> = plist(&root, "sync").unwrap_or_default();
        let sep: Vec<String> = plist(&root, "sep").unwrap_or_default();

        let mut tokens_node = None;
        let mut rules_node = None;
        let mut pratt_nodes: Vec<auto_val::Node> = Vec::new();
        let mut outline = OutlineSpec {
            decl_rule: String::new(),
            name_field: String::new(),
            params_field: None,
        };
        let mut has_outline = false;
        for (_k, kid) in root.kids_iter() {
            let node = match kid {
                auto_val::Kid::Node(n) => n.as_ref(),
                auto_val::Kid::Lazy(_) => continue,
            };
            match node.name.as_str() {
                "tokens" => tokens_node = Some(node.clone()),
                "rules" => rules_node = Some(node.clone()),
                "pratt" => pratt_nodes.push(node.clone()),
                "outline" => {
                    has_outline = true;
                    outline.decl_rule = pstr(node, "decl-rule").unwrap_or_default();
                    outline.name_field = pstr(node, "name-field").unwrap_or_default();
                    outline.params_field = pstr(node, "params-field");
                }
                other => self.err("grammar", format!("unknown section `{other}`")),
            }
        }

        // ---- tokens ----
        let mut token_names: Vec<String> = Vec::new();
        let mut scan_rules: Vec<ScanRule> = Vec::new();
        let mut literal_of: Vec<Option<String>> = Vec::new();
        if let Some(tn) = &tokens_node {
            for (_k, kid) in tn.kids_iter() {
                let auto_val::Kid::Node(t) = kid else {
                    continue;
                };
                let t = t.as_ref();
                let tname = t.name.to_string();
                let id = token_names.len() as u32;
                if token_names.contains(&tname) {
                    self.err(
                        &format!("tokens.{tname}"),
                        format!("duplicate token `{tname}`"),
                    );
                }
                let trivia = pbool(t, "trivia").unwrap_or(false);
                let matcher = if let Some(lit) = pstr(t, "literal") {
                    if lit.is_empty() {
                        self.err(&format!("tokens.{tname}"), "empty literal".into());
                    }
                    literal_of.push(Some(lit.clone()));
                    Matcher::Literal(lit)
                } else if let Some(cs) = pstr(t, "charset") {
                    let cont_owned = pstr(t, "cont").unwrap_or_else(|| cs.clone());
                    let first = parse_charset(&cs);
                    let cont_set = parse_charset(&cont_owned);
                    if first.is_empty() {
                        self.err(
                            &format!("tokens.{tname}"),
                            format!(
                                "charset `{cs}` matches nothing (non-progressing scanner rule)"
                            ),
                        );
                    }
                    literal_of.push(None);
                    Matcher::Charset {
                        first,
                        cont: cont_set,
                        min_len: 1,
                    }
                } else if phas(t, "quote") {
                    let q = pstr(t, "quote").unwrap_or_else(|| "\"".into());
                    let qb = q.as_bytes().first().copied().unwrap_or(b'"');
                    let esc = pbool(t, "escape").unwrap_or(true);
                    literal_of.push(None);
                    Matcher::StringLit {
                        quote: qb,
                        escape: esc,
                    }
                } else if let Some(start) = pstr(t, "line-comment") {
                    literal_of.push(None);
                    Matcher::LineComment(start)
                } else if phas(t, "block-comment") {
                    let start = pstr(t, "block-comment").unwrap_or_else(|| "/*".into());
                    let end = pstr(t, "close").unwrap_or_else(|| "*/".into());
                    let nested = pbool(t, "nested").unwrap_or(false);
                    literal_of.push(None);
                    Matcher::BlockComment { start, end, nested }
                } else {
                    self.err(
                        &format!("tokens.{tname}"),
                        format!("token `{tname}` has no matcher (literal/charset/quote/line-comment/block-comment)"),
                    );
                    literal_of.push(None);
                    Matcher::Literal("\u{0}impossible".into())
                };
                token_names.push(tname.clone());
                scan_rules.push(ScanRule {
                    mode: 0,
                    matcher,
                    token: id,
                    trivia,
                    next_mode: 0,
                });
            }
        }
        if token_names.is_empty() {
            self.err("tokens", "no tokens declared".into());
        }

        let tok_of_literal = |lit: &str| -> Option<u32> {
            literal_of
                .iter()
                .position(|l| l.as_deref() == Some(lit))
                .map(|i| i as u32)
        };
        let tok_of_name =
            |n: &str| -> Option<u32> { token_names.iter().position(|t| t == n).map(|i| i as u32) };

        // ---- rules（两遍：名字先行，允许前向引用）----
        let mut rule_names: Vec<String> = Vec::new();
        let mut rule_recover: Vec<bool> = Vec::new();
        let mut raw_rules: Vec<auto_val::Node> = Vec::new();
        if let Some(rn) = &rules_node {
            for (_k, kid) in rn.kids_iter() {
                let auto_val::Kid::Node(r) = kid else {
                    continue;
                };
                let r = r.as_ref();
                let rname = r.name.to_string();
                if rule_names.contains(&rname) {
                    self.err(
                        &format!("rules.{rname}"),
                        format!("duplicate rule `{rname}`"),
                    );
                }
                rule_recover.push(pbool(r, "recover").unwrap_or(false));
                rule_names.push(rname);
                raw_rules.push(r.clone());
            }
        }
        let rule_of =
            |n: &str| -> Option<u32> { rule_names.iter().position(|r| r == n).map(|i| i as u32) };

        let mut field_names: Vec<String> = Vec::new();
        let mut field_of = |n: &str| -> u32 {
            match field_names.iter().position(|f| f == n) {
                Some(i) => i as u32,
                None => {
                    field_names.push(n.to_string());
                    (field_names.len() - 1) as u32
                }
            }
        };

        let mut parse_steps = |ctx: &mut Ctx,
                               node: &auto_val::Node,
                               rule_path: &str|
         -> Vec<Step> {
            let mut steps = Vec::new();
            for (_k, kid) in node.kids_iter() {
                let auto_val::Kid::Node(s) = kid else {
                    continue;
                };
                let s = s.as_ref();
                let sname = s.name.to_string();
                // `list(...)`/`list-nl(...)`/`many(...)` 包装：把被包装步骤升级为列表步。
                let (wrapper, inner): (Option<&str>, Option<&auto_val::Node>) = match sname.as_str()
                {
                    "list" | "list-nl" | "many" | "opt" => {
                        // 形态：子节点 `list { x() {} }`（x 为被包装步骤）。
                        let mut inner_node = None;
                        for (_k2, kid2) in s.kids_iter() {
                            if let auto_val::Kid::Node(sub) = kid2 {
                                inner_node = Some(sub.as_ref());
                            }
                        }
                        (Some(sname.as_str()), inner_node)
                    }
                    _ => (None, None),
                };
                let target: &auto_val::Node = inner.unwrap_or(s);
                let sname2 = if inner.is_some() {
                    target.name.to_string()
                } else {
                    sname.clone()
                };
                let repeat = if let Some(w) = wrapper {
                    match w {
                        "list" => Repeat::ListPlain,
                        "list-nl" => Repeat::ListNewline,
                        "many" => Repeat::ZeroOrMore,
                        _ => Repeat::Optional,
                    }
                } else {
                    match pstr(s, "repeat").as_deref() {
                        None | Some("once") => Repeat::Once,
                        Some("opt") => Repeat::Optional,
                        Some("many") => Repeat::ZeroOrMore,
                        Some("list") => Repeat::ListPlain,
                        Some("list-nl") => Repeat::ListNewline,
                        Some(other) => {
                            ctx.err(rule_path, format!("unknown repeat `{other}`"));
                            Repeat::Once
                        }
                    }
                };
                let src_props = if inner.is_some() { target } else { s };
                let field = pstr(src_props, "field").map(|f| field_of(&f));
                let same_line = pbool(src_props, "same-line").unwrap_or(false);
                let atom = match sname2.as_str() {
                    "lit" => {
                        let litv = pstr(src_props, "of").unwrap_or_default();
                        match tok_of_literal(&litv) {
                            Some(t) => Atom::Tok(t),
                            None => {
                                ctx.err(
                                    rule_path,
                                    format!("lit `{litv}` has no matching token literal"),
                                );
                                Atom::Tok(0)
                            }
                        }
                    }
                    "tok" => {
                        let tn = pstr(src_props, "of").unwrap_or_default();
                        match tok_of_name(&tn) {
                            Some(t) => Atom::Tok(t),
                            None => {
                                ctx.err(rule_path, format!("tok `{tn}` unknown"));
                                Atom::Tok(0)
                            }
                        }
                    }
                    "ref" => {
                        let rn = pstr(src_props, "of").unwrap_or_default();
                        match rule_of(&rn) {
                            Some(r) => Atom::Rule(r),
                            None => {
                                ctx.err(rule_path, format!("ref `{rn}` unknown"));
                                Atom::Rule(0)
                            }
                        }
                    }
                    "pratt" => {
                        let pn = pstr(src_props, "of").unwrap_or_default();
                        match pratt_nodes
                            .iter()
                            .position(|p| pos_name(p).as_deref() == Some(pn.as_str()))
                        {
                            Some(i) => Atom::Pratt(i as u32),
                            None => {
                                ctx.err(rule_path, format!("pratt `{pn}` unknown"));
                                Atom::Pratt(0)
                            }
                        }
                    }
                    other => {
                        if let Some(r) = rule_of(other) {
                            Atom::Rule(r)
                        } else if let Some(t) = tok_of_name(other) {
                            Atom::Tok(t)
                        } else {
                            ctx.err(
                                rule_path,
                                format!("step `{other}` is neither lit/tok/ref/pratt nor a known rule/token"),
                            );
                            Atom::Tok(0)
                        }
                    }
                };
                steps.push(Step {
                    atom,
                    field,
                    repeat,
                    same_line,
                });
            }
            steps
        };

        let mut rules: Vec<RuleSpec> = Vec::new();
        for (i, r) in raw_rules.iter().enumerate() {
            let rname = &rule_names[i];
            let path = format!("rules.{rname}");
            let mut alts: Vec<Alt> = Vec::new();
            let has_alt_children = r
                .kids_iter()
                .any(|(_k, kid)| matches!(kid, auto_val::Kid::Node(n) if n.name.as_str() == "alt"));
            if has_alt_children {
                for (_k, kid) in r.kids_iter() {
                    let auto_val::Kid::Node(a) = kid else {
                        continue;
                    };
                    let a = a.as_ref();
                    if a.name.as_str() != "alt" {
                        self.err(
                            &path,
                            format!("mixed alt/non-alt children in rule `{rname}`"),
                        );
                        continue;
                    }
                    alts.push(Alt {
                        steps: parse_steps(self, a, &path),
                    });
                }
            } else {
                alts.push(Alt {
                    steps: parse_steps(self, r, &path),
                });
            }
            if alts.is_empty() {
                self.err(&path, format!("rule `{rname}` has no alternatives"));
            }
            rules.push(RuleSpec {
                name: rname.clone(),
                alts,
                recover: rule_recover[i],
            });
        }

        // ---- pratt 表体 ----
        let mut pratt: Vec<PrattSpec> = Vec::new();
        for pn in &pratt_nodes {
            let path = format!("pratt.{}", pn.name);
            let primary = pstr(pn, "primary")
                .and_then(|n| rule_of(&n))
                .unwrap_or_else(|| {
                    self.err(&path, "primary rule unknown".into());
                    0
                });
            let mut ops = Vec::new();
            for (_k, kid) in pn.kids_iter() {
                let auto_val::Kid::Node(o) = kid else {
                    continue;
                };
                let o = o.as_ref();
                if o.name.as_str() != "op" {
                    continue;
                }
                let binding = pint(o, "binding").unwrap_or(0) as u8;
                let assoc = match pstr(o, "assoc").as_deref() {
                    Some("right") => Assoc::Right,
                    _ => Assoc::Left,
                };
                if binding == 0 || binding > 100 {
                    self.err(&path, format!("op binding {binding} out of (0,100]"));
                }
                let lit = pstr(o, "of").unwrap_or_default();
                match tok_of_literal(&lit) {
                    Some(t) => ops.push(PrattOp {
                        op: t,
                        binding,
                        assoc,
                    }),
                    None => self.err(&path, format!("op literal `{lit}` has no token")),
                }
            }
            let group = plist(pn, "group");
            let (group_open, group_close) = match &group {
                Some(g) if g.len() == 2 => (tok_of_literal(&g[0]), tok_of_literal(&g[1])),
                _ => (None, None),
            };
            let mut call_paren = None;
            let mut call_args = None;
            for (_k, kid) in pn.kids_iter() {
                let auto_val::Kid::Node(c) = kid else {
                    continue;
                };
                let c = c.as_ref();
                if c.name.as_str() == "call" {
                    let open = pstr(c, "open").unwrap_or_else(|| "(".into());
                    let close = pstr(c, "close").unwrap_or_else(|| ")".into());
                    let same_line = pbool(c, "same-line").unwrap_or(false);
                    let nl_free = pbool(c, "nl-free").unwrap_or(true);
                    call_paren = Some(CallParenRule {
                        open: tok_of_literal(&open).unwrap_or(0),
                        close: tok_of_literal(&close).unwrap_or(0),
                        newline_free_inside: nl_free,
                        callee_same_line: same_line,
                    });
                    if let Some(ar) = pstr(c, "args") {
                        call_args = rule_of(&ar);
                        if call_args.is_none() {
                            self.err(&path, format!("call args rule `{ar}` unknown"));
                        }
                    }
                }
            }
            pratt.push(PrattSpec {
                primary,
                ops,
                group_open,
                group_close,
                call_paren,
                call_args,
            });
        }

        if has_outline {
            if outline.decl_rule.is_empty() || outline.name_field.is_empty() {
                self.err("outline", "decl-rule/name-field required".into());
            }
            if rule_of(&outline.decl_rule).is_none() {
                self.err(
                    "outline",
                    format!("decl-rule `{}` unknown", outline.decl_rule),
                );
            }
        }

        self.check_grammar(&rules)?;

        let stmt_end_ids: Vec<u32> = stmt_end
            .iter()
            .filter_map(|s| {
                let id = tok_of_name(s);
                if id.is_none() {
                    self.err("grammar", format!("stmt-end `{s}` unknown"));
                }
                id
            })
            .collect();
        let sync_ids: Vec<u32> = sync
            .iter()
            .filter_map(|s| {
                let id = tok_of_name(s);
                if id.is_none() {
                    self.err("grammar", format!("sync `{s}` unknown"));
                }
                id
            })
            .collect();
        let sep_ids: Vec<u32> = sep
            .iter()
            .filter_map(|s| {
                let id = tok_of_name(s);
                if id.is_none() {
                    self.err("grammar", format!("sep `{s}` unknown"));
                }
                id
            })
            .collect();
        // 未声明 sep 时回退为 stmt_end 全集（兼容旧形态；含 `}` 由边界检查兜底）。
        let sep_ids = if sep_ids.is_empty() {
            stmt_end_ids.clone()
        } else {
            sep_ids
        };
        let start = rule_of("program").unwrap_or_else(|| {
            self.err("rules", "start rule `program` missing".into());
            0
        });

        let mut p = GrammarProgram {
            name,
            revision,
            schema_version: GRAMMAR_SCHEMA_VERSION,
            scanner: ScannerProgram {
                mode_names: vec!["code".into()],
                rules: scan_rules,
            },
            tokens: token_names
                .into_iter()
                .map(|n| TokenSpec { name: n })
                .collect(),
            rules,
            pratt,
            field_names,
            start,
            newline: NewlinePolicy {
                significant: newline_significant,
                stmt_end: stmt_end_ids,
                sep_consume: sep_ids,
            },
            recovery: RecoverySpec { sync: sync_ids },
            outline: if has_outline { Some(outline) } else { None },
            identity: identity::PackageIdentity {
                name: String::new(),
                grammar_id: identity::GrammarId(String::new()),
            },
        };
        let canonical = p.canonical_bytes();
        let versions = [
            GRAMMAR_SCHEMA_VERSION_STR,
            RUNTIME_VERSION,
            SCANNER_ABI_VERSION,
            PARSER_ABI_VERSION,
        ];
        p.identity = identity::package_identity(&p.name, &canonical, &versions);
        Ok(p)
    }

    /// 生成期检查：可空性 fixpoint、可空 repeat 拒绝、左递归（经可空前缀的环）。
    fn check_grammar(&mut self, rules: &[RuleSpec]) -> Result<(), Vec<CompileDiag>> {
        let n = rules.len();
        if n == 0 {
            self.err("rules", "no rules".into());
            return Err(self.diags.clone());
        }
        let mut nullable = vec![false; n];
        let mut changed = true;
        while changed {
            changed = false;
            for (ri, r) in rules.iter().enumerate() {
                if nullable[ri] {
                    continue;
                }
                for alt in &r.alts {
                    // 步可空 = 原子可空 **或** repeat 为 opt/many/list（fixpoint 语义）。
                    let alt_nullable = alt.steps.iter().all(|s| {
                        let atom_null = match s.atom {
                            Atom::Tok(_) => false,
                            Atom::Rule(sub) => nullable.get(sub as usize).copied().unwrap_or(false),
                            Atom::Pratt(_) => false,
                        };
                        atom_null
                            || matches!(
                                s.repeat,
                                Repeat::Optional
                                    | Repeat::ZeroOrMore
                                    | Repeat::ListPlain
                                    | Repeat::ListNewline
                            )
                    });
                    if alt_nullable {
                        nullable[ri] = true;
                        changed = true;
                        break;
                    }
                }
            }
        }
        for r in rules {
            for alt in &r.alts {
                for s in &alt.steps {
                    let item_null = match s.atom {
                        Atom::Tok(_) => false,
                        Atom::Rule(sub) => nullable.get(sub as usize).copied().unwrap_or(false),
                        Atom::Pratt(_) => false,
                    };
                    if item_null
                        && matches!(
                            s.repeat,
                            Repeat::ZeroOrMore | Repeat::ListPlain | Repeat::ListNewline
                        )
                    {
                        self.err(
                            &format!("rules.{}", r.name),
                            "repeat of nullable item is forbidden: zero-progress loop".into(),
                        );
                    }
                }
            }
        }
        let mut state = vec![0u8; n];
        fn dfs(
            ri: usize,
            rules: &[RuleSpec],
            nullable: &[bool],
            state: &mut [u8],
            diags: &mut Vec<CompileDiag>,
            path: &mut Vec<usize>,
        ) {
            if state[ri] == 1 {
                let cycle: Vec<String> = path.iter().map(|&i| rules[i].name.clone()).collect();
                diags.push(CompileDiag {
                    message: format!("left recursion detected: {}", cycle.join(" -> ")),
                    path: format!("rules.{}", rules[ri].name),
                });
                return;
            }
            if state[ri] == 2 {
                return;
            }
            state[ri] = 1;
            path.push(ri);
            for alt in &rules[ri].alts {
                for s in &alt.steps {
                    if let Atom::Rule(sub) = s.atom {
                        dfs(sub as usize, rules, nullable, state, diags, path);
                    }
                    let snull = match s.atom {
                        Atom::Tok(_) => false,
                        Atom::Rule(sub) => nullable[sub as usize],
                        Atom::Pratt(_) => false,
                    };
                    let rep_nullable = matches!(
                        s.repeat,
                        Repeat::Optional
                            | Repeat::ZeroOrMore
                            | Repeat::ListPlain
                            | Repeat::ListNewline
                    );
                    if !(snull || rep_nullable) {
                        break;
                    }
                }
            }
            path.pop();
            state[ri] = 2;
        }
        let mut path = Vec::new();
        for i in 0..n {
            dfs(i, rules, &nullable, &mut state, &mut self.diags, &mut path);
        }
        Ok(())
    }
}

/// 属性读取（auto-val `get_prop_of` 返回 Value，Nil = 缺席）。
fn pstr(n: &auto_val::Node, key: &str) -> Option<String> {
    as_str(&n.get_prop_of(key))
}
fn pint(n: &auto_val::Node, key: &str) -> Option<i64> {
    as_int(&n.get_prop_of(key))
}
fn pbool(n: &auto_val::Node, key: &str) -> Option<bool> {
    as_bool(&n.get_prop_of(key))
}
fn plist(n: &auto_val::Node, key: &str) -> Option<Vec<String>> {
    as_str_list(&n.get_prop_of(key))
}
fn phas(n: &auto_val::Node, key: &str) -> bool {
    !matches!(n.get_prop_of(key), auto_val::Value::Nil)
}
/// 位置参数名（`pratt(expr-table)` 的表名）。
fn pos_name(n: &auto_val::Node) -> Option<String> {
    for (k, v) in n.props_iter() {
        if let (auto_val::ValueKey::Str(key), auto_val::Value::Str(s)) = (k, v) {
            if key.is_empty() {
                return Some(s.to_string());
            }
        }
    }
    None
}

fn as_str(v: &auto_val::Value) -> Option<String> {
    match v {
        auto_val::Value::Str(s) => Some(s.to_string()),
        _ => None,
    }
}
fn as_int(v: &auto_val::Value) -> Option<i64> {
    match v {
        auto_val::Value::Int(i) => Some(*i as i64),
        auto_val::Value::Double(d) => Some(*d as i64),
        _ => None,
    }
}
fn as_bool(v: &auto_val::Value) -> Option<bool> {
    match v {
        auto_val::Value::Bool(b) => Some(*b),
        _ => None,
    }
}
fn as_str_list(v: &auto_val::Value) -> Option<Vec<String>> {
    match v {
        auto_val::Value::Array(a) => {
            let mut out = Vec::new();
            for x in &a.values {
                if let Some(s) = as_str(x) {
                    out.push(s);
                }
            }
            Some(out)
        }
        _ => None,
    }
}

/// 字符集语法：`a-zA-Z0-9_ \t` —— 区间与显式字符（受限集合，非任意 regex）。
pub fn parse_charset(spec: &str) -> CharSet {
    let mut cs = CharSet::empty();
    let ch: Vec<char> = spec.chars().collect();
    let mut i = 0;
    while i < ch.len() {
        let a = ch[i];
        if i + 2 < ch.len() && ch[i + 1] == '-' {
            let b = ch[i + 2];
            if (b as u32) >= (a as u32) && (b as u32) < 128 {
                let mut c = a as u32;
                while c <= b as u32 {
                    cs.add(c as u8);
                    c += 1;
                }
                i += 3;
                continue;
            }
        }
        if (a as u32) < 128 {
            cs.add(a as u8);
        }
        i += 1;
    }
    cs
}
