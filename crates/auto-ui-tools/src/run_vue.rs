//! PLAN-815 T-05: Vue dev 运行面自 auto-man `vue_run_compat` 迁入（M814-B
//! 门解除后的最后一批 T-04 残留）。生成/构建主体在 T-04 已迁本 crate
//! （`crate::vue`）；本模块补齐 run 主链与画廊 back-proxy，全部经新公开
//! 包消费（`auto_ui_runtime::back_proxy` / `ui::back_provision`），零
//! legacy `auto_lang` 依赖。
//!
//! 托管模式（工具宿主安装 [`crate::proc_host::RunIo`] 时）：六步边界发
//! Progress 事件、取消检查点收口（`FlowEnd::Cancelled`）；`npm install`/
//! `npm run dev` 子进程经 `crate::pkg` 托管分支（stdout 转发日志流、
//! stdin 接 `Input` 通道、取消即杀）。未安装时行为与迁移前逐字一致
//! （继承 stdio、println 步骤横幅）——Man 门面转发即旧行为。

use std::fs;
use std::path::Path;

use colored::Colorize;

use auto_val::{AutoError, AutoResult};

use crate::proc_host::{self, FlowEnd};
// 生成主体（T-04 已迁）。
use crate::vue::{
    desktop_mode, gallery_apps_dir, gallery_mode, gallery_rows_via_cache, generate_index_css,
    generate_index_html, generate_package_json, generate_select_overlay_ts,
    generate_tailwind_config, generate_vite_config, incremental_compile_changed, pac_media_root,
    package_json_deps_drifted, resolve_front_dir, set_gallery_proxy_root, write_auto_sources_ts,
    VueProject,
};

/// Run Vue dev server (auto run command) — legacy signature (behavior
/// frozen; hosted hooks engage only when the tool host installed a RunIo).
pub fn run_vue_project(root_dir: &Path, args: Vec<String>) -> AutoResult<()> {
    run_vue_flow(root_dir, args, None).map(|_| ())
}

/// 请求路径（PLAN-809 R1-F02：run 的项目读取经 I-PROJECT）。
pub fn run_vue_project_with_request(
    root_dir: &Path,
    args: Vec<String>,
    req: &auto_project_model::generation_request::GenerationRequest,
) -> AutoResult<()> {
    run_vue_flow(root_dir, args, Some(req)).map(|_| ())
}

/// Map a step result onto the flow outcome, recognizing cancellation that
/// surfaced as a child-process kill.
fn flow_step(result: AutoResult<()>) -> Result<FlowEnd, AutoError> {
    if proc_host::cancelled() {
        return Ok(FlowEnd::Cancelled);
    }
    result.map(|_| FlowEnd::Completed)
}

/// Steps (progress oracle):
/// 1. Incrementally compile changed .at files (see `incremental_compile_changed`)
/// 2. Generate project structure if not exists
/// 3. Generate API client code (if api.at exists)
/// 4. Materialize bundled shadcn-vue components (PLAN-457, before install)
/// 5. npm install (+ registry fallback for non-bundled components)
/// 6. Copy public assets, start backend, start dev server
pub fn run_vue_flow(
    root_dir: &Path,
    args: Vec<String>,
    req: Option<&auto_project_model::generation_request::GenerationRequest>,
) -> Result<FlowEnd, AutoError> {
    let total_steps = 6u64;
    let step = |n: u64, what: &str| proc_host::progress(n, Some(total_steps), what);
    let finish_cancelled = || {
        proc_host::progress(total_steps, Some(total_steps), "cancelled");
        Ok(FlowEnd::Cancelled)
    };

    // PLAN-646: dev 运行面开启 Select Anything DOM 标记（data-auto-*）——
    // SFC 注入 data-auto-{tag,id,span} 供 overlay 框选采集；产物构建
    // （build_vue_project）不设，保持输出逐字节不变。VueGenerator::new()
    // 构造期读取（AUTO_API_FUNCTIONS 同款进程级 env 通道）。
    // SAFETY: edition 2021——set_var 安全；在任一 VueGenerator::new() 之前设置。
    std::env::set_var("AUTOUI_SELECT_MARKERS", "1");
    println!("{}", "Running Vue dev server (backend: vue)".bright_cyan());

    // PLAN-646: 源码映射随每次运行刷新（内容 hash 防抖，覆盖首启/全量生成路径
    // ——incremental_compile_changed 内的同步点只在有增量时触达）。
    let p646_vue_root = root_dir.join("gen").join("front").join("vue");
    write_auto_sources_ts(&resolve_front_dir(root_dir), &p646_vue_root);
    // PLAN-646: overlay 资产自愈刷新（旧工程 scaffold 停在旧版 overlay）。
    {
        let overlay_path = p646_vue_root
            .join("src")
            .join("auto-select")
            .join("overlay.ts");
        let overlay_new = generate_select_overlay_ts();
        let stale = match std::fs::read_to_string(&overlay_path) {
            Ok(existing) => existing != overlay_new,
            Err(_) => true,
        };
        if stale {
            std::fs::create_dir_all(overlay_path.parent().unwrap()).ok();
            std::fs::write(&overlay_path, overlay_new).ok();
        }
    }
    // P660-D1 自愈：旧工程 scaffold 无 vite-env.d.ts（增量路径不重写 scaffold
    // 文件）——缺即补，vue-tsc TS2339 防线。
    {
        let vite_env = p646_vue_root.join("src").join("vite-env.d.ts");
        if !vite_env.exists() {
            std::fs::write(&vite_env, "/// <reference types=\"vite/client\" />\n").ok();
        }
    }

    step(1, "incremental compile");
    let changed_count = incremental_compile_changed(root_dir, req)?;
    if proc_host::cancelled() {
        return finish_cancelled();
    }
    step(1, "project context");

    // Load project context（R1-F02：请求在场则经合同读取，不再回读 pac.at）
    let project = match req {
        Some(r) => VueProject::from_workspace_with_request(root_dir, r)?,
        None => VueProject::from_workspace(root_dir)?,
    };

    let mut current_step = 0u64;

    // Step 1: Generate project structure if not exists, or regenerate source files
    current_step += 1;
    step(current_step, "generate project structure");
    println!();
    if !project.exists() && changed_count == 0 {
        // Fresh project: no incremental changes were written, so generate everything
        println!(
            "▶ Step {}/{}: Generating Vue project...",
            current_step, total_steps
        );
        flow_step(project.generate())?;
    } else if !project.exists() && changed_count > 0 {
        // Project dir was removed but we already wrote files incrementally.
        // Only generate scaffolding (package.json, vite.config, etc), don't
        // overwrite the incrementally-written component .vue files.
        println!(
            "▶ Step {}/{}: Generating project scaffolding...",
            current_step, total_steps
        );
        flow_step(project.generate_scaffolding_only())?;
    } else if changed_count == 0 {
        println!(
            "▶ Step {}/{}: Checking source files...",
            current_step, total_steps
        );
        // Self-heal router/index.ts — older scaffolding-only runs left it
        // missing, which breaks `import router from './router'` in main.ts.
        flow_step(project.ensure_router_file())?;
        // Plan 413: self-heal the CodeEditor shell the same way.
        flow_step(project.ensure_code_editor_component())?;
    }
    if proc_host::cancelled() {
        return finish_cancelled();
    }

    // Plan 465: the desktop host shell + app registry must refresh on EVERY
    // run (apps may change) — same reasoning as the index.html rewrite below.
    if desktop_mode() {
        flow_step(project.generate_desktop_host())?;
    }

    // PLAN-528 W6: pac.at npm_deps（如 @autodown/* link 依赖）必须每次 run
    // 自愈进 package.json——增量路径不走 regenerate_source_files 的漂移重写，
    // 只在 pac.at 新增的依赖永远不会被安装。自愈理由同上方 index.html。
    //
    // Plan 672: 此块必须在 generate_gallery_host **之前**执行——W6 以宿主单项目
    // usage 整体重写 package.json，会把画廊刚合并的跨 demo 依赖当作残留清除
    // （实测 vue-sonner 被冲掉 → vite 解析失败整站崩溃）。gallery 场景让
    // merge_host_npm_deps 做最后写入者；非 gallery 项目本块与下方 no-op 无序依赖。
    {
        let pkg_path = project.output_dir.join("package.json");
        if pkg_path.exists() {
            let existing = fs::read_to_string(&pkg_path).unwrap_or_default();
            let usage = project.dependency_usage();
            if package_json_deps_drifted(&existing, &usage, &project.npm_deps) {
                let new_pkg = generate_package_json(
                    &project.name,
                    project.has_routes,
                    project.i18n.enabled,
                    &project.npm_deps,
                    &usage,
                );
                match fs::write(&pkg_path, new_pkg) {
                    Ok(_) => println!(
                        "{}",
                        "  ✓ Updated package.json (npm_deps sync)".bright_green()
                    ),
                    Err(e) => println!("  ⚠ package.json refresh skipped: {}", e),
                }
            }
        }
    }

    // Plan 549: UI gallery host + demos registry refresh on EVERY run
    if project.is_ui_gallery() || gallery_mode() {
        flow_step(project.generate_gallery_host())?;

        // Plan 672 条目 4: Vue 臂 back-proxy 接线——PLAN-658 多后端 proxy
        // 此前仅 rust_ui（VM 臂宿主）启动，fullstack 档（013/015 等
        // back.api CRUD demo）在 Vue 臂只能显示「独立运行」提示。此处
        // 补启动（generate_gallery_host 之后 = demo 行缓存已热），端口经
        // env 传给 vite 子进程（generate_vite_config 的 `/apps` 条目消费，
        // npm_run_dev → run_script 子进程继承进程 env）。启动失败照 658
        // 降级语义：env 不注入，fullstack demo 运行期 fetch 失败走错误
        // 横幅，不阻断画廊。
        match start_gallery_back_proxy(root_dir) {
            Some(port) => std::env::set_var(
                "AUTO_GALLERY_BACK_PROXY",
                format!("http://127.0.0.1:{port}"),
            ),
            None => {
                std::env::remove_var("AUTO_GALLERY_BACK_PROXY");
            }
        }
    }

    // Plan 458: index.html carries the theme default (`class="dark"`) and the
    // accent bootstrap (`--primary`). It is tiny, so rewrite it on EVERY run —
    // otherwise a stale index.html (e.g., a pre-Plan-043-M5 template without
    // `class="dark"`, or a theme flag flip) survives the "project exists,
    // nothing changed" fast path forever.
    {
        let index_html_path = project.output_dir.join("index.html");
        if index_html_path
            .parent()
            .map(|p| p.exists())
            .unwrap_or(false)
        {
            let index_html = generate_index_html(
                &project.name,
                project.index_title.as_deref(),
                project.theme.as_ref(),
            );
            if let Err(e) = fs::write(&index_html_path, index_html) {
                println!("  ⚠ index.html refresh skipped: {}", e);
            }
        }
    }

    // Plan 548: tailwind.config.cjs / index.css 同理自愈——它们是确定性模板，
    // 增量路径不重写，生成器侧新增的主题色（如 sidebar 色阶）与 CSS 变量
    // 在老项目里会永久缺失。两个文件都不含用户手改入口（样式走 pac.at
    // `styles:` 注入），每次 run 重写零风险。
    {
        let tw_path = project.output_dir.join("tailwind.config.cjs");
        if tw_path.exists() {
            if let Err(e) = fs::write(&tw_path, generate_tailwind_config()) {
                println!("  ⚠ tailwind.config.cjs refresh skipped: {}", e);
            }
        }
        let index_css_path = project.output_dir.join("src/assets/index.css");
        if index_css_path.exists() {
            if let Err(e) = fs::write(&index_css_path, generate_index_css(project.theme.as_ref())) {
                println!("  ⚠ index.css refresh skipped: {}", e);
            }
        }
        // Plan 672 条目 4: vite.config.ts 同律自愈——`/apps` back-proxy 条目
        // （env 运行时读取）只在生成器模板里，增量 run 不重写会让老项目
        // 永久缺失该路由（实测：proxy 起了但 vite 无 /apps 转发 → fullstack
        // demo 404）。模板确定性，重写零风险。
        let vite_cfg_path = project.output_dir.join("vite.config.ts");
        if vite_cfg_path.exists() {
            if let Err(e) = fs::write(&vite_cfg_path, generate_vite_config()) {
                println!("  ⚠ vite.config.ts refresh skipped: {}", e);
            }
        }
    }

    // Copy handmade theme assets if available
    let handmade_css = root_dir
        .join("vue")
        .join("src")
        .join("assets")
        .join("index.css");
    let gen_css = root_dir
        .join("gen")
        .join("front")
        .join("vue")
        .join("src")
        .join("assets")
        .join("index.css");
    if handmade_css.exists() && gen_css.exists() {
        if let Ok(content) = fs::read_to_string(&handmade_css) {
            fs::write(&gen_css, content)
                .map_err(|e| format!("Failed to copy handmade index.css: {}", e))?;
            println!("{}", "  ✓ Copied handmade theme CSS".bright_green());
        }
    }
    let handmade_theme_toggle = root_dir
        .join("vue")
        .join("src")
        .join("components")
        .join("ThemeToggle.vue");
    let gen_components_dir = root_dir
        .join("gen")
        .join("front")
        .join("vue")
        .join("src")
        .join("components");
    if handmade_theme_toggle.exists() {
        let gen_theme_toggle = gen_components_dir.join("ThemeToggle.vue");
        if let Ok(content) = fs::read_to_string(&handmade_theme_toggle) {
            fs::write(&gen_theme_toggle, content)
                .map_err(|e| format!("Failed to copy ThemeToggle.vue: {}", e))?;
        }
    }

    // Step 2: Generate API client code (if api.at exists)
    current_step += 1;
    step(current_step, "generate API client");
    println!();
    println!(
        "▶ Step {}/{}: Generating API client...",
        current_step, total_steps
    );
    // PLAN-734 T-06：Err 传播（错误不能只 warn 后继续启动旧产物）。
    auto_api_gen::api_gen::generate_api(root_dir, "vue")?;
    if proc_host::cancelled() {
        return finish_cancelled();
    }

    // Load API function names for Vue generator (dynamic detection)
    let api_fns_path = root_dir.join("dist").join(".api_functions");
    if api_fns_path.exists() {
        if let Ok(content) = fs::read_to_string(&api_fns_path) {
            let fns: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
            if !fns.is_empty() {
                // SAFETY: Setting a process-wide env var before spawning Vue generation
                unsafe {
                    std::env::set_var("AUTO_API_FUNCTIONS", fns.join(","));
                }
            }
        }
    }

    // Step 3: Materialize bundled shadcn-vue ui components (PLAN-457).
    // Offline copies land BEFORE the dependency install so package.json is
    // complete when pnpm resolves — no post-install dependency surgery.
    current_step += 1;
    step(current_step, "materialize UI components");
    println!();
    println!(
        "▶ Step {}/{}: Materializing UI components...",
        current_step, total_steps
    );
    flow_step(project.materialize_ui_components())?;

    // Step 4: npm install
    current_step += 1;
    step(current_step, "install dependencies");
    println!();
    println!(
        "▶ Step {}/{}: Installing dependencies...",
        current_step, total_steps
    );
    flow_step(project.npm_install())?;
    if proc_host::cancelled() {
        return finish_cancelled();
    }

    // Step 5 (fallback): registry add for long-tail components outside the
    // bundle; a no-op with "already installed (skipping)" when fully bundled.
    flow_step(project.install_shadcn_components())?;

    // Step 5: Copy public assets
    current_step += 1;
    step(current_step, "copy public assets");
    println!();
    println!(
        "▶ Step {}/{}: Copying public assets...",
        current_step, total_steps
    );
    flow_step(project.copy_public_assets())?;

    // Step 5.5: Start API backend server.
    // Plan 346: --server=vm starts AutoVM HTTP server; --server=rust (default)
    // starts the a2r-generated Rust axum server.
    let mut _api_child: Option<std::process::Child> = None;
    let backend_impl = std::env::var("AUTO_BACKEND_IMPL").unwrap_or_else(|_| "rust".to_string());
    if backend_impl == "vm" {
        // Vue+VM: AutoVM HTTP server as backend.
        crate::run_vm::start_vm_server(
            root_dir,
            req.and_then(|r| crate::util::request_front_dir(root_dir, r))
                .as_deref(),
        );
    } else {
        // Vue+Rust: a2r-generated Rust axum server.
        // PLAN-026 T-05: ready 失败显式中止(不进 dev-server 步)。
        match crate::rust_ui::start_api_server(root_dir) {
            Ok(Some(child)) => _api_child = Some(child),
            Ok(None) => {}
            Err(e) => return Err(format!("vue backend start failed: {}", e).into()),
        }
    }
    if proc_host::cancelled() {
        if let Some(mut child) = _api_child {
            let _ = child.kill();
        }
        return finish_cancelled();
    }

    // Step 6: npm run dev
    current_step += 1;
    step(current_step, "start dev server");
    println!();
    println!(
        "▶ Step {}/{}: Starting dev server...",
        current_step, total_steps
    );
    let dev_result = project.npm_run_dev(args);

    // Cleanup: stop API backend server when dev server exits
    if let Some(mut child) = _api_child {
        let _ = child.kill();
        println!("  ✓ API server (Rust) stopped");
    } else if backend_impl == "vm" {
        // VM server runs on a background thread — process exit cleans it up.
        println!("  ✓ API server (VM) stopped");
    }

    if proc_host::cancelled() {
        return finish_cancelled();
    }
    flow_step(dev_result)
}

/// PLAN-658 T-03: 画廊多后端 proxy 启动编排（rust_ui 画廊钩子在
/// refresh_gallery_registry **之前**调用——发射器 T-02 前缀化消费线程局部
/// 根）。v1 注册面：全部内嵌档 demo 的宿主原生 media 路由
/// （`/api/media/scan` + `/api/media/stream/:id`，media_root 取自各自
/// pac.at；无 root 亦注册——诚实空列表与生成器一致）。注册判定镜像
/// registry.at 的 loadable 三档并集（write_registry_at 同式）——020 为
/// loadable 档（前端语料无 `@/lib/api`，fullstack=false），按 fullstack
/// 过滤会漏（T-03 实测修正）。T-04/T-05 起 stream/native-ns demo 的 VM
/// session 在此追加。失败降级：告警 + None（画廊继续跑，内嵌 demo 维持
/// 静态诚实空态，不阻断一键体验）。
pub fn start_gallery_back_proxy(project_dir: &Path) -> Option<u16> {
    let Ok(apps_dir) = gallery_apps_dir(project_dir) else {
        return None;
    };
    let mut sessions: Vec<auto_ui_runtime::back_proxy::SessionSpec> = Vec::new();
    let native_media: Vec<auto_ui_runtime::back_proxy::NativeMediaApp> =
        gallery_rows_via_cache(&apps_dir, true)
            .into_iter()
            .filter_map(|row| {
                // PLAN-658 T-04/T-05: stream（~Stream）与 native-ns（use auto.*）
                // 后端 demo 加载为 VM session——#[api] CRUD 在 session 内执行，
                // 流端点按 proxy 签名特路服务，原生命名空间经进程级注册表可用。
                let back_api = apps_dir
                    .join(&row.id)
                    .join("src")
                    .join("back")
                    .join("api.at");
                // PLAN-037 T-02: 谓词（back_needs_session）随 UI 运行域在
                // auto-ui-runtime 公开（814-B 迁移，815 T-05 消费）。
                let needs_session = std::fs::read_to_string(&back_api)
                    .map(|c| auto_ui_runtime::ui::back_provision::back_needs_session(&c))
                    .unwrap_or(false);
                // Plan 672 条目 4: fullstack 档（普通 #[api] CRUD）同样建
                // session——VM 臂此前靠 inproc 合并编译 CALL 面消费（谓词原
                // 注释），Vue 臂浏览器 fetch 必须经 proxy /apps/<id>/ 伺服，
                // 无会话即 404。VM 臂侧多出的空闲 session 无害（demo 语料
                // 仍走 inproc，不经 proxy 路由）。
                let fullstack_session = row.fullstack && back_api.is_file();
                // PLAN-675 T-08: routable 档会话准入——路由 demo 的 back 链同为
                // 普通 CRUD（needs_session 谓词只认 ~Stream/~Promise/use auto.），
                // 不补此臂则五家路由 demo 的 API 全数 `404 unknown app`（走查
                // 数据冻结真身之二，与 N2 死亡窗口叠加）。
                let routable_session = row.routable && back_api.is_file();
                if (needs_session || fullstack_session || routable_session) && back_api.is_file() {
                    sessions.push(auto_ui_runtime::back_proxy::SessionSpec {
                        app_id: row.id.clone(),
                        back_entry: back_api,
                    });
                }
                if !(row.loadable || row.fullstack || row.route_stub) {
                    return None;
                }
                Some(auto_ui_runtime::back_proxy::NativeMediaApp {
                    app_id: row.id.clone(),
                    media_root: pac_media_root(&row.pac),
                })
            })
            .collect();
    if native_media.is_empty() && sessions.is_empty() {
        return None;
    }
    let app_count = native_media.len();
    let session_count = sessions.len();
    let config = auto_ui_runtime::back_proxy::BackProxyConfig {
        port: 0,
        sessions,
        // PLAN-675 T-08: lazy 档——点开 demo 即载对应会话（全量预装载
        // 赶不上点击：走查实证 N2 死亡窗口内路由 demo 会话永远轮空，
        // 且 12 家 VM 常驻改为按访问付费更省）。
        lazy_sessions: true,
        native_media,
        // PLAN-043 Part 1: ui-gallery 当前无 photo 声明者——空表。
        native_photos: Vec::new(),
    };
    match auto_ui_runtime::back_proxy::start(config) {
        Ok(proxy) => {
            let base = format!("http://127.0.0.1:{}", proxy.port);
            set_gallery_proxy_root(Some(base.clone()));
            println!(
                "  {} Gallery back-proxy: {base} ({app_count} apps, {session_count} sessions)",
                "✓".bright_green(),
            );
            Some(proxy.port)
        }
        Err(e) => {
            set_gallery_proxy_root(None);
            println!(
                "  {} Gallery back-proxy failed to start: {e} (embedded demos stay static)",
                "⚠".bright_yellow()
            );
            None
        }
    }
}
