//! Compatibility dispatcher: IPC stays here, HTTP delegates to its Web owner.
pub use auto_generation_core::shared_generation::*;
pub use auto_generation_core::source_owner::fnv1a;
use auto_val::AutoResult;
pub use auto_web::api_gen::{
    backend_generation_is_fresh, current_generated_api_assembly, generate_initial_data_pub,
    install_project_api_glue,
};
use std::path::Path;
pub fn generate_api(root_dir: &Path, backend: &str) -> AutoResult<()> {
    if backend != "tauri" {
        return auto_web::generate_api(root_dir, backend);
    }

    // Plan 061:统一契约定位 —— 本地 src/back/ | back/,或 pac.at
    // `back: { project }` 指向的外部后端项目(前端可无本地 back/)。
    let api_file = match auto_lang_core::config::resolve_back_api(root_dir) {
        Some(f) => f,
        // No backend contract found anywhere, skip generation
        None => return Ok(()),
    };
    let back_dir = api_file
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| root_dir.to_path_buf());

    // Read API file
    let api_content = std::fs::read_to_string(&api_file)
        .map_err(|e| format!("Failed to read {}: {}", api_file.display(), e))?;

    // PLAN-734 T-04（D4）：strict 解析门——full-parse 失败 = Err（解析错误
    // 不再静默落 lenient/空提取；AC-04）。lenient 退役为显式
    // `AUTO_API_LENIENT=1`（产物按 scaffold 语义——不含业务体）。
    let strict_parse = std::env::var("AUTO_API_LENIENT")
        .map(|v| v != "1")
        .unwrap_or(true);
    let api_module = match try_full_parse(&api_content) {
        Some(module) => module,
        None if strict_parse => {
            return Err(format!(
                "api.at parse failed ({}); fix the parse error or set AUTO_API_LENIENT=1 for scaffold-only extraction",
                api_file.display()
            )
            .into());
        }
        None => {
            // Lenient extraction for files with module references like `use db`
            match extract_api_lenient(&api_content) {
                Some(m) => {
                    println!("  ℹ Using lenient API extraction (scaffold: no bodies)");
                    m
                }
                None => {
                    println!("  ⚠ Could not extract API definitions");
                    if backend == "vue" {
                        install_project_api_glue(root_dir);
                    }
                    return Ok(());
                }
            }
        }
    };

    // Check if any endpoints or types were extracted
    if api_module.endpoints.is_empty() && api_module.types.is_empty() {
        println!("  ⚠ No API endpoints or types found");
        // Plan 559 W2: implementation-style back/api.at (VM-dialect bodies —
        // os-config's 80 fn http/json surface) yields no contract endpoints.
        // If the project ships the web implementation as a hand-written TS
        // twin at src/back/api.ts (the "vue codegen → @/lib/api" arm of the
        // same `use back.api` import line), install it as the gen tree's
        // lib/api.ts so the vue track type-checks. Without a twin, the gen
        // tree gets no glue (desktop-host v1 keeps skipping such apps).
        if backend == "vue" {
            install_project_api_glue(root_dir);
        }
        return Ok(());
    }

    // PLAN-734 T-04：契约预检——HTTP 专属种类（File/Upload）在非 HTTP 消费
    // 面（tauri 命令生成）按矩阵拒绝；显式 Response/Stream 保持各自形态。
    // （rust/vue 后端经 HTTP 承载，无此限制。）
    if backend == "tauri" {
        for ep in &api_module.endpoints {
            // PLAN-776 T-06：读提取期冻结的来源身份分类。
            let kind = ep.response;
            if kind.is_http_exclusive() {
                return Err(format!(
                    "api endpoint `{}.{}` returns an HTTP-exclusive kind ({:?}); Tauri IPC cannot carry it — call the HTTP URL instead",
                    api_file.display(),
                    ep.fn_name,
                    kind
                )
                .into());
            }
        }
    }

    // Generate code based on backend
    match backend {
        "tauri" => {
            generate_tauri_api(&api_module, root_dir)?;
        }
        _ => {
            // No API generation for other backends
        }
    }

    Ok(())
}

/// Generate Tauri API code
fn generate_tauri_api(
    api_module: &auto_lang_core::api::ApiModule,
    root_dir: &Path,
) -> AutoResult<()> {
    use auto_lang_core::api::Target;

    let vue_dir = root_dir.join("gen").join("front").join("vue");
    let tauri_src_dir = vue_dir.join("src-tauri").join("src");

    // Ensure directories exist
    std::fs::create_dir_all(&tauri_src_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    // Generate Tauri commands
    let tauri_gen = Target::Tauri.generator();
    let tauri_code = tauri_gen.generate(api_module);
    std::fs::write(tauri_src_dir.join("commands.rs"), &tauri_code)
        .map_err(|e| format!("Failed to write commands.rs: {}", e))?;

    // Generate TypeScript IPC client for Tauri (uses invoke instead of fetch)
    let ts_ipc_code = generate_tauri_ts_client(api_module);

    // Write to src/lib/api.ts so Vue imports resolve correctly
    let lib_dir = vue_dir.join("src").join("lib");
    std::fs::create_dir_all(&lib_dir)
        .map_err(|e| format!("Failed to create lib directory: {}", e))?;
    std::fs::write(lib_dir.join("api.ts"), &ts_ipc_code)
        .map_err(|e| format!("Failed to write src/lib/api.ts: {}", e))?;

    // Also write to src/api/client.ts for backward compatibility
    let api_dir = vue_dir.join("src").join("api");
    std::fs::create_dir_all(&api_dir)
        .map_err(|e| format!("Failed to create api directory: {}", e))?;
    std::fs::write(api_dir.join("client.ts"), &ts_ipc_code)
        .map_err(|e| format!("Failed to write client.ts: {}", e))?;

    println!("  ✓ Generated Tauri commands: src-tauri/src/commands.rs");
    println!("  ✓ Generated TypeScript IPC client: src/lib/api.ts");

    Ok(())
}

/// Generate a Tauri IPC TypeScript client using `invoke`
fn generate_tauri_ts_client(api_module: &auto_lang_core::api::ApiModule) -> String {
    let mut lines = vec![
        "import { invoke } from '@tauri-apps/api/core';".to_string(),
        "".to_string(),
    ];

    // Type definitions
    for api_type in &api_module.types {
        lines.push(format!("export interface {} {{", api_type.name));
        for field in &api_type.fields {
            let ts_type = auto_type_to_ts(&field.ty);
            let optional = if field.optional { "?" } else { "" };
            lines.push(format!("    {}{}: {};", field.name, optional, ts_type));
        }
        lines.push("}".to_string());
        lines.push("".to_string());
    }

    // IPC functions
    for endpoint in &api_module.endpoints {
        let params_ts: Vec<String> = endpoint
            .params
            .iter()
            .map(|p| {
                let ts_type = auto_type_to_ts(&p.ty);
                format!("{}: {}", p.name, ts_type)
            })
            .collect();

        let return_ts = auto_type_to_ts(&endpoint.return_type);
        let args_str = if params_ts.is_empty() {
            "".to_string()
        } else {
            params_ts.join(", ")
        };

        if params_ts.is_empty() {
            lines.push(format!(
                "export async function {}(): Promise<{}> {{",
                endpoint.fn_name, return_ts
            ));
            lines.push(format!("    return invoke('{}');", endpoint.fn_name));
        } else {
            lines.push(format!(
                "export async function {}({}): Promise<{}> {{",
                endpoint.fn_name, args_str, return_ts
            ));
            lines.push(format!(
                "    return invoke('{}', {{ {} }});",
                endpoint.fn_name,
                endpoint
                    .params
                    .iter()
                    .map(|p| format!("{}", p.name))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        lines.push("}".to_string());
        lines.push("".to_string());
    }

    lines.join("\n")
}

/// Convert Auto type to TypeScript type
fn auto_type_to_ts(auto_type: &str) -> String {
    let auto_type = auto_type.trim();
    // Handle prefix ?T (Auto Option syntax: ?Note, ?int)
    if let Some(inner) = auto_type.strip_prefix('?') {
        return format!("{} | null", auto_type_to_ts(inner));
    }
    // Handle suffix T? (alternative Option syntax)
    if auto_type.ends_with('?') {
        let inner = &auto_type[..auto_type.len() - 1];
        return format!("{} | null", auto_type_to_ts(inner));
    }
    if auto_type.starts_with("[]") || auto_type.starts_with("List<") {
        let inner = if auto_type.starts_with("[]") {
            &auto_type[2..]
        } else if let Some(close) = auto_type.find('>') {
            &auto_type[5..close]
        } else {
            auto_type
        };
        return format!("{}[]", auto_type_to_ts(inner));
    }
    match auto_type {
        "int" | "i32" | "i64" | "long" | "uint" | "u32" | "u64" | "ulong" => "number".to_string(),
        "float" | "f32" | "double" | "f64" => "number".to_string(),
        "bool" | "boolean" => "boolean".to_string(),
        "str" | "string" | "String" => "string".to_string(),
        "void" | "()" => "void".to_string(),
        _ => auto_type.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn plan734_tauri_mock_dispatcher_invokes_generated_command() {
        // 最小 api 实现（生成命令引用 crate::api::echo——本测试以局部模块
        // 提供同形实现，command 体本身来自生成器 fixture）。
        mod api {
            use serde::{Deserialize, Serialize};

            #[derive(Debug, Clone, Serialize, serde::Deserialize)]
            pub struct Echo {
                pub ok: bool,
                pub n: i32,
                pub tag: String,
            }

            pub fn echo(n: i32, tag: String) -> Echo {
                Echo { ok: true, n, tag }
            }
        }
        // 生成的命令体（与 fixture 逐字一致由上面的锁测试保证——此处仅
        // 保留 dispatcher 所需的 command 定义）。
        #[tauri::command]
        fn echo(n: i32, tag: String) -> api::Echo {
            api::echo(n, tag)
        }

        let app = tauri::test::mock_builder()
            .invoke_handler(tauri::generate_handler![echo])
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .expect("mock app builds");
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("mock webview builds");

        let body = serde_json::json!({ "n": 42, "tag": "hi" });
        let resp = tauri::test::get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "echo".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: tauri::ipc::InvokeBody::Json(body),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_string(),
            },
        )
        .expect("dispatcher resolves");

        let echo: api::Echo = resp.deserialize().expect("body deserializes to Echo");
        assert!(echo.ok);
        assert_eq!(echo.n, 42);
        assert_eq!(echo.tag, "hi");
    }
    #[test]
    fn test_plan734_tauri_fixture_matches_generator() {
        let api = r#"
pub type Echo = { ok: bool, n: int, tag: str }

#[api(method = "POST", path = "/api/echo")]
pub fn echo(n int, tag str) Echo {
    return Echo { ok: true, n: n, tag: tag }
}
"#;
        let module = try_full_parse(api).expect("parse");
        use auto_lang_core::api::TargetGenerator as _;
        let generated = auto_lang_core::api::TauriGenerator::new().generate(&module);
        let fixture = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../auto-man/crates/auto-man/tests/fixtures/plan734_commands_fixture.rs.txt"
        ))
        .expect("fixture readable");
        assert_eq!(
            generated.trim(),
            fixture.trim(),
            "generator drift vs fixture"
        );
    }
    #[test]
    fn test_plan730_tauri_upload_unsupported() {
        use auto_lang_core::api::TauriGenerator;
        let api = r#"
pub type UploadMeta = { name: str }

#[api(method = "POST", path = "/api/uploads")]
pub fn upload(req UploadRequest) ~UploadReceipt {
    return http.upload_error(201, "x")
}
"#;
        let module = try_full_parse(api).expect("full parse");
        let gen = TauriGenerator::new();
        use auto_lang_core::api::TargetGenerator as _;
        let cmds = gen.generate(&module);
        assert!(
            cmds.contains("PLAN-730: upload endpoint `upload` requires HTTP transport"),
            "unsupported diagnostic: {cmds}"
        );
        assert!(!cmds.contains("invoke_upload"), "{cmds}");
    }
}
