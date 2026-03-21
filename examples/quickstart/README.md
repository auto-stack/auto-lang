# QuickStart Tutorials

HarmonyOS QuickStart tutorials reimplemented in Auto (AURA syntax).

## Tutorials

| # | Name | Topics |
|---|------|--------|
| 01 | HelloWorld | Column, Text, Image, custom components |
| 02 | Banner | Swiper, ForEach, auto-play |
| 03 | Components | Custom widgets, props, Row, layoutWeight |

## Running

```bash
cargo run -- ark examples/quickstart/01-HelloWorld/aura
```

Output appears in `examples/quickstart/01-HelloWorld/ark/`.

## Generator Features

This sprint added the following features to the ArkTS generator:

- Swiper component support
- Image src prop handling
- ForEach with key function
- Tailwind modifiers: fontFamily, lineHeight, objectFit, layoutWeight
- Swiper modifiers: autoPlay, loop, indicator
- Corner-specific borderRadius
- @Preview decorator for child components
