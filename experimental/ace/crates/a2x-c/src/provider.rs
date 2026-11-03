//! PLAN-921 CA-A + PLAN-925 CA-B + PLAN-926 CA-C: the C
//! [`TargetProvider`] — HIR → C11 for the scalar channel (AXC1,
//! PLAN-921), the record-text channel (AXC2, PLAN-925), and the
//! collections channel (AXC3, PLAN-926); every other family is a NAMED
//! rejection, never a fallback.
//!
//! Frozen identities: backend `auto-a2x-c`, standard `c11`. Scalar:
//! fragment layer `AXC1`, fileset schema `auto.a2x.c.fileset/1`, print
//! pass `a2x-c-print/1`, recipe `ac-a2c/hir-scalar-emit-v1`. Record-text:
//! fragment layer `AXC2`, fileset schema `auto.a2x.c.record-text.fileset/1`,
//! recipe `ac-a2c/hir-record-text-emit-v1`, prelude seed `a2xc-rt-seed/1`
//! (the record table, provider-opaque). Collections: fragment layer
//! `AXC3`, fileset schema `auto.a2x.c.collections.fileset/1`, recipe
//! `ac-a2c/hir-collections-emit-v1`, prelude seed `a2xc-coll-seed/1`
//! (the nominal type table — records + variants + list instantiations).
//! The scalar file set is `main.c` + `a2x_c_runtime.h` + `generated.c` +
//! `manifest.json`; the record-text one adds the seed-derived record
//! table (structs + per-record helpers) to `generated.c` and ships the
//! typed resource runtime header; the collections one ships the
//! collections runtime header and the seed-derived nominal type table
//! (records + variants + typed list helpers).

use crate::coll_runtime;
use crate::coll_table;
use crate::lower::{self, c_fn_name, lower_fn_c, print_fragment};
use crate::lower_coll;
use crate::lower_rt;
use crate::rt_runtime;
use crate::runtime;
use crate::seed;
use crate::seed_coll;
use ace_a2x_core::sha256::sha256_hex;
use ace_a2x_core::target::{Family, FileSet, FilesetRequest, TargetDescriptor, TargetProvider};
use ace_a2x_core::{A2xReject, FnFacts};

/// The C backend version constant (cert/manifest identity source).
pub const C_BACKEND_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The C print-pass version (fragment key input).
pub const C_PRINT_VERSION: &str = "a2x-c-print/1";
/// The C language standard bound into target identities.
pub const C_STANDARD: &str = "c11";
/// The C scalar channel's file-set schema.
pub const C_FILESET_SCHEMA: &str = "auto.a2x.c.fileset/1";
/// The C scalar channel's recipe identity (declared here; wired into the
/// worker protocol and certification by PLAN-921 T-09/T-10).
pub const C_RECIPE: &str = "ac-a2c/hir-scalar-emit-v1";

/// The C record-text channel's file-set schema (PLAN-925 CA-B). Fragment
/// keys distinguish the channels by layer (AXC1/AXC2) + runtime hash (the
/// scalar and record-text runtime texts differ).
pub const C_RT_FILESET_SCHEMA: &str = "auto.a2x.c.record-text.fileset/1";
/// The C record-text channel's recipe identity (PLAN-925 CA-B).
pub const C_RT_RECIPE: &str = "ac-a2c/hir-record-text-emit-v1";

/// The C collections channel's file-set schema (PLAN-926 CA-C).
pub const C_COLL_FILESET_SCHEMA: &str = "auto.a2x.c.collections.fileset/1";
/// The C collections channel's recipe identity (PLAN-926 CA-C).
pub const C_COLL_RECIPE: &str = "ac-a2c/hir-collections-emit-v1";

/// The C target provider (stateless value type). Serves the SCALAR family
/// only — record-text/collections are named rejections (CA-B/CA-C are
/// successor packages; no silent widening).
#[derive(Clone, Copy, Debug, Default)]
pub struct CTargetProvider;

impl CTargetProvider {
    pub fn v1() -> Self {
        CTargetProvider
    }
}

impl TargetProvider for CTargetProvider {
    fn descriptor(&self) -> TargetDescriptor {
        TargetDescriptor {
            lang: "c",
            standard: C_STANDARD,
            backend: "auto-a2x-c",
            backend_version: env!("CARGO_PKG_VERSION"),
        }
    }

    fn fragment_key_inputs(&self, runtime_sha256: &str) -> Vec<(String, String)> {
        // Both C channels share the target identity inputs; the print
        // version is per-channel through the store's family-dispatched
        // pass (see [ace_a2x_store] pass_version_of).
        vec![
            ("lang".into(), "c".into()),
            ("standard".into(), C_STANDARD.into()),
            ("backend".into(), "auto-a2x-c".into()),
            ("backend_version".into(), env!("CARGO_PKG_VERSION").into()),
            ("runtime_sha256".into(), runtime_sha256.into()),
            ("print_version".into(), C_PRINT_VERSION.into()),
        ]
    }

    fn fragment_layer_id(&self, family: Family) -> Result<&'static str, A2xReject> {
        match family {
            Family::Scalar => Ok("AXC1"),
            Family::RecordText => Ok("AXC2"),
            Family::Collections => Ok("AXC3"),
        }
    }

    fn runtime_source(
        &self,
        view: &ace_a2x_core::SemanticView<'_>,
        family: Family,
    ) -> Result<String, A2xReject> {
        let marks: Vec<&'static str> = view.trace_symbols().into_iter().collect();
        match family {
            Family::Scalar => Ok(runtime::runtime_source(&marks)),
            Family::RecordText => Ok(rt_runtime::record_text_runtime_source(&marks)),
            Family::Collections => Ok(coll_runtime::collections_runtime_source(&marks)),
        }
    }

    fn produce_fragments(
        &self,
        view: &ace_a2x_core::SemanticView<'_>,
        fns: &[&FnFacts<'_>],
        family: Family,
    ) -> Result<Vec<String>, A2xReject> {
        match family {
            Family::Scalar => {
                let mut out = Vec::with_capacity(fns.len());
                for f in fns {
                    let cfn = lower_fn_c(view, f)?;
                    out.push(print_fragment(&cfn));
                }
                Ok(out)
            }
            Family::RecordText => {
                let cfns = lower_rt::lower_functions_rt(view, fns)?;
                let mut out = Vec::with_capacity(cfns.len());
                for cfn in &cfns {
                    out.push(lower_rt::print_fragment_rt(cfn));
                }
                Ok(out)
            }
            Family::Collections => {
                let mut out = Vec::with_capacity(fns.len());
                for f in fns {
                    let cfn = lower_coll::lower_coll_fn_c(view, f)?;
                    out.push(lower_coll::print_fragment_coll(&cfn));
                }
                Ok(out)
            }
        }
    }

    fn capture_prelude_seed(
        &self,
        view: &ace_a2x_core::SemanticView<'_>,
        family: Family,
    ) -> Result<Option<String>, A2xReject> {
        match family {
            // the scalar channel needs no module-level prelude inputs
            Family::Scalar => Ok(None),
            // the record-text channel's generated.c needs the record
            // table (structs + helpers) — captured while the fragment
            // view is alive, fed back on the pure-restore re-assembly
            Family::RecordText => seed::encode_seed(view).map(Some),
            // the collections channel's generated.c needs the nominal
            // type table (records + variants + list instantiations)
            Family::Collections => seed_coll::encode_seed_coll(view).map(Some),
        }
    }

    fn assemble_fileset(&self, req: FilesetRequest<'_>) -> Result<FileSet, A2xReject> {
        match req.family {
            Family::Scalar => self.assemble_scalar_fileset(req),
            Family::RecordText => assemble_record_text_fileset(req),
            Family::Collections => assemble_collections_fileset(req),
        }
    }
}

impl CTargetProvider {
    /// The scalar channel's 3-file set (PLAN-921 CA-A, unchanged shape).
    fn assemble_scalar_fileset(&self, req: FilesetRequest<'_>) -> Result<FileSet, A2xReject> {
        let FilesetRequest {
            entry,
            input_hash,
            family: _,
            fragments,
            view,
            runtime,
            origins,
            ..
        } = req;
        // generated.c: header comment + runtime include + the
        // forward-declaration block (each fragment's prototype line,
        // collected in fragment order) + the definitions
        let mut generated = String::new();
        generated.push_str(&format!(
            "// AUTO-GENERATED by auto-a2x-c (PLAN-921 CA-A) — session assembly, entry `{}`\n\n",
            entry
        ));
        generated.push_str("#include \"a2x_c_runtime.h\"\n\n");
        for r in fragments {
            let proto = prototype_line(r.text).ok_or_else(|| {
                A2xReject::new(
                    "a2x.internal",
                    format!("fragment `{}` carries no prototype line", r.source_name),
                )
            })?;
            generated.push_str(proto);
            generated.push('\n');
        }
        generated.push('\n');
        for r in fragments {
            generated.push_str(r.text);
            generated.push('\n');
        }
        if !fragments.iter().any(|r| r.source_name == entry) {
            return Err(A2xReject::entry("entry missing"));
        }
        let entry_c = c_fn_name(entry);
        let wrapper = format!(
            "// AUTO-GENERATED (PLAN-921 CA-A) — DO NOT EDIT.\n#include <fcntl.h>\n#include <inttypes.h>\n#include <io.h>\n#include <stdint.h>\n#include <stdio.h>\n#include \"a2x_c_runtime.h\"\n\na2x_result {entry}(void);\n\nint main(void) {{\n    _setmode(_fileno(stdout), _O_BINARY);\n    _setmode(_fileno(stderr), _O_BINARY);\n    a2x_result outcome = {entry}();\n    if (outcome.trap == A2X_TRAP_NONE) {{\n        printf(\"value:%\" PRId32 \"\\n\", outcome.value);\n    }} else {{\n        printf(\"trap:%s\\n\", a2x_trap_name(outcome.trap));\n    }}\n    return 0;\n}}\n",
            entry = entry_c
        );
        let files = vec![
            ("main.c".to_string(), wrapper),
            ("a2x_c_runtime.h".to_string(), runtime.to_string()),
            ("generated.c".to_string(), generated),
        ];
        let mut manifest = String::new();
        manifest.push_str(&format!("{{\n  \"schema\": \"{}\",\n", C_FILESET_SCHEMA));
        manifest.push_str(&format!("  \"input_hash\": \"{}\",\n", input_hash));
        manifest.push_str(&format!(
            "  \"entry\": \"{}\",\n  \"functions\": [\n",
            entry
        ));
        for (i, r) in fragments.iter().enumerate() {
            let comma = if i + 1 < fragments.len() { "," } else { "" };
            // origins are included whenever the assembly carries its view
            // (the cold path); warm assemblies list declarations only —
            // mirroring the frozen Rust manifest behavior
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
                sha256_hex(content.as_bytes()),
                comma
            ));
        }
        manifest.push_str("  ]\n}\n");
        Ok(FileSet { files, manifest })
    }
}

/// The fragment's prototype line (its second line — the shape contract of
/// [`crate::lower::print_fragment`] / [`crate::lower_rt::print_fragment_rt`]).
fn prototype_line(fragment_text: &str) -> Option<&str> {
    fragment_text.lines().nth(1).filter(|l| l.ends_with(';'))
}

/// The C value type of a seed field type (the record table's
/// representation truth: text fields own their buffer).
fn seed_field_c_ty(ty: &seed::SeedFieldTy) -> String {
    match ty {
        seed::SeedFieldTy::I32 => "int32_t".into(),
        seed::SeedFieldTy::Bool => "bool".into(),
        seed::SeedFieldTy::Char => "uint32_t".into(),
        seed::SeedFieldTy::Str | seed::SeedFieldTy::StrOwned => "a2x_str".into(),
        seed::SeedFieldTy::Record(n) => lower_rt::c_record_name(n),
    }
}

/// The record-table block of `generated.c`: struct definitions, result
/// wrappers, value/fwd/clone/free helpers (prototypes first, then
/// definitions — nested-record clone helpers call each other).
fn record_table_block(recs: &[seed::SeedRecord]) -> Result<String, A2xReject> {
    // every record name must be unique (the verifier's nominal identity;
    // a duplicate in a seed is corruption)
    let mut seen = std::collections::BTreeSet::new();
    for r in recs {
        if !seen.insert(r.name.as_str()) {
            return Err(A2xReject::new(
                "a2x.seed",
                format!("duplicate record name `{}` in the record table", r.name),
            ));
        }
    }
    // field record references must name a table member
    for r in recs {
        for (_, ty) in &r.fields {
            if let seed::SeedFieldTy::Record(n) = ty {
                if !seen.contains(n.as_str()) {
                    return Err(A2xReject::new(
                        "a2x.seed",
                        format!(
                            "record `{}` field references unknown record `{}`",
                            r.name, n
                        ),
                    ));
                }
            }
        }
    }
    let mut out = String::new();
    // 1) struct definitions (seed order == declaration order, which is
    //    topological for the non-recursive nesting domain)
    for r in recs {
        out.push_str("typedef struct {\n");
        for (name, ty) in &r.fields {
            out.push_str(&format!(
                "    {} f_{};\n",
                seed_field_c_ty(ty),
                lower::sanitize(name)
            ));
        }
        out.push_str(&format!("}} {};\n\n", lower_rt::c_record_name(&r.name)));
    }
    // 2) result wrappers
    for r in recs {
        out.push_str(&format!(
            "typedef struct {{ {} value; a2x_trap_code trap; }} a2x_res_{};\n",
            lower_rt::c_record_name(&r.name),
            lower_rt::c_record_name(&r.name)
        ));
    }
    out.push('\n');
    // 3) helper prototypes (value ctor / trap fwd / deep clone / free)
    for r in recs {
        let cr = lower_rt::c_record_name(&r.name);
        out.push_str(&format!(
            "a2x_res_{cr} a2x_{cr}_value({cr} v);\n\
             a2x_res_{cr} a2x_fwd_{cr}(a2x_trap_code t);\n\
             a2x_res_{cr} a2x_rt_clone_{cr}(const {cr}* src);\n\
             void a2x_rt_free_{cr}({cr}* r);\n",
            cr = cr
        ));
    }
    out.push('\n');
    // 4) helper definitions
    for r in recs {
        let cr = lower_rt::c_record_name(&r.name);
        out.push_str(&format!(
            "a2x_res_{cr} a2x_{cr}_value({cr} v) {{\n\
             \x20   a2x_res_{cr} r;\n\
             \x20   r.value = v; r.trap = A2X_TRAP_NONE;\n\
             \x20   return r;\n\
             }}\n\n\
             a2x_res_{cr} a2x_fwd_{cr}(a2x_trap_code t) {{\n\
             \x20   a2x_res_{cr} r;\n\
             \x20   memset(&r.value, 0, sizeof r.value);\n\
             \x20   r.trap = t;\n\
             \x20   return r;\n\
             }}\n\n",
            cr = cr
        ));
        // deep clone: fallible preparation field by field in declaration
        // order; a failure frees the PARTIAL copy (precommit atomicity —
        // the source stays valid, nothing half-committeed); one
        // allocation per NON-EMPTY owned Str field (PLAN-782)
        out.push_str(&format!(
            "a2x_res_{cr} a2x_rt_clone_{cr}(const {cr}* src) {{\n\
             \x20   {cr} out;\n\
             \x20   memset(&out, 0, sizeof out);\n",
            cr = cr
        ));
        for (name, ty) in &r.fields {
            match ty {
                seed::SeedFieldTy::Str | seed::SeedFieldTy::StrOwned => {
                    out.push_str(&format!(
                        "\x20   if (src->f_{n}.len == 0) {{\n\
                         \x20       out.f_{n}.data = NULL; out.f_{n}.len = 0;\n\
                         \x20   }} else {{\n\
                         \x20       a2x_stro_result b = a2x_str_clone(a2x_str_view_of(&src->f_{n}));\n\
                         \x20       if (b.trap != A2X_TRAP_NONE) {{ a2x_rt_free_{cr}(&out); return a2x_fwd_{cr}(b.trap); }}\n\
                         \x20       out.f_{n} = b.value;\n\
                         \x20   }}\n",
                        n = lower::sanitize(name),
                        cr = cr
                    ));
                }
                seed::SeedFieldTy::Record(_) => {
                    out.push_str(&format!(
                        "\x20   {{\n\
                         \x20       a2x_res_{inner_cr} c = a2x_rt_clone_{inner_cr}(&src->f_{n});\n\
                         \x20       if (c.trap != A2X_TRAP_NONE) {{ a2x_rt_free_{cr}(&out); return a2x_fwd_{cr}(c.trap); }}\n\
                         \x20       out.f_{n} = c.value;\n\
                         \x20   }}\n",
                        inner_cr = seed_field_c_ty(ty),
                        n = lower::sanitize(name),
                        cr = cr
                    ));
                }
                _ => {
                    out.push_str(&format!(
                        "\x20   out.f_{n} = src->f_{n};\n",
                        n = lower::sanitize(name)
                    ));
                }
            }
        }
        out.push_str(&format!(
            "\x20   return a2x_{cr}_value(out);\n\
             }}\n\n\
             void a2x_rt_free_{cr}({cr}* r) {{\n\
             \x20   (void)r; /* /W4: a field-free record frees nothing */\n",
            cr = cr
        ));
        for (name, ty) in &r.fields {
            match ty {
                seed::SeedFieldTy::Str | seed::SeedFieldTy::StrOwned => {
                    out.push_str(&format!(
                        "\x20   a2x_str_free(&r->f_{});\n",
                        lower::sanitize(name)
                    ));
                }
                seed::SeedFieldTy::Record(_) => {
                    out.push_str(&format!(
                        "\x20   a2x_rt_free_{}(&r->f_{});\n",
                        seed_field_c_ty(ty),
                        lower::sanitize(name)
                    ));
                }
                _ => {}
            }
        }
        out.push_str("}\n\n");
    }
    Ok(out)
}

/// The record-text channel's file set: `main.c` (the frozen observation
/// wrapper) + `a2x_c_rt_runtime.h` (the typed resource runtime) +
/// `generated.c` (the seed-derived record table + forward-declaration
/// block + fragment definitions) + manifest
/// (`auto.a2x.c.record-text.fileset/1`). Deterministic: the same
/// fragments + runtime + seed assemble to identical bytes.
fn assemble_record_text_fileset(req: FilesetRequest<'_>) -> Result<FileSet, A2xReject> {
    let FilesetRequest {
        entry,
        input_hash,
        family: _,
        fragments,
        view,
        runtime,
        prelude_seed,
        origins,
        ..
    } = req;
    // the record table: the SEED is the assembly's single source (the
    // cold path captured it from the same live view; the pure-restore
    // path carries it in the anchor — no view needed)
    let seed_text = match (prelude_seed, view) {
        (Some(s), _) => s.to_string(),
        (None, Some(v)) => seed::encode_seed(v)?,
        (None, None) => {
            return Err(A2xReject::new(
                "a2x.assembly",
                "record-text assembly needs the record table (prelude seed or live view)",
            ))
        }
    };
    let recs = seed::decode_seed(&seed_text)?;
    let mut generated = String::new();
    generated.push_str(&format!(
        "// AUTO-GENERATED by auto-a2x-c (PLAN-925 CA-B) — session assembly, entry `{}`\n\n",
        entry
    ));
    generated.push_str("#include \"a2x_c_rt_runtime.h\"\n\n");
    generated.push_str(&record_table_block(&recs)?);
    for r in fragments {
        let proto = prototype_line(r.text).ok_or_else(|| {
            A2xReject::new(
                "a2x.internal",
                format!("fragment `{}` carries no prototype line", r.source_name),
            )
        })?;
        generated.push_str(proto);
        generated.push('\n');
    }
    generated.push('\n');
    for r in fragments {
        generated.push_str(r.text);
        generated.push('\n');
    }
    if !fragments.iter().any(|r| r.source_name == entry) {
        return Err(A2xReject::entry("entry missing"));
    }
    // the observation wrapper protocol is the i32 channel — the entry's
    // prototype must be an `a2x_result` function (checked against the
    // fragment's own prototype line)
    let entry_proto = fragments
        .iter()
        .find(|r| r.source_name == entry)
        .and_then(|r| prototype_line(r.text))
        .unwrap_or("");
    if !entry_proto.starts_with("a2x_result ") {
        return Err(A2xReject::entry(
            "the entry must return i32 (the value:<i32>/trap:<Name> observation channel)",
        ));
    }
    let entry_c = lower::c_fn_name(entry);
    let wrapper = format!(
        "// AUTO-GENERATED (PLAN-925 CA-B) — DO NOT EDIT.\n#include <fcntl.h>\n#include <inttypes.h>\n#include <io.h>\n#include <stdint.h>\n#include <stdio.h>\n#include \"a2x_c_rt_runtime.h\"\n\na2x_result {entry}(void);\n\nint main(void) {{\n    _setmode(_fileno(stdout), _O_BINARY);\n    _setmode(_fileno(stderr), _O_BINARY);\n    a2x_result outcome = {entry}();\n    if (outcome.trap == A2X_TRAP_NONE) {{\n        printf(\"value:%\" PRId32 \"\\n\", outcome.value);\n    }} else {{\n        printf(\"trap:%s\\n\", a2x_trap_name(outcome.trap));\n    }}\n    return 0;\n}}\n",
        entry = entry_c
    );
    let files = vec![
        ("main.c".to_string(), wrapper),
        ("a2x_c_rt_runtime.h".to_string(), runtime.to_string()),
        ("generated.c".to_string(), generated),
    ];
    let mut manifest = String::new();
    manifest.push_str(&format!("{{\n  \"schema\": \"{}\",\n", C_RT_FILESET_SCHEMA));
    manifest.push_str(&format!("  \"input_hash\": \"{}\",\n", input_hash));
    manifest.push_str(&format!(
        "  \"entry\": \"{}\",\n  \"functions\": [\n",
        entry
    ));
    for (i, r) in fragments.iter().enumerate() {
        let comma = if i + 1 < fragments.len() { "," } else { "" };
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
            sha256_hex(content.as_bytes()),
            comma
        ));
    }
    manifest.push_str("  ]\n}\n");
    Ok(FileSet { files, manifest })
}

/// The collections channel's file set: `main.c` (the frozen observation
/// wrapper) + `a2x_c_coll_runtime.h` (the collections runtime) +
/// `generated.c` (the seed-derived nominal type table — records +
/// variants + typed list helpers — + forward-declaration block +
/// fragment definitions) + manifest (`auto.a2x.c.collections.fileset/1`).
/// Deterministic: the same fragments + runtime + seed assemble to
/// identical bytes.
fn assemble_collections_fileset(req: FilesetRequest<'_>) -> Result<FileSet, A2xReject> {
    let FilesetRequest {
        entry,
        input_hash,
        family: _,
        fragments,
        view,
        runtime,
        prelude_seed,
        origins,
        ..
    } = req;
    // the nominal type table: the SEED is the assembly's single source
    // (the cold path captured it from the same live view; the
    // pure-restore path carries it in the anchor — no view needed)
    let seed_text = match (prelude_seed, view) {
        (Some(s), _) => s.to_string(),
        (None, Some(v)) => seed_coll::encode_seed_coll(v)?,
        (None, None) => {
            return Err(A2xReject::new(
                "a2x.assembly",
                "collections assembly needs the nominal type table (prelude seed or live view)",
            ))
        }
    };
    let seed = seed_coll::decode_seed_coll(&seed_text)?;
    let type_table = coll_table::coll_type_table_block(&seed)
        .map_err(|e| A2xReject::new("a2x.seed", format!("collections type table: {}", e)))?;
    let mut generated = String::new();
    generated.push_str(&format!(
        "// AUTO-GENERATED by auto-a2x-c (PLAN-926 CA-C) — session assembly, entry `{}`\n\n",
        entry
    ));
    generated.push_str("#include \"a2x_c_coll_runtime.h\"\n\n");
    generated.push_str(&type_table);
    for r in fragments {
        let proto = prototype_line(r.text).ok_or_else(|| {
            A2xReject::new(
                "a2x.internal",
                format!("fragment `{}` carries no prototype line", r.source_name),
            )
        })?;
        generated.push_str(proto);
        generated.push('\n');
    }
    generated.push('\n');
    for r in fragments {
        generated.push_str(r.text);
        generated.push('\n');
    }
    if !fragments.iter().any(|r| r.source_name == entry) {
        return Err(A2xReject::entry("entry missing"));
    }
    // the observation wrapper protocol is the i32 channel — the entry's
    // prototype must be an `a2x_result` function (checked against the
    // fragment's own prototype line)
    let entry_proto = fragments
        .iter()
        .find(|r| r.source_name == entry)
        .and_then(|r| prototype_line(r.text))
        .unwrap_or("");
    if !entry_proto.starts_with("a2x_result ") {
        return Err(A2xReject::entry(
            "the entry must return i32 (the value:<i32>/trap:<Name> observation channel)",
        ));
    }
    let entry_c = lower::c_fn_name(entry);
    let wrapper = format!(
        "// AUTO-GENERATED (PLAN-926 CA-C) — DO NOT EDIT.\n#include <fcntl.h>\n#include <inttypes.h>\n#include <io.h>\n#include <stdint.h>\n#include <stdio.h>\n#include \"a2x_c_coll_runtime.h\"\n\na2x_result {entry}(void);\n\nint main(void) {{\n    _setmode(_fileno(stdout), _O_BINARY);\n    _setmode(_fileno(stderr), _O_BINARY);\n    a2x_result outcome = {entry}();\n    if (outcome.trap == A2X_TRAP_NONE) {{\n        printf(\"value:%\" PRId32 \"\\n\", outcome.value);\n    }} else {{\n        printf(\"trap:%s\\n\", a2x_trap_name(outcome.trap));\n    }}\n    return 0;\n}}\n",
        entry = entry_c
    );
    let files = vec![
        ("main.c".to_string(), wrapper),
        ("a2x_c_coll_runtime.h".to_string(), runtime.to_string()),
        ("generated.c".to_string(), generated),
    ];
    let mut manifest = String::new();
    manifest.push_str(&format!(
        "{{\n  \"schema\": \"{}\",\n",
        C_COLL_FILESET_SCHEMA
    ));
    manifest.push_str(&format!("  \"input_hash\": \"{}\",\n", input_hash));
    manifest.push_str(&format!(
        "  \"entry\": \"{}\",\n  \"functions\": [\n",
        entry
    ));
    for (i, r) in fragments.iter().enumerate() {
        let comma = if i + 1 < fragments.len() { "," } else { "" };
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
            sha256_hex(content.as_bytes()),
            comma
        ));
    }
    manifest.push_str("  ]\n}\n");
    Ok(FileSet { files, manifest })
}
