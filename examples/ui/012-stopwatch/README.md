# 012-stopwatch — Start/Stop/Reset Stopwatch with Laps

A stopwatch displaying formatted time (MM:SS.cc) with start, stop, reset controls and a lap times list.

## Concepts

- **Timer/async state** — `running` flag tracks whether the stopwatch is active (actual timer ticks require async support)
- **Start/stop/reset state machine** — three-state control flow with conditional button rendering
- **Lap list** — fixed lap slots (lap1, lap2, lap3) displayed in a column
- **Conditional rendering** — different buttons shown based on `running` state (`if .running == "true"`)

## Source

See `src/front/app.at`:

```auto
widget App {
    msg Msg { Start, Stop, Reset, Lap }

    model {
        var running str = "false"
        var time_display str = "00:00"
        var ms_display str = ".00"
        var lap_count str = "0"
        var lap1 str = ""
        var lap2 str = ""
        var lap3 str = ""
    }

    view {
        center {
            col {
                col {
                    text .time_display {
                        class: "text-7xl font-mono font-bold text-gray-900 tracking-wider"
                    }
                    text .ms_display {
                        class: "text-3xl font-mono text-gray-400"
                    }
                    class: "items-center gap-1 py-12"
                }
                row {
                    if .running == "true" {
                        button "Stop" { onclick: .Stop, class: "..." }
                        button "Lap" { onclick: .Lap, class: "..." }
                    } else {
                        button "Start" { onclick: .Start, class: "..." }
                    }
                }
                col {
                    text "Laps" { class: "..." }
                    text .lap1 { class: "..." }
                    text .lap2 { class: "..." }
                    text .lap3 { class: "..." }
                }
            }
        }
    }

    on {
        .Start -> { .running = "true" }
        .Stop -> { .running = "false" }
        .Reset -> { .running = "false"; .lap_count = "0"; .time_display = "00:00"; .ms_display = ".00" }
        .Lap -> { .lap_count = .lap_count + 1; .lap1 = "Lap 1: 00:00.00" }
    }
}
```

## How to Run

```bash
cd examples/ui/012-stopwatch
auto gen              # Generate code for all backends (vue, jet, ark, rust)
auto run              # Run dev server
```

After `auto gen`, generated projects appear in:
- `gen/vue/` — Vue 3 + shadcn-vue
- `gen/jet/` — Jetpack Compose (Kotlin)
- `gen/ark/` — ArkTS (HarmonyOS)
- `gen/rust/` — Rust GPUI

## Inspiration

iced stopwatch example.
