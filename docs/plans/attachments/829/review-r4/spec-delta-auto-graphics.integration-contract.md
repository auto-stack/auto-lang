# [PROVISIONAL — SD-03] docs/specs/auto-graphics/integration-contract.md

> Status: proposal drafted by PLAN-829 T-09 (condensed from
> `research/integration-handoff.md`); frozen by review, published by merge.

# auto-graphics — integration contract

## Verified (PLAN-829, probe evidence cited)

IH-01 (independent probe level): an iced 0.14 application can host genuine
widgets plus 2D/3D graphics views on ONE wgpu device/queue (the instance
iced_wgpu hands to a custom primitive's prepare). Compositing is opaque
graphics under UI with per-widget clipping; resize/DPI/exit verified; the
blank-final-draw mutation is caught by in-band composite counters
(GDI window capture is environment-blocked — recorded). Same-device proof
is functional + architectural: wgpu 27 does not reject cross-device binds
at the API (global resource ids) — the view path must never construct its
own device (permanent audit test). Iced holds no game business state.
STOCK iced_widget 0.14.2 suffices for this minimal spike; the production
PLAN-043 patch is not required here (r1 F09, recorded).

IH-05 (boundary): independent workspace, stock iced_widget 0.14.2 from
crates.io (r1 F09 de-vendoring; no provenance-tracked patch remains),
frozen sha256 assets, zero root-workspace collateral (V-09 gate).

## Pending (must not read as delivered)

IH-02 Auto SDK: G-base runtime contract exists (identity/time/actions/
events/replay), but no published SDK/profile; `.at` entry is I1 (pending).
IH-03 RQHost: no GPU protocol; current rqhost remains CPU/software.
IH-04 Nesting/Forge: single-window compositing only; no UI→3D→UI matrix,
pick-through, IME, or Forge loop.

## I1–I3 gates

I1 real Auto-source game entry: requires published SDK symbols + receipts;
acceptance = both frozen games driven by `.at` through the public SDK.
I2 production AutoUI/nesting/RQHost: requires migration-completion receipts
(M814-B and successors); the iced probe does not advance it.
I3 Forge & distribution: requires I2 + editing-loop design receipts.

Each successor plan must cite this file and replace "pending" only with
receipt-bound evidence.
