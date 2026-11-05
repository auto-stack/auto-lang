//! PLAN-938: HIR-consuming source foreign import, independent of native/MIR.
use ace_a2x_core::{target::FileSet, A2xReject, SemanticView};
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
    if view.domain != ace_core::hir::HirDomain::Scalar
        || view.foreign_declarations().next().is_none()
    {
        return Err(A2xReject::new(
            "a2c.foreign-profile",
            "expected checked scalar foreign HIR",
        ));
    }
    for d in view.foreign_declarations() {
        if !environment.admits_declaration(d) {
            return Err(A2xReject::new(
                "a2c.foreign-authority",
                "foreign declaration is not admitted by the current host",
            ));
        }
    }
    let mut functions = Vec::new();
    let mut fragments = Vec::new();
    for facts in view.functions() {
        let f = crate::lower::lower_foreign_fn_c(view, facts)?;
        functions.push(serde_json::json!({"name":facts.source_name,"semantic":ace_a2x_core::prepare::semantic_fingerprint(view,facts)?}));
        fragments.push(crate::lower::print_fragment(&f));
    }
    let mut generated =
        "#include \"a2x_c_runtime.h\"\n#include \"foreign_imports.h\"\n".to_string();
    for fragment in &fragments {
        generated.push_str(
            fragment.lines().nth(1).ok_or_else(|| {
                A2xReject::new("a2c.foreign-fragment", "missing function prototype")
            })?,
        );
        generated.push('\n');
    }
    for fragment in &fragments {
        generated.push_str(fragment);
        generated.push('\n');
    }
    let mut imports =
        "#ifndef ACE_FOREIGN_IMPORTS_H\n#define ACE_FOREIGN_IMPORTS_H\n#include <stdint.h>\n"
            .to_string();
    for d in view.foreign_declarations() {
        imports.push_str(&prototype(d));
        imports.push('\n');
        if let Some(symbol) = &d.context_symbol {
            imports.push_str(&format!("extern unsigned char {symbol};\n"));
        }
    }
    imports.push_str("void ace_foreign_fail(int32_t status);\n#endif\n");
    let marks: Vec<_> = view.trace_symbols().into_iter().collect();
    let files = vec![
        ("generated.c".into(), generated),
        (
            "a2x_c_runtime.h".into(),
            crate::runtime::runtime_source(&marks),
        ),
        ("foreign_imports.h".into(), imports),
        ("foreign_runtime.c".into(), FAILURE_RUNTIME.into()),
    ];
    let manifest = serde_json::json!({"schema":SCHEMA,"recipe":RECIPE,"direction":"source-import","input_profile":view.profile,
        "input_hash":input_hash,"contract":environment.contract_digest(),"target":environment.manifest().target,
        "functions":functions,"files":files.iter().map(|(path,text): &(String,String)| serde_json::json!({"path":path,"sha256":ace_a2x_core::sha256::sha256_hex(text.as_bytes())})).collect::<Vec<_>>()}).to_string();
    Ok(FileSet { files, manifest })
}
