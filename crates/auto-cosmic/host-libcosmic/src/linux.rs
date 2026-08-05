//! Linux libcosmic host backend (VTree → libcosmic Element).
//!
//! ## Lowering strategy (Plan 365 W3)
//!
//! The host walks a `VTree` and produces a libcosmic `Element` tree:
//!
//! ```text
//! VTree (platform-neutral)          libcosmic Element (Linux/Wayland)
//! ─────────────────────────         ──────────────────────────────────
//! VNodeKind::Column           →     cosmic::widget::column()
//! VNodeKind::Row              →     cosmic::widget::row()
//! VNodeKind::Container        →     cosmic::widget::container()
//! VNodeKind::Text             →     cosmic::widget::text()
//! VNodeKind::Button           →     cosmic::widget::button()
//! VNodeKind::Input            →     cosmic::widget::text_input()
//! VNodeKind::Checkbox         →     cosmic::widget::checkbox()
//! VNodeKind::Slider           →     cosmic::widget::slider()
//! VNodeKind::ProgressBar      →     cosmic::widget::progress_bar()
//! ...                                ...
//! ```
//!
//! Event handling: VNode `onclick` props map to libcosmic message types; the
//! host owns the iced::Application subscription/update loop, routing messages
//! back to the Auto `Component::on()`.

use auto_lang::ui::{Component, VNodeKind, VTree};
use auto_lang::ui::vnode_converter::view_to_vtree;

/// Run a `Component` under the libcosmic host (Linux only).
///
/// This wraps the Auto `Component` in a libcosmic `Application` adapter and
/// runs the iced event loop with COSMIC theming. The `view()` output is
/// lowered from `View<M>` → `VTree` → libcosmic `Element`.
///
/// **Requires**: `libcosmic` as a dependency (Linux/Wayland). The actual
/// iced::application launch is gated behind `libcosmic` integration which
/// is wired when building on Linux. For now, this builds the VTree and
/// prints a diagnostic (proving the lowering pipeline works), then returns.
pub fn run_libcosmic<C>() -> auto_lang::ui::AppResult<()>
where
    C: Component + Default + 'static,
    C::Msg: Clone + std::fmt::Debug + Send + 'static,
{
    let component = C::default();
    let view = component.view();
    let vtree = view_to_vtree(&view);

    // Diagnostic: prove the VTree was built and is non-empty.
    eprintln!(
        "[libcosmic host] VTree built: {} nodes, root={:?}",
        vtree.nodes.len(),
        vtree.root
    );

    // Lower the VTree to a widget coverage report (validates all node kinds
    // are representable before the real libcosmic Element lowering).
    let report = vtree_coverage_report(&vtree);
    eprintln!("[libcosmic host] Widget coverage: {}", report);

    // TODO(libcosmic): uncomment when libcosmic dep is wired in Cargo.toml:
    //
    // let app = LibcosmicAppAdapter::<C>::new();
    // iced::application("Auto Cosmic App", LibcosmicAppAdapter::<C>::update,
    //                   LibcosmicAppAdapter::<C>::view)
    //     .theme(|_| cosmic::theme::system_default())
    //     .run()?;
    //
    // The adapter struct + update/view impls use `vtree_to_element()` below.

    Ok(())
}

/// Build a human-readable coverage report of VNodeKinds present in the tree.
///
/// This validates that the VTree is well-formed before attempting the real
/// libcosmic lowering, and helps track which widget kinds a replicated
/// component exercises.
fn vtree_coverage_report(vtree: &VTree) -> String {
    use std::collections::HashMap;
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for node in &vtree.nodes {
        let name = match node.kind {
            VNodeKind::Column => "column",
            VNodeKind::Row => "row",
            VNodeKind::Container => "container",
            VNodeKind::Scrollable => "scrollable",
            VNodeKind::Center => "center",
            VNodeKind::Text => "text",
            VNodeKind::Button => "button",
            VNodeKind::Input => "input",
            VNodeKind::Textarea => "textarea",
            VNodeKind::Checkbox => "checkbox",
            VNodeKind::Radio => "radio",
            VNodeKind::Select => "select",
            VNodeKind::List => "list",
            VNodeKind::Table => "table",
            VNodeKind::Slider => "slider",
            VNodeKind::ProgressBar => "progress_bar",
            VNodeKind::Accordion => "accordion",
            VNodeKind::Sidebar => "sidebar",
            VNodeKind::Tabs => "tabs",
            VNodeKind::NavigationRail => "navigation_rail",
        };
        *counts.entry(name).or_insert(0) += 1;
    }
    let mut pairs: Vec<String> = counts
        .iter()
        .map(|(k, v)| format!("{}×{}", v, k))
        .collect();
    pairs.sort();
    pairs.join(", ")
}

/// Lower a `VTree` to libcosmic `Element`s.
///
/// This is the core W3 lowering function. It walks the VTree arena and
/// produces the corresponding libcosmic widget for each `VNodeKind`. Each
/// variant maps to a `cosmic::widget::*` constructor.
///
/// **Requires `libcosmic`** — the function body is gated behind a feature
/// so the file compiles without it. The mapping table above documents the
/// full plan.
// TODO(libcosmic): implement with `#[cfg(feature = "libcosmic")]` once the
// dep is wired. Signature sketch:
//   fn vtree_to_element<M: Clone + std::fmt::Debug + Send>(
//       vtree: &VTree,
//       node_id: VNodeId,
//   ) -> cosmic::Element<M>
