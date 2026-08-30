//! Plan 022 Phase 3 (auto-down jade-garden) VM server probe.
//!
//! jade back embeds the AutoVM via `run_file` and serves its /api/* surface
//! from axum-adapter route chains (Plan 442-c2 pattern). The full jade
//! route module panics the VM at compile time with a bad-opcode transmute
//! (`opcode.rs OpCode::from` invalid value 0x28/0x29) while a single
//! no-arg-closure route compiles+runs clean — so the trigger is a specific
//! handler/router construct, bisected here against a standalone temp dir
//! (entry + module, no sibling checkout needed).
//!
//! `#[ignore]` — manual bisect driver:
//!   cargo test -p auto-lang --lib auto_down_vm_probe -- --ignored --nocapture

#[cfg(test)]
mod auto_down_vm_probe {
    /// One bisect case = (name, module source). The entry file is fixed.
    /// run_file serves forever once routes install, so it runs on a
    /// detached big-stack thread and we poll the route table (442-c2
    /// probe pattern); the process exits when all tests finish.
    fn case(name: &str, module_src: &str) {
        let dir = std::env::temp_dir().join(format!("auto-down-probe-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        std::fs::write(
            dir.join("jade_probe.at"),
            format!("dep axum\ndep serde_json\n\nuse jade_module\n\nfn main() {{\n    let app = build_router()\n    print(\"built\")\n}}\n"),
        )
        .unwrap();
        std::fs::write(dir.join("jade_module.at"), module_src).unwrap();

        // Route tables are process-global — clear between cases. The
        // per-case server binds a throwaway high port; clashes only make
        // run_file return early, which still counts as compile-clean.
        crate::vm::ffi::stdlib::clear_http_routes();
        crate::vm::ffi::axum_adapter::reset();

        let entry = dir.join("jade_probe.at").to_string_lossy().to_string();
        let label = name.to_string();
        let _server = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(move || match crate::run_file(&entry) {
                Ok(_) => eprintln!("[auto-down-probe] {label}: run_file returned"),
                Err(e) => panic!("[auto-down-probe] {label}: run_file failed: {e:?}"),
            })
            .unwrap();

        let mut route_count = 0usize;
        for _ in 0..300 {
            route_count = crate::vm::ffi::axum_adapter::installed_routes().len();
            if route_count > 0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        if route_count == 0 {
            panic!("[auto-down-probe] {name}: no routes installed within 30s");
        }
        eprintln!("[auto-down-probe] {name}: OK ({route_count} route(s) installed)");
    }

    /// Baseline (proven green from the jade embed): single no-arg route.
    #[test]
    #[ignore]
    fn baseline_single_no_arg_route() {
        case(
            "baseline",
            r#"
use.rs axum::Router
use.rs axum::routing::get
use.rs serde_json::Value

fn graphGet() Value {
    return json.parse("{}")
}

pub fn build_router() Router {
    var app = Router.new()
    app = app.route("/api/graph", get(graphGet))
    return app
}
"#,
        );
    }

    /// One extractor param typed Query<Value>.
    #[test]
    #[ignore]
    fn single_query_value_param() {
        case(
            "query-value",
            r#"
use.rs axum::Router
use.rs axum::routing::get
use.rs axum::extract::Query
use.rs serde_json::Value

fn filesList(q Query<Value>) Value {
    return json.parse("{}")
}

pub fn build_router() Router {
    var app = Router.new()
    app = app.route("/api/files", get(filesList))
    return app
}
"#,
        );
    }

    /// One Path<str> param.
    #[test]
    #[ignore]
    fn single_path_param() {
        case(
            "path-str",
            r#"
use.rs axum::Router
use.rs axum::routing::get
use.rs axum::extract::Path
use.rs serde_json::Value

fn wikiRead(p Path<str>) Value {
    return json.parse("{}")
}

pub fn build_router() Router {
    var app = Router.new()
    app = app.route("/api/wiki/{*path}", get(wikiRead))
    return app
}
"#,
        );
    }

    /// Two extractor params (Path + Query), like the jade wiki route.
    #[test]
    #[ignore]
    fn two_extractor_params() {
        case(
            "path-query",
            r#"
use.rs axum::Router
use.rs axum::routing::get
use.rs axum::extract::Query
use.rs axum::extract::Path
use.rs serde_json::Value

fn wikiRead(p Path<str>, q Query<Value>) Value {
    return json.parse("{}")
}

pub fn build_router() Router {
    var app = Router.new()
    app = app.route("/api/wiki/{*path}", get(wikiRead))
    return app
}
"#,
        );
    }

    /// host.call inside the handler (the host bridge delegation).
    #[test]
    #[ignore]
    fn host_call_handler() {
        case(
            "host-call",
            r#"
use.rs axum::Router
use.rs axum::routing::get
use.rs serde_json::Value

fn callApi(method str, path str, q Value, body Value) Value {
    let env = "{\"method\":\"" + method + "\",\"path\":\"" + path + "\"}"
    let resp = host.call("jade.api", env)
    let v Value = json.parse(resp)
    return json.get(v, "body")
}

fn graphGet() Value {
    return callApi("GET", "/api/graph", json.parse("{}"), json.parse("null"))
}

pub fn build_router() Router {
    var app = Router.new()
    app = app.route("/api/graph", get(graphGet))
    return app
}
"#,
        );
    }

    /// The full jade route module shape (26 routes, mixed extractors).
    /// Source via `JADE_MODULE_AT=<path to jade_server.at>`; skips if unset.
    #[test]
    #[ignore]
    fn full_jade_router() {
        let Some(path) = std::env::var("JADE_MODULE_AT").ok() else {
            eprintln!("[auto-down-probe] full-jade: SKIPPED — set JADE_MODULE_AT");
            return;
        };
        let src = std::fs::read_to_string(&path).unwrap();
        case("full-jade", &src);
    }
}
