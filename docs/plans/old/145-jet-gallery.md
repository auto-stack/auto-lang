# Jet Gallery Android Compose Gallery Plan

## Status

- Status: Implemented
- Implementation date: `2026-03-24`
- Result: `examples/jet-gallery` has been added as a standalone Jetpack Compose reference app with an adaptive phone/tablet shell, a central widget registry, and demo coverage for the current stdlib widget set.
- Verification: source and tests were added, but final command-line Gradle verification in this terminal session remains partially blocked by local Java/Gradle environment issues. Rebuild in Android Studio or a clean Gradle shell is still recommended.

## Summary

Create a new standalone Android app at `examples/jet-gallery` as a hand-written Jetpack Compose reference gallery for the stdlib AURA widgets that have a real or reasonable Compose analogue. This app is a native Compose target/reference for future `a2jet` work rather than an AURA-authored example.

## Key Changes

- Add `examples/jet-gallery` as a standalone Android/Compose app.
- Keep the source of truth Compose-only; no generator changes in this pass.
- Cover the current stdlib widget surface that maps cleanly or reasonably to Compose.
- Omit the web-only component-gallery pages that are not stdlib widgets.
- Build an adaptive shell:
  - phone: top app bar + bottom navigation + full-screen list/detail flow
  - tablet: navigation rail + persistent two-pane list/detail layout
  - breakpoint: `840dp`
- Add a central widget demo registry with category, stdlib path, support tier, and target Compose pattern notes.
- Use the app as a human-readable reference target for later `a2jet` implementation work.

## Widget Scope

- Layout: `Col`, `Row`, `Center`, `Card`, `ScrollArea`, `AspectRatio`, `Collapsible`, `Accordion`
- Form: `Button`, `Input`, `Checkbox`, `Switch`, `Select`, `Slider`, `RadioGroup`, `Textarea`, `Form`
- Display: `Text`, `Image`, `Badge`, `Avatar`, `Separator`, `Skeleton`, `Swiper`
- Navigation: `Tabs`, `Breadcrumb`, `NavigationMenu`, `Pagination`, `Sidebar`, `MenuBar`, `DropdownMenu`, `NavLink`
- Overlay: `Dialog`, `AlertDialog`, `Sheet`, `Drawer`, `Popover`, `Tooltip`, `HoverCard`, `ContextMenu`
- Feedback: `Alert`, `Toast`, `Progress`, `Sonner`
- Data: `Table`, `DataTable`, `Calendar`, `Grid`, `GridItem`, `List`, `ListItem`

## Test Plan

- `./gradlew assembleDebug` succeeds in `examples/jet-gallery`.
- Unit tests verify widget registry count and exclusion policy.
- Adaptive UI tests verify phone mode uses bottom navigation and tablet mode uses navigation rail.
- Smoke tests cover representative native and composite widget detail pages.

## Assumptions

- Plan number `145` is the lowest free slot in `docs/plans/`.
- The gallery is a target/reference app, not an `a2jet` generator change.
- Phone/tablet responsiveness is determined from screen width with the `840dp` breakpoint.
