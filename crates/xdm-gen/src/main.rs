pub mod trans;
pub mod var;

use auto_lang::Parser;
use auto_lang::trans::Sink;
use auto_lang::types::TypeStore;
use clap::Parser as ClapParser;
use std::sync::{Arc, RwLock};

use crate::trans::XdmTrans;

const XDM_GEN_VERSION: &str = "v0.1.1";

const XDM_HEAD: &str = r#"
// XDM type aliases — used as kind: values in node bodies
var bool = "bool"
var int = "int"
var float = "float"
var str = "str"

// XDM definitions
type select {
    options []str
}

type v {
    id str
    desc str
    default str
    enable bool
    uuid str
    origin str = "AUTOSAR_ECUC"
    scope str = "LOCAL"
}

type ref {
    id str
    desc str
    default str
    enable bool
    uuid str
    origin str = "AUTOSAR_ECUC"
    scope str = "LOCAL"
}"#;

#[derive(ClapParser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    // Path to the xdm.at file
    path: Option<String>,
}

pub fn trans_xdm(path: &str) -> Result<String, String> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let code = format!("{}\n{}", XDM_HEAD, code);
    let dest = path.replace(".at", ".xdm");

    let type_store = Arc::new(RwLock::new(TypeStore::new()));
    let mut parser = Parser::new_with_type_store(code.as_str(), type_store.clone());
    let ast = parser.parse().map_err(|e| e.to_string())?;

    let sink = Sink::new(path.into());
    let mut trans = XdmTrans::new(type_store, sink);
    trans.trans(ast).map_err(|e| e.to_string())?;

    let mut sink = trans.finish().map_err(|e| e.to_string())?;
    let out = sink.done().map_err(|e| e.to_string())?;
    println!("OUT: {}", String::from_utf8(out.clone()).unwrap());

    // write to .xdm file
    std::fs::write(&dest, &out).map_err(|e| e.to_string())?;
    Ok(format!("[trans] {} -> {}", path, dest))
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    println!("----------------------------------");
    println!("XDM Generator {}", XDM_GEN_VERSION);
    println!("----------------------------------");
    println!();

    let template_file = if let Some(path) = &cli.path {
        path.clone()
    } else {
        "./config/xdm.at".to_string()
    };

    // Spawn a thread with larger stack size to avoid stack overflow
    // from deeply nested parser recursion (xdm.at can have many nesting levels)
    // Spawn a thread with larger stack size to avoid stack overflow
    // from deeply nested parser recursion (xdm.at can have many nesting levels)
    let child = std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024) // 8 MB stack
        .spawn(move || trans_xdm(&template_file))
        .map_err(|e| e.to_string())?;

    let result: Result<String, String> = child.join().map_err(|e| format!("{:?}", e))?;
    let _ = result?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use auto_val::AutoStr;

    #[test]
    fn test_auto_str() {
        let s: AutoStr = "Hello".into();
        assert_eq!(s, "Hello");
    }
}
