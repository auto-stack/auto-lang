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

## 5. `style: if` × responsive classes — the label-face/icon-face recipe

**Wrong**
Building two sibling subtrees (wide + rail) and toggling their visibility,
or reaching for a window-size primitive to drive widths.

**Why**
The single-tree composition does it with zero new primitives:

```auto
// label face (texts/chevrons/badges): visible = ¬collapsed ∧ width ≥1024
style: if .collapsed { "hidden" } else { "hidden lg:flex" }
// sidebar column width: responsive classes enter base in declaration
// order when the breakpoint hits — the later lg:w-56 wins
style: if .collapsed { "w-14 items-center" } else { "w-14 items-center lg:w-56" }
```

`hidden` + a display class (`lg:flex`) resolves as display-wins (Plan 409
§10 semantics); the breakpoint re-gates on resize → view rebuild → reparse
(Plan 527 T7). Regression anchor: `test_lg_hidden_override_and_width_cascade`
(ui/style/mod.rs). Copyable to any BP needing manual-state × orientation
AND-combination.

**Right**
Keep one tree; mark the two faces per element (icon face always rendered,
label face behind the conditional). The bp-private `collapsed` var +
`ToggleCollapsed` message (`!` negation, form/login ToggleRemember
precedent) is the whole state surface.

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
