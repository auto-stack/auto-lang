# 019-video-app — Bilibili-Style Video Browser

A video browsing app with a search bar, category chips, recommendation tabs, and a responsive video thumbnail grid.

## Concepts

- **Search bar** — `input { placeholder: "Search videos..." }` in the top navigation bar
- **Category chips** — styled `text` elements acting as filter chips with `rounded-full` styling
- **Tabs widget** — `tabs { tab "Recommend" tab "Trending" tab "Following" }` for content switching
- **Responsive grid** — `grid { cols: 3, gap: 4 }` for video thumbnail cards
- **Video cards** — each `grid-item` contains a col with title, author, and view count

## Source

See `src/front/app.at`:

```auto
widget App {
    msg Msg { TabChanged, CategoryChanged }

    model {
        var chip1 str = "All"
        var chip2 str = "Gaming"
        var chip3 str = "Music"
        var vid1_title str = "Amazing Sunset Timelapse"
        var vid1_author str = "NatureCam"
        var vid1_views str = "1.2M views"
        // ... more video entries
    }

    view {
        col {
            row {
                text "VideoApp" { class: "text-xl font-bold text-pink-500" }
                input { placeholder: "Search videos...", class: "flex-1 rounded-full" }
                class: "w-full items-center py-3 border-b"
            }
            row {
                text .chip1 { class: "bg-pink-500 text-white rounded-full" }
                text .chip2 { class: "bg-gray-100 text-gray-700 rounded-full" }
                // ... more chips
            }
            tabs {
                tab "Recommend" { }
                tab "Trending" { }
                tab "Following" { }
            }
            grid {
                grid-item { col { text .vid1_title; row { text .vid1_author; text .vid1_views } } }
                // ... 6 video cards
                cols: 3
                gap: 4
            }
        }
    }
}
```

## How to Run

```bash
cd examples/ui/019-video-app
auto gen              # Generate code for all backends (vue, jet, ark, rust)
auto run              # Run dev server
```

After `auto gen`, generated projects appear in:
- `gen/vue/` — Vue 3 + shadcn-vue
- `gen/jet/` — Jetpack Compose (Kotlin)
- `gen/ark/` — ArkTS (HarmonyOS)
- `gen/rust/` — Rust GPUI

## Inspiration

Bilibili, YouTube.
