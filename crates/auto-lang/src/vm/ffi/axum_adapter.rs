//! Plan 442 C2 serve adapter: axum-shaped Router assembly + extractor
//! marshalling onto the `#[api]` HTTP-server machinery.
//!
//! musk's backend builds routers with axum idioms — `Router.new()`,
//! `app.route("/x/{id}", get(h1).post(h2))` — and its handlers take
//! extractor-typed params (`s State<AppState>`, `body Json<T>`,
//! `q Query<T>`, `p Path<T>`). This adapter lets the SAME source run on
//! the AutoVM:
//!
//! - `Router`/`MethodRouter` become heap objects (RustStdlibObject tagged
//!   `axum::Router`/`axum::MethodRouter`, so CALL_SPEC's `contains("::")`
//!   funnel routes `.route`/`.post` back here via dispatch 3000).
//! - `get(h)`/`post(h)`/... (bare calls, type_name "") capture the handler
//!   fn-ref (Plan 383 CLOSURE id) and resolve its param shapes by inverting
//!   `exports_by_name` against the closure's `func_addr`, then looking up
//!   PARAM_SIGS (codegen side-channel, same pattern as api_routes).
//! - `app.route(path, mr)` EAGERLY installs routes into the stdlib HTTP
//!   registry under synthetic names (`__axum:<n>`): in the VM world the
//!   extern `serve_build_app`/`serve_listen` fns are no-ops, so the
//!   existing auto-serve check in execute_autovm (post-main) finds the
//!   routes and starts the server loop unchanged. `.merge` needs no VM
//!   semantics for the same reason.
//! - At dispatch time `handle_connection_async` detects the synthetic name
//!   and marshals request data per param shape instead of the positional
//!   `#[api]` convention, then calls the closure via `call_closure`:
//!   State → opaque AppState singleton, Json/Query → `json_to_vm_value`
//!   (GenericInstanceData, so `body.username` field access works), Path →
//!   path segment strings, HeaderMap → opaque placeholder.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::vm::engine::{AutoVM, VMError};
use crate::vm::ffi::rust_stdlib::RustStdlibObject;
use crate::vm::task::AutoTask;

/// One handler parameter's extractor shape, derived from the declared type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractorKind {
    State,
    Json,
    Query,
    Path,
    Headers,
    /// Non-extractor param (plain str/int) — bound to the next path segment.
    Plain,
}

impl ExtractorKind {
    fn from_type_name(ty: &str) -> Self {
        let mut base = ty.split('<').next().unwrap_or(ty).trim();
        // PLAN-044: 路径限定形态(axum::http::HeaderMap)取尾段——Type Display
        // 带完整路径,裸名匹配漏判成 Plain(auth_me 的 headers 参数即此坑)。
        if let Some(pos) = base.rfind("::") {
            base = &base[pos + 2..];
        }
        match base {
            "State" => ExtractorKind::State,
            "Json" => ExtractorKind::Json,
            "Query" => ExtractorKind::Query,
            "Path" => ExtractorKind::Path,
            "HeaderMap" => ExtractorKind::Headers,
            _ => ExtractorKind::Plain,
        }
    }
}

/// A route registered through the adapter. The handler is a Plan-383
/// fn-ref closure id; `params` drives request marshalling at dispatch time.
#[derive(Debug, Clone)]
pub struct AxumRoute {
    pub method: String,
    pub path: String,
    pub closure_id: u32,
    pub params: Vec<ExtractorKind>,
}

/// One (method, handler) slot — the payload of a `MethodRouter` heap object.
#[derive(Debug, Clone)]
pub struct MethodRouterEntry {
    pub method: String,
    pub closure_id: u32,
    pub params: Vec<ExtractorKind>,
}

/// Heap payload for `axum::MethodRouter`: `get(h).post(h2)` accumulates
/// multiple method slots on one object.
#[derive(Debug, Default, Clone)]
pub struct MethodRouter {
    pub handlers: Vec<MethodRouterEntry>,
}

/// Heap payload for `axum::Router`: path template → method router.
#[derive(Debug, Default, Clone)]
pub struct RouterBuilder {
    pub entries: Vec<(String, MethodRouter)>,
}

// Process-global registries (same lifetime model as stdlib's HTTP_ROUTES:
// one program per driver process; the run pipeline resets before compiling).
static AXUM_ROUTES: Mutex<Vec<AxumRoute>> = Mutex::new(Vec::new());
static PARAM_SIGS: std::sync::LazyLock<Mutex<HashMap<String, Vec<ExtractorKind>>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));
static APP_STATE: Mutex<Option<u64>> = Mutex::new(None);

/// Codegen publishes each fn's param shapes at fn-compile time (all modules
/// of the program share this table — dep-module Codegen instances publish
/// here too, so extractor shapes resolve for cross-module fn-ref closures).
pub fn record_param_sig(fn_name: &str, param_type_names: Vec<String>) {
    PARAM_SIGS
        .lock()
        .unwrap()
        .insert(fn_name.to_string(), sig_entry(&param_type_names));
}

/// Resolve a handler fn's param shapes by name.
fn param_shapes_for(fn_name: &str) -> Option<Vec<ExtractorKind>> {
    PARAM_SIGS.lock().unwrap().get(fn_name).cloned()
}

/// Map declared param type strings → extractor kinds (codegen side helper).
pub fn sig_entry(param_type_names: &[String]) -> Vec<ExtractorKind> {
    param_type_names
        .iter()
        .map(|t| {
            let k = ExtractorKind::from_type_name(t);
            k
        })
        .collect()
}

/// Resolve a fn-ref closure's param shapes: closure → func_addr → export
/// name (inverted exports table) → PARAM_SIGS. Falls back to the last
/// path-segment match for module-qualified export names.
fn resolve_params(vm: &AutoVM, closure_id: u32) -> Vec<ExtractorKind> {
    let func_addr = match vm.closures.get(&closure_id) {
        Some(g) => g.func_addr,
        None => return Vec::new(),
    };
    let export_name = vm
        .flash
        .exports_by_name
        .iter()
        .find(|(_, &addr)| addr == func_addr)
        .map(|(n, _)| n.clone());
    let name = match export_name {
        Some(n) => n,
        None => return Vec::new(),
    };
    param_shapes_for(&name)
        .or_else(|| {
            let short = name.rsplit('.').next().unwrap_or(&name).to_string();
            param_shapes_for(&short)
        })
        .unwrap_or_default()
}

/// axum path template → VM match_route syntax: `{x}` → `:x`, `{*x}` → `*x`.
fn convert_path(axum: &str) -> String {
    let mut out = String::with_capacity(axum.len());
    let mut chars = axum.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            if chars.peek() == Some(&'*') {
                chars.next();
                out.push('*');
            } else {
                out.push(':');
            }
            for nc in chars.by_ref() {
                if nc == '}' {
                    break;
                }
                out.push(nc);
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Append one path's handlers to AXUM_ROUTES and re-publish the merged
/// stdlib route table (register_http_routes overwrites, so non-axum routes
/// from #[api] registration are preserved alongside).
fn install_path(axum_path: &str, mr: &MethodRouter) -> usize {
    let vm_path = convert_path(axum_path);
    let mut added = 0usize;
    {
        let mut reg = AXUM_ROUTES.lock().unwrap();
        for h in &mr.handlers {
            reg.push(AxumRoute {
                method: h.method.clone(),
                path: vm_path.clone(),
                closure_id: h.closure_id,
                params: h.params.clone(),
            });
            added += 1;
        }
        // Re-publish: keep foreign (#[api]) entries, replace axum ones with
        // the current snapshot.
        let mut table: Vec<(String, String, String)> = crate::vm::ffi::stdlib::get_http_routes()
            .into_iter()
            .filter(|(_, name, _)| !is_axum_route(name))
            .collect();
        for (idx, r) in reg.iter().enumerate() {
            table.push((r.method.clone(), r.path.clone(), format!("__axum:{idx}")));
        }
        crate::vm::ffi::stdlib::register_http_routes(table);
    }
    added
}

/// Is this synthetic route name adapter-owned?
pub fn is_axum_route(fn_name: &str) -> bool {
    fn_name.starts_with("__axum:")
}

/// Look up an installed route by its synthetic name.
pub fn route_by_synthetic_name(fn_name: &str) -> Option<AxumRoute> {
    let idx: usize = fn_name.strip_prefix("__axum:")?.parse().ok()?;
    AXUM_ROUTES.lock().unwrap().get(idx).cloned()
}

/// Test/diagnostic introspection: snapshot of the installed route table.
pub fn installed_routes() -> Vec<AxumRoute> {
    AXUM_ROUTES.lock().unwrap().clone()
}

/// Reset (the run pipeline calls this before compiling a program; the
/// compile-side record_param_sig publishers rebuild the table from scratch).
pub fn reset() {
    AXUM_ROUTES.lock().unwrap().clear();
    PARAM_SIGS.lock().unwrap().clear();
    *APP_STATE.lock().unwrap() = None;
}

/// The AppState singleton's heap handle (`s State<AppState>` binds to this;
/// handlers reach extern state through its `.view`-style opaque accessors).
pub fn app_state_handle(vm: &AutoVM) -> u64 {
    let mut guard = APP_STATE.lock().unwrap();
    if let Some(h) = *guard {
        return h;
    }
    let obj = RustStdlibObject::new("crate::server::AppState", ());
    let handle = vm.insert_heap_object(obj);
    *guard = Some(handle);
    handle
}

/// Marshall request data onto `task`'s stack per the handler's param shapes
/// (declaration order). Mirrors `build_handler_args`' pushing conventions:
/// strings via the pool with rc bookkeeping, compound JSON via
/// `json_to_vm_value`. Returns the arg count pushed.
pub fn push_extractor_args(
    vm: &AutoVM,
    task: &mut AutoTask,
    route: &AxumRoute,
    path_values: &[(String, String)],
    query_json: &str,
    body_json: &str,
    headers_json: &str,
) -> usize {
    let mut n = 0usize;
    let mut path_iter = path_values.iter().map(|(_, v)| v.as_str());
    for kind in &route.params {
        match kind {
            ExtractorKind::State => {
                let handle = app_state_handle(vm);
                vm.rc_push_id(task, handle);
                n += 1;
            }
            ExtractorKind::Json | ExtractorKind::Query => {
                let raw = if *kind == ExtractorKind::Json { body_json } else { query_json };
                let parsed: serde_json::Value =
                    serde_json::from_str(raw).unwrap_or(serde_json::Value::Null);
                // Marshal failure degrades to null — the handler's field
                // reads yield nulls, but the connection stays alive.
                if let Err(e) = crate::vm::ffi::stdlib::json_to_vm_value(task, vm, &parsed, 0) {
                    eprintln!("[AXUM] extractor json marshal failed: {e:?}");
                    task.ram.push_nv(auto_val::encode_null());
                }
                n += 1;
            }
            ExtractorKind::Path | ExtractorKind::Plain => {
                let val = path_iter.next().unwrap_or("");
                let idx = vm.add_string(val.as_bytes().to_vec());
                vm.rc_push_str_idx(task, idx);
                n += 1;
            }
            ExtractorKind::Headers => {
                // PLAN-044: real-header marshalling — musk auth handlers
                // forward the HeaderMap to externs (auth_token_from_headers
                // etc.); the opaque placeholder carried no data, so bearer
                // extraction was impossible. Marshal the connection-level
                // header JSON (authorization/cookie) as the arg instead.
                let parsed: serde_json::Value =
                    serde_json::from_str(headers_json).unwrap_or(serde_json::Value::Null);
                if let Err(e) = crate::vm::ffi::stdlib::json_to_vm_value(task, vm, &parsed, 0) {
                    eprintln!("[AXUM] extractor headers marshal failed: {e:?}");
                    task.ram.push_nv(auto_val::encode_null());
                }
                n += 1;
            }
        }
    }
    n
}

// ── Dispatch-3000 shims (invoked from shim_rust_stdlib_dispatch arms) ──

/// `Router.new()` — static call, no stack args after type/method pop.
pub(crate) fn shim_router_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let obj = RustStdlibObject::new("axum::Router", RouterBuilder::default());
    let handle = vm.insert_heap_object(obj);
    vm.rc_push(task, auto_val::encode_object(handle as u32));
    Ok(())
}

/// Bare `get(h)` / `post(h)` / ... — stack after type/method pop:
/// `[closure_id]`. Creates a one-slot MethodRouter.
pub(crate) fn shim_method_bare(
    task: &mut AutoTask,
    vm: &AutoVM,
    method: &str,
) -> Result<(), VMError> {
    let closure_id = pop_closure_id(task)?;
    let params = resolve_params(vm, closure_id);
    let mr = MethodRouter {
        handlers: vec![MethodRouterEntry {
            // HTTP requests arrive uppercased ("GET") — store canonical form.
            method: method.to_uppercase(),
            closure_id,
            params,
        }],
    };
    let obj = RustStdlibObject::new("axum::MethodRouter", mr);
    let handle = vm.insert_heap_object(obj);
    vm.rc_push(task, auto_val::encode_object(handle as u32));
    Ok(())
}

/// Chained `mr.post(h)` — stack: `[mr_handle, closure_id]`. Appends a slot.
pub(crate) fn shim_method_chain(
    task: &mut AutoTask,
    vm: &AutoVM,
    method: &str,
) -> Result<(), VMError> {
    let closure_id = pop_closure_id(task)?;
    let handle = task.ram.pop_i32() as u64;
    let entry = MethodRouterEntry {
        // HTTP requests arrive uppercased ("POST") — store canonical form.
        method: method.to_uppercase(),
        closure_id,
        params: resolve_params(vm, closure_id),
    };
    let obj = vm
        .get_heap_object(handle)
        .ok_or_else(|| VMError::RuntimeError(format!("axum {method}: bad MethodRouter handle {handle}")))?;
    let mut guard = obj.write().unwrap();
    let rust_obj = guard
        .as_any_mut()
        .downcast_mut::<RustStdlibObject>()
        .ok_or_else(|| VMError::RuntimeError(format!("axum {method}: not a RustStdlibObject")))?;
    let mr = rust_obj
        .downcast_mut::<MethodRouter>()
        .ok_or_else(|| VMError::RuntimeError(format!("axum {method}: not a MethodRouter")))?;
    mr.handlers.push(entry);
    drop(guard);
    vm.rc_push(task, auto_val::encode_object(handle as u32));
    Ok(())
}

/// `app.route(path, mr)` — stack: `[router_handle, path_str, mr_handle]`.
/// Mutates the builder AND eagerly installs the routes (the VM world's
/// serve_listen/serve_build_app are extern no-ops; the auto-serve check
/// picks the registry up after main returns). Returns the router handle
/// for chaining.
pub(crate) fn shim_router_route(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let mr_handle = task.ram.pop_i32() as u64;
    let path = pop_string_arg(task, vm, "Router.route")?;
    let router_handle = task.ram.pop_i32() as u64;

    // Move the method router's handler list out of the heap object.
    let handlers: Vec<MethodRouterEntry> = {
        let obj = vm
            .get_heap_object(mr_handle)
            .ok_or_else(|| VMError::RuntimeError("Router.route: bad MethodRouter handle".into()))?;
        let guard = obj.read().unwrap();
        let rust_obj = guard
            .as_any()
            .downcast_ref::<RustStdlibObject>()
            .ok_or_else(|| VMError::RuntimeError("Router.route: not a RustStdlibObject".into()))?;
        rust_obj
            .downcast_ref::<MethodRouter>()
            .ok_or_else(|| VMError::RuntimeError("Router.route: not a MethodRouter".into()))?
            .handlers
            .clone()
    };

    let router_obj = vm
        .get_heap_object(router_handle)
        .ok_or_else(|| VMError::RuntimeError("Router.route: bad Router handle".into()))?;
    let mut guard = router_obj.write().unwrap();
    let rust_obj = guard
        .as_any_mut()
        .downcast_mut::<RustStdlibObject>()
        .ok_or_else(|| VMError::RuntimeError("Router.route: router not RustStdlibObject".into()))?;
    let builder = rust_obj
        .downcast_mut::<RouterBuilder>()
        .ok_or_else(|| VMError::RuntimeError("Router.route: not a RouterBuilder".into()))?;
    match builder.entries.iter_mut().find(|(p, _)| p == &path) {
        Some(bucket) => bucket.1.handlers.extend(handlers.clone()),
        None => builder.entries.push((
            path.clone(),
            MethodRouter { handlers: handlers.clone() },
        )),
    }
    drop(guard);

    let installed = install_path(&path, &MethodRouter { handlers });
    eprintln!("[AXUM] route {} → {} handler(s) installed", path, installed);

    vm.rc_push(task, auto_val::encode_object(router_handle as u32));
    Ok(())
}

/// `app.serve(addr)` (rare direct form) — install everything, return unit.
pub(crate) fn shim_router_serve(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let _addr = pop_string_arg(task, vm, "Router.serve")?;
    let handle = task.ram.pop_i32() as u64;
    let builder = {
        let obj = vm
            .get_heap_object(handle)
            .ok_or_else(|| VMError::RuntimeError("Router.serve: bad Router handle".into()))?;
        let guard = obj.read().unwrap();
        let rust_obj = guard
            .as_any()
            .downcast_ref::<RustStdlibObject>()
            .ok_or_else(|| VMError::RuntimeError("Router.serve: not RustStdlibObject".into()))?;
        rust_obj
            .downcast_ref::<RouterBuilder>()
            .ok_or_else(|| VMError::RuntimeError("Router.serve: not a RouterBuilder".into()))?
            .clone()
    };
    let mut total = 0usize;
    for (path, mr) in &builder.entries {
        total += install_path(path, mr);
    }
    eprintln!("[AXUM] serve: {} route(s) installed", total);
    task.ram.push_nv(auto_val::encode_i32(0));
    Ok(())
}

/// Pop a Plan-383 fn-ref closure id (Int on the stack).
fn pop_closure_id(task: &mut AutoTask) -> Result<u32, VMError> {
    let nv = task.ram.pop_nv();
    if auto_val::is_i32(nv) {
        Ok(auto_val::decode_i32(nv) as u32)
    } else {
        Err(VMError::RuntimeError(format!(
            "axum handler: expected closure id, got non-i32 ({:016x})",
            nv
        )))
    }
}

/// Pop a string argument (2-slot nanbox string via the pool).
fn pop_string_arg(task: &mut AutoTask, vm: &AutoVM, ctx: &str) -> Result<String, VMError> {
    <String as crate::vm::ffi::convert::VMConvertible>::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(format!("{}: {:?}", ctx, e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_conversion() {
        assert_eq!(convert_path("/api/notes/:id"), "/api/notes/:id");
        assert_eq!(convert_path("/api/roles/{name}"), "/api/roles/:name");
        assert_eq!(
            convert_path("/api/files/{workspace_id}/{*path}"),
            "/api/files/:workspace_id/*path"
        );
    }

    #[test]
    fn extractor_kinds() {
        assert_eq!(ExtractorKind::from_type_name("State<AppState>"), ExtractorKind::State);
        assert_eq!(ExtractorKind::from_type_name("Json<LoginRequest>"), ExtractorKind::Json);
        assert_eq!(ExtractorKind::from_type_name("Query<WorkspaceQuery>"), ExtractorKind::Query);
        assert_eq!(ExtractorKind::from_type_name("Path<(str, str)>"), ExtractorKind::Path);
        assert_eq!(ExtractorKind::from_type_name("HeaderMap"), ExtractorKind::Headers);
        assert_eq!(ExtractorKind::from_type_name("str"), ExtractorKind::Plain);
    }
}
