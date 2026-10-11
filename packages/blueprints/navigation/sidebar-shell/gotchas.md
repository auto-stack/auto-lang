# Gotchas — navigation/sidebar-shell

## 1. Forking the shell per page

**Wrong**
Each route re-implements its own header + sidebar copy.

**Why**
The shell is the app's single frame. Per-page forks drift within weeks —
different badge counts, missing the new nav entry, inconsistent user menus
(the store-twin lesson at shell scale).

**Right**
Mount the shell once; routes render into the `content` EDIT region. Nav and
user identity come from props/dataSource, never from page-local copies.

## 2. Duplicating user/session state inside the shell

**Wrong**
```auto
model {
    var user_name str = ""
    // …populated by a second session store owned by the shell
}
```

**Why**
Platform services (session, theme, workspace) are singleton-injected. A
shell-private copy drifts from the real session — the user menu shows a
stale identity after account switch.

**Right**
Take `user` via the prop contract from the app's single session source, and
route identity changes through `actions.sign_out` — never a private store.

## 3. Building the content slot with bp-owned chrome

**Wrong**
Wrapping the `content` region in page padding, headings, or scroll
containers owned by the shell.

**Why**
The routed surface needs full layout control; shell-owned chrome fights
route-level layout and produces double scrollbars and misaligned headers.

**Right**
Keep `content` a single transparent slot; anything visual belongs to the
sidebar/header regions.

## 4. Portrait (<1024) forces the rail — the collapse button is a visual no-op there

**Wrong**
Expecting the collapse toggle to widen the sidebar on a 768-wide portrait
window, or adding an `.at`-readable window-size check to "fix" it ad hoc.

**Why**
PLAN-688's auto-collapse is pure breakpoint gating: the label face is
`hidden lg:flex`, so below 1024 the rail form holds regardless of the
`collapsed` state. The manual state is ANDed with the breakpoint, never
overriding it. Widening in portrait would need a window-size primitive
readable from `.at` — deliberately out of scope (evaluated with the mobile
☰ reversal, see the spec Non-goals).

**Right**
Treat portrait as always-rail v1. If a product needs a manual override,
propose the window-signal primitive at the mobile milestone instead of
forking this package.

## 5. Two-pane collapse — the mutual-exclusion recipe (and its display-class trap)

**Wrong**
Toggling per-element label visibility inside one shared tree (icon button +
label button as siblings per row — they drift apart visually), or writing the
rail pane as `flex lg:hidden`.

**Why**
The collapse is two independent panes with mutually exclusive visibility
(user ruling 2026-09-22: expanded pane shows icon+title as ONE button per
item; the rail pane is icon-only with icons one size larger):

```auto
// wide pane root: visible = ¬collapsed ∧ width ≥1024
style: if .collapsed { "hidden" }
                 else { "hidden lg:block w-56 shrink-0 border-r p-2 gap-2" }
// rail pane root:  visible = collapsed ∨ width <1024
style: if .collapsed { "w-14 shrink-0 border-r p-2 gap-2 items-center" }
                 else { "lg:hidden w-14 shrink-0 border-r p-2 gap-2 items-center" }
```

**The trap**: the rail pane root must carry NO display class (`flex`,
`lg:flex`, …). `Style::is_hidden` uses display-wins-over-Hidden semantics
(Plan 409 §10): a base `flex` makes `lg:hidden` unable to hide the pane at
≥1024 — `flex lg:hidden` stays visible forever. Columns stack their children
without any flex class (iced Column / CSS block), so omitting display costs
nothing. The wide pane uses `lg:block` (not `lg:flex`) for the same reason on
a column root. Resize re-gates via resize → view rebuild → reparse
(Plan 527 T7, machine-verified). Regression anchor:
`test_lg_hidden_override_and_width_cascade` (ui/style/mod.rs).

**Right**
Two subtrees under the sidebar; one `ToggleCollapsed` message (`!`
negation, form/login ToggleRemember precedent) from the wide chevron ◂ and
the rail logo ▲. Rail icon buttons carry `title:` tooltips (PLAN-053 EE03
wiring); rail icons run one size up (`text-lg w-10 h-10` vs the wide pane's
`text-sm`).

## 6. mobile ☰ floating menu — recorded, not built

**Wrong**
Wiring a ☰ button to a Sheet here preemptively, or assuming the floating
layer is missing.

**Why**
The floating-layer primitives exist (VM dropdown-menu/sheet/drawer/popover,
PLAN-533/534). What's missing is an `isMobile` signal and an explicit
reversal of the "mobile mechanism batch not built" non-goal
(`docs/design/autoui/sidebar-family-and-nav-retirement.md` §2.2). Both are
mobile-milestone decisions (spec Non-goals records the demand: ☰ in the top
title bar / bottom tab bar opening a floating nav menu).

**Right**
Leave mobile chrome out of consumers too; revisit at the mobile milestone
together with the portrait manual-override question (#4).
