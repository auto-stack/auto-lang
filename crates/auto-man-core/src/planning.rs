//! 构建规划纯规则（PLAN-809 T-05，R-05）：端口/后端绑定选择与三 Builder
//! 的目标规划半。效果执行（make_builder 构造、ninja/cargo/npm 进程、
//! 文件写入）留在 auto-man adapter。

use crate::port::Port;
use crate::target::{Target, TargetKind};
use auto_val::{AutoPath, AutoStr};

/// set_port 的规则产物（原 pac.rs `set_port` 前半的决策半）：
/// builder 名选择、build_at/build_location 计算、show_headers 联动。
#[derive(Debug, Clone, PartialEq)]
pub struct PortSelection {
    pub builder_name: AutoStr,
    pub build_at: AutoStr,
    pub build_location: AutoStr,
    pub show_headers: bool,
}

/// 端口绑定选择（逐字对应原 set_port 决策段）：
/// - at 去尾 `/` → build_at；build_location = at 路径；
/// - backend "rust" → cargo + build_location "rust"；
/// - 否则 builder 名 = port.builder；
/// - builder ghs|ninja → show_headers 联动开。
pub fn select_port_binding(port: &Port, backend: &AutoStr) -> PortSelection {
    let mut at = port.at.clone();
    if at.ends_with("/") {
        at = at[0..at.len() - 1].into();
    }
    let build_at = at.clone();
    let build_location = AutoPath::new(at.clone()).to_astr();
    let builder_name: AutoStr;
    let mut build_location = build_location;
    if backend.as_str() == "rust" {
        builder_name = "cargo".into();
        build_location = "rust".into();
    } else {
        builder_name = port.builder.clone();
    }
    let show_headers = port.builder == "ghs" || port.builder == "ninja";
    PortSelection {
        builder_name,
        build_at,
        build_location,
        show_headers,
    }
}

/// Builder 目标规划输入项（PLAN-809 R1-F03：ninja deps 先建/rename 去重
/// 序的纯决策——adapter 把 pac.targets 及其内嵌 deps 克隆映射为此抽象，
/// 执行半按返回的 rename 序回到具体句柄）。
#[derive(Debug, Clone, PartialEq)]
pub struct BuildOrderItem {
    /// 目标的 rename（去重键）。
    pub rename: AutoStr,
    /// 其内嵌依赖（t.deps）的 rename 序（声明序保留）。
    pub dep_renames: Vec<AutoStr>,
}

/// 规划步骤的可回查身份（R2-01：rename 不携带身份——共享 rename 时
/// 顶层与内嵌覆盖副本无法区分；步骤必须指回**首次访问的具体对象**）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildOrderStep {
    /// items 中的下标（adapter 的 pac.targets 下标）。
    pub item_index: usize,
    /// None = 顶层目标自身；Some(d) = items[item_index].deps[d] 内嵌副本。
    pub from_dep: Option<usize>,
}

/// deps 先建、rename 全局去重的执行序（原 ninja `build` 迭代规则）：
/// 逐 item 先其 deps 再自身，首个 rename 出现的**具体对象**胜出——
/// 返回带身份的步骤，adapter 按索引取回首次访问实例（含 merge_link
/// 覆盖态），不做任何按名字的全局猜测。
pub fn ordered_build_steps(items: &[BuildOrderItem]) -> Vec<BuildOrderStep> {
    let mut steps = Vec::new();
    let mut done: std::collections::HashSet<AutoStr> = std::collections::HashSet::new();
    for (i, item) in items.iter().enumerate() {
        for (d, dep) in item.dep_renames.iter().enumerate() {
            if !done.contains(dep) {
                steps.push(BuildOrderStep {
                    item_index: i,
                    from_dep: Some(d),
                });
                done.insert(dep.clone());
            }
        }
        if !done.contains(&item.rename) {
            steps.push(BuildOrderStep {
                item_index: i,
                from_dep: None,
            });
            done.insert(item.rename.clone());
        }
    }
    steps
}

/// Cargo 包选择规格（原 builder/cargo.rs `setup` 的选择决策半）：
/// 首个 rust App/Lib 目标；版本空/latest → 0.1.0；数字开头包名加
/// `app-` 前缀（Plan 531 P523-1）；空名回落 pac 名。
#[derive(Debug, Clone, PartialEq)]
pub struct RustPackageSpec {
    pub base_name: String,
    pub pkg_name: String,
    pub version: String,
    pub is_app: bool,
}

pub fn select_rust_package(targets: &[Target], pac_name: &AutoStr) -> Option<RustPackageSpec> {
    let target = targets.iter().find(|t| {
        t.lang.as_str() == "rust" && (t.kind == TargetKind::App || t.kind == TargetKind::Lib)
    })?;

    let version = target.version.as_str();
    let version = if version.is_empty() || version == "latest" {
        "0.1.0".to_string()
    } else {
        version.to_string()
    };

    let base_name = if target.name.as_str().is_empty() {
        pac_name.as_str().to_string()
    } else {
        target.name.as_str().to_string()
    };
    let pkg_name = if base_name
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        format!("app-{}", base_name)
    } else {
        base_name.clone()
    };

    Some(RustPackageSpec {
        base_name,
        pkg_name,
        version,
        is_app: target.kind == TargetKind::App,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_binding_rules() {
        // 普通 ninja 端口
        let mut port = Port::default();
        port.at = "build/".into();
        port.builder = "ninja".into();
        let sel = select_port_binding(&port, &"c".into());
        assert_eq!(sel.builder_name, "ninja");
        assert_eq!(sel.build_at, "build");
        assert_eq!(sel.build_location, "build");
        assert!(sel.show_headers);

        // rust 后端覆盖
        let sel = select_port_binding(&port, &"rust".into());
        assert_eq!(sel.builder_name, "cargo");
        assert_eq!(sel.build_location, "rust");

        // ghs 联动 show_headers；非 ninja/ghs 不开
        port.builder = "ghs".into();
        assert!(select_port_binding(&port, &"c".into()).show_headers);
        port.builder = "cmake".into();
        assert!(!select_port_binding(&port, &"c".into()).show_headers);
    }

    #[test]
    fn rust_package_selection() {
        let mut t = Target::new("myapp", TargetKind::App);
        t.lang = "rust".into();
        t.version = "latest".into();
        let spec = select_rust_package(&[t], &"pac".into()).unwrap();
        assert_eq!(spec.pkg_name, "myapp");
        assert_eq!(spec.version, "0.1.0");
        assert!(spec.is_app);

        // 空名回落 pac 名；数字前缀加 app-
        let mut t2 = Target::new("", TargetKind::Lib);
        t2.lang = "rust".into();
        t2.version = "1.2".into();
        let spec = select_rust_package(&[t2], &"015-notes".into()).unwrap();
        assert_eq!(spec.base_name, "015-notes");
        assert_eq!(spec.pkg_name, "app-015-notes");
        assert!(!spec.is_app);
        assert_eq!(spec.version, "1.2");

        // 无 rust 目标 → None
        let mut c = Target::new("c1", TargetKind::App);
        c.lang = "c".into();
        assert!(select_rust_package(&[c], &"p".into()).is_none());
    }
}
