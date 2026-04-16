# 009-article-feed — Blog Article Card Feed

A vertical feed of blog article cards. Each card displays a cover image, title, excerpt text, and a footer row with author avatar, name, and publish date.

## Concepts
- Image cards (cover image + avatar)
- Text with line clamping / truncation
- Repeated items (for loop over articles)
- Avatar (small circular image)

## Source

```auto
widget ArticleFeed {
    model {
        var articles = [
            { title: "Getting Started with AutoLang", excerpt: "Learn the basics of AutoLang and build your first cross-platform app in under ten minutes.", author: "Alice", avatar: "/avatars/alice.png", date: "Apr 10, 2026", image: "/images/getting-started.png" }
            { title: "Cross-Platform UI with AURA", excerpt: "Build once, deploy everywhere. AURA widgets compile to Vue, Compose, ArkTS, and GPUI.", author: "Bob", avatar: "/avatars/bob.png", date: "Apr 12, 2026", image: "/images/cross-platform.png" }
            { title: "Advanced Patterns in AutoLang", excerpt: "Take your skills to the next level with tasks, specs, and the type system.", author: "Charlie", avatar: "/avatars/charlie.png", date: "Apr 14, 2026", image: "/images/advanced-patterns.png" }
        ]
    }

    view {
        col {
            text "Latest Articles"
            col {
                for article in .articles {
                    col {
                        image { src: article.image, class: "w-full h-40 object-cover rounded" }
                        col {
                            text article.title
                            text article.excerpt
                            row {
                                image { src: article.avatar, class: "w-8 h-8 rounded-full" }
                                text article.author
                                text article.date
                                class: "gap-2 items-center"
                            }
                            class: "p-4 gap-2"
                        }
                        class: "bg-white rounded-lg shadow overflow-hidden"
                    }
                }
                class: "gap-6"
            }
            class: "w-full max-w-2xl p-8 gap-6"
        }
    }
}
```

## Generated Output

### Vue 3
<!-- Placeholder: coming soon -->

### Jetpack Compose
<!-- Placeholder: coming soon -->

### ArkTS (HarmonyOS)
<!-- Placeholder: coming soon -->

### GPUI (Rust)
<!-- Placeholder: coming soon -->

### Tauri
<!-- Placeholder: coming soon -->

### VSCode WebView
<!-- Placeholder: coming soon -->

## Platform Notes
- The cover image uses `w-full h-40 object-cover rounded` for a consistent card header. On Jetpack Compose this maps to a `Modifier.fillMaxWidth().height(160.dp).clip(RoundedCornerShape)`.
- The avatar image uses `w-8 h-8 rounded-full` to produce a circular crop. On GPUI, the generator maps `rounded-full` to a circular image container.
- The excerpt text may benefit from line clamping (e.g., 2-line limit). Tailwind provides `line-clamp-2`; on platforms without Tailwind the generator may need to add native line limiting (e.g., `maxLines(2)` in Compose, `textOverflow` in ArkTS).
- `max-w-2xl` constrains the feed width for readability. On mobile targets this is ignored in favor of full-width layout.
- The nested column structure (card > image + content col + footer row) demonstrates a common card composition pattern reusable across many list-based views.
