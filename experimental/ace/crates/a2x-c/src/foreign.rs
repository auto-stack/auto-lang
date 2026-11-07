//! PLAN-938: HIR-consuming source foreign import, independent of native/MIR.
use ace_a2x_core::{
    target::{Family, FileSet, FilesetRequest, FragmentText, TargetDescriptor, TargetProvider},
    A2xReject, FnFacts, SemanticView,
};
use ace_foreign_contract::{
    manual::{AdmittedForeign, PageProduction, SourceDecl},
    resource::{FieldKind, ResKind},
    FnForm, LaneKind,
};
use std::sync::{Arc, Mutex};

pub const SCHEMA: &str = "auto.a2x.c.foreign-library.fileset/1";
pub const RECIPE: &str = "ac-a2c/hir-foreign-scalar-import-v1";
/// PLAN-939: the record-text RESOURCE import recipe (a distinct identity —
/// a scalar-import fileset never settles it and vice versa).
pub const RESOURCE_RECIPE: &str = "ac-a2c/hir-foreign-resource-import-v1";
pub const FAILURE_RUNTIME: &str = "#include <stdint.h>\n#include <stdio.h>\n#include <stdlib.h>\nvoid ace_foreign_fail(int32_t status) { fprintf(stderr, \"foreign.status=%ld\\n\", (long)status); fflush(stderr); exit(70); }\n";

/// Host admission of a produced text's exact byte size (the request's
/// cumulative output account plus the live memory ticket). Shared behind an
/// `Arc<Mutex<..>>` because the neutral `TargetProvider` seam is `&self`.
pub type SharedTextProduction = Arc<Mutex<dyn PageProduction + Send>>;

fn lane_ty(k: LaneKind) -> &'static str {
    if k == LaneKind::Bool {
        "uint8_t"
    } else {
        "int32_t"
    }
}

/// The ONE prototype formatter (counting pass allocates nothing).
pub fn write_prototype(
    declaration: &SourceDecl,
    out: &mut dyn std::fmt::Write,
) -> std::fmt::Result {
    let fallible = matches!(declaration.function.form, FnForm::Fallible { .. });
    out.write_str("extern ")?;
    out.write_str(if fallible {
        "int32_t"
    } else {
        lane_ty(declaration.result())
    })?;
    out.write_str(" ")?;
    out.write_str(&declaration.function.symbol)?;
    out.write_str("(")?;
    let mut first = true;
    let mut push = |out: &mut dyn std::fmt::Write, text: &str| -> std::fmt::Result {
        if first {
            first = false;
        } else {
            out.write_str(", ")?;
        }
        out.write_str(text)
    };
    if declaration.context_symbol.is_some() {
        push(out, "void *")?;
    }
    for &k in declaration.params() {
        push(out, lane_ty(k))?;
    }
    if fallible {
        // The fallible out-param is always last: its lane then the pointer
        // star, without a temporary formatted String in the counting pass.
        push(out, lane_ty(declaration.result()))?;
        out.write_str(" *")?;
    }
    if first {
        out.write_str("void")?;
    }
    out.write_str(");")
}

pub fn prototype(declaration: &SourceDecl) -> String {
    let mut out = String::new();
    write_prototype(declaration, &mut out).expect("formatting into a String cannot fail");
    out
}

/// PLAN-939: the resource-entry C prototype. Provider-side nominal types
/// are spelled from the manifest record ids (the hand-audited header
/// defines them; the manifest field closure plus the core nominal-shape
/// check are the layout guarantee). The value out is the trailing
/// pointer; the i32 status RETURNS (the 938 convention).
pub fn resource_prototype(declaration: &SourceDecl) -> String {
    use ace_foreign_contract::resource::{ResKind, ResOut, ResourceForm, ViewRole};
    let Some(res) = declaration.resource() else {
        return prototype(declaration);
    };
    res.validate().expect("admitted resource signature");
    let mut out = String::new();
    let ret_c = match &res.form {
        ResourceForm::Pure { ret } => match ret {
            ResOut::I32 => "int32_t".to_string(),
            ResOut::Bool => "uint8_t".to_string(),
            ResOut::I64 | ResOut::Handle { .. } => "long long".to_string(),
            ResOut::U64 => "unsigned long long".to_string(),
        },
        ResourceForm::Fallible { .. } => "int32_t".to_string(),
    };
    out.push_str(&format!(
        "extern {ret_c} {}(",
        declaration.function.symbol
    ));
    let mut first = true;
    let mut push = |out: &mut String, text: &str| {
        if first {
            first = false;
        } else {
            out.push_str(", ");
        }
        out.push_str(text);
    };
    if let ResourceForm::Fallible { ctx: Some(_), .. } = res.form {
        push(&mut out, "void *");
    }
    for (i, p) in res.params.iter().enumerate() {
        let after_cb = i > 0 && matches!(res.params[i - 1], ResKind::Callback { ctx: true });
        match p {
            ResKind::I32 if after_cb => push(&mut out, "long long"),
            ResKind::I32 => push(&mut out, "int32_t"),
            ResKind::Bool => push(&mut out, "uint8_t"),
            ResKind::ViewText => {
                push(&mut out, "const uint8_t *");
                push(&mut out, "uint64_t");
            }
            ResKind::Rec { rec, role, .. } => match role {
                ViewRole::Const => push(&mut out, &format!("const {rec} *")),
                ViewRole::Mut | ViewRole::Out => push(&mut out, &format!("{rec} *")),
            },
            ResKind::Handle { .. } => push(&mut out, "long long"),
            ResKind::Callback { .. } => push(&mut out, "long long (*cb)(long long, long long)"),
        }
    }
    if let ResourceForm::Fallible {
        out: Some(lane), ..
    } = &res.form
    {
        let c = match lane {
            ResOut::I32 => "int32_t *",
            ResOut::Bool => "uint8_t *",
            ResOut::I64 | ResOut::U64 | ResOut::Handle { .. } => "long long *",
        };
        push(&mut out, c);
    }
    out.push_str(");");
    out
}

/// A genuine library file set, not a main-stripped executable. It includes all
/// business functions, checked arithmetic runtime, imported prototypes, and the
/// status transport runtime. Provider object bytes are a host link-root input.
pub fn assemble_library(
    view: &SemanticView<'_>,
    environment: &AdmittedForeign,
    input_hash: &str,
) -> Result<FileSet, A2xReject> {
    let provider = ForeignTargetProvider::new(environment.clone());
    let fns: Vec<_> = view.functions().iter().collect();
    let texts = provider.produce_fragments(view, &fns, Family::Scalar)?;
    let fingerprints = fns
        .iter()
        .map(|f| ace_a2x_core::prepare::semantic_fingerprint(view, f))
        .collect::<Result<Vec<_>, _>>()?;
    let fragments = fns
        .iter()
        .zip(&texts)
        .zip(&fingerprints)
        .map(|((f, text), sem_fp)| FragmentText {
            def_index: f.def_index,
            source_name: f.source_name,
            sem_fp,
            text,
        })
        .collect::<Vec<_>>();
    let runtime = provider.runtime_source(view, Family::Scalar)?;
    provider.assemble_fileset(FilesetRequest {
        entry: "main",
        input_hash,
        family: Family::Scalar,
        fragments: &fragments,
        view: Some(view),
        runtime: &runtime,
        prelude_seed: None,
        origins: None,
        source_sha256: None,
    })
}

/// Current authority is process-local provider state, never a persisted safe
/// marker. TargetProvider remains the common C cold/fragment/assembly seam.
pub struct ForeignTargetProvider {
    environment: AdmittedForeign,
    production: Option<SharedTextProduction>,
}
impl ForeignTargetProvider {
    pub fn new(environment: AdmittedForeign) -> Self {
        Self {
            environment,
            production: None,
        }
    }
    /// PLAN-938: exact text-output admission against the host's request
    /// account for every fragment/runtime/fileset text this provider
    /// produces. Callers that hold their own budgets keep `new`.
    pub fn with_production(environment: AdmittedForeign, production: SharedTextProduction) -> Self {
        Self {
            environment,
            production: Some(production),
        }
    }
    fn reserve_text(&self, size: usize) -> Result<(), String> {
        match &self.production {
            Some(policy) => policy.lock().expect("text production policy").reserve(size),
            None => Ok(()),
        }
    }
    fn complete_text(&self, size: usize) -> Result<(), String> {
        match &self.production {
            Some(policy) => policy
                .lock()
                .expect("text production policy")
                .complete(size),
            None => Ok(()),
        }
    }
    fn admit(&self, view: &SemanticView<'_>) -> Result<(), A2xReject> {
        let resource_env = resource_environment(&self.environment);
        let domain_ok = if resource_env {
            view.domain == ace_core::hir::HirDomain::TextRecord
        } else {
            view.domain == ace_core::hir::HirDomain::Scalar
        };
        if !domain_ok || view.foreign_declarations().next().is_none() {
            return Err(A2xReject::new(
                "a2c.foreign-profile",
                if resource_env {
                    "expected checked text-record resource foreign HIR"
                } else {
                    "expected checked scalar foreign HIR"
                },
            ));
        }
        for d in view.foreign_declarations() {
            if !self.environment.admits_declaration(d) {
                return Err(A2xReject::new(
                    "a2c.foreign-authority",
                    "foreign declaration is not admitted by the current host",
                ));
            }
        }
        Ok(())
    }
    /// The admitted runtime entry for the source host: same bytes as the
    /// trait's `runtime_source`, with the exact size admitted by `admit`
    /// BEFORE the single exactly-sized allocation.
    pub fn runtime_source_admitted(
        &self,
        view: &SemanticView<'_>,
        family: Family,
        admit: impl FnOnce(usize) -> Result<(), String>,
    ) -> Result<String, A2xReject> {
        family_admits(family)?;
        self.admit(view)?;
        let marks = view.trace_symbols().into_iter().collect::<Vec<_>>();
        match family {
            Family::RecordText => {
                crate::rt_runtime::record_text_runtime_source_admitted(&marks, admit)
                    .map_err(|e| A2xReject::new("a2c.foreign-resource", e))
            }
            _ => crate::runtime::runtime_source_admitted(&marks, admit)
                .map_err(|e| A2xReject::new("a2c.foreign-resource", e)),
        }
    }
}

/// PLAN-939: the foreign C channel also admits the RECORD-TEXT family —
/// RESOURCE foreign (str views, nominal records, handles, callbacks)
/// rides the 925 record-text lowering with resource-aware marshaling.
fn family_admits(family: Family) -> Result<(), A2xReject> {
    match family {
        Family::Scalar | Family::RecordText => Ok(()),
        Family::Collections => Err(A2xReject::new(
            "a2c.foreign-profile",
            "foreign is admitted on the scalar and record-text families only",
        )),
    }
}

/// True when the environment carries at least one resource entry.
fn resource_environment(environment: &AdmittedForeign) -> bool {
    environment
        .source_declarations_ref()
        .iter()
        .any(|d| d.resource().is_some())
}
impl TargetProvider for ForeignTargetProvider {
    fn validate_view(&self, view: &SemanticView<'_>) -> Result<(), A2xReject> {
        self.admit(view)
    }
    fn descriptor(&self) -> TargetDescriptor {
        TargetDescriptor {
            lang: "c",
            standard: "c11",
            backend: "auto-a2x-c-foreign",
            backend_version: "1",
        }
    }
    fn fragment_key_inputs(&self, runtime_sha256: &str) -> Vec<(String, String)> {
        vec![
            ("lang".into(), "c".into()),
            ("standard".into(), "c11".into()),
            ("backend".into(), "auto-a2x-c-foreign".into()),
            ("backend_version".into(), "1".into()),
            ("runtime_sha256".into(), runtime_sha256.into()),
            ("print_version".into(), "a2x-c-foreign-print/1".into()),
        ]
    }
    fn fragment_layer_id(&self, family: Family) -> Result<&'static str, A2xReject> {
        family_admits(family)?;
        Ok(match family {
            Family::RecordText => "AXCFR1",
            _ => "AXCF1",
        })
    }
    fn runtime_source(&self, view: &SemanticView<'_>, family: Family) -> Result<String, A2xReject> {
        family_admits(family)?;
        self.admit(view)?;
        match family {
            Family::RecordText => Ok(crate::rt_runtime::record_text_runtime_source(
                &view.trace_symbols().into_iter().collect::<Vec<_>>(),
            )),
            _ => Ok(crate::runtime::runtime_source(
                &view.trace_symbols().into_iter().collect::<Vec<_>>(),
            )),
        }
    }
    fn produce_fragments(
        &self,
        view: &SemanticView<'_>,
        fns: &[&FnFacts<'_>],
        family: Family,
    ) -> Result<Vec<String>, A2xReject> {
        family_admits(family)?;
        self.admit(view)?;
        match family {
            Family::RecordText => {
                let cfns = crate::lower_rt::lower_functions_rt(view, fns)?;
                let mut out = Vec::with_capacity(cfns.len());
                for cfn in &cfns {
                    let text = crate::lower_rt::print_fragment_rt(cfn);
                    self.reserve_text(text.len())
                        .map_err(|e| A2xReject::new("a2c.foreign-resource", e))?;
                    self.complete_text(text.len())
                        .map_err(|e| A2xReject::new("a2c.foreign-resource", e))?;
                    out.push(text);
                }
                Ok(out)
            }
            _ => fns
                .iter()
                .map(|f| {
                    crate::lower::lower_foreign_fn_c(view, f).and_then(|f| {
                        let text = crate::lower::print_fragment_admitted(&f, |size| {
                            self.reserve_text(size)
                        })?;
                        self.complete_text(text.len())
                            .map_err(|e| A2xReject::new("a2c.foreign-resource", e))?;
                        Ok(text)
                    })
                })
                .collect(),
        }
    }
    fn capture_prelude_seed(
        &self,
        view: &SemanticView<'_>,
        family: Family,
    ) -> Result<Option<String>, A2xReject> {
        family_admits(family)?;
        self.admit(view)?;
        match family {
            Family::RecordText => crate::seed::encode_seed(view).map(Some),
            _ => Ok(None),
        }
    }
    fn assemble_fileset(&self, req: FilesetRequest<'_>) -> Result<FileSet, A2xReject> {
        self.assemble_fileset_admitted(req, &mut (), &mut |_| Ok(()))
    }
}

const GENERATED_HEAD: &str = "#include \"a2x_c_runtime.h\"\n#include \"foreign_imports.h\"\n";
const IMPORTS_HEAD: &str =
    "#ifndef ACE_FOREIGN_IMPORTS_H\n#define ACE_FOREIGN_IMPORTS_H\n#include <stdint.h>\n";
const IMPORTS_TAIL: &str = "void ace_foreign_fail(int32_t status);\n#endif\n";

/// The fileset manifest, borrowed. Field order is the historical
/// `serde_json::json!` object's insertion order (this workspace pins serde_json
/// `preserve_order`), so the manifest keeps its exact historical bytes without
/// building a `Value` tree. The assembler test pins both against `json!`.
#[derive(serde::Serialize)]
struct FilesetManifest<'a> {
    schema: &'a str,
    recipe: &'a str,
    direction: &'a str,
    input_profile: &'a str,
    input_hash: &'a str,
    contract: &'a str,
    target: &'a str,
    functions: Vec<FunctionEntry<'a>>,
    files: Vec<FileEntry<'a>>,
}
#[derive(serde::Serialize)]
struct FileEntry<'a> {
    path: &'a str,
    sha256: String,
}
#[derive(serde::Serialize)]
struct FunctionEntry<'a> {
    name: &'a str,
    semantic: &'a str,
}

fn exact_copy(text: &str) -> Result<String, A2xReject> {
    let mut out = String::new();
    out.try_reserve_exact(text.len())
        .map_err(|e| A2xReject::new("a2c.foreign-resource", format!("text allocation: {e}")))?;
    out.push_str(text);
    Ok(out)
}

fn write_generated(
    fragments: &[FragmentText<'_>],
    record_table: Option<&str>,
    out: &mut dyn std::fmt::Write,
) -> std::fmt::Result {
    out.write_str(GENERATED_HEAD)?;
    if let Some(block) = record_table {
        // PLAN-939: the record-text fragments reference `a2x_rec_<Name>`
        // internal record types; the seed-derived record table makes the
        // translation unit self-contained (same block the 925 session
        // assembly emits).
        out.write_str(block)?;
    }
    for fragment in fragments {
        // Validated before any counting pass; see `assemble_fileset_admitted`.
        out.write_str(fragment.text.lines().nth(1).unwrap_or_default())?;
        out.write_str("\n")?;
    }
    for fragment in fragments {
        out.write_str(fragment.text)?;
        out.write_str("\n")?;
    }
    Ok(())
}

fn write_imports(declarations: &[SourceDecl], out: &mut dyn std::fmt::Write) -> std::fmt::Result {
    out.write_str(IMPORTS_HEAD)?;
    // PLAN-939: the imports header is SELF-CONTAINED. Public record
    // closures (naturally aligned scalar layouts) are emitted from the
    // reviewed resource signatures BEFORE any prototype references them —
    // the provider's own header defines layout-identical types in its own
    // translation unit (C11 6.2.7 compatible across TUs); nothing in this
    // TU may include the provider's private header.
    let mut records: Vec<(&str, &[(String, FieldKind)])> = Vec::new();
    for d in declarations {
        let Some(res) = d.resource() else {
            continue;
        };
        for p in &res.params {
            if let ResKind::Rec { rec, fields, .. } = p {
                match records.iter().find(|(name, _)| *name == rec.as_str()) {
                    Some((_, known)) if *known == fields.as_slice() => {}
                    Some(_) => {
                        // A conflicting closure for one public record name
                        // is an unreviewed ABI, not a C layout choice.
                        return Err(std::fmt::Error);
                    }
                    None => records.push((rec.as_str(), fields.as_slice())),
                }
            }
        }
    }
    for (rec, fields) in &records {
        out.write_str("typedef struct {")?;
        for (name, kind) in *fields {
            write!(out, " {} {};", field_c_type(*kind), name)?;
        }
        write!(out, " }} {rec};\n")?;
    }
    for d in declarations {
        if d.resource().is_some() {
            out.write_str(&resource_prototype(d))?;
        } else {
            write_prototype(d, out)?;
        }
        out.write_str("\n")?;
        if let Some(symbol) = &d.context_symbol {
            write!(out, "extern unsigned char {symbol};\n")?;
        }
    }
    out.write_str(IMPORTS_TAIL)
}

/// The public record-closure field's C scalar (§5.1: natural alignment, no
/// padding surprises — the physical table digest carries the same layout).
fn field_c_type(kind: FieldKind) -> &'static str {
    match kind {
        FieldKind::I32 => "int32_t",
        FieldKind::Bool => "uint8_t",
        FieldKind::Char => "uint32_t",
        FieldKind::U32 => "uint32_t",
        FieldKind::I64 => "long long",
        FieldKind::U64 => "unsigned long long",
        FieldKind::F64 => "double",
    }
}

impl ForeignTargetProvider {
    /// A genuine library file set with exact production admission. New
    /// products (`generated.c`, `foreign_imports.h`, the manifest) reserve
    /// their exact byte size on the request output account BEFORE allocation;
    /// the byte-identical runtime header copy and the fixed status runtime are
    /// live memory only, never re-produced output. `admit_memory` is asked
    /// for the live text bytes before they exist (the texts, then the
    /// manifest with its per-file digests). Working metadata of the manifest
    /// entries is bounded by the fragment count and declared with the texts.
    pub fn assemble_fileset_admitted(
        &self,
        req: FilesetRequest<'_>,
        output: &mut dyn PageProduction,
        admit_memory: &mut dyn FnMut(usize) -> Result<(), String>,
    ) -> Result<FileSet, A2xReject> {
        family_admits(req.family)?;
        if let Some(view) = req.view {
            self.admit(view)?;
        }
        let environment = &self.environment;
        let fragments = req.fragments;
        for fragment in fragments {
            fragment.text.lines().nth(1).ok_or_else(|| {
                A2xReject::new("a2c.foreign-fragment", "missing function prototype")
            })?;
        }
        let resource = |e: String| A2xReject::new("a2c.foreign-resource", e);
        let declarations = environment.source_declarations_ref();
        // PLAN-939: the record-text family's fragments reference internal
        // `a2x_rec_<Name>` types — the record table comes from the prelude
        // seed (cold capture or restore anchor) or the live view; the
        // scalar family needs none.
        let record_table = if req.family == Family::RecordText {
            let seed_text = match (req.prelude_seed, req.view) {
                (Some(s), _) => s.to_string(),
                (None, Some(v)) => crate::seed::encode_seed(v)?,
                (None, None) => {
                    return Err(A2xReject::new(
                        "a2c.foreign-resource",
                        "record-text fileset needs the record table (prelude seed or live view)",
                    ))
                }
            };
            let recs = crate::seed::decode_seed(&seed_text)?;
            Some(crate::provider::record_table_block(&recs)?)
        } else {
            None
        };
        let generated_size =
            crate::admitted_text::text_len(|out| write_generated(fragments, record_table.as_deref(), out));
        let imports_size = crate::admitted_text::text_len(|out| write_imports(declarations, out));
        let string_header = std::mem::size_of::<String>();
        let live_texts = generated_size
            .checked_add(imports_size)
            .and_then(|n| n.checked_add(req.runtime.len()))
            .and_then(|n| n.checked_add(FAILURE_RUNTIME.len()))
            .and_then(|n| n.checked_add(4 * (2 * string_header + 16 + 64)))
            .and_then(|n| n.checked_add(fragments.len().checked_mul(2 * string_header)?))
            .and_then(|n| n.checked_add(record_table.as_deref().map(str::len).unwrap_or(0)))
            .ok_or_else(|| resource("fileset size overflow".into()))?;
        admit_memory(live_texts).map_err(resource)?;
        let generated = crate::admitted_text::render_admitted(
            |out| write_generated(fragments, record_table.as_deref(), out),
            |size| output.reserve(size),
        )
        .map_err(resource)?;
        output.complete(generated.len()).map_err(resource)?;
        let imports = crate::admitted_text::render_admitted(
            |out| write_imports(declarations, out),
            |size| output.reserve(size),
        )
        .map_err(resource)?;
        output.complete(imports.len()).map_err(resource)?;
        let runtime = exact_copy(req.runtime)?;
        let failure = exact_copy(FAILURE_RUNTIME)?;
        let files = vec![
            ("generated.c".to_string(), generated),
            ("a2x_c_runtime.h".to_string(), runtime),
            ("foreign_imports.h".to_string(), imports),
            ("foreign_runtime.c".to_string(), failure),
        ];
        let manifest = FilesetManifest {
            contract: environment.contract_digest(),
            direction: "source-import",
            files: files
                .iter()
                .map(|(path, text)| FileEntry {
                    path,
                    sha256: ace_a2x_core::sha256::sha256_hex(text.as_bytes()),
                })
                .collect(),
            functions: fragments
                .iter()
                .map(|f| FunctionEntry {
                    name: f.source_name,
                    semantic: f.sem_fp,
                })
                .collect(),
            input_hash: req.input_hash,
            input_profile: if req.family == Family::RecordText {
                ace_core::source::TEXT_RECORD_SOURCE_PROFILE
            } else {
                ace_core::source::SCALAR_SOURCE_PROFILE
            },
            recipe: if req.family == Family::RecordText {
                RESOURCE_RECIPE
            } else {
                RECIPE
            },
            schema: SCHEMA,
            target: &environment.manifest().target,
        };
        let manifest_size = ace_foreign_contract::codec::json_size(
            &manifest,
            ace_foreign_contract::codec::MAX_CODEC_BYTES,
        )
        .map_err(|e| resource(e.to_string()))?;
        admit_memory(manifest_size + string_header).map_err(resource)?;
        let manifest = ace_foreign_contract::codec::encode_json_admitted(
            &manifest,
            ace_foreign_contract::codec::MAX_CODEC_BYTES,
            |size| {
                output
                    .reserve(size)
                    .map_err(ace_foreign_contract::codec::CodecError)
            },
        )
        .map_err(|e| resource(e.to_string()))?;
        output.complete(manifest.len()).map_err(resource)?;
        let manifest = String::from_utf8(manifest)
            .map_err(|e| A2xReject::new("a2x.internal", format!("manifest utf-8: {e}")))?;
        Ok(FileSet { files, manifest })
    }
}
