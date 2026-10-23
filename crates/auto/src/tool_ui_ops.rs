//! ToolUiOps——runtime `ui_ops` seam 的工具进程实现（PLAN-816 T-05）。
//!
//! 薄 CLI 的 UI 工程臂（build/run/gen 的 UI 后端）**不静态链接
//! auto-ui-tools**：经 [`auto_tool_client`] spawn 真实 `auto-ui-tool`
//! 进程，以 I-TOOL v1 五操作（generate_ui/build_ui/run_ui）执行。
//! 本适配器是纯委托（UiOps 方法→wire 请求→工具进程回执→结果映射），
//! 算法零复制；工具发现经 T-04 discovery（env→workspace→邻接，缺件具名）。
//!
//! 惰性连接：首次 UI 臂调用才发现并 spawn 工具——非 UI 命令（裸脚本/
//! REPL/语言/管理）不触达工具，Core/无工具安装形态下全部非 UI 命令
//! 照常工作（V-01 语义）。工具缺失/版本/能力不匹配=具名失败，无静态
//! UI 大库 fallback。
//!
//! 已知边界（T-05 交接注记，V-04 待验）：
//! - `--lenient/--strict`（build）与 vue run 的透传 args 不入 I-TOOL v1
//!   请求字段——工具侧按其请求规则执行；协议字段增补属 815 消费协商。
//! - run 臂的交互（`q`）经客户端 stdin 泵转发（进程生命周期注记见
//!   `ToolClient::run_ui`）。

use std::path::Path;
use std::sync::{Arc, Mutex};

use auto_man_runtime::ui_ops::UiOps;
use auto_project_model::GenerationRequest;
use auto_tool_client::discovery::{self, DiscoverySpec};
use auto_tool_client::ToolClient;
use auto_val::AutoResult;

/// 生成/构建的 strict 信号（PLAN-816 T-05 R1 F-05：旧 CLI build 默认
/// 严格校验——CLI 命令臂在此登记，适配器在 op 时刻读取并**显式穿进程
/// 边界**给工具；不用本进程全局状态冒充跨进程传递）。None=命令未声明
/// （工具默认）。
static GENERATION_STRICT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static GENERATION_STRICT_SET: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// 登记本次进程生命周期的 strict 语义（CLI build/gen 臂调用）。
pub fn set_generation_strict(strict: Option<bool>) {
    GENERATION_STRICT.store(
        strict.unwrap_or(false),
        std::sync::atomic::Ordering::Relaxed,
    );
    GENERATION_STRICT_SET.store(strict.is_some(), std::sync::atomic::Ordering::Relaxed);
}

fn generation_strict() -> Option<bool> {
    if GENERATION_STRICT_SET.load(std::sync::atomic::Ordering::Relaxed) {
        Some(GENERATION_STRICT.load(std::sync::atomic::Ordering::Relaxed))
    } else {
        None
    }
}

/// 安装 ToolUiOps 到 runtime 的显式槽（CLI 启动时调用一次；幂等顺序上
/// 先于任何管理编排）。本函数**不发现工具**——惰性到首个 UI 臂调用。
pub fn install() {
    auto_man_runtime::ui_ops::install(Arc::new(ToolUiOps {
        client: Mutex::new(None),
    }));
}

struct ToolUiOps {
    /// 惰性工具连接（进程内单连接复用；启动失败在每次臂调用重试发现）。
    client: Mutex<Option<ToolClient>>,
}

impl ToolUiOps {
    /// 取活动连接；无则发现并 spawn。发现/spawn 失败=具名失败。
    /// 泛型 `FnOnce`（单发请求语义——每次臂调用恰好一个操作）。
    fn with_client<T, F>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&mut ToolClient) -> Result<T, String>,
    {
        let mut guard = self.client.lock().unwrap();
        if guard.is_none() {
            let spec = DiscoverySpec::auto_ui_tool();
            let discovered = discovery::discover(&spec)
                .map_err(|e| format!("auto-ui-tool discovery failed: {e}"))?;
            let mut spec = auto_tool_client::ToolClientSpec::new(discovered.path.clone());
            // 活日志：工具 stderr 实时转发到本进程 stderr（dev-server 等
            // 长运行输出的人读 parity；旧行为=子进程继承 stdio 直达终端）。
            spec.live_log = Some(Arc::new(|chunk: &[u8]| {
                eprint!("{}", String::from_utf8_lossy(chunk));
            }));
            let client =
                ToolClient::spawn(spec).map_err(|e| format!("auto-ui-tool spawn failed: {e}"))?;
            *guard = Some(client);
        }
        let client = guard.as_mut().unwrap();
        f(client)
    }

    //（R1-04 后所有生产臂消费传入的有效请求，无回读 pac 的基请求构造。）
    /// 工具合同要求绝对 project_root（work ops 的路径锚）。runtime 侧
    /// root_dir 可能相对（Automan::new(".")）——以 cwd join 绝对化
    /// （不用 canonicalize：避免 Windows verbatim `\?\` 前缀扰动
    /// 工具侧路径拼接/显示）。
    fn absolutized(&self, mut request: GenerationRequest) -> GenerationRequest {
        if !request
            .project_root
            .as_ref()
            .is_some_and(|r| r.is_absolute())
        {
            let rel = request
                .project_root
                .clone()
                .unwrap_or_else(|| std::path::PathBuf::from("."));
            let abs = match std::env::current_dir() {
                Ok(cwd) if rel == std::path::Path::new(".") => cwd,
                Ok(cwd) => cwd.join(&rel),
                Err(_) => rel,
            };
            request.project_root = Some(abs);
        }
        request
    }

    fn generate(&self, request: GenerationRequest) -> AutoResult<()> {
        self.generate_opts(request, auto_tool_client::GenerateUiOpts::default())
    }

    /// PLAN-816 T-05 R1 F-01/F-05：执行选项（output/project/delivery/
    /// strict）显式跨进程传递；strict 未显式声明时读 CLI 登记。
    fn generate_opts(
        &self,
        request: GenerationRequest,
        mut opts: auto_tool_client::GenerateUiOpts,
    ) -> AutoResult<()> {
        if opts.strict.is_none() {
            opts.strict = generation_strict();
        }
        let request = self.absolutized(request);
        self.with_client(|c| match c.generate_ui_opts(request, &opts) {
            Ok(Ok(report)) => {
                println!(
                    "  [tool] generated {} files under {} (frontends: {})",
                    report.files_written,
                    report.output_root,
                    report.frontends.join(",")
                );
                Ok(())
            }
            Ok(Err(e)) => Err(format!("generate_ui rejected: {e} [kind={:?}]", e.kind)),
            Err(e) => Err(format!("generate_ui failed: {e}")),
        })
        .map_err(|e| -> auto_val::AutoError { e.into() })
    }

    fn build(&self, request: GenerationRequest) -> AutoResult<()> {
        let strict = generation_strict();
        let request = self.absolutized(request);
        self.with_client(|c| match c.build_ui_opts(request, strict) {
            Ok(Ok(report)) => {
                println!("  [tool] build {} ({:?})", report.profile, report.outcome);
                if let Some(err) = &report.error {
                    eprintln!("  [tool] build error: {err}");
                }
                if report.outcome == auto_tool_protocol::RunOutcome::Failed {
                    return Err(report.error.unwrap_or_else(|| "build failed".into()));
                }
                Ok(())
            }
            Ok(Err(e)) => Err(format!("build_ui rejected: {e} [kind={:?}]", e.kind)),
            Err(e) => Err(format!("build_ui failed: {e}")),
        })
        .map_err(|e| -> auto_val::AutoError { e.into() })
    }

    /// run 臂的透传参数（F-05：`auto run -- <args>` 旧语义穿边界）。
    fn run_with_args(&self, request: GenerationRequest, args: Vec<String>) -> AutoResult<()> {
        let request = self.absolutized(request);
        self.with_client(|c| match c.run_ui_opts(request, args, true) {
            Ok(Ok(report)) => {
                if let Some(url) = &report.launch {
                    println!("  [tool] service launched at {url}");
                }
                println!("  [tool] run {} -> {:?}", report.profile, report.outcome);
                if report.outcome == auto_tool_protocol::RunOutcome::Failed {
                    return Err(report.error.unwrap_or_else(|| "run failed".into()));
                }
                Ok(())
            }
            Ok(Err(e)) => Err(format!("run_ui rejected: {e} [kind={:?}]", e.kind)),
            Err(e) => Err(format!("run_ui failed: {e}")),
        })
        .map_err(|e| -> auto_val::AutoError { e.into() })
    }
}

impl UiOps for ToolUiOps {
    fn vue_build(&self, root_dir: &Path, request: &GenerationRequest) -> AutoResult<()> {
        // F-816-T05-R1-04：消费本次有效请求（render 覆盖在场即权威）。
        let _ = root_dir;
        self.build(request.clone())
    }

    fn vue_generate(&self, root_dir: &Path, request: &GenerationRequest) -> AutoResult<()> {
        let _ = root_dir;
        self.generate(request.clone())
    }

    fn vue_generate_workspace(
        &self,
        root_dir: &Path,
        request: &GenerationRequest,
    ) -> AutoResult<()> {
        let _ = root_dir;
        self.generate(request.clone())
    }

    fn vue_run(
        &self,
        root_dir: &Path,
        args: Vec<String>,
        request: &GenerationRequest,
    ) -> AutoResult<()> {
        let _ = root_dir;
        self.run_with_args(request.clone(), args)
    }

    fn rust_build(&self, root_dir: &Path, request: &GenerationRequest) -> AutoResult<()> {
        let _ = root_dir;
        self.build(request.clone())
    }

    fn rust_generate(
        &self,
        root_dir: &Path,
        output: Option<&Path>,
        project: bool,
        request: &GenerationRequest,
    ) -> AutoResult<()> {
        let _ = root_dir;
        let opts = auto_tool_client::GenerateUiOpts {
            output: output.map(|p| p.to_path_buf()),
            project: Some(project),
            ..Default::default()
        };
        self.generate_opts(request.clone(), opts)
    }

    fn rust_generate_delivery(
        &self,
        root_dir: &Path,
        delivery_root: &Path,
        request: &GenerationRequest,
    ) -> AutoResult<()> {
        let _ = root_dir;
        // F-816-T05-R1-01：792 显式交付模式是**独立 op 选项**（生成可
        // 搬移 bundle + delivery-manifest），不是请求 output_root 字段
        // （该字段不重定向生成布局——815 字段矩阵已钉）。
        let opts = auto_tool_client::GenerateUiOpts {
            delivery_root: Some(delivery_root.to_path_buf()),
            ..Default::default()
        };
        self.generate_opts(request.clone(), opts)
    }

    fn rust_run(
        &self,
        root_dir: &Path,
        args: Vec<String>,
        request: &GenerationRequest,
    ) -> AutoResult<()> {
        let _ = root_dir;
        self.run_with_args(request.clone(), args)
    }

    fn vm_run(
        &self,
        root_dir: &Path,
        args: Vec<String>,
        request: &GenerationRequest,
    ) -> AutoResult<()> {
        let _ = root_dir;
        self.run_with_args(request.clone(), args)
    }

    fn jet_generate(
        &self,
        root_dir: &Path,
        output: Option<&Path>,
        project: bool,
        request: &GenerationRequest,
    ) -> AutoResult<()> {
        let _ = root_dir;
        let opts = auto_tool_client::GenerateUiOpts {
            output: output.map(|p| p.to_path_buf()),
            project: Some(project),
            ..Default::default()
        };
        self.generate_opts(request.clone(), opts)
    }

    fn jet_build(&self, root_dir: &Path, request: &GenerationRequest) -> AutoResult<()> {
        let _ = root_dir;
        self.build(request.clone())
    }

    fn jet_run(
        &self,
        root_dir: &Path,
        args: Vec<String>,
        request: &GenerationRequest,
    ) -> AutoResult<()> {
        let _ = root_dir;
        self.run_with_args(request.clone(), args)
    }

    fn ark_generate(
        &self,
        root_dir: &Path,
        output: Option<&Path>,
        project: bool,
        request: &GenerationRequest,
    ) -> AutoResult<()> {
        let _ = root_dir;
        let opts = auto_tool_client::GenerateUiOpts {
            output: output.map(|p| p.to_path_buf()),
            project: Some(project),
            ..Default::default()
        };
        self.generate_opts(request.clone(), opts)
    }

    fn ark_build(&self, root_dir: &Path, request: &GenerationRequest) -> AutoResult<()> {
        let _ = root_dir;
        self.build(request.clone())
    }

    fn ark_run(
        &self,
        root_dir: &Path,
        args: Vec<String>,
        request: &GenerationRequest,
    ) -> AutoResult<()> {
        let _ = root_dir;
        self.run_with_args(request.clone(), args)
    }

    fn vscode_generate(
        &self,
        root_dir: &Path,
        output: Option<&Path>,
        project: bool,
        request: &GenerationRequest,
    ) -> AutoResult<()> {
        let opts = auto_tool_client::GenerateUiOpts {
            output: output.map(|p| p.to_path_buf()),
            project: Some(project),
            ..Default::default()
        };
        let _ = root_dir;
        self.generate_opts(request.clone(), opts)
    }

    fn vscode_build(&self, root_dir: &Path, request: &GenerationRequest) -> AutoResult<()> {
        let _ = root_dir;
        self.build(request.clone())
    }

    fn vscode_run(
        &self,
        root_dir: &Path,
        args: Vec<String>,
        request: &GenerationRequest,
    ) -> AutoResult<()> {
        let _ = root_dir;
        self.run_with_args(request.clone(), args)
    }

    fn tauri_run(
        &self,
        root_dir: &Path,
        args: Vec<String>,
        request: &GenerationRequest,
    ) -> AutoResult<()> {
        let _ = root_dir;
        self.run_with_args(request.clone(), args)
    }

    fn create_jet_project(&self, name: &str) -> AutoResult<()> {
        // Jetpack 脚手架（JetProjectConfig 全流程）不是 I-TOOL 五操作——
        // ui-extra 形态经 auto-ui-compiler 单源执行（compat 同体）；Core
        // 形态具名失败。
        #[cfg(feature = "ui-extra")]
        {
            use auto_ui_compiler::ui_gen::jet::{JetProjectConfig, ProjectGenerator};
            let path = Path::new(name);
            if path.is_file() {
                return Err(format!("A file named with {name} already exists").into());
            }
            if path.is_dir()
                && path
                    .read_dir()
                    .map(|mut d| d.next().is_some())
                    .unwrap_or(false)
            {
                return Err(
                    format!("A non-empty directory named with {name} already exists").into(),
                );
            }
            let pac_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("MyApp");
            let config = JetProjectConfig::new(pac_name);
            let mut generator = ProjectGenerator::with_config(config);
            let files = generator.generate();
            std::fs::create_dir_all(path)?;
            for (file_path, content) in files {
                let full_path = path.join(&file_path);
                if let Some(parent) = full_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&full_path, content)?;
                log::info!("Created {}", full_path.display());
            }
            println!("Jetpack Compose project '{pac_name}' created successfully!");
            println!("Open with: auto open");
            Ok(())
        }
        #[cfg(not(feature = "ui-extra"))]
        {
            Err(format!(
                "`auto new -t jet` requires the full profile (ui-extra): jet scaffold (JetProjectConfig) is not linked in core builds (name={name})"
            )
            .into())
        }
    }

    fn pkg_display_name(&self) -> &'static str {
        "npm"
    }

    fn pkg_run_script(
        &self,
        script: &str,
        _extra_args: &[&str],
        _cwd: &Path,
    ) -> Result<(), String> {
        // 无 runtime 生产调用方（T-01 全量盘点）；工具进程路径的 npm
        // 编排由工具自身承担——诚实具名失败，不假成功。
        Err(format!(
            "pkg_run_script('{script}') is not served by the tool-process UI ops; \
             npm orchestration belongs to the tool host"
        ))
    }
}
