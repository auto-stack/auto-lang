// PLAN-759 post-landing diagnostic probe; use the production auto_lang library.
fn main() {
    for tag in ["command_input", "command-input", "command_item", "command-item"] {
        let support = auto_lang::ui::render_support::get_support(tag);
        println!(
            "SUPPORT {tag}: {:?}; ignored={:?}; note={}",
            support.level, support.ignored_props, support.note
        );
    }
    let source = r#"widget App {
        model { var query str = "hello" }
        view { col { command-input (placeholder: "Search", value: .query) {} } }
    }"#;
    let component = auto_lang::build_dynamic_component(source, None).expect("build input fixture");
    let (view, _, _) = component.view_with_debug_gated(false);
    println!("ACTUAL_VIEW {view:?}");
}
