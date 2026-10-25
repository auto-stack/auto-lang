use auto_man_core::{select_port_binding, Port};
use auto_project_model::{descriptor::SourceInfo, parse_document, GenerationRequest};
use auto_tool_protocol::{write_frame, ClientBody, ClientEnvelope, WireRequest, PROTOCOL_VERSION};
use std::io::Cursor;

#[test]
fn management_rules_execute_outside_their_source_crate() {
    let mut port = Port::default();
    port.builder = "ninja".into();
    port.at = "build/".into();
    let selected = select_port_binding(&port, &auto_val::AutoStr::from("c"));
    assert_eq!(selected.builder_name.as_str(), "ninja");
    assert_eq!(selected.build_at.as_str(), "build");
    assert!(selected.show_headers);
}

#[test]
fn language_sdk_executes_a_real_program_without_ui() {
    let (result, stdout) = auto_lang_core::run_with_capture("fn main() {\n    print(6 * 7)\n}\n")
        .expect("Core program must execute");
    assert!(result.is_empty() || result.contains("ok") || !stdout.is_empty());
    assert!(stdout.contains("42"), "stdout={stdout:?} result={result:?}");
}

#[test]
fn protocol_round_trip_uses_the_language_owned_request_type() {
    let descriptor = parse_document(
        "name: \"three-repo\"\nrender: \"vue\"\nwindow: \"fit\"\n",
        SourceInfo::from_path("consumer/pac.at"),
    )
    .expect("parse I-PROJECT descriptor");
    let request = GenerationRequest::from_descriptor(&descriptor);
    let envelope = ClientEnvelope {
        protocol_version: PROTOCOL_VERSION,
        id: 832,
        body: ClientBody::ValidateProject {
            request: WireRequest(request),
        },
    };
    let mut framed = Vec::new();
    write_frame(
        &mut framed,
        &serde_json::to_value(&envelope).expect("serialize envelope"),
        8192,
    )
    .expect("frame envelope");
    let decoded: ClientEnvelope = serde_json::from_value(
        auto_tool_protocol::read_frame(&mut Cursor::new(framed), 8192)
            .expect("read frame")
            .expect("one frame"),
    )
    .expect("decode same envelope type");
    assert_eq!(decoded, envelope);
    assert_eq!(PROTOCOL_VERSION, 1);
}
