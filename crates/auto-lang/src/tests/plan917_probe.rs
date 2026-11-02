//! PLAN-917 T-02/T-03 probe: emit corpus R07/T08 and the legal-neighbor
//! sources through the production legacy A2R entries, dumping the real
//! artifact sets to the plan-artifacts scratch dir. Temporary probe —
//! superseded by the formal case group (case-ace-m1-a2r-emit-917) at T-05.

use std::path::PathBuf;

fn emit_one(corpus: &PathBuf, extra: &PathBuf, out: &PathBuf, name: &str, project: bool) {
    let src_path = if (extra.join(format!("{}.at", name))).is_file() {
        extra.join(format!("{}.at", name))
    } else {
        corpus.join(format!("{}.at", name))
    };
    let src = std::fs::read_to_string(&src_path).unwrap();
    match crate::trans::rust::transpile_rust_with_source_dir(
        src_path.parent().unwrap_or(corpus),
        name,
        &src,
    ) {
        Ok(mut sink) => {
            let bytes = sink.done().unwrap();
            std::fs::write(out.join(format!("{}_single.rs", name)), bytes).unwrap();
        }
        Err(e) => {
            std::fs::write(out.join(format!("{}_single_ERROR.txt", name)), format!("{:?}", e))
                .unwrap();
        }
    }
    if project {
        match crate::trans::rust::transpile_rust_project(src_path.to_str().unwrap()) {
            Ok(map) => {
                for (k, (b, _maps)) in &map {
                    std::fs::write(
                        out.join(format!("{}_proj_{}", name, k.replace('/', "_"))),
                        b,
                    )
                    .unwrap();
                }
            }
            Err(e) => {
                std::fs::write(
                    out.join(format!("{}_proj_ERROR.txt", name)),
                    format!("{:?}", e),
                )
                .unwrap();
            }
        }
    }
}

#[test]
fn plan917_probe_emit_r07_t08() {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let corpus = d.join("test/cases/ace-m1/src");
    let extra = d.join("test/cases/ace-m1/a2r-emit-917/src");
    let out = PathBuf::from("D:/autostack/plan-artifacts/plan-917/probe");
    std::fs::create_dir_all(&out).unwrap();

    for name in ["R07", "T08"] {
        emit_one(&corpus, &extra, &out, name, true);
    }
    for name in [
        "nb_now21",
        "nb_caller21",
        "nb_view",
        "nb_move",
        "nb_two_roots",
        "nb_nested",
        "nb_slice_ok",
    ] {
        emit_one(&corpus, &extra, &out, name, false);
    }
}

#[test]
fn plan917_debug_single_path() {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = std::fs::read_to_string(d.join("test/cases/ace-m1/src/R07.at")).unwrap();
    let mut parser = crate::parser::Parser::from(src.as_str());
    parser.set_dest(crate::parser::CompileDest::TransRust);
    let mut ast = parser.parse().unwrap();
    let mut ctee = crate::comptime::CTEE::new();
    ctee.transform(&mut ast).unwrap();
    let mut dump = String::new();
    for stmt in &ast.stmts {
        match stmt {
            crate::ast::Stmt::TypeDecl(td) => {
                dump.push_str(&format!("TypeDecl {} members:", td.name));
                for m in &td.members {
                    dump.push_str(&format!(" {}={:?}", m.name, m.ty));
                }
                dump.push('\n');
            }
            crate::ast::Stmt::Fn(f) => {
                dump.push_str(&format!("Fn {} params:", f.name));
                for p in &f.params {
                    dump.push_str(&format!(" {}={:?}", p.name, p.ty));
                }
                dump.push('\n');
                for s in &f.body.stmts {
                    if let crate::ast::Stmt::Store(st) = s {
                        dump.push_str(&format!(
                            "  store {} ty={:?} expr={:?}\n",
                            st.name, st.ty, st.expr
                        ));
                    }
                    if let crate::ast::Stmt::Return(e) = s {
                        dump.push_str(&format!("  return {:?}\n", e));
                    }
                }
            }
            _ => {}
        }
    }
    std::fs::write("D:/autostack/plan-artifacts/plan-917/probe/debug_ast.txt", dump).unwrap();
}
