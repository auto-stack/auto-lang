//! PLAN-921 T-03: the Rust [`TargetProvider`] — the byte-identical
//! relocation of everything the a2x-store session used to statically own
//! (fragment key target fields, fragment production, runtime source
//! selection, record/variant prelude inputs and the whole file-set
//! assembly).
//!
//! Identity rules (frozen): every emitted key/manifest byte this provider
//! produces equals the pre-921 session output — the AXR1/2/3 key field
//! order (lang/edition/backend/backend_version/runtime_sha256/
//! print_version) is reproduced exactly, the file-set schemas and the
//! wrapper/manifest format strings are copied verbatim. The existing Rust
//! corpus (store_cache/incremental_cache/902/904 suites) is the
//! byte-identity oracle for this relocation.

use crate::coll;
use crate::lower::{
    lower_functions, lower_records, rust_fn_name, rust_method_name, RustRecord, RustVariant,
};
use crate::print;
use crate::rt;
use crate::target::TargetProfile;
use ace_a2x_core::target::{Family, FileSet, FilesetRequest, TargetDescriptor, TargetProvider};
use ace_a2x_core::{A2xReject, FnFacts, FnKind, SemanticView, Ty};

/// The Rust target provider (stateless value type). Serves ALL three
/// families (scalar/record-text/collections).
#[derive(Clone, Copy, Debug, Default)]
pub struct RustTargetProvider;

/// The print-pass version bound into Rust fragment keys. PLAN-931 bumped
/// 1→2: the scoped consumer-anchored view-temp frees, verified external
/// modes and MoveOwn arg transfer change the printed fragment bytes, so
/// every pre-931 Rust fragment (all three families share this axis) must
/// miss once and re-lower — never serve the old premature-free shape.
/// PLAN-941 F-941-05 bumps 2→3: field writes now mark value roots mutable.
/// A cached pre-fix fragment must miss even when its semantic facts match.
pub const RUST_PRINT_VERSION: &str = "a2x-print/3";

impl RustTargetProvider {
    pub fn v1() -> Self {
        RustTargetProvider
    }

    /// The lowered Rust function name of `f` (free functions and
    /// owner-qualified methods alike).
    fn fragment_name(f: &FnFacts<'_>) -> String {
        match f.kind {
            FnKind::Free => rust_fn_name(f.source_name),
            FnKind::Method { .. } => rust_method_name(f.source_name),
        }
    }
}

impl TargetProvider for RustTargetProvider {
    fn descriptor(&self) -> TargetDescriptor {
        let t = TargetProfile::v1();
        TargetDescriptor {
            lang: t.lang,
            standard: t.edition,
            backend: t.backend,
            backend_version: t.backend_version,
        }
    }

    fn fragment_key_inputs(&self, runtime_sha256: &str) -> Vec<(String, String)> {
        // EXACTLY the frozen AXR1/2/3 key tail (order included) —
        // byte-identical keys for existing Rust sessions.
        let t = TargetProfile::v1();
        vec![
            ("lang".into(), t.lang.into()),
            ("edition".into(), t.edition.into()),
            ("backend".into(), t.backend.into()),
            ("backend_version".into(), t.backend_version.into()),
            ("runtime_sha256".into(), runtime_sha256.into()),
            ("print_version".into(), RUST_PRINT_VERSION.into()),
        ]
    }

    fn fragment_layer_id(&self, family: Family) -> Result<&'static str, A2xReject> {
        match family {
            Family::Scalar => Ok("AXR1"),
            Family::RecordText => Ok("AXR2"),
            Family::Collections => Ok("AXR3"),
        }
    }

    fn runtime_source(
        &self,
        view: &SemanticView<'_>,
        _family: Family,
    ) -> Result<String, A2xReject> {
        if view.foreign_declarations().next().is_some() {
            return Err(A2xReject::new(
                "a2r.foreign-unsupported",
                "Rust target does not execute source foreign calls",
            ));
        }
        let marks: Vec<&'static str> = view.trace_symbols().into_iter().collect();
        Ok(match view.domain {
            ace_core::hir::HirDomain::Text
            | ace_core::hir::HirDomain::Record
            | ace_core::hir::HirDomain::TextRecord => rt::record_text_runtime_source(&marks),
            ace_core::hir::HirDomain::Collections => coll::collections_runtime_source(&marks),
            _ => crate::artifacts::runtime_source(&marks),
        })
    }

    fn produce_fragments(
        &self,
        view: &SemanticView<'_>,
        fns: &[&FnFacts<'_>],
        family: Family,
    ) -> Result<Vec<String>, A2xReject> {
        if fns.is_empty() {
            return Ok(Vec::new());
        }
        match family {
            Family::Collections => {
                // one whole-view lowering per batch (the frozen cold/warm
                // collections behavior: lower_coll_functions over the view)
                let program = crate::lower_coll::lower_coll_functions(view)?;
                let mut out = Vec::with_capacity(fns.len());
                for f in fns {
                    let name = Self::fragment_name(f);
                    let rust_fn = program
                        .functions
                        .iter()
                        .find(|rf| rf.name == name)
                        .cloned()
                        .ok_or_else(|| {
                            A2xReject::new(
                                "a2x.internal",
                                format!("fragment for `{}` missing after lowering", f.source_name),
                            )
                        })?;
                    out.push(print::print_coll_fn(
                        &program.records,
                        &program.variants,
                        &rust_fn,
                    ));
                }
                Ok(out)
            }
            Family::Scalar | Family::RecordText => {
                // ONE whole-view lowering memoized for the whole batch (the
                // frozen scalar/rt behavior — per-function lowering would
                // re-lower the module O(n) times at load scale)
                let program = lower_functions(view)?;
                let mut out = Vec::with_capacity(fns.len());
                for f in fns {
                    let name = Self::fragment_name(f);
                    let rust_fn = program
                        .functions
                        .iter()
                        .find(|rf| rf.name == name)
                        .cloned()
                        .ok_or_else(|| {
                            A2xReject::new(
                                "a2x.internal",
                                format!("fragment for `{}` missing after lowering", f.source_name),
                            )
                        })?;
                    out.push(print_fragment_text(&program.records, &rust_fn));
                }
                Ok(out)
            }
        }
    }

    fn capture_prelude_seed(
        &self,
        view: &SemanticView<'_>,
        family: Family,
    ) -> Result<Option<String>, A2xReject> {
        match family {
            Family::Scalar => Ok(None),
            Family::RecordText => {
                let records = lower_records(view)?;
                Ok(Some(encode_seed(&records, &[])))
            }
            Family::Collections => {
                let records = lower_records(view)?;
                let variants = crate::lower_coll::lower_coll_variants(view);
                Ok(Some(encode_seed(&records, &variants)))
            }
        }
    }

    fn assemble_fileset(&self, req: FilesetRequest<'_>) -> Result<FileSet, A2xReject> {
        let FilesetRequest {
            entry,
            input_hash,
            family,
            fragments,
            view,
            runtime,
            prelude_seed,
            origins,
            source_sha256,
        } = req;
        let mut generated = String::new();
        generated.push_str(&format!(
            "// AUTO-GENERATED by auto-a2x-rust (PLAN-892 AX-A) — session assembly, entry `{}`\n\n",
            entry
        ));
        generated.push_str("use crate::a2x_runtime::*;\n\n");
        if family == Family::RecordText {
            let (records, _variants) = decode_seed(prelude_seed).ok_or_else(|| {
                A2xReject::new(
                    "a2x.internal",
                    "record-text assembly needs its record table",
                )
            })?;
            let texts: Vec<&str> = fragments.iter().map(|r| r.text).collect();
            generated.push_str(&print::print_rt_prelude_from(&records, &texts));
        }
        if family == Family::Collections {
            let (records, variants) = decode_seed(prelude_seed).ok_or_else(|| {
                A2xReject::new("a2x.internal", "collections assembly needs its type tables")
            })?;
            generated.push_str(&print::print_coll_prelude_from(&records, &variants));
        }
        for r in fragments {
            generated.push_str(r.text);
            generated.push('\n');
        }
        if !fragments.iter().any(|r| r.source_name == entry) {
            return Err(A2xReject::entry("entry missing"));
        }
        let entry_rust = rust_fn_name(entry);
        let wrapper = format!(
            "// AUTO-GENERATED (PLAN-892 AX-A) — DO NOT EDIT.\nmod a2x_runtime;\nmod generated;\n\nuse a2x_runtime::A2xResult;\n\nfn main() {{\n    let outcome: A2xResult<i32> = generated::{}();\n    match outcome {{\n        Ok(v) => println!(\"value:{{}}\", v),\n        Err(t) => println!(\"trap:{{}}\", t.name()),\n    }}\n}}\n",
            entry_rust
        );
        let files = vec![
            ("main.rs".to_string(), wrapper),
            ("a2x_runtime.rs".to_string(), runtime.to_string()),
            ("generated.rs".to_string(), generated),
        ];
        let schema = match family {
            Family::Scalar => "auto.a2x.rust.fileset/1",
            Family::RecordText => "auto.a2x.rust.record-text.fileset/1",
            Family::Collections => coll::COLL_FILESET_SCHEMA,
        };
        let mut manifest = String::new();
        manifest.push_str(&format!("{{\n  \"schema\": \"{}\",\n", schema));
        manifest.push_str(&format!("  \"input_hash\": \"{}\",\n", input_hash));
        if family == Family::Collections {
            // PLAN-904: the manifest binds the REPLACED FULL SOURCE hash
            // (never a fragment-only hash) and the execution-config identity
            let sha = source_sha256.unwrap_or_default();
            manifest.push_str(&format!("  \"source_sha256\": \"{}\",\n", sha));
            manifest.push_str(&format!(
                "  \"execution_config\": {},\n",
                coll::canonical_execution_config("debug", 16_777_216, None, false, false)
            ));
        }
        manifest.push_str(&format!(
            "  \"entry\": \"{}\",\n  \"functions\": [\n",
            entry
        ));
        for (i, r) in fragments.iter().enumerate() {
            let comma = if i + 1 < fragments.len() { "," } else { "" };
            // origins are included whenever the assembly carries its view
            // (the cold path); warm assemblies list declarations only —
            // frozen per-family behavior
            // PLAN-921: view origins first (cold); the anchor's origin
            // map second (pure restore); declaration-only otherwise
            let origin_opt = if let Some(v) = view {
                Some(v.fn_by_def(r.def_index).map(|f| f.origin).unwrap_or((0, 0)))
            } else {
                origins
                    .and_then(|m| m.iter().find(|(d, _)| *d == r.def_index))
                    .map(|o| o.1)
            };
            if let Some(origin) = origin_opt {
                manifest.push_str(&format!(
                    "    {{ \"decl\": \"{}\", \"sem_fp\": \"{}\", \"origin\": [{}, {}] }}{}\n",
                    r.source_name, r.sem_fp, origin.0, origin.1, comma
                ));
            } else {
                manifest.push_str(&format!(
                    "    {{ \"decl\": \"{}\", \"sem_fp\": \"{}\" }}{}\n",
                    r.source_name, r.sem_fp, comma
                ));
            }
        }
        manifest.push_str("  ],\n  \"files\": [\n");
        for (i, (path, content)) in files.iter().enumerate() {
            let comma = if i + 1 < files.len() { "," } else { "" };
            manifest.push_str(&format!(
                "    {{ \"path\": \"{}\", \"sha256\": \"{}\" }}{}\n",
                path,
                ace_a2x_core::sha256::sha256_hex(content.as_bytes()),
                comma
            ));
        }
        manifest.push_str("  ]\n}\n");
        Ok(FileSet { files, manifest })
    }
}

/// Print one fragment: the record-text family renders against the module
/// record table (nominal names); the scalar family keeps the frozen path.
fn print_fragment_text(records: &[RustRecord], rust_fn: &crate::lower::RustFn) -> String {
    if records.is_empty() {
        print::print_fn(rust_fn)
    } else {
        print::print_fn_records(records, rust_fn)
    }
}

// ---------------------------------------------------------------------------
// The prelude seed codec — a provider-internal, line-based, lossless
// encoding of the record/variant tables the warm path cannot re-derive
// (its fragment view is dead by assembly time). Deterministic, no deps.
//
// Grammar (one record/variant per line, fields in declaration order):
//   R <type_index> <name> <resource 0|1> <field_count>
//   F <field_name> <ty>
//   V <type_index> <name> <arm_count>
//   A <arm_name> <ty|->            (`-` = no payload)
// <ty> ∈ i32|bool|char|str|strowned|unit|rec:<n>|list:<n>|var:<n>
// ---------------------------------------------------------------------------

fn encode_ty(ty: &Ty) -> String {
    match ty {
        Ty::I32 => "i32".into(),
        // PLAN-961: numeric wire words (additive to the frozen tag set).
        Ty::I8 => "i8".into(),
        Ty::U8 => "u8".into(),
        Ty::U32 => "u32".into(),
        Ty::I64 => "i64".into(),
        Ty::U64 => "u64".into(),
        Ty::F32 => "f32".into(),
        Ty::F64 => "f64".into(),
        Ty::Bool => "bool".into(),
        Ty::Char => "char".into(),
        Ty::Str => "str".into(),
        Ty::StrOwned => "strowned".into(),
        Ty::Unit => "unit".into(),
        Ty::Record(n) => format!("rec:{}", n),
        Ty::List(n) => format!("list:{}", n),
        Ty::Variant(n) => format!("var:{}", n),
    }
}

fn decode_ty(tok: &str) -> Option<Ty> {
    Some(match tok {
        "i32" => Ty::I32,
        "bool" => Ty::Bool,
        "char" => Ty::Char,
        "str" => Ty::Str,
        "strowned" => Ty::StrOwned,
        "unit" => Ty::Unit,
        other => {
            let (kind, n) = other.split_once(':')?;
            let n: u32 = n.parse().ok()?;
            match kind {
                "rec" => Ty::Record(n),
                "list" => Ty::List(n),
                "var" => Ty::Variant(n),
                _ => return None,
            }
        }
    })
}

fn encode_seed(records: &[RustRecord], variants: &[RustVariant]) -> String {
    let mut s = String::new();
    for r in records {
        s.push_str(&format!(
            "R {} {} {} {}\n",
            r.type_index,
            r.name,
            if r.resource { 1 } else { 0 },
            r.fields.len()
        ));
        for (fname, fty) in &r.fields {
            s.push_str(&format!("F {} {}\n", fname, encode_ty(fty)));
        }
    }
    for v in variants {
        s.push_str(&format!("V {} {} {}\n", v.type_index, v.name, v.arms.len()));
        for (aname, aty) in &v.arms {
            match aty {
                Some(t) => s.push_str(&format!("A {} {}\n", aname, encode_ty(t))),
                None => s.push_str(&format!("A {} -\n", aname)),
            }
        }
    }
    s
}

fn decode_seed(seed: Option<&str>) -> Option<(Vec<RustRecord>, Vec<RustVariant>)> {
    let seed = seed?;
    let mut records = Vec::new();
    let mut variants = Vec::new();
    let mut lines = seed.lines();
    while let Some(line) = lines.next() {
        let toks: Vec<&str> = line.split(' ').collect();
        match toks.first().copied() {
            Some("R") => {
                if toks.len() != 5 {
                    return None;
                }
                let type_index: u32 = toks[1].parse().ok()?;
                let field_count: usize = toks[4].parse().ok()?;
                let mut fields = Vec::with_capacity(field_count);
                for _ in 0..field_count {
                    let fline = lines.next()?;
                    let ftoks: Vec<&str> = fline.split(' ').collect();
                    if ftoks.len() != 3 || ftoks[0] != "F" {
                        return None;
                    }
                    fields.push((ftoks[1].to_string(), decode_ty(ftoks[2])?));
                }
                records.push(RustRecord {
                    type_index,
                    name: toks[2].to_string(),
                    fields,
                    resource: toks[3] == "1",
                });
            }
            Some("V") => {
                if toks.len() != 4 {
                    return None;
                }
                let type_index: u32 = toks[1].parse().ok()?;
                let arm_count: usize = toks[3].parse().ok()?;
                let mut arms = Vec::with_capacity(arm_count);
                for _ in 0..arm_count {
                    let aline = lines.next()?;
                    let atoks: Vec<&str> = aline.split(' ').collect();
                    if atoks.len() != 3 || atoks[0] != "A" {
                        return None;
                    }
                    let payload = if atoks[2] == "-" {
                        None
                    } else {
                        Some(decode_ty(atoks[2])?)
                    };
                    arms.push((atoks[1].to_string(), payload));
                }
                variants.push(RustVariant {
                    type_index,
                    name: toks[2].to_string(),
                    arms,
                });
            }
            _ => return None,
        }
    }
    Some((records, variants))
}
