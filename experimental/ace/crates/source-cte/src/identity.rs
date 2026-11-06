//! Identity of the source-CTE lane (PLAN-936 §5.7).
//!
//! Three axes, kept strictly apart:
//!
//! * **Execution semantics** ([`ExecKey`]) — everything that must match
//!   for a frozen execution to be REUSABLE: the callee's logical id, its
//!   signature version, its body version, the transitive
//!   callee-closure versions (a helper body edit changes the key), the
//!   canonical argument values, the capability set and the engine axis.
//!   Deliberately ABSENT: any whole-module source hash, the calling
//!   site's span, and any provenance revision — an unrelated edit
//!   elsewhere in the module must not invalidate a CT result.
//! * **This owner's consumption** ([`UseBinding`]) — the current module,
//!   owner function and expected type of ONE consuming site. It travels
//!   with the request, gates the value's admission at the owner (strict
//!   tag: an `i32` site accepts only `I32`), and is NEVER written into
//!   the cached page: the same ExecKey answering two owners is reuse,
//!   not a stale binding.
//! * **Engine/producer identity** — the schema/producer/lane constants
//!   below. A page from any other producer (the 817 `plan817/cte-probe`
//!   included) is a controlled admission rejection, never a
//!   reinterpretation.
//!
//! Hashing reuses the 934 discipline: length-prefixed blake3 over
//! producer ‖ domain ‖ parts, so an `ac936/*` id can never collide with
//! an `ac817/*` or `ac934/*` id even for identical parts.

use ace_core::source::ast::{Block, Expr, ExprKind, Item, Program, Stmt};
use ace_semantics_contract::ContentVersion;
use ace_source_query::ident::{owner_logical, signature_version};
use ace_source_query::walk_body;
use blake3::Hasher;

/// Producer identity of the source-CTE service.
pub const CT_PRODUCER: &str = "plan936/source-cte";
/// Executed-envelope schema of the new lane (a version bump must MISS,
/// never reinterpret old bytes).
pub const CT_SCHEMA: &str = "ac936/ct-exec/v1";
/// The capability profile that admits `#{ expr }` on the source surface
/// (§5.2; the grammar gate itself lands with T-04).
pub const CT_PROFILE: &str = "ac.source.cte.scalar/v1";
/// Query algorithm identity of this lane.
pub const CT_ALGO: &str = "ac936-ct-query/1";
/// The engine lane axis. The 817 closure/snapshot lanes key their own
/// rows; this lane never shares a cache line with either.
pub const CT_LANE: &str = "source-cte";

/// Hash domain of a source-CTE execution key.
pub const EXEC_KEY_DOMAIN: &str = "ac936/ct-exec-key/v1";

/// The scalar CT value surface (i32/bool only — §5.2 non-goals keep
/// every other runtime value out of the CT lane).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CtValue {
    I32(i32),
    Bool(bool),
}

impl CtValue {
    /// The canonical argument/value encoding that keys and envelopes
    /// carry (the 817 `args_canonical` shape: `i32:20`, `bool:true`).
    pub fn canonical(self) -> String {
        match self {
            CtValue::I32(x) => format!("i32:{x}"),
            CtValue::Bool(b) => format!("bool:{b}"),
        }
    }
}

/// The expected type of one CT demand. Strict: an `I32` expectation
/// accepts only an `I32` AV value and a `Bool` expectation only a `Bool`
/// — the 817 nonzero-I32→true projection does not exist in this lane.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExpectTy {
    I32,
    Bool,
}

impl ExpectTy {
    pub fn as_str(self) -> &'static str {
        match self {
            ExpectTy::I32 => "i32",
            ExpectTy::Bool => "bool",
        }
    }
}

/// The engine axis of one execution (schema constants re-read from
/// av-core so any future bump re-keys automatically, mirroring the 817
/// `CteEngineId` discipline with this lane's own identity strings).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CtEngine {
    pub av_profile: String,
    pub mir_schema: String,
    pub abc_schema: String,
    pub mir_semantics: String,
    pub abc_semantics: String,
    pub av_semantics: String,
    pub lane: &'static str,
}

impl CtEngine {
    /// The engine identity of a real AV execution on this lane.
    pub fn current() -> CtEngine {
        use ace_av_core::contract::{
            ABC_SCHEMA, ABC_SEMANTICS, AV_PROFILE, AV_SEMANTICS, MIR_SCHEMA, MIR_SEMANTICS,
        };
        CtEngine {
            av_profile: AV_PROFILE.to_string(),
            mir_schema: MIR_SCHEMA.to_string(),
            abc_schema: ABC_SCHEMA.to_string(),
            mir_semantics: MIR_SEMANTICS.0.to_string(),
            abc_semantics: ABC_SEMANTICS.0.to_string(),
            av_semantics: AV_SEMANTICS.0.to_string(),
            lane: CT_LANE,
        }
    }

    /// Canonical encoding for keying/hashing.
    pub fn canonical(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}|{}|lane:{}",
            self.av_profile,
            self.mir_schema,
            self.abc_schema,
            self.mir_semantics,
            self.abc_semantics,
            self.av_semantics,
            self.lane
        )
    }
}

/// One closure member's identity in the execution key: the callee's
/// logical id with the signature and body versions that were in force.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ClosureVersion {
    pub name: String,
    pub logical: String,
    pub sig_version: String,
    pub body_version: String,
}

/// The execution-semantics key (§5.7 table, row 1).
///
/// T-02 surface: free functions of one module; the transitive closure is
/// harvested from the program's real call graph (an edit to ANY closure
/// member's body or signature changes the key). The instance/meta axes
/// (935 recipes, `ct_field_count` selectors) join the key with T-08.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExecKey {
    pub package: String,
    pub module: String,
    pub callee: String,
    pub callee_logical: String,
    pub callee_sig_version: String,
    pub callee_body_version: String,
    /// Signature+body versions of every transitively called function,
    /// canonical-sorted — the helper-edit invalidation axis (V-11-lite).
    pub closure: Vec<ClosureVersion>,
    pub args_canonical: String,
    pub expect: ExpectTy,
    pub caps: Vec<String>,
    pub algo: &'static str,
    pub engine: CtEngine,
}

impl ExecKey {
    /// The complete canonical string. Every component participates; a
    /// change of any one is a different execution.
    pub fn canonical(&self) -> String {
        let mut closure = self
            .closure
            .iter()
            .map(|c| {
                format!(
                    "{}:{}:sig={}:body={}",
                    c.name, c.logical, c.sig_version, c.body_version
                )
            })
            .collect::<Vec<_>>();
        closure.sort();
        format!(
            "pkg:{}|mod:{}|callee:{}|logical:{}|sig:{}|body:{}|closure[{}]|args:{}|expect:{}|caps[{}]|algo:{}|engine:{}",
            self.package,
            self.module,
            self.callee,
            self.callee_logical,
            self.callee_sig_version,
            self.callee_body_version,
            closure.join(","),
            self.args_canonical,
            self.expect.as_str(),
            self.caps.join(","),
            self.algo,
            self.engine.canonical(),
        )
    }

    /// The content hash of the canonical key (the store's single key
    /// component).
    pub fn hash(&self) -> String {
        ct_hash(EXEC_KEY_DOMAIN, &[self.canonical().as_bytes()])
    }
}

/// The this-owner consumption binding (§5.7 table, row 2). Travels with
/// the request; never enters the cached page.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct UseBinding {
    pub package: String,
    pub module: String,
    /// The owner function whose body carries the CT demand.
    pub owner: String,
    /// The expected type at the site (strict admission at the owner).
    pub expect: ExpectTy,
}

/// Domain-separated content hash over `CT_PRODUCER ‖ domain ‖ parts`
/// (same discipline as 934's `sq_hash`; different producer prefix).
pub fn ct_hash(domain: &str, parts: &[&[u8]]) -> String {
    let mut h = Hasher::new();
    h.update(CT_PRODUCER.as_bytes());
    h.update(&[0]);
    h.update(domain.as_bytes());
    h.update(&[0]);
    for p in parts {
        h.update(&(p.len() as u64).to_le_bytes());
        h.update(p);
    }
    h.finalize().to_hex().to_string()
}

/// Mint a content version in this lane's domain.
pub fn ct_content_version(canonical: &[u8]) -> ContentVersion {
    ContentVersion(ct_hash(EXEC_KEY_DOMAIN, &[canonical]))
}

/// The general-expression wrapper entry name (T-06; distinct from the
/// call-form entry so the two can never collide).
pub const CT_EXPR_ENTRY: &str = "ct936_expr";

/// Canonical structural encoding of one (substituted) CT expression —
/// spans excluded (whitespace drift keeps the key), structure included.
pub fn expr_canonical(e: &Expr, out: &mut Vec<u8>) {
    use ace_core::source::ast::Arg;
    match &e.kind {
        ExprKind::IntLit(v) => {
            out.extend_from_slice(b"int:");
            out.extend_from_slice(v.to_string().as_bytes());
        }
        ExprKind::BoolLit(v) => {
            out.extend_from_slice(b"bool:");
            out.extend_from_slice(v.to_string().as_bytes());
        }
        ExprKind::Var(id) => {
            out.extend_from_slice(b"var:");
            out.extend_from_slice(id.name.as_bytes());
        }
        ExprKind::Call { callee, args } => {
            out.extend_from_slice(b"call:");
            out.extend_from_slice(callee.name.as_bytes());
            out.push(b'(');
            for a in args {
                match a {
                    Arg::Pos(x) | Arg::MovePos(x) => expr_canonical(x, out),
                    _ => out.push(b'?'),
                }
                out.push(b',');
            }
            out.push(b')');
        }
        ExprKind::Binary { op, lhs, rhs } => {
            out.extend_from_slice(b"bin:");
            out.extend_from_slice(op.as_str().as_bytes());
            out.push(b'(');
            expr_canonical(lhs, out);
            out.push(b',');
            expr_canonical(rhs, out);
            out.push(b')');
        }
        ExprKind::Unary { op, operand } => {
            out.extend_from_slice(b"un:");
            out.extend_from_slice(op.as_str().as_bytes());
            out.push(b'(');
            expr_canonical(operand, out);
            out.push(b')');
        }
        other => {
            out.extend_from_slice(format!("other:{other:?}").as_bytes());
        }
    }
}

/// Build the execution key for one GENERAL-expression CT demand: the
/// expression's canonical hash IS the callee-axis identity; the closure
/// versions of every called function ride the same closure axis (T-06).
pub fn exec_key_of_expr(
    program: &Program,
    package: &str,
    module: &str,
    expr: &Expr,
    expect: ExpectTy,
    caps: &[String],
) -> Result<ExecKey, String> {
    let mut canon = Vec::new();
    expr_canonical(expr, &mut canon);
    let expr_hash = ct_hash(EXEC_KEY_DOMAIN, &[&canon]);
    // The expression's direct callees + their transitive closure.
    let mut names = Vec::new();
    walk_expr(expr, &mut names);
    let mut closure = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    let mut queue = names;
    while let Some(name) = queue.pop() {
        if seen.contains(&name) {
            continue;
        }
        seen.push(name.clone());
        if let Some(f) = program.items.iter().find_map(|i| match i {
            Item::Fn(f) if f.name.name == name => Some(f),
            _ => None,
        }) {
            let sig = signature_version(f).as_str().to_string();
            let body = walk_body(&f.body)
                .map_err(|e| format!("closure member `{name}` body walk failed: {e:?}"))?
                .version
                .as_str()
                .to_string();
            closure.push(ClosureVersion {
                logical: owner_logical(package, module, &name).as_str().to_string(),
                name: name.clone(),
                sig_version: sig,
                body_version: body,
            });
            let mut next = Vec::new();
            walk_expr_fn_body(&f.body, &mut next);
            for n2 in next {
                if !seen.contains(&n2) {
                    queue.push(n2);
                }
            }
        }
    }
    Ok(ExecKey {
        package: package.to_string(),
        module: module.to_string(),
        callee: CT_EXPR_ENTRY.to_string(),
        callee_logical: CT_EXPR_ENTRY.to_string(),
        callee_sig_version: expr_hash.clone(),
        callee_body_version: expr_hash,
        closure,
        args_canonical: String::new(),
        expect,
        caps: caps.to_vec(),
        algo: CT_ALGO,
        engine: CtEngine::current(),
    })
}

fn walk_expr_fn_body(body: &Block, out: &mut Vec<String>) {
    for stmt in &body.stmts {
        walk_stmt(stmt, out);
    }
}

/// Build the execution key for one CT demand against a parsed program.
///
/// The callee must be a declared free function of the program; its
/// signature/body versions and the transitive closure versions are
/// derived from the CURRENT program (the real structural walk — never a
/// name-only list), so a helper body edit between two requests yields a
/// different key with no store interaction.
pub fn exec_key_of(
    program: &Program,
    package: &str,
    module: &str,
    callee: &str,
    args: &[CtValue],
    expect: ExpectTy,
    caps: &[String],
) -> Result<ExecKey, String> {
    let target = find_fn(program, callee).ok_or_else(|| {
        format!("callee `{callee}` is not a declared free function of the program")
    })?;
    let sig = signature_version(target).as_str().to_string();
    let body = walk_body(&target.body)
        .map_err(|e| format!("callee `{callee}` body walk failed: {e:?}"))?
        .version
        .as_str()
        .to_string();
    let mut closure = vec![ClosureVersion {
        name: callee.to_string(),
        logical: owner_logical(package, module, callee).as_str().to_string(),
        sig_version: sig.clone(),
        body_version: body.clone(),
    }];
    // Transitive closure: DFS over the intra-module call graph from the
    // callee. Cycle-safe (recursion among functions is legal source and
    // simply rejoins the visited set); a call to an undeclared name is
    // left to the resolver's rejection later — the key only records
    // declared closure members.
    let mut queue = direct_callees(&target.body);
    let mut seen: Vec<String> = vec![callee.to_string()];
    while let Some(name) = queue.pop() {
        if seen.contains(&name) {
            continue;
        }
        seen.push(name.clone());
        let Some(f) = find_fn(program, &name) else {
            continue;
        };
        let c_sig = signature_version(f).as_str().to_string();
        let c_body = walk_body(&f.body)
            .map_err(|e| format!("closure member `{name}` body walk failed: {e:?}"))?
            .version
            .as_str()
            .to_string();
        closure.push(ClosureVersion {
            logical: owner_logical(package, module, &name).as_str().to_string(),
            name: name.clone(),
            sig_version: c_sig,
            body_version: c_body,
        });
        for next in direct_callees(&f.body) {
            if !seen.contains(&next) {
                queue.push(next);
            }
        }
    }
    Ok(ExecKey {
        package: package.to_string(),
        module: module.to_string(),
        callee: callee.to_string(),
        callee_logical: owner_logical(package, module, callee).as_str().to_string(),
        callee_sig_version: sig,
        callee_body_version: body,
        closure,
        args_canonical: args
            .iter()
            .map(|v| v.canonical())
            .collect::<Vec<_>>()
            .join(","),
        expect,
        caps: caps.to_vec(),
        algo: CT_ALGO,
        engine: CtEngine::current(),
    })
}

/// The transitive closure versions of a program's callee — the T-08
/// upgrade seam carried as data (the full key already embeds them).
pub fn transitive_closure_versions(
    program: &Program,
    package: &str,
    module: &str,
    callee: &str,
) -> Result<Vec<ClosureVersion>, String> {
    Ok(exec_key_of(program, package, module, callee, &[], ExpectTy::I32, &[])?.closure)
}

fn find_fn<'a>(p: &'a Program, name: &str) -> Option<&'a ace_core::source::ast::FnItem> {
    p.items.iter().find_map(|i| match i {
        Item::Fn(f) if f.name.name == name => Some(f),
        _ => None,
    })
}

/// The direct free-function callees of one body (syntactic harvest; the
/// resolver remains the binding authority when the entry is checked).
fn direct_callees(body: &Block) -> Vec<String> {
    let mut out = Vec::new();
    for stmt in &body.stmts {
        walk_stmt(stmt, &mut out);
    }
    out
}

fn walk_stmt(stmt: &Stmt, out: &mut Vec<String>) {
    match stmt {
        Stmt::Return { value, .. } => walk_expr(value, out),
        Stmt::ExprStmt { value, .. } => walk_expr(value, out),
        Stmt::Let { value, .. } => walk_expr(value, out),
        Stmt::Assign { value, .. } => walk_expr(value, out),
        Stmt::If {
            cond, then, els, ..
        } => {
            walk_expr(cond, out);
            for s in &then.stmts {
                walk_stmt(s, out);
            }
            if let Some(b) = els {
                for s in &b.stmts {
                    walk_stmt(s, out);
                }
            }
        }
        Stmt::While { cond, body, .. } => {
            walk_expr(cond, out);
            for s in &body.stmts {
                walk_stmt(s, out);
            }
        }
        Stmt::Loop { body, .. } => {
            for s in &body.stmts {
                walk_stmt(s, out);
            }
        }
        Stmt::Break { .. } | Stmt::Continue { .. } => {}
        Stmt::IsMatch {
            scrutinee, arms, ..
        } => {
            if !out.contains(&scrutinee.name) {
                // scrutinee is a local, not a call — nothing to record
            }
            for a in arms {
                for s in &a.body.stmts {
                    walk_stmt(s, out);
                }
            }
        }
        Stmt::ForIn { iter, body, .. } => {
            walk_expr(iter, out);
            for s in &body.stmts {
                walk_stmt(s, out);
            }
        }
    }
}

fn walk_expr(e: &Expr, out: &mut Vec<String>) {
    match &e.kind {
        // PLAN-936：Hash 内的被调方进入执行闭包轴（其 body 版本入 key）。
        ExprKind::HashExpr { expr: inner } => walk_expr(inner, out),
        ExprKind::Call { callee, args } => {
            out.push(callee.name.clone());
            for a in args {
                walk_arg(a, out);
            }
        }
        ExprKind::Method { recv, args, .. } | ExprKind::MethodCall { recv, args, .. } => {
            walk_expr(recv, out);
            for a in args {
                walk_arg(a, out);
            }
        }
        ExprKind::CallValue { callee, args } => {
            walk_expr(callee, out);
            for a in args {
                walk_arg(a, out);
            }
        }
        ExprKind::Binary { lhs, rhs, .. } => {
            walk_expr(lhs, out);
            walk_expr(rhs, out);
        }
        ExprKind::Unary { operand, .. } => walk_expr(operand, out),
        ExprKind::Field { base, .. } => walk_expr(base, out),
        ExprKind::CtorNamed { .. } => {}
        ExprKind::Closure { .. } => {}
        _ => {}
    }
}

fn walk_arg(a: &ace_core::source::ast::Arg, out: &mut Vec<String>) {
    match a {
        ace_core::source::ast::Arg::Pos(e)
        | ace_core::source::ast::Arg::MovePos(e)
        | ace_core::source::ast::Arg::Named { value: e, .. } => walk_expr(e, out),
    }
}
