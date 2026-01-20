use auto_lang::{
    parser::Parser,
    trans::{c::CTrans, Sink, Trans},
    Universe,
};
use auto_val::AutoPath;

use miette::{IntoDiagnostic, Result};
use std::{cell::RefCell, fs, path::Path, rc::Rc};
use walkdir::WalkDir;

pub fn run() -> Result<()> {
    let stdlib_path = Path::new("stdlib");
    if !stdlib_path.exists() {
        return Err(miette::miette!(
            "stdlib directory not found at '{}'",
            stdlib_path.display()
        ));
    }

    println!("Transpiling stdlib...");

    let mut files = Vec::new();
    for entry in WalkDir::new(stdlib_path) {
        let entry = entry.into_diagnostic()?;
        let path = entry.path();
        let path_str = path.to_string_lossy();
        // Skip .vm.at files
        if path.is_file()
            && (path_str.ends_with(".at") || path_str.ends_with(".c.at"))
            && !path_str.ends_with(".vm.at")
        {
            files.push(path.to_path_buf());
        }
    }

    if files.is_empty() {
        println!("No files found to transpile.");
        return Ok(());
    }

    for path in files {
        let path_str = path.to_string_lossy();
        println!("Transpiling {} ...", path_str);

        let code = fs::read_to_string(&path).into_diagnostic()?;

        // Calculate output filenames
        // Handle .c.at -> .c carefully to avoid .c.c
        let c_path_str = if path_str.ends_with(".c.at") {
            path_str.replace(".c.at", ".c.c")
        } else {
            path_str.replace(".at", ".c")
        };
        let h_path_str = if path_str.ends_with(".c.at") {
            path_str.replace(".c.at", ".c.h")
        } else {
            path_str.replace(".at", ".h")
        };

        let fname = AutoPath::new(path_str.as_ref()).filename();

        let scope = Rc::new(RefCell::new(Universe::new()));
        let mut parser = Parser::new(&code, scope.clone());
        let ast = match parser.parse() {
            Ok(ast) => ast,
            Err(e) => {
                println!("Error parsing {}: {:?}", path_str, e);
                return Err(miette::miette!("{}", e));
            }
        };

        let mut sink = Sink::new(fname);
        let mut trans = CTrans::new(c_path_str.clone().into());
        trans.set_scope(parser.scope.clone());

        trans
            .trans(ast, &mut sink)
            .map_err(|e| miette::miette!("{}", e))?;

        // Write C file
        fs::write(
            &c_path_str,
            sink.done().map_err(|e| miette::miette!("{}", e))?,
        )
        .into_diagnostic()?;

        // Write Header file
        fs::write(&h_path_str, sink.header).into_diagnostic()?;
    }

    Ok(())
}
