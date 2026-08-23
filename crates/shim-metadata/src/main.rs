//! shim-metadata CLI(plan-430)。
//!
//! 用法:
//!   shim-metadata std-plan                    # std 目录 → 分类 + 例外 → 报告
//!   shim-metadata std-emit                    # 生成 std 追加段源码到 stdout
//!   shim-metadata parse-rustdoc <file.json>   # 三方 crate rustdoc → 方法清单报告
//!   shim-metadata shim-plan <file.json>       # 三方 crate rustdoc → 分类计划 + 跳过清单
//!   shim-metadata shim-emit-pack <file.json> --crate X --version V --dep-line '...'
//!                                             # 三方 crate rustdoc → 完整 shim 包文件集到 stdout

mod classify;
mod emit;
mod emit_cdylib;
mod rustdoc;
mod std_catalog;
mod types;

use classify::{classify_all, classify_all_third_party, Exceptions};

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
        Some("shim-plan") => {
            let path = args
                .get(2)
                .expect("usage: shim-plan <file.json> [--rules rules.json]");
            let doc = std::fs::read_to_string(path).expect("read rustdoc json");
            let exc = load_rules(args.get(3));
            match rustdoc::parse_all(&doc) {
                Ok(parsed) => {
                    let c = classify_all_third_party(&parsed.methods, &exc);
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
                    println!(
                        "== free functions ({}) ==",
                        parsed.free_fns.len()
                    );
                    for f in &parsed.free_fns {
                        println!(
                            "{n}({p}) -> {r}",
                            n = f.method,
                            p = f.params.iter().map(|t| t.rust_name()).collect::<Vec<_>>().join(","),
                            r = f.ret.rust_name()
                        );
                    }
                }
                Err(e) => {
                    eprintln!("parse error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Some("shim-emit-pack") => {
            // shim-emit-pack <file.json> --crate X --version V --dep-line '...' [--rules rules.json]
            let path = args
                .get(2)
                .expect("usage: shim-emit-pack <file.json> --crate X --version V --dep-line '...'");
            let flag = |name: &str| {
                args.iter()
                    .position(|a| a == name)
                    .and_then(|i| args.get(i + 1))
                    .cloned()
            };
            let crate_name = flag("--crate").expect("--crate required");
            let crate_version = flag("--version").unwrap_or_else(|| "0.0.0".into());
            let dep_line = flag("--dep-line").unwrap_or_else(|| format!("{crate_name} = \"1\""));
            let rules_path = args.iter().position(|a| a == "--rules").and_then(|i| args.get(i + 1)).cloned();

            let doc = std::fs::read_to_string(path).expect("read rustdoc json");
            let exc = match rules_path {
                Some(p) => serde_json::from_str(&std::fs::read_to_string(&p).expect("read rules json"))
                    .expect("parse rules json"),
                None => Exceptions::default(),
            };
            let parsed = rustdoc::parse_all(&doc).expect("parse rustdoc");
            let c = classify_all_third_party(&parsed.methods, &exc);
            let meta = emit_cdylib::PackMeta {
                crate_name,
                crate_version,
                toolchain: format!("rustdoc v53 ({})", std::env::consts::OS),
            };
            let (fp, files) = emit_cdylib::emit_pack(&meta, &dep_line, &c, &exc, &parsed.free_fns);
            eprintln!(
                "fingerprint={fp} methods={} skips={} free_fns={}",
                c.plans.len(),
                c.skips.len(),
                parsed.free_fns.len()
            );
            println!("===== Cargo.toml =====");
            println!("{}", files.cargo_toml);
            println!("===== src/lib.rs =====");
            println!("{}", files.lib_rs);
            println!("===== manifest.json =====");
            println!("{}", files.manifest_json);
            println!("===== signatures.json =====");
            println!("{}", files.signatures_json);
            println!("===== rules.json =====");
            println!("{}", files.rules_json);
        }
        _ => {
            eprintln!(
                "usage: shim-metadata <std-plan|std-emit|parse-rustdoc f.json|shim-plan f.json|shim-emit-pack f.json --crate X --version V --dep-line '...'>"
            );
            std::process::exit(2);
        }
    }
}

fn load_rules(flag_arg: Option<&String>) -> Exceptions {
    match flag_arg {
        Some(s) if s == "--rules" => Exceptions::default(),
        Some(p) => serde_json::from_str(&std::fs::read_to_string(p).unwrap_or_default().as_str())
            .unwrap_or_default(),
        _ => Exceptions::default(),
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
