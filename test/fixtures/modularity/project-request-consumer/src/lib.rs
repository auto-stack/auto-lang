//! PLAN-809 T-02 V-02: external consumer of the generation request data
//! contract ("I-PROJECT"). Consumes only `auto-project-model` — proving the
//! request is light (no language/Man/UI stack in the closure) and complete
//! enough to carry the generator-facing fields both from a statically
//! admitted pac.at and from a dynamically evaluated projection.

use auto_project_model::descriptor::SourceInfo;
use auto_project_model::{parse_document, GenerationRequest, RequestMode, RequestOrigin};

fn static_request(text: &str) -> GenerationRequest {
    let descriptor = parse_document(text, SourceInfo::inline()).expect("static parse ok");
    GenerationRequest::from_descriptor(&descriptor)
}

/// A generator-side read of the full A-group face: identity, display,
/// runtime, render selection, toggles — all field access, no Pac involved.
#[test]
fn consumer_reads_static_request_fields() {
    let req = static_request(
        r#"
        name: "contact-form"
        render: "vue"
        title: "Contact"
        title_zh: "联系表单"
        window: "fit"
        shadcn: off
        api: "rust"
        front_port: 3018
        dep "common" {
            path: "../common"
        }
        "#,
    );
    assert_eq!(req.identity.name.as_deref(), Some("contact-form"));
    assert_eq!(req.frontend_names(), vec!["vue".to_string()]);
    assert_eq!(req.display_title(true).as_deref(), Some("联系表单"));
    assert_eq!(req.display_title(false).as_deref(), Some("Contact"));
    assert!(req.window_fit());
    assert!(!req.shadcn());
    assert!(req.default_classes());
    assert_eq!(req.runtime.front_port, Some(3018));
    assert_eq!(req.dep_roots.len(), 1);
    assert_eq!(req.dep_roots[0].path.as_deref(), Some("../common"));
    assert!(req.is_evaluated());
}

/// The dynamic arm: adapter fills evaluated values directly; the request
/// stays honest about origin and mode.
#[test]
fn consumer_builds_dynamic_request() {
    let mut req = GenerationRequest::dynamic(SourceInfo::from_path("proj/pac.at"));
    req.identity.name = Some("dyn".into());
    req.mode = RequestMode::Run;
    req.input_sources.push("proj/main.at".into());
    req.output_root = Some("proj/out".into());
    assert!(matches!(req.origin, RequestOrigin::DynamicEvaluated { .. }));
    assert!(req.is_evaluated());
    assert_eq!(req.mode, RequestMode::Run);
    // path algebra against the source root, not the consumer's cwd
    assert_eq!(
        req.resolve_against_root("main.at"),
        Some(std::path::PathBuf::from("proj/main.at"))
    );
}

/// Unsupported static constructs never fake evaluation (V-03 honesty at the
/// external boundary).
#[test]
fn consumer_sees_unsupported_as_not_evaluated() {
    let req = static_request("name: \"x\"\ntitle: \"${env.NAME}\"\n");
    assert!(!req.is_evaluated());
    assert_eq!(req.display.title, None);
    match req.origin {
        RequestOrigin::StaticAdmitted { .. } => {}
        RequestOrigin::DynamicEvaluated { .. } => panic!("static origin expected"),
    }
}
