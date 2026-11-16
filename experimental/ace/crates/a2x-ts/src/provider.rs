//! PLAN-976: the shared A2X `TargetProvider` for the TypeScript and
//! JavaScript legs (Scalar / RecordText / Collections families).

use ace_a2x_core::{
    A2xReject, Family, FileSet, FilesetRequest, FnFacts, SemanticView, TargetDescriptor,
    TargetProvider,
};
use ace_core::runtime_contract::sha256_hex;

use crate::lower::{lower_collections_fn, lower_functions, lower_rt_fn, print_fragment};
use crate::ports::default_stdlib_port_catalog;
use crate::print::{print_program, TS_PRINT_VERSION};
use crate::runtime::{
    runtime_body_sha256, script_runtime_body, script_runtime_declaration, A2TS_RUNTIME_VERSION,
    RUNTIME_BODY,
};
use crate::seed::{
    decode_coll_seed, decode_rt_seed, encode_coll_seed, encode_rt_seed, render_coll_seed_helpers,
    render_rt_seed_helpers,
};
use crate::target::{
    ScriptLang, BACKEND_VERSION, TS_COLL_FILESET_SCHEMA, TS_PRODUCER, TS_RT_FILESET_SCHEMA,
    TS_SCALAR_FILESET_SCHEMA,
};

/// The TypeScript provider (`auto-a2x-typescript`).
#[derive(Clone, Copy, Debug, Default)]
pub struct TypeScriptTargetProvider;

/// The JavaScript provider (`auto-a2x-javascript`).
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaScriptTargetProvider;

impl TypeScriptTargetProvider {
    pub const fn v1() -> Self {
        TypeScriptTargetProvider
    }
}

impl JavaScriptTargetProvider {
    pub const fn v1() -> Self {
        JavaScriptTargetProvider
    }
}

macro_rules! delegate_provider {
    ($ty:ident, $typed:expr) => {
        impl TargetProvider for $ty {
            fn validate_view(&self, view: &SemanticView<'_>) -> Result<(), A2xReject> {
                ScriptProvider::<$typed>.validate_view(view)
            }
            fn descriptor(&self) -> TargetDescriptor {
                ScriptProvider::<$typed>.descriptor()
            }
            fn fragment_key_inputs(&self, runtime_sha256: &str) -> Vec<(String, String)> {
                ScriptProvider::<$typed>.fragment_key_inputs(runtime_sha256)
            }
            fn fragment_layer_id(&self, family: Family) -> Result<&'static str, A2xReject> {
                ScriptProvider::<$typed>.fragment_layer_id(family)
            }
            fn runtime_source(
                &self,
                view: &SemanticView<'_>,
                family: Family,
            ) -> Result<String, A2xReject> {
                ScriptProvider::<$typed>.runtime_source(view, family)
            }
            fn produce_fragments(
                &self,
                view: &SemanticView<'_>,
                fns: &[&FnFacts<'_>],
                family: Family,
            ) -> Result<Vec<String>, A2xReject> {
                ScriptProvider::<$typed>.produce_fragments(view, fns, family)
            }
            fn capture_prelude_seed(
                &self,
                view: &SemanticView<'_>,
                family: Family,
            ) -> Result<Option<String>, A2xReject> {
                ScriptProvider::<$typed>.capture_prelude_seed(view, family)
            }
            fn assemble_fileset(&self, req: FilesetRequest<'_>) -> Result<FileSet, A2xReject> {
                ScriptProvider::<$typed>.assemble_fileset(req)
            }
        }
    };
}

delegate_provider!(TypeScriptTargetProvider, true);
delegate_provider!(JavaScriptTargetProvider, false);

/// Stateless provider value: the const parameter selects the leg (typed
/// TypeScript vs plain ESM JavaScript).
#[derive(Clone, Copy, Debug, Default)]
pub struct ScriptProvider<const TYPED: bool>;

impl<const TYPED: bool> ScriptProvider<TYPED> {
    pub const fn v1() -> Self {
        ScriptProvider
    }

    fn lang(&self) -> ScriptLang {
        if TYPED {
            ScriptLang::TypeScript
        } else {
            ScriptLang::JavaScript
        }
    }

    fn family_schema(family: Family) -> &'static str {
        match family {
            Family::Scalar => TS_SCALAR_FILESET_SCHEMA,
            Family::RecordText => TS_RT_FILESET_SCHEMA,
            Family::Collections => TS_COLL_FILESET_SCHEMA,
        }
    }

    fn catalog_fingerprint() -> String {
        default_stdlib_port_catalog(RUNTIME_BODY).canonical_fingerprint()
    }
}

impl<const TYPED: bool> TargetProvider for ScriptProvider<TYPED> {
    fn validate_view(&self, view: &SemanticView<'_>) -> Result<(), A2xReject> {
        // Host authority: the script target has no native FFI. A module that
        // declares foreign resources is refused by name, never silently
        // ignored (nothing is emitted for it, so silence would corrupt).
        if let Some(decl) = view.foreign_declarations().next() {
            return Err(A2xReject::new(
                "a2ts.foreign-unsupported",
                format!(
                    "foreign native declaration `{}` is outside the TypeScript/JavaScript target domain",
                    decl.name
                ),
            ));
        }
        Ok(())
    }

    fn descriptor(&self) -> TargetDescriptor {
        let lang = self.lang();
        TargetDescriptor {
            lang: lang.lang_name(),
            standard: lang.standard(),
            backend: lang.backend(),
            backend_version: BACKEND_VERSION,
        }
    }

    fn fragment_key_inputs(&self, runtime_sha256: &str) -> Vec<(String, String)> {
        vec![
            (
                "target_lang".to_string(),
                self.lang().lang_name().to_string(),
            ),
            (
                "runtime_version".to_string(),
                A2TS_RUNTIME_VERSION.to_string(),
            ),
            ("runtime_sha256".to_string(), runtime_sha256.to_string()),
            ("print_version".to_string(), TS_PRINT_VERSION.to_string()),
            (
                "stdlib_port_catalog".to_string(),
                Self::catalog_fingerprint(),
            ),
        ]
    }

    fn fragment_layer_id(&self, family: Family) -> Result<&'static str, A2xReject> {
        let id = self.lang().fragment_layer(match family {
            Family::Scalar => 0,
            Family::RecordText => 1,
            Family::Collections => 2,
        });
        if id.is_empty() {
            return Err(A2xReject::new("a2ts.family", "unregistered script family"));
        }
        Ok(id)
    }

    fn runtime_source(&self, view: &SemanticView<'_>, family: Family) -> Result<String, A2xReject> {
        let _ = family;
        let marks: Vec<&'static str> = view.trace_symbols().into_iter().collect();
        Ok(script_runtime_body(self.lang(), &marks))
    }

    fn produce_fragments(
        &self,
        view: &SemanticView<'_>,
        fns: &[&FnFacts<'_>],
        family: Family,
    ) -> Result<Vec<String>, A2xReject> {
        self.validate_view(view)?;
        let lang = self.lang();
        match family {
            Family::Scalar => {
                let lowered = lower_functions(lang, view, fns)?;
                Ok(lowered
                    .iter()
                    .map(|f| print_fragment(lang, view, f))
                    .collect())
            }
            Family::RecordText => {
                let mut out = Vec::with_capacity(fns.len());
                for f in fns {
                    let lowered = lower_rt_fn(lang, view, f)?;
                    out.push(print_fragment(lang, view, &lowered));
                }
                Ok(out)
            }
            Family::Collections => {
                let mut out = Vec::with_capacity(fns.len());
                for f in fns {
                    let lowered = lower_collections_fn(lang, view, f)?;
                    out.push(print_fragment(lang, view, &lowered));
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
            Family::RecordText => Ok(Some(encode_rt_seed(view)?)),
            Family::Collections => Ok(Some(encode_coll_seed(view)?)),
        }
    }

    fn assemble_fileset(&self, req: FilesetRequest<'_>) -> Result<FileSet, A2xReject> {
        let lang = self.lang();
        let schema = Self::family_schema(req.family);
        let prelude_helpers = match req.family {
            Family::Scalar => String::new(),
            Family::RecordText => {
                if let Some(v) = req.view {
                    render_rt_seed_helpers(&decode_rt_seed(&encode_rt_seed(v)?)?, lang.typed())
                } else if let Some(raw) = req.prelude_seed {
                    render_rt_seed_helpers(&decode_rt_seed(raw)?, lang.typed())
                } else {
                    return Err(A2xReject::new(
                        "a2ts.seed",
                        "RecordText view-less assembly requires prelude_seed",
                    ));
                }
            }
            Family::Collections => {
                if let Some(v) = req.view {
                    render_coll_seed_helpers(
                        &decode_coll_seed(&encode_coll_seed(v)?)?,
                        lang.typed(),
                    )
                } else if let Some(raw) = req.prelude_seed {
                    render_coll_seed_helpers(&decode_coll_seed(raw)?, lang.typed())
                } else {
                    return Err(A2xReject::new(
                        "a2ts.seed",
                        "Collections view-less assembly requires prelude_seed",
                    ));
                }
            }
        };

        let printed = print_program(
            lang,
            &prelude_helpers,
            req.fragments,
            req.entry,
            req.origins,
            schema,
            req.input_hash,
        )?;

        let generated_file = lang.generated_file();
        let main_file = lang.main_file();
        let mut files: Vec<(String, String)> = vec![
            (lang.runtime_file().to_string(), req.runtime.to_string()),
            (generated_file.clone(), printed.generated),
            (main_file.clone(), printed.main),
            ("source_map.json".to_string(), printed.source_map_json),
        ];
        if let Some(decl) = lang.runtime_declaration_file() {
            files.push((decl.to_string(), script_runtime_declaration()));
        }
        files.sort_by(|a, b| a.0.cmp(&b.0));

        let mut manifest = String::new();
        manifest.push_str("{\n");
        manifest.push_str(&format!("  \"schema\": \"{schema}\",\n"));
        manifest.push_str(&format!("  \"producer\": \"{TS_PRODUCER}\",\n"));
        manifest.push_str(&format!("  \"backend\": \"{}\",\n", lang.backend()));
        manifest.push_str(&format!("  \"backend_version\": \"{BACKEND_VERSION}\",\n"));
        manifest.push_str(&format!("  \"target_lang\": \"{}\",\n", lang.lang_name()));
        manifest.push_str(&format!("  \"standard\": \"{}\",\n", lang.standard()));
        manifest.push_str(&format!(
            "  \"runtime_version\": \"{A2TS_RUNTIME_VERSION}\",\n"
        ));
        manifest.push_str(&format!(
            "  \"runtime_sha256\": \"{}\",\n",
            runtime_body_sha256()
        ));
        manifest.push_str(&format!("  \"print_version\": \"{TS_PRINT_VERSION}\",\n"));
        manifest.push_str(&format!(
            "  \"stdlib_port_catalog\": \"{}\",\n",
            Self::catalog_fingerprint()
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
