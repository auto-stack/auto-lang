//! Strict source-import certification report mirror. No experimental compiler
//! dependency enters the language SDK; the existing ABI transport is reused.
use serde::{Deserialize, Serialize};
pub(super) const RECIPE: &str = "abi/source-foreign-certify-v1";
// Parent-owned acceptance pins for this finite reviewed recipe. A producer's
// well-formed hash is evidence only when it names these independent materials.
const REVIEWED_CONTRACT: &str = "cef4c3885f1686a00c0bb8c44ca6b95559a634fff297f8193becca2d1d35d71c";
const REVIEWED_SCALAR_CONTRACT: &str =
    "aaa3a8b32d0723d77c2c12202d86445ca8626a9368374fd787517801601ebd7e";
const REVIEWED_HEADER: &str = "4051cd6af2acd0c07eeed8495f50ad51fdfac94c3b91928f86422e486b408b98";
const REVIEWED_PHYSICAL_TABLE: &str =
    "0ea8be97d1026bd57dcaff7b35673e45a6a9f17e2f5c991bdd1b1fb04a94daa3";
fn reviewed_c_toolchain(value: &str) -> bool {
    if value.len() > 8192 || value.chars().any(char::is_control) {
        return false;
    }
    let mut fields = std::collections::BTreeMap::new();
    for atom in value.split('|') {
        let Some((key, value)) = atom.split_once(':') else {
            return false;
        };
        if value.is_empty() || fields.insert(key, value).is_some() {
            return false;
        }
    }
    let digest = |key| {
        fields.get(key).is_some_and(|value| {
            value.len() == 64
                && value
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        })
    };
    if !fields.contains_key("cl")
        || !fields.contains_key("ver")
        || !digest("hash")
        || fields.get("flags") != Some(&"-c -nologo -GS- -Zl -W4 -WX -O2")
        || fields.get("crt-flags") != Some(&"-nologo -std:c11 -TC -W4 -WX -O2 -MT")
    {
        return false;
    }
    match fields.get("crt").copied() {
        Some("explicit-sdk/1") => {
            fields.len() == 9
                && fields.contains_key("crt-sdk")
                && fields.contains_key("crt-msvc")
                && digest("crt-link-hash")
        }
        Some("vcvars/1") => {
            fields.len() == 8 && fields.contains_key("crt-vcvars") && digest("crt-vcvars-hash")
        }
        _ => false,
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SourceDiagnostic {
    pub stage: String,
    pub code: String,
    pub message: String,
    pub span: Option<(usize, usize)>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SourceReport {
    pub diagnostics: Vec<SourceDiagnostic>,
    pub schema: String,
    pub direction: String,
    pub input_profile: String,
    pub abi_domain: String,
    pub abi_version: String,
    pub abi_profile: String,
    pub callconv: String,
    pub target: String,
    pub consumer: String,
    pub source_sha256: String,
    pub compiler: String,
    pub c_toolchain: String,
    pub contract_digest: String,
    pub provider_digest: String,
    pub header_digest: String,
    pub physical_table_digest: String,
    pub review_id: String,
    pub actual_exit: Option<i32>,
    pub reference_exit: Option<i32>,
    pub actual_events: Vec<String>,
    pub reference_events: Vec<String>,
    pub actual_stderr: String,
    pub reference_stderr: String,
    pub source_parse: u64,
    pub solver: u64,
    pub hir: u64,
    pub mir: u64,
    pub native: u64,
    pub c_fragment: u64,
    pub diagnostic_stage: String,
}
impl SourceReport {
    /// A resource refusal before source production may not have decoded the
    /// source field. The enclosing worker envelope still binds the exact deck,
    /// nonce and request hash. This exception cannot carry compiler/tool work,
    /// a source verdict or execution evidence, and never grants a PASS.
    pub fn admit_query(
        &self,
        consumer: &str,
        outcome: &str,
        exit: Option<i32>,
        source: Option<&str>,
        reference: Option<&str>,
    ) -> Result<(), String> {
        if !matches!(outcome, "ok" | "diags" | "resource" | "infra")
            || (outcome != "ok" && exit.is_some())
        {
            return Err("invalid source outcome or exit on a refusal".into());
        }
        let unprocessed = outcome == "resource" && self.source_sha256.is_empty();
        if unprocessed
            && (self.source_parse != 0
                || self.solver != 0
                || self.hir != 0
                || self.mir != 0
                || self.native != 0
                || self.c_fragment != 0
                || !self.provider_digest.is_empty()
                || !self.c_toolchain.is_empty()
                || self.diagnostic_stage != "parse"
                || self
                    .diagnostics
                    .iter()
                    .any(|d| d.code != "foreign.resource" || d.span.is_some()))
        {
            return Err(
                "unprocessed source resource refusal carried production/provenance claims".into(),
            );
        }
        self.admit_reference(
            consumer,
            outcome == "ok",
            exit,
            if unprocessed { None } else { source },
            // An undecoded refusal has no reference or production evidence;
            // only the original trusted placeholder identity is permitted.
            if unprocessed && self.contract_digest == REVIEWED_CONTRACT {
                "add"
            } else {
                reference.ok_or("source-import request without a reviewed reference")?
            },
        )
    }
    #[cfg(test)]
    pub fn admit(
        &self,
        consumer: &str,
        success: bool,
        exit: Option<i32>,
        source: Option<&str>,
    ) -> Result<(), String> {
        self.admit_reference(consumer, success, exit, source, "add")
    }
    fn admit_reference(
        &self,
        consumer: &str,
        success: bool,
        exit: Option<i32>,
        source: Option<&str>,
        reference: &str,
    ) -> Result<(), String> {
        // These policies belong to the parent. They are deliberately not
        // generated from the worker's manifest, call log or returned result.
        let expected_events: &[&str] = match reference {
            "add" => &["provider_add"],
            "bool-false" | "bool-true" => &["provider_not"],
            "fallible" | "zero" | "overflow" => &["provider_div"],
            "order" => &[
                "provider_tick",
                "provider_tick",
                "provider_add",
                "provider_trace",
                "provider_calls",
            ],
            "and" | "or" => &["provider_calls"],
            "mixed" => &["provider_mix"],
            "named-order" => &[
                "provider_tick",
                "provider_tick",
                "provider_add",
                "provider_trace",
            ],
            _ => {
                return Err("source-import request reference is outside independent review".into())
            }
        };
        let expected_status: &[&str] = match reference {
            "zero" => &["-1"],
            "overflow" => &["-2"],
            _ => &[],
        };
        let expected_contract = if reference == "add" {
            REVIEWED_CONTRACT
        } else {
            REVIEWED_SCALAR_CONTRACT
        };
        let digest = |s: &str| {
            s.len() == 64
                && s.bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        };
        if self.schema != "abi.source-import-report/1"
            || self.direction != "source-import"
            || self.input_profile != "auto.source.scalar-i32.draft"
            || self.abi_profile != "autoabi.win-x64.v0.1-candidate"
            || self.abi_domain != "autoabi"
            || self.abi_version != "0.1"
            || self.callconv != "windows-x64"
            || self.target != "x86_64-pc-windows-msvc"
            || self.consumer != consumer
            || !matches!(consumer, "native" | "c")
            || self.review_id != "plan938-independent-provider-v1"
            || self.contract_digest != expected_contract
            || self.header_digest != REVIEWED_HEADER
            || self.physical_table_digest != REVIEWED_PHYSICAL_TABLE
        {
            return Err("source-import report identity mismatch".into());
        }
        if let Some(source) = source {
            if self.source_sha256
                != auto_test_case::sha256::hex(&auto_test_case::sha256::sha256(source.as_bytes()))
            {
                return Err(
                    "source-import report source digest differs from the submitted ordinary source"
                        .into(),
                );
            }
        } else if success || !self.source_sha256.is_empty() {
            return Err("source-import report without source provenance".into());
        }
        if (consumer == "native"
            && self.compiler != "ace-native-foreign/1+ace-native/0.1.0+cranelift/0.126.2")
            || (consumer == "c"
                && self.compiler != "auto-a2x-c-foreign/1+c11+a2x-c-foreign-print/1")
        {
            return Err("source-import compiler identity mismatch".into());
        }
        if success {
            if self.actual_exit != exit
                || self.reference_exit.is_none()
                || !digest(&self.provider_digest)
                || !reviewed_c_toolchain(&self.c_toolchain)
                || self.actual_events != self.reference_events
                || self.reference_events != expected_events
                || !self.diagnostic_stage.is_empty()
                || !self.diagnostics.is_empty()
            {
                return Err("source-import actual/reference/call evidence is incomplete".into());
            }
            for (stderr, events) in [
                (&self.actual_stderr, &self.actual_events),
                (&self.reference_stderr, &self.reference_events),
            ] {
                let observed = stderr
                    .lines()
                    .filter_map(|l| l.strip_prefix("source938.call=").map(String::from))
                    .collect::<Vec<_>>();
                if observed != *events || stderr.len() > 1 << 20 {
                    return Err("source-import raw call events disagree".into());
                }
                let statuses = stderr
                    .lines()
                    .filter_map(|line| line.strip_prefix("foreign.status="))
                    .collect::<Vec<_>>();
                if statuses != expected_status {
                    return Err("source-import controlled failure cause is missing or differs from independent review".into());
                }
            }
        } else if self.actual_exit.is_some()
            || self.reference_exit.is_some()
            || !self.actual_events.is_empty()
            || !self.reference_events.is_empty()
            || !self.actual_stderr.is_empty()
            || !self.reference_stderr.is_empty()
            || !matches!(
                self.diagnostic_stage.as_str(),
                "parse" | "resolve" | "typecheck" | "verify"
            )
        {
            return Err("rejected source-import candidate carried execution evidence".into());
        }
        if !success {
            if self.diagnostics.is_empty() {
                return Err("source-import rejection without original diagnostics".into());
            }
            for d in &self.diagnostics {
                if d.stage != self.diagnostic_stage
                    || d.code.is_empty()
                    || d.message.is_empty()
                    || d.span.is_some_and(|(a, b)| {
                        source.is_none_or(|s| {
                            a > b || b > s.len() || !s.is_char_boundary(a) || !s.is_char_boundary(b)
                        })
                    })
                {
                    return Err("source-import diagnostic stage/span is invalid".into());
                }
            }
        }
        if (consumer == "native" && self.c_fragment != 0)
            || (consumer == "c" && (self.native != 0 || self.mir != 0))
        {
            return Err("source-import consumer crossed target lowering boundaries".into());
        }
        // These real production axes are retained for the parent report; no
        // inference of zero work from a cache word or successful value.
        let _front_production = (self.source_parse, self.solver, self.hir);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn certification_requires_complete_current_crt_inputs_and_frozen_flags() {
        let source = "fn main() int { return foreign_add(20,22) }";
        let report = valid();
        report
            .admit("native", true, Some(42), Some(source))
            .unwrap();
        for axis in 0..6 {
            let mut changed = report.clone();
            changed.c_toolchain = match axis {
                0 => changed.c_toolchain.split("|crt:").next().unwrap().into(),
                1 => changed
                    .c_toolchain
                    .replace("explicit-sdk/1", "unreviewed-sdk/1"),
                2 => changed
                    .c_toolchain
                    .replace(&format!("|crt-link-hash:{}", "d".repeat(64)), ""),
                3 => changed.c_toolchain.replace(
                    "-std:c11 -TC -W4 -WX -O2 -MT",
                    "-std:c11 -TC -W4 -WX -O0 -MT",
                ),
                4 => format!(
                    "{}|crt-flags:-nologo -std:c11 -TC -W4 -WX -O2 -MT",
                    changed.c_toolchain
                ),
                _ => format!("{}|invented:claim", changed.c_toolchain),
            };
            assert!(
                changed
                    .admit("native", true, Some(42), Some(source))
                    .is_err(),
                "axis {axis}"
            );
        }
        let mut explicit = report.clone();
        explicit.c_toolchain = format!("cl:C:/tools/cl.exe|ver:19.44.1|hash:{}|flags:-c -nologo -GS- -Zl -W4 -WX -O2|crt:vcvars/1|crt-vcvars:C:/tools/setup.bat|crt-vcvars-hash:{}|crt-flags:-nologo -std:c11 -TC -W4 -WX -O2 -MT", "c".repeat(64), "e".repeat(64));
        explicit
            .admit("native", true, Some(42), Some(source))
            .unwrap();
        explicit.c_toolchain = explicit
            .c_toolchain
            .replace(&format!("|crt-vcvars-hash:{}", "e".repeat(64)), "");
        assert!(explicit
            .admit("native", true, Some(42), Some(source))
            .is_err());
    }
    #[test]
    fn scalar_certificate_requires_the_submitted_review_calls_and_original_failure_cause() {
        let source = "fn main() int { return foreign_div(84,0) }";
        let mut report = valid();
        report.source_sha256 =
            auto_test_case::sha256::hex(&auto_test_case::sha256::sha256(source.as_bytes()));
        report.contract_digest = REVIEWED_SCALAR_CONTRACT.into();
        report.actual_exit = Some(70);
        report.reference_exit = Some(70);
        report.actual_events = vec!["provider_div".into()];
        report.reference_events = report.actual_events.clone();
        report.actual_stderr = "source938.call=provider_div\nforeign.status=-1\n".into();
        report.reference_stderr = report.actual_stderr.clone();
        report
            .admit_query("native", "ok", Some(70), Some(source), Some("zero"))
            .unwrap();
        for axis in 0..6 {
            let mut bad = report.clone();
            let reference = match axis {
                0 => "fallible",
                1 => "unreviewed",
                _ => "zero",
            };
            match axis {
                2 => bad.actual_stderr = "source938.call=provider_div\n".into(),
                3 => {
                    bad.reference_stderr = "source938.call=provider_div\nforeign.status=-2\n".into()
                }
                4 => {
                    bad.actual_events.clear();
                    bad.reference_events.clear();
                    bad.actual_stderr = "foreign.status=-1\n".into();
                    bad.reference_stderr = bad.actual_stderr.clone();
                }
                5 => bad.contract_digest = REVIEWED_CONTRACT.into(),
                _ => {}
            }
            assert!(
                bad.admit_query("native", "ok", Some(70), Some(source), Some(reference))
                    .is_err(),
                "axis {axis}"
            );
        }
        let source = "fn main() int { return foreign_calls() }";
        report.source_sha256 =
            auto_test_case::sha256::hex(&auto_test_case::sha256::sha256(source.as_bytes()));
        report.actual_exit = Some(1);
        report.reference_exit = Some(1);
        report.actual_events = vec!["provider_calls".into()];
        report.reference_events = report.actual_events.clone();
        report.actual_stderr = "source938.call=provider_calls\n".into();
        report.reference_stderr = report.actual_stderr.clone();
        report
            .admit_query("native", "ok", Some(1), Some(source), Some("and"))
            .unwrap();
        // A legal short circuit has no provider_not call. An eager actual+ref
        // agreement must still fail the independent call oracle.
        report.actual_events.insert(0, "provider_not".into());
        report.reference_events = report.actual_events.clone();
        report
            .actual_stderr
            .insert_str(0, "source938.call=provider_not\n");
        report.reference_stderr = report.actual_stderr.clone();
        assert!(report
            .admit_query("native", "ok", Some(1), Some(source), Some("and"))
            .is_err());
    }
    fn valid() -> SourceReport {
        let source = "fn main() int { return foreign_add(20,22) }";
        serde_json::from_value(serde_json::json!({
            "schema":"abi.source-import-report/1","direction":"source-import","input_profile":"auto.source.scalar-i32.draft",
            "abi_domain":"autoabi","abi_version":"0.1","abi_profile":"autoabi.win-x64.v0.1-candidate","callconv":"windows-x64",
            "target":"x86_64-pc-windows-msvc","consumer":"native","source_sha256":auto_test_case::sha256::hex(&auto_test_case::sha256::sha256(source.as_bytes())),
            "compiler":"ace-native-foreign/1+ace-native/0.1.0+cranelift/0.126.2","c_toolchain":format!("cl:C:/tools/cl.exe|ver:19.44.1|hash:{}|flags:-c -nologo -GS- -Zl -W4 -WX -O2|crt:explicit-sdk/1|crt-sdk:C:/sdk/um/x64|crt-msvc:C:/vc/include|crt-link-hash:{}|crt-flags:-nologo -std:c11 -TC -W4 -WX -O2 -MT", "c".repeat(64), "d".repeat(64)),
            "contract_digest":REVIEWED_CONTRACT,"provider_digest":"b".repeat(64),"header_digest":REVIEWED_HEADER,"physical_table_digest":REVIEWED_PHYSICAL_TABLE,
            "review_id":"plan938-independent-provider-v1","actual_exit":42,"reference_exit":42,
            "actual_events":["provider_add"],"reference_events":["provider_add"],"actual_stderr":"source938.call=provider_add\n","reference_stderr":"source938.call=provider_add\n",
            "source_parse":2,"solver":1,"hir":1,"mir":1,"native":1,"c_fragment":0,"diagnostic_stage":"","diagnostics":[]
        })).unwrap()
    }
    #[test]
    fn source_report_requires_the_current_independent_review_content() {
        let source = "fn main() int { return foreign_add(20,22) }";
        let mut reviewed = valid();
        reviewed.contract_digest =
            "cef4c3885f1686a00c0bb8c44ca6b95559a634fff297f8193becca2d1d35d71c".into();
        reviewed.header_digest =
            "4051cd6af2acd0c07eeed8495f50ad51fdfac94c3b91928f86422e486b408b98".into();
        reviewed.physical_table_digest =
            "0ea8be97d1026bd57dcaff7b35673e45a6a9f17e2f5c991bdd1b1fb04a94daa3".into();
        reviewed
            .admit("native", true, Some(42), Some(source))
            .unwrap();
        for axis in 0..3 {
            let mut forged = reviewed.clone();
            match axis {
                0 => forged.contract_digest = "a".repeat(64),
                1 => forged.header_digest = "a".repeat(64),
                _ => forged.physical_table_digest = "a".repeat(64),
            }
            assert!(
                forged
                    .admit("native", true, Some(42), Some(source))
                    .is_err(),
                "review content axis {axis}: a well-formed but unrelated digest was accepted"
            );
        }
    }

    #[test]
    fn predecode_resource_refusal_never_substitutes_for_source_or_execution_evidence() {
        let source = "fn main() int { return foreign_add(20,22) }";
        let mut report = valid();
        report.source_sha256.clear();
        report.provider_digest.clear();
        report.c_toolchain.clear();
        report.source_parse = 0;
        report.solver = 0;
        report.hir = 0;
        report.mir = 0;
        report.native = 0;
        report.actual_exit = None;
        report.reference_exit = None;
        report.actual_events.clear();
        report.reference_events.clear();
        report.actual_stderr.clear();
        report.reference_stderr.clear();
        report.diagnostic_stage = "parse".into();
        report.diagnostics = vec![SourceDiagnostic {
            stage: "parse".into(),
            code: "foreign.resource".into(),
            message: "input model budget".into(),
            span: None,
        }];
        report
            .admit_query("native", "resource", None, Some(source), Some("add"))
            .unwrap();
        assert!(report
            .admit_query("native", "ok", None, Some(source), Some("add"))
            .is_err());
        assert!(report
            .admit_query("native", "diags", None, Some(source), Some("add"))
            .is_err());
        assert!(report
            .admit_query("native", "resource", Some(42), Some(source), Some("add"))
            .is_err());
        for axis in 0..8 {
            let mut bad = report.clone();
            match axis {
                0 => bad.source_parse = 1,
                1 => bad.provider_digest = "b".repeat(64),
                2 => bad.c_toolchain = "cl:tool".into(),
                3 => bad.actual_events.push("provider_add".into()),
                4 => bad.diagnostics[0].span = Some((0, 1)),
                5 => bad.diagnostics[0].code = "source.foreign.type-mismatch".into(),
                6 => bad.source_sha256 = "0".repeat(64),
                _ => bad.diagnostics.clear(),
            }
            assert!(
                bad.admit_query("native", "resource", None, Some(source), Some("add"))
                    .is_err(),
                "axis {axis}"
            );
        }
    }

    #[test]
    fn source_report_refuses_cross_recipe_target_missing_calls_and_replayed_source() {
        let source = "fn main() int { return foreign_add(20,22) }";
        let original = valid();
        original
            .admit("native", true, Some(42), Some(source))
            .unwrap();
        for axis in 0..10 {
            let mut changed = original.clone();
            match axis {
                0 => changed.direction = "source-export".into(),
                1 => changed.input_profile = "typed".into(),
                2 => changed.abi_domain = "mn-private".into(),
                3 => changed.compiler = "ace-native-foreign/1+other".into(),
                4 => changed.consumer = "c".into(),
                5 => changed.actual_events.clear(),
                6 => changed.reference_events.clear(),
                7 => changed.actual_stderr.clear(),
                8 => changed.c_fragment = 1,
                _ => changed.source_sha256 = "0".repeat(64),
            }
            assert!(
                changed
                    .admit("native", true, Some(42), Some(source))
                    .is_err(),
                "axis {axis}"
            );
        }
        let mut unknown = serde_json::to_value(&original).unwrap();
        unknown["safe"] = true.into();
        assert!(serde_json::from_value::<SourceReport>(unknown).is_err());
        let mut missing = serde_json::to_value(&original).unwrap();
        missing.as_object_mut().unwrap().remove("direction");
        assert!(serde_json::from_value::<SourceReport>(missing).is_err());
        assert!(original
            .admit("native", true, Some(43), Some(source))
            .is_err());
    }
    #[test]
    fn source_rejection_must_not_carry_execution_and_diagnostic_span_stays_current() {
        let source = "fn main() int { return foreign_add(true,22) }";
        let mut rejected = valid();
        rejected.source_sha256 =
            auto_test_case::sha256::hex(&auto_test_case::sha256::sha256(source.as_bytes()));
        rejected.actual_exit = None;
        rejected.reference_exit = None;
        rejected.actual_events.clear();
        rejected.reference_events.clear();
        rejected.actual_stderr.clear();
        rejected.reference_stderr.clear();
        rejected.diagnostic_stage = "resolve".into();
        rejected.diagnostics = vec![SourceDiagnostic {
            stage: "resolve".into(),
            code: "source.foreign.type-mismatch".into(),
            message: "type mismatch".into(),
            span: Some((35, 39)),
        }];
        rejected.admit("native", false, None, Some(source)).unwrap();
        let mut bad = rejected.clone();
        bad.actual_exit = Some(42);
        assert!(bad.admit("native", false, None, Some(source)).is_err());
        let mut bad = rejected.clone();
        bad.diagnostics[0].span = Some((1, source.len() + 1));
        assert!(bad.admit("native", false, None, Some(source)).is_err());
        rejected.diagnostics.clear();
        assert!(rejected.admit("native", false, None, Some(source)).is_err());
    }
}
