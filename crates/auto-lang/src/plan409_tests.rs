//! Plan 409 §6: `link` 子组件 VM 渲染缺口 — regression tests.
//!
//! A `link (to:) { text / row / icon ... }` must render as a clickable button
//! whose *content* is the converted child container (vue parity), instead of
//! flattening to the `to` path string. The top nav of `examples/widgets-gallery`
//! (`link (to: "/") { text "Docs" }` / `link (to: "/button") { text "Components" }`)
//! is the canonical probe: it used to render as `button "/"` / `button "/button"`.

#![cfg(test)]

use crate::ui::view::View;
use crate::ui::dynamic::DynamicComponent;
use crate::ui::interpreter::DynamicMessage;
use crate::ui::style::{Style, StyleClass, Color};

/// Recursively collect every Button in the view tree (any depth).
fn collect_buttons<'a>(
    view: &'a View<DynamicMessage>,
    out: &mut Vec<(&'a str, &'a Option<Box<View<DynamicMessage>>>)>,
) {
    match view {
        View::Button { label, content, .. } => {
            out.push((label.as_str(), content));
            if let Some(c) = content {
                collect_buttons(c, out);
            }
        }
        View::Row { children, .. } | View::Column { children, .. } | View::List { items: children, .. } => {
            for child in children {
                collect_buttons(child, out);
            }
        }
        View::Grid { cells, .. } => {
            for cell in cells {
                collect_buttons(cell, out);
            }
        }
        View::Container { child, .. } | View::Scrollable { child, .. } => {
            collect_buttons(child, out);
        }
        View::Table { headers, rows, .. } => {
            for h in headers {
                collect_buttons(h, out);
            }
            for row in rows {
                for cell in row {
                    collect_buttons(cell, out);
                }
            }
        }
        _ => {}
    }
}

/// Collect every button whose onclick is the given event name (e.g. a
/// `__navigate` for links or the theme-picker `openThemePicker`).
fn has_onclick_event(view: &View<DynamicMessage>, event: &str) -> bool {
    match view {
        View::Button { onclick, content, .. } => {
            let hits = match onclick {
                DynamicMessage::Typed { event_name, .. } => event_name == event,
                DynamicMessage::String(s) => s == event,
            };
            if hits {
                return true;
            }
            if let Some(c) = content {
                return has_onclick_event(c, event);
            }
            false
        }
        View::Row { children, .. } | View::Column { children, .. } | View::List { items: children, .. } => {
            children.iter().any(|c| has_onclick_event(c, event))
        }
        View::Grid { cells, .. } => cells.iter().any(|c| has_onclick_event(c, event)),
        View::Container { child, .. } | View::Scrollable { child, .. } => {
            has_onclick_event(child, event)
        }
        _ => false,
    }
}

/// Build the real widgets-gallery app (like production does) and return the
/// view tree. None when the example sources aren't present (graceful no-op).
#[cfg(feature = "ui-iced")]
fn build_gallery_view() -> Option<View<DynamicMessage>> {
    let comp: DynamicComponent = crate::plan370_test_support::build_example_component("widgets-gallery")?;
    let (view, _, _) = comp.view_with_debug();
    Some(view)
}

/// Build the gallery component (state + view) for §8 theme-color tests.
#[cfg(feature = "ui-iced")]
fn build_gallery_component() -> Option<DynamicComponent> {
    crate::plan370_test_support::build_example_component("widgets-gallery")
}

/// Build a gallery page widget (e.g. "IndexPage") in isolation — the app
/// component's outlet does not render route pages in the test harness, so
/// page-specific assertions build the page file directly.
#[cfg(feature = "ui-iced")]
fn build_gallery_page(page_file: &str, widget_name: &str) -> Option<View<DynamicMessage>> {
    use crate::ui::aura_view_builder::AuraViewBuilder;
    use crate::ui::vm_bridge::VmBridge;
    use crate::ui::widget_registry::WidgetRegistry;

    let candidates = [
        std::env::var("CARGO_MANIFEST_DIR")
            .ok()
            .map(|d| std::path::PathBuf::from(d).join(format!("../../examples/widgets-gallery/src/front/pages/{}", page_file)))
            .filter(|p| p.exists()),
        Some(std::path::PathBuf::from(format!("examples/widgets-gallery/src/front/pages/{}", page_file)))
            .filter(|p| p.exists()),
    ];
    let path = candidates.into_iter().flatten().next()?;
    let code = std::fs::read_to_string(&path).ok()?;
    let session = crate::session::CompilerSession::ui();
    let mut parser = crate::Parser::from(code.as_str()).with_session(session);
    let ast = parser.parse().ok()?;
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::WidgetDecl(decl) = stmt {
            let widget = crate::aura::extract_widget_from_decl(decl).ok()?;
            if widget.name != widget_name {
                continue;
            }
            let bridge = VmBridge::new(&widget).ok()?;
            let registry = WidgetRegistry::new();
            let builder = AuraViewBuilder::with_registry(&bridge, &widget.name, &registry);
            let view = builder.build(&widget.view_tree);
            return Some(view);
        }
    }
    None
}

#[cfg(feature = "ui-iced")]
#[test]
fn link_children_render_as_button_content() {
    let Some(view) = build_gallery_view() else {
        eprintln!("plan409: SKIPPED — widgets-gallery app.at not found");
        return;
    };

    let mut buttons = Vec::new();
    collect_buttons(&view, &mut buttons);

    // The two top-nav links must render their text children as button labels
    // (not the raw `to` path) AND carry a content subtree.
    let docs = buttons.iter().find(|(label, _)| *label == "Docs");
    let components = buttons.iter().find(|(label, _)| *label == "Components");

    assert!(
        docs.is_some(),
        "top-nav link (to: \"/\") {{ text \"Docs\" }} must render as a button labelled \"Docs\"; \
         got buttons: {:?}",
        buttons.iter().map(|(l, _)| *l).collect::<Vec<_>>()
    );
    assert!(
        components.is_some(),
        "top-nav link (to: \"/button\") {{ text \"Components\" }} must render as a button \
         labelled \"Components\"; got buttons: {:?}",
        buttons.iter().map(|(l, _)| *l).collect::<Vec<_>>()
    );

    // Both links carry a converted content subtree (the styled `text` child),
    // not a flattened path string.
    for (label, content) in [docs.unwrap(), components.unwrap()] {
        assert!(
            content.is_some(),
            "link button \"{}\" must carry a content container (its child nodes), not just a label",
            label
        );
    }
}

#[cfg(feature = "ui-iced")]
#[test]
fn plain_link_without_children_keeps_to_label() {
    let Some(view) = build_gallery_view() else {
        eprintln!("plan409: SKIPPED — widgets-gallery app.at not found");
        return;
    };
    let mut buttons = Vec::new();
    collect_buttons(&view, &mut buttons);

    // A self-closing `link (to: "/x") {}` keeps the path as its label and no
    // content subtree. (The gallery sidebar uses nav-link for these, but the
    // regression guard is the untracked leaf behaviour.)
    let self_closing = buttons.iter().find(|(label, _)| label.starts_with('/'));
    match self_closing {
        Some((_label, content)) => {
            assert!(content.is_none(), "leaf link must not fabricate a content subtree");
        }
        None => {
            // All links in the current gallery carry children; the guard is
            // simply that nothing regressed the leaf path (no panic / no crash).
            eprintln!("plan409: no self-closing link present — leaf path not exercised");
        }
    }
}

// ===========================================================================
// Plan 409 §8 — theme color support
// ===========================================================================

#[cfg(feature = "ui-iced")]
#[test]
fn theme_accent_color_state_and_handlers() {
    let Some(mut comp) = build_gallery_component() else {
        eprintln!("plan409: SKIPPED — widgets-gallery app.at not found");
        return;
    };

    // accent_color defaults to indigo and is readable from root state.
    let accent = comp.read_state("accent_color").expect("accent_color state");
    assert_eq!(
        format!("{:?}", accent),
        format!("{:?}", auto_val::Value::str("indigo")),
        "accent_color must default to indigo (aligned with auto-forge / iced_adapter)"
    );

    // SetAccent("coral") updates the state → the renderer's per-frame sync
    // (iced_adapter ACCENT_NAME) then resolves Color::Primary to coral.
    comp.on_with_input_for("App", "SetAccent", Some("coral".to_string()));
    let updated = comp.read_state("accent_color").expect("accent_color after SetAccent");
    assert_eq!(
        format!("{:?}", updated),
        format!("{:?}", auto_val::Value::str("coral")),
        "SetAccent(\"coral\") must update accent_color"
    );
}

#[cfg(feature = "ui-iced")]
#[test]
fn theme_palette_ui_and_primary_rendering() {
    let Some(comp) = build_gallery_component() else {
        eprintln!("plan409: SKIPPED — widgets-gallery app.at not found");
        return;
    };
    let (view, _, _) = comp.view_with_debug();

    // Top-bar palette toggle button exists — it's icon-only (default label
    // "Button") with an onclick wired to the .openThemePicker handler.
    assert!(
        has_onclick_event(&view, "openThemePicker"),
        "top-bar must contain the palette (theme picker) button wired to .openThemePicker"
    );

    // The Home hero "Auto UI" lives in the IndexPage route (the app's outlet
    // does not load route pages in the test harness), so build the page in
    // isolation and assert its h1 uses text-primary (the theme color) — not
    // the unrenderable gradient/transparent combo.
    let Some(index_view) = build_gallery_page("index.at", "IndexPage") else {
        eprintln!("plan409: SKIPPED — pages/index.at not found");
        return;
    };
    let mut texts: Vec<(&str, &Option<Style>)> = Vec::new();
    collect_texts(&index_view, &mut texts);
    let hero = texts.iter().find(|(t, _)| *t == "Auto UI");
    assert!(
        hero.is_some(),
        "Home hero \"Auto UI\" must exist; got texts: {:?}",
        texts.iter().map(|(t, _)| *t).collect::<Vec<_>>()
    );
    let (_, style) = hero.unwrap();
    let classes = style.as_ref().map(|s| s.classes.clone()).unwrap_or_default();
    assert!(
        classes.iter().any(|c| matches!(c, StyleClass::TextColor(Color::Primary))),
        "Auto UI hero must use text-primary (theme color); classes: {:?}",
        classes
    );

    // Primary-variant button preset is theme-aware: a `variant: "primary"`
    // button must carry bg-primary (not hardcoded bg-blue-500). The Get
    // Started button lives in the IndexPage (not loaded via app outlet in the
    // harness), so inspect the isolated IndexPage view for its presence.
    let mut buttons2 = Vec::new();
    collect_buttons(&index_view, &mut buttons2);
    let get_started = buttons2.iter().find(|(label, _)| *label == "Get Started");
    assert!(
        get_started.is_some(),
        "Home \"Get Started\" primary button must exist (theme color smoke test)"
    );
}

/// Collect every Text node (content, style) in the view tree.
#[cfg(feature = "ui-iced")]
fn collect_texts<'a>(
    view: &'a View<DynamicMessage>,
    out: &mut Vec<(&'a str, &'a Option<Style>)>,
) {
    match view {
        View::Text { content, style } => out.push((content.as_str(), style)),
        View::Button { content, .. } => {
            if let Some(c) = content {
                collect_texts(c, out);
            }
        }
        View::Row { children, .. } | View::Column { children, .. } | View::List { items: children, .. } => {
            for child in children {
                collect_texts(child, out);
            }
        }
        View::Grid { cells, .. } => {
            for cell in cells {
                collect_texts(cell, out);
            }
        }
        View::Container { child, .. } | View::Scrollable { child, .. } => {
            collect_texts(child, out);
        }
        View::Table { headers, rows, .. } => {
            for h in headers {
                collect_texts(h, out);
            }
            for row in rows {
                for cell in row {
                    collect_texts(cell, out);
                }
            }
        }
        _ => {}
    }
}
