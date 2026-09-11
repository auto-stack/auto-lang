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
├── pac.at                 # Config: scene "ui", render "vue", api "rust", ports 3019/8019
├── src/
│   ├── front/
│   │   ├── app.at         # Root widget with responsive layout & navigation
│   │   ├── stores/        # Frontend state management (Vue stores)
│   │   │   └── video.at   # Video store (active tab, selected video, search)
│   │   ├── components/    # Reusable UI components
│   │   │   ├── nav_header.at     # Top navbar with search & notifications
│   │   │   ├── video_card.at     # Video card with thumbnail & meta
│   │   │   ├── video_player.at   # Interactive player with controls
│   │   │   ├── comment_section.at# Comments list & input form
│   │   │   └── settings_panel.at # Slide-over settings drawer
│   │   └── pages/
│   │       ├── home.at    # Main feed with categories & video grid
│   │       ├── watch.at   # Video detail page with player & recommendations
│   │       └── profile.at # User profile & uploaded videos
│   └── back/
│       ├── api.at         # REST API declarations (#[api] routes)
│       └── db.at          # In-memory database with seed data
└── tests/
    ├── package.json       # Playwright dependencies
    ├── playwright.config.ts # E2E test configuration
    ├── smoke.spec.ts      # 10 comprehensive E2E tests
    └── vm-smoke.mjs       # Headless VM smoke test (10 assertions)
```

## Running

```bash
# Standard run (Vue mode)
auto run

# Frontend runs on `http://localhost:3019` and backend API runs on `http://127.0.0.1:8019`.
```

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
