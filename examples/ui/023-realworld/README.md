# 023-realworld — Medium.com Clone (Conduit)

A Medium.com clone implementing the Conduit/RealWorld spec: home feed, article detail, and profile pages with tag-based categorization.

## Concepts

- **Multi-page routing** — `current_view` state switches between "home", "article", and "profile" views via `if` blocks
- **Article feed** — article cards with tag badges, title, excerpt, author, and date
- **Article detail** — full article body with author avatar, name, and date
- **Profile page** — user avatar, bio, follower/article counts, and post list
- **Tag badges** — colored pill badges (`bg-green-100 text-green-700 rounded-full`) for article tags
- **Avatar widget** — `avatar` with `rounded-full` for user profile images
- **Typography hierarchy** — `h1`, `h2`, `h3`, `h4`, `p`, `span` elements for content structure
- **Spacer widget** — `spacer` to push navigation items to the right

## Source

See `src/front/app.at`:

```auto
widget App {
    model {
        var current_view str = "home"
        var art1_title str = "Understanding React Server Components"
        var art1_author str = "Sarah Chen"
        var art1_tag str = "React"
        var art1_excerpt str = "Server Components represent..."
        var art1_body str = "Server Components represent..."
        // ... 3 articles + profile data
    }

    view {
        col {
            // Nav bar
            row {
                h3 "Medium"
                spacer
                row { span "Home"; span "About"; span "Write" }
            }

            if .current_view == "home" {
                col {
                    // Article feed with tag badges, titles, excerpts
                    col { h2 .art1_title; p .art1_excerpt; span .art1_tag { class: "rounded-full" } }
                    // ... more articles
                }
            }

            if .current_view == "article" {
                col {
                    h1 .art1_title
                    row { avatar; col { span .art1_author; span .art1_date } }
                    p .art1_body
                }
            }

            if .current_view == "profile" {
                col {
                    row { avatar; col { h2 .profile_name; p .profile_bio } }
                    // User's articles
                }
            }
        }
    }
}
```

## How to Run

```bash
cd examples/ui/023-realworld
auto gen              # Generate code for all backends (vue, jet, ark, rust)
auto run              # Run dev server
```

After `auto gen`, generated projects appear in:
- `gen/vue/` — Vue 3 + shadcn-vue
- `gen/jet/` — Jetpack Compose (Kotlin)
- `gen/ark/` — ArkTS (HarmonyOS)
- `gen/rust/` — Rust GPUI

## Notes

This is a simplified Conduit implementation. Full Conduit spec requires auth flow (login/register), article CRUD, comments, pagination, and tag filtering — these need typed msg variants and control flow in `on` handlers (see Plan 183 parser limitations).

## Inspiration

RealWorld/Conduit (github.com/gothinkster/realworld), Medium.com.
