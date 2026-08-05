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

use auto_lang::ui_gen::{VueGenerator, WidgetCategory, WidgetRegistry};
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
        UiAction::Inspect { file } => inspect(&file),
        UiAction::Backlog => backlog(),
    }
}

/// Plan 337 Task 2.2: List AURA widgets not yet covered by @auto-ui/widgets,
/// grouped by WidgetCategory.
fn backlog() -> Result<()> {
    let reg = WidgetRegistry::with_defaults();
    let all = reg.all_widgets();

    let total = all.len();
    let uncovered: Vec<(&String, &WidgetCategory)> = all
        .iter()
        .filter(|(tag, _)| !VueGenerator::covers_aura_tag(tag.as_str()))
        .map(|(tag, spec)| (tag, &spec.category))
        .collect();
    let covered = total - uncovered.len();

    // Group by category.
    let mut by_cat: std::collections::BTreeMap<&str, Vec<&str>> = std::collections::BTreeMap::new();
    for (tag, cat) in &uncovered {
        let cat_name = match cat {
            WidgetCategory::Layout => "Layout",
            WidgetCategory::Form => "Form",
            WidgetCategory::Display => "Display",
            WidgetCategory::Navigation => "Navigation",
            WidgetCategory::Semantic => "Semantic",
            WidgetCategory::Overlay => "Overlay",
            WidgetCategory::Feedback => "Feedback",
            WidgetCategory::Data => "Data",
        };
        by_cat.entry(cat_name).or_default().push(tag.as_str());
    }

    if uncovered.is_empty() {
        println!("All {} AURA widgets are covered by @auto-ui/widgets.", total);
        return Ok(());
    }

    println!("AURA widgets not yet in @auto-ui/widgets ({} uncovered / {} total):\n", uncovered.len(), total);
    for (cat, tags) in &by_cat {
        println!("  {}:", cat);
        for tag in tags {
            println!("    - {}", tag);
        }
    }
    println!("\n{} covered / {} total ({} uncovered)", covered, total, uncovered.len());
    Ok(())
}

/// Generate self-contained widget SFCs into `out`.
fn build(target: &str, out: &str, widgets: &[String]) -> Result<()> {
    // Plan 337 Task 3.2: gallery-stubs target generates vue-gallery scaffolding.
    if target == "gallery-stubs" {
        return build_gallery_stubs(out);
    }

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

    // Shared files (registry root): the `cn` helper every widget imports.
    for (rel, content) in gen.library_shared_files() {
        let path = out_dir.join(rel);
        fs::write(&path, content)
            .map_err(|e| miette::miette!("write shared {path:?}: {e}"))?;
    }

    println!(
        "wrote {} widget{} to {}",
        written,
        if written == 1 { "" } else { "s" },
        out
    );
    Ok(())
}

/// Plan 337 Task 3.2: Generate vue-gallery scaffolding.
///
/// - Rewrites `widgets.generated.ts` (name+route, sorted).
/// - For each widget without an existing `pages/<name>.vue`, writes a minimal
///   stub page (does NOT overwrite existing hand-written pages).
fn build_gallery_stubs(out: &str) -> Result<()> {
    let src_dir = Path::new(out);
    let pages_dir = src_dir.join("pages");
    fs::create_dir_all(&pages_dir)
        .map_err(|e| miette::miette!("failed to create pages dir: {e}"))?;

    // 1. Rewrite widgets.generated.ts
    let mut gen_ts = String::from(
        "// DO NOT EDIT — generated by `auto ui build --target gallery-stubs`.\n\
         // Widget names + routes, derived from LIBRARY_WIDGETS (sorted).\n\
         export interface GeneratedWidgetEntry {\n  name: string\n  route: string\n}\n\n\
         export const generatedWidgets: GeneratedWidgetEntry[] = [\n",
    );
    for name in VueGenerator::LIBRARY_WIDGETS {
        gen_ts.push_str(&format!("  {{ name: '{name}', route: '/{name}' }},\n"));
    }
    gen_ts.push_str("]\n");
    let gen_path = src_dir.join("widgets.generated.ts");
    fs::write(&gen_path, &gen_ts)
        .map_err(|e| miette::miette!("write {gen_path:?}: {e}"))?;

    // 2. Generate stub pages for missing widgets
    let total = VueGenerator::LIBRARY_WIDGETS.len();
    let mut existing = 0usize;
    let mut created = 0usize;
    for name in VueGenerator::LIBRARY_WIDGETS {
        let page_path = pages_dir.join(format!("{name}.vue"));
        if page_path.exists() {
            existing += 1;
        } else {
            let stub = gallery_page_stub(name);
            fs::write(&page_path, &stub)
                .map_err(|e| miette::miette!("write {page_path:?}: {e}"))?;
            created += 1;
        }
    }

    println!(
        "regenerated widgets.generated.ts ({}); pages existing: {}, created: {}",
        total, existing, created
    );
    Ok(())
}

/// Minimal vue-gallery page stub for a widget (Plan 337 Task 3.2).
fn gallery_page_stub(name: &str) -> String {
    let pascal = pascal_case(name);
    format!(
        r#"<script setup lang="ts">
// Plan 337: auto-generated stub page for {name}
// TODO: add variants, states, code examples
import {{ {pascal} }} from '@auto-ui/widgets/registry/{name}'
</script>

<template>
  <div class="page">
    <h2>{name}</h2>
    <!-- TODO: variants/states -->
    <div class="demo-block">
      <{pascal} />
    </div>
  </div>
</template>
"#
    )
}

/// Inspect an .at file: parse, extract widgets, show structure + validation.
///
/// Plan 362 Phase 5: provides the core value of `auto ui repl` without
/// the full interactive REPL. Shows:
///  - Widget count, names
///  - Props, state vars, handlers, messages per widget
///  - API imports and store dependencies
///  - SFC code preview (first 5 lines)
///  - Validation warnings (from Plan 361 validators)
fn inspect(path: &str) -> Result<()> {
    use auto_lang::ui_gen::{generate_component_from_file, ComponentGenOptions};
    use std::path::Path;

    let at_path = Path::new(path);
    if !at_path.exists() {
        return Err(miette::miette!("File not found: {}", path));
    }

    println!("{} {}", "▸".bright_cyan(), path);
    println!();

    let opts = ComponentGenOptions::default();
    let result = generate_component_from_file(at_path, opts)
        .map_err(|e| miette::miette!("{}", e))?;

    println!("Widgets: {}", result.widgets.len());
    for w in &result.widgets {
        println!("  ┌─ {}", w.name.bright_white().bold());
        println!("  │  props:    {}", w.props.len());
        for p in &w.props {
            println!("  │    • {}: {:?}", p.name, p.type_info);
        }
        println!("  │  state:    {}", w.state_vars.len());
        for s in &w.state_vars {
            println!("  │    • {}: {:?}", s.name, s.type_info);
        }
        let mut handler_names: Vec<&str> =
            w.handlers.keys().map(|k| k.as_str()).collect();
        handler_names.sort();
        println!("  │  handlers: {}", handler_names.len());
        for h in &handler_names {
            println!("  │    • {}", h);
        }
        println!("  │  messages: {}", w.messages.len());
        for m in &w.messages {
            println!("  │    • {} ({} variants)", m.name, m.variants.len());
        }
        println!("  └─");
    }

    // API & Store
    if !result.detected_api_imports.is_empty() {
        println!(
            "API imports: {}",
            result.detected_api_imports.join(", ")
        );
    }
    if !result.detected_store_deps.is_empty() {
        println!(
            "Store deps:  {}",
            result.detected_store_deps.join(", ")
        );
    }
    if !result.store_composables.is_empty() {
        println!("Store composables:");
        for (filename, _code) in &result.store_composables {
            println!("  • {}", filename);
        }
    }

    // SFC preview (first few lines)
    for (name, code) in &result.all_widget_codes {
        let preview: String = code.lines().take(5).collect::<Vec<_>>().join("\n");
        println!("\n{} SFC preview ({} bytes):", name, code.len());
        println!("{}", preview);
        if code.lines().count() > 5 {
            println!("  ... ({} more lines)", code.lines().count() - 5);
        }
    }

    // Validation warnings
    if !result.validation_warnings.is_empty() {
        println!();
        let warn_text =
            auto_lang::ui_gen::validators::format_warnings(&result.validation_warnings);
        println!("{}", warn_text);
    } else {
        println!("\n{} No validation warnings", "✓".bright_green());
    }

    Ok(())
}

use colored::Colorize;
