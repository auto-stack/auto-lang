# 023-realworld — Conduit (Medium clone)

A RealWorld (Conduit) spec implementation, upgraded from a single-file toy to a
full App (Plan 405, stages 1 + 2), aligned with 015/017/018/022. playwright 14/14.

**Scope**: authentication (login/register/settings/logout) + global feed with
tag filtering + article detail + article CRUD (editor) + post/delete comments +
follow/unfollow + favorite + profile page. (Pagination + markdown rendering
deferred.) A hand-written `vue-ref/` prototype validated interactions before
the `.at` port.

> **Auth is mock** (stage 1/2): any password works for a known email;
> `current_user` returns id=0 (logged out). No real token auth — that needs
> codegen support for reading request headers (separate effort).

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

Acceptance contract in `tests/acceptance.atd`. Tests cover (T1-T14): feed
render, tag filter, article detail, register, login, settings + logout,
unauthenticated guard, no console errors, **create article (editor)**, **edit
article**, feed list, **post comment**, **favorite**, **profile page**.

## Notes / known gotchas (Plan 401 §技术约定, encountered here)

- **`tag` is a reserved soft keyword** (TokenKind::Tag, an enum alias) — cannot
  be a parameter name; a2r fails with "expected '{' or type after 'str'". Used
  `filter_tag` instead. (Backend.)
- **store model fields with struct-literal init** (e.g. `var x User = User{...}`)
  flatten to `null` in codegen — guard templates with `!= nil` before reading
  fields. (Frontend. Not yet root-fixed; see Plan 401.)
- **store model field name must not match an imported api function name**
  (stage 2): codegen emits both `import { fn }` and `const fn = ref(...)`;
  the const shadows the import → `fn()` fails. Named the field `me` instead of
  `current_user`. (Frontend.)
- **a widget cannot back multiple routes** (stage 2): codegen gives them the
  same route `name`; vue-router drops the duplicate. Merged `/editor` +
  `/editor/:slug` into one route, using `/editor/new` for create mode.
- **a2r `List<T>` declaration is order-sensitive** (stage 2): a `List<T>` line
  after `var result T = T { ...多字段 }` fails to parse. Workaround: declare
  `List<T>` before the struct-literal `var result`. (Backend. Not root-fixed.)
- **a2r dual path params** (`/:a/:b`): only the first is extracted. Used a
  single path param for delete-comment. (Backend.)
- **Hash routing** — playwright tests must `goto('/#/path')`, not `goto('/path')`.
- **a2r codegen bugs fixed** (benefit all backends with string ops): a2r_std
  import path (`use auto_lang::a2r_std`) + back Cargo.toml `auto-lang` dep +
  `endpoint_has_body` (POST without body params no longer requires a Json body).

## Inspiration

RealWorld (Conduit) — https://realworld-docs.netlify.app/
