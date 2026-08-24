#[test]
fn payload_probe() {
    let code = std::fs::read_to_string(
        "../../examples/widgets-gallery/src/front/components/combobox.at",
    )
    .unwrap();
    let session = auto_lang::session::CompilerSession::new(auto_lang::session::Scenario::UI);
    let mut parser = auto_lang::Parser::from(code.as_str());
    parser = parser.with_session(session);
    let ast = parser.parse().expect("parse");
    for stmt in &ast.stmts {
        if let auto_lang::ast::Stmt::WidgetDecl(w) = stmt {
            for msg in &w.messages {
                for v in &msg.variants {
                    eprintln!(
                        "{}::{} payload={:?} names={:?}",
                        w.name.as_str(),
                        v.name.as_str(),
                        v.payload,
                        v.payload_names
                    );
                }
            }
        }
    }
}
