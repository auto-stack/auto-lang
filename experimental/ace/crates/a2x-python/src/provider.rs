//! PLAN-1042: `PythonTargetProvider` — shared A2X `TargetProvider` for Python 3
//! across Scalar (`AXPY1`), RecordText (`AXPY2`), and Collections (`AXPY3`).

use ace_a2x_core::{
    A2xReject, Family, FileSet, FilesetRequest, FnFacts, SemanticView, TargetDescriptor,
    TargetProvider,
};
use ace_core::runtime_contract::sha256_hex;

use crate::lower::{lower_collections_fn_py, lower_functions_py, lower_rt_fn_py, print_fragment};
use crate::ports::default_stdlib_port_catalog;
use crate::print::{print_program_py, PY_PRINT_VERSION};
use crate::runtime::{python_runtime_source, PY_RUNTIME_VERSION};
use crate::seed::{
    decode_coll_seed, decode_rt_seed, encode_coll_seed, encode_rt_seed, render_coll_seed_helpers,
    render_rt_seed_helpers,
};

/// Backend identifier registered in `a2x-store` and manifests.
pub const PY_BACKEND: &str = "auto-a2x-python";
/// Backend provider version.
pub const PY_BACKEND_VERSION: &str = "0.1.0";
/// Target Python language level contract.
pub const PY_STANDARD: &str = "python3.11+";
/// Worker/module-bundle producer identity accepted by `semantics-contract`.
pub const PY_PRODUCER: &str = "ace-a2x-python/modules/1";

/// FileSet manifest schema for Python Scalar.
pub const PY_FILESET_SCHEMA: &str = "ace.a2py-hir.scalar.fileset/1";
/// FileSet manifest schema for Python RecordText.
pub const PY_RT_FILESET_SCHEMA: &str = "ace.a2py-hir.record-text.fileset/1";
/// FileSet manifest schema for Python Collections.
pub const PY_COLL_FILESET_SCHEMA: &str = "ace.a2py-hir.collections.fileset/1";

/// Recipe identifier for Python Scalar certification.
pub const PY_RECIPE: &str = "a2py-hir-python-3.11-checked";
/// Recipe identifier for Python RecordText certification.
pub const PY_RT_RECIPE: &str = "a2py-hir-record-text-python-3.11";
/// Recipe identifier for Python Collections certification.
pub const PY_COLL_RECIPE: &str = "a2py-hir-collections-python-3.11";

/// Shared A2X target provider emitting standalone Python 3 filesets from Checked HIR.
#[derive(Clone, Copy, Debug, Default)]
pub struct PythonTargetProvider;

impl PythonTargetProvider {
    fn family_schema(family: Family) -> Result<&'static str, A2xReject> {
        match family {
            Family::Scalar => Ok(PY_FILESET_SCHEMA),
            Family::RecordText => Ok(PY_RT_FILESET_SCHEMA),
            Family::Collections => Ok(PY_COLL_FILESET_SCHEMA),
        }
    }
}

impl TargetProvider for PythonTargetProvider {
    fn validate_view(&self, view: &SemanticView<'_>) -> Result<(), A2xReject> {
        if view.foreign_declarations().next().is_some() {
            return Err(A2xReject::new(
                "a2py.foreign-authority",
                "foreign FFI declarations require explicit C/Rust FFI provider",
            ));
        }
        Ok(())
    }

    fn descriptor(&self) -> TargetDescriptor {
        TargetDescriptor {
            lang: "python",
            standard: PY_STANDARD,
            backend: PY_BACKEND,
            backend_version: PY_BACKEND_VERSION,
        }
    }

    fn fragment_key_inputs(&self, runtime_sha256: &str) -> Vec<(String, String)> {
        let catalog = default_stdlib_port_catalog(PY_RUNTIME_VERSION);
        vec![
            (
                "runtime_version".to_string(),
                PY_RUNTIME_VERSION.to_string(),
            ),
            ("runtime_sha256".to_string(), runtime_sha256.to_string()),
            ("print_version".to_string(), PY_PRINT_VERSION.to_string()),
            (
                "stdlib_port_catalog".to_string(),
                catalog.canonical_fingerprint(),
            ),
        ]
    }

    fn fragment_layer_id(&self, family: Family) -> Result<&'static str, A2xReject> {
        match family {
            Family::Scalar => Ok("AXPY1"),
            Family::RecordText => Ok("AXPY2"),
            Family::Collections => Ok("AXPY3"),
        }
    }

    fn runtime_source(
        &self,
        _view: &SemanticView<'_>,
        family: Family,
    ) -> Result<String, A2xReject> {
        let marks: &[&'static str] = match family {
            Family::Scalar | Family::RecordText | Family::Collections => {
                &["hir.test.mark_a", "hir.test.mark_b"]
            }
        };
        Ok(python_runtime_source(marks))
    }

    fn produce_fragments(
        &self,
        view: &SemanticView<'_>,
        fns: &[&FnFacts<'_>],
        family: Family,
    ) -> Result<Vec<String>, A2xReject> {
        self.validate_view(view)?;
        match family {
            Family::Scalar => {
                let lowered = lower_functions_py(view, fns)?;
                Ok(lowered.iter().map(print_fragment).collect())
            }
            Family::RecordText => {
                let mut out = Vec::with_capacity(fns.len());
                for f in fns {
                    out.push(print_fragment(&lower_rt_fn_py(view, f)?));
                }
                Ok(out)
            }
            Family::Collections => {
                let mut out = Vec::with_capacity(fns.len());
                for f in fns {
                    out.push(print_fragment(&lower_collections_fn_py(view, f)?));
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
        self.validate_view(view)?;
        match family {
            Family::Scalar => Ok(None),
            Family::RecordText => Ok(Some(encode_rt_seed(view)?)),
            Family::Collections => Ok(Some(encode_coll_seed(view)?)),
        }
    }

    fn assemble_fileset(&self, req: FilesetRequest<'_>) -> Result<FileSet, A2xReject> {
        let schema = Self::family_schema(req.family)?;
        let prelude_helpers = match req.family {
            Family::Scalar => String::new(),
            Family::RecordText => {
                if let Some(v) = req.view {
                    let encoded = encode_rt_seed(v)?;
                    render_rt_seed_helpers(&decode_rt_seed(&encoded)?)
                } else if let Some(raw) = req.prelude_seed {
                    render_rt_seed_helpers(&decode_rt_seed(raw)?)
                } else {
                    return Err(A2xReject::new(
                        "a2py.seed",
                        "RecordText viewless assembly requires prelude_seed",
                    ));
                }
            }
            Family::Collections => {
                if let Some(v) = req.view {
                    let encoded = encode_coll_seed(v)?;
                    render_coll_seed_helpers(&decode_coll_seed(&encoded)?)
                } else if let Some(raw) = req.prelude_seed {
                    render_coll_seed_helpers(&decode_coll_seed(raw)?)
                } else {
                    return Err(A2xReject::new(
                        "a2py.seed",
                        "Collections viewless assembly requires prelude_seed",
                    ));
                }
            }
        };

        let printed = print_program_py(
            &prelude_helpers,
            req.fragments,
            req.entry,
            req.origins,
            schema,
            req.input_hash,
        )?;

        let files = vec![
            ("a2x_py_runtime.py".to_string(), req.runtime.to_string()),
            ("generated.py".to_string(), printed.generated_py),
            ("a2x_main.py".to_string(), printed.main_py),
            ("source_map.json".to_string(), printed.source_map_json),
        ];

        let catalog = default_stdlib_port_catalog(PY_RUNTIME_VERSION);
        let mut manifest = String::new();
        manifest.push_str("{\n");
        manifest.push_str(&format!("  \"schema\": \"{schema}\",\n"));
        manifest.push_str(&format!("  \"producer\": \"{PY_PRODUCER}\",\n"));
        manifest.push_str(&format!("  \"backend\": \"{PY_BACKEND}\",\n"));
        manifest.push_str(&format!(
            "  \"backend_version\": \"{PY_BACKEND_VERSION}\",\n"
        ));
        manifest.push_str(&format!(
            "  \"runtime_version\": \"{PY_RUNTIME_VERSION}\",\n"
        ));
        manifest.push_str(&format!("  \"print_version\": \"{PY_PRINT_VERSION}\",\n"));
        manifest.push_str(&format!(
            "  \"stdlib_port_catalog\": \"{}\",\n",
            catalog.canonical_fingerprint()
        ));
        manifest.push_str(&format!("  \"input_hash\": \"{}\",\n", req.input_hash));
        manifest.push_str(&format!(
            "  \"source_sha256\": \"{}\",\n",
            req.source_sha256.unwrap_or_default()
        ));
        manifest.push_str(&format!("  \"entry\": \"{}\",\n", req.entry));
        manifest.push_str("  \"functions\": [\n");
        for (i, r) in req.fragments.iter().enumerate() {
            let comma = if i + 1 < req.fragments.len() { "," } else { "" };
            let origin_opt = if let Some(v) = req.view {
                Some(v.fn_by_def(r.def_index).map(|f| f.origin).unwrap_or((0, 0)))
            } else {
                req.origins
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
