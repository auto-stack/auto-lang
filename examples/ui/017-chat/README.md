# 017-chat — WeChat-Style Messenger (Plan 544 Full-Stack Enhancement)

A modern WeChat/Slack-style instant messaging application demonstrating AutoUI's responsive dual-column layout, shared store state management, and full-stack integration with Rust Axum + SSE multi-event broadcast.

## Features & Highlights

- **Split-Pane Architecture** — Left contact navigation sidebar (`w-72`/`w-80`) + Right conversation main view (`flex-1`).
- **Contact & Conversation Sidebar** (`ChatSidebar`):
  - Current user profile status card (🧑‍💻 You • Online).
  - Real-time search filter bar for contacts and conversations.
  - Multi-session contact list (Alice, Bob, AutoBot 🤖, Tech Support) with avatars, last message preview, timestamps, unread badges, and active state highlight.
- **Rich Message Thread** (`MessageThread`):
  - Conversation header displaying active contact avatar, display name, online status, and quick action buttons.
  - Right-aligned bubbles for sent messages (`bg-primary text-primary-foreground`) and left-aligned bubbles with avatars for received messages (`bg-muted text-foreground`).
  - Cross-tab real-time typing indicator (`... is typing`).
- **Emoji Picker & Action Toolbar** (`Composer`):
  - Integrated Emoji picker toolbar (😀, 😂, 👍, ❤️, 🎉, 🔥, 🚀, 🤖) for 1-click emoji insertion into draft.
  - Preset quick response chips (`Hello! 👋`, `Got it! 👍`) for rapid replies.
  - Send on Enter, disabled/guarded on empty input, automatic input clearing.
- **Full-Stack Backend Integration (`src/back/`)**:
  - `GET /api/contacts` — Contact list and session metadata.
  - `GET /api/messages` — Conversation message history.
  - `POST /api/messages` — Send message and fan-out via SSE `NewMessage` event.
  - `POST /api/bot_reply` — Intelligent assistant auto-responder simulation with SSE broadcast.
  - `POST /api/typing` — Broadcast transient typing indicator across all connected clients.
  - `GET /api/stream` — SSE multi-event stream (`NewMessage`, `Typing`).

## Project Structure

```
017-chat/
├── pac.at                     # Package manifest (scene: "ui", render: "vue", api: "rust")
├── src/
│   ├── front/
│   │   ├── app.at             # Top-level shell assembling Sidebar + MessageThread + Composer
│   │   ├── chat_sidebar.at    # Left contact list, user profile & search filter
│   │   ├── message_thread.at  # Active chat header, bubble stream, typing indicator
│   │   ├── composer.at        # Emoji toolbar, quick action chips, input bar
│   │   ├── chat_store.at      # SharedStore consuming SSE multi-event stream
│   │   └── types.at           # Contact & Message frontend type models
│   └── back/
│       ├── api.at             # HTTP & SSE endpoint contracts (compiled via api_gen to Rust Axum)
│       └── db.at              # In-memory database, contacts service, and bot reply engine
└── tests/
    ├── acceptance.atd         # T1-T13 acceptance test contract
    └── smoke.spec.ts          # Playwright end-to-end smoke tests
```

## How to Run

```bash
cd examples/ui/017-chat
auto gen              # Generate Vue 3 + Tailwind + shadcn-vue project
auto run              # Run frontend dev server (:3000) & backend Axum API server (:8080)
```

## Running Tests

```bash
# Backend compilation and API contract verification
cargo test -p auto-man test_017_chat

# End-to-end UI Playwright smoke tests (requires dev server running)
pnpm test
```
