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

/// One source-function export specification. Lanes are restricted to
/// the source channel's compute types (i32/bool); wider public lanes
/// belong to the typed bridge, not the source exporter.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExportSpec {
    /// Public C symbol of the wrapper (hygiene: ASCII identifier).
    pub public_symbol: String,
    /// C name of the generated business body (the provider's `fn_*`).
    pub source_fn: String,
    /// Logical param lanes (i32/bool only for the source channel).
    pub params: Vec<LaneKind>,
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
        for k in self.params.iter().chain(std::iter::once(&self.ret)) {
            if !matches!(k, LaneKind::I32 | LaneKind::Bool) {
                return Err(format!(
                    "export '{}': lane {k:?} is outside the source channel (i32/bool)",
                    self.public_symbol
                ));
            }
        }
        Ok(())
    }
}

fn c_lane(k: LaneKind) -> &'static str {
    k.physical().c_spell()
}

/// The public prototype of one export (the form both the generated
/// header and the hand-written independent header spell).
pub fn public_prototype(spec: &ExportSpec) -> String {
    let mut s = format!("int32_t {}(void* ctx", spec.public_symbol);
    for (i, p) in spec.params.iter().enumerate() {
        s.push_str(&format!(", {} p{}", c_lane(*p), i));
    }
    s.push_str(&format!(", {}* out, int32_t* reason)", c_lane(spec.ret)));
    s
}

/// The wrapper definitions text (`library.c` body): explicit ctx
/// discard, trap-table switch, reason-preserving failure path, and
/// success committing out/reason in declaration order.
pub fn wrapper_c(specs: &[ExportSpec]) -> String {
    let mut s = String::new();
    s.push_str("// PLAN-937 CA-D1: public scalar export wrappers (generated; fallible form).\n");
    s.push_str("// The private a2x_result never crosses this boundary.\n");
    s.push_str("#include \"a2x_c_runtime.h\"\n\n");
    // extern prototypes of the real business bodies (generated.c is a
    // separate translation unit in the library link).
    for spec in specs {
        let params = if spec.params.is_empty() {
            "void".to_string()
        } else {
            spec.params
                .iter()
                .enumerate()
                .map(|(i, p)| format!("{} p{}", c_lane(*p), i))
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
            s.push_str(&format!(", {} p{}", c_lane(*p), i));
        }
        s.push_str(&format!(
            ", {}* out, int32_t* reason) {{\n",
            c_lane(spec.ret)
        ));
        s.push_str("    (void)ctx;\n");
        s.push_str("    a2x_result r = ");
        s.push_str(&spec.source_fn);
        s.push('(');
        s.push_str(
            &spec
                .params
                .iter()
                .enumerate()
                .map(|(i, _)| format!("p{i}"))
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

/// The generated public header (`abi_export.h`) — prototypes only.
pub fn public_header(specs: &[ExportSpec]) -> String {
    let mut s = String::new();
    s.push_str("// PLAN-937 CA-D1: public scalar export header (generated).\n");
    s.push_str("// Fallible form: 0 = success (out/reason committed), negative =\n");
    s.push_str("// mapped status (reason carries the original trap code).\n");
    s.push_str("#ifndef A2X_ABI_EXPORT_H\n#define A2X_ABI_EXPORT_H\n\n");
    s.push_str("#include <stdint.h>\n#include <stdbool.h>\n\n");
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
    inputs: &LibraryFilesetInputs,
) -> Result<FileSet, String> {
    for spec in specs {
        spec.check()?;
    }
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
    // generated.c: header comment + runtime include + forward
    // declarations (each fragment's prototype line) + definitions.
    let mut generated = String::new();
    generated.push_str(&format!(
        "// PLAN-937 CA-D1 library form (module {}, entry {})\n",
        inputs.module, inputs.entry
    ));
    generated.push_str("#include \"a2x_c_runtime.h\"\n\n");
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
    let library_c = wrapper_c(specs);
    let header = public_header(specs);
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
            params: vec![LaneKind::I32, LaneKind::I32],
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
            params: vec![LaneKind::I32],
            ret: LaneKind::Bool,
        };
        assert_eq!(
            public_prototype(&b),
            "int32_t t937_flag(void* ctx, int32_t p0, bool* out, int32_t* reason)"
        );
    }

    #[test]
    fn wrapper_maps_traps_by_enum_not_by_string_or_exit() {
        let c = wrapper_c(&[sum_spec()]);
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
        let fs = assemble_library_fileset(&[frag], "RT", &[sum_spec()], &inputs).unwrap();
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
        let fs2 = assemble_library_fileset(&[frag], "RT", &[sum_spec()], &inputs).unwrap();
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
            assemble_library_fileset(&[frag2.as_str()], "RT", &[sum_spec()], &inputs).unwrap();
        assert_ne!(fs.manifest, fs3.manifest);
    }

    #[test]
    fn lane_restriction_enforced() {
        let bad = ExportSpec {
            public_symbol: "x".into(),
            source_fn: "fn_x".into(),
            params: vec![LaneKind::F64],
            ret: LaneKind::I32,
        };
        let err = match assemble_library_fileset(
            &["// origin: [0,1]\na2x_result fn_x(double a);"],
            "R",
            &[bad],
            &LibraryFilesetInputs {
                entry: "e".into(),
                input_hash: "i".into(),
                module: "m".into(),
            },
        ) {
            Ok(_) => panic!("f64 lane must be rejected for the source channel"),
            Err(e) => e,
        };
        assert!(err.contains("source channel"), "{err}");
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
