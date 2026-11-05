//! PLAN-938: HIR-consuming source foreign import, independent of native/MIR.
use ace_a2x_core::{
    target::{Family, FileSet, FilesetRequest, FragmentText, TargetDescriptor, TargetProvider},
    A2xReject, FnFacts, SemanticView,
};
use ace_foreign_contract::{
    manual::{AdmittedForeign, SourceDecl},
    FnForm, LaneKind,
};

pub const SCHEMA: &str = "auto.a2x.c.foreign-library.fileset/1";
pub const RECIPE: &str = "ac-a2c/hir-foreign-scalar-import-v1";
pub const FAILURE_RUNTIME: &str = "#include <stdint.h>\n#include <stdio.h>\n#include <stdlib.h>\nvoid ace_foreign_fail(int32_t status) { fprintf(stderr, \"foreign.status=%ld\\n\", (long)status); fflush(stderr); exit(70); }\n";

pub fn prototype(declaration: &SourceDecl) -> String {
    let lane = |k| {
        if k == LaneKind::Bool {
            "uint8_t"
        } else {
            "int32_t"
        }
    };
    let mut params = Vec::new();
    if declaration.context_symbol.is_some() {
        params.push("void *".to_string());
    }
    params.extend(declaration.params().iter().map(|k| lane(*k).to_string()));
    let result = if matches!(declaration.function.form, FnForm::Fallible { .. }) {
        params.push(format!("{} *", lane(declaration.result())));
        "int32_t"
    } else {
        lane(declaration.result())
    };
    if params.is_empty() {
        params.push("void".into());
    }
    format!(
        "extern {result} {}({});",
        declaration.function.symbol,
        params.join(", ")
    )
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
}
impl ForeignTargetProvider {
    pub fn new(environment: AdmittedForeign) -> Self {
        Self { environment }
    }
    fn admit(&self, view: &SemanticView<'_>) -> Result<(), A2xReject> {
        if view.domain != ace_core::hir::HirDomain::Scalar
            || view.foreign_declarations().next().is_none()
        {
            return Err(A2xReject::new(
                "a2c.foreign-profile",
                "expected checked scalar foreign HIR",
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
}
fn scalar(family: Family) -> Result<(), A2xReject> {
    if family == Family::Scalar {
        Ok(())
    } else {
        Err(A2xReject::new(
            "a2c.foreign-profile",
            "only scalar foreign is admitted",
        ))
    }
}
impl TargetProvider for ForeignTargetProvider {
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
        scalar(family)?;
        Ok("AXCF1")
    }
    fn runtime_source(&self, view: &SemanticView<'_>, family: Family) -> Result<String, A2xReject> {
        scalar(family)?;
        self.admit(view)?;
        Ok(crate::runtime::runtime_source(
            &view.trace_symbols().into_iter().collect::<Vec<_>>(),
        ))
    }
    fn produce_fragments(
        &self,
        view: &SemanticView<'_>,
        fns: &[&FnFacts<'_>],
        family: Family,
    ) -> Result<Vec<String>, A2xReject> {
        scalar(family)?;
        self.admit(view)?;
        fns.iter()
            .map(|f| {
                crate::lower::lower_foreign_fn_c(view, f).map(|f| crate::lower::print_fragment(&f))
            })
            .collect()
    }
    fn capture_prelude_seed(
        &self,
        view: &SemanticView<'_>,
        family: Family,
    ) -> Result<Option<String>, A2xReject> {
        scalar(family)?;
        self.admit(view)?;
        Ok(None)
    }
    fn assemble_fileset(&self, req: FilesetRequest<'_>) -> Result<FileSet, A2xReject> {
        scalar(req.family)?;
        if let Some(view) = req.view {
            self.admit(view)?;
        }
        let environment = &self.environment;
        let fragments = req.fragments;
        let functions = fragments
            .iter()
            .map(|f| serde_json::json!({"name":f.source_name,"semantic":f.sem_fp}))
            .collect::<Vec<_>>();
        let mut generated =
            "#include \"a2x_c_runtime.h\"\n#include \"foreign_imports.h\"\n".to_string();
        for fragment in fragments {
            generated.push_str(fragment.text.lines().nth(1).ok_or_else(|| {
                A2xReject::new("a2c.foreign-fragment", "missing function prototype")
            })?);
            generated.push('\n');
        }
        for fragment in fragments {
            generated.push_str(fragment.text);
            generated.push('\n');
        }
        let mut imports =
            "#ifndef ACE_FOREIGN_IMPORTS_H\n#define ACE_FOREIGN_IMPORTS_H\n#include <stdint.h>\n"
                .to_string();
        for d in environment.source_declarations() {
            imports.push_str(&prototype(&d));
            imports.push('\n');
            if let Some(symbol) = &d.context_symbol {
                imports.push_str(&format!("extern unsigned char {symbol};\n"));
            }
        }
        imports.push_str("void ace_foreign_fail(int32_t status);\n#endif\n");
        let files = vec![
            ("generated.c".into(), generated),
            ("a2x_c_runtime.h".into(), req.runtime.to_string()),
            ("foreign_imports.h".into(), imports),
            ("foreign_runtime.c".into(), FAILURE_RUNTIME.into()),
        ];
        let manifest = serde_json::json!({"schema":SCHEMA,"recipe":RECIPE,"direction":"source-import","input_profile":ace_core::source::SCALAR_SOURCE_PROFILE,
        "input_hash":req.input_hash,"contract":environment.contract_digest(),"target":environment.manifest().target,
        "functions":functions,"files":files.iter().map(|(path,text): &(String,String)| serde_json::json!({"path":path,"sha256":ace_a2x_core::sha256::sha256_hex(text.as_bytes())})).collect::<Vec<_>>()}).to_string();
        Ok(FileSet { files, manifest })
    }
}
