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
│   │       └── FeatureCard.vue
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

## Production Deployment

The production deployment consists of two parts: the **VitePress frontend** (static site) and the **Playground backend** (Rust Axum server).

### Architecture

```
┌─────────────────┐     ┌─────────────┐     ┌─────────────────┐
│   User Browser  │────▶│  nginx:80   │────▶│  VitePress      │
│                 │     │             │     │  Static Files   │
│                 │────▶│  /api/*     │────▶│  Axum Backend   │
└─────────────────┘     │  (proxy)    │     │  :3030          │
                        └─────────────┘     └─────────────────┘
```

### 1. Build the Standalone Playground

The full Playground app is built from `crates/auto-playground/frontend/` and is used in two places:

- `crates/auto-playground/frontend/dist/` — served directly by the Rust backend.
- `website/public/playground/` — deployed as the standalone `/playground` page on the website.

To keep them in sync, use the root build script:

```bash
# From the repository root
npm run build:playground
```

This runs `scripts/build-playground.mjs`, which:

1. Builds the frontend in `crates/auto-playground/frontend/`.
2. Copies the output to `website/public/playground/`.

> **Do not manually edit `website/public/playground/` or rely on a pre-existing `crates/auto-playground/frontend/dist/`.** Both are build artifacts. If `dist/` is stale, the backend will serve an outdated Playground without newer features (e.g. debug/replay buttons).

### 2. Build the Frontend

```bash
cd website

# Install dependencies
bun install

# Prepare content
node scripts/prepare-content.js

# Build for production
bun run build
```

The output is generated in `website/.vitepress/dist/`.

**Important — Playground API URL:**

The `<AutoPlayground>` component from `auto-playground-vue` uses `apiUrl` prop with a default of empty string (`''`). This makes the frontend call `/api/run` and `/api/trans` as **same-origin requests**, which are then reverse-proxied by nginx to the backend.

If you need to point to a different backend during development, pass the prop:

```vue
<AutoPlayground apiUrl="http://localhost:3030" />
```

But for production builds, **keep the default empty string** so nginx handles the proxying.

#### ⚠️ Why `localhost:3030` must not be used in production

If the built static files contain `apiUrl: "http://localhost:3030"`, the user's browser will try to connect to `localhost:3030` **on their own machine** — not the server. This causes the Playground to fail silently with "Could not connect to playground server."

**Verify after deployment:**

```bash
# Check if the built JS still contains localhost
grep -r "localhost:3030" website/.vitepress/dist/
# Should return nothing. If it does, the build used the wrong default.
```

**Emergency fix (if you already deployed with localhost):**

If the build already went out with `localhost:3030` and you cannot rebuild immediately, you can hot-fix the deployed files directly on the server:

```bash
ssh root@112.74.45.241 "python3 -c \"
path = '/home/visus/auto-website/assets/chunks/theme.4JN5UQsF.js'
with open(path, 'r') as f: c = f.read()
c = c.replace('apiUrl:{default:\"http://localhost:3030\"}', 'apiUrl:{default:\"\"}')
with open(path, 'w') as f: f.write(c)
print('fixed')
\""
```

> Note: The exact filename under `assets/chunks/` will vary with each build (hash changes). Find it with `grep -r "localhost:3030" /home/visus/auto-website/`. This is a temporary workaround — always fix the source (`packages/auto-playground-vue/src/AutoPlayground.vue`) and rebuild for the next deployment.

### 3. Build the Backend (Cross-Compilation)

The Playground backend is `crates/auto-playground/`, a Rust Axum server. It serves the static files from `crates/auto-playground/frontend/dist/`, so make sure you have already run `npm run build:playground` from the repo root before starting the backend.

On resource-constrained servers (e.g., 1.6 GB RAM, limited disk), **remote compilation often fails** due to cargo crate downloads and linker OOM. The solution is to build locally and copy the binary.

#### Option A: Build in WSL (Recommended for Windows dev machines)

**Important:** Building on `/mnt/d/` (Windows-mounted drives in WSL) is extremely slow. Clone or copy the repo into WSL's native ext4 filesystem first:

```bash
# Clone to WSL native filesystem (NOT /mnt/d/)
cd ~
git clone <repo-url> auto-lang
cd auto-lang

# Build the release binary (~17 MB)
cargo build -p auto-playground --release

# Output:
# target/release/auto-playground
```

#### Option B: Build on a Linux build machine

```bash
cargo build -p auto-playground --release
```

### 4. Deploy to Server

#### 4.1 Copy Frontend

```bash
# Copy built static files to nginx root
scp -r website/.vitepress/dist/* root@112.74.45.241:/home/visus/auto-website/
```

#### 4.2 Copy Backend Binary

```bash
# Copy the binary to server
scp target/release/auto-playground root@112.74.45.241:/usr/local/bin/auto-playground
```

The server does **not** need Rust installed — the binary is statically linked and self-contained.

#### 4.3 Create systemd Service

Create `/etc/systemd/system/auto-playground.service`:

```ini
[Unit]
Description=Auto Playground Server
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/usr/local/bin
Environment="RUST_LOG=info"
ExecStart=/usr/local/bin/auto-playground
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Start and enable:

```bash
systemctl daemon-reload
systemctl start auto-playground
systemctl enable auto-playground
```

#### 4.4 Configure nginx

Create `/etc/nginx/sites-enabled/auto-playground`:

```nginx
server {
    listen 80;
    server_name 112.74.45.241;

    root /home/visus/auto-website;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    location /api/ {
        proxy_pass http://127.0.0.1:3030;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }

    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2)$ {
        expires 1M;
        add_header Cache-Control "public, immutable";
    }

    gzip on;
    gzip_types text/plain text/css application/json application/javascript text/xml;
}
```

Validate and reload:

```bash
nginx -t
systemctl reload nginx
```

### 5. Verify Deployment

```bash
# Test backend directly
curl -X POST http://127.0.0.1:3030/api/run \
  -H 'Content-Type: application/json' \
  -d '{"source":"fn main() { print(\"hello\") }"}'

# Test via nginx proxy
curl -X POST http://112.74.45.241/api/run \
  -H 'Content-Type: application/json' \
  -d '{"source":"fn main() { print(\"hello\") }"}'
```

Both should return:

```json
{"stdout":"hello\n","result":"","time_ms":30}
```

### 5. Server Requirements

| Component | Requirement |
|-----------|-------------|
| OS | Linux (tested on Ubuntu / Alibaba Cloud Linux) |
| RAM | 512 MB+ for running; 2 GB+ for compiling |
| Disk | 1 GB for binary + website; 10 GB+ for compiling with cargo |
| Runtime | No Rust toolchain needed on server if using prebuilt binary |
| Reverse Proxy | nginx (recommended) |

## GitHub Pages Deployment (Alternative)

GitHub Actions workflow: `.github/workflows/deploy-website.yml`

Pushes to `main` that modify `website/`, `docs/`, or the workflow file will trigger a build and deployment to GitHub Pages. Note that this only deploys the static frontend; the Playground backend is not included.
