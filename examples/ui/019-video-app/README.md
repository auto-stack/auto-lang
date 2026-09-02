# 019-video-app — Bilibili / YouTube-Style Video App

A dual-backend AutoUI video application supporting both **Vue** mode (`auto run`) and **VM/Iced** mode (`auto run -r vm`).

## Features

- **Multi-Route Navigation**:
  - `/` (Home): Video feed with search, category filter chips, tabs (Recommend / Trending / Following), and responsive video grid.
  - `/watch/:id` (Watch): Player screen preview, channel info, view counter, like button, video description, and related video recommendations.
- **Dual-Backend Themes & Settings**:
  - Light / Dark mode switcher (🌙 / ☀)
  - 5-Color Accent palette (Pink, Indigo, Ocean, Sage, Amber) using the same Settings panel architecture as `017-chat` and `018-book-reader`.
- **Rust Axum Backend**:
  - `src/back/api.at` & `src/back/db.at` providing typed REST endpoints for video listing, filtering, search, view increment, and like toggling.
- **Automated Verification**:
  - Playwright E2E smoke tests (`tests/smoke.spec.ts`)
  - AutoUI MCP VM mode smoke test (`tests/vm-smoke.mjs`)

## Architecture

```
examples/ui/019-video-app/
├── pac.at                 # Config: scene "ui", render "vue", api "rust", ports 3019/8319
├── src/
│   ├── back/
│   │   ├── api.at         # Typed REST endpoints (list_videos, get_video, add_view, etc.)
│   │   └── db.at          # In-memory database with 12 seed videos and filtering/sorting logic
│   └── front/
│       ├── app.at         # Router shell with left sidebar, logo, and settings popover
│       ├── video_store.at # Global VideoStore (videos, filters, search, theme state)
│       ├── settings.at    # SettingsPanel (Theme mode + 5 Accent colors)
│       └── pages/
│           ├── home.at    # Home page with search bar, chips, tabs, and video grid
│           └── watch.at   # Watch page with video player, like button, and related list
└── tests/
    ├── smoke.spec.ts      # 10 Playwright E2E test cases
    └── vm-smoke.mjs       # AutoUI MCP VM-mode smoke test
```

## How to Run

### Vue Mode (Default)

```bash
cd examples/ui/019-video-app
auto run
```

Frontend runs on `http://localhost:3019` and backend API runs on `http://127.0.0.1:8319`.

### VM / Iced Mode

```bash
cd examples/ui/019-video-app
auto run -r vm
```

### Running Tests

```bash
cd examples/ui/019-video-app

# VM Mode MCP Smoke Test
node tests/vm-smoke.mjs

# Playwright E2E Tests (requires auto run)
cd tests && pnpm exec playwright test
```
