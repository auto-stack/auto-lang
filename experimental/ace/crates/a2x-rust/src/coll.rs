//! PLAN-904 (AX-C) collections emit file-set assembly: the typed
//! collections runtime (`a2x_runtime.rs` for the collections recipe) +
//! `generated.rs` + `main.rs` wrapper + deterministic manifest, schema
//! `auto.a2x.rust.collections.fileset/1`.
//!
//! Runtime contract (plan §5.3/§5.4, T-02 freeze):
//! * `A2xList<E>` owns one Vec-backed buffer (logical start capacity 4);
//!   growth is an explicit `try_reserve` the budget/fault layer checks
//!   BEFORE the real request; the type has NO Drop — the only release
//!   path is `a2x_coll_list_free` (elements in reverse index order,
//!   buffer last), mirroring the frame-owned LIFO contract;
//! * every mutation prechecks budget/bounds BEFORE committing — a failed
//!   push/insert/set frees its uncommitted owned input and leaves the
//!   old target intact (failure atomicity); `set` frees the old element
//!   at commit;
//! * checked views (`a2x_coll_list_borrow_elem`) never materialize: the
//!   index is checked (single `ListIndex` trap) and a reference is
//!   returned;
//! * billing (frozen table): 2 WU per element slot (new base 4 + growth),
//!   1 WU per 8 Str bytes copied, 1 WU per record/variant ctor, 4 WU per
//!   call frame, 1 WU per loop back-edge (the poll); work is deducted at
//!   the real work site and `a2x_coll_poll` is the cooperative
//!   cancellation point (trap `Cancelled`); budget exhaustion is the
//!   typed `Budget` trap;
//! * cleanup is ZERO-charge/zero-poll/zero-allocation: free paths never
//!   bill, never cancel, never allocate (the element loop walks the
//!   existing buffer);
//! * fault injection `A2X_RT_FAULT_ALLOC=<n>` fails the n-th real
//!   allocation request (list growth, Str clone/concat/literal) BEFORE
//!   the request; `A2X_RT_EVENTS=1` mirrors logical events and real
//!   backing requests to stderr (`coll:*` lines) — logical events and
//!   backing requests are listed separately, never a fake alloc/free;
//! * `A2X_RT_BUDGET=<wu>` overrides the default work budget (2^24);
//!   `A2X_RT_CANCEL=1` arms cooperative cancellation.

use crate::lower::RustProgram;
use crate::print::print_program;
use crate::target::TargetProfile;
use ace_a2x_core::{sha256::sha256_hex, SemanticView};

pub const COLL_FILESET_SCHEMA: &str = "auto.a2x.rust.collections.fileset/1";
pub const COLL_RECIPE: &str = "ac-a2r/hir-collections-emit-v1";
pub const COLL_CERT_SCHEMA: &str = "ace.a2r-hir.collections.cert/1";

/// PLAN-1115 (T-02): the ordinary-language A2R channel identities — a
/// narrow channel NEXT TO the 904 collections channel (same runtime
/// substrate, separate recipe/fileset/cert identities; cross-channel
/// pairings stay rejected).
pub const LANG_FILESET_SCHEMA: &str = "auto.a2x.rust.language.fileset/1";
pub const LANG_RECIPE: &str = "ac-a2r/hir-language-emit-v1";
pub const LANG_CERT_SCHEMA: &str = "ace.a2r-hir.language.cert/1";

/// An assembled collections file set.
pub struct CollFileSet {
    pub files: Vec<(String, String)>,
    pub manifest: String,
}

/// Canonical execution configuration (PLAN-904 §5.6): keys in fixed
/// order; any field difference is a different config identity (certs are
/// keyed per config/mode and never cross-trust).
pub fn canonical_execution_config(
    mode: &str,
    budget: u64,
    fault_alloc: Option<u64>,
    events: bool,
    cancel: bool,
) -> String {
    format!(
        "{{\"budget_work_units\":{},\"cancel\":{},\"events\":{},\"fault_alloc\":{},\"mode\":\"{}\"}}",
        budget,
        if cancel { "true" } else { "false" },
        if events { "true" } else { "false" },
        match fault_alloc {
            Some(n) => n.to_string(),
            None => "null".to_string(),
        },
        mode
    )
}

/// Assemble the complete collections file set for a lowered program. The
/// manifest binds the REPLACED FULL SOURCE sha256 (never a fragment-only
/// hash) plus the execution config identity.
pub fn assemble_collections_file_set(
    view: &SemanticView<'_>,
    program: &RustProgram,
    input_hash: &str,
    full_source_sha256: &str,
    execution_config: &str,
    entry_source_name: &str,
) -> CollFileSet {
    assemble_family_file_set(
        COLL_FILESET_SCHEMA,
        "PLAN-904 AX-C",
        view,
        program,
        input_hash,
        full_source_sha256,
        execution_config,
        entry_source_name,
    )
}

/// PLAN-1115 (T-02): assemble the ordinary-language file set — the same
/// substrate as the collections channel, under the LANGUAGE channel's own
/// fileset schema (narrow-channel identity; never interchangeable with
/// the 904 schema).
pub fn assemble_language_file_set(
    view: &SemanticView<'_>,
    program: &RustProgram,
    input_hash: &str,
    full_source_sha256: &str,
    execution_config: &str,
    entry_source_name: &str,
) -> CollFileSet {
    assemble_family_file_set(
        LANG_FILESET_SCHEMA,
        "PLAN-1115 IF-ACE-A2R-LANGUAGE/1",
        view,
        program,
        input_hash,
        full_source_sha256,
        execution_config,
        entry_source_name,
    )
}

#[allow(clippy::too_many_arguments)]
fn assemble_family_file_set(
    schema: &str,
    plan_label: &str,
    view: &SemanticView<'_>,
    program: &RustProgram,
    input_hash: &str,
    full_source_sha256: &str,
    execution_config: &str,
    entry_source_name: &str,
) -> CollFileSet {
    let target = TargetProfile::v1();

    let mut generated = String::new();
    generated.push_str(&format!(
        "// AUTO-GENERATED by {} v{} ({}) — DO NOT EDIT.\n\
         // source profile: {} | hir domain: {} | input hash (fnv1a-64): {}\n\
         // source sha256 (full replaced source): {}\n\
         // execution config: {}\n\
         // entry: `{}` -> `{}`\n\n",
        target.backend,
        target.backend_version,
        plan_label,
        view.profile,
        view.domain.as_str(),
        input_hash,
        full_source_sha256,
        execution_config,
        entry_source_name,
        program.entry
    ));
    generated.push_str("use crate::a2x_runtime::*;\n\n");
    generated.push_str(&print_program(program));

    let mut runtime = if schema == LANG_FILESET_SCHEMA {
        language_runtime_source(&program.marks)
    } else {
        collections_runtime_source(&program.marks)
    };
    // PLAN-968: append the scheduler runtime for scheduler-surface
    // programs (the generated __sched calls bind to it). PLAN-1116
    // (T-02): console-only programs print through the same runtime's
    // print helpers (a bare generator loop has no sched ops).
    if view.has_sched_ops() || view.has_console_ops() {
        runtime.push_str(include_str!("sched_rt.txt"));
    }

    let mut wrapper = String::new();
    wrapper.push_str(&format!(
        "// AUTO-GENERATED by {} v{} ({}) — DO NOT EDIT.\n",
        target.backend, target.backend_version, plan_label
    ));
    wrapper.push_str("mod a2x_runtime;\nmod generated;\n\n");
    wrapper.push_str("use a2x_runtime::A2xResult;\n\n");
    wrapper.push_str("fn main() {\n");
    // PLAN-1115 (T-03, from 981): the module init chain (dependency
    // order) runs before the entry; init failure reports as a trap.
    for init in &program.init_fns {
        wrapper.push_str(&format!(
            "    if let Err(t) = generated::{init}() {{ println!(\"trap:{{}}\", t.name()); return; }}\n"
        ));
    }
    wrapper.push_str(&format!(
        "    let outcome: A2xResult<i32> = generated::{}();\n",
        program.entry
    ));
    // PLAN-968: the orderly scheduler drain runs after the user main
    // returns (pending mailboxes deliver, stop hooks run, threads join).
    if view.has_sched_ops() {
        wrapper.push_str("    a2x_runtime::a2x_sched_drain();\n");
    }
    wrapper.push_str("    match outcome {\n");
    wrapper.push_str("        Ok(v) => println!(\"value:{}\", v),\n");
    wrapper.push_str("        Err(t) => println!(\"trap:{}\", t.name()),\n");
    wrapper.push_str("    }\n}\n");

    let files = vec![
        ("main.rs".to_string(), wrapper),
        ("a2x_runtime.rs".to_string(), runtime),
        ("generated.rs".to_string(), generated),
    ];
    let manifest = build_family_manifest(
        schema,
        &files,
        view,
        input_hash,
        full_source_sha256,
        execution_config,
        entry_source_name,
        program,
        &target,
    );
    CollFileSet { files, manifest }
}

#[allow(clippy::too_many_arguments)]
fn build_family_manifest(
    schema: &str,
    files: &[(String, String)],
    view: &SemanticView<'_>,
    input_hash: &str,
    full_source_sha256: &str,
    execution_config: &str,
    entry_source_name: &str,
    program: &RustProgram,
    target: &TargetProfile,
) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&format!("  \"schema\": \"{}\",\n", schema));
    out.push_str(&format!("  \"source_profile\": \"{}\",\n", view.profile));
    out.push_str(&format!(
        "  \"hir_domain\": \"{}\",\n",
        view.domain.as_str()
    ));
    out.push_str(&format!("  \"input_hash\": \"{}\",\n", input_hash));
    out.push_str(&format!(
        "  \"source_sha256\": \"{}\",\n",
        full_source_sha256
    ));
    out.push_str(&format!("  \"execution_config\": {},\n", execution_config));
    out.push_str(&format!("  \"entry\": \"{}\",\n", entry_source_name));
    out.push_str(&format!("  \"entry_fn\": \"{}\",\n", program.entry));
    out.push_str(&format!(
        "  \"target\": {{ \"lang\": \"{}\", \"edition\": \"{}\", \"backend\": \"{}\", \"backend_version\": \"{}\", \"resource_runtime\": \"a2x-coll/1\" }},\n",
        target.lang, target.edition, target.backend, target.backend_version
    ));
    out.push_str("  \"files\": [\n");
    for (i, (path, content)) in files.iter().enumerate() {
        let comma = if i + 1 < files.len() { "," } else { "" };
        out.push_str(&format!(
            "    {{ \"path\": \"{}\", \"sha256\": \"{}\" }}{}\n",
            path,
            sha256_hex(content.as_bytes()),
            comma
        ));
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

const DYN_RT: &str = r#"
// ---- PLAN-1115 (T-03): the ordinary dynamic surface (PLAN-964) --------
//
// A2xDyn is the checked dynamic value. Objects are SHARED mutable maps
// and dynamic lists are SHARED element buffers (Rc handles - assignment,
// field reads and index reads ALIAS, exactly the AV handle semantics);
// explicit `.clone()` deep-copies. Typed (static) lists cross the Dyn
// boundary only through DynCast conversions, which check every element.

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum A2xKey {
    Text(String),
    Int(i32),
    Bool(bool),
}

pub type A2xDynList = std::rc::Rc<std::cell::RefCell<std::vec::Vec<A2xDyn>>>;
pub type A2xObject = std::rc::Rc<std::cell::RefCell<std::collections::BTreeMap<A2xKey, A2xDyn>>>;

pub enum A2xDyn {
    Nil,
    Int(i32),
    Bool(bool),
    Char(char),
    Str(A2xStr),
    List(A2xDynList),
    Object(A2xObject),
}

pub fn a2x_dyn_nil() -> A2xDyn { A2xDyn::Nil }
pub fn a2x_dyn_int(v: i32) -> A2xDyn { A2xDyn::Int(v) }
pub fn a2x_dyn_bool(v: bool) -> A2xDyn { A2xDyn::Bool(v) }
pub fn a2x_dyn_char(v: char) -> A2xDyn { A2xDyn::Char(v) }
pub fn a2x_dyn_str(v: A2xStr) -> A2xDyn { A2xDyn::Str(v) }
/// The READ-pack: an owned text operand packs as a COPY (AV value-read
/// semantics — the source local stays live; never moves).
pub fn a2x_dyn_str_share(v: &A2xStr) -> A2xResult<A2xDyn> { Ok(A2xDyn::Str(a2x_str_clone(v.as_str())?)) }
pub fn a2x_dyn_str_view(v: &str) -> A2xResult<A2xDyn> { Ok(A2xDyn::Str(a2x_str_clone(v)?)) }
pub fn a2x_dyn_list(v: A2xDynList) -> A2xDyn { A2xDyn::List(v) }
pub fn a2x_dyn_object(v: A2xObject) -> A2xDyn { A2xDyn::Object(v) }

pub fn a2x_dyn_is_nil(v: &A2xDyn) -> bool { matches!(v, A2xDyn::Nil) }

pub fn a2x_dyn_eq(a: &A2xDyn, b: &A2xDyn) -> bool {
    match (a, b) {
        (A2xDyn::Nil, A2xDyn::Nil) => true,
        (A2xDyn::Int(x), A2xDyn::Int(y)) => x == y,
        (A2xDyn::Bool(x), A2xDyn::Bool(y)) => x == y,
        (A2xDyn::Char(x), A2xDyn::Char(y)) => x == y,
        (A2xDyn::Str(x), A2xDyn::Str(y)) => a2x_str_eq(x.as_str(), y.as_str()),
        (A2xDyn::Object(x), A2xDyn::Object(y)) => std::rc::Rc::ptr_eq(x, y),
        (A2xDyn::List(x), A2xDyn::List(y)) => std::rc::Rc::ptr_eq(x, y),
        _ => false,
    }
}

fn a2x_dyn_key(v: &A2xDyn) -> A2xResult<A2xKey> {
    match v {
        A2xDyn::Int(i) => Ok(A2xKey::Int(*i)),
        A2xDyn::Bool(b) => Ok(A2xKey::Bool(*b)),
        A2xDyn::Str(s) => Ok(A2xKey::Text(s.as_str().to_string())),
        _ => Err(AutoTrap::DynType),
    }
}

pub fn a2x_dyn_as_int(v: &A2xDyn) -> A2xResult<i32> {
    match v { A2xDyn::Int(i) => Ok(*i), _ => Err(AutoTrap::DynType) }
}
pub fn a2x_dyn_as_bool(v: &A2xDyn) -> A2xResult<bool> {
    match v { A2xDyn::Bool(b) => Ok(*b), _ => Err(AutoTrap::DynType) }
}
pub fn a2x_dyn_as_char(v: &A2xDyn) -> A2xResult<char> {
    match v { A2xDyn::Char(c) => Ok(*c), _ => Err(AutoTrap::DynType) }
}
pub fn a2x_dyn_as_str(v: &A2xDyn) -> A2xResult<A2xStr> {
    match v { A2xDyn::Str(s) => a2x_str_clone(s.as_str()), _ => Err(AutoTrap::DynType) }
}
pub fn a2x_dyn_as_object(v: &A2xDyn) -> A2xResult<A2xObject> {
    match v { A2xDyn::Object(o) => Ok(std::rc::Rc::clone(o)), _ => Err(AutoTrap::DynType) }
}
pub fn a2x_dyn_as_list(v: &A2xDyn) -> A2xResult<A2xDynList> {
    match v { A2xDyn::List(l) => Ok(std::rc::Rc::clone(l)), _ => Err(AutoTrap::DynType) }
}

/// Pack a TYPED (static) dyn-element list into the shared dynamic list
/// (elements move; the source list is consumed).
pub fn a2x_dyn_pack_list(mut l: A2xList<A2xDyn>) -> A2xResult<A2xDyn> {
    let mut out: Vec<A2xDyn> = Vec::new();
    while a2x_coll_list_len(&l) > 0 {
        let n = a2x_coll_list_len(&l);
        let v = a2x_coll_list_remove(&mut l, n - 1)?;
        out.insert(0, v);
    }
    a2x_coll_list_free(&mut l);
    Ok(A2xDyn::List(std::rc::Rc::new(std::cell::RefCell::new(out))))
}

pub fn a2x_key_text(s: &str) -> A2xKey { A2xKey::Text(s.to_string()) }
pub fn a2x_key_int(i: i32) -> A2xKey { A2xKey::Int(i) }
pub fn a2x_key_bool(b: bool) -> A2xKey { A2xKey::Bool(b) }

pub fn a2x_obj_new() -> A2xObject {
    std::rc::Rc::new(std::cell::RefCell::new(std::collections::BTreeMap::new()))
}

/// ctor-time insert (entries cannot reference the object being built).
pub fn a2x_obj_init(obj: &A2xObject, key: A2xKey, value: A2xDyn) {
    obj.borrow_mut().insert(key, value);
}

/// `obj[k]` - runtime dispatch: a map keyed by the dynamic key; a list
/// indexed by an i32 key (out-of-range = ListIndex, the collections name).
pub fn a2x_dyn_get(base: &A2xDyn, key: &A2xDyn) -> A2xResult<A2xDyn> {
    match base {
        A2xDyn::Object(o) => {
            let k = a2x_dyn_key(key)?;
            let hit = o.borrow().get(&k).map(|v| a2x_dyn_share(v));
            match hit { Some(r) => r, None => Err(AutoTrap::ObjectMissing) }
        }
        A2xDyn::List(l) => {
            let A2xKey::Int(i) = a2x_dyn_key(key)? else { return Err(AutoTrap::DynType) };
            let buf = l.borrow();
            if i < 0 || i as usize >= buf.len() { return Err(AutoTrap::ListIndex) }
            a2x_dyn_share(&buf[i as usize])
        }
        _ => Err(AutoTrap::DynType),
    }
}

/// `obj[k] = v` - upsert (the old entry drops exactly once at commit);
/// a list element write by i32 key. Self-referential inserts trap
/// ObjectCycle (inserting a container into itself or into anything it
/// already reaches).
pub fn a2x_dyn_set(base: &A2xDyn, key: &A2xDyn, value: A2xDyn) -> A2xResult<()> {
    match base {
        A2xDyn::Object(o) => {
            let k = a2x_dyn_key(key)?;
            if matches!(&value, A2xDyn::Object(_))
                && a2x_dyn_reaches_object(&value, o, &mut std::vec::Vec::new())
            {
                return Err(AutoTrap::ObjectCycle)
            }
            o.borrow_mut().insert(k, value);
            Ok(())
        }
        A2xDyn::List(l) => {
            let A2xKey::Int(i) = a2x_dyn_key(key)? else { return Err(AutoTrap::DynType) };
            if matches!(&value, A2xDyn::List(_) | A2xDyn::Object(_))
                && a2x_dyn_reaches_list(&value, l, &mut std::vec::Vec::new())
            {
                return Err(AutoTrap::ObjectCycle)
            }
            let mut buf = l.borrow_mut();
            if i < 0 || i as usize >= buf.len() { return Err(AutoTrap::ListIndex) }
            buf[i as usize] = value;
            Ok(())
        }
        _ => Err(AutoTrap::DynType),
    }
}

/// Whether the container value `x` already reaches the shared list
/// `target` by Rc identity - the anti-cycle admission, list side.
fn a2x_dyn_reaches_list(x: &A2xDyn, target: &A2xDynList, seen: &mut Vec<*const ()>) -> bool {
    match x {
        A2xDyn::List(l) => {
            if std::rc::Rc::ptr_eq(l, target) { return true }
            let p = std::rc::Rc::as_ptr(l) as *const ();
            if seen.contains(&p) { return false }
            seen.push(p);
            let guard = l.borrow();
            guard.iter().any(|v| a2x_dyn_reaches_list(v, target, seen))
        }
        A2xDyn::Object(o) => {
            let p = std::rc::Rc::as_ptr(o) as *const ();
            if seen.contains(&p) { return false }
            seen.push(p);
            let guard = o.borrow();
            guard.values().any(|v| a2x_dyn_reaches_list(v, target, seen))
        }
        _ => false,
    }
}

/// Whether the container value `x` already contains the object `target`
/// by Rc identity - the anti-cycle admission for dynamic inserts.
fn a2x_dyn_reaches_object(x: &A2xDyn, target: &A2xObject, seen: &mut Vec<*const ()>) -> bool {
    match x {
        A2xDyn::Object(o) => {
            let p = std::rc::Rc::as_ptr(o) as *const ();
            if std::rc::Rc::ptr_eq(o, target) { return true }
            if seen.contains(&p) { return false }
            seen.push(p);
            let guard = o.borrow();
            guard.values().any(|v| a2x_dyn_reaches_object(v, target, seen))
        }
        A2xDyn::List(l) => {
            let guard = l.borrow();
            guard.iter().any(|v| a2x_dyn_reaches_object(v, target, seen))
        }
        _ => false,
    }
}

/// The value read out of a dynamic container: containers SHARE (handle
/// semantics); scalars/text copy.
pub fn a2x_dyn_share(v: &A2xDyn) -> A2xResult<A2xDyn> {
    Ok(match v {
        A2xDyn::Nil => A2xDyn::Nil,
        A2xDyn::Int(i) => A2xDyn::Int(*i),
        A2xDyn::Bool(b) => A2xDyn::Bool(*b),
        A2xDyn::Char(c) => A2xDyn::Char(*c),
        A2xDyn::Str(s) => A2xDyn::Str(a2x_str_clone(s.as_str())?),
        A2xDyn::List(l) => A2xDyn::List(std::rc::Rc::clone(l)),
        A2xDyn::Object(o) => A2xDyn::Object(std::rc::Rc::clone(o)),
    })
}

/// PLAN-1115 (T-05): the dynamic list's length (shared buffer read).
pub fn a2x_dyn_list_len(v: &A2xDyn) -> i32 {
    match v {
        A2xDyn::List(l) => l.borrow().len() as i32,
        _ => 0,
    }
}

/// PLAN-1115 (T-05): the deep copy of a dynamic value (explicit
/// `.clone()` — a NEW buffer/vec; Rc payloads still share per element
/// deep-clone semantics).
pub fn a2x_dyn_deep(v: &A2xDyn) -> A2xResult<A2xDyn> {
    use A2xDyn as D;
    Ok(match v {
        D::Nil => D::Nil,
        D::Int(i) => D::Int(*i),
        D::Bool(b) => D::Bool(*b),
        D::Char(c) => D::Char(*c),
        D::Str(s) => D::Str(a2x_str_clone(s.as_str())?),
        D::List(_) | D::Object(_) => v.a2x_elem_deep_clone()?,
    })
}

// ---- PLAN-1115 (T-05): the checked text-scan helpers (the assembled
// stdlib's text intrinsics; scalar indices throughout, -1 = absent). ----
pub fn a2x_str_find(s: &str, sub: &str, start: i32) -> A2xResult<i32> {
    let scalars: Vec<(usize, char)> = s.char_indices().collect();
    let total = scalars.len() as i32;
    if start < 0 || start > total { return Err(AutoTrap::TextIndex) }
    let byte_start = scalars.get(start as usize).map(|(b, _)| *b).unwrap_or(s.len());
    match s[byte_start..].find(sub) {
        Some(hit) => {
            let prefix = s[..byte_start + hit].chars().count() as i32;
            Ok(prefix)
        }
        None => Ok(-1),
    }
}
pub fn a2x_str_trim(s: &str) -> A2xResult<A2xStr> { a2x_str_clone(s.trim()) }
pub fn a2x_str_trim_len(s: &str) -> A2xResult<i32> { Ok(s.trim().chars().count() as i32) }
pub fn a2x_str_upper(s: &str) -> A2xResult<A2xStr> { a2x_str_clone(&s.to_uppercase()) }

// ---- PLAN-1115 (T-05): the widening conversion remainder (never trap;
// the frozen 961 matrix's language-lane completion — the shared RT_NUMERIC
// keeps its frozen bytes for the 902 channel). ----
pub fn a2x_c_u8_u32(a: u8) -> A2xResult<u32> { Ok(a as u32) }
pub fn a2x_c_u8_u64(a: u8) -> A2xResult<u64> { Ok(a as u64) }
pub fn a2x_c_u32_u64(a: u32) -> A2xResult<u64> { Ok(a as u64) }
pub fn a2x_c_i8_u8(a: i8) -> A2xResult<u8> { u8::try_from(a).map_err(|_| AutoTrap::Overflow) }
pub fn a2x_c_i8_u32(a: i8) -> A2xResult<u32> { i32::from(a).try_into().map_err(|_| AutoTrap::Overflow) }
pub fn a2x_c_i8_u64(a: i8) -> A2xResult<u64> { i64::from(a).try_into().map_err(|_| AutoTrap::Overflow) }

// ---- PLAN-1115 (T-03): the cross-frame language-error channel ----------
//
// `fail(msg)` stores its owned message in the frame's error slot and
// unwinds as the LangError trap; the enclosing protected region's catch
// arm takes it back (empty text when the slot is empty).

thread_local! {
    static A2X_LANG_ERR: std::cell::RefCell<Option<A2xStr>> = const { std::cell::RefCell::new(None) };
}

pub fn a2x_lang_fail<T>(msg: A2xStr) -> A2xResult<T> {
    A2X_LANG_ERR.with(|s| *s.borrow_mut() = Some(msg));
    Err(AutoTrap::LangError)
}

pub fn a2x_lang_take_err() -> A2xStr {
    A2X_LANG_ERR.with(|s| s.borrow_mut().take())
        .unwrap_or(A2xStr { buf: Some(String::new()) })
}

impl A2xCollElem for A2xDyn {
    fn a2x_elem_free(&mut self) { *self = A2xDyn::Nil }
    fn a2x_elem_deep_clone(&self) -> A2xResult<Self> {
        match self {
            A2xDyn::List(l) => {
                let mut out = std::vec::Vec::with_capacity(l.borrow().len());
                for e in l.borrow().iter() { out.push(e.a2x_elem_deep_clone()?) }
                Ok(A2xDyn::List(std::rc::Rc::new(std::cell::RefCell::new(out))))
            }
            other => a2x_dyn_share(other),
        }
    }
    fn a2x_elem_str_bytes(&self) -> u64 {
        match self { A2xDyn::Str(s) => s.as_str().len() as u64, _ => 0 }
    }
}
"#;

/// The typed collections runtime source: the frozen scalar core (same
/// helper names/shapes as the 902 core, with the collections trap set) +
/// the list/billing/cleanup layer. Byte-stable for a crate version.
pub fn collections_runtime_source(marks: &[&'static str]) -> String {
    coll_family_runtime(marks, COLL_TRAPS_FROZEN, "")
}

/// PLAN-1115 (T-03): the LANGUAGE channel runtime - the collections
/// substrate with the widened dynamic-surface trap set and the dynamic
/// value/object runtime appended.
pub fn language_runtime_source(marks: &[&'static str]) -> String {
    coll_family_runtime(marks, LANG_TRAPS, DYN_RT)
}

fn coll_family_runtime(marks: &[&'static str], traps: &str, extra: &str) -> String {
    let mut s = String::new();
    s.push_str("// AUTO-GENERATED typed collections runtime (PLAN-904 AX-C) — DO NOT EDIT.\n");
    s.push_str("#![allow(dead_code)]\n\n");
    s.push_str(&coll_core(traps));
    if !extra.is_empty() {
        s.push_str(extra);
    }
    for &mark in marks {
        let (suffix, value) = if mark == "hir.test.mark_a" {
            ("a", "1")
        } else {
            ("b", "2")
        };
        s.push_str(&format!(
            "\npub fn a2x_mark_{suf}() -> i32 {{\n    eprintln!(\"mark_{suf}\");\n    {val}\n}}\n",
            suf = suffix,
            val = value
        ));
    }
    // PLAN-972 (T-03): the numeric runtime helpers join the collections
    // family fileset too (the numeric surface — typed literals/binops/casts —
    // rides the collections family for the assembled stdlib; mirrors the
    // record-text family's RT_NUMERIC join).
    s.push_str(crate::rt::RT_NUMERIC);
    // PLAN-972 (T-03): the char expression helpers (scalar codepoints).
    s.push_str(
        "
pub fn a2x_char_to_codepoint(c: char) -> i32 { c as i32 }

pub fn a2x_char_from_codepoint(n: i32) -> A2xResult<char> {
    char::from_u32(n as u32).ok_or(AutoTrap::InvalidScalar)
}
",
    );
    // PLAN-972 (T-03): the scalar-indexed text expression helpers
    // (byte length, char access, slice view, owned copy).
    s.push_str(
        "
pub fn a2x_str_len_bytes(s: &str) -> A2xResult<i32> {
    i32::try_from(s.len()).map_err(|_| AutoTrap::Resource)
}

pub fn a2x_str_char_at(s: &str, i: i32) -> A2xResult<char> {
    if i < 0 {
        return Err(AutoTrap::TextIndex);
    }
    s.chars().nth(i as usize).ok_or(AutoTrap::TextIndex)
}

pub fn a2x_str_slice<'b>(s: &'b str, start: i32, end: i32) -> A2xResult<&'b str> {
    if start < 0 || end < start {
        return Err(AutoTrap::TextIndex);
    }
    let start = start as usize;
    let end = end as usize;
    let mut byte_start = None;
    let mut byte_end = None;
    for (i, (b, _)) in s.char_indices().enumerate() {
        if i == start {
            byte_start = Some(b);
        }
        if i == end {
            byte_end = Some(b);
        }
    }
    let byte_start = byte_start.ok_or(AutoTrap::TextIndex)?;
    let byte_end = match byte_end {
        Some(b) => b,
        None if start <= s.chars().count() && end == s.chars().count() => s.len(),
        None => return Err(AutoTrap::TextIndex),
    };
    s.get(byte_start..byte_end).ok_or(AutoTrap::TextIndex)
}

pub fn a2x_str_to_owned(s: &str) -> A2xResult<A2xStr> {
    a2x_str_clone(s)
}
",
    );
    s
}

// PLAN-1115 (T-03): the coll core splits around the trap enum so the
// LANGUAGE channel widens the trap set (ObjectMissing/DynType/ObjectCycle)
// while the 904 collections channel keeps its frozen runtime bytes.
const COLL_TRAP_HEAD: &str = r#"#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoTrap {
    Overflow,
    DivByZero,
    DivOverflow,
    RemByZero,
    RemOverflow,
    InvalidScalar,
    TextIndex,
    Resource,
    ListIndex,
    VariantBranch,
    Budget,
    Cancelled,
    LangError,
"#;
const COLL_TRAP_MID: &str = r#"
}

impl AutoTrap {
    pub fn name(self) -> &'static str {
        match self {
            AutoTrap::Overflow => "Overflow",
            AutoTrap::DivByZero => "DivByZero",
            AutoTrap::DivOverflow => "DivOverflow",
            AutoTrap::RemByZero => "RemByZero",
            AutoTrap::RemOverflow => "RemOverflow",
            AutoTrap::InvalidScalar => "InvalidScalar",
            AutoTrap::TextIndex => "TextIndex",
            AutoTrap::Resource => "Resource",
            AutoTrap::ListIndex => "ListIndex",
            AutoTrap::VariantBranch => "VariantBranch",
            AutoTrap::Budget => "Budget",
            AutoTrap::Cancelled => "Cancelled",
            AutoTrap::LangError => "LangError",
"#;
const COLL_TRAP_TAIL: &str = r#"
        }
    }
}
"#;
/// The 904 channel's frozen trap list (reconstructed byte-identically).
const COLL_TRAPS_FROZEN: &str = "";
/// PLAN-1115 (T-03): the language channel's dynamic-surface traps —
/// names align with the AV codes' suffixes (av.dyn.object-missing /
/// av.trap.dyn-type / av.dyn.object-cycle).
const LANG_TRAPS: &str = "    ObjectMissing,
    DynType,
    ObjectCycle,
";

fn coll_core(traps: &str) -> String {
    let mut names = String::new();
    if !traps.is_empty() {
        for name in ["ObjectMissing", "DynType", "ObjectCycle"] {
            names.push_str(&format!("            AutoTrap::{name} => \"{name}\",
"));
        }
    }
    format!("{COLL_TRAP_HEAD}{traps}{COLL_TRAP_MID}{names}{COLL_TRAP_TAIL}{COLL_REST}")
}

const COLL_REST: &str = r#"
pub type A2xResult<T> = Result<T, AutoTrap>;

pub fn a2x_add(a: i32, b: i32) -> A2xResult<i32> { a.checked_add(b).ok_or(AutoTrap::Overflow) }
pub fn a2x_sub(a: i32, b: i32) -> A2xResult<i32> { a.checked_sub(b).ok_or(AutoTrap::Overflow) }
pub fn a2x_mul(a: i32, b: i32) -> A2xResult<i32> { a.checked_mul(b).ok_or(AutoTrap::Overflow) }
pub fn a2x_neg(a: i32) -> A2xResult<i32> { a.checked_neg().ok_or(AutoTrap::Overflow) }
pub fn a2x_div(a: i32, b: i32) -> A2xResult<i32> {
    if b == 0 { Err(AutoTrap::DivByZero) } else { a.checked_div(b).ok_or(AutoTrap::DivOverflow) }
}
pub fn a2x_rem(a: i32, b: i32) -> A2xResult<i32> {
    if b == 0 { Err(AutoTrap::RemByZero) } else { a.checked_rem(b).ok_or(AutoTrap::RemOverflow) }
}

// ---- work budget / cooperative cancellation / fault hooks -------------
//
// Frozen billing table (PLAN-904 T-02): 2 WU per list element slot, 1 WU
// per 8 copied Str bytes, 1 WU per ctor, 4 WU per call frame, 1 WU per
// loop back-edge poll. Budget default 2^24; A2X_RT_BUDGET overrides;
// A2X_RT_CANCEL=1 arms cancellation (observed at polling points only).
// Cleanup paths never bill, never poll, never allocate.

use std::cell::RefCell;
use std::rc::Rc;

const DEFAULT_BUDGET: i64 = 16_777_216;

struct CollState {
    work_remaining: i64,
    cancelled: bool,
    fault_alloc: i64,
    alloc_count: i64,
    free_count: i64,
    live_nodes: i64,
    live_str_bytes: i64,
    live_backing_bytes: i64,
    events: bool,
}

thread_local! {
    static COLL: RefCell<CollState> = RefCell::new(CollState {
        work_remaining: 0,
        cancelled: false,
        fault_alloc: -1,
        alloc_count: -1,
        free_count: 0,
        live_nodes: 0,
        live_str_bytes: 0,
        live_backing_bytes: 0,
        events: false,
    });
}

fn coll_with<T>(f: impl FnOnce(&mut CollState) -> T) -> T {
    COLL.with(|c| f(&mut c.borrow_mut()))
}

fn coll_state() {
    // lazy init from the environment exactly once (the emitted file set is
    // identical in daily and certified runs; knobs are runtime-observed)
    coll_with(|s| {
        if s.alloc_count < 0 {
            s.alloc_count = 0;
            s.work_remaining = std::env::var("A2X_RT_BUDGET")
                .ok()
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(DEFAULT_BUDGET);
            s.cancelled = std::env::var("A2X_RT_CANCEL").ok().as_deref() == Some("1");
            s.fault_alloc = std::env::var("A2X_RT_FAULT_ALLOC")
                .ok()
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(0);
            s.events = std::env::var("A2X_RT_EVENTS").ok().as_deref() == Some("1");
        }
    });
}

/// Deduct work units at a real work site; exhaustion is the typed Budget
/// trap. Charged BEFORE the allocation/commit it pays for.
fn coll_charge(units: i64, why: &str) -> A2xResult<()> {
    coll_state();
    coll_with(|s| {
        s.work_remaining -= units;
        if s.events {
            eprintln!("coll:charge:{}:{}", units, why);
        }
        if s.work_remaining < 0 {
            return Err(AutoTrap::Budget);
        }
        Ok(())
    })
}

/// Language-error exit (`fail` / uncaught `.?`): the generated target carries
/// the error CLASS only — the message/span channel does not cross A2X.
pub fn a2x_coll_fail<T>() -> A2xResult<T> {
    Err(AutoTrap::LangError)
}

/// Cooperative cancellation point (loop back-edges; 1 WU each).
pub fn a2x_coll_poll() -> A2xResult<()> {
    coll_state();
    coll_with(|s| {
        s.work_remaining -= 1;
        if s.events {
            eprintln!("coll:poll");
        }
        if s.work_remaining < 0 {
            return Err(AutoTrap::Budget);
        }
        if s.cancelled {
            return Err(AutoTrap::Cancelled);
        }
        Ok(())
    })
}

/// Call-frame charge (4 WU; callee prologue).
pub fn a2x_coll_charge_call() -> A2xResult<()> {
    coll_charge(4, "call")
}

/// Record/variant construction charge (1 WU).
pub fn a2x_coll_charge_ctor() -> A2xResult<()> {
    coll_charge(1, "ctor")
}

/// Str copy charge (1 WU per 8 bytes, rounded up).
fn coll_charge_str_bytes(bytes: usize) -> A2xResult<()> {
    let units = ((bytes as i64) + 7) / 8;
    coll_charge(units.max(1), "str-bytes")
}

/// The n-th allocation fault gate; fires BEFORE the real request.
fn coll_alloc_gate(bytes: usize, what: &str) -> A2xResult<()> {
    coll_state();
    coll_with(|s| {
        s.alloc_count += 1;
        if s.fault_alloc > 0 && s.alloc_count == s.fault_alloc {
            if s.events {
                eprintln!("coll:fault:{}:{}", what, bytes);
            }
            return Err(AutoTrap::Resource);
        }
        if s.events {
            eprintln!("coll:alloc:{}:{}", what, bytes);
        }
        s.live_backing_bytes += bytes as i64;
        Ok(())
    })
}

fn coll_alloc_release(bytes: usize) {
    coll_with(|s| {
        s.free_count += 1;
        s.live_backing_bytes -= bytes as i64;
        if s.events {
            eprintln!("coll:free-backing:{}", bytes);
        }
    });
}

/// Live accounting observers (test observation; the language never reads
/// them — helper probes verify cleanup by these counters, not by logs).
pub fn a2x_coll_live_nodes() -> i64 {
    coll_with(|s| s.live_nodes)
}
pub fn a2x_coll_live_str_bytes() -> i64 {
    coll_with(|s| s.live_str_bytes)
}
pub fn a2x_coll_live_backing_bytes() -> i64 {
    coll_with(|s| s.live_backing_bytes)
}
pub fn a2x_coll_alloc_count() -> i64 {
    coll_state();
    coll_with(|s| s.alloc_count)
}
pub fn a2x_coll_free_count() -> i64 {
    coll_with(|s| s.free_count)
}

// ---- owned Str layer (same contract as the 902 runtime) ----------------

pub struct A2xStr {
    buf: Option<String>,
}

impl A2xStr {
    pub fn as_str(&self) -> &str {
        self.buf.as_deref().unwrap_or("")
    }
}

fn alloc_buffer(cap: usize, fill: &str) -> A2xResult<A2xStr> {
    let mut b = String::new();
    if b.try_reserve(cap).is_err() {
        return Err(AutoTrap::Resource);
    }
    coll_charge_str_bytes(cap)?;
    coll_alloc_gate(cap, "str")?;
    coll_with(|s| s.live_str_bytes += cap as i64);
    b.push_str(fill);
    Ok(A2xStr { buf: Some(b) })
}

pub fn a2x_str_from_lit(s: &str) -> A2xResult<A2xStr> {
    alloc_buffer(s.len(), s)
}
pub fn a2x_str_from_char(c: char) -> A2xResult<A2xStr> {
    let mut buf = [0u8; 4];
    let s = c.encode_utf8(&mut buf);
    alloc_buffer(s.len(), s)
}
pub fn a2x_str_clone(s: &str) -> A2xResult<A2xStr> {
    alloc_buffer(s.len(), s)
}
pub fn a2x_str_display_i32(value: i32) -> A2xResult<A2xStr> {
    let rendered = format!("{}", value);
    a2x_str_from_lit(&rendered)
}
pub fn a2x_str_concat(l: &str, r: &str) -> A2xResult<A2xStr> {
    let cap = l.len().checked_add(r.len()).ok_or(AutoTrap::Resource)?;
    let mut b = String::new();
    if b.try_reserve(cap).is_err() {
        return Err(AutoTrap::Resource);
    }
    coll_charge_str_bytes(cap)?;
    coll_alloc_gate(cap, "str")?;
    coll_with(|s| s.live_str_bytes += cap as i64);
    b.push_str(l);
    b.push_str(r);
    Ok(A2xStr { buf: Some(b) })
}

pub fn a2x_str_free(s: &mut A2xStr) {
    // zero-charge, zero-poll, zero-allocation release path
    if let Some(b) = s.buf.take() {
        coll_with(|st| {
            st.live_str_bytes -= b.len() as i64;
        });
    }
}

pub fn a2x_str_len(s: &str) -> A2xResult<i32> {
    i32::try_from(s.chars().count()).map_err(|_| AutoTrap::Resource)
}
pub fn a2x_str_eq(l: &str, r: &str) -> bool {
    l == r
}
pub fn a2x_str_ne(l: &str, r: &str) -> bool {
    l != r
}

impl A2xCollElem for A2xStr {
    fn a2x_elem_free(&mut self) {
        a2x_str_free(self)
    }
    fn a2x_elem_deep_clone(&self) -> A2xResult<Self> {
        a2x_str_clone(self.as_str())
    }
    fn a2x_elem_str_bytes(&self) -> u64 {
        self.as_str().len() as u64
    }
}

// ---- list element contract ----------------------------------------------

pub trait A2xCollElem: Sized {
    /// Explicit release (zero-charge/zero-poll/zero-allocation).
    fn a2x_elem_free(&mut self);
    /// Deep clone with billing + partial-output rollback at the helper.
    fn a2x_elem_deep_clone(&self) -> A2xResult<Self>;
    /// Owned Str payload bytes inside (live accounting).
    fn a2x_elem_str_bytes(&self) -> u64;
}

impl A2xCollElem for i32 {
    fn a2x_elem_free(&mut self) {}
    fn a2x_elem_deep_clone(&self) -> A2xResult<Self> {
        a2x_coll_charge_ctor()?;
        Ok(*self)
    }
    fn a2x_elem_str_bytes(&self) -> u64 {
        0
    }
}

impl A2xCollElem for bool {
    fn a2x_elem_free(&mut self) {}
    fn a2x_elem_deep_clone(&self) -> A2xResult<Self> {
        a2x_coll_charge_ctor()?;
        Ok(*self)
    }
    fn a2x_elem_str_bytes(&self) -> u64 {
        0
    }
}

// ---- typed list ----------------------------------------------------------

pub struct A2xList<E> {
    buf: std::mem::ManuallyDrop<Vec<E>>,
}

const LIST_START_CAP: usize = 4;
pub const LIST_MAX_ELEMS: usize = 65536;

fn list_cap_bytes<E>(cap: usize) -> usize {
    cap.saturating_mul(std::mem::size_of::<E>())
}

impl<E: A2xCollElem> A2xList<E> {
    fn reserve_exact_cap(&mut self, want: usize) -> A2xResult<()> {
        if self.buf.len() + want <= self.buf.capacity() {
            return Ok(());
        }
        let mut new_cap = if self.buf.capacity() == 0 {
            LIST_START_CAP.max(want)
        } else {
            self.buf.capacity()
        };
        while new_cap < self.buf.len() + want {
            new_cap = new_cap.checked_mul(2).ok_or(AutoTrap::Resource)?;
        }
        if new_cap > LIST_MAX_ELEMS {
            return Err(AutoTrap::ListIndex);
        }
        // bill BEFORE the request; fault BEFORE the request
        coll_charge(2 * (new_cap - self.buf.capacity()) as i64, "list-slots")?;
        let old_cap = self.buf.capacity();
        let add_bytes = list_cap_bytes::<E>(new_cap).saturating_sub(list_cap_bytes::<E>(old_cap));
        coll_alloc_gate(add_bytes, "list-grow")?;
        if self.buf.try_reserve_exact(new_cap - old_cap).is_err() {
            return Err(AutoTrap::Resource);
        }
        Ok(())
    }
}

pub fn a2x_coll_list_new<E: A2xCollElem>() -> A2xResult<A2xList<E>> {
    coll_state();
    let mut v: Vec<E> = Vec::new();
    // the ledger shows exactly one buffer allocation per .new(): the base
    // capacity request (billed 2 WU/slot, fault-gated before the request)
    coll_charge(2 * LIST_START_CAP as i64, "list-new")?;
    let bytes = list_cap_bytes::<E>(LIST_START_CAP);
    coll_alloc_gate(bytes, "list-new")?;
    if v.try_reserve_exact(LIST_START_CAP).is_err() {
        return Err(AutoTrap::Resource);
    }
    Ok(A2xList {
        buf: std::mem::ManuallyDrop::new(v),
    })
}

pub fn a2x_coll_list_len<E>(l: &A2xList<E>) -> i32 {
    l.buf.len() as i32
}

pub fn a2x_coll_list_push<E: A2xCollElem>(l: &mut A2xList<E>, v: E) -> A2xResult<()> {
    // precheck: capacity first (no mutation), then commit
    let need = l.buf.len() + 1;
    if need > LIST_MAX_ELEMS {
        let mut v = v;
        v.a2x_elem_free();
        return Err(AutoTrap::ListIndex);
    }
    // growth (if any) bills + faults BEFORE the element work
    if l.buf.len() + 1 > l.buf.capacity() {
        l.reserve_exact_cap(1)?;
    }
    // element slot work
    if let Err(e) = coll_charge(2, "list-push") {
        let mut v = v;
        v.a2x_elem_free();
        return Err(e);
    }
    coll_with(|s| s.live_nodes += 1);
    l.buf.push(v);
    if coll_with(|s| s.events) {
        eprintln!("coll:list_push:{}", l.buf.len());
    }
    Ok(())
}

pub fn a2x_coll_list_insert<E: A2xCollElem>(
    l: &mut A2xList<E>,
    index: i32,
    v: E,
) -> A2xResult<()> {
    let idx = usize::try_from(index).map_err(|_| AutoTrap::ListIndex)?;
    if idx > l.buf.len() {
        let mut v = v;
        v.a2x_elem_free();
        return Err(AutoTrap::ListIndex);
    }
    if l.buf.len() + 1 > l.buf.capacity() {
        l.reserve_exact_cap(1)?;
    }
    if let Err(e) = coll_charge(2, "list-insert") {
        let mut v = v;
        v.a2x_elem_free();
        return Err(e);
    }
    coll_with(|s| s.live_nodes += 1);
    l.buf.insert(idx, v);
    Ok(())
}

pub fn a2x_coll_list_get<E: A2xCollElem + Copy>(l: &A2xList<E>, index: i32) -> A2xResult<E> {
    let idx = usize::try_from(index).map_err(|_| AutoTrap::ListIndex)?;
    l.buf.get(idx).copied().ok_or(AutoTrap::ListIndex)
}

/// Checked read-only view of one owned element (never materializes; the
/// index check traps ListIndex BEFORE any access).
pub fn a2x_coll_list_borrow_elem<'a, E>(
    l: &'a A2xList<E>,
    index: i32,
) -> A2xResult<&'a E> {
    let idx = usize::try_from(index).map_err(|_| AutoTrap::ListIndex)?;
    l.buf.get(idx).ok_or(AutoTrap::ListIndex)
}

pub fn a2x_coll_list_set<E: A2xCollElem>(l: &mut A2xList<E>, index: i32, v: E) -> A2xResult<()> {
    let idx = usize::try_from(index).map_err(|_| AutoTrap::ListIndex)?;
    if idx >= l.buf.len() {
        let mut v = v;
        v.a2x_elem_free();
        return Err(AutoTrap::ListIndex);
    }
    if let Err(e) = coll_charge(2, "list-set") {
        let mut v = v;
        v.a2x_elem_free();
        return Err(e);
    }
    // commit: the old element frees exactly once at commit
    let mut old = std::mem::replace(&mut l.buf[idx], v);
    old.a2x_elem_free();
    Ok(())
}

pub fn a2x_coll_list_remove<E: A2xCollElem>(l: &mut A2xList<E>, index: i32) -> A2xResult<E> {
    let idx = usize::try_from(index).map_err(|_| AutoTrap::ListIndex)?;
    if idx >= l.buf.len() {
        return Err(AutoTrap::ListIndex);
    }
    coll_with(|s| s.live_nodes -= 1);
    Ok(l.buf.remove(idx))
}

/// Tail-delete clear: elements release in REVERSE index order; capacity
/// stays (no reallocation); zero-charge/zero-poll.
pub fn a2x_coll_list_clear<E: A2xCollElem>(l: &mut A2xList<E>) {
    while let Some(mut e) = l.buf.pop() {
        e.a2x_elem_free();
        coll_with(|s| s.live_nodes -= 1);
    }
}

/// The ONLY list release path: elements in reverse index order, buffer
/// last. Zero-charge/zero-poll/zero-allocation.
///
/// PLAN-1036 (F-R1-03): release is IDEMPOTENT — the frozen cleanup rule
/// (`checked-resource-facts` "list free 后 data=NULL/cap=0"; the C runtime
/// already guards on `data == NULL`) is load-bearing because the back-edge
/// poll's error path deliberately RE-LISTS the per-iteration deaths and the
/// loop survivors. Taking the buffer out first makes a second call a true
/// no-op: no second drop (heap corruption), no second release event.
pub fn a2x_coll_list_free<E: A2xCollElem>(l: &mut A2xList<E>) {
    let mut taken = std::mem::replace(&mut l.buf, std::mem::ManuallyDrop::new(Vec::new()));
    if taken.capacity() == 0 {
        // already released (or never allocated): nothing to free, no event
        return;
    }
    while let Some(mut e) = taken.pop() {
        e.a2x_elem_free();
        coll_with(|s| s.live_nodes -= 1);
    }
    let bytes = list_cap_bytes::<E>(taken.capacity());
    unsafe { std::mem::ManuallyDrop::drop(&mut taken) };
    coll_alloc_release(bytes);
}

pub fn a2x_coll_list_clone<E: A2xCollElem>(src: &A2xList<E>) -> A2xResult<A2xList<E>> {
    let mut out = a2x_coll_list_new::<E>()?;
    for elem in src.buf.iter() {
        let cloned = match elem.a2x_elem_deep_clone() {
            Ok(v) => v,
            Err(e) => {
                a2x_coll_list_free(&mut out);
                return Err(e);
            }
        };
        if let Err(e) = a2x_coll_list_push(&mut out, cloned) {
            a2x_coll_list_free(&mut out);
            return Err(e);
        }
    }
    Ok(out)
}

impl<E: A2xCollElem> A2xCollElem for A2xList<E> {
    fn a2x_elem_free(&mut self) {
        a2x_coll_list_free(self);
    }
    fn a2x_elem_deep_clone(&self) -> A2xResult<Self> {
        a2x_coll_list_clone(self)
    }
    fn a2x_elem_str_bytes(&self) -> u64 {
        self.buf.iter().map(|e| e.a2x_elem_str_bytes()).sum()
    }
}

// ---- deep-link witness chain (helper probes; iterative, no host
// recursion — proves the cleanup/fault mechanism scales past host stack
// limits under a small thread stack) --------------------------------------

pub struct A2xDeepLink {
    payload: Option<Box<A2xStr>>,
    next: Option<Box<A2xDeepLink>>,
}

pub fn a2x_coll_deep_push(head: &mut Option<Box<A2xDeepLink>>, s: &str) -> A2xResult<()> {
    let node = A2xDeepLink {
        payload: Some(Box::new(a2x_str_from_lit(s)?)),
        next: head.take(),
    };
    *head = Some(Box::new(node));
    Ok(())
}

/// Iterative free of a chain of ANY length (explicit work loop, no
/// recursion — allocation-free).
pub fn a2x_coll_deep_free(head: &mut Option<Box<A2xDeepLink>>) {
    while let Some(mut node) = head.take() {
        if let Some(mut p) = node.payload.take() {
            a2x_str_free(&mut p);
        }
        *head = node.next.take();
    }
}

/// Iterative deep clone with billing and rollback: the n-th payload
/// failure frees the partially cloned prefix.
pub fn a2x_coll_deep_clone(
    head: &Option<Box<A2xDeepLink>>,
) -> A2xResult<Option<Box<A2xDeepLink>>> {
    let mut out: Option<Box<A2xDeepLink>> = None;
    let mut cur = head.as_deref();
    while let Some(node) = cur {
        let payload = match &node.payload {
            Some(p) => Some(Box::new(a2x_str_clone(p.as_str()).map_err(|e| {
                a2x_coll_deep_free(&mut out);
                e
            })?)),
            None => None,
        };
        let next_node = A2xDeepLink {
            payload,
            next: out.take(),
        };
        out = Some(Box::new(next_node));
        cur = node.next.as_deref();
    }
    Ok(out)
}

pub fn a2x_coll_deep_len(head: &Option<Box<A2xDeepLink>>) -> i32 {
    let mut n = 0i32;
    let mut cur = head.as_deref();
    while let Some(node) = cur {
        n += 1;
        cur = node.next.as_deref();
    }
    n
}

// marker: Rc/RefCell kept for the thread-local state cell only
#[allow(dead_code)]
fn _rc_marker(_: Option<Rc<()>>) {}
"#;
