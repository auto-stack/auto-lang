//! `auto docs` (Plan 435 P8-1/D13) —— schema 驱动文档生成 CLI。
//!
//! `auto docs gen` 再生成两份产物(等价于测试内 DOCS_GEN_UPDATE /
//! KITCHEN_SINK_UPDATE 环境变量路径,走同一库实现 ui_gen::docs_gen):
//! - docs/components/core.md —— 核心组件参考
//! - examples/widgets-gallery/src/front/pages/kitchen-sink.at —— demo 页
//!
//! 生成后请复核 diff;kitchen-sink 变更需同步重采样 gallery golden。

use clap::Subcommand;
use miette::Result;

#[derive(Subcommand, Debug)]
pub enum DocsAction {
    /// Regenerate schema-driven docs artifacts (core.md + kitchen-sink.at)
    Gen {
        /// Only generate this artifact (core | kitchen-sink); default both
        #[arg(long)]
        only: Option<String>,
    },
}

pub fn run(action: DocsAction) -> Result<()> {
    match action {
        DocsAction::Gen { only } => {
            let want = |name: &str| only.as_deref().map_or(true, |o| o == name);
            let mut written: Vec<String> = Vec::new();
            if want("core") {
                let path = "docs/components/core.md";
                let text = auto_lang::ui_gen::docs_gen::generate_core_reference(std::path::Path::new("."));
                std::fs::create_dir_all("docs/components")
                    .map_err(|e| miette::miette!("create docs dir: {e}"))?;
                std::fs::write(path, text)
                    .map_err(|e| miette::miette!("write {path}: {e}"))?;
                written.push(path.to_string());
            }
            if want("kitchen-sink") || want("kitchen_sink") {
                let path = "examples/widgets-gallery/src/front/pages/kitchen-sink.at";
                let text = auto_lang::ui_gen::docs_gen::generate_kitchen_sink();
                std::fs::create_dir_all("examples/widgets-gallery/src/front/pages")
                    .map_err(|e| miette::miette!("create pages dir: {e}"))?;
                std::fs::write(path, text)
                    .map_err(|e| miette::miette!("write {path}: {e}"))?;
                written.push(path.to_string());
            }
            if written.is_empty() {
                return Err(miette::miette!(
                    "--only 仅支持 core | kitchen-sink"
                ));
            }
            println!(
                "wrote {} artifact{} —— 复核 diff;kitchen-sink 变更需重采样 gallery golden",
                written.len(),
                if written.len() == 1 { "" } else { "s" }
            );
            for w in &written {
                println!("  {w}");
            }
            Ok(())
        }
    }
}
