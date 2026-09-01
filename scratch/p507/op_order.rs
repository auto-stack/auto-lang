// scratch probe: op order for a bg container with children
#[test]
fn probe_op_order() {
    let src = r#"widget P {
    view {
        col {
            text "visible?"
            style: "p-2 bg-slate-800"
        }
    }
}
"#;
    let component = auto_lang::build_dynamic_component(src, None).unwrap();
    let mut p = auto_lang::ui::desktop_protocol::client_runtime::AppProjector::new(component, 400.0, 300.0);
    use auto_lang::ui::desktop_protocol::endpoint::FrameSource;
    let frame = p.render_frame();
    for (i, op) in frame.ops.iter().enumerate() {
        match op {
            auto_lang::ui::desktop_protocol::message::DrawOp::Text { text, .. } => println!("[{i}] TEXT {text:?}"),
            auto_lang::ui::desktop_protocol::message::DrawOp::Quad { .. } => println!("[{i}] QUAD"),
        }
    }
}
