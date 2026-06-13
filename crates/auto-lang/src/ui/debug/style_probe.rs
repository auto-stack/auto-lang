//! Style probe — turn a Tailwind-style class string into `(property, value)` pairs.
//!
//! Used by the inspector's "Computed" tab. AutoUI styles are Tailwind-style
//! utility classes (e.g. `"w-full p-4 bg-blue-500"`) rather than a CSS cascade;
//! this module maps the parsed `StyleClass` IR back to human-readable CSS-like
//! key/value pairs (e.g. `[("width","100%"), ("padding","16px"), ...]`).

use crate::ui::style::{Color, SizeValue, Style, StyleClass};

/// Parse a class string into `(property, value)` key/value pairs for the
/// inspector Computed tab.
///
/// Unknown classes are silently skipped by `Style::parse`; an empty or wholly
/// unparseable input yields an empty `Vec`.
pub fn compute_style(class: &str) -> Vec<(String, String)> {
    let trimmed = class.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let Ok(style) = Style::parse(trimmed) else {
        return Vec::new();
    };
    style.classes.iter().flat_map(class_to_kv).collect()
}

/// Map a single `StyleClass` to zero or more `(property, value)` pairs.
///
/// Some variants expand to multiple pairs (e.g. `PaddingX` → both
/// `padding-left` and `padding-right`), hence `Vec` rather than `Option`.
fn class_to_kv(c: &StyleClass) -> Vec<(String, String)> {
    use StyleClass::*;
    match c {
        Padding(v) => vec![("padding".into(), format_size(v))],
        PaddingTop(v) => vec![("padding-top".into(), format_size(v))],
        PaddingBottom(v) => vec![("padding-bottom".into(), format_size(v))],
        PaddingLeft(v) => vec![("padding-left".into(), format_size(v))],
        PaddingRight(v) => vec![("padding-right".into(), format_size(v))],
        PaddingX(v) => vec![
            ("padding-left".into(), format_size(v)),
            ("padding-right".into(), format_size(v)),
        ],
        PaddingY(v) => vec![
            ("padding-top".into(), format_size(v)),
            ("padding-bottom".into(), format_size(v)),
        ],
        Margin(v) => vec![("margin".into(), format_size(v))],
        MarginTop(v) => vec![("margin-top".into(), format_size(v))],
        MarginBottom(v) => vec![("margin-bottom".into(), format_size(v))],
        MarginLeft(v) => vec![("margin-left".into(), format_size(v))],
        MarginRight(v) => vec![("margin-right".into(), format_size(v))],
        MarginX(v) => vec![
            ("margin-left".into(), format_size(v)),
            ("margin-right".into(), format_size(v)),
        ],
        MarginY(v) => vec![
            ("margin-top".into(), format_size(v)),
            ("margin-bottom".into(), format_size(v)),
        ],
        Width(v) => vec![("width".into(), format_size(v))],
        Height(v) => vec![("height".into(), format_size(v))],
        Gap(v) => vec![("gap".into(), format_size(v))],
        BackgroundColor(col) => vec![("background-color".into(), format_color(col))],
        TextColor(col) => vec![("color".into(), format_color(col))],
        BorderColor(col) => vec![("border-color".into(), format_color(col))],
        AccentColor(col) => vec![("accent-color".into(), format_color(col))],
        GradientFrom(col) => vec![("--gradient-from".into(), format_color(col))],
        GradientTo(col) => vec![("--gradient-to".into(), format_color(col))],
        BorderWidth(w) => vec![("border-width".into(), format!("{w}px"))],
        MaxWidth(px) => vec![("max-width".into(), format!("{px}px"))],
        MaxHeight(px) => vec![("max-height".into(), format!("{px}px"))],
        MinWidth(px) => vec![("min-width".into(), format!("{px}px"))],
        MinHeight(px) => vec![("min-height".into(), format!("{px}px"))],
        Opacity(o) => vec![("opacity".into(), format_percent_u8(*o))],
        Rotate(deg) => vec![("transform".into(), format!("rotate({deg}deg)"))],
        ZIndex(z) => vec![("z-index".into(), z.to_string())],
        // Layout / display variants without a numeric value: report the
        // canonical CSS keyword so the inspector still shows something useful.
        Flex => vec![("display".into(), "flex".into())],
        Flex1 => vec![("flex".into(), "1 1 0%".into())],
        FlexRow => vec![("flex-direction".into(), "row".into())],
        FlexCol => vec![("flex-direction".into(), "column".into())],
        ItemsCenter => vec![("align-items".into(), "center".into())],
        ItemsStart => vec![("align-items".into(), "flex-start".into())],
        ItemsEnd => vec![("align-items".into(), "flex-end".into())],
        JustifyCenter => vec![("justify-content".into(), "center".into())],
        JustifyBetween => vec![("justify-content".into(), "space-between".into())],
        JustifyStart => vec![("justify-content".into(), "flex-start".into())],
        JustifyEnd => vec![("justify-content".into(), "flex-end".into())],
        Hidden => vec![("display".into(), "none".into())],
        OverflowHidden => vec![("overflow".into(), "hidden".into())],
        OverflowAuto => vec![("overflow".into(), "auto".into())],
        OverflowVisible => vec![("overflow".into(), "visible".into())],
        OverflowScroll => vec![("overflow".into(), "scroll".into())],
        // Everything else is intentionally unmapped for now; return nothing so
        // the inspector only shows entries we know how to render.
        _ => Vec::new(),
    }
}

/// Render a `SizeValue` as a human-readable CSS value string.
///
/// Tailwind spacing units map to pixels at 4px per unit (`Fixed(1)` = `4px`).
fn format_size(v: &SizeValue) -> String {
    match v {
        SizeValue::Full => "100%".into(),
        SizeValue::Half => "50%".into(),
        SizeValue::Third => "33.333%".into(),
        SizeValue::TwoThirds => "66.666%".into(),
        SizeValue::Quarter => "25%".into(),
        SizeValue::ThreeQuarters => "75%".into(),
        SizeValue::Auto => "auto".into(),
        SizeValue::Fixed(units) => format!("{}px", *units as u32 * 4),
        SizeValue::Pixels(px) => format!("{px}px"),
    }
}

/// Render a `Color` as a CSS hex string (`#rrggbb`) using the Tailwind palette.
fn format_color(c: &Color) -> String {
    let (r, g, b) = c.to_rgb8();
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// Render a `0..=100` opacity integer as a CSS percentage string.
fn format_percent_u8(o: u8) -> String {
    // Cap at 100 to stay meaningful even if the parser ever relaxes its bound.
    let clamped = o.min(100);
    format!("{}%", clamped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn style_probe_parses_class_string() {
        let pairs = compute_style("w-full p-4 bg-blue-500");
        let m: HashMap<&str, &str> = pairs
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        assert_eq!(m.get("width"), Some(&"100%"));
        // p-4 → padding (4 Tailwind units = 16px)
        assert_eq!(m.get("padding"), Some(&"16px"));
        // bg-blue-500 → background-color #3b82f6 (Tailwind blue-500 RGB)
        assert_eq!(m.get("background-color"), Some(&"#3b82f6"));
    }

    #[test]
    fn style_probe_invalid_class_returns_empty() {
        // Style::parse silently skips unknown classes, so a wholly unknown
        // input yields no mapped property pairs.
        let r = compute_style("this-is-not-a-real-class-zzz");
        assert!(
            !r.iter().any(|(k, _)| k == "width"),
            "unknown class should not produce a width entry: {r:?}"
        );
        assert!(r.is_empty(), "fully unknown input should map to nothing: {r:?}");
    }

    #[test]
    fn style_probe_empty_input() {
        assert!(compute_style("").is_empty());
        assert!(compute_style("   ").is_empty());
    }

    #[test]
    fn style_probe_padding_x_expands() {
        // px-2 expands to both padding-left and padding-right (2 units = 8px).
        let pairs = compute_style("px-2");
        let m: HashMap<&str, &str> = pairs
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        assert_eq!(m.get("padding-left"), Some(&"8px"));
        assert_eq!(m.get("padding-right"), Some(&"8px"));
    }

    #[test]
    fn style_probe_text_color_maps_to_color() {
        let pairs = compute_style("text-red-500");
        let m: HashMap<&str, &str> = pairs
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        // Tailwind red-500 = #ef4444
        assert_eq!(m.get("color"), Some(&"#ef4444"));
    }

    #[test]
    fn style_probe_skips_unknown_keeps_known() {
        // Mix of known and unknown: unknown is dropped, known survives.
        let pairs = compute_style("w-full bogus-class gap-2");
        let m: HashMap<&str, &str> = pairs
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        assert_eq!(m.get("width"), Some(&"100%"));
        assert_eq!(m.get("gap"), Some(&"8px"));
        assert!(!m.contains_key("bogus"));
    }
}
