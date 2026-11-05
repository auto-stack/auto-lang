//! Strict source-import certification report mirror. No experimental compiler
//! dependency enters the language SDK; the existing ABI transport is reused.
use serde::{Deserialize, Serialize};
pub(super) const RECIPE: &str = "abi/source-foreign-certify-v1";
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
    pub fn admit(
        &self,
        consumer: &str,
        success: bool,
        exit: Option<i32>,
        source: Option<&str>,
    ) -> Result<(), String> {
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
            || !digest(&self.contract_digest)
            || !digest(&self.header_digest)
            || !digest(&self.physical_table_digest)
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
                || !self.c_toolchain.starts_with("cl:")
                || !self.c_toolchain.contains("|hash:")
                || self.c_toolchain.len() > 8192
                || self.actual_events != self.reference_events
                || self.reference_events != ["provider_add".to_string()]
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
    fn valid() -> SourceReport {
        let source = "fn main() int { return foreign_add(20,22) }";
        serde_json::from_value(serde_json::json!({
            "schema":"abi.source-import-report/1","direction":"source-import","input_profile":"auto.source.scalar-i32.draft",
            "abi_domain":"autoabi","abi_version":"0.1","abi_profile":"autoabi.win-x64.v0.1-candidate","callconv":"windows-x64",
            "target":"x86_64-pc-windows-msvc","consumer":"native","source_sha256":auto_test_case::sha256::hex(&auto_test_case::sha256::sha256(source.as_bytes())),
            "compiler":"ace-native-foreign/1+ace-native/0.1.0+cranelift/0.126.2","c_toolchain":"cl:reviewed-tool|ver:1|hash:live|flags:frozen",
            "contract_digest":"a".repeat(64),"provider_digest":"b".repeat(64),"header_digest":"c".repeat(64),"physical_table_digest":"d".repeat(64),
            "review_id":"plan938-independent-provider-v1","actual_exit":42,"reference_exit":42,
            "actual_events":["provider_add"],"reference_events":["provider_add"],"actual_stderr":"source938.call=provider_add\n","reference_stderr":"source938.call=provider_add\n",
            "source_parse":2,"solver":1,"hir":1,"mir":1,"native":1,"c_fragment":0,"diagnostic_stage":"","diagnostics":[]
        })).unwrap()
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
