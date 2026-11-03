//! PLAN-921 T-06: the source-bound anchor codec (layer AXP1).
//!
//! One anchor artifact per published assembly state binds the FULL
//! SOURCE (by sha256) to its per-function source slices (byte ranges +
//! slice hashes + semantic fingerprints + signatures), the target
//! identity (backend + runtime content hash) and the ASSEMBLY RECEIPT
//! (every emitted file's sha256). A new-process pure restore is then:
//! look the anchor up by the caller's source sha, decode STRICTLY,
//! re-verify the source binding slice by slice, re-fetch and re-verify
//! the stored semantic/fragment payloads, re-assemble and re-verify the
//! file-set bytes against the receipt — with ZERO production (P0) and
//! real verification work (R>0). Nothing in the store is trusted blind.
//!
//! Wire format: line-based, tab-separated, escaped string fields
//! (`\\` and `\t` escaped; identifiers/hashes are ASCII anyway) —
//! deterministic, strictly parseable, no dependencies. Unknown lines or
//! trailing garbage are decode failures (strict-by-construction).

/// The anchor schema id (also the LAYER_SOURCE_ANCHOR schema).
pub const ANCHOR_SCHEMA: &str = "a2921/axp-source-anchor/v1";
/// PLAN-925: the RECORD-TEXT anchor schema (layer AXP2) — the scalar
/// shape plus the provider-opaque prelude seed the blind re-assembly
/// needs (the record table). Byte-disjoint from v1: a v1 anchor never
/// decodes as an RT one or vice versa.
pub const ANCHOR_SCHEMA_RT: &str = "a2925/axp-rt-source-anchor/v1";
/// PLAN-926: the COLLECTIONS anchor schema (layer AXP3) — the scalar
/// shape plus the provider-opaque prelude seed the blind re-assembly
/// needs (the nominal type table: records + variants + list
/// instantiations). Byte-disjoint from v1/RT: a foreign anchor never
/// decodes as a collections one or vice versa.
pub const ANCHOR_SCHEMA_COLL: &str = "a2926/axp-coll-source-anchor/v1";

/// One function's source binding inside an anchor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnchorFn {
    /// Source name (stable across whole-module and single-body bundles).
    pub decl: String,
    /// Module def-table index at the anchored assembly (ordering only).
    pub def_index: u32,
    /// Byte span of the declaration in the bound full source.
    pub origin: (u32, u32),
    /// sha256 of the source slice `src[origin.0..origin.1]`.
    pub slice_sha256: String,
    /// The function's semantic fingerprint (shared, target-neutral).
    pub sem_fp: String,
    /// The canonical signature (semantic-layer key input at production).
    pub signature: String,
}

/// A decoded anchor (strict).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Anchor {
    pub schema: String,
    pub owner: String,
    pub profile: String,
    pub backend: String,
    pub entry: String,
    pub source_sha256: String,
    pub input_hash: String,
    pub hir_schema: String,
    /// Capability set at production (comma-joined in the key).
    pub caps: Vec<String>,
    /// Content hash of the runtime the fragments were keyed with.
    pub runtime_sha256: String,
    /// The runtime TEXT itself (restore re-verifies it by the hash — it
    /// has no view to re-derive marks from).
    pub runtime_text: String,
    pub functions: Vec<AnchorFn>,
    /// The assembly receipt: (path, sha256) of every emitted file.
    pub fileset: Vec<(String, String)>,
    /// PLAN-925: the provider-opaque prelude seed (record table) of the
    /// RT schema; `None` on the scalar schema (encode/decode enforce the
    /// pairing strictly).
    pub prelude_seed: Option<String>,
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
}

fn unesc(s: &str) -> Option<String> {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars();
    while let Some(c) = it.next() {
        match c {
            '\\' => match it.next()? {
                '\\' => out.push('\\'),
                't' => out.push('\t'),
                'n' => out.push('\n'),
                _ => return None,
            },
            '\t' => return None, // unescaped tab = field split
            other => out.push(other),
        }
    }
    Some(out)
}

impl Anchor {
    /// Byte-stable encoding (strict decode round-trips it).
    pub fn encode(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "A\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            esc(&self.schema),
            esc(&self.owner),
            esc(&self.profile),
            esc(&self.backend),
            esc(&self.entry),
            &self.source_sha256,
            &self.input_hash,
            &self.hir_schema,
            esc(&self.caps.join(","))
        ));
        s.push_str(&format!("R\t{}\n", &self.runtime_sha256));
        s.push_str(&format!("F\t{}\n", self.functions.len()));
        for f in &self.functions {
            s.push_str(&format!(
                "f\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                esc(&f.decl),
                f.def_index,
                f.origin.0,
                f.origin.1,
                &f.slice_sha256,
                &f.sem_fp,
                esc(&f.signature)
            ));
        }
        s.push_str(&format!("T\t{}\n", esc(&self.runtime_text)));
        if self.schema == ANCHOR_SCHEMA_RT || self.schema == ANCHOR_SCHEMA_COLL {
            // the RT/COLL schemas REQUIRE the seed line (an absent seed
            // is an internal wiring error, never an empty table — an
            // empty table still carries its schema line)
            s.push_str(&format!(
                "S\t{}\n",
                esc(self.prelude_seed.as_deref().unwrap_or(""))
            ));
        }
        s.push_str(&format!("W\t{}\n", self.fileset.len()));
        for (path, sha) in &self.fileset {
            s.push_str(&format!("w\t{}\t{}\n", esc(path), sha));
        }
        s
    }

    /// STRICT decode: the schema line must match, every line must parse,
    /// counts must line up, and nothing may trail. `None` on any
    /// deviation — the caller turns that into a named rejection.
    pub fn decode(text: &str) -> Option<Anchor> {
        let mut lines = text.lines();
        // header: A + 9 fields (escaped strings stay one field — real tabs
        // are escaped, so a raw tab split is field-safe)
        let header = lines.next()?;
        let h: Vec<&str> = header.split('\t').collect();
        if h.len() != 10 || h[0] != "A" {
            return None;
        }
        let schema = h[1].to_string();
        if schema != ANCHOR_SCHEMA && schema != ANCHOR_SCHEMA_RT && schema != ANCHOR_SCHEMA_COLL {
            return None;
        }
        let owner = unesc(h[2])?;
        let profile = unesc(h[3])?;
        let backend = unesc(h[4])?;
        let entry = unesc(h[5])?;
        let source_sha256 = sha64(h[6])?;
        let input_hash = h[7].to_string();
        let hir_schema = unesc(h[8])?;
        let caps: Vec<String> = match h[9] {
            "" => Vec::new(),
            joined => joined.split(',').map(|c| c.to_string()).collect(),
        };

        let rline = lines.next()?;
        let mut r = rline.split('\t');
        if r.next()? != "R" {
            return None;
        }
        let runtime_sha256 = sha64(r.next()?)?;
        if r.next().is_some() {
            return None;
        }

        let fline = lines.next()?;
        let mut f = fline.split('\t');
        if f.next()? != "F" {
            return None;
        }
        let fcount: usize = f.next()?.parse().ok()?;
        if f.next().is_some() {
            return None;
        }
        let mut functions = Vec::with_capacity(fcount);
        for _ in 0..fcount {
            let l = lines.next()?;
            let mut p = l.split('\t');
            if p.next()? != "f" {
                return None;
            }
            let decl = unesc(p.next()?)?;
            let def_index: u32 = p.next()?.parse().ok()?;
            let o0: u32 = p.next()?.parse().ok()?;
            let o1: u32 = p.next()?.parse().ok()?;
            let slice_sha256 = sha64(p.next()?)?;
            let sem_fp = sha64(p.next()?)?;
            let signature = unesc(p.next()?)?;
            if p.next().is_some() {
                return None;
            }
            if o0 > o1 {
                return None;
            }
            functions.push(AnchorFn {
                decl,
                def_index,
                origin: (o0, o1),
                slice_sha256,
                sem_fp,
                signature,
            });
        }

        let tline = lines.next()?;
        let mut t = tline.split('\t');
        if t.next()? != "T" {
            return None;
        }
        let runtime_text = unesc(t.next()?)?;
        if t.next().is_some() {
            return None;
        }

        // PLAN-925/926: the RT/COLL schemas carry exactly one seed line
        // here; the scalar schema carries none (strict pairing both ways)
        let prelude_seed = if schema == ANCHOR_SCHEMA_RT || schema == ANCHOR_SCHEMA_COLL {
            let sline = lines.next()?;
            let mut sp = sline.split('\t');
            if sp.next()? != "S" {
                return None;
            }
            let seed = unesc(sp.next()?)?;
            if sp.next().is_some() || seed.is_empty() {
                return None;
            }
            Some(seed)
        } else {
            None
        };

        let wline = lines.next()?;
        let mut w = wline.split('\t');
        if w.next()? != "W" {
            return None;
        }
        let wcount: usize = w.next()?.parse().ok()?;
        if w.next().is_some() {
            return None;
        }
        let mut fileset = Vec::with_capacity(wcount);
        for _ in 0..wcount {
            let l = lines.next()?;
            let mut p = l.split('\t');
            if p.next()? != "w" {
                return None;
            }
            let path = unesc(p.next()?)?;
            let sha = sha64(p.next()?)?;
            if p.next().is_some() {
                return None;
            }
            fileset.push((path, sha));
        }
        if lines.next().is_some() {
            return None; // strict: nothing may trail
        }
        Some(Anchor {
            schema,
            owner,
            profile,
            backend,
            entry,
            source_sha256,
            input_hash,
            hir_schema,
            caps,
            runtime_sha256,
            runtime_text,
            functions,
            fileset,
            prelude_seed,
        })
    }
}

fn sha64(s: &str) -> Option<String> {
    if s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()) {
        Some(s.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Anchor {
        Anchor {
            schema: ANCHOR_SCHEMA.to_string(),
            owner: "probe".into(),
            profile: "scalar".into(),
            backend: "auto-a2x-c".into(),
            entry: "main".into(),
            source_sha256: "a".repeat(64),
            input_hash: "deadbeef".into(),
            hir_schema: "hir/1".into(),
            caps: vec!["trace.hir.test".into()],
            runtime_sha256: "b".repeat(64),
            runtime_text: "// runtime
line2
"
            .into(),
            prelude_seed: None,
            functions: vec![AnchorFn {
                decl: "main".into(),
                def_index: 0,
                origin: (0, 34),
                slice_sha256: "c".repeat(64),
                sem_fp: "d".repeat(64),
                signature: "fn main() -> i32".into(),
            }],
            fileset: vec![("main.c".into(), "e".repeat(64))],
        }
    }

    #[test]
    fn roundtrip_is_lossless_and_strict() {
        let a = sample();
        let text = a.encode();
        assert_eq!(Anchor::decode(&text), Some(a.clone()));
        // trailing garbage / tampered fields / bad schema all reject
        assert_eq!(Anchor::decode(&format!("{text}X")), None);
        let bad = a
            .encode()
            .replace(ANCHOR_SCHEMA, "a2921/axp-source-anchor/v2");
        assert_eq!(Anchor::decode(&bad), None);
        assert_eq!(Anchor::decode(""), None);
    }
}
