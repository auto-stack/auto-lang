use axum::Json;
use serde::{Deserialize, Serialize};
use crate::error::AppError;
use auto_lang::trans::SourceMapEntry;

#[derive(Deserialize)]
pub struct TransRequest {
    pub source: String,
    pub target: String, // "rust" | "c" | "python" | "javascript" | "typescript"
}

#[derive(Serialize)]
pub struct TransResponse {
    pub code: String,
    pub target: String,
    pub source_map: Vec<SourceMapEntry>,
}

pub async fn trans_handler(
    Json(req): Json<TransRequest>,
) -> Result<Json<TransResponse>, AppError> {
    let target = req.target.clone();
    let source = req.source.clone();

    let (code, source_map) = tokio::task::spawn_blocking(move || match target.as_str() {
        "rust" => transpile_rust(&source),
        "c" => transpile_c(&source),
        "python" => transpile_python(&source),
        "javascript" => transpile_javascript(&source),
        "typescript" => transpile_typescript(&source),
        _ => Err(AppError::Internal(format!("Unknown target: {target}"))),
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(TransResponse {
        code,
        target: req.target,
        source_map,
    }))
}

fn transpile_rust(source: &str) -> Result<(String, Vec<SourceMapEntry>), AppError> {
    use auto_lang::trans::rust::transpile_rust as auto_transpile_rust;
    use auto_lang::trans::Sink;

    let mut sink: Sink = auto_transpile_rust("playground", source)
        .map_err(|e| AppError::CompileError(e.to_string()))?;

    let source_map = sink.source_map.clone();
    let output = sink.done().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok((String::from_utf8_lossy(output).to_string(), source_map))
}

fn transpile_c(source: &str) -> Result<(String, Vec<SourceMapEntry>), AppError> {
    use auto_lang::trans::c::transpile_c as auto_transpile_c;
    use auto_lang::trans::Sink;

    let mut sink: Sink = auto_transpile_c("playground", source)
        .map_err(|e| AppError::CompileError(e.to_string()))?;

    let source_map = sink.source_map.clone();
    let output = sink.done().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok((String::from_utf8_lossy(output).to_string(), source_map))
}

fn transpile_python(source: &str) -> Result<(String, Vec<SourceMapEntry>), AppError> {
    use auto_lang::trans::{Sink, Trans};
    use auto_lang::trans::python::PythonTrans;
    use auto_lang::Parser;

    let mut parser = Parser::from(source);
    let ast = parser.parse().map_err(|e| AppError::CompileError(e.to_string()))?;
    let mut sink = Sink::new("playground".into());
    let mut trans = PythonTrans::new("playground".into());
    trans.trans(ast, &mut sink).map_err(|e| AppError::CompileError(e.to_string()))?;

    let source_map = sink.source_map.clone();
    let output = sink.done().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok((String::from_utf8_lossy(output).to_string(), source_map))
}

fn transpile_javascript(source: &str) -> Result<(String, Vec<SourceMapEntry>), AppError> {
    use auto_lang::trans::{Sink, Trans};
    use auto_lang::trans::javascript::JavaScriptTrans;
    use auto_lang::Parser;

    let mut parser = Parser::from(source);
    let ast = parser.parse().map_err(|e| AppError::CompileError(e.to_string()))?;
    let mut sink = Sink::new("playground".into());
    let mut trans = JavaScriptTrans::new("playground".into());
    trans.trans(ast, &mut sink).map_err(|e| AppError::CompileError(e.to_string()))?;

    let source_map = sink.source_map.clone();
    let output = sink.done().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok((String::from_utf8_lossy(output).to_string(), source_map))
}

fn transpile_typescript(source: &str) -> Result<(String, Vec<SourceMapEntry>), AppError> {
    use auto_lang::trans::{Sink, Trans};
    use auto_lang::trans::typescript::TypeScriptTrans;
    use auto_lang::Parser;

    let mut parser = Parser::from(source);
    let ast = parser.parse().map_err(|e| AppError::CompileError(e.to_string()))?;
    let mut sink = Sink::new("playground".into());
    let mut trans = TypeScriptTrans::new("playground".into());
    trans.trans(ast, &mut sink).map_err(|e| AppError::CompileError(e.to_string()))?;

    let source_map = sink.source_map.clone();
    let output = sink.done().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok((String::from_utf8_lossy(output).to_string(), source_map))
}
