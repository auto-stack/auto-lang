# 021-blog-viewer — Medium-Style Blog Viewer

A multi-page Medium-style blog application: article stream with category filtering,
rich reading detail view with paragraph splitting, interactive clap feedback,
writing editor with validation and publishing, and two-step confirmation article
deletion. Fully powered by AutoUI **multi-route navigation** (vue-router) and a
**strong-typed Rust backend** (`axum` via route B `db.at` delegation).

## Concepts

- **Multi-route SPA** — `routes { ... }` + `<outlet>` in `src/front/app.at` drives
  a real vue-router with three pages: `/` (home article stream), `/post/:id`
  (article reader), `/new` (story editor).
- **Shared Store across routes** — `blog_store.at` (`BlogStore`) is consumed by
  every page via `use store:`; articles list, active category, and loading state
  survive navigation.
- **Strong-typed Rust backend** — `src/back/api.at` declares `Article` schema and
  5 `#[api]` endpoints (list with optional `?category=`, get single post, create,
  delete, toggle clap). `db.at` maintains in-memory article storage and seed data.
- **Rich reading view & paragraph splitting** — detail view splits article body by
  double newlines (`\n\n`) into readable paragraphs with author attribution and metadata.
- **Interactive clapping** — toggles claps (`POST /api/posts/:id/clap`) with +1/-1
  state tracking per post.
- **Controlled editor & form validation** — `/new` validates non-empty title and content,
  supporting category selection (`Rust`, `AutoUI`, `WASM`) and instant publishing.
- **Two-step delete safety** — detail view provides an inline confirmation box before
  permanently deleting an article and navigating back to home.

## Source

App shell (`src/front/app.at`) — router shell + top bar + outlet:

```auto
use blog_store: BlogStore

widget App {
    routes {
        "/" -> use home
        "/post/:id" -> use detail
        "/new" -> use editor
    }
    msg { Init, GoHome, WritePost }
    view {
        col {
            row {
                row {
                    text "✍️" { class: "text-xl" }
                    text "My Blog" { class: "text-xl font-bold tracking-tight text-foreground" }
                    onclick: .GoHome
                }
                row {
                    text "${.store.articles.len()} articles" { class: "text-sm text-muted-foreground font-medium" }
                    button "Write" {
                        class: "inline-flex items-center gap-1.5 px-4 py-2 bg-primary text-primary-foreground rounded-full text-sm font-semibold hover:bg-primary/90 transition-colors shadow-xs cursor-pointer"
                        onclick: .WritePost
                    }
                }
            }
            col {
                outlet
            }
        }
    }
    on {
        .Init -> { store.Init() }
        .GoHome -> { navigate("/") }
        .WritePost -> { navigate("/new") }
    }
}
```

Layout:
- `src/front/app.at`: Root router shell and sticky header
- `src/front/pages/home.at`: Article cards and category filter chips
- `src/front/pages/detail.at`: Full reader, clap counter, delete confirm
- `src/front/pages/editor.at`: Story publishing form with validation
- `src/front/blog_store.at`: Unified store managing API synchronization
- `src/back/api.at`: Declarative Axum API routes
- `src/back/db.at`: In-memory article database and business operations

## How to Run

```bash
cd examples/ui/021-blog-viewer
auto gen    # generate Vue frontend (gen/front/vue) + Rust backend
```

This project declares dedicated ports in `pac.at` (`front_port: 3021`, `back_port: 8421`)
to prevent port collisions with other examples or system services:

```bash
# Start backend:
$env:AUTO_HTTP_PORT="8421"; cargo run -p app-021-blog-viewer-back

# Start frontend (in gen/front/vue):
$env:AUTO_FRONT_PORT="3021"; $env:AUTO_HTTP_PORT="8421"; pnpm dev
```

Open **http://localhost:3021/** in your browser.

## Tests

```bash
cd examples/ui/021-blog-viewer/tests
pnpm install
pnpm exec playwright test
```

7/7 smoke test cases (`tests/smoke.spec.ts`) cover:
- T1: Home page initial render (top bar, 6 articles count, 6 seed cards)
- T2: Category chip filtering (Rust filter narrows to 2 cards; All restores)
- T3: Detail view navigation (URL `/post/:id`, metadata, >=3 paragraphs)
- T4: Clap toggle (+1 on first click, falls back on second click)
- T5: Story publishing flow (fill form, publish, prepend to home, count becomes 7)
- T6: Form validation (empty title displays warning, prevents submission)
- T7: Deletion flow (delete button, confirmation prompt, removal from list, count restored)

Acceptance contract recorded in `tests/acceptance.atd`.

## Inspiration

Medium, Dev.to, Substack, and JetNews.
