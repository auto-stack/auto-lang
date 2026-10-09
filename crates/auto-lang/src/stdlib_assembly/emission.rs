//! Target emission consumes the frozen resolver selections, including full AST
//! declarations which the former C function-fragment route discarded.
use crate::ast::{FnKind, Stmt};
use crate::trans::{Sink, Trans};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub struct CArtifact {
    pub stem: String,
    pub source: Vec<u8>,
    pub header: Vec<u8>,
}

/// Project Auto modules are emitted from their selected source snapshot. Host
/// stdlib modules continue to select their independently inspected Rust callee.
pub fn emit_rust_assembly(
    session: &crate::compile::CompileSession,
    source: &str,
    path: &Path,
) -> crate::AutoResult<(Vec<u8>, Vec<super::reference::ReferenceProof>)> {
    let stdlib = super::loader::repo_stdlib_root()?;
    let project_modules: Vec<_> = session
        .layer_selections
        .iter()
        .filter(|selection| !Path::new(&selection.public_file).starts_with(&stdlib))
        .collect();
    let names: std::collections::HashSet<String> =
        project_modules.iter().map(|s| s.module.clone()).collect();
    if names.iter().any(|name| name.contains('.')) {
        return Err("STDASSEMBLY.SIGNATURE_UNVERIFIED: nested Rust Auto module layout needs an explicit namespace adapter".into());
    }
    let mut output = Vec::new();
    let mut references = Vec::new();
    for selection in project_modules {
        let mut parser = crate::parser::Parser::from(selection.source.as_str());
        parser.set_dest(crate::parser::CompileDest::TransRust);
        let mut ast = parser.parse()?;
        ast.stmts.retain(|stmt| match stmt {
            Stmt::Fn(function) => {
                matches!(function.kind, FnKind::CFunction)
                    || super::loader::source_has_body(&selection.source, function.span)
            }
            _ => true,
        });
        let mut generator = crate::trans::rust::RustTrans::new(selection.module.clone().into());
        generator.assembly_modules = names.clone();
        generator.source_dir = Path::new(&selection.public_file)
            .parent()
            .map(Path::to_path_buf);
        let mut sink = Sink::new(selection.module.clone().into());
        generator.trans(ast, &mut sink)?;
        output.extend(format!("mod {} {{\n", selection.module).as_bytes());
        output.extend(sink.body);
        output.extend(b"\n}\n");
        references.extend(generator.assembly_references);
    }
    let snapshot = std::sync::Arc::new(std::sync::RwLock::new(
        session.type_store().read().unwrap().clone(),
    ));
    let mut parser = crate::parser::Parser::new_with_type_store(source, snapshot.clone());
    parser.set_dest(crate::parser::CompileDest::TransRust);
    let ast = parser.parse()?;
    let mut generator = crate::trans::rust::RustTrans::new("assembly".into());
    generator.assembly_modules = names;
    generator.source_dir = path.parent().map(Path::to_path_buf);
    generator.set_shared_type_store(Some(snapshot));
    let mut sink = Sink::new("assembly".into());
    generator.trans(ast, &mut sink)?;
    output.extend(sink.body);
    references.extend(generator.assembly_references);
    Ok((output, references))
}

pub fn emit_c_assembly(
    session: &crate::compile::CompileSession,
    source: &str,
    path: &Path,
) -> crate::AutoResult<Vec<CArtifact>> {
    let mut modules = BTreeMap::new();
    let mut owners = BTreeMap::new();
    for selected in &session.layer_selections {
        let mut parser = crate::parser::Parser::from(selected.source.as_str());
        parser.set_dest(crate::parser::CompileDest::TransC);
        let mut ast = parser.parse()?;
        // Public semicolon declarations are anchors; a selected definition is
        // emitted once. Unknown/missing implementations remain absent.
        ast.stmts.retain(|stmt| match stmt {
            Stmt::Fn(function) => {
                matches!(function.kind, FnKind::CFunction)
                    || super::loader::source_has_body(&selected.source, function.span)
            }
            _ => true,
        });
        let mut functions = BTreeSet::new();
        for stmt in &ast.stmts {
            if let Stmt::Fn(function) = stmt {
                if !matches!(function.kind, FnKind::CFunction) {
                    let name = function.name.to_string();
                    if let Some(previous) = owners.insert(name.clone(), selected.module.clone()) {
                        return Err(format!("STDASSEMBLY.PROVIDER_CONFLICT: C symbol {name} from {previous} and {} needs namespace adaptation", selected.module).into());
                    }
                    functions.insert(name);
                }
            }
        }
        modules.insert(selected.module.clone(), (functions, ast));
    }
    let module_functions: BTreeMap<_, _> = modules
        .iter()
        .map(|(name, (functions, _))| (name.clone(), functions.clone()))
        .collect();
    let mut artifacts = Vec::new();
    for (module, (_, ast)) in modules {
        let stem = module.replace('.', "_");
        let mut generator = crate::trans::c::CTrans::new(stem.clone().into());
        generator.assembly_modules = module_functions.clone();
        let mut sink = Sink::new(stem.clone().into());
        generator.trans(ast, &mut sink)?;
        artifacts.push(CArtifact {
            stem,
            source: sink.done()?.clone(),
            header: sink.header,
        });
    }
    let snapshot = std::sync::Arc::new(std::sync::RwLock::new(
        session.type_store().read().unwrap().clone(),
    ));
    let mut parser = crate::parser::Parser::new_with_type_store(source, snapshot);
    parser.set_dest(crate::parser::CompileDest::TransC);
    let ast = parser.parse()?;
    for statement in &ast.stmts {
        if let Stmt::Fn(function) = statement {
            if owners.contains_key(function.name.as_str()) {
                return Err(format!("STDASSEMBLY.PROVIDER_CONFLICT: C module symbol {} conflicts with project declaration", function.name).into());
            }
        }
    }
    let stem = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let mut generator = crate::trans::c::CTrans::new(stem.clone().into());
    generator.assembly_modules = module_functions;
    let mut sink = Sink::new(stem.clone().into());
    generator.trans(ast, &mut sink)?;
    artifacts.push(CArtifact {
        stem,
        source: sink.done()?.clone(),
        header: sink.header,
    });
    Ok(artifacts)
}

#[cfg(test)]
mod plan738_emission {
    use super::*;
    #[test]
    #[cfg(windows)]
    #[ignore = "on-demand real MSVC/rustc compilation witness"]
    fn same_source_vm_rust_c_execute_their_selected_bodies() {
        let dir = tempfile::tempdir().unwrap();
        let main = dir.path().join("main.at");
        let source = "use proto: *\nprint(value())\n";
        std::fs::write(&main, source).unwrap();
        std::fs::write(dir.path().join("proto.at"), "pub fn value() int;\n").unwrap();
        for (target, value) in [("vm", 41), ("rs", 42), ("c", 43)] {
            std::fs::write(
                dir.path().join(format!("proto.{target}.at")),
                format!("pub fn value() int {{ return {value} }}\n"),
            )
            .unwrap();
        }
        // C and Rust use their production public entry points, without editing outputs.
        let mut c = crate::compile::CompileSession::new();
        crate::trans_c_with_session(&mut c, main.to_str().unwrap()).unwrap();
        assert!(c.layer_selections[0]
            .context_file
            .as_ref()
            .unwrap()
            .ends_with("proto.c.at"));
        let vcvars = "C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Auxiliary/Build/vcvars64.bat";
        use std::os::windows::process::CommandExt;
        let compile = std::process::Command::new("cmd")
            .args(["/D", "/C"])
            .raw_arg(format!(
                "call \"{vcvars}\" >nul && cl.exe /nologo main.c proto.c /Fe:witness.exe"
            ))
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert!(
            compile.status.success(),
            "MSVC: {} {}",
            String::from_utf8_lossy(&compile.stdout),
            String::from_utf8_lossy(&compile.stderr)
        );
        let result = std::process::Command::new(dir.path().join("witness.exe"))
            .output()
            .unwrap();
        assert!(result.status.success());
        assert_eq!(String::from_utf8_lossy(&result.stdout).trim(), "43");
        let mut rust = crate::compile::CompileSession::new();
        crate::trans_rust_with_session(&mut rust, main.to_str().unwrap()).unwrap();
        assert!(rust.layer_selections[0]
            .context_file
            .as_ref()
            .unwrap()
            .ends_with("proto.rs.at"));
        let rustc = std::process::Command::new("rustc")
            .args(["--edition=2021", "--crate-name", "witness"])
            .arg(dir.path().join("main.a2r.rs"))
            .arg("-o")
            .arg(dir.path().join("rust-witness.exe"))
            .output()
            .unwrap();
        assert!(
            rustc.status.success(),
            "rustc: {}",
            String::from_utf8_lossy(&rustc.stderr)
        );
        let result = std::process::Command::new(dir.path().join("rust-witness.exe"))
            .output()
            .unwrap();
        assert!(result.status.success());
        assert_eq!(String::from_utf8_lossy(&result.stdout).trim(), "42");
        // VM entry uses a source-dir-aware session; the same import is resolved first.
        let mut vm = crate::compile::CompileSession::new();
        vm.add_source_dir(dir.path().to_path_buf());
        vm.resolve_uses(source).unwrap();
        let (_, stdout) = crate::run_with_capture_and_path(source, main.to_str().unwrap()).unwrap();
        assert_eq!(stdout.trim(), "41");
        assert!(vm.layer_selections[0]
            .context_file
            .as_ref()
            .unwrap()
            .ends_with("proto.vm.at"));
    }

    #[test]
    #[cfg(windows)]
    #[ignore = "on-demand real C stdlib IO compilation witness"]
    fn c_public_io_selected_layer_compiles_and_reads_real_file() {
        use std::os::windows::process::CommandExt;
        let dir = tempfile::tempdir().unwrap();
        let main = dir.path().join("main.at");
        std::fs::write(dir.path().join("input.txt"), "A").unwrap();
        std::fs::write(&main, "use auto.io: File\nlet f = File.open(\"input.txt\")\nprint(f.read_char())\nf.close()\n").unwrap();
        let mut session = crate::compile::CompileSession::new();
        crate::trans_c_with_session(&mut session, main.to_str().unwrap()).unwrap();
        let vcvars = "C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Auxiliary/Build/vcvars64.bat";
        let compile = std::process::Command::new("cmd")
            .args(["/D", "/C"])
            .raw_arg(format!(
                "call \"{vcvars}\" >nul && cl.exe /nologo main.c auto_io.c /Fe:witness.exe"
            ))
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert!(
            compile.status.success(),
            "C IO: {} {}",
            String::from_utf8_lossy(&compile.stdout),
            String::from_utf8_lossy(&compile.stderr)
        );
        let result = std::process::Command::new(dir.path().join("witness.exe"))
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert!(result.status.success());
        assert_eq!(String::from_utf8_lossy(&result.stdout).trim(), "65");
    }
}
