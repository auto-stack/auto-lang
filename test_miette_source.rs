// Test miette source code display
use auto_lang::error::{pos_to_span, SyntaxError, SyntaxErrorWithSource};
use auto_lang::token::Pos;
use miette::{Diagnostic, NamedSource, SourceSpan};

fn main() {
    let source_code = "let x: int = \"invalid\"";
    let syntax_err = SyntaxError::Generic {
        message: "type error".to_string(),
        span: pos_to_span(Pos {
            line: 1,
            at: 10,
            pos: 10,
            len: 10,
        }),
    };

    let err = SyntaxErrorWithSource {
        source: NamedSource::new("test.at".to_string(), source_code.to_string()),
        error: syntax_err,
    };

    // Check if source_code() works
    if let Some(source) = err.source_code() {
        println!("✓ Source code accessible: {} bytes", source.len());

        // Try to read a line
        if let Some(line) = source.read_line(&miette::MietteSpan::new(10, 10)) {
            println!("✓ Line read: {:?}", line);
        }
    }

    // Try to display with miette
    println!("\n--- Diagnostic Display ---");
    let handler = miette::MietteHandlerOpts::new().build();
    handler.render_report(&err, &mut std::io::stdout()).unwrap();
}
