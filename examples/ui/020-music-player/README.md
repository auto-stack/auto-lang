# 020-music-player — AutoUI Music & Video Player

A modern, full-featured music & video player with dual-backend AutoUI support (Vue mode and VM/Iced mode).

## Features

- **8-track playlist** — Classical masterpieces with title, artist, tag, icon, duration, and like counts
- **Video/Album Stage** — Full-width cover art card with HD/Lossless badge overlay and genre tag
- **Playback controls** — Prev / Play-Pause / Next with Shuffle and Repeat (All / One / Off) modes
- **Progress bar with seek** — `progress { value: .progress_val, max: 100 }` with +10s seek step button
- **Like system** — Heart toggle with live like count update
- **Up Next queue** — Scrollable playlist; clicking any track row instantly switches and auto-plays
- **Dark / Light theme** — Full dark/light mode toggle with distinct color palettes for all elements
- **5 accent colors** — Indigo / Coral / Ocean / Sage / Amber with live color swatch picker
- **Settings panel** — Expandable in-page Settings popover (017-chat style) in the bottom-right corner

## Running

```bash
# Vue dev server (browser)
cd examples/ui/020-music-player
auto run

# VM / Iced native window
auto run -r vm
```

## Architecture

```auto
widget App {
    msg {
        PlayPause, NextTrack, PrevTrack,
        SelectTrack1..SelectTrack8,
        ToggleShuffle, CycleRepeat,
        ToggleLike, SeekStep,
        ToggleSettings, ToggleDarkMode,
        SetTheme(str), SetAccent(str)
    }

    model {
        var dark_mode bool = true
        var accent_color str = "indigo"
        var show_settings bool = false
        var is_playing bool = true
        var current_index int = 1
        var progress_val int = 42
        // ... 8 track data fields + current track state
    }

    view {
        // Two-column layout:
        // Left (560px): Stage card + track metadata + controls + progress
        // Right (flex): Scrollable playlist queue + bottom Settings panel
    }

    on { /* ... handlers for all messages */ }
}
```

## Generated Backends

After `auto gen`, generated projects appear in:
- `gen/vue/` — Vue 3 + Tailwind CSS
- `gen/rust/` — Rust + Iced (via VM backend)

## Testing

```bash
# VM MCP automated test (requires auto run -r vm)
python tests/test_020_vm.py
```

The MCP test exercises: Settings toggle, Dark→Light→Dark theme, Play/Pause, Next track, Playlist track select, Like toggle.

## Inspiration

Spotify, Apple Music, YouTube Music.
