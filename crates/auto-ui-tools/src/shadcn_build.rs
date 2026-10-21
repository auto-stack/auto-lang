//! shadcn 组件构建组合口（PLAN-815 T-04 自 auto-lang lib.rs 根包装器平移）。
//!
//! 单算法合同：组件生成的唯一实现在 `auto-ui-compiler::ui_gen::
//! generate_component_from_file`；本模块只是选项组装 + 产物写盘的组合
//! 胶水（原 `auto_lang::ui_build_shadcn_*` 根包装器的冻结平移，供
//! vue.rs 画廊/桌面生成消费）。旧 auto-lang 根包装器作为兼容公开面原样
//! 保留（其自有消费者不受影响）；两处胶水不承载第二份生成算法。

use std::collections::HashMap;
use std::path::Path;

use auto_ui_compiler::aura::AuraWidget;
use auto_ui_compiler::ui_gen;
use auto_ui_compiler::ui_gen::api::resolve_stream_endpoints_for_project;
use auto_ui_compiler::ui_gen::validators::print_warnings_once;
use auto_ui_compiler::ui_gen::{
    generate_component_from_file, ComponentGenOptions, GeneratedComponent,
};

use auto_val::AutoResult;

pub fn ui_build_shadcn_with_widgets(
    path: &str,
    output: Option<&str>,
) -> AutoResult<(String, Vec<AuraWidget>)> {
    let (vue_code, widgets, _stores) =
        ui_build_shadcn_with_widgets_and_stores(path, output, None, None, None)?;
    Ok((vue_code, widgets))
}

/// Plan 043 store-codegen: same as [`ui_build_shadcn_with_widgets`], but also
/// returns the generated store composable files `(filename, code)`. Use this
/// when compiling multiple .at files in a loop and you need to collect stores
/// explicitly (the `STORE_EXTRA_FILES` thread-local is cleared per call).
pub fn ui_build_shadcn_with_widgets_and_stores(
    path: &str,
    output: Option<&str>,
    root_dir: Option<&str>,
    shadcn: Option<bool>,
    default_classes: Option<bool>,
) -> AutoResult<(String, Vec<AuraWidget>, Vec<(String, String)>)> {
    let at_path = Path::new(path);
    let stream_endpoints = root_dir.map(resolve_stream_endpoints_for_project);
    let opts = ComponentGenOptions {
        stream_endpoints,
        shadcn,
        default_classes,
        ..Default::default()
    };
    let result = generate_component_from_file(at_path, opts).map_err(|e| format!("{}", e))?;
    let store_composables = result.store_composables.clone();

    // Plan 012 Batch A: surface codegen validation warnings on the build path
    // (deduplicated — the tool pre-scans each file before the real pass).
    print_warnings_once(path, &result.validation_warnings);

    // Write output if specified (legacy behavior)
    if let Some(out_dir) = output {
        std::fs::create_dir_all(out_dir).ok();
        for (name, code) in &result.all_widget_codes {
            let out_path = Path::new(out_dir).join(format!("{}.vue", name));
            std::fs::write(&out_path, code)
                .map_err(|e| format!("Failed to write output file: {}", e))?;
        }
        // Also write store composables
        for (filename, code) in &result.store_composables {
            let out_path = Path::new(out_dir).join(filename);
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::write(&out_path, code).ok();
        }
    }

    Ok((result.vue_code, result.widgets, store_composables))
}

/// Plan 408 P11 / KNOWN-DEBT pages 写盘：薄包装，返回完整 `GeneratedComponent`
/// （含 `all_widget_codes`——每个 component fn / widget 一个 SFC）。
pub fn ui_build_shadcn_all_widget_codes(
    path: &str,
    root_dir: Option<&str>,
    shadcn: Option<bool>,
    default_classes: Option<bool>,
) -> AutoResult<GeneratedComponent> {
    ui_build_shadcn_all_widget_codes_with_bound(path, root_dir, shadcn, default_classes, None)
}

/// Plan 443: `ui_build_shadcn_all_widget_codes` + the workspace-aggregated
/// bound-channel map (which model channels some parent actually binds).
pub fn ui_build_shadcn_all_widget_codes_with_bound(
    path: &str,
    root_dir: Option<&str>,
    shadcn: Option<bool>,
    default_classes: Option<bool>,
    bound_model_channels: Option<HashMap<String, Vec<String>>>,
) -> AutoResult<GeneratedComponent> {
    let at_path = Path::new(path);
    let stream_endpoints = root_dir.map(resolve_stream_endpoints_for_project);
    let opts = ComponentGenOptions {
        stream_endpoints,
        shadcn,
        default_classes,
        bound_model_channels,
        ..Default::default()
    };
    let result = generate_component_from_file(at_path, opts).map_err(|e| format!("{}", e))?;
    print_warnings_once(path, &result.validation_warnings);
    Ok(result)
}

/// Like `ui_build_shadcn_with_widgets`, but accepts a list of known sub-widget
/// names to avoid naming collisions with shadcn-vue components (e.g. custom
/// "Sidebar" widget should not be mapped to shadcn's Sidebar). PLAN-024 第四
/// 返回段 = named_view_codes（widget, view, SFC code）——桌面 vue 宿主
/// Mini.vue 产物源。
#[allow(clippy::type_complexity)]
pub fn ui_build_shadcn_with_sub_widgets_and_stores_full(
    path: &str,
    output: Option<&str>,
    sub_widget_names: Vec<String>,
    sub_widget_models: Option<HashMap<String, Vec<String>>>,
    root_dir: Option<&str>,
    shadcn: Option<bool>,
    default_classes: Option<bool>,
    bound_model_channels: Option<HashMap<String, Vec<String>>>,
    sub_widget_msgs: Option<HashMap<String, Vec<String>>>,
) -> AutoResult<(
    String,
    Vec<AuraWidget>,
    Vec<(String, String)>,
    Vec<(String, String, String)>,
)> {
    let at_path = Path::new(path);
    let stream_endpoints = root_dir.map(resolve_stream_endpoints_for_project);
    let opts = ComponentGenOptions {
        sub_widgets: Some(sub_widget_names),
        sub_widget_models,
        bound_model_channels,
        stream_endpoints,
        shadcn,
        default_classes,
        sub_widget_msgs,
        ..Default::default()
    };
    let result = generate_component_from_file(at_path, opts).map_err(|e| format!("{}", e))?;
    let store_composables = result.store_composables.clone();

    print_warnings_once(path, &result.validation_warnings);

    if let Some(out_dir) = output {
        std::fs::create_dir_all(out_dir).ok();
        for (name, code) in &result.all_widget_codes {
            let out_path = Path::new(out_dir).join(format!("{}.vue", name));
            std::fs::write(&out_path, code)
                .map_err(|e| format!("Failed to write output file: {}", e))?;
        }
        for (filename, code) in &result.store_composables {
            let out_path = Path::new(out_dir).join(filename);
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::write(&out_path, code)
                .map_err(|e| format!("Failed to write store file: {}", e))?;
        }
    }

    Ok((
        result.vue_code,
        result.widgets,
        store_composables,
        result.named_view_codes,
    ))
}

// 组合胶水仅消费 ui_gen 公开面；此别名仅为压住未用 import 误报前的占位。
#[allow(unused_imports)]
use ui_gen as _ui_gen;
