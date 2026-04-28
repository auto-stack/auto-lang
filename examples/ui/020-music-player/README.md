# 020-music-player — Spotify-Style Mini Player

A music player with album art, track info, playback controls, a progress bar, and an "Up Next" playlist.

## Concepts

- **Album art display** — gradient placeholder `bg-gradient-to-br from-purple-400 to-indigo-600` acting as album cover
- **Playback controls** — prev/play-pause/next buttons with `PlayPause` / `NextTrack` / `PrevTrack` messages
- **Progress bar with scrubbing** — `progress { value: .progress_val, max: 100 }` bound to model
- **Playlist list** — "Up Next" section with track name and artist rows
- **Toggle state** — `is_playing` toggles between "Playing" and "Paused" text

## Source

See `src/front/app.at`:

```auto
widget App {
    msg Msg { PlayPause, NextTrack, PrevTrack, SeekChanged }

    model {
        var is_playing str = "Playing"
        var track_title str = "Moonlight Sonata"
        var track_artist str = "Beethoven"
        var current_time str = "2:15"
        var total_time str = "5:30"
        var progress_val int = 41
        var track1 str = "Moonlight Sonata"
        var artist1 str = "Beethoven"
        // ... 5 tracks
    }

    view {
        col {
            col {
                row {
                    col { class: "w-64 h-64 bg-gradient-to-br from-purple-400 to-indigo-600 rounded-2xl" }
                }
                text .track_title { class: "text-2xl font-bold mt-8" }
                text .track_artist { class: "text-sm text-gray-500" }
            }
            col {
                progress { value: .progress_val, max: 100 }
                row { text .current_time; text .total_time }
            }
            row {
                button "Prev" { onclick: .PrevTrack }
                button .is_playing { onclick: .PlayPause, class: "w-14 h-14 bg-purple-500 rounded-full" }
                button "Next" { onclick: .NextTrack }
            }
            col {
                text "Up Next"
                row { text .track1 { class: "text-purple-700" }; text .artist1 }
                // ... playlist rows
            }
        }
    }

    on {
        .PlayPause -> {
            if .is_playing == "Playing" { .is_playing = "Paused" } else { .is_playing = "Playing" }
        }
        .NextTrack -> { .track_title = .track2; .track_artist = .artist2 }
    }
}
```

## How to Run

```bash
cd examples/ui/020-music-player
auto gen              # Generate code for all backends (vue, jet, ark, rust)
auto run              # Run dev server
```

After `auto gen`, generated projects appear in:
- `gen/vue/` — Vue 3 + shadcn-vue
- `gen/jet/` — Jetpack Compose (Kotlin)
- `gen/ark/` — ArkTS (HarmonyOS)
- `gen/rust/` — Rust GPUI

## Inspiration

Spotify, Apple Music.
