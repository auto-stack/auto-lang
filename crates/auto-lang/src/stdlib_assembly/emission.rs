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
    let stdlib = super::loader::repo_stdlib_root()?;
    let mut modules = BTreeMap::new();
    let mut owners = BTreeMap::new();
    let mut c_opaque_types = std::collections::HashSet::new();
    let mut stdlib_ext_types = std::collections::HashSet::new();
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
        let module_path = Path::new(&selected.public_file);
        if module_path.starts_with(&stdlib) {
            // stdlib auto.* 目标层的 ext body 无 C 发射（stdlib 的 C
            // provider = c.* 声明模块 + libc）。只登记方法面（公共
            // `type X { fn ... }` 与目标层 `ext X { fn ... }` 两形态）
            // 供诚实目标诊断消费，不转译出坏产物（与 Rust 发射的项目
            // 模块过滤对称）。
            for stmt in &ast.stmts {
                match stmt {
                    Stmt::Ext(ext) => {
                        stdlib_ext_types.insert(ext.target.as_str().to_string());
                    }
                    Stmt::TypeDecl(decl) if !decl.methods.is_empty() => {
                        stdlib_ext_types.insert(decl.name.as_str().to_string());
                    }
                    _ => {}
                }
            }
            continue;
        }
        if selected.module.starts_with("c.") {
            // `#[c] type X;` 不透明名按 C 符号本名渲染（如 `*FILE` → FILE*）。
            for stmt in &ast.stmts {
                if let Stmt::TypeDecl(decl) = stmt {
                    if decl.kind == crate::ast::TypeDeclKind::CType
                        || decl.attrs.iter().any(|attr| attr.as_str() == "c")
                    {
                        c_opaque_types.insert(decl.name.as_str().to_string());
                    }
                }
            }
        }
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
        generator.c_opaque_types = c_opaque_types.clone();
        generator.stdlib_ext_types = stdlib_ext_types.clone();
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
    generator.c_opaque_types = c_opaque_types;
    generator.stdlib_ext_types = stdlib_ext_types;
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

    /// C 目标的既有 stdlib IO 能力 = `use c.stdio` 直接绑定（stdlib/c/README
    /// 记载的合同；`#[c]` 声明 → libc 链接）。真实 MSVC 实编实跑：读文件
    /// 首字节 'A' → 65；`*FILE` 按符号本名渲染为 FILE*，不退化 void**。
    /// stdlib auto.* ext 面（File.open 等）无 C 发射，见负测
    /// `c_session_rejects_stdlib_ext_face`。
    #[test]
    #[cfg(windows)]
    #[ignore = "on-demand real C stdlib IO compilation witness"]
    fn c_stdio_provider_bindings_compile_and_read_real_file() {
        use std::os::windows::process::CommandExt;
        let dir = tempfile::tempdir().unwrap();
        let main = dir.path().join("main.at");
        std::fs::write(dir.path().join("input.txt"), "A").unwrap();
        std::fs::write(&main, "use c.stdio: fopen, fgetc, fclose, FILE\n\nfn main() {\n    let f *FILE = fopen(\"input.txt\", c\"r\")\n    print(fgetc(f))\n    fclose(f)\n}\n").unwrap();
        let mut session = crate::compile::CompileSession::new();
        crate::trans_c_with_session(&mut session, main.to_str().unwrap()).unwrap();
        let emitted = std::fs::read_to_string(dir.path().join("main.c")).unwrap();
        assert!(
            emitted.contains("FILE* f"),
            "opaque FILE renders by name: {emitted}"
        );
        let vcvars = "C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Auxiliary/Build/vcvars64.bat";
        let compile = std::process::Command::new("cmd")
            .args(["/D", "/C"])
            .raw_arg(format!(
                "call \"{vcvars}\" >nul && cl.exe /nologo main.c /Fe:witness.exe"
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
        assert!(
            result.status.success(),
            "witness run: {} {}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&result.stdout).trim(), "65");
    }

    /// stdlib auto.* 的 ext 面（File.open 等）在 C 会话无发射——诚实目标
    /// 诊断（§5.3/§6.1：不造实现，也不产坏 C 裸透传）；仅导入未引用 ext
    /// 不在此拒绝（导入本身不是引用）。
    #[test]
    fn c_session_rejects_stdlib_ext_static_face() {
        let dir = tempfile::tempdir().unwrap();
        let main = dir.path().join("main.at");
        std::fs::write(
            &main,
            "use auto.io: File\n\nfn main() {\n    let f = File.open(\"x\")\n    print(f)\n}\n",
        )
        .unwrap();
        let mut session = crate::compile::CompileSession::new();
        let error = crate::trans_c_with_session(&mut session, main.to_str().unwrap()).unwrap_err();
        let message = error.to_string();
        assert!(
            message.contains("STDASSEMBLY.TARGET_UNSUPPORTED"),
            "{message}"
        );
        assert!(message.contains("File.open"), "{message}");
    }
}
