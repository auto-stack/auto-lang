//! PLAN-937 CA-D1 (T-05): PURE generation of the public scalar export
//! surface for the C channel — the thin `library` form of the scalar
//! emit target.
//!
//! What this module is: a deterministic text generator + file-set
//! assembler. It reads NOTHING from disk, spawns nothing, and reads no
//! expected outputs — the independent oracle is the hand-written
//! `bridge_source_caller.c` on the abi side, whose expectations are
//! hand-computed. Its inputs are the REAL provider outputs (the exe
//! fileset's `generated.c` + `a2x_c_runtime.h` bytes) plus explicit
//! export specs; the old exe fileset is passed through untouched (the
//! library form is a SEPARATE assembly, never a main-stripped copy —
//! 937 plan §5.2).
//!
//! The public form (v0.1, source channel):
//! * every source export is FALLIBLE — the private `a2x_result` is
//!   NEVER exposed publicly; the wrapper converts explicitly
//!   (ctx/status/out lanes);
//! * `int32_t <sym>(void* ctx, <params...>, <ret>* out, int32_t* reason)`
//!   with a NULLABLE ctx (never dereferenced), success writing
//!   `*out`/`*reason=0` and returning 0, failure writing `*reason` to
//!   the ORIGINAL trap code and returning the mapped status;
//! * the trap→status mapping is an EXPLICIT frozen table keyed by the
//!   trap ENUM (`a2x_trap_code`), never exit codes or strings; the
//!   reason lane preserves the original cause (937 plan §5.3).
//!
//! Lane spellings come from the shared foreign-contract projection
//! (`PhysLane::c_spell`) so the generated header and the neutral
//! physical plan can never drift apart.

use ace_a2x_core::target::FileSet;
use ace_foreign_contract::LaneKind;

/// The library file-set schema (distinct from the exe scalar fileset
/// `auto.a2x.c.fileset/1`; a library manifest can never satisfy an exe
/// identity or vice versa).
pub const LIBRARY_FILESET_SCHEMA: &str = "auto.a2x.c.library.fileset/1";
/// The recipe identity of the source-export chain (worker/parent
/// admission binds this; the old `ac-a2c/hir-scalar-emit-v1` recipe is
/// the EXE form and must keep rejecting library decks).
pub const LIBRARY_EXPORT_RECIPE: &str = "ac-a2c/hir-scalar-library-export-v1";

/// Public status codes of the source-export wrapper. Frozen table —
/// these are the values the facts' error model declares and the
/// independent caller asserts; changing any of them is an interface
/// revision, not a local edit.
pub mod status {
    pub const OK: i32 = 0;
    /// A trap the table does not know (defensive default — never a
    /// guess about the cause; reason carries the raw trap code).
    pub const UNKNOWN_TRAP: i32 = -100;
    pub const OVERFLOW: i32 = -101;
    pub const DIV_BY_ZERO: i32 = -102;
    pub const DIV_OVERFLOW: i32 = -103;
    pub const REM_BY_ZERO: i32 = -104;
    pub const REM_OVERFLOW: i32 = -105;
}

/// One row of the explicit trap→status table (trap code + public status
/// + human reason string). The trap codes are the PRIVATE runtime's
/// enum values (runtime.rs); only this table makes them public.
pub struct TrapStatusRow {
    pub trap_code: i32,
    pub trap_name: &'static str,
    pub status: i32,
    pub reason: &'static str,
}

/// The frozen trap→status table (order = runtime enum order).
pub const TRAP_STATUS_TABLE: [TrapStatusRow; 5] = [
    TrapStatusRow {
        trap_code: 1,
        trap_name: "A2X_TRAP_OVERFLOW",
        status: status::OVERFLOW,
        reason: "checked-arithmetic overflow",
    },
    TrapStatusRow {
        trap_code: 2,
        trap_name: "A2X_TRAP_DIV_BY_ZERO",
        status: status::DIV_BY_ZERO,
        reason: "division by zero",
    },
    TrapStatusRow {
        trap_code: 3,
        trap_name: "A2X_TRAP_DIV_OVERFLOW",
        status: status::DIV_OVERFLOW,
        reason: "division overflow (INT32_MIN / -1)",
    },
    TrapStatusRow {
        trap_code: 4,
        trap_name: "A2X_TRAP_REM_BY_ZERO",
        status: status::REM_BY_ZERO,
        reason: "remainder by zero",
    },
    TrapStatusRow {
        trap_code: 5,
        trap_name: "A2X_TRAP_REM_OVERFLOW",
        status: status::REM_OVERFLOW,
        reason: "remainder overflow (INT32_MIN % -1)",
    },
];

/// The error model rows for foreign-contract facts (sorted by status —
/// the facts validator requires ascending unique negative codes).
pub fn public_error_model_rows() -> Vec<(i32, String)> {
    let mut rows: Vec<(i32, String)> = TRAP_STATUS_TABLE
        .iter()
        .map(|r| (r.status, r.reason.to_string()))
        .collect();
    rows.push((status::UNKNOWN_TRAP, "unknown trap code".into()));
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows
}

/// One PUBLIC param lane of a source export. PLAN-939 T-11 (D-939-CE-1's
/// bridge arm): besides the 937 scalar lanes (i32/bool), the source
/// channel exports the resource shapes the 939 matrix promises for CE —
/// a borrowed UTF-8 view and closed record views. Wider scalar lanes
/// (i64/u64/f64) still belong to the typed bridge, never the exporter.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ExportLane {
    I32,
    Bool,
    /// Borrowed UTF-8 view — public `(const uint8_t* data, size_t len)`
    /// pair, marshalled to the fragment's `a2x_str_view`.
    ViewText,
    /// Closed record view — public `const <Rec>*` (natural scalar layout).
    RecConst {
        rec: String,
    },
    /// Closed record mut view — public `<Rec>*` (written back through it).
    RecMut {
        rec: String,
    },
}

impl ExportLane {
    /// The admission-grammar spelling (must match descriptor3's
    /// `ParamKind::spell` for the same shape).
    pub fn spell(&self) -> String {
        match self {
            ExportLane::I32 => "i32".into(),
            ExportLane::Bool => "bool".into(),
            ExportLane::ViewText => "buf:utf8".into(),
            ExportLane::RecConst { rec } => format!("rec:{rec}:const"),
            ExportLane::RecMut { rec } => format!("rec:{rec}:mut"),
        }
    }
}

/// A public record closure emitted into the export header: closed
/// scalar fields in declaration order, natural alignment (§5.1: the
/// public record closure is naturally aligned scalars — never the
/// internal resource record's private representation).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PublicRecDef {
    pub name: String,
    pub fields: Vec<(String, PublicRecField)>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PublicRecField {
    I32,
    Bool,
}

impl PublicRecField {
    fn c_spell(self) -> &'static str {
        match self {
            PublicRecField::I32 => "int32_t",
            PublicRecField::Bool => "bool",
        }
    }
}

/// One source-function export specification. Params carry the source
/// channel's `ExportLane` vocabulary; the result lane stays i32/bool.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExportSpec {
    /// Public C symbol of the wrapper (hygiene: ASCII identifier).
    pub public_symbol: String,
    /// C name of the generated business body (the provider's `fn_*`).
    pub source_fn: String,
    /// Logical param lanes (i32/bool + the 939 resource view lanes).
    pub params: Vec<ExportLane>,
    /// Logical result lane (written to `*out` on success).
    pub ret: LaneKind,
}

impl ExportSpec {
    fn check(&self) -> Result<(), String> {
        let ident =
            |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_');
        if !ident(&self.public_symbol) {
            return Err(format!(
                "public symbol '{}' is not a C identifier",
                self.public_symbol
            ));
        }
        if !ident(&self.source_fn) {
            return Err(format!(
                "source fn '{}' is not a C identifier",
                self.source_fn
            ));
        }
        if !matches!(self.ret, LaneKind::I32 | LaneKind::Bool) {
            return Err(format!(
                "export '{}': ret lane {:?} is outside the source channel (i32/bool)",
                self.public_symbol, self.ret
            ));
        }
        Ok(())
    }
    /// Every record name referenced by a param lane.
    pub fn referenced_recs(&self) -> Vec<&str> {
        self.params
            .iter()
            .filter_map(|p| match p {
                ExportLane::RecConst { rec } | ExportLane::RecMut { rec } => Some(rec.as_str()),
                _ => None,
            })
            .collect()
    }
}

/// The INTERNAL record table for the compiled TUs (library.c +
/// generated.c): the fragments spell `a2x_rec_<Rec>` with hygienic
/// `f_`-prefixed field names — the exact struct the record-text
/// lowering emits (same layout as the public closure by construction:
/// same field types, same declaration order).
fn record_block(recs: &[PublicRecDef]) -> String {
    let mut s = String::new();
    for r in recs {
        s.push_str("typedef struct {");
        for (name, ty) in &r.fields {
            s.push_str(&format!(" {} f_{};", ty.c_spell(), name));
        }
        s.push_str(&format!(" }} a2x_rec_{};\n", r.name));
    }
    s
}

/// The public-header record closures (public field names; the internal
/// `a2x_rec_*` hygienic names never cross the public boundary).
fn public_record_block(recs: &[PublicRecDef]) -> String {
    let mut s = String::new();
    for r in recs {
        s.push_str("typedef struct {");
        for (name, ty) in &r.fields {
            s.push_str(&format!(" {} {};", ty.c_spell(), name));
        }
        s.push_str(&format!(" }} {};\n", r.name));
    }
    s
}

/// Check that every record a spec's lanes reference has exactly one
/// closed definition in the table.
fn check_rec_refs(specs: &[ExportSpec], recs: &[PublicRecDef]) -> Result<(), String> {
    let ident =
        |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_');
    for r in recs {
        if !ident(&r.name) {
            return Err(format!("record name '{}' is not a C identifier", r.name));
        }
    }
    for spec in specs {
        for rec in spec.referenced_recs() {
            if !recs.iter().any(|r| r.name == rec) {
                return Err(format!(
                    "export '{}': record '{}' has no closed public definition",
                    spec.public_symbol, rec
                ));
            }
        }
    }
    Ok(())
}

fn c_lane(k: LaneKind) -> &'static str {
    k.physical().c_spell()
}

/// The public C parameter declaration(s) of one export lane.
fn public_param_decl(p: &ExportLane, i: usize) -> String {
    match p {
        ExportLane::I32 => format!("int32_t p{i}"),
        ExportLane::Bool => format!("bool p{i}"),
        ExportLane::ViewText => format!("const uint8_t* p{i}_data, size_t p{i}_len"),
        ExportLane::RecConst { rec } => format!("const {rec}* p{i}"),
        ExportLane::RecMut { rec } => format!("{rec}* p{i}"),
    }
}

/// The fragment-side C parameter declaration of one export lane (the
/// internal ABI the generated business body actually takes).
fn fragment_param_decl(p: &ExportLane, i: usize) -> String {
    match p {
        ExportLane::I32 => format!("int32_t p{i}"),
        ExportLane::Bool => format!("bool p{i}"),
        ExportLane::ViewText => format!("a2x_str_view p{i}_v"),
        ExportLane::RecConst { rec } => format!("const a2x_rec_{rec}* p{i}"),
        ExportLane::RecMut { rec } => format!("a2x_rec_{rec}* p{i}"),
    }
}

/// The wrapper's call argument for one export lane (built after any
/// marshalling locals the lane needs).
fn call_argument(p: &ExportLane, i: usize) -> String {
    match p {
        ExportLane::I32 | ExportLane::Bool => format!("p{i}"),
        ExportLane::ViewText => format!("p{i}_v"),
        ExportLane::RecConst { rec } => format!("(const a2x_rec_{rec}*)p{i}"),
        ExportLane::RecMut { rec } => format!("(a2x_rec_{rec}*)p{i}"),
    }
}

/// The wrapper's marshalling locals for one lane (the UTF-8 view pair
/// becomes one `a2x_str_view`; record pointers need no local — the call
/// argument casts to the layout-identical internal struct).
fn marshalling_locals(p: &ExportLane, i: usize) -> String {
    match p {
        ExportLane::ViewText => format!(
            "    a2x_str_view p{i}_v;\n    p{i}_v.data = p{i}_data;\n    p{i}_v.len = p{i}_len;\n"
        ),
        _ => String::new(),
    }
}

/// The public prototype of one export (the form both the generated
/// header and the hand-written independent header spell).
pub fn public_prototype(spec: &ExportSpec) -> String {
    let mut s = format!("int32_t {}(void* ctx", spec.public_symbol);
    for (i, p) in spec.params.iter().enumerate() {
        s.push_str(&format!(", {}", public_param_decl(p, i)));
    }
    s.push_str(&format!(", {}* out, int32_t* reason)", c_lane(spec.ret)));
    s
}

/// The wrapper definitions text (`library.c` body): explicit ctx
/// discard, trap-table switch, reason-preserving failure path, and
/// success committing out/reason in declaration order.
pub fn wrapper_c(specs: &[ExportSpec], recs: &[PublicRecDef]) -> String {
    let mut s = String::new();
    s.push_str("// PLAN-937 CA-D1: public scalar export wrappers (generated; fallible form).\n");
    s.push_str("// The private a2x_result never crosses this boundary.\n");
    s.push_str("#include \"a2x_c_runtime.h\"\n\n");
    // PLAN-939 T-11: BOTH record spellings live in this TU — the PUBLIC
    // closure (the prototypes below spell its names) and the INTERNAL
    // table the fragments spell (hygienic f_-prefixed fields; same
    // layout by construction: same field types, same order).
    s.push_str(&public_record_block(recs));
    s.push_str(&record_block(recs));
    s.push('\n');
    // extern prototypes of the real business bodies (generated.c is a
    // separate translation unit in the library link).
    for spec in specs {
        let params = if spec.params.is_empty() {
            "void".to_string()
        } else {
            spec.params
                .iter()
                .enumerate()
                .map(|(i, p)| fragment_param_decl(p, i))
                .collect::<Vec<_>>()
                .join(", ")
        };
        s.push_str(&format!(
            "extern a2x_result {}({params});\n",
            spec.source_fn
        ));
    }
    s.push('\n');
    for spec in specs {
        s.push_str(&format!("// {}\n", public_prototype(spec)));
        s.push_str(&format!("int32_t {}(void* ctx", spec.public_symbol));
        for (i, p) in spec.params.iter().enumerate() {
            s.push_str(&format!(", {}", public_param_decl(p, i)));
        }
        s.push_str(&format!(
            ", {}* out, int32_t* reason) {{\n",
            c_lane(spec.ret)
        ));
        s.push_str("    (void)ctx;\n");
        for (i, p) in spec.params.iter().enumerate() {
            s.push_str(&marshalling_locals(p, i));
        }
        s.push_str("    a2x_result r = ");
        s.push_str(&spec.source_fn);
        s.push('(');
        s.push_str(
            &spec
                .params
                .iter()
                .enumerate()
                .map(|(i, p)| call_argument(p, i))
                .collect::<Vec<_>>()
                .join(", "),
        );
        s.push_str(");\n");
        s.push_str("    if (r.trap != A2X_TRAP_NONE) {\n");
        s.push_str("        *reason = (int32_t)r.trap;\n");
        s.push_str("        switch (r.trap) {\n");
        for row in TRAP_STATUS_TABLE.iter() {
            s.push_str(&format!(
                "        case {}: return {};\n",
                row.trap_name, row.status
            ));
        }
        s.push_str(&format!(
            "        default: return {}; /* no guess: reason carries the raw code */\n",
            status::UNKNOWN_TRAP
        ));
        s.push_str("        }\n");
        s.push_str("    }\n");
        s.push_str("    *reason = 0;\n");
        if spec.ret == LaneKind::Bool {
            s.push_str("    *out = r.value ? 1 : 0;\n");
        } else {
            s.push_str("    *out = r.value;\n");
        }
        s.push_str("    return 0;\n");
        s.push_str("}\n\n");
    }
    s
}

/// The generated public header (`abi_export.h`) — record closures plus
/// prototypes only.
pub fn public_header(specs: &[ExportSpec], recs: &[PublicRecDef]) -> String {
    let mut s = String::new();
    s.push_str("// PLAN-937 CA-D1: public scalar export header (generated).\n");
    s.push_str("// Fallible form: 0 = success (out/reason committed), negative =\n");
    s.push_str("// mapped status (reason carries the original trap code).\n");
    s.push_str("#ifndef A2X_ABI_EXPORT_H\n#define A2X_ABI_EXPORT_H\n\n");
    s.push_str("#include <stdint.h>\n#include <stdbool.h>\n#include <stddef.h>\n\n");
    // PLAN-939 T-11: the public record closures (closed scalar fields,
    // natural alignment — the internal a2x_rec_* names never appear).
    s.push_str(&public_record_block(recs));
    s.push('\n');
    for spec in specs {
        s.push_str(&public_prototype(spec));
        s.push_str(";\n");
    }
    s.push_str("\n#endif\n");
    s
}

/// Identity inputs recorded in the library manifest.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LibraryFilesetInputs {
    /// Entry function of the underlying exe fileset (identity carryover).
    pub entry: String,
    /// The source input hash of the underlying assembly.
    pub input_hash: String,
    /// Module/profile name the assembly was compiled under.
    pub module: String,
}

/// Assemble the LIBRARY file set from the real provider outputs.
///
/// `fragments` are the provider's fragment texts (the `// origin` line,
/// the PROTOTYPE line, then the definition — same shape the exe
/// assembler consumes); the library's `generated.c` is assembled HERE
/// from them (header + include + prototype forward-declaration block +
/// definitions). Files (fixed order): `library.c` (wrappers),
/// `abi_export.h` (public header), `generated.c` (REAL business),
/// `a2x_c_runtime.h` (runtime, passthrough), `manifest.json`. There is
/// deliberately NO `main.c` and no process entry — the library form is
/// assembled explicitly, never by stripping the exe form.
pub fn assemble_library_fileset(
    fragments: &[&str],
    runtime_h: &str,
    specs: &[ExportSpec],
    recs: &[PublicRecDef],
    inputs: &LibraryFilesetInputs,
) -> Result<FileSet, String> {
    for spec in specs {
        spec.check()?;
    }
    check_rec_refs(specs, recs)?;
    if specs.is_empty() {
        return Err("library fileset needs at least one export".into());
    }
    if fragments.is_empty() {
        return Err("library fileset needs the provider fragments".into());
    }
    let mut seen = std::collections::BTreeSet::new();
    for spec in specs {
        if !seen.insert(spec.public_symbol.clone()) {
            return Err(format!("duplicate public symbol '{}'", spec.public_symbol));
        }
    }
    // generated.c: header comment + runtime include + the record table
    // (the fragments spell a2x_rec_* — the TU is self-contained) +
    // forward declarations (each fragment's prototype line) + definitions.
    let mut generated = String::new();
    generated.push_str(&format!(
        "// PLAN-937 CA-D1 library form (module {}, entry {})\n",
        inputs.module, inputs.entry
    ));
    generated.push_str("#include \"a2x_c_runtime.h\"\n\n");
    generated.push_str(&record_block(recs));
    generated.push('\n');
    for frag in fragments {
        let proto = frag
            .lines()
            .nth(1)
            .ok_or_else(|| "fragment missing prototype line".to_string())?;
        generated.push_str(proto);
        generated.push('\n');
    }
    generated.push('\n');
    for frag in fragments {
        generated.push_str(frag);
        generated.push_str("\n\n");
    }
    let library_c = wrapper_c(specs, recs);
    let header = public_header(specs, recs);
    let files: Vec<(String, String)> = vec![
        ("library.c".to_string(), library_c),
        ("abi_export.h".to_string(), header),
        ("generated.c".to_string(), generated),
        ("a2x_c_runtime.h".to_string(), runtime_h.to_string()),
    ];
    let file_rows: Vec<String> = files
        .iter()
        .map(|(p, text)| {
            format!(
                "{{\"path\":\"{p}\",\"sha256\":\"{}\"}}",
                ace_a2x_core::sha256::sha256_hex(text.as_bytes())
            )
        })
        .collect();
    let export_rows: Vec<String> = specs
        .iter()
        .map(|s| {
            format!(
                "{{\"symbol\":\"{}\",\"source_fn\":\"{}\",\"params\":\"{}\",\"ret\":\"{}\",\"form\":\"fallible\",\"outs\":\"{},reason\"}}",
                s.public_symbol,
                s.source_fn,
                s.params.iter().map(|p| p.spell()).collect::<Vec<_>>().join(","),
                s.ret.spell(),
                s.ret.spell()
            )
        })
        .collect();
    let manifest = format!(
        "{{\"schema\":\"{LIBRARY_FILESET_SCHEMA}\",\"recipe\":\"{LIBRARY_EXPORT_RECIPE}\",\"input_hash\":\"{}\",\"module\":\"{}\",\"entry\":\"{}\",\"exports\":[{}],\"files\":[{}]}}",
        inputs.input_hash,
        inputs.module,
        inputs.entry,
        export_rows.join(","),
        file_rows.join(",")
    );
    Ok(FileSet { files, manifest })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sum_spec() -> ExportSpec {
        ExportSpec {
            public_symbol: "t937_sum".into(),
            source_fn: "fn_sum".into(),
            params: vec![ExportLane::I32, ExportLane::I32],
            ret: LaneKind::I32,
        }
    }

    #[test]
    fn prototype_is_the_fallible_public_form() {
        assert_eq!(
            public_prototype(&sum_spec()),
            "int32_t t937_sum(void* ctx, int32_t p0, int32_t p1, int32_t* out, int32_t* reason)"
        );
        let b = ExportSpec {
            public_symbol: "t937_flag".into(),
            source_fn: "fn_flag".into(),
            params: vec![ExportLane::I32],
            ret: LaneKind::Bool,
        };
        assert_eq!(
            public_prototype(&b),
            "int32_t t937_flag(void* ctx, int32_t p0, bool* out, int32_t* reason)"
        );
    }

    #[test]
    fn wrapper_maps_traps_by_enum_not_by_string_or_exit() {
        let c = wrapper_c(&[sum_spec()], &[]);
        assert!(c.contains("case A2X_TRAP_DIV_BY_ZERO: return -102;"));
        assert!(c.contains("case A2X_TRAP_OVERFLOW: return -101;"));
        assert!(c.contains("default: return -100;"));
        // reason preserves the RAW trap code before the switch
        assert!(c.contains("*reason = (int32_t)r.trap;"));
        // success commits out+reason only after the trap branch
        let trap_at = c.find("if (r.trap").unwrap();
        let ok_at = c.find("*reason = 0;").unwrap();
        assert!(trap_at < ok_at);
        // no string/exit-code guessing surface
        assert!(!c.contains("strcmp"));
        assert!(!c.contains("exit("));
        // the real business bodies are declared extern (cross-TU link)
        assert!(c.contains("extern a2x_result fn_sum(int32_t p0, int32_t p1);"));
        // ctx is never dereferenced
        assert!(c.contains("(void)ctx;"));
        assert!(!c.contains("*ctx"));
    }

    #[test]
    fn library_fileset_shape_and_determinism() {
        let frag = "// origin: [0,10]\na2x_result fn_sum(int32_t a, int32_t b);\na2x_result fn_sum(int32_t a, int32_t b) { return a2x_add(a, b); }\n";
        let inputs = LibraryFilesetInputs {
            entry: "main".into(),
            input_hash: "ih".into(),
            module: "m".into(),
        };
        let fs = assemble_library_fileset(&[frag], "RT", &[sum_spec()], &[], &inputs).unwrap();
        let names: Vec<&str> = fs.files.iter().map(|(p, _)| p.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "library.c",
                "abi_export.h",
                "generated.c",
                "a2x_c_runtime.h"
            ]
        );
        assert!(!names.contains(&"main.c"));
        // generated.c carries the REAL fragment bytes + prototype block
        assert!(fs.files[2]
            .1
            .contains("a2x_result fn_sum(int32_t a, int32_t b);"));
        assert!(fs.files[2].1.contains("return a2x_add(a, b);"));
        assert_eq!(fs.files[3].1, "RT");
        assert!(fs.manifest.contains(LIBRARY_FILESET_SCHEMA));
        assert!(fs.manifest.contains(LIBRARY_EXPORT_RECIPE));
        assert!(fs.manifest.contains("\"symbol\":\"t937_sum\""));
        // determinism: identical inputs -> byte-identical outputs
        let fs2 = assemble_library_fileset(&[frag], "RT", &[sum_spec()], &[], &inputs).unwrap();
        assert!(
            fs.files == fs2.files,
            "library fileset must be byte-deterministic"
        );
        assert!(fs.manifest == fs2.manifest);
        // a body edit changes generated.c bytes AND manifest hashes
        // (reassociated body: same observable semantics, distinct bytes —
        // the fragment text carries the runtime helper, not raw )
        let frag2 = frag.replace("a2x_add(a, b)", "a2x_add(b, a)");
        let fs3 =
            assemble_library_fileset(&[frag2.as_str()], "RT", &[sum_spec()], &[], &inputs).unwrap();
        assert_ne!(fs.manifest, fs3.manifest);
    }

    #[test]
    fn lane_restriction_enforced() {
        // The RET lane stays the finite scalar set (i32/bool); params
        // carry the ExportLane vocabulary by construction.
        let bad = ExportSpec {
            public_symbol: "x".into(),
            source_fn: "fn_x".into(),
            params: vec![ExportLane::I32],
            ret: LaneKind::F64,
        };
        let err = match assemble_library_fileset(
            &["// origin: [0,1]\na2x_result fn_x(double a);"],
            "R",
            &[bad],
            &[],
            &LibraryFilesetInputs {
                entry: "e".into(),
                input_hash: "i".into(),
                module: "m".into(),
            },
        ) {
            Ok(_) => panic!("f64 ret lane must be rejected for the source channel"),
            Err(e) => e,
        };
        assert!(err.contains("source channel"), "{err}");
        // A record lane without a closed public definition is refused.
        let rec_lane = ExportSpec {
            public_symbol: "y".into(),
            source_fn: "fn_y".into(),
            params: vec![ExportLane::RecMut {
                rec: "Counter".into(),
            }],
            ret: LaneKind::I32,
        };
        let err = match assemble_library_fileset(
            &["// origin: [0,1]\na2x_result fn_y(a2x_rec_Counter* p0);"],
            "R",
            &[rec_lane.clone()],
            &[],
            &LibraryFilesetInputs {
                entry: "e".into(),
                input_hash: "i".into(),
                module: "m".into(),
            },
        ) {
            Ok(_) => panic!("an undefined record closure must be rejected"),
            Err(e) => e,
        };
        assert!(err.contains("no closed public definition"), "{err}");
    }

    /// PLAN-939 T-11 (D-939-CE-1 bridge arm): the resource view lanes'
    /// public form — the UTF-8 view pair, the closed record pointers,
    /// the marshalling locals, and the public/internal record duality.
    #[test]
    fn resource_view_lanes_generate_the_public_ce_form() {
        use ExportLane::{RecConst, RecMut, ViewText};
        let counter = PublicRecDef {
            name: "Counter".into(),
            fields: vec![("value".into(), PublicRecField::I32)],
        };
        let spec = ExportSpec {
            public_symbol: "t939ce_probe".into(),
            source_fn: "fn_probe".into(),
            params: vec![
                ViewText,
                RecConst {
                    rec: "Counter".into(),
                },
                RecMut {
                    rec: "Counter".into(),
                },
            ],
            ret: LaneKind::I32,
        };
        let proto = public_prototype(&spec);
        assert_eq!(
            proto,
            "int32_t t939ce_probe(void* ctx, const uint8_t* p0_data, size_t p0_len, \
const Counter* p1, Counter* p2, int32_t* out, int32_t* reason)"
        );
        let header = public_header(&[spec.clone()], &[counter.clone()]);
        assert!(
            header.contains("typedef struct { int32_t value; } Counter;"),
            "the public header carries the closed record closure"
        );
        assert!(
            !header.contains("a2x_rec_"),
            "the internal a2x_rec_* names never cross the public boundary"
        );
        let c = wrapper_c(&[spec], &[counter]);
        // the UTF-8 view pair marshals to ONE a2x_str_view before the call
        assert!(c.contains("a2x_str_view p0_v;"));
        assert!(c.contains("p0_v.data = p0_data;"));
        assert!(c.contains("p0_v.len = p0_len;"));
        // record pointers pass through typed via the public/internal alias
        assert!(c.contains("typedef struct { int32_t f_value; } a2x_rec_Counter;"));
        assert!(c.contains(
            "extern a2x_result fn_probe(a2x_str_view p0_v, \
const a2x_rec_Counter* p1, a2x_rec_Counter* p2);"
        ));
        assert!(c.contains("r = fn_probe(p0_v, (const a2x_rec_Counter*)p1, (a2x_rec_Counter*)p2);"));
        // the admission spellings match descriptor3's ParamKind vocabulary
        assert_eq!(ViewText.spell(), "buf:utf8");
        assert_eq!(RecConst { rec: "C".into() }.spell(), "rec:C:const");
        assert_eq!(RecMut { rec: "C".into() }.spell(), "rec:C:mut");
    }

    #[test]
    fn error_model_rows_sorted_negative_unique() {
        let rows = public_error_model_rows();
        let codes: Vec<i32> = rows.iter().map(|(c, _)| *c).collect();
        let mut sorted = codes.clone();
        sorted.sort();
        assert_eq!(codes, sorted);
        sorted.dedup();
        assert_eq!(codes.len(), sorted.len());
        assert!(codes.iter().all(|c| *c < 0));
        assert!(codes.contains(&status::DIV_BY_ZERO));
    }
}
