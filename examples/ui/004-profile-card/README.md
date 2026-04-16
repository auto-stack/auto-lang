# 004-profile-card — User Profile Card

A visually rich profile card displaying an avatar image, name, online status badge, role, bio, and action buttons. Demonstrates visual composition without interactivity.

Inspired by PrimeBlocks card patterns.

## Concepts
- **Image widget** — `image` renders an image from a URL source via the `src` property
- **Badge** — Styled text in a rounded pill container to indicate role or status
- **Card layout** — A `col` with shadow, rounded corners, and padding creates a card appearance
- **Col/row nesting** — Mixing `col` and `row` containers for vertical and horizontal arrangements within the same card
- **Styling with classes** — Tailwind-style utility classes control spacing, colors, borders, shadows, and gradients

## Source

```auto
widget ProfileCard {
    model {
        var name str = "Jane Cooper"
        var role str = "Full Stack Developer"
        var bio str = "Passionate about building great user experiences. Open source contributor and coffee enthusiast."
        var avatar_url str = "https://i.pravatar.cc/150?img=47"
        var status str = "online"
    }

    view {
        col {
            // Header with gradient background
            col {
                class: "h-20 bg-gradient-to-r from-blue-500 to-purple-600 rounded-t-lg"
            }

            // Avatar overlapping the header
            col {
                image (src: .avatar_url) {
                    class: "w-20 h-20 rounded-full border-4 border-white shadow-md"
                }
                class: "-mt-10 items-center"
            }

            // Name and status
            col {
                text .name
                class: "text-xl font-bold text-gray-900"

                row {
                    text .status
                    class: "w-3 h-3 rounded-full bg-green-400"
                    text "Active"
                    class: "text-sm text-gray-500"
                }
                class: "gap-2 items-center"
            }

            // Role badge
            row {
                text .role
                class: "px-3 py-1 bg-blue-100 text-blue-800 text-sm rounded-full font-medium"
            }

            // Bio
            text .bio
            class: "text-gray-600 text-sm text-center px-6 leading-relaxed"

            // Action buttons
            row {
                button "Follow" {
                    class: "px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600"
                }
                button "Message" {
                    class: "px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300"
                }
                class: "gap-3"
            }

            class: "bg-white rounded-lg shadow-lg max-w-sm mx-auto items-center gap-4 pb-6"
        }
    }
}
```

## Generated Output

### Vue 3

*(Placeholder: coming soon)*

### Jetpack Compose

*(Placeholder: coming soon)*

### ArkTS (HarmonyOS)

*(Placeholder: coming soon)*

### GPUI (Rust)

*(Placeholder: coming soon)*

### Tauri

*(Placeholder: coming soon)*

### VSCode WebView

*(Placeholder: coming soon)*

## Platform Notes

- The avatar uses negative margin (`-mt-10`) to overlap the gradient header — this is a CSS-specific technique
- On Jetpack Compose, the overlap is achieved with `Box` + `offset` or `paddingTop` on the image
- On GPUI, absolute positioning or overlapping elements in a `div` with negative `mt` achieves the same effect
- The `rounded-full` class creates a circular avatar on web targets; native targets use platform-specific circle clipping
- Gradient backgrounds (`bg-gradient-to-r`) map to `LinearGradient` on Compose, `linearGradient` on ArkTS, and CSS gradients on web
- The status indicator dot uses a small colored `text` element styled as a circle via `w-3 h-3 rounded-full`
