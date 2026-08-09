# 023-realworld — Conduit (Medium clone)

A RealWorld (Conduit) spec implementation: authentication, article feed with
tag filtering, article detail with comments. Stage 1 of a two-stage upgrade
(Plan 405) from a single-file toy to a full App, aligned with 015/017/018/022.

> **Stage 1 (this)**: auth (login/register/settings/logout) + global feed +
> tag filter + article detail (read-only) + comments list.
> **Stage 2 (later)**: article CRUD (editor) + post/delete comments +
> follow/unfollow + favorite + profile page + pagination.

## Concepts

- **Multi-route frontend** — `app.at` routes{5} + outlet, cf. 018. Hash routing.
- **Two shared stores** — `AuthStore` (current_user, login/register/logout) +
  `ArticleStore` (articles/current_article/comments/tags). Each page does
  `use store: X`.
- **Strongly-typed Rust backend** — `src/back/{api.at, db.at}`, 6 `#[api]`
  endpoints delegating to db.rs (Plan 399 route B).
- **Client-side tag filtering** — `article.tagList.contains(active_tag)` in the
  view (a2r doesn't clone borrowed String fields when passing to a server
  helper fn, so server-side tag matching is impractical).
- **vue-ref prototype** — `vue-ref/` is a hand-written Vue3 + Tailwind
  reference (mock data) that mirrors the auto codegen stack, used to validate
  interactions before porting to `.at`.

## Source

```
src/front/
  app.at              # App shell: routes{5} + nav (login-aware) + outlet
  auth_store.at       # AuthStore: current_user (id=0 = logged out)
  article_store.at    # ArticleStore: articles/comments/tags, client filter
  pages/
    home.at           # / → global feed + Popular Tags sidebar
    login.at          # /login → login form (value:+oninput:, no v-model)
    register.at       # /register → register form
    article_detail.at # /article/:slug → body + comments (router.param)
    settings.at       # /settings → profile (display-only) + logout
src/back/
  api.at              # pub type User/Article/Comment + 6 #[api] endpoints
  db.at               # typed in-memory store + seed (2 users, 3 articles, 3 comments)
vue-ref/              # Vue3 + Tailwind reference prototype (mock data)
tests/                # playwright suite (T1-T8)
```

## How to Run

> ⚠️ **Port gap (Plan 401)**: `auto run` doesn't read `pac.at`'s front_port; it
> defaults vary. Generate first, then run, and read the printed `Local:` port.

```bash
cd examples/ui/023-realworld
auto gen          # generate vue (routes must exist before run)
auto run          # backend (:8023) + vite dev (watch the printed Local: port)
```

Open the printed URL (hash router — home is `/#/`).

## API (stage 1)

| Method | Path | Action |
|---|---|---|
| POST | `/api/users/login` | log in (email + password → User) |
| POST | `/api/users` | register (username + email + password → User) |
| GET | `/api/user` | current user (stage 1: always id=0 = logged out; no token auth yet) |
| GET | `/api/articles` | list all articles (no server filter) |
| GET | `/api/articles/:slug` | single article |
| GET | `/api/articles/:slug/comments` | comments for an article |

> Authentication is mock: any password works for a known email; `current_user`
> returns id=0 (no real token auth in stage 1). Stage 2 adds JWT verification.

## Tests

Start the servers (above), note the vite port, then:

```bash
cd tests && npm install && npm test    # playwright T1-T8
RW_URL=http://localhost:<vite-port> npm test   # if port differs from default
```

Acceptance contract in `tests/acceptance.atd`. Tests cover: feed render,
tag filter, article detail, register, login, settings + logout, unauthenticated
guard, no console errors.

## Notes / known gotchas (Plan 401 §技术约定, encountered here)

- **`tag` is a reserved soft keyword** (TokenKind::Tag, an enum alias) — cannot
  be a parameter name; a2r fails with "expected '{' or type after 'str'". Used
  `filter_tag` instead. (Backend.)
- **store model fields with struct-literal init** (e.g. `var x User = User{...}`)
  flatten to `null` in codegen — guard templates with `!= nil` before reading
  fields. (Frontend.)
- **Hash routing** — playwright tests must `goto('/#/path')`, not `goto('/path')`.
- **2 a2r codegen bugs fixed** (benefit all backends with string ops): a2r_std
  import path (`use auto_lang::a2r_std`) + back Cargo.toml `auto-lang` dep.

## Inspiration

RealWorld (Conduit) — https://realworld-docs.netlify.app/
