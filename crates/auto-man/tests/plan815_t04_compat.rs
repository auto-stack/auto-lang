//! PLAN-815 T-04 R1-F03 兼容断言：迁移后的旧公开路径必须可从 crate 外部
//! 导入并解析（编译即 oracle——函数项引用钉住路径与签名形态；运行主体
//! 的真实执行归 M814-B 后的 T-05，此处不冒充运行验证）。
//!
//! 覆盖同一旧模块的全公开支持面抽样：门面生成主体（VueProject/
//! generate_rust_ui/ensure_shared_workspace/api_gen/sidecar/pkg/util）与
//! R1 修复的六个 run 入口。

use std::path::Path;

// R1-F03 六个断裂入口（vue/rust_ui run 家族）。
#[allow(dead_code)]
fn vue_run_entries() {
    let _ = auto_man::vue::run_vue_project;
    let _ = auto_man::vue::run_vue_project_with_request;
    let _ = auto_man::vue::start_gallery_back_proxy;
    let _ = auto_man::rust_ui::run_vm_ui;
    let _ = auto_man::rust_ui::run_vm_ui_with_request;
    let _ = auto_man::rust_ui::start_vm_server;
}

// 同一旧模块的生成主体支持面（抽样，非穷举）。
#[allow(dead_code)]
fn generation_surface_entries() {
    let _ = auto_man::vue::gen_vue_project;
    let _ = auto_man::vue::build_vue_project;
    let _ = auto_man::vue::VueProject::from_workspace;
    let _ = auto_man::vue::gallery_mode;
    let _ = auto_man::rust_ui::generate_rust_ui;
    let _ = auto_man::rust_ui::generate_rust_ui_delivery;
    let _ = auto_man::rust_ui::ensure_shared_workspace;
    let _ = auto_man::rust_ui::DELIVERY_CRATES;
    let _ = auto_man::api_gen::generate_api;
    let _ = auto_man::api_gen::try_full_parse;
    let _ = auto_man::sidecar::load_sidecar;
    let _ = auto_man::pkg::install;
    let _ = auto_man::http_port;
    let _ = auto_man::vue_shadcn::bundled_components;
    let _ = auto_man::gallery_assets::materialize;
    let _ = auto_man::image_viewer::ImageViewerService::default;
}

#[test]
fn historic_public_paths_resolve() {
    // 路径可解析即通过；真实调用行为由各归属域测试验证。
    let _ = Path::new(".");
    vue_run_entries();
    generation_surface_entries();
}
