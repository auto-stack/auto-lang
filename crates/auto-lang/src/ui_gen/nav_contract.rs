//! Plan 482: nav-item / nav-group class-token contract — the single source of
//! truth shared by both backends.
//!
//! - VM side: `ui/aura_view_builder` builds `View::Button`/container styles from
//!   these constants (tokens must be parseable by `StyleClass::parse_single`,
//!   i.e. survive the iced adapter — integer spacing steps or `[…]` arbitrary
//!   values; no `uppercase`/`tracking-*` web-only visual effects).
//! - Vue side: `auto-man/assets/shadcn-ui/nav/NavItem.vue` / `NavGroup.vue`
//!   embed the same strings (mirrored; a unit test compares them against this
//!   file so the two ends cannot drift).
//!
//! Marker classes (`nav-item`, `nav-group-toggle`, `nav-search`, …) are inert
//! test/style hooks (web CSS + Playwright/e2e selectors); the VM parser skips
//! them and that is by design.

/// Base layout for the default single-line item (h-9 = 36px both ends).
pub const ITEM_BASE_MD: &str = "nav-item flex w-full items-center gap-2 rounded-md px-3 h-9 text-sm text-left text-foreground select-none cursor-pointer transition-colors";
/// Two-line item (label + desc): auto height via bracket padding (10px).
pub const ITEM_BASE_LG: &str = "nav-item flex w-full items-start gap-3 rounded-md px-3 py-[10px] text-sm text-left text-foreground select-none cursor-pointer transition-colors";
/// Compact single-line item (h-7 = 28px).
pub const ITEM_BASE_SM: &str = "nav-item flex w-full items-center gap-2 rounded-md px-2 h-7 text-xs text-left text-foreground select-none cursor-pointer transition-colors";
/// Hover feedback — attached ONLY when the item is not active (build-time
/// either/or, mirroring NavItem.vue, so hover can never override the selected
/// background regardless of Tailwind stylesheet order).
pub const ITEM_HOVER: &str = "hover:bg-accent hover:text-accent-foreground";
/// Selected state — primary-tinted block (user requirement: 选中态符合主色调).
pub const ITEM_ACTIVE: &str = "bg-primary/10 text-primary font-medium";
/// Disabled (web); the VM goes through the Button disabled gray-text path.
pub const ITEM_DISABLED: &str = "opacity-60 cursor-default";
/// Right-side badge pill.
pub const BADGE_PILL: &str = "ml-auto inline-flex items-center justify-center rounded-full bg-primary/15 text-primary px-2 py-[2px] text-xs font-medium shrink-0";
/// Icon box sizes (md/sm = 16px, lg = 20px).
pub const ICON_MD: &str = "h-4 w-4 shrink-0";
pub const ICON_LG: &str = "h-5 w-5 shrink-0";
/// Text column fill — attached when the item has a badge or desc so the badge
/// lands on the row's trailing edge on both ends (iced Fill vs CSS flex-1).
pub const TEXTS_FILL: &str = "flex-1 min-w-0";
/// Secondary line text (desc) color/size.
pub const TEXT_DESC: &str = "text-xs text-muted-foreground";
/// Non-collapsible group label header.
pub const GROUP_LABEL: &str = "nav-group-label px-3 pt-2 pb-1 text-xs font-medium text-muted-foreground";
/// Collapsible group header (a button).
pub const GROUP_TOGGLE: &str = "nav-group-toggle flex w-full items-center gap-2 px-3 py-2 rounded-md text-sm font-medium text-foreground cursor-pointer select-none";
/// Collapsible header hover feedback.
pub const GROUP_TOGGLE_HOVER: &str = "hover:bg-accent";
/// Group member column; `indent` appends " pl-3".
pub const GROUP_CONTENT: &str = "nav-group-content flex flex-col gap-1";
pub const GROUP_CONTENT_INDENT: &str = "pl-3";
/// Integrated search row (nav search: true) wrapper.
pub const SEARCH_ROW: &str = "nav-search flex items-center gap-2 mx-3 mb-2 px-3 h-9 rounded-md border border-input bg-muted/50 text-sm shrink-0";
/// Search input inside the row.
pub const SEARCH_INPUT: &str = "w-full bg-transparent border-0 outline-none placeholder:text-muted-foreground text-foreground text-sm";

/// Token classes that are web-only enhancements (VM ignores them with no visual
/// difference — iced has its own cursor/hover semantics and no text selection).
const WEB_ONLY_TOKENS: &[&str] = &[
    "select-none",
    "cursor-pointer",
    "cursor-default",
    "transition-colors",
    "outline-none",
    // placeholder pseudo-variant: web placeholder color; the VM text field has
    // its own placeholder rendering.
    "placeholder:text-muted-foreground",
];

/// Marker (inert hook) classes carried inside the contract strings.
const MARKER_TOKENS: &[&str] = &["nav-item", "nav-group-label", "nav-group-toggle", "nav-group-content", "nav-search"];

/// Every whitespace-separated token that carries geometry/color on both ends.
/// Excludes web-only enhancement tokens and inert marker classes.
pub fn parity_tokens() -> Vec<&'static str> {
    let all = [
        ITEM_BASE_MD, ITEM_BASE_LG, ITEM_BASE_SM, ITEM_HOVER, ITEM_ACTIVE, ITEM_DISABLED,
        BADGE_PILL, ICON_MD, ICON_LG, TEXTS_FILL, TEXT_DESC, GROUP_LABEL, GROUP_TOGGLE,
        GROUP_TOGGLE_HOVER, GROUP_CONTENT, GROUP_CONTENT_INDENT, SEARCH_ROW, SEARCH_INPUT,
    ];
    let mut tokens = Vec::new();
    for s in all {
        for token in s.split_whitespace() {
            if !WEB_ONLY_TOKENS.contains(&token) && !MARKER_TOKENS.contains(&token) && !tokens.contains(&token) {
                tokens.push(token);
            }
        }
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "ui")]
    use crate::ui::style::StyleClass;

    /// Every parity token must be understood by the VM style parser — this is
    /// the pixel-parity gate for the contract (unknown tokens are silently
    /// dropped by `Style::parse`, so we assert per-token parse success).
    #[cfg(feature = "ui")]
    #[test]
    fn nav_contract_tokens_parse_on_vm() {
        let tokens = parity_tokens();
        assert!(!tokens.is_empty());
        for token in &tokens {
            // hover: utilities are prefixed in the string; Style::parse strips
            // the prefix before delegating to parse_single — mirror that here.
            let bare = token.strip_prefix("hover:").unwrap_or(token);
            assert!(
                StyleClass::parse_single(bare).is_ok(),
                "nav contract token `{token}` is not parseable by the VM style parser — remove it or make it web-only"
            );
        }
    }

    /// The active/hover strings survive `Style::parse` with the hover list
    /// populated (the iced button renderer consumes `hover_classes`).
    #[cfg(feature = "ui")]
    #[test]
    fn nav_contract_hover_parses_into_hover_classes() {
        let style = crate::ui::style::Style::parse(ITEM_HOVER).unwrap();
        assert!(!style.hover_classes.is_empty(), "hover utilities must land in hover_classes");
        let style = crate::ui::style::Style::parse(GROUP_TOGGLE_HOVER).unwrap();
        assert!(!style.hover_classes.is_empty());
    }

    /// The scaffold assets (NavItem.vue / NavGroup.vue) must embed the exact
    /// contract strings — read from the auto-man assets dir so the two ends
    /// cannot drift (Plan 482 G6).
    #[test]
    fn nav_contract_matches_scaffold_assets() {
        let manifest = env!("CARGO_MANIFEST_DIR");
        for (file, consts) in [
            ("nav/NavItem.vue", vec![ITEM_BASE_MD, ITEM_BASE_LG, ITEM_BASE_SM, ITEM_ACTIVE, ITEM_DISABLED, BADGE_PILL, ICON_MD, ICON_LG, TEXTS_FILL, TEXT_DESC]),
            ("nav/NavGroup.vue", vec![GROUP_LABEL, GROUP_TOGGLE, GROUP_TOGGLE_HOVER, GROUP_CONTENT, GROUP_CONTENT_INDENT]),
        ] {
            let path = std::path::Path::new(manifest)
                .join("../auto-man/assets/shadcn-ui")
                .join(file);
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => panic!("scaffold asset {} missing ({e}) — Vue/VM contract drift", path.display()),
            };
            for c in consts {
                assert!(
                    content.contains(c),
                    "scaffold asset {file} no longer embeds contract string `{c}` — update the asset or nav_contract.rs"
                );
            }
        }
    }
}
