# 018-book-reader — AutoRead

A multi-page e-book reader: a bookshelf, book detail with chapter lists, an
immersive reading view with prev/next navigation, persisted reading progress,
and a real runtime light/dark theme toggle. The first AutoUI example to fuse
**multi-route navigation** (vue-router) with a **strong-typed rust backend**
(CRUD + progress persistence) — modeled on a production reader (Apple Books /
Kindle) within AutoUI's reach.

## Concepts

- **Multi-route SPA** — `routes { ... }` + `<outlet>` in `app.at` drives a real
  vue-router with four pages: `/` (bookshelf), `/book/:id` (detail),
  `/book/:id/chapter/:ch` (reading), `/settings`. Route pages live in
  `src/front/pages/*.at`.
- **Shared store across routes** — `book_store.at` (`BooksStore`) is consumed by
  every page via `use store:`; library state survives navigation.
- **Strong-typed rust backend** — `src/back/api.at` declares `Book`/`Chapter`
  + `#[api]` endpoints (list/get/create/delete books, list/get chapters,
  update progress). `db.at` holds the in-memory store + business logic; the
  generator delegates every handler to `db.rs` (Plan 399 route B, full cover).
- **Reading progress persistence** — entering a chapter PUTs `% read`; the
  bookshelf card and detail "Continue Reading" reflect saved progress.
- **Runtime theme toggle (escape hatch)** — a handmade `vue/src/components/
  ThemeToggle.vue` toggles `<html class="dark">` at runtime (the `theme-toggle`
  tag auto-maps to it), backed by a handmade `index.css`. Demonstrates the
  declarative-DOM + escape-hatch combo (cf. 028-dom-escape).
- **Prev/next chapter navigation** — the reading view fetches the adjacent
  chapter and updates the URL + progress on click.

## Source

App shell (`src/front/app.at`) — the router + sidebar + outlet:

```auto
use book_store: BooksStore

widget App {
    routes {
        "/" -> use bookshelf
        "/book/:id" -> use book_detail
        "/book/:id/chapter/:ch" -> use reading
        "/settings" -> use settings
    }
    msg Msg { Init }
    view {
        row {
            col { /* sidebar: Library / Settings links + theme-toggle */ }
            col { outlet }
        }
    }
    on { .Init -> { store.Init() } }
}
```

Layout: `app.at` = router shell; `pages/*.at` = the four route targets;
`book_store.at` = shared store; `src/back/{api,db}.at` = rust backend.

## How to Run

```bash
cd examples/ui/018-book-reader
auto gen    # generate frontend (gen/front/vue) + rust backend
auto run    # serve frontend (:3018) + backend (:8018), with vite proxy
```

This project declares **dedicated ports** in `pac.at` (`front_port: 3018`,
`back_port: 8018`) so it can run concurrently with other examples (e.g. the
default 3000/8080 used elsewhere) without vite/backend port clashes. `auto run`
reads them automatically; override with `-F`/`-B` if needed:

```bash
auto run -F 3000 -B 8080   # force the default ports instead
```

Open **http://localhost:3018/** (the hash router serves all pages from there).

Generated artifacts (gitignored): `gen/front/vue/` (Vue 3 + shadcn-vue) and
`examples/rust-workspace/018-book-reader-back/` (axum CRUD server).

## Tests

```bash
cd tests && npm install && npm test   # playwright, 10/10 (T1-T10)
```

Acceptance contract in `tests/acceptance.atd`. Start `auto run` first.

## Inspiration

Apple Books, Kindle, and a production reader app (auto-read). Notes, reading
statistics, and Tauri file import are intentionally out of scope for this
example (see Plan 399).
