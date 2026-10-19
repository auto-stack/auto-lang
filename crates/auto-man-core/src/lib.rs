//! # auto-man-core
//!
//! 管理/构建规划规则核心（PLAN-809 T-03..T-05）。承接 pac.at 声明归一、
//! Target/Pac 纯选择与投影、链接闭包、源文件选择与构建规划的**纯规则**；
//! 语言执行（AutoConfig 求值/转译）、Git/FS 观测与物化、Builder 实际执行
//! 留在 auto-man 的具名 adapter（contract-baseline.md §3 归属表 R-*/E-*）。
//!
//! 依赖闭包恒轻：auto-val / auto-atom / auto-toolchain-core /
//! auto-project-model（+ log）——无 auto-lang/Parser/VM/UI/进程/网络。
//! 输入是 auto_val 层的 `Node`/`Obj`（AutoConfig 的 `{code, root, args}`
//! 中只有 `root`/`args` 被 `Pac::new` 消费，语言求值在 adapter 完成）。
//!
//! 公开签名冻结策略（contract-baseline.md §8.2）：
//! - 无效果字段的结构体（Target/Dir/Port/Scene/Group/...）**整体定义于本
//!   crate**，auto-man 旧路径 `pub use` 保持类型身份（805 re-export 模式）；
//! - 效果方法（transpile/scan/clean）在 auto-man 以扩展 trait 承接；
//! - 旧公开 2 参入口（`Target::from`/`get_root_dir`/`merge`）保留原直接
//!   FS 探针以冻结行为，纯入口为 `*_with_probe`（观测注入）；
//! - Pac（有效果字段）结构体留在 auto-man，纯方法逐项委托本 crate。

pub mod backend;
pub mod closure;
pub mod dir;
pub mod file_types;
pub mod group;
pub mod node_ext;
pub mod pac;
pub mod planning;
pub mod port;
pub mod scene;
pub mod selection;
pub mod sidecar;
pub mod target;

pub use backend::{
    default_frontend_with_legacy, frontend_types_with_legacy, output_dirs, BackendName,
    RenderSelection,
};
pub use closure::{
    admit_new_dep_libs, admit_transitive_deps, assemble_major_targets, build_resolved_links,
    check_all_resolved, dep_fetch_action, pinned_version, resolve_target_closure,
    resolve_target_deps, try_resolve_target_links, DepAction, GraphError,
};
pub use dir::Dir;
pub use file_types::{FileFilter, FileType};
pub use group::{dir_groups, File, Group};
pub use node_ext::NodeExt;
pub use pac::{
    all_incs, apps_of, backend_type_with_legacy, build_targets_of, deps_of, display_title_in,
    exe_path, get_build_names, has_backend_config, has_device_prop, is_backend_with_legacy,
    libs_of, list_port_names, pac_from_decl, pac_from_decl_with_probe, pac_is_ui, pac_is_workspace,
    pac_to_node, path_depth, relative_path, targets_map, PacDecl,
};
pub use planning::{
    ordered_build_steps, select_port_binding, select_rust_package, BuildOrderItem, BuildOrderStep,
    PortSelection, RustPackageSpec,
};
pub use port::{port_from_node, Port};
pub use scene::{is_ui, is_workspace, Scene};
pub use selection::{
    auto_recurse_into, check_incs_with, is_excluded_auto_config, scan_specific_srcs_with,
    scan_std_subdirs_aggregate, select_flat_entries, ScanEntry, STD_SOURCE_SUBDIRS,
};
pub use sidecar::{sidecar_decl_from_root, SidecarDecl};
pub use target::{
    get_root_dir, get_root_dir_with_probe, set_to_vec, Target, TargetKind, TargetOrigin,
};
