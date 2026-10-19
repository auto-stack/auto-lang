//! Target 链接闭包与有界图准入（PLAN-809 T-04）。
//!
//! `resolve_target_closure` 是 `Pac::resolve` 中段（原 pac.rs 的
//! `resolve_target_deps` 五函数族 + `while !done` 循环）的单源迁移：
//!
//! - 合法 DAG：迭代解析传递闭包（direct/transitive links → dep_names/
//!   all_links/is_resolved），行为与旧臂逐字段一致；
//! - **内部 target 环（自环/A↔B/更大环）**：旧臂 `while !done` 永不收敛
//!   （挂死，无 oracle 可冻结）；本函数按 PLAN-809 新增的失败合同返回
//!   [`GraphError::NoProgress`]——具名列出全部 unresolved target 与环
//!   候选链，**不开始后续 scan/build**；
//! - **外部 link（名字不在 targets_map）不阻塞**：与旧臂一致被跳过，
//!   不计 missing dependency，不进传递闭包；
//! - `assemble_major_targets`：闭包后的 major 合并段（deps 装配 +
//!   Device/App/Lib/Test 收集），HashMap 迭代序与旧臂同非确定（集合
//!   语义，V-04 oracle 用集合/偏序比较，不承诺字节稳定排序）。

use crate::target::{Target, TargetKind};
use auto_val::{AutoStr, Node};
use std::collections::{HashMap, HashSet};

/// 有界图准入错误（PLAN-809 §5.3 新增失败合同）。
#[derive(Debug, Clone, PartialEq)]
pub enum GraphError {
    /// 一轮迭代零进展且仍有未 resolved target——存在内部 target 环。
    NoProgress {
        /// 全部未 resolved 的 target 名（集合语义，无序承诺）。
        unresolved: Vec<AutoStr>,
        /// 环候选链（每条为名字序列，首尾相同）；DFS 找环失败时为空，
        /// 此时 unresolved+边集仍完整可诊断。
        cycle_candidates: Vec<Vec<AutoStr>>,
    },
}

impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphError::NoProgress {
                unresolved,
                cycle_candidates,
            } => {
                write!(f, "target dependency graph made no progress (internal cycle): unresolved targets [")?;
                let mut first = true;
                for name in unresolved {
                    if !first {
                        write!(f, ", ")?;
                    }
                    first = false;
                    write!(f, "{}", name)?;
                }
                write!(f, "]")?;
                if !cycle_candidates.is_empty() {
                    write!(f, "; cycle candidates:")?;
                    for chain in cycle_candidates {
                        write!(f, " {}", chain.join(" -> "))?;
                    }
                } else {
                    write!(
                        f,
                        "; no simple cycle isolated (inspect unresolved link edges)"
                    )?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for GraphError {}

/// 一轮闭包解析（原 `Pac::resolve_target_deps` 逐字迁移）。
/// 返回本轮新 resolved 的 target 数（无进展检测用）。
pub fn resolve_target_deps(targets_map: &mut HashMap<AutoStr, Target>) -> usize {
    let resolved_before = targets_map.values().filter(|t| t.is_resolved).count();

    let resolved_links = build_resolved_links(targets_map);
    apply_resolved_links(targets_map, resolved_links);

    let resolved_after = targets_map.values().filter(|t| t.is_resolved).count();
    resolved_after - resolved_before
}

/// 全部 resolved 判定（原 `Pac::check_all_resolved`）。
pub fn check_all_resolved(targets_map: &HashMap<AutoStr, Target>) -> bool {
    targets_map.values().all(|t| t.is_resolved)
}

/// 为 deps 全 resolved 的 target 构建传递闭包 links（原 `build_resolved_links`）。
pub fn build_resolved_links(targets_map: &HashMap<AutoStr, Target>) -> HashMap<AutoStr, Vec<Node>> {
    let mut resolved_links: HashMap<AutoStr, Vec<Node>> = HashMap::new();

    for target in targets_map.values() {
        if target.links.is_empty() {
            // Target with no dependencies is trivially resolved
            resolved_links.insert(target.name.clone(), vec![]);
            continue;
        }

        if let Some(links) = try_resolve_target_links(target, targets_map) {
            resolved_links.insert(target.name.clone(), links);
        }
    }

    resolved_links
}

/// 解析单个 target 的全部 links（原 `try_resolve_target_links`）：
/// 直接 links 起步，逐 dep 追加传递 links；dep 未 resolved 则本轮放弃；
/// 名字不在 map 的 link（外部/系统链接）跳过不阻塞。
pub fn try_resolve_target_links(
    target: &Target,
    targets_map: &HashMap<AutoStr, Target>,
) -> Option<Vec<Node>> {
    let mut all_deps_resolved = true;
    let mut links = Vec::new();

    links.extend(target.links.iter().cloned());

    for link in &target.links {
        let Some(dep) = targets_map.get(&link.id()) else {
            continue;
        };

        if !dep.is_resolved {
            all_deps_resolved = false;
            break;
        }

        links.extend(dep.links.iter().cloned());
    }

    if all_deps_resolved {
        Some(links)
    } else {
        None
    }
}

/// 应用 resolved links（原 `apply_resolved_links`）：置 is_resolved、写
/// dep_names 与 all_links。
pub fn apply_resolved_links(
    targets_map: &mut HashMap<AutoStr, Target>,
    resolved_links: HashMap<AutoStr, Vec<Node>>,
) {
    for (target_name, links) in resolved_links {
        let dep_names: Vec<AutoStr> = links
            .iter()
            .filter_map(|link| {
                let id = link.id();
                targets_map.get(&id).map(|dep| dep.name.clone())
            })
            .collect();

        if let Some(target) = targets_map.get_mut(&target_name) {
            target.is_resolved = true;
            target.dep_names = dep_names;
            target.all_links = links;
        }
    }
}

/// 有界图准入闭包（替代旧 `while !done`）：
/// 每轮迭代后若 resolved 计数零增长且仍有未 resolved，返回
/// [`GraphError::NoProgress`]（内部环；外部 link 不触发）。
pub fn resolve_target_closure(
    targets_map: &mut HashMap<AutoStr, Target>,
) -> Result<(), GraphError> {
    loop {
        if check_all_resolved(targets_map) {
            return Ok(());
        }
        let progressed = resolve_target_deps(targets_map);
        if check_all_resolved(targets_map) {
            return Ok(());
        }
        if progressed == 0 {
            let unresolved: Vec<AutoStr> = targets_map
                .values()
                .filter(|t| !t.is_resolved)
                .map(|t| t.name.clone())
                .collect();
            let cycle_candidates = find_cycles(targets_map, &unresolved);
            return Err(GraphError::NoProgress {
                unresolved,
                cycle_candidates,
            });
        }
    }
}

/// 在 unresolved 子图（边=links 中名字在 map 内的）找简单环，DFS 灰色
/// 标记法。返回每条环一条链（首尾同名）；找不到返回空 Vec（退化诊断
/// 仍成立）。递归深度 = unresolved 链长（受 target 数上界约束）。
fn find_cycles(
    targets_map: &HashMap<AutoStr, Target>,
    unresolved: &[AutoStr],
) -> Vec<Vec<AutoStr>> {
    let unresolved_set: HashSet<&AutoStr> = unresolved.iter().collect();
    let mut cycles = Vec::new();
    let mut visited: HashSet<AutoStr> = HashSet::new();
    let mut on_path: Vec<AutoStr> = Vec::new();

    fn dfs(
        node: &AutoStr,
        on_path: &mut Vec<AutoStr>,
        visited: &mut HashSet<AutoStr>,
        targets_map: &HashMap<AutoStr, Target>,
        unresolved_set: &HashSet<&AutoStr>,
        cycles: &mut Vec<Vec<AutoStr>>,
    ) {
        if visited.contains(node) {
            return;
        }
        visited.insert(node.clone());
        on_path.push(node.clone());
        if let Some(t) = targets_map.get(node) {
            for link in &t.links {
                let id = link.id();
                if !unresolved_set.contains(&id) {
                    continue; // 外部/已 resolved 边不参与环
                }
                if let Some(pos) = on_path.iter().position(|n| n == &id) {
                    let mut chain = on_path[pos..].to_vec();
                    chain.push(id.clone());
                    cycles.push(chain);
                } else if !visited.contains(&id) {
                    dfs(&id, on_path, visited, targets_map, unresolved_set, cycles);
                }
            }
        }
        on_path.pop();
    }

    for start in unresolved {
        dfs(
            start,
            &mut on_path,
            &mut visited,
            targets_map,
            &unresolved_set,
            &mut cycles,
        );
    }
    cycles
}

/// 闭包后的 major 合并段（原 `Pac::resolve` 中后段规则半）：
/// major targets（App/Lib）的 deps 装配（all_links → merge_link 后的
/// dep 克隆），并收集 Device/App/Lib/Test 为结果列表。
/// HashMap 迭代序非确定（与旧臂一致；集合语义 oracle）。
pub fn assemble_major_targets(targets_map: &mut HashMap<AutoStr, Target>) -> Vec<Target> {
    let mut major_targets = Vec::new();

    let major_target_names: Vec<AutoStr> = targets_map
        .values()
        .filter(|t| t.is_major())
        .map(|t| t.name.clone())
        .collect();

    for n in major_target_names {
        let mut deps = Vec::new();
        let t = targets_map.get(&n).unwrap();
        for l in &t.all_links {
            let Some(dep) = targets_map.get(&l.id()) else {
                continue;
            };
            let mut merge = dep.clone();
            merge.merge_link(l);
            deps.push(merge);
        }
        if let Some(t) = targets_map.get_mut(&n) {
            t.deps = deps;
        }
    }

    targets_map
        .values()
        .filter(|t| {
            t.kind == TargetKind::Device
                || t.kind == TargetKind::App
                || t.kind == TargetKind::Lib
                || t.kind == TargetKind::Test
        })
        .for_each(|t| major_targets.push(t.clone()));

    major_targets
}

// ========== 依赖下载决策（PLAN-809 R1-F03 / contract R-07） ==========
// 纯决策单源：按目标决定动作、按已知名集合决定并入；Git/索引/FS 执行
// 与递归求值留在 auto-man adapter。

/// 单个 Dep/Device 目标的下载动作决策。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepAction {
    /// 已物化（exists），跳过。
    Skip,
    /// 需要获取（按 origin 分派执行）。
    Fetch,
}

/// 决策：仅 Dep/Device 参与下载；exists 观测注入（adapter 传真实 FS）。
pub fn dep_fetch_action(target: &Target, exists: bool) -> Option<DepAction> {
    match target.kind {
        TargetKind::Dep | TargetKind::Device => {
            if exists {
                Some(DepAction::Skip)
            } else {
                Some(DepAction::Fetch)
            }
        }
        _ => None,
    }
}

/// 版本 pin 决策：父传播 version=="latest" 时目标置 latest。
pub fn pinned_version(target_version: &AutoStr, parent_version: &AutoStr) -> AutoStr {
    if parent_version == "latest" {
        "latest".into()
    } else {
        target_version.clone()
    }
}

/// dep-pac 的其余 lib 目标并入 new_deps 的决策（原 download_deps 段）：
/// 排除与当前 dep 同名者、已知名（target_names）不重入；dirs 的
/// append_root 以 dep 根相对存在性观测注入。
pub fn admit_new_dep_libs(
    libs: &[Target],
    known: &std::collections::HashSet<AutoStr>,
    current_dep_name: &AutoStr,
    dep_root_at: &AutoStr,
    dir_exists: impl Fn(&str) -> bool,
) -> Vec<Target> {
    let mut admitted = Vec::new();
    for lib in libs {
        if lib.name == *current_dep_name {
            continue;
        }
        if known.contains(&lib.name) {
            continue;
        }
        let mut lib = lib.clone();
        for (_, v) in lib.dirs.iter_mut() {
            let path = format!("{}/{}", dep_root_at, v.at);
            if dir_exists(&path) {
                v.append_root(dep_root_at.clone());
            }
        }
        admitted.push(lib);
    }
    admitted
}

/// 递归下载返回的传递 deps 并入决策：已知名不重入。
pub fn admit_transitive_deps<'a>(
    deps: impl IntoIterator<Item = &'a Target>,
    known: &std::collections::HashSet<AutoStr>,
) -> Vec<Target> {
    let mut admitted = Vec::new();
    for dep in deps {
        if known.contains(&dep.name) {
            continue;
        }
        admitted.push(dep.clone());
    }
    admitted
}

#[cfg(test)]
mod tests {
    use super::*;

    /// R1-F03：下载动作/版本 pin/并入决策的纯规则 oracle。
    #[test]
    fn download_decisions_rules() {
        use super::*;
        use std::collections::HashSet;

        let mut dep = Target::new("d", TargetKind::Dep);
        dep.version = "1.0".into();
        let app = Target::new("a", TargetKind::App);

        // 动作：仅 Dep/Device；exists 跳过
        assert_eq!(dep_fetch_action(&dep, true), Some(DepAction::Skip));
        assert_eq!(dep_fetch_action(&dep, false), Some(DepAction::Fetch));
        assert_eq!(dep_fetch_action(&app, false), None);

        // 版本 pin：父 latest 传播
        assert_eq!(pinned_version(&"1.0".into(), &"latest".into()), "latest");
        assert_eq!(pinned_version(&"1.0".into(), &"0.9".into()), "1.0");

        // lib 并入：同名排除、已知不重入、append_root 观测注入
        let mut lib = Target::new("libx", TargetKind::Lib);
        lib.dirs.insert(
            "src".into(),
            crate::dir::Dir::from_str("src".into(), "libs/libx".into(), TargetKind::Lib),
        );
        let same = Target::new("d", TargetKind::Lib);
        let known: HashSet<AutoStr> = ["known".into()].into_iter().collect();
        let admitted: Vec<Target> = admit_new_dep_libs(
            &[lib.clone(), same],
            &known,
            &"d".into(),
            &"deps/d".into(),
            |p| p == "deps/d/libs/libx/src",
        );
        assert_eq!(admitted.len(), 1, "同名 dep 排除");
        assert_eq!(admitted[0].name, "libx");
        let src_dir = admitted[0].dirs.get(&AutoStr::from("src")).unwrap();
        assert!(
            src_dir.at.starts_with("deps/d/"),
            "append_root 按观测应用：{}",
            src_dir.at
        );
        // 观测为假 → 不 append
        let admitted2 =
            admit_new_dep_libs(&[lib], &known, &"d".into(), &"deps/d".into(), |_| false);
        let d2 = admitted2[0].dirs.get(&AutoStr::from("src")).unwrap();
        assert_eq!(d2.at, "libs/libx/src");

        // 传递 dep 并入：已知不重入
        let t1 = Target::new("t1", TargetKind::Dep);
        let mut t2 = Target::new("known", TargetKind::Dep);
        t2.version = "9".into();
        let adm = admit_transitive_deps([&t1, &t2], &known);
        assert_eq!(adm.len(), 1);
        assert_eq!(adm[0].name, "t1");
    }

    use crate::target::{Target, TargetKind};
    use auto_val::{AutoStr, Node, Value};

    fn link_node(id: &str) -> Node {
        let mut n = Node::new("link");
        n.args.add_pos(Value::Str(id.into()));
        n
    }

    fn target(name: &str, kind: TargetKind, links: &[&str]) -> Target {
        let mut t = Target::new(name, kind);
        for l in links {
            t.links.push(link_node(l));
        }
        t
    }

    fn map_of(targets: Vec<Target>) -> HashMap<AutoStr, Target> {
        crate::pac::targets_map(targets)
    }

    /// diamond：app→liba/libb→libc——闭包后 app 的 all_links 含 liba/libb/libc。
    #[test]
    fn diamond_closure() {
        let mut map = map_of(vec![
            target("app", TargetKind::App, &["liba", "libb"]),
            target("liba", TargetKind::Lib, &["libc"]),
            target("libb", TargetKind::Lib, &["libc"]),
            target("libc", TargetKind::Lib, &[]),
        ]);
        resolve_target_closure(&mut map).expect("DAG resolves");
        let app = &map[&AutoStr::from("app")];
        let ids: HashSet<String> = app.all_links.iter().map(|l| l.id().to_string()).collect();
        assert!(ids.contains("liba") && ids.contains("libb") && ids.contains("libc"));
        let dep_names: HashSet<String> = app.dep_names.iter().map(|d| d.to_string()).collect();
        assert!(dep_names.contains("liba") && dep_names.contains("libc"));
        assert!(app.is_resolved);
    }

    /// 重复 link：app 两连 liba——all_links 保留两份（旧臂不去重）。
    #[test]
    fn duplicate_links_preserved() {
        let mut map = map_of(vec![
            target("app", TargetKind::App, &["liba", "liba"]),
            target("liba", TargetKind::Lib, &[]),
        ]);
        resolve_target_closure(&mut map).unwrap();
        let app = &map[&AutoStr::from("app")];
        assert_eq!(app.all_links.len(), 2, "重复 link 不去重（旧臂语义）");
    }

    /// 断开图（外部/系统 link 名不在 map）：不阻塞、不报错；直接 link
    /// 保留在 all_links（旧臂起步即 extend），但不贡献传递边、不入
    /// dep_names。
    #[test]
    fn external_link_allowed() {
        let mut map = map_of(vec![
            target("app", TargetKind::App, &["liba", "m", "kernel32"]),
            target("liba", TargetKind::Lib, &[]),
        ]);
        resolve_target_closure(&mut map).expect("外部 link 不构成 missing dependency");
        let app = &map[&AutoStr::from("app")];
        let ids: HashSet<String> = app.all_links.iter().map(|l| l.id().to_string()).collect();
        assert!(ids.contains("liba"), "内部闭包");
        assert!(
            ids.contains("m") && ids.contains("kernel32"),
            "外部直接 link 保留在 all_links（旧臂语义）"
        );
        let dep_names: HashSet<String> = app.dep_names.iter().map(|d| d.to_string()).collect();
        assert!(
            !dep_names.contains("m") && !dep_names.contains("kernel32"),
            "外部名不入 dep_names"
        );
        assert!(app.is_resolved, "外部 link 不阻塞解析");
    }

    /// 自环：app→app——旧臂挂死；新合同 NoProgress 具名。
    #[test]
    fn self_loop_named_error() {
        let mut map = map_of(vec![
            target("app", TargetKind::App, &["app"]),
            target("lib", TargetKind::Lib, &[]),
        ]);
        let err = resolve_target_closure(&mut map).unwrap_err();
        match &err {
            GraphError::NoProgress {
                unresolved,
                cycle_candidates,
            } => {
                assert!(unresolved.contains(&AutoStr::from("app")));
                assert!(!unresolved.contains(&AutoStr::from("lib")));
                assert!(
                    cycle_candidates
                        .iter()
                        .any(|c| c.first() == Some(&AutoStr::from("app"))),
                    "自环候选链具名: {:?}",
                    cycle_candidates
                );
            }
        }
        let msg = err.to_string();
        assert!(msg.contains("no progress"), "错误文本可诊断: {}", msg);
        assert!(msg.contains("app"));
    }

    /// 双环 A↔B：环链 a→b→a 具名。
    #[test]
    fn two_node_cycle_named_error() {
        let mut map = map_of(vec![
            target("a", TargetKind::App, &["b"]),
            target("b", TargetKind::Lib, &["a"]),
        ]);
        let err = resolve_target_closure(&mut map).unwrap_err();
        match &err {
            GraphError::NoProgress {
                unresolved,
                cycle_candidates,
            } => {
                let set: HashSet<&AutoStr> = unresolved.iter().collect();
                assert!(set.contains(&AutoStr::from("a")) && set.contains(&AutoStr::from("b")));
                assert!(
                    cycle_candidates
                        .iter()
                        .any(|c| c.len() == 3 && c.first() == c.last()),
                    "环链首尾闭合: {:?}",
                    cycle_candidates
                );
            }
        }
    }

    /// 无进展混合图：resolved 段正常 + 环段具名（lib 不被拖累）。
    #[test]
    fn mixed_graph_partial_progress_then_named_error() {
        let mut map = map_of(vec![
            target("app", TargetKind::App, &["c1", "lib"]),
            target("c1", TargetKind::Lib, &["c2"]),
            target("c2", TargetKind::Lib, &["c1"]),
            target("lib", TargetKind::Lib, &[]),
        ]);
        let err = resolve_target_closure(&mut map).unwrap_err();
        match &err {
            GraphError::NoProgress { unresolved, .. } => {
                let set: HashSet<&AutoStr> = unresolved.iter().collect();
                assert!(set.contains(&AutoStr::from("app")));
                assert!(set.contains(&AutoStr::from("c1")));
                assert!(set.contains(&AutoStr::from("c2")));
                assert!(!set.contains(&AutoStr::from("lib")), "无环成员不被拖累");
            }
        }
    }

    /// assemble_major_targets：major deps 装配 + Device/App/Lib/Test 收集
    /// + Dep/Bag 不入结果（集合语义）。
    #[test]
    fn assemble_major_semantics() {
        let mut map = map_of(vec![
            target("app", TargetKind::App, &["lib", "dep1"]),
            target("lib", TargetKind::Lib, &[]),
            target("dep1", TargetKind::Dep, &[]),
            target("bag1", TargetKind::Bag, &[]),
        ]);
        resolve_target_closure(&mut map).unwrap();
        let majors = assemble_major_targets(&mut map);
        let names: HashSet<String> = majors.iter().map(|t| t.name.to_string()).collect();
        assert!(names.contains("app") && names.contains("lib"));
        assert!(
            !names.contains("dep1") && !names.contains("bag1"),
            "Dep/Bag 不入结果"
        );
        // app.deps 装配了 lib 与 dep1（两者都在 app 直接 links 且在 map 内；
        // Dep kind 不阻断 links→deps 装配，只是自身不入结果列表）
        let app = majors.iter().find(|t| t.name == "app").unwrap();
        assert!(app.deps.iter().any(|d| d.name == "lib"));
        assert!(app.deps.iter().any(|d| d.name == "dep1"));
    }

    /// 字段变更后二次 plan 可见新值（无双状态/无缓存；V-04）。
    #[test]
    fn replan_sees_mutated_fields() {
        let mut map = map_of(vec![
            target("app", TargetKind::App, &["liba"]),
            target("liba", TargetKind::Lib, &[]),
        ]);
        resolve_target_closure(&mut map).unwrap();
        // 变更：app 改连 libb（重建 map 模拟门面字段后二次 plan）
        let mut app = map.remove(&AutoStr::from("app")).unwrap();
        app.links = vec![link_node("libb")];
        app.is_resolved = false;
        app.dep_names.clear();
        app.all_links.clear();
        let mut map2 = map_of(vec![app, target("libb", TargetKind::Lib, &[])]);
        resolve_target_closure(&mut map2).unwrap();
        let app2 = &map2[&AutoStr::from("app")];
        let ids: HashSet<String> = app2.all_links.iter().map(|l| l.id().to_string()).collect();
        assert!(
            ids.contains("libb") && !ids.contains("liba"),
            "新值即时可见"
        );
    }

    /// 独立手算 oracle：三层链 app→l1→l2 的 all_links 恰为 {l1,l2}，
    /// dep_names 恰为 {l1,l2}（有序无关）。
    #[test]
    fn hand_computed_chain_oracle() {
        let mut map = map_of(vec![
            target("app", TargetKind::App, &["l1"]),
            target("l1", TargetKind::Lib, &["l2"]),
            target("l2", TargetKind::Lib, &[]),
        ]);
        resolve_target_closure(&mut map).unwrap();
        for name in ["app", "l1", "l2"] {
            assert!(map[&AutoStr::from(name)].is_resolved);
        }
        let app = &map[&AutoStr::from("app")];
        let ids: HashSet<String> = app.all_links.iter().map(|l| l.id().to_string()).collect();
        assert_eq!(ids, ["l1", "l2"].into_iter().map(String::from).collect());
    }
}
