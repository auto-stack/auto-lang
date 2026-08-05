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
//! VNodeKind::Text             →     cosmic::widget::text()
//! VNodeKind::Button           →     cosmic::widget::button()
//! VNodeKind::Input            →     cosmic::widget::text_input()
//! ...                                ...
//! ```
//!
//! Event handling: VNode `onclick` props map to libcosmic message types; the
//! host owns the iced::Application subscription/update loop, routing messages
//! back to the Auto `Component::on()`.
//!
//! This is ⭐⭐⭐⭐ difficulty (the plan's highest): it must cover all 18
//! `VNodeKind` variants, handle the full libcosmic theming system, and
//! integrate with cosmic-protocols (layer-shell for panels/applets). It is
//! built incrementally — each replicated COSMIC component drives the widget
//! coverage forward.

// NOTE: The actual implementation requires `libcosmic` as a dependency.
// Uncomment the deps in Cargo.toml and implement the lowering here.
// This file compiles (empty) on Linux so the crate structure is valid.

/// Run a `Component` under the libcosmic host (Linux only).
///
/// This is the libcosmic equivalent of `iced::run_app` — it owns the
/// iced::Application lifecycle, lowering the Auto `Component::view()` to
/// libcosmic Elements.
#[allow(unused_variables)]
pub fn run_libcosmic<C>()
where
    C: auto_lang::ui::Component + Default + 'static,
    C::Msg: Clone + std::fmt::Debug + Send + 'static,
{
    // TODO W3: implement once libcosmic dep is wired:
    // 1. Wrap C in a libcosmic Application adapter
    // 2. view(): C::view() → VTree → vtree_to_libcosmic_element()
    // 3. update(): route libcosmic messages → C::on()
    // 4. Run the iced::application with cosmic-themed defaults
    unimplemented!("Plan 365 W3: libcosmic host backend — requires libcosmic dependency");
}
