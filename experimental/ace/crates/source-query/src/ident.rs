//! Logical identity of the SQ-A layer (PLAN-934 §5.2/§5.5).
//!
//! Three axes are kept strictly apart:
//!
//! * **Logical identity** — package + real module + owner name. Stable
//!   across restarts, unrelated to any directory instance, content hash
//!   or span. The 928 `UnitCatalog` content hash is source/directory
//!   AUTHORIZATION (provenance); it never becomes a semantic key.
//! * **Semantic version per axis** — a function's signature version and
//!   its body version bump independently: a body-only edit bumps only the
//!   body axis, so signature readers (callers) stay warm; a signature or
//!   parameter-mode edit bumps the signature axis and invalidates the
//!   bodies that actually read it.
//! * **Query capability** — profile / algorithm / schema / producer
//!   identity, all new and separate from the 796 `ac796/*` domain (a new
//!   key that silently reused the old payload/naming would be a forged
//!   hit).
//!
//! Domain separation reuses the contract crate's discipline
//! (length-prefixed blake3 over producer ‖ domain ‖ parts) but with the
//! SQ-A producer, so an `ac934/*` id can never collide with an
//! `ac796/*` id even for identical parts.

use ace_core::source::ast::{FnItem, ParamMode, TypeAnn, TypeRef, TypeWord};
use ace_semantics_contract::{ContentVersion, LogicalId};
use blake3::Hasher;

/// Producer identity of the ordinary-source query layer. Separate from
/// the 796 probe (`ac796.sema-probe/1`) and from every backend producer.
pub const SQ_PRODUCER: &str = "plan934/source-sq";

/// Page/envelope schema of the new SQ-A domain (also participates in
/// every query key: a schema change must miss, never reinterpret old
/// bytes).
pub const SQ_SCHEMA: &str = "ac934/source-sq/v1";

/// Limited ordinary-source profile (the consumer side of the same profile
/// name lives in the core port).
pub const SQ_PROFILE: &str = "source-sq-scalar-i32/1";

/// The REUSED solver algorithm identity (796's `ac796-bidir-worklist/1`).
/// Changing the algorithm must invalidate every infer-body fact; the
/// adapter extension of PLAN-934 is a different *input builder*, not a
/// second solver.
pub const SQ_ALGO: &str = "ac796-bidir-worklist/1";

/// Hash domain of SQ-A logical declaration ids.
pub const LOGICAL_DOMAIN: &str = "ac934/logical/v1";
/// Hash domain of one function's signature version.
pub const SIG_DOMAIN: &str = "ac934/fn-sig/v1";
/// Hash domain of one function body's structural version.
pub const BODY_DOMAIN: &str = "ac934/fn-body/v1";

/// Domain-separated content hash over `SQ_PRODUCER ‖ domain ‖ parts`.
pub fn sq_hash(domain: &str, parts: &[&[u8]]) -> String {
    let mut h = Hasher::new();
    h.update(SQ_PRODUCER.as_bytes());
    h.update(&[0]);
    h.update(domain.as_bytes());
    h.update(&[0]);
    for p in parts {
        h.update(&(p.len() as u64).to_le_bytes());
        h.update(p);
    }
    h.finalize().to_hex().to_string()
}

/// Mint the SQ-A content version of already-canonicalised bytes.
pub fn content_version(domain: &str, canonical: &[u8]) -> ContentVersion {
    ContentVersion(sq_hash(domain, &[canonical]))
}

/// Stable logical id of a free function (`package`, real module, `fn`,
/// name). Never derived from a source hash, span, ordinal or intern slot.
pub fn owner_logical(package: &str, module: &str, name: &str) -> LogicalId {
    LogicalId(sq_hash(
        LOGICAL_DOMAIN,
        &[
            package.as_bytes(),
            module.as_bytes(),
            b"fn",
            name.as_bytes(),
        ],
    ))
}

/// Canonical, span-free encoding of one type annotation. Record names and
/// generic arguments keep their NAMES (two records with the same shape
/// but different names are different types); positions never enter.
pub fn type_canonical(t: &TypeAnn, out: &mut Vec<u8>) {
    match &t.kind {
        TypeRef::Builtin(w) => {
            out.extend_from_slice(b"builtin:");
            out.extend_from_slice(builtin_word(w));
        }
        TypeRef::Record(id) => {
            out.extend_from_slice(b"record:");
            out.extend_from_slice(id.name.as_bytes());
        }
        // PLAN-935: a template type parameter keeps its name (two distinct
        // type parameters are distinct types); concrete instance pages carry
        // only substituted builtins — a TyVar inside a query key means the
        // key was built from an un-concretized template.
        TypeRef::TyVar(id) => {
            out.extend_from_slice(b"tyvar:");
            out.extend_from_slice(id.name.as_bytes());
        }
        TypeRef::Optional(inner) => {
            out.extend_from_slice(b"optional:");
            type_canonical(inner, out);
        }
        TypeRef::Generic { name, args } => {
            out.extend_from_slice(b"generic:");
            out.extend_from_slice(name.name.as_bytes());
            out.push(b'(');
            for a in args {
                type_canonical(a, out);
                out.push(b',');
            }
            out.push(b')');
        }
        TypeRef::Function { params, result } => {
            out.extend_from_slice(b"fn(");
            for p in params {
                type_canonical(p, out);
                out.push(b',');
            }
            out.extend_from_slice(b")->");
            type_canonical(result, out);
        }
    }
    out.push(0);
}

fn builtin_word(w: &TypeWord) -> &'static [u8] {
    match w {
        TypeWord::Int | TypeWord::I32 => b"i32",
        TypeWord::Bool => b"bool",
        TypeWord::Str => b"str",
        TypeWord::StrOwned => b"Str",
        TypeWord::Char => b"char",
        TypeWord::Void => b"void",
    }
}

fn mode_tag(m: ParamMode) -> &'static [u8] {
    match m {
        ParamMode::Default => b"default",
        ParamMode::Mut => b"mut",
        ParamMode::Move => b"move",
    }
}

/// The SIGNATURE axis: name + parameter names/modes/types + result type.
/// It deliberately excludes the body: a body-only edit must keep this
/// version, so signature readers stay warm (V-03).
pub fn signature_version(f: &FnItem) -> ContentVersion {
    let mut c = Vec::new();
    c.extend_from_slice(b"fn:");
    c.extend_from_slice(f.name.name.as_bytes());
    c.push(b'(');
    for p in &f.params {
        c.extend_from_slice(p.name.name.as_bytes());
        c.push(b':');
        c.extend_from_slice(mode_tag(p.mode));
        c.push(b':');
        type_canonical(&p.ty, &mut c);
        c.push(b',');
    }
    c.extend_from_slice(b")->");
    type_canonical(&f.ret, &mut c);
    content_version(SIG_DOMAIN, &c)
}
