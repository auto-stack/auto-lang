# 014-weather — Weather Dashboard

A weather dashboard showing current conditions, temperature, and a tabbed forecast view (daily vs hourly).

## Concepts

- **Tabs** — `TabDaily` and `TabHourly` messages switch between daily and hourly forecast views
- **Conditional rendering** — `if .tab == "daily"` and `if .tab == "hourly"` show different content
- **Data display** — city, temperature, condition, humidity/wind info as styled text
- **Divider widget** — `divider {}` separates forecast sections
- **Gradient styling** — `bg-gradient-to-br from-blue-400 to-blue-600` for the weather card

## Source

See `src/front/app.at`:

```auto
widget App {
    msg Msg { TabDaily, TabHourly, Refresh }

    model {
        var tab str = "daily"
        var city str = "Beijing"
        var temp str = "23°"
        var condition str = "Partly Cloudy"
        var info str = "Humidity: 65% | Wind: 12 km/h"
        var forecast str = "Mon 18~25°  Tue 20~28°  Wed 16~22° ..."
        var hourly str = "14:00 24°  15:00 25°  16:00 24° ..."
    }

    view {
        center {
            col {
                row {
                    text .city { class: "text-2xl font-bold text-gray-800" }
                    button "Refresh" { onclick: .Refresh }
                }
                col {
                    text .condition { class: "text-lg text-blue-100" }
                    text .temp { class: "text-8xl font-thin text-white" }
                    text .info { class: "text-sm text-blue-200" }
                    class: "bg-gradient-to-br from-blue-400 to-blue-600 rounded-2xl p-6"
                }
                row {
                    button "Daily" { onclick: .TabDaily }
                    button "Hourly" { onclick: .TabHourly }
                }
                if .tab == "daily" {
                    col { text "5-Day Forecast"; divider {}; text .forecast }
                }
                if .tab == "hourly" {
                    col { text "Today"; divider {}; text .hourly }
                }
            }
        }
    }

    on {
        .TabDaily -> { .tab = "daily" }
        .TabHourly -> { .tab = "hourly" }
    }
}
```

## How to Run

```bash
cd examples/ui/014-weather
auto gen              # Generate code for all backends (vue, jet, ark, rust)
auto run              # Run dev server
```

After `auto gen`, generated projects appear in:
- `gen/vue/` — Vue 3 + shadcn-vue
- `gen/jet/` — Jetpack Compose (Kotlin)
- `gen/ark/` — ArkTS (HarmonyOS)
- `gen/rust/` — Rust GPUI
