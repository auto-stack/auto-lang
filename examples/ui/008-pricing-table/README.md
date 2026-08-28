# 008-pricing-table — Three-Tier Pricing Table with Theme Settings

Three-tier pricing plans (Single Developer, Team, Enterprise) with default Dark theme, a top title bar, and a Settings panel for switching Light/Dark theme and Accent colors.

## Features

- **Default Theme**: Dark Mode (`theme: "dark"` in `pac.at`, `var dark_mode bool = true`).
- **Top Title Bar**: Header with app icon (`💳`), title (`Pricing Table`), example badge (`008`), and settings trigger button (`⚙ Settings`).
- **Settings Dropdown Panel**:
  - Theme mode toggle: ☀️ Light / 🌙 Dark.
  - Accent color picker: 5 semantic palettes (Indigo, Coral, Ocean, Sage, Amber).
- **Three Pricing Cards**:
  - Single Developer ($39/mo): Starter tier with blue styling.
  - Team ($99/mo): "RECOMMENDED" highlight tier with orange badge and borders.
  - Enterprise ("Exclusive Deals"): High-tier card with zinc styling.
- **Cross-Backend Parity**: Full visual & functional parity verified across Vue (`auto run`) and VM/Iced (`auto run -r vm`).

## How to Run

```bash
cd examples/ui/008-pricing-table
auto build            # Compile and build Vue project
auto run              # Run Vue dev server
auto run -r vm        # Run Native VM + Iced Desktop mode
```

Override theme from CLI:
```bash
auto run --theme light --accent coral
auto run -r vm --theme light --accent ocean
```

