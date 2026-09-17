# 020-music-player — AutoUI Modern Desktop Music Player

A modern desktop music player inspired by Spotify, QQ Music, NetEase Cloud Music, and Soda Music (汽水音乐), supporting full local music library indexing and streaming from `E:\Music\`.

## Features

- **Modern 3-Column Layout**:
  - **NavSidebar**: Brand logo, navigation tabs (Local Music, Stage, Playlists, Favorites), dynamic stats badge, and dark mode toggle.
  - **Stage View**: Immersive vinyl turntable animation with realistic tonearm, 16-band animated audio spectrum equalizer, lyrics preview card, and Hi-Res / Lossless format badges.
  - **Tracklist View**: Full local music table with search filter, format pills (All, FLAC, MP3, WAV), song metadata (title, artist, size, extension), play/shuffle action buttons, and scan refresh.
  - **Queue Drawer**: Collapsible right-hand upcoming queue drawer showing queue count and instant song switching.
  - **Bottom Player Bar**: Spotify/NetEase style 3-column sticky bar with track cover info, center transport controls (Shuffle, Prev, Play/Pause, Next, Repeat), interactive draggable seekbar with time stamps, volume control, and view/queue toggles.
- **Local Audio Streaming & Media Service**:
  - Automatically indexes local audio files from `media_root: "E:\\Music\\"` in `pac.at`.
  - Supports FLAC, MP3, WAV, OGG, M4A, AAC.
  - Automatic `Artist - Title` parsing from standard music filenames.
  - HTTP 206 partial content byte-range streaming via `/api/media/stream/:id` served by the native backend.
  - Controlled HTML5 `<video>` / audio element syncing seek position, volume, playback state, duration, and error reporting.

## Running

```bash
# Run with Vue frontend (browser) + Rust media backend
cd examples/ui/020-music-player
auto run

# VM / Iced native window
auto run -r vm
```

## Architecture

- `src/front/player_store.at`: Global reactive player store (state machine, library scan, playlist management, seek target, volume, repeat/shuffle modes, computed labels).
- `src/front/nav_sidebar.at`: Left navigation sidebar with branding, views, and playlists.
- `src/front/stage.at`: Immersive playback stage with spinning vinyl disc, tonearm, 16-bar spectrum analyzer, and lyrics card.
- `src/front/tracklist.at`: Local music library table, instant search, format filter tags, and track rows.
- `src/front/queue_drawer.at`: Upcoming play queue drawer with count indicator and track jump.
- `src/front/controls.at`: Bottom 3-column playback controller with draggable progress bar and volume controls.
- `src/front/viewport.at`: Invisible controlled media element binding playback events to the store.
- `src/front/app.at`: Root layout combining sidebar, main stage/tracklist, queue drawer, and bottom controls.

## Inspiration & Design Reference

QQ Music, NetEase Cloud Music (网易云音乐), Soda Music (汽水音乐), Spotify.
