#!/usr/bin/env python3
"""Generate the Tailwind v3.4 core utility manifest for Plan 527.

Expands the official tailwindcss 3.4.x corePlugins scales (base utilities
only — no variant prefixes, no arbitrary values, no opacity modifiers) into
`crates/auto-lang/tests/fixtures/tailwind-v34-utilities.txt`, one class per
line, grouped by the docs-family sections the style-parity audit table uses.

The scales below are the authoritative constants from tailwindcss v3.4's
default theme (`tailwindcss/stubs/config.full.js` + corePlugins). The script
is deterministic and dependency-free so the manifest is reproducible offline
(Plan 527 T1 待澄清④裁定: 入库 tools/).

Usage: python tools/gen_tailwind_manifest.py
"""

from __future__ import annotations

import argparse
from pathlib import Path

# ---------------------------------------------------------------------------
# Official Tailwind v3.4 scales
# ---------------------------------------------------------------------------

SPACING = [
    "0", "px", "0.5", "1", "1.5", "2", "2.5", "3", "3.5", "4", "5", "6", "7",
    "8", "9", "10", "11", "12", "14", "16", "20", "24", "28", "32", "36", "40",
    "44", "48", "52", "56", "60", "64", "72", "80", "96",
]

# inset/top/right/bottom/left support these fractions
INSET_FRACTIONS = ["1/2", "1/3", "2/3", "1/4", "2/4", "3/4"]

# width/height/translate flex fractions (n/12 set included)
SIZE_FRACTIONS = [
    "1/2", "1/3", "2/3", "1/4", "2/4", "3/4",
    "1/5", "2/5", "3/5", "4/5",
    "1/6", "2/6", "3/6", "4/6", "5/6",
] + [f"{n}/12" for n in range(1, 12)]

COLOR_FAMILIES = [
    "red", "orange", "amber", "yellow", "lime", "green", "emerald", "teal",
    "cyan", "sky", "blue", "indigo", "violet", "purple", "fuchsia", "pink",
    "rose", "slate", "gray", "zinc", "neutral", "stone",
]
SHADES = ["50", "100", "200", "300", "400", "500", "600", "700", "800", "900", "950"]
# color-scale entries (family-shade) plus the bare tokens usable as colors
COLOR_NAMES = [f"{fam}-{shade}" for fam in COLOR_FAMILIES for shade in SHADES] + [
    "inherit", "current", "transparent", "black", "white",
]

MAX_W_NAMED = ["none", "xs", "sm", "md", "lg", "xl", "2xl", "3xl", "4xl", "5xl",
               "6xl", "7xl", "full", "min", "max", "fit", "prose",
               "screen-sm", "screen-md", "screen-lg", "screen-xl", "screen-2xl"]

BORDER_RADIUS = ["none", "sm", "DEFAULT", "md", "lg", "xl", "2xl", "3xl", "full"]

BORDER_WIDTHS = ["DEFAULT", "0", "2", "4", "8"]

OUTLINE_OFFSETS = ["0", "1", "2", "4", "8"]
DECORATION_THICKNESS = ["auto", "from-font", "0", "1", "2", "4", "8"]

RING_WIDTHS = ["DEFAULT", "0", "1", "2", "4", "8"]

OPACITY = [str(n) for n in range(0, 101, 5)]

BRIGHTNESS = ["0", "50", "75", "90", "95", "100", "105", "110", "125", "150", "200"]
CONTRAST = ["0", "50", "75", "100", "125", "150"]
PERCENT_FILTER = ["0", "25", "50", "75", "100"]  # grayscale / invert / sepia
HUE_ROTATE = ["0", "15", "30", "60", "90", "180"]
SATURATE = ["0", "50", "100", "150", "200"]

BLUR = ["none", "sm", "DEFAULT", "md", "lg", "xl", "2xl", "3xl"]
DROP_SHADOW = ["sm", "DEFAULT", "md", "lg", "xl", "2xl", "none"]

SCALE = ["0", "50", "75", "90", "95", "100", "105", "110", "125", "150"]
ROTATE = ["0", "1", "2", "3", "6", "12", "45", "90", "180"]
SKEW = ["0", "1", "2", "3", "6", "12"]

TRANSITION_TIMES = ["75", "100", "150", "200", "300", "500", "700", "1000"]

Z_INDEX = ["0", "10", "20", "30", "40", "50", "auto"]

COLUMNS_NAMED = ["auto", "3xs", "2xs", "xs", "sm", "md", "lg", "xl",
                 "2xl", "3xl", "4xl", "5xl", "6xl", "7xl"]

BLEND_MODES = ["normal", "multiply", "screen", "overlay", "darken", "lighten",
               "color-dodge", "color-burn", "hard-light", "soft-light",
               "difference", "exclusion", "hue", "saturation", "color",
               "luminosity", "plus-lighter"]

CURSORS = [
    "auto", "default", "pointer", "wait", "text", "move", "help",
    "not-allowed", "none", "context-menu", "progress", "cell", "crosshair",
    "vertical-text", "alias", "copy", "no-drop", "grab", "grabbing",
    "all-scroll", "col-resize", "row-resize", "n-resize", "e-resize",
    "s-resize", "w-resize", "ne-resize", "nw-resize", "se-resize",
    "sw-resize", "ew-resize", "ns-resize", "nesw-resize", "nwse-resize",
    "zoom-in", "zoom-out",
]

OBJECT_POSITIONS = ["bottom", "center", "left", "left-bottom", "left-top",
                    "right", "right-bottom", "right-top", "top"]

GRADIENT_DIRS = ["t", "tr", "r", "br", "b", "bl", "l", "tl"]

GRADIENT_STOP_PCT = [str(n) for n in range(0, 101, 5)]

FONT_SIZES = ["xs", "sm", "base", "lg", "xl", "2xl", "3xl", "4xl", "5xl",
              "6xl", "7xl", "8xl", "9xl"]

FONT_WEIGHTS = ["thin", "extralight", "light", "normal", "medium",
                "semibold", "bold", "extrabold", "black"]

LEADING_NAMED = ["none", "tight", "snug", "normal", "relaxed", "loose"]
LEADING_NUMERIC = ["3", "4", "5", "6", "7", "8", "9", "10"]
TRACKING = ["tighter", "tight", "normal", "wide", "wider", "widest"]

LINE_CLAMP = ["1", "2", "3", "4", "5", "6", "none"]


def bare(scale: list[str]) -> list[str]:
    """Scale entries without the DEFAULT sentinel (rendered as the bare word)."""
    return [s for s in scale if s != "DEFAULT"]


def d(class_name: str) -> str:
    """Render DEFAULT as the bare utility (rounded-DEFAULT → rounded)."""
    if class_name.endswith("-DEFAULT"):
        return class_name[: -len("-DEFAULT")]
    return class_name


def neg(classes: list[str]) -> list[str]:
    return ["-" + c for c in classes]


# ---------------------------------------------------------------------------
# Family sections, in docs order
# ---------------------------------------------------------------------------

def section_layout() -> list[str]:
    out = [
        # boxSizing
        "box-border", "box-content",
        # container
        "container",
        # display
        "block", "inline-block", "inline", "flex", "inline-flex", "table",
        "inline-table", "table-caption", "table-cell", "table-column",
        "table-column-group", "table-footer-group", "table-header-group",
        "table-row-group", "table-row", "flow-root", "grid", "inline-grid",
        "contents", "list-item", "hidden",
        # float / clear
        "float-right", "float-left", "float-none", "float-start", "float-end",
        "clear-left", "clear-right", "clear-both", "clear-start", "clear-end",
        "clear-none",
        # isolation
        "isolate", "isolation-auto",
        # objectFit
        "object-contain", "object-cover", "object-fill", "object-none",
        "object-scale-down",
        # objectPosition
        *[f"object-{p}" for p in OBJECT_POSITIONS],
        # overflow / overscroll
        *[f"overflow-{v}" for v in ["auto", "hidden", "clip", "visible", "scroll"]],
        *[f"overflow-{ax}-{v}" for ax in "xy" for v in ["auto", "hidden", "clip", "visible", "scroll"]],
        *[f"overscroll-{v}" for v in ["auto", "contain", "none"]],
        *[f"overscroll-{ax}-{v}" for ax in "xy" for v in ["auto", "contain", "none"]],
        # position
        "static", "fixed", "absolute", "relative", "sticky",
        # visibility
        "visible", "invisible",
        # zIndex
        *[f"z-{v}" for v in Z_INDEX],
        # columns
        *[f"columns-{n}" for n in range(1, 13)],
        *[f"columns-{v}" for v in COLUMNS_NAMED],
        # breakBefore / breakAfter / breakInside
        *[f"break-before-{v}" for v in ["auto", "avoid", "avoid-page", "avoid-column"]],
        *[f"break-after-{v}" for v in ["auto", "avoid", "avoid-page", "avoid-column"]],
        *[f"break-inside-{v}" for v in ["auto", "avoid", "avoid-page", "avoid-column"]],
        # boxDecorationBreak
        "decoration-slice", "decoration-clone",
        # aspectRatio (v3.4 static scale; ratios are arbitrary values)
        "aspect-auto", "aspect-square", "aspect-video",
    ]
    # inset / sides: spacing + fractions + auto + full, plus negatives
    for prefix in ["inset", "inset-x", "inset-y", "top", "right", "bottom", "left"]:
        out += [f"{prefix}-{v}" for v in SPACING]
        out += [f"{prefix}-{v}" for v in INSET_FRACTIONS]
        out += [f"{prefix}-auto", f"{prefix}-full"]
        out += neg([f"{prefix}-{v}" for v in SPACING[1:] if v != "px"]
                   + [f"{prefix}-{v}" for v in INSET_FRACTIONS])
    return out


def section_flexbox() -> list[str]:
    out = [
        "flex-row", "flex-row-reverse", "flex-col", "flex-col-reverse",
        "flex-wrap", "flex-wrap-reverse", "flex-nowrap",
        "items-start", "items-end", "items-center", "items-baseline", "items-stretch",
        "justify-start", "justify-end", "justify-center", "justify-between",
        "justify-around", "justify-evenly",
        "justify-items-start", "justify-items-end", "justify-items-center",
        "justify-items-stretch",
        "justify-self-auto", "justify-self-start", "justify-self-end",
        "justify-self-center", "justify-self-stretch",
        "grow", "grow-0", "shrink", "shrink-0",
        "flex-1", "flex-auto", "flex-initial", "flex-none",
        "order-first", "order-last", "order-none",
        *[f"order-{n}" for n in range(1, 13)],
        "col-auto", "col-start-auto", "col-end-auto",
        *[f"col-span-{n}" for n in range(1, 13)],
        *[f"col-start-{n}" for n in range(1, 14)],
        *[f"col-end-{n}" for n in range(1, 14)],
        "row-auto", "row-start-auto", "row-end-auto",
        *[f"row-span-{n}" for n in range(1, 7)],
        *[f"row-start-{n}" for n in range(1, 8)],
        *[f"row-end-{n}" for n in range(1, 8)],
        "auto-cols-auto", "auto-cols-min", "auto-cols-max", "auto-cols-fr",
        "auto-rows-auto", "auto-rows-min", "auto-rows-max", "auto-rows-fr",
        "grid-flow-row", "grid-flow-col", "grid-flow-dense",
        "grid-flow-row-dense", "grid-flow-col-dense",
        "grid-cols-none", "grid-rows-none",
        *[f"grid-cols-{n}" for n in range(1, 13)],
        *[f"grid-rows-{n}" for n in range(1, 7)],
    ]
    # flex-basis: spacing + fractions + auto + full
    out += ["basis-auto", "basis-full"]
    out += [f"basis-{v}" for v in SPACING]
    out += [f"basis-{v}" for v in INSET_FRACTIONS]
    return out


def section_spacing() -> list[str]:
    out = []
    for prefix in ["p", "px", "py", "pt", "pr", "pb", "pl"]:
        out += [f"{prefix}-{v}" for v in SPACING]
    for prefix in ["m", "mx", "my", "mt", "mr", "mb", "ml"]:
        out += [f"{prefix}-{v}" for v in SPACING]
        out += [f"{prefix}-auto"]
        out += neg([f"{prefix}-{v}" for v in SPACING[1:] if v != "px"])
    for ax in ["x", "y"]:
        out += [f"space-{ax}-{v}" for v in SPACING]
        out += [f"space-{ax}-reverse"]
        out += neg([f"space-{ax}-{v}" for v in SPACING[1:] if v != "px"])
    return out


def section_sizing() -> list[str]:
    out = [
        *[f"w-{v}" for v in SPACING],
        *[f"w-{v}" for v in SIZE_FRACTIONS],
        "w-auto", "w-min", "w-max", "w-fit", "w-full", "w-screen",
        *[f"min-w-{v}" for v in SPACING],
        "min-w-min", "min-w-max", "min-w-fit",
        *[f"max-w-{v}" for v in MAX_W_NAMED],
        *[f"max-w-{v}" for v in SPACING],
        *[f"h-{v}" for v in SPACING],
        *[f"h-{v}" for v in SIZE_FRACTIONS],
        "h-auto", "h-min", "h-max", "h-fit", "h-full", "h-screen",
        "h-svh", "h-lvh", "h-dvh",
        *[f"min-h-{v}" for v in SPACING],
        "min-h-min", "min-h-max", "min-h-fit", "min-h-full", "min-h-screen",
        "min-h-svh", "min-h-lvh", "min-h-dvh",
        *[f"max-h-{v}" for v in SPACING],
        "max-h-none", "max-h-min", "max-h-max", "max-h-fit", "max-h-full",
        "max-h-screen",
    ]
    return out


def section_typography(colors: list[str]) -> list[str]:
    out = [
        "font-sans", "font-serif", "font-mono",
        *[f"text-{v}" for v in FONT_SIZES],
        *[f"font-{v}" for v in FONT_WEIGHTS],
        *[f"tracking-{v}" for v in TRACKING],
        *[f"leading-{v}" for v in LEADING_NAMED + LEADING_NUMERIC],
        "list-none", "list-disc", "list-decimal", "list-inside", "list-outside",
        "text-left", "text-center", "text-right", "text-justify",
        "text-start", "text-end",
        *[f"indent-{v}" for v in SPACING],
        *neg([f"indent-{v}" for v in SPACING[1:] if v != "px"]),
        "align-baseline", "align-top", "align-middle", "align-bottom",
        "align-text-top", "align-text-bottom", "align-sub", "align-super",
        "whitespace-normal", "whitespace-nowrap", "whitespace-pre",
        "whitespace-pre-line", "whitespace-pre-wrap",
        "whitespace-break-spaces",
        "break-normal", "break-words", "break-all",
        "wrap-normal", "wrap-anywhere", "wrap-break-word",
        "content-none",
        "hyphens-none", "hyphens-manual", "hyphens-auto",
        "uppercase", "lowercase", "capitalize", "normal-case",
        "truncate", "text-ellipsis", "text-clip",
        "underline", "overline", "line-through", "no-underline",
        "decoration-solid", "decoration-double", "decoration-dotted",
        "decoration-dashed", "decoration-wavy",
        *[f"decoration-{v}" for v in DECORATION_THICKNESS],
        *[f"underline-offset-{v}" for v in OUTLINE_OFFSETS + ["auto"]],
        "antialiased", "subpixel-antialiased",
        "normal-nums", "ordinal", "slashed-zero", "lining-nums",
        "oldstyle-nums", "proportional-nums", "tabular-nums",
        "diagonal-fractions", "stacked-fractions",
        *[f"line-clamp-{v}" for v in LINE_CLAMP],
        # textColor
        *[f"text-{c}" for c in colors],
    ]
    return out


def section_backgrounds(colors: list[str]) -> list[str]:
    out = [
        "bg-fixed", "bg-local", "bg-scroll",
        "bg-clip-border", "bg-clip-padding", "bg-clip-content", "bg-clip-text",
        "bg-none",
        *[f"bg-gradient-to-{dir_}" for dir_ in GRADIENT_DIRS],
        "bg-origin-border", "bg-origin-padding", "bg-origin-content",
        *[f"bg-{p}" for p in OBJECT_POSITIONS],
        "bg-repeat", "bg-no-repeat", "bg-repeat-x", "bg-repeat-y",
        "bg-repeat-round", "bg-repeat-space",
        "bg-auto", "bg-cover", "bg-contain",
        # gradientColorStops: colors + percent positions
        *[f"from-{v}" for v in GRADIENT_STOP_PCT],
        *[f"via-{v}" for v in GRADIENT_STOP_PCT],
        *[f"to-{v}" for v in GRADIENT_STOP_PCT],
    ]
    out += [f"bg-{c}" for c in colors]
    out += [f"from-{c}" for c in colors]
    out += [f"via-{c}" for c in colors]
    out += [f"to-{c}" for c in colors]
    return out


def section_borders(colors: list[str]) -> list[str]:
    out = [
        *[d(f"rounded-{v}") for v in BORDER_RADIUS],
        *[f"rounded-{side}-{v}"
          for side in ["t", "r", "b", "l", "tl", "tr", "br", "bl"]
          for v in bare(BORDER_RADIUS)],
        *[d(f"border-{v}") for v in BORDER_WIDTHS],
        *[f"border-{side}-{v}"
          for side in ["x", "y", "t", "r", "b", "l"]
          for v in bare(BORDER_WIDTHS)],
        "border-solid", "border-dashed", "border-dotted", "border-double",
        "border-hidden", "border-none",
        *[d(f"divide-x-{v}") for v in bare(BORDER_WIDTHS) + ["DEFAULT"]],
        *[d(f"divide-y-{v}") for v in bare(BORDER_WIDTHS) + ["DEFAULT"]],
        "divide-x-reverse", "divide-y-reverse",
        "divide-solid", "divide-dashed", "divide-dotted", "divide-double",
        "divide-none",
        "outline", "outline-none", "outline-dashed", "outline-dotted",
        "outline-double", "outline-hidden",
        *[f"outline-offset-{v}" for v in OUTLINE_OFFSETS],
        *[d(f"ring-{v}") for v in RING_WIDTHS],
        "ring-inset",
        *[d(f"ring-offset-{v}") for v in bare(RING_WIDTHS)],
    ]
    out += [f"border-{c}" for c in colors]
    out += [f"border-{side}-{c}" for side in ["x", "y", "t", "r", "b", "l"] for c in colors]
    out += [f"divide-{c}" for c in colors]
    out += [f"outline-{c}" for c in colors]
    out += [f"ring-{c}" for c in colors]
    out += [f"ring-offset-{c}" for c in colors]
    return out


def section_effects(colors: list[str]) -> list[str]:
    out = [
        "shadow-sm", "shadow", "shadow-md", "shadow-lg", "shadow-xl",
        "shadow-2xl", "shadow-inner", "shadow-none",
        *[f"opacity-{v}" for v in OPACITY],
        *[f"mix-blend-{m}" for m in BLEND_MODES],
        *[f"bg-blend-{m}" for m in BLEND_MODES if m != "plus-lighter"],
    ]
    out += [f"shadow-{c}" for c in colors]
    return out


def filter_utilities(base: str) -> list[str]:
    """base = "" (plain) or "backdrop" — f"{base}-" prefixed names."""
    p = f"{base}-" if base else ""
    out = [f"{p}filter", f"{p}filter-none"]
    if base == "backdrop":
        out += [f"backdrop-blur-{v}" if v != "DEFAULT" else "backdrop-blur" for v in BLUR]
        out += [f"backdrop-opacity-{v}" for v in OPACITY]
        out += [f"backdrop-saturate-{v}" for v in SATURATE]
    else:
        out += [f"blur-{v}" if v != "DEFAULT" else "blur" for v in BLUR]
        out += [f"saturate-{v}" for v in SATURATE]
        out += [f"drop-shadow-{v}" if v != "DEFAULT" else "drop-shadow" for v in DROP_SHADOW]
    out += [f"{p}brightness-{v}" for v in BRIGHTNESS]
    out += [f"{p}contrast-{v}" for v in CONTRAST]
    out += [f"{p}grayscale-{v}" for v in PERCENT_FILTER]
    out += [f"{p}invert-{v}" for v in PERCENT_FILTER]
    out += [f"{p}sepia-{v}" for v in PERCENT_FILTER]
    out += [f"{p}hue-rotate-{v}" for v in HUE_ROTATE]
    out += neg([f"{p}hue-rotate-{v}" for v in HUE_ROTATE[1:]])
    return out


def section_filters() -> list[str]:
    return filter_utilities("") + filter_utilities("backdrop")


def section_tables() -> list[str]:
    out = [
        "border-collapse", "border-separate",
        *[f"border-spacing-{v}" for v in SPACING],
        *[f"border-spacing-x-{v}" for v in SPACING],
        *[f"border-spacing-y-{v}" for v in SPACING],
        "table-auto", "table-fixed",
        "caption-top", "caption-bottom",
    ]
    return out


def section_transitions() -> list[str]:
    return [
        "transition", "transition-none", "transition-all",
        "transition-colors", "transition-opacity", "transition-shadow",
        "transition-transform",
        *[f"delay-{v}" for v in TRANSITION_TIMES],
        *[f"duration-{v}" for v in TRANSITION_TIMES],
        "ease-linear", "ease-in", "ease-out", "ease-in-out",
        "animate-none", "animate-spin", "animate-ping", "animate-pulse",
        "animate-bounce",
    ]


def section_transforms() -> list[str]:
    out = [
        "transform", "transform-gpu", "transform-none",
        *[f"scale-{v}" for v in SCALE],
        *[f"scale-x-{v}" for v in SCALE],
        *[f"scale-y-{v}" for v in SCALE],
        *[f"rotate-{v}" for v in ROTATE],
        *neg([f"rotate-{v}" for v in ROTATE[1:]]),
        "origin-center", "origin-top", "origin-top-right", "origin-right",
        "origin-bottom-right", "origin-bottom", "origin-bottom-left",
        "origin-left", "origin-top-left",
        *[f"skew-x-{v}" for v in SKEW],
        *[f"skew-y-{v}" for v in SKEW],
        *neg([f"skew-x-{v}" for v in SKEW[1:]]),
        *neg([f"skew-y-{v}" for v in SKEW[1:]]),
    ]
    for ax in ["x", "y"]:
        out += [f"translate-{ax}-{v}" for v in SPACING]
        out += [f"translate-{ax}-{v}" for v in INSET_FRACTIONS + ["full"]]
        out += neg([f"translate-{ax}-{v}" for v in SPACING[1:] if v != "px"]
                   + [f"translate-{ax}-{v}" for v in INSET_FRACTIONS])
    return out


def section_interactivity(colors: list[str]) -> list[str]:
    out = [
        "appearance-none", "appearance-auto",
        *[f"cursor-{c}" for c in CURSORS],
        "pointer-events-none", "pointer-events-auto",
        "resize-none", "resize-y", "resize-x", "resize",
        "scroll-auto", "scroll-smooth",
        *[f"scroll-{kind}{'' if side == '' else '-' + side}-{v}"
          for kind in ["m", "p"]
          for side in ["", "x", "y", "t", "r", "b", "l"]
          for v in SPACING],
        "touch-auto", "touch-none", "touch-pan-x", "touch-pan-left",
        "touch-pan-right", "touch-pan-y", "touch-pan-up", "touch-pan-down",
        "touch-pinch-zoom", "touch-manipulation",
        "select-none", "select-text", "select-all", "select-auto",
        "will-change-auto", "will-change-scroll", "will-change-contents",
        "will-change-transform",
    ]
    out += [f"accent-{c}" for c in colors]
    out += [f"caret-{c}" for c in colors]
    return out


def section_svg(colors: list[str]) -> list[str]:
    out = [f"fill-{c}" for c in colors]
    out += [f"stroke-{c}" for c in colors]
    out += [f"stroke-{v}" for v in ["0", "1", "2"]]
    return out


def section_accessibility() -> list[str]:
    return [
        "sr-only", "not-sr-only",
        "forced-color-adjust-auto", "forced-color-adjust-none",
    ]


SECTIONS = [
    ("layout", section_layout),
    ("flexbox", section_flexbox),
    ("spacing", section_spacing),
    ("sizing", section_sizing),
    ("typography", lambda: section_typography(COLOR_NAMES)),
    ("backgrounds", lambda: section_backgrounds(COLOR_NAMES)),
    ("borders", lambda: section_borders(COLOR_NAMES)),
    ("effects", lambda: section_effects(COLOR_NAMES)),
    ("filters", section_filters),
    ("tables", section_tables),
    ("transitions", section_transitions),
    ("transforms", section_transforms),
    ("interactivity", lambda: section_interactivity(COLOR_NAMES)),
    ("svg", lambda: section_svg(COLOR_NAMES)),
    ("accessibility", section_accessibility),
]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--out",
        default="crates/auto-lang/tests/fixtures/tailwind-v34-utilities.txt",
        help="output manifest path (repo-root relative by default)",
    )
    args = parser.parse_args()

    lines = [
        "# Tailwind v3.4 core utility manifest — GENERATED, do not edit by hand.",
        "# Base utilities only: no variant prefixes (hover:/dark:/md:…), no",
        "# arbitrary values (…-[Npx]), no opacity modifiers (…/75).",
        "# Source: official tailwindcss 3.4.x corePlugins default-theme scales,",
        "# expanded & deduped by tools/gen_tailwind_manifest.py (Plan 527 T1).",
        "# Format: `# family <name>` starts a docs-family section; each",
        "# non-comment line is one utility class consumed by tests/style_parity.rs.",
        "",
    ]
    seen: set[str] = set()
    total = 0
    for name, build in SECTIONS:
        section_lines = []
        for cls in build():
            if cls in seen:
                continue
            seen.add(cls)
            section_lines.append(cls)
            total += 1
        lines.append(f"# family {name}")
        lines.extend(section_lines)
        lines.append("")

    out_path = Path(args.out)
    out_path.write_text("\n".join(lines), encoding="utf-8", newline="\n")
    print(f"wrote {total} utilities across {len(SECTIONS)} families -> {out_path}")


if __name__ == "__main__":
    main()
