# Auto Language Website

The official static website for the Auto programming language, built with [VitePress](https://vitepress.dev/).

## Features

- **Modern design** — Clean, gradient-based hero with animated elements
- **Bilingual** — English (default) and Chinese (`/zh/`) via VitePress i18n
- **Docs & Books** — Unified site for project docs and 8 translated books
- **Embedded Playground** — Interactive Auto code editor with CodeMirror 6
- **Auto-generated sidebars** — Parsed from directory structure and SUMMARY.md files

## Development

```bash
# Install dependencies
bun install

# Prepare content (copies docs/ and ../book/ into this directory)
node scripts/prepare-content.js

# Start dev server
bun run dev

# Build for production
bun run build
```

## Content Preparation

The `scripts/prepare-content.js` script:

1. Copies documentation from `../../docs/` into `./docs/` (EN) and `./zh/docs/` (ZH)
2. Copies books from `../../../book/` into `./books/` (EN) and `./zh/books/` (ZH)
3. Generates VitePress sidebar configs from directory structure and `SUMMARY.md` files
4. Preprocesses markdown to fix Vue template compilation issues

## Project Structure

```
website/
├── .vitepress/
│   ├── config.ts              # Main VitePress config
│   ├── config/
│   │   ├── shared.ts          # Shared config
│   │   ├── en.ts              # English locale
│   │   ├── zh.ts              # Chinese locale
│   │   └── sidebar-*.ts       # Auto-generated sidebars
│   ├── theme/
│   │   ├── index.ts           # Theme entry
│   │   ├── style.css          # Tailwind + custom styles
│   │   └── components/
│   │       ├── HomeHero.vue   # Animated hero section
│   │       ├── FeatureCard.vue
│   │       └── AutoPlayground.vue
│   └── tailwind.config.js
├── docs/                      # Generated from ../../docs/
├── books/                     # Generated from ../../../book/
├── zh/
│   ├── docs/                  # Generated Chinese docs
│   ├── books/                 # Generated Chinese books
│   └── index.md               # Chinese homepage
├── index.md                   # English homepage
├── playground.md              # Playground page
├── scripts/
│   └── prepare-content.js     # Content generation script
└── package.json
```

## Deployment

GitHub Actions workflow: `.github/workflows/deploy-website.yml`

Pushes to `main` that modify `website/`, `docs/`, or the workflow file will trigger a build and deployment to GitHub Pages.
