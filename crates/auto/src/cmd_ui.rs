//! `auto ui` command (Plan 331) — generate self-contained library widget SFCs.
//!
//! Usage:
//!   auto ui build --target vue --out packages/widgets/registry
//!   auto ui build --widgets button,input,label --out tmp/ui_build_test
//!   auto ui list
//!
//! This drives `VueGenerator::new_library()` to emit one independent `.vue`
//! per primitive (reka-ui import + Tailwind class, never `@/components/ui/*`),
//! plus the per-widget support files (`index.ts`, `variants.ts`, ...).

use std::fs;
use std::path::Path;

use auto_lang::ui_gen::VueGenerator;
use miette::Result;

use crate::UiAction;

/// PascalCase a kebab/lower widget key (`button` -> `Button`).
fn pascal_case(name: &str) -> String {
    name.split('_')
        .flat_map(|part| part.split('-'))
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect()
}

/// Entry point dispatched from `main`.
pub fn run(action: UiAction) -> Result<()> {
    match action {
        UiAction::Build { target, out, widgets } => build(&target, &out, &widgets),
        UiAction::List => {
            for name in VueGenerator::LIBRARY_WIDGETS {
                println!("{name}");
            }
            Ok(())
        }
    }
}

/// Generate self-contained widget SFCs into `out`.
fn build(target: &str, out: &str, widgets: &[String]) -> Result<()> {
    if target != "vue" {
        return Err(miette::miette!(
            "unsupported --target '{target}'; only 'vue' is supported (Plan 331)"
        ));
    }

    let names: Vec<&str> = if widgets.is_empty() {
        VueGenerator::LIBRARY_WIDGETS.to_vec()
    } else {
        widgets.iter().map(String::as_str).collect()
    };

    let out_dir = Path::new(out);
    fs::create_dir_all(out_dir)
        .map_err(|e| miette::miette!("failed to create output dir {out}: {e}"))?;

    let mut gen = VueGenerator::new_library();
    let mut written = 0usize;
    for name in &names {
        let widget_dir = out_dir.join(name);
        fs::create_dir_all(&widget_dir)
            .map_err(|e| miette::miette!("failed to create {widget_dir:?}: {e}"))?;

        let sfc = gen
            .generate_widget_sfc(name)
            .map_err(|e| miette::miette!("generate {name}: {e}"))?;

        let pascal = pascal_case(name);
        let sfc_path = widget_dir.join(format!("{pascal}.vue"));
        fs::write(&sfc_path, &sfc)
            .map_err(|e| miette::miette!("write {sfc_path:?}: {e}"))?;

        for (rel, content) in gen.generate_widget_support_files(name) {
            let path = widget_dir.join(&rel);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| miette::miette!("create dir for {path:?}: {e}"))?;
            }
            fs::write(&path, content)
                .map_err(|e| miette::miette!("write {path:?}: {e}"))?;
        }

        written += 1;
    }

    println!(
        "wrote {} widget{} to {}",
        written,
        if written == 1 { "" } else { "s" },
        out
    );
    Ok(())
}
