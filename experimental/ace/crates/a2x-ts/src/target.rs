//! PLAN-976 T-01: the frozen TypeScript/JavaScript target identities.
//!
//! One crate serves two targets that share one lowering: `typescript` (typed,
//! node type-strip execution + `tsc --noEmit` type checking) and `javascript`
//! (plain ESM). Everything target-specific is confined to the file naming,
//! the type-annotation sink and the identity constants below; the lowering and
//! the runtime helper catalogue are shared so the two legs cannot drift.

/// The script language of one provider instantiation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScriptLang {
    TypeScript,
    JavaScript,
}

impl ScriptLang {
    pub const fn backend(self) -> &'static str {
        match self {
            ScriptLang::TypeScript => TS_BACKEND,
            ScriptLang::JavaScript => JS_BACKEND,
        }
    }

    /// Language level contract string (part of fragment keys + manifests).
    pub const fn standard(self) -> &'static str {
        match self {
            ScriptLang::TypeScript => TS_STANDARD,
            ScriptLang::JavaScript => JS_STANDARD,
        }
    }

    pub const fn lang_name(self) -> &'static str {
        match self {
            ScriptLang::TypeScript => "typescript",
            ScriptLang::JavaScript => "javascript",
        }
    }

    /// Source extension of the generated function module.
    pub const fn source_ext(self) -> &'static str {
        match self {
            ScriptLang::TypeScript => "ts",
            ScriptLang::JavaScript => "mjs",
        }
    }

    /// Whether the emitted script carries static type annotations.
    pub const fn typed(self) -> bool {
        matches!(self, ScriptLang::TypeScript)
    }

    pub fn generated_file(self) -> String {
        format!("generated.{}", self.source_ext())
    }

    pub fn main_file(self) -> String {
        format!("a2x_main.{}", self.source_ext())
    }

    pub const fn runtime_file(self) -> &'static str {
        // The runtime is always plain ESM JavaScript: both node's type
        // stripping and tsc (via the sibling .d.mts declaration) accept it.
        "a2x_rt.mjs"
    }

    pub const fn runtime_declaration_file(self) -> Option<&'static str> {
        match self {
            ScriptLang::TypeScript => Some("a2x_rt.d.mts"),
            ScriptLang::JavaScript => None,
        }
    }

    pub const fn fragment_layer(self, family_family: u8) -> &'static str {
        match (self, family_family) {
            (ScriptLang::TypeScript, 0) => "AXTS1",
            (ScriptLang::TypeScript, 1) => "AXTS2",
            (ScriptLang::TypeScript, 2) => "AXTS3",
            (ScriptLang::JavaScript, 0) => "AXJS1",
            (ScriptLang::JavaScript, 1) => "AXJS2",
            (ScriptLang::JavaScript, 2) => "AXJS3",
            _ => "",
        }
    }

    pub const fn fileset_schema(self, family_family: u8) -> &'static str {
        match family_family {
            0 => TS_SCALAR_FILESET_SCHEMA,
            1 => TS_RT_FILESET_SCHEMA,
            2 => TS_COLL_FILESET_SCHEMA,
            _ => "",
        }
    }

    pub fn recipe(self, family_family: u8) -> &'static str {
        match (self, family_family) {
            (ScriptLang::TypeScript, 0) => TS_RECIPE,
            (ScriptLang::TypeScript, 1) => TS_RT_RECIPE,
            (ScriptLang::TypeScript, 2) => TS_COLL_RECIPE,
            (ScriptLang::JavaScript, 0) => JS_RECIPE,
            (ScriptLang::JavaScript, 1) => JS_RT_RECIPE,
            _ => JS_COLL_RECIPE,
        }
    }
}

/// Backend identifier for the TypeScript leg (registered in `a2x-store`).
pub const TS_BACKEND: &str = "auto-a2x-typescript";
/// Backend identifier for the JavaScript leg.
pub const JS_BACKEND: &str = "auto-a2x-javascript";

pub const BACKEND_VERSION: &str = "0.1.0";

/// TypeScript language-level contract: erasable syntax only (node type strip).
pub const TS_STANDARD: &str = "typescript5.9-es2022-erasable";
/// JavaScript language-level contract: ESM, node >= 22 (ES2022 baseline).
pub const JS_STANDARD: &str = "es2022-node-esm";

/// Worker/module-bundle producer identity accepted by `semantics-contract`.
/// Must stay byte-equal to `semantics_contract::assembly::TS_PRODUCER`.
pub const TS_PRODUCER: &str = "ace-a2x-ts/modules/1";

pub const TS_SCALAR_FILESET_SCHEMA: &str = "ace.a2ts-hir.scalar.fileset/1";
pub const TS_RT_FILESET_SCHEMA: &str = "ace.a2ts-hir.record-text.fileset/1";
pub const TS_COLL_FILESET_SCHEMA: &str = "ace.a2ts-hir.collections.fileset/1";

pub const TS_RECIPE: &str = "a2ts-hir-typescript-checked";
pub const TS_RT_RECIPE: &str = "a2ts-hir-typescript-record-text";
pub const TS_COLL_RECIPE: &str = "a2ts-hir-typescript-collections";
pub const JS_RECIPE: &str = "a2js-hir-javascript-checked";
pub const JS_RT_RECIPE: &str = "a2js-hir-javascript-record-text";
pub const JS_COLL_RECIPE: &str = "a2js-hir-javascript-collections";
