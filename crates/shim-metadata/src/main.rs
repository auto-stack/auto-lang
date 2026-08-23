//! shim-metadata CLI(plan-430)。
//!
//! 用法:
//!   shim-metadata std-plan                    # std 目录 → 分类 + 例外 → 报告
//!   shim-metadata std-emit                    # 生成 std 追加段源码到 stdout
//!   shim-metadata parse-rustdoc <file.json>   # 三方 crate rustdoc → 方法清单报告

mod classify;
mod emit;
mod rustdoc;
mod std_catalog;
mod types;

use classify::{classify_all, Exceptions};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("std-plan") => cmd_std_plan(),
        Some("std-emit") => cmd_std_emit(),
        Some("parse-rustdoc") => {
            let path = args.get(2).expect("usage: parse-rustdoc <file.json>");
            let doc = std::fs::read_to_string(path).expect("read rustdoc json");
            match rustdoc::parse(&doc) {
                Ok(methods) => {
                    for m in &methods {
                        println!(
                            "{t}.{m}\tself={s:?}\tparams=[{p}]\tret={r}\tgeneric={g}",
                            t = m.type_name,
                            m = m.method,
                            s = m.self_kind,
                            p = m.params.iter().map(|t| t.rust_name()).collect::<Vec<_>>().join(","),
                            r = m.ret.rust_name(),
                            g = m.generic,
                        );
                    }
                    println!("# {} methods", methods.len());
                }
                Err(e) => {
                    eprintln!("parse error: {e}");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("usage: shim-metadata <std-plan|std-emit|parse-rustdoc file.json>");
            std::process::exit(2);
        }
    }
}

fn cmd_std_plan() {
    let methods = std_catalog::std_methods();
    let exc = Exceptions::default();
    let c = classify_all(&methods, &exc);
    println!("== plans ({}) ==", c.plans.len());
    for p in &c.plans {
        println!(
            "{t}.{m}: ret={r:?} args={a:?} self={s:?}",
            t = p.method.type_name,
            m = p.method.method,
            r = p.ret,
            a = p.args,
            s = p.method.self_kind
        );
    }
    println!("== skips ({}) ==", c.skips.len());
    for s in &c.skips {
        println!("{}.{}: {}", s.type_name, s.method, s.reason);
    }
}

fn cmd_std_emit() {
    let methods = std_catalog::std_methods();
    let exc = Exceptions::default();
    let c = classify_all(&methods, &exc);
    print!("{}", emit::emit_std_append_segment(&c.plans));
}
