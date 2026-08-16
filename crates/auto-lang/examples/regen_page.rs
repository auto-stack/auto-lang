//! 一次性工具:把指定 .at 页面重新生成为 Vue SFC(与 gallery gen/ 产物
//! 同管线:parse → extract → VueGenerator)。
//! 用法: cargo run -p auto-lang --example regen_page --features ui-iced -- <input.at> <output.vue> [shadcn|plain|library]
//! 模式默认 shadcn(gallery 的 gen 产物即该模式;必须与原产物一致,
//! 否则 button 等组件会退化为裸 HTML)。

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: regen_page <input.at> <output.vue> [shadcn|plain|library]");
        std::process::exit(2);
    }
    let mode = match args.get(3).map(|s| s.as_str()) {
        Some("plain") => auto_lang::ui_gen::vue::VueMode::Plain,
        Some("library") => auto_lang::ui_gen::vue::VueMode::Library,
        _ => auto_lang::ui_gen::vue::VueMode::Shadcn,
    };
    let code = std::fs::read_to_string(&args[1]).expect("read .at");
    let session = auto_lang::session::CompilerSession::ui();
    let mut parser = auto_lang::Parser::from(code.as_str()).with_session(session);
    let ast = parser.parse().expect("parse");
    let decl = ast
        .stmts
        .iter()
        .find_map(|s| match s {
            auto_lang::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        })
        .expect("widget decl");
    let widget = auto_lang::aura::extract_widget_from_decl(decl).expect("extract");
    use auto_lang::ui_gen::BackendGenerator;
    let sfc = auto_lang::ui_gen::VueGenerator::new()
        .with_mode(mode)
        .generate(&widget)
        .expect("vue gen");
    std::fs::write(&args[2], &sfc).expect("write");
    println!("written {} ({} bytes, mode={:?})", args[2], sfc.len(), mode);
}
