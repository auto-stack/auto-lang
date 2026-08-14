//! Vue project generation and build utilities
//!
//! This module provides the complete Vue + shadcn-vue project workflow:
//! 1. Generate project structure (package.json, vite.config.ts, etc.)
//! 2. bun install (or npm install as fallback)
//! 3. Install shadcn-vue components
//! 4. Build (bun run build) or Run dev server (bun run dev)

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use colored::Colorize;
use auto_lang::aura::{AuraRoute, AuraWidget};
use auto_lang::database::{UIArtifact, UIBackend, UICache};
use auto_lang::ui_gen::{BackendGenerator, VueGenerator};

use crate::util::hash_string;
use crate::AutoResult;

/// Recursively copy a directory and all its contents
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Check if shadcn-vue components are already installed
fn are_shadcn_components_installed(output_path: &Path, components: &[String]) -> bool {
    // Check if components.json exists (shadcn-vue config file)
    let components_json = output_path.join("components.json");
    if !components_json.exists() {
        return false;
    }

    // Check if all required component files exist
    for component in components {
        let ui_dir = output_path.join("src/components/ui");

        let component_folder = ui_dir.join(component);
        let pascal_name = component
            .split('-')
            .map(|s| {
                let mut chars = s.chars();
                match chars.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<String>();

        let folder_vue = component_folder.join(format!("{}.vue", pascal_name));
        let folder_index = component_folder.join("index.ts");
        let primitive_ts = ui_dir.join(format!("{}.ts", component));

        if !folder_vue.exists() && !folder_index.exists() && !primitive_ts.exists() {
            return false;
        }
    }
    true
}

/// Detect which shadcn-vue components are needed from generated Vue code
fn detect_shadcn_components(vue_code: &str) -> Vec<String> {
    let mut components = HashSet::new();

    let component_patterns = [
        ("@/components/ui/button", "button"),
        ("@/components/ui/input", "input"),
        ("@/components/ui/textarea", "textarea"),
        ("@/components/ui/checkbox", "checkbox"),
        ("@/components/ui/switch", "switch"),
        ("@/components/ui/select", "select"),
        ("@/components/ui/tabs", "tabs"),
        ("@/components/ui/dialog", "dialog"),
        ("@/components/ui/tooltip", "tooltip"),
        ("@/components/ui/slider", "slider"),
        ("@/components/ui/radio-group", "radio-group"),
        ("@/components/ui/progress", "progress"),
        ("@/components/ui/badge", "badge"),
        ("@/components/ui/skeleton", "skeleton"),
        ("@/components/ui/card", "card"),
        ("@/components/ui/avatar", "avatar"),
        ("@/components/ui/table", "table"),
        ("@/components/ui/separator", "separator"),
        ("@/components/ui/scroll-area", "scroll-area"),
        ("@/components/ui/label", "label"),
        ("@/components/ui/alert", "alert"),
        ("@/components/ui/sonner", "sonner"),
        ("@/components/ui/dropdown-menu", "dropdown-menu"),
        ("@/components/ui/popover", "popover"),
        ("@/components/ui/sheet", "sheet"),
        ("@/components/ui/breadcrumb", "breadcrumb"),
        ("@/components/ui/accordion", "accordion"),
        ("@/components/ui/alert-dialog", "alert-dialog"),
        ("@/components/ui/command", "command"),
        ("@/components/ui/form", "form"),
        ("@/components/ui/navigation-menu", "navigation-menu"),
        ("@/components/ui/sidebar", "sidebar"),
        ("@/components/ui/stepper", "stepper"),
        ("@/components/ui/calendar", "calendar"),
        ("@/components/ui/carousel", "carousel"),
        ("@/components/ui/combobox", "combobox"),
        ("@/components/ui/context-menu", "context-menu"),
        ("@/components/ui/drawer", "drawer"),
        ("@/components/ui/hover-card", "hover-card"),
        ("@/components/ui/number-field", "number-field"),
        ("@/components/ui/pagination", "pagination"),
        ("@/components/ui/pin-input", "pin-input"),
        ("@/components/ui/tags-input", "tags-input"),
        ("@/components/ui/toggle-group", "toggle-group"),
        ("@/components/ui/aspect-ratio", "aspect-ratio"),
        ("@/components/ui/button-group", "button-group"),
        ("@/components/ui/chart", "chart"),
        ("@/components/ui/chart-area", "chart-area"),
        ("@/components/ui/chart-bar", "chart-bar"),
        ("@/components/ui/chart-line", "chart-line"),
        ("@/components/ui/chart-donut", "chart-donut"),
        ("@/components/ui/collapsible", "collapsible"),
        ("@/components/ui/input-group", "input-group"),
        ("@/components/ui/input-otp", "input-otp"),
        ("@/components/ui/kbd", "kbd"),
        ("@/components/ui/menubar", "menubar"),
        ("@/components/ui/native-select", "native-select"),
        ("@/components/ui/range-calendar", "range-calendar"),
        ("@/components/ui/resizable", "resizable"),
        ("@/components/ui/auto-complete", "auto-complete"),
    ];

    for (pattern, component) in component_patterns {
        if vue_code.contains(pattern) {
            components.insert(component.to_string());
        }
    }

    let mut result: Vec<String> = components.into_iter().collect();
    result.sort();
    result
}

// Template generators

fn generate_package_json(
    name: &str,
    has_routes: bool,
    i18n_enabled: bool,
    extra_deps: &[(String, String)],
) -> String {
    let router_dep = if has_routes {
        r#"    "vue-router": "^4.2.0",
"#
    } else {
        ""
    };

    // Plan musk-022 Phase 2: vue-i18n dependency when i18n is enabled.
    let i18n_dep = if i18n_enabled {
        r#"    "vue-i18n": "^9.14.0",
"#
    } else {
        ""
    };

    // Build extra deps lines from pac.at npm_deps.
    // Each entry is (package_name, version_spec) where version_spec may be
    // "^1.0.0", "latest", "link:/path", "file:../path", etc.
    let extra_lines: String = extra_deps.iter().map(|(pkg, ver)| {
        format!("    \"{}\": \"{}\",\n", pkg, ver)
    }).collect();

    format!(r#"{{
  "name": "{}",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {{
    "dev": "vite",
    "build": "vue-tsc && vite build",
    "preview": "vite preview"
  }},
  "dependencies": {{
    "vue": ">=3.4.0 <3.5.36",
{}{}{}    "@vueuse/core": "^10.7.0",
    "reka-ui": "^2.0.0",
    "class-variance-authority": "^0.7.0",
    "clsx": "^2.1.0",
    "tailwind-merge": "^2.2.0",
    "lucide-vue-next": "^0.312.0",
    "prismjs": "^1.29.0",
    "embla-carousel-vue": "^8.5.1",
    "vaul-vue": "^0.4.1",
    "vue-sonner": "^2.0.9",
    "vee-validate": "^4.15.1",
    "@vee-validate/zod": "^4.15.1",
    "zod": "^3.25.76"
  }},
  "devDependencies": {{
    "@vitejs/plugin-vue": "^5.0.0",
    "vite": "^5.0.0",
    "typescript": "^5.3.0",
    "vue-tsc": "^2.0.0",
    "tailwindcss": "^3.4.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0",
    "tailwindcss-animate": "^1.0.7",
    "@types/prismjs": "^1.26.0"
  }}
}}
"#, name, router_dep, i18n_dep, extra_lines)
}

fn generate_vite_config() -> String {
    // AUTO_HTTP_PORT lets multiple `auto run` instances coexist; default 8080.
    let _proxy_target = format!("http://127.0.0.1:{}", crate::util::http_port());
    r#"import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  base: './',
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
  build: {
    rollupOptions: {
      output: {
        entryFileNames: 'assets/index.js',
        chunkFileNames: 'assets/[name].js',
        assetFileNames: 'assets/[name].[ext]',
      },
    },
  },
  server: {
    // AUTO_FRONT_PORT (default 3000) lets multiple `auto run` instances coexist.
    port: Number(process.env.AUTO_FRONT_PORT || 3000),
    // Only auto-open browser when NOT running under Tauri
    // Tauri sets TAURI_ENV before running vite
    open: !process.env.TAURI_ENV,
    // Proxy API requests to Rust backend.
    // Read the backend port at RUNTIME from AUTO_HTTP_PORT (set by `auto run -B`),
    // so the proxy target updates without regenerating vite.config.ts.
    proxy: {
      '/api': {
        target: process.env.AUTO_HTTP_PROXY || `http://127.0.0.1:${process.env.AUTO_HTTP_PORT || __PROXY_PORT__}`,
        changeOrigin: true,
      }
    }
  }
})
"#.replace("__PROXY_PORT__", &crate::util::http_port().to_string())
}

fn generate_tsconfig() -> String {
    // Plan 053 M4: ES2020 → ES2021 — the codegen maps `.at` `replace` to
    // JS `replaceAll` (Rust str::replace is full-replace), which needs the
    // es2021 lib for vue-tsc. WebView2/Chromium supports it (Chrome 85+).
    r#"{
  "compilerOptions": {
    "target": "ES2021",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "lib": ["ES2021", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "preserve",
    "strict": true,
    "noUnusedLocals": false,
    "noUnusedParameters": false,
    "noFallthroughCasesInSwitch": true,
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src/**/*.ts", "src/**/*.tsx", "src/**/*.vue"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
"#.to_string()
}

fn generate_tsconfig_node() -> String {
    r#"{
  "compilerOptions": {
    "composite": true,
    "skipLibCheck": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowSyntheticDefaultImports": true
  },
  "include": ["vite.config.ts"]
}
"#.to_string()
}

fn generate_tailwind_config() -> String {
    r#"/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: ["class"],
  content: [
    './index.html',
    './src/**/*.{ts,tsx,vue}',
  ],
  theme: {
    container: {
      center: true,
      padding: "2rem",
      screens: {
        "2xl": "1400px",
      },
    },
    extend: {
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        popover: {
          DEFAULT: "hsl(var(--popover))",
          foreground: "hsl(var(--popover-foreground))",
        },
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
      keyframes: {
        "accordion-down": {
          from: { height: 0 },
          to: { height: "var(--reka-accordion-content-height)" },
        },
        "accordion-up": {
          from: { height: "var(--reka-accordion-content-height)" },
          to: { height: 0 },
        },
        "collapsible-down": {
          from: { height: 0 },
          to: { height: "var(--reka-collapsible-content-height)" },
        },
        "collapsible-up": {
          from: { height: "var(--reka-collapsible-content-height)" },
          to: { height: 0 },
        },
      },
      animation: {
        "accordion-down": "accordion-down 0.2s ease-out",
        "accordion-up": "accordion-up 0.2s ease-out",
        "collapsible-down": "collapsible-down 0.2s ease-out",
        "collapsible-up": "collapsible-up 0.2s ease-out",
      },
    },
  },
  plugins: [require("tailwindcss-animate")],
}
"#.to_string()
}

fn generate_postcss_config() -> String {
    r#"module.exports = {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}
"#.to_string()
}

fn generate_index_html(name: &str) -> String {
    // Plan 043 M5: the shadcn template ships fully-populated `.dark` tokens
    // in index.css; the handwritten ash-gui (and the shadcn default) render
    // dark. Without `class="dark"` on <html> the app falls back to the light
    // `:root` tokens and looks broken (light bg + dark-designed text).
    format!(r#"<!DOCTYPE html>
<html lang="en" class="dark">
  <head>
    <meta charset="UTF-8">
    <link rel="icon" href="/favicon.ico">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
"#, name)
}

fn generate_main_ts(
    has_routes: bool,
    uses_autodown: bool,
    style_files: &[String],
    i18n: &I18nConfig,
    locale_files: &[String],
) -> String {
    let autodown_css = if uses_autodown {
        "\nimport '@autodown/editor/style.css'"
    } else {
        ""
    };
    // pac.at `styles:` files — copied verbatim into src/styles/ and imported
    // here so Vite bundles them. Content is never modified.
    let style_imports: String = style_files
        .iter()
        .map(|f| format!("\nimport './styles/{}'", f))
        .collect();
    // Plan musk-022 Phase 2: i18n imports + createI18n. Locale files (copied
    // into src/locales/) are imported by basename and assembled into messages.
    // Each locale's language key is derived from its filename stem (e.g.
    // `en.json` → `en`). When locale_files is empty, an empty messages object
    // is used (caller is expected to populate it later).
    let (i18n_imports, i18n_setup): (String, String) = if i18n.enabled {
        let locale_imports: String = locale_files
            .iter()
            .map(|f| {
                let stem = std::path::Path::new(f)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("locale");
                format!("\nimport {} from './locales/{}'", stem, basename(f))
            })
            .collect();
        let messages_entries: String = locale_files
            .iter()
            .map(|f| {
                let stem = std::path::Path::new(f)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("locale");
                format!("    {},\n", stem)
            })
            .collect();
        let setup = format!(
            "\nimport {{ createI18n }} from 'vue-i18n'{locale_imports}\n\n\
const i18n = createI18n({{\n  legacy: false,\n  locale: {default_locale:?},\n\
  messages: {{\n{messages_entries}  }},\n}})\n",
            locale_imports = locale_imports,
            default_locale = locale_files
                .first()
                .and_then(|f| std::path::Path::new(f).file_stem().and_then(|s| s.to_str()))
                .unwrap_or("en"),
            messages_entries = messages_entries,
        );
        (String::new(), setup)
    } else {
        (String::new(), String::new())
    };
    let base = format!(
        r#"import {{ createApp }} from 'vue'
import App from './App.vue'
import './assets/index.css'{autodown_css}{style_imports}
import 'prismjs/themes/prism-tomorrow.css'
import Prism from 'prismjs'

// Define custom 'auto' language for Prism
Prism.languages.auto = {{
  'comment': /\/\/.*|\/\*[\s\S]*?\*\//,
  'string': {{
    pattern: /f?"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'/,
    greedy: true
  }},
  'keyword': /\b(?:widget|view|model|msg|fn|let|mut|const|if|else|for|in|return|use|type|spec|import|export|struct|enum|interface|extends|implements|new|true|false|null)\b/,
  'function': /\b[a-z_][a-z0-9_]*(?=\s*\()/i,
  'number': /\b\d+\.?\d*\b/,
  'operator': /[+\-*/%=<>!&|^~?:]+/,
  'punctuation': /[{{}}[\]();,.]/,
  'property': /\.[a-z_][a-z0-9_]*/i,
  'element': /\b(?:col|row|button|text|input|card|link|div|span|p|h1|h2|h3|h4|h5|h6|ul|ol|li|table|thead|tbody|tr|td|th|form|label|checkbox|switch|select|option|dialog|modal|toast|dropdown|menu|tab|tabs|accordion|badge|avatar|progress|slider|scroll|codeblock|pre|code|img|video|audio|canvas|svg|path|rect|circle|ellipse|line|polyline|polygon|header|footer|nav|main|aside|section|article|header|footer|sidebar|outlet|slot)\b/,
  'attr': /\([^)]*\)/,
}};
"#,
        autodown_css = autodown_css,
        style_imports = style_imports
    );
    // Plan musk-022 Phase 2: unify the app construction so i18n + router can
    // both be `.use()`'d. Previously the non-route branch used a one-liner
    // `createApp(App).mount('#app')` which couldn't accept `app.use(i18n)`.
    let router_import = if has_routes {
        "\nimport router from './router'"
    } else {
        ""
    };
    let app_use_router = if has_routes { "app.use(router)\n" } else { "" };
    let app_use_i18n = if i18n.enabled { "app.use(i18n)\n" } else { "" };
    format!(
        "{base}{i18n_setup}{router_import}\n\nconst app = createApp(App)\n{app_use_i18n}{app_use_router}app.mount('#app')\n",
        base = base,
        i18n_setup = i18n_setup,
        router_import = router_import,
        app_use_i18n = app_use_i18n,
        app_use_router = app_use_router,
    )
}

/// Plan musk-022 Phase 2: return the file name (last path component) of a
/// relative path, e.g. `src/i18n/locales/en.json` → `en.json`. Used to name
/// copied locale files inside `src/locales/`.
fn basename(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_string()
}

fn generate_app_vue(vue_code: &str) -> String {
    vue_code.to_string()
}

fn generate_index_css() -> String {
    r#"@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  :root {
    --background: 0 0% 100%;
    --foreground: 222.2 84% 4.9%;

    --card: 0 0% 100%;
    --card-foreground: 222.2 84% 4.9%;

    --popover: 0 0% 100%;
    --popover-foreground: 222.2 84% 4.9%;

    --primary: 239 84% 67%;
    --primary-foreground: 210 40% 98%;

    --secondary: 210 40% 96.1%;
    --secondary-foreground: 222.2 47.4% 11.2%;

    --muted: 210 40% 96.1%;
    --muted-foreground: 215.4 16.3% 46.9%;

    --accent: 210 40% 96.1%;
    --accent-foreground: 222.2 47.4% 11.2%;

    --destructive: 0 84.2% 60.2%;
    --destructive-foreground: 210 40% 98%;

    --border: 214.3 31.8% 91.4%;
    --input: 214.3 31.8% 91.4%;
    --ring: 239 84% 67%;

    --radius: 0.5rem;
  }

  .dark {
    --background: 222.2 47% 7%;
    --foreground: 210 40% 98%;

    --card: 222.2 47% 10%;
    --card-foreground: 210 40% 98%;

    --popover: 222.2 47% 10%;
    --popover-foreground: 210 40% 98%;

    --primary: 239 84% 77%;
    --primary-foreground: 222.2 47.4% 11.2%;

    --secondary: 217.2 32.6% 17.5%;
    --secondary-foreground: 210 40% 98%;

    --muted: 217.2 32.6% 15%;
    --muted-foreground: 215 20.2% 65.1%;

    --accent: 217.2 32.6% 17.5%;
    --accent-foreground: 210 40% 98%;

    --destructive: 0 62.8% 30.6%;
    --destructive-foreground: 210 40% 98%;

    --border: 217.2 32.6% 17.5%;
    --input: 217.2 32.6% 17.5%;
    --ring: 239 84% 77%;
  }
}

@layer base {
  * {
    @apply border-border;
  }
  html, body, #app {
    height: 100%;
    margin: 0;
  }
  body {
    @apply bg-background text-foreground;
    transition: background-color 0.3s ease, color 0.3s ease;
  }
}

/* Plan 053 后续: AutoUI 风格(细 / 半透明 / 圆角)的原生滚动条。
   reka-ui ScrollArea 在其 viewport 上隐藏原生滚动条并自绘 ScrollBar,但自绘
   bar 需「确定高度」才能检测溢出 —— 与 max-h(限高)不兼容(block 输出限高时
   reka-ui bar 不显示 → 看不见)。故限高容器用原生 overflow-y-auto + 此
   .ash-scroll 样式,视觉与 reka-ui bar 一致(用 --border,随明暗主题)。
   仅作用于带 .ash-scroll 的元素,不干扰 reka-ui 的 ScrollArea。 */
.ash-scroll { scrollbar-width: thin; scrollbar-color: hsl(var(--border)) transparent; }
.ash-scroll::-webkit-scrollbar { width: 8px; height: 8px; }
.ash-scroll::-webkit-scrollbar-track { background: transparent; }
.ash-scroll::-webkit-scrollbar-thumb { background-color: hsl(var(--border)); border-radius: 9999px; }
.ash-scroll::-webkit-scrollbar-thumb:hover { background-color: hsl(var(--muted-foreground)); }
"#.to_string()
}

fn generate_utils_ts() -> String {
    r#"import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}
"#.to_string()
}

/// Build approvals for esbuild + vue-demi are declared in the `pnpm` field of
/// `package.json` (see [`generate_package_json`]), which pnpm 10/11 reads
/// directly.
///
/// Plan 328: Configure pnpm build approvals via `pnpm-workspace.yaml`.
///
/// pnpm v10+ blocks postinstall build scripts (esbuild, vue-demi, …) unless
/// they are explicitly approved.
///
/// **Format matters by version:**
/// - pnpm v10: reads `onlyBuiltDependencies:` (a YAML list) from
///   `pnpm-workspace.yaml`.
/// - pnpm v11: reads `allowBuilds:` (a YAML map of `name: true/false`) from
///   `pnpm-workspace.yaml`. It does **not** honor `.npmrc`'s
///   `only-built-dependencies[]` for this.
///
/// We write **both** keys so the file works under either major version. We
/// must also set `packages: []` so pnpm accepts the file as a valid workspace
/// manifest (required even though this is a single-package project; `pnpm add`
/// works fine under it).
///
/// Crucially, the values are real booleans (`true`), not the placeholder
/// string `"set this to true or false"` that pnpm v11's *interactive*
/// `approve-builds` writes when stdin has no answer in a non-interactive
/// context — that placeholder is what caused the cascading failures.
fn ensure_pnpm_build_approvals(dir: &Path) -> bool {
    let yaml_path = dir.join("pnpm-workspace.yaml");

    // Build the set of approved deps, starting from the defaults we always want.
    let mut deps: Vec<String> = vec!["esbuild".to_string(), "vue-demi".to_string()];

    // Preserve any approvals already present in the file (added by other tools
    // or the user) so we never drop a needed approval.
    if let Ok(existing) = fs::read_to_string(&yaml_path) {
        // pnpm v10 list form: "- name"
        let mut in_list = false;
        for line in existing.lines() {
            let t = line.trim();
            if t.starts_with("onlyBuiltDependencies:") {
                in_list = true;
                continue;
            }
            if in_list {
                if let Some(name) = t.strip_prefix("- ") {
                    let name = name.trim().to_string();
                    if !name.is_empty() && !deps.iter().any(|d| d == &name) {
                        deps.push(name);
                    }
                } else if !t.is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
                    in_list = false;
                }
            }
        }
        // pnpm v11 map form: "  name: true"
        for line in existing.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_suffix(": true") {
                let name = rest.trim().to_string();
                if !name.is_empty() && !name.contains(' ') && !deps.iter().any(|d| d == &name) {
                    deps.push(name);
                }
            }
        }
    }

    let mut content = String::from("packages: []\n");
    // pnpm v10 form
    content.push_str("onlyBuiltDependencies:\n");
    for d in &deps {
        content.push_str("  - ");
        content.push_str(d);
        content.push('\n');
    }
    // pnpm v11 form (real booleans, not placeholders)
    content.push_str("allowBuilds:\n");
    for d in &deps {
        content.push_str(&format!("  {}: true\n", d));
    }
    // Plan 346: Disable pnpm 11.x supply-chain minimum-release-age check.
    // Without this, freshly-published transitive deps (caniuse-lite, etc.)
    // are rejected with ERR_PNPM_MINIMUM_RELEASE_AGE_VIOLATION.
    content.push_str("minimumReleaseAge: 0\n");
    match fs::write(&yaml_path, content) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Write all project files
fn write_project_files(
    output_path: &Path,
    name: &str,
    vue_code: &str,
    _components: &[String],
    has_routes: bool,
    extra_deps: &[(String, String)],
    style_files: &[String],
    i18n: &I18nConfig,
    locale_files: &[String],
) -> Result<(), String> {
    // package.json
    let package_json = generate_package_json(name, has_routes, i18n.enabled, extra_deps);
    fs::write(output_path.join("package.json"), package_json)
        .map_err(|e| format!("Failed to write package.json: {}", e))?;

    // Plan 328: Write pnpm-workspace.yaml with build approvals (pnpm v10+).
    // Writes both onlyBuiltDependencies (v10) and allowBuilds (v11) forms.
    ensure_pnpm_build_approvals(output_path);

    // .npmrc — basic pnpm settings.
    // `ignore-workspace-root-check=true` lets `pnpm add` (run internally by
    // `shadcn-vue add`) write to the workspace root instead of failing with
    // ERR_PNPM_ADDING_TO_ROOT. `verify-deps-before-run=false` stops pnpm from
    // re-running install (and re-triggering build approval) before `vite`.
    // `minimum-release-age=0` disables pnpm's supply-chain minimum-age check
    // which blocks freshly-published transitive deps (caniuse-lite etc).
    fs::write(output_path.join(".npmrc"),
        "manage-package-manager-versions=true\nverify-deps-before-run=false\nignore-workspace-root-check=true\nminimum-release-age=0\n")
        .map_err(|e| format!("Failed to write .npmrc: {}", e))?;

    // components.json (shadcn-vue config)
    let components_json = auto_lang::ui_gen::VueGenerator::generate_components_json();
    fs::write(output_path.join("components.json"), components_json)
        .map_err(|e| format!("Failed to write components.json: {}", e))?;

    // vite.config.ts
    let vite_config = generate_vite_config();
    fs::write(output_path.join("vite.config.ts"), vite_config)
        .map_err(|e| format!("Failed to write vite.config.ts: {}", e))?;

    // tsconfig.json
    let tsconfig = generate_tsconfig();
    fs::write(output_path.join("tsconfig.json"), tsconfig)
        .map_err(|e| format!("Failed to write tsconfig.json: {}", e))?;

    // tsconfig.node.json
    let tsconfig_node = generate_tsconfig_node();
    fs::write(output_path.join("tsconfig.node.json"), tsconfig_node)
        .map_err(|e| format!("Failed to write tsconfig.node.json: {}", e))?;

    // tailwind.config.cjs
    let tailwind_config = generate_tailwind_config();
    fs::write(output_path.join("tailwind.config.cjs"), tailwind_config)
        .map_err(|e| format!("Failed to write tailwind.config.cjs: {}", e))?;

    // postcss.config.cjs
    let postcss_config = generate_postcss_config();
    fs::write(output_path.join("postcss.config.cjs"), postcss_config)
        .map_err(|e| format!("Failed to write postcss.config.cjs: {}", e))?;

    // index.html
    let index_html = generate_index_html(name);
    fs::write(output_path.join("index.html"), index_html)
        .map_err(|e| format!("Failed to write index.html: {}", e))?;

    // src/main.ts
    let uses_autodown = extra_deps.iter().any(|(name, _)| name == "@autodown/editor");
    let main_ts = generate_main_ts(has_routes, uses_autodown, style_files, i18n, locale_files);
    fs::write(output_path.join("src/main.ts"), main_ts)
        .map_err(|e| format!("Failed to write src/main.ts: {}", e))?;

    // src/App.vue
    let app_vue = generate_app_vue(vue_code);
    fs::write(output_path.join("src/App.vue"), app_vue)
        .map_err(|e| format!("Failed to write src/App.vue: {}", e))?;

    // src/assets/index.css
    let index_css = generate_index_css();
    fs::write(output_path.join("src/assets/index.css"), index_css)
        .map_err(|e| format!("Failed to write src/assets/index.css: {}", e))?;

    // src/lib/utils.ts
    let utils_ts = generate_utils_ts();
    fs::write(output_path.join("src/lib/utils.ts"), utils_ts)
        .map_err(|e| format!("Failed to write src/lib/utils.ts: {}", e))?;

    Ok(())
}

/// Parse workspace path from pac.at content
///
/// Plan 129: Supports two syntaxes:
/// 1. app("front") {} - source in ./front/ (implied by name)
/// 2. front: "./source/front" - explicit path (legacy)
/// Resolve the front directory for a workspace root.
/// Checks src/front, source/front, front — matching VueProject::from_workspace logic.
fn resolve_front_dir(root_dir: &Path) -> std::path::PathBuf {
    if root_dir.join("src").join("front").exists() {
        root_dir.join("src").join("front")
    } else if root_dir.join("source").join("front").exists() {
        root_dir.join("source").join("front")
    } else if root_dir.join("front").exists() {
        root_dir.join("front")
    } else {
        root_dir.join("src").join("front")
    }
}

fn parse_workspace_path(content: &str, key: &str) -> Option<String> {
    // First, look for app("key") syntax (Plan 129)
    // Pattern: app("front") or app("back")
    let app_pattern = format!("app(\"{}\")", key);
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with(&app_pattern) {
            // app("front") implies source directory is "./front"
            return Some(format!("./{}", key));
        }
    }

    // Fallback: Look for explicit path: front: "./source/front"
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with(&format!("{}:", key)) {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..].trim();
                let value = value.trim_matches('"').trim_matches('\'');
                let value = value.trim_end_matches(',');
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Parse project name from pac.at content
fn parse_pac_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name:") {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..].trim();
                let value = value.trim_matches('"').trim_matches('\'');
                let value = value.trim_end_matches(',');
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Parse npm_deps from pac.at content.
///
/// Returns a list of (package_name, version_spec) pairs where version_spec
/// is the string written into package.json (e.g. "^1.0.0", "latest",
/// "link:D:/path", "file:../path").
///
/// Supports three syntaxes:
///
/// 1. **Array** (inline): `npm_deps: ["@autodown/editor", "marked@^12.0.0"]`
/// 2. **Object** (link/file paths):
///    ```text
///    npm_deps: {
///      "@autodown/editor": {
///        link: "D:/autostack/auto-down/autodown/packages/editor"
///      }
///    }
///    ```
///    Also supports shorthand:
///    ```text
///    npm_deps: {
///      "marked": "^12.0.0",
///      "@autodown/editor": "link:D:/path/to/editor"
///    }
///    ```
/// 3. **Single string**: `npm_deps: "@autodown/editor"`
fn parse_npm_deps(content: &str) -> Vec<(String, String)> {
    let mut deps = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.starts_with("npm_deps:") {
            let rest = line["npm_deps:".len()..].trim();
            if rest.starts_with('{') {
                // Object syntax — parse key/value pairs across multiple lines
                let mut j = i + 1;
                while j < lines.len() {
                    let next = lines[j].trim();
                    // End of object
                    if next.starts_with('}') {
                        break;
                    }
                    // Skip blank lines
                    if next.is_empty() {
                        j += 1;
                        continue;
                    }
                    // Try to parse a key (quoted package name)
                    if let Some(key_end) = find_quoted_string_end(next) {
                        let pkg = next[..key_end].trim_matches('"').trim_matches('\'');
                        let after_key = next[key_end..].trim();
                        if after_key.starts_with(':') {
                            let value_part = after_key[1..].trim();
                            if value_part.starts_with('{') {
                                // Nested object: { link: "path" } or { file: "path" }
                                // Parse the inner key:value on this line or next lines
                                let spec = parse_dep_object_spec(&lines, &mut j, value_part);
                                if !pkg.is_empty() && !spec.is_empty() {
                                    deps.push((pkg.to_string(), spec));
                                }
                            } else {
                                // Shorthand string value: "pkg": "^1.0.0" or "pkg": "link:path"
                                let ver = value_part.trim_matches('"').trim_matches('\'').trim_end_matches(',');
                                if !pkg.is_empty() && !ver.is_empty() {
                                    deps.push((pkg.to_string(), ver.to_string()));
                                }
                            }
                        }
                    }
                    j += 1;
                }
            } else if rest.starts_with('[') {
                // Inline array: ["a", "b"] or ["pkg@^1.0.0"]
                let value = rest.trim_start_matches('[').trim_end_matches(']');
                for part in value.split(',') {
                    let dep = part.trim().trim_matches('"').trim_matches('\'').trim();
                    if !dep.is_empty() {
                        let (pkg, ver) = split_pkg_version(dep);
                        deps.push((pkg, ver));
                    }
                }
            } else if rest.starts_with('"') || rest.starts_with('\'') {
                // Single string: "package"
                let dep = rest.trim_matches('"').trim_matches('\'').trim_end_matches(',');
                if !dep.is_empty() {
                    let (pkg, ver) = split_pkg_version(dep);
                    deps.push((pkg, ver));
                }
            } else {
                // Multi-line Auto-style: each following indented line is a dep
                let mut j = i + 1;
                while j < lines.len() {
                    let next = lines[j];
                    if next.trim().is_empty() || (!next.starts_with(' ') && !next.starts_with('\t')) {
                        break;
                    }
                    let dep = next.trim().trim_matches('"').trim_matches('\'').trim_end_matches(',');
                    if !dep.is_empty() {
                        let (pkg, ver) = split_pkg_version(dep);
                        deps.push((pkg, ver));
                    }
                    j += 1;
                }
            }
            break;
        }
        i += 1;
    }
    deps
}

/// Parse `styles` from pac.at content — project-level native CSS files to
/// copy verbatim into the generated Vue project.
///
/// Returns the declared paths (relative to the pac.at directory).
///
/// Supported syntaxes:
/// 1. **Inline array**: `styles: ["src/front/autodown-editor.css", "src/front/theme.css"]`
/// 2. **Single string**: `styles: "src/front/autodown-editor.css"`
fn parse_style_files(content: &str) -> Vec<String> {
    let mut files = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("styles:") {
            let rest = rest.trim();
            if rest.starts_with('[') {
                // Inline array: ["a.css", "b.css"]
                let value = rest.trim_start_matches('[').trim_end_matches(']');
                for part in value.split(',') {
                    let f = part.trim().trim_matches('"').trim_matches('\'').trim();
                    if !f.is_empty() {
                        files.push(f.to_string());
                    }
                }
            } else if rest.starts_with('"') || rest.starts_with('\'') {
                // Single string: "a.css"
                let f = rest.trim_matches('"').trim_matches('\'').trim_end_matches(',');
                if !f.is_empty() {
                    files.push(f.to_string());
                }
            }
            break;
        }
    }
    files
}

/// Plan musk-022 Phase 2: i18n (vue-i18n) configuration parsed from pac.at.
/// When `enabled`, the generated project gets:
///   - `vue-i18n` added to package.json dependencies
///   - `createI18n({ messages }) + app.use(i18n)` injected into main.ts
///   - locale files (in `locale_files`) copied byte-for-byte into `src/locales/`
///     and imported as the i18n `messages`.
/// When `enabled` is false, no i18n machinery is emitted (default, backward
/// compatible). `locale_files` may be empty when `i18n: true` is set without
/// paths — in that case an empty messages object is used.
#[derive(Debug, Clone, Default)]
pub struct I18nConfig {
    pub enabled: bool,
    /// Locale files (relative to root_dir) to copy into `src/locales/`.
    /// e.g. `["src/i18n/locales/en.json", "src/i18n/locales/zh.json"]`.
    pub locale_files: Vec<String>,
}

/// Plan musk-022 Phase 2: parse the `i18n` field from pac.at content.
/// Recognized forms:
///   - `i18n: true`               → enabled, no locale files (inline messages)
///   - `i18n: "path/en.json"`     → enabled, single locale file
///   - `i18n: ["en.json", ...]`   → enabled, multiple locale files
/// Absent / other values → disabled (default).
fn parse_i18n(content: &str) -> I18nConfig {
    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("i18n:") {
            let rest = rest.trim().trim_end_matches(',');
            if rest == "true" {
                return I18nConfig { enabled: true, locale_files: vec![] };
            } else if rest.starts_with('[') {
                let value = rest.trim_start_matches('[').trim_end_matches(']');
                let files: Vec<String> = value
                    .split(',')
                    .map(|p| p.trim().trim_matches('"').trim_matches('\'').trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                return I18nConfig { enabled: true, locale_files: files };
            } else if rest.starts_with('"') || rest.starts_with('\'') {
                let f = rest.trim_matches('"').trim_matches('\'').to_string();
                if !f.is_empty() {
                    return I18nConfig { enabled: true, locale_files: vec![f] };
                }
            }
            break;
        }
    }
    I18nConfig::default()
}

/// True when a widget `use { ... }` import path refers to a project-local
/// file (copied into `src/ext/`) rather than an npm package specifier.
/// Mirrors `VueGenerator::ext_is_local_path` in auto-lang — the two must
/// agree so the emitted `@/ext/...` specifier matches the copied location.
fn is_local_ext_path(path: &str) -> bool {
    path.starts_with('.')
        || path.starts_with('/')
        || path.ends_with(".vue")
        || path.ends_with(".ts")
        || path.ends_with(".tsx")
        || path.ends_with(".js")
        || path.ends_with(".mjs")
}

/// Collect project-local file paths declared in widget `use { ... }`
/// import blocks into `out` (normalized, deduped, project-root-relative).
fn collect_ext_import_files(widgets: &[AuraWidget], out: &mut std::collections::BTreeSet<String>) {
    for widget in widgets {
        for imp in &widget.ext_imports {
            let path = imp.path.as_str();
            if is_local_ext_path(path) {
                let normalized = path.trim_start_matches("./").trim_start_matches('/');
                out.insert(normalized.to_string());
            }
        }
    }
}

/// Split a dep spec into (package_name, version_spec).
///
/// Supports three formats:
/// - `"package"` → ("package", "latest")
/// - `"package@^1.0.0"` → ("package", "^1.0.0")  (scoped-aware)
/// - `"package:link:/path/to/pkg"` → ("package", "link:/path/to/pkg")
/// - `"package:file:../pkg"` → ("package", "file:../pkg")
fn split_pkg_version(dep: &str) -> (String, String) {
    // Check for :link: or :file: suffix first (local path deps)
    for sep in &[":link:", ":file:"] {
        if let Some(pos) = dep.find(sep) {
            let pkg = dep[..pos].to_string();
            // dep = "pkg:link:/path", pos points at ":link:"
            // sep[1..] = "link:", dep[pos+sep.len()..] = "/path"
            let spec = format!("{}{}", &sep[1..], &dep[pos + sep.len()..]);
            return (pkg, spec);
        }
    }
    // Version via @ separator
    if dep.starts_with('@') {
        if let Some(pos) = dep[1..].find('@') {
            (dep[..pos + 1].to_string(), dep[pos + 2..].to_string())
        } else {
            (dep.to_string(), "latest".to_string())
        }
    } else if let Some(pos) = dep.find('@') {
        (dep[..pos].to_string(), dep[pos + 1..].to_string())
    } else {
        (dep.to_string(), "latest".to_string())
    }
}

/// Find the end index of a quoted string at the start of `s`.
fn find_quoted_string_end(s: &str) -> Option<usize> {
        let bytes = s.as_bytes();
        if bytes.is_empty() {
            return None;
        }
        let quote = bytes[0];
        if quote != b'"' && quote != b'\'' {
            return None;
        }
        for i in 1..bytes.len() {
            if bytes[i] == quote {
                return Some(i + 1);
            }
        }
        None
    }

    /// Parse a nested dep object like `{ link: "path" }` or `{ file: "path" }`.
    /// Returns the version spec string (e.g. "link:path" or "file:path").
/// Parse a nested dep object like `{ link: "path" }` or `{ file: "path" }`.
/// Returns the version spec string (e.g. "link:path" or "file:path").
fn parse_dep_object_spec(lines: &[&str], j: &mut usize, inline: &str) -> String {
        // Check if the object closes on the same line
        if inline.contains('}') {
            // Single-line: { link: "path" }
            for kind in &["link", "file"] {
                if let Some(pos) = inline.find(kind) {
                    let after = &inline[pos + kind.len()..];
                    // Find the quoted value
                    let val_start = after.find('"').or_else(|| after.find('\''));
                    if let Some(start) = val_start {
                        let q = &after[start..start + 1];
                        let val = &after[start + 1..];
                        let end = val.find(q).unwrap_or(val.len());
                        let path = &val[..end];
                        return format!("{}:{}", kind, path);
                    }
                }
            }
            return String::new();
        }
        // Multi-line: scan subsequent lines for link:/file: key
        let mut k = *j + 1;
        while k < lines.len() {
            let next = lines[k].trim();
            if next.starts_with('}') {
                *j = k; // consume up to closing brace
                break;
            }
            for kind in &["link", "file"] {
                if next.starts_with(kind) {
                    let after = &next[kind.len()..].trim();
                    let val = after.trim_matches('"').trim_matches('\'').trim_end_matches(',');
                    if !val.is_empty() {
                        *j = k;
                        return format!("{}:{}", kind, val);
                    }
                }
            }
            k += 1;
        }
        String::new()
    }


/// Vue project generation context
pub struct VueProject {
    /// Project root directory (where pac.at is)
    pub root_dir: std::path::PathBuf,
    /// Output directory (dist)
    pub output_dir: std::path::PathBuf,
    /// Project name
    pub name: String,
    /// Front source directory
    pub front_dir: std::path::PathBuf,
    /// Public assets source directory
    pub public_dir: std::path::PathBuf,
    /// Detected shadcn-vue components
    pub shadcn_components: Vec<String>,
    /// Whether routes are detected
    pub has_routes: bool,
    /// Generated App.vue code
    pub app_vue_code: String,
    /// All components (relative_dir, name, code, widget_name)
    pub components: Vec<(String, String, String, String)>,
    /// All routes
    pub routes: Vec<AuraRoute>,
    /// Extra npm dependencies from pac.at (package_name, version_spec)
    pub npm_deps: Vec<(String, String)>,
    /// Native CSS files from pac.at `styles:` — copied verbatim into
    /// `src/styles/` and imported from `main.ts`. Paths are relative to
    /// the pac.at directory (root_dir).
    pub style_files: Vec<String>,
    /// Plan musk-022 Phase 2: i18n config from pac.at `i18n:` field. When
    /// enabled, the generated project wires vue-i18n (dependency + createI18n
    /// in main.ts + copied locale files).
    pub i18n: I18nConfig,
    /// Project-local TS/Vue files referenced by widget-level
    /// `use { fn/component/composable: ... from "<path>" }` blocks —
    /// copied into `src/ext/` (layout preserved) so the generated SFCs can
    /// import them as `@/ext/<path>`. Paths are relative to root_dir.
    pub ext_files: Vec<String>,
    /// Plan 043 store-codegen: generated store composable files
    /// `(filename, code)` — e.g. `("stores/useShellStoreStore.ts", ...)`.
    /// Collected explicitly from each .at's `store_composables` during
    /// `from_workspace`, then written to `src/stores/` in `generate()` /
    /// `regenerate_source_files()`. Replaces the fragile
    /// `STORE_EXTRA_FILES` thread-local (which is cleared per
    /// `generate_component_from_file` call and loses stores when multiple
    /// .at files are compiled in sequence).
    pub store_files: Vec<(String, String)>,
}

impl VueProject {
    /// Generate router file with support for nested page directories.
    /// Maps route modules to actual file paths under src/pages/.
    pub fn generate_router_file(&self) -> String {
        // Build a map from file stem -> pages subdirectory path
        // e.g., "login_01" -> "blocks/login_01"
        let mut page_paths: HashMap<String, String> = HashMap::new();
        for (relative_dir, name, _code, _widget_name) in &self.components {
            if relative_dir.starts_with("pages/") || relative_dir == "pages" {
                let sub_path = if relative_dir == "pages" {
                    name.clone()
                } else {
                    let dir_part = relative_dir.strip_prefix("pages/").unwrap_or(relative_dir);
                    format!("{}/{}", dir_part, name)
                };
                page_paths.insert(name.clone(), sub_path);
            }
        }

        let mut route_defs = Vec::new();
        for route in &self.routes {
            let path = &route.path;
            let module = &route.module;
            let import_path = page_paths.get(module).cloned().unwrap_or_else(|| module.clone());

            if route.params.is_empty() {
                route_defs.push(format!(
                    "  {{ path: '{}', name: '{}', component: () => import('@/pages/{}.vue') }}",
                    path, module, import_path
                ));
            } else {
                route_defs.push(format!(
                    "  {{ path: '{}', name: '{}', component: () => import('@/pages/{}.vue'), props: true }}",
                    path, module, import_path
                ));
            }
        }

        format!(
            r#"import {{ createRouter, createWebHashHistory }} from 'vue-router'
import type {{ RouteRecordRaw }} from 'vue-router'

const routes: RouteRecordRaw[] = [
{}
]

const router = createRouter({{
  history: createWebHashHistory(),
  routes,
}})

export default router
"#,
            route_defs.join(",\n")
        )
    }
    /// Create a new Vue project context from a workspace directory
    pub fn from_workspace(root_dir: &Path) -> AutoResult<Self> {
        let pac_path = root_dir.join("pac.at");
        if !pac_path.exists() {
            return Err("pac.at not found in workspace".into());
        }

        let pac_content = fs::read_to_string(&pac_path)
            .map_err(|e| format!("Failed to read pac.at: {}", e))?;

        // Parse workspace paths (Plan 129: app("front") syntax)
        let front_rel_path = parse_workspace_path(&pac_content, "front")
            .unwrap_or_else(|| "src/front".to_string());

        // Try the parsed path, then src/front, source/front, front
        let front_dir = if root_dir.join(&front_rel_path).exists() {
            root_dir.join(&front_rel_path)
        } else if root_dir.join("src").join("front").exists() {
            root_dir.join("src").join("front")
        } else if root_dir.join("source").join("front").exists() {
            root_dir.join("source").join("front")
        } else if root_dir.join("front").exists() {
            root_dir.join("front")
        } else {
            root_dir.join("src").join("front")
        };

        // Check if front directory exists
        if !front_dir.exists() {
            return Err(format!("Front directory '{}' not found", front_dir.display()).into());
        }

        // Find app.at in front directory
        let app_at = front_dir.join("app.at");
        if !app_at.exists() {
            return Err(format!("Entry file '{}' not found", app_at.display()).into());
        }

        // Get project name
        let name = parse_pac_name(&pac_content)
            .unwrap_or_else(|| "aura-app".to_string());

        // Output directory (Plan 129: vue/ instead of dist/)
        let output_dir = root_dir.join("gen").join("front").join("vue");
        let public_dir = front_dir.join("public");

        // Compile .at files
        let mut all_components: Vec<(String, String, String, String)> = Vec::new();
        let mut all_shadcn_components = HashSet::new();
        let mut all_routes: Vec<AuraRoute> = Vec::new();
        // Project-local files referenced by widget `use { ... }` imports
        let mut ext_file_set: std::collections::BTreeSet<String> = Default::default();
        // Plan 043 store-codegen: collect store composable files explicitly.
        // The STORE_EXTRA_FILES thread-local is cleared at the start of every
        // generate_component_from_file call (api.rs), so it only ever holds
        // the last .at's stores — unusable for multi-file workspaces.
        let mut all_store_files: Vec<(String, String)> = Vec::new();

        // Phase 1: Collect sub-widget names from front_dir .at files (to avoid shadcn name collisions)
        let mut sub_widget_names: Vec<String> = Vec::new();
        // Slot outlets declared by each sub-widget (name → outlet names,
        // "" = default). Used to warn when a parent passes slot children a
        // widget cannot render.
        let mut sub_widget_slot_outlets: std::collections::HashMap<String, Vec<String>> = Default::default();
        {
            for entry in fs::read_dir(&front_dir)
                .map_err(|e| format!("Failed to read front directory: {}", e))?
            {
                let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                let path = entry.path();

                if path.extension().map(|e| e == "at").unwrap_or(false) {
                    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                    // Skip app.at and pac.at
                    if file_name == "app.at" || file_name == "pac.at" {
                        continue;
                    }
                    // Quick-scan to collect widget names (lightweight parse)
                    if let Ok((_code, widgets)) = auto_lang::ui_build_shadcn_with_widgets(path.to_str().unwrap(), None) {
                        for widget in &widgets {
                            sub_widget_slot_outlets.insert(widget.name.clone(), widget.slot_outlet_names());
                            sub_widget_names.push(widget.name.clone());
                        }
                    }
                }
            }
        }

        // Process app.at — generate each widget independently, with known sub-widget names
        if app_at.exists() {
            match auto_lang::ui_build_shadcn_with_sub_widgets_and_stores(app_at.to_str().unwrap(), None, sub_widget_names.clone(), Some(root_dir.to_str().unwrap())) {
                Ok((vue_code, widgets, stores)) => {
                    collect_ext_import_files(&widgets, &mut ext_file_set);
                    let components = detect_shadcn_components(&vue_code);
                    for comp in &components {
                        all_shadcn_components.insert(comp.clone());
                    }
                    all_store_files.extend(stores);
                    for (i, widget) in widgets.iter().enumerate() {
                        if let Some(ref routes) = widget.routes {
                            all_routes.extend(routes.routes.clone());
                        }
                        // Slots: warn when app.at passes (default or named)
                        // slot children to a sub-widget with no matching outlet.
                        for warning in widget.slot_children_warnings(&sub_widget_slot_outlets) {
                            println!("{} {}", "Warning:".bright_yellow(), warning);
                        }
                        if i == 0 {
                            // First widget is the App root
                            all_components.push(("".to_string(), "app".to_string(), vue_code.clone(), widget.name.clone()));
                        } else {
                            // Additional widgets in app.at become components
                            // Extract store deps from app.at so these components get store imports
                            let app_store_deps = auto_lang::extract_store_deps_from_file(
                                app_at.to_str().unwrap()
                            );
                            let mut gen = VueGenerator::new_shadcn()
                                .with_sub_widgets(sub_widget_names.clone());
                            if !widget.api_imports.is_empty() {
                                gen = gen.with_project_api_functions(widget.api_imports.clone());
                            }
                            if !app_store_deps.is_empty() {
                                gen = gen.with_store_deps(app_store_deps.clone());
                            }
                            match gen.generate(widget) {
                                Ok(widget_code) => {
                                    let comp_names = detect_shadcn_components(&widget_code);
                                    for comp in &comp_names {
                                        all_shadcn_components.insert(comp.clone());
                                    }
                                    auto_lang::ui_gen::validators::print_warnings_once(
                                        &app_at.display().to_string(),
                                        &gen.last_validation_warnings,
                                    );
                                    all_components.push(("".to_string(), widget.name.to_lowercase(), widget_code, widget.name.clone()));
                                }
                                Err(e) => {
                                    println!("{} Failed to generate widget {}: {}", "Warning:".bright_yellow(), widget.name, e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    // Plan 012 Batch A: in strict mode a codegen failure (e.g.
                    // escalated validation warnings) must fail the whole build.
                    if auto_lang::ui_gen::validators::strict_enabled() {
                        return Err(format!("Failed to compile app.at: {}", e).into());
                    }
                    println!("{} {}", "Warning: Failed to compile app.at:".bright_yellow(), e);
                }
            }
        }

        // Process pages/ directory recursively
        fn scan_pages_dir(
            dir: &Path,
            front_dir: &Path,
            root_dir: &Path,
            all_components: &mut Vec<(String, String, String, String)>,
            all_shadcn_components: &mut HashSet<String>,
            all_routes: &mut Vec<AuraRoute>,
            ext_file_set: &mut std::collections::BTreeSet<String>,
            all_store_files: &mut Vec<(String, String)>,
        ) -> Result<(), String> {
            for entry in fs::read_dir(dir)
                .map_err(|e| format!("Failed to read pages directory: {}", e))?
            {
                let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                let path = entry.path();

                if path.is_dir() {
                    scan_pages_dir(&path, front_dir, root_dir, all_components, all_shadcn_components, all_routes, ext_file_set, all_store_files)?;
                } else if path.extension().map(|e| e == "at").unwrap_or(false) {
                    let file_stem = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("page");

                    let rel_path = path.strip_prefix(front_dir)
                        .map(|p| p.parent().unwrap_or(Path::new("")).to_string_lossy().to_string().replace('\\', "/"))
                        .unwrap_or_else(|_| "pages".to_string());

                    match auto_lang::ui_build_shadcn_all_widget_codes(path.to_str().unwrap(), Some(root_dir.to_str().unwrap())) {
                        Ok(result) => {
                            let vue_code = result.vue_code.clone();
                            let widgets = result.widgets.clone();
                            let stores = result.store_composables.clone();
                            collect_ext_import_files(&widgets, ext_file_set);
                            let components = detect_shadcn_components(&vue_code);
                            for comp in &components {
                                all_shadcn_components.insert(comp.clone());
                            }
                            for widget in &widgets {
                                if let Some(ref routes) = widget.routes {
                                    all_routes.extend(routes.routes.clone());
                                }
                            }
                            let widget_name = widgets.first().map(|w| w.name.as_str()).unwrap_or(file_stem);
                            all_components.push((rel_path, file_stem.to_string(), vue_code, widget_name.to_string()));
                            // Plan 408 P11 / KNOWN-DEBT: write any additional
                            // component fn SFCs from this pages .at file to
                            // components/ (previously discarded — only the first
                            // widget's vue_code was kept). The first entry is
                            // the page widget (already pushed above as a page);
                            // the rest are component fn SFCs.
                            for (i, (cname, ccode)) in result.all_widget_codes.iter().enumerate() {
                                if i == 0 { continue; }
                                all_components.push(("".to_string(), cname.clone(), ccode.clone(), cname.clone()));
                            }
                            all_store_files.extend(stores);
                        }
                        Err(e) => {
                            if auto_lang::ui_gen::validators::strict_enabled() {
                                return Err(format!("Failed to compile {}: {}", path.display(), e));
                            }
                            println!("{} Failed to compile {}: {}", "Warning:".bright_yellow(), path.display(), e);
                        }
                    }
                }
            }
            Ok(())
        }

        let pages_dir = front_dir.join("pages");
        if pages_dir.exists() {
            scan_pages_dir(&pages_dir, &front_dir, root_dir, &mut all_components, &mut all_shadcn_components, &mut all_routes, &mut ext_file_set, &mut all_store_files)
                .map_err(|e| format!("Failed to scan pages directory: {}", e))?;
        }

        // Process .at files directly in front_dir (sub-widgets like sidebar.at, editor.at)
        // Skip app.at (already processed) and pac.at (project config)
        for entry in fs::read_dir(&front_dir)
            .map_err(|e| format!("Failed to read front directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if path.extension().map(|e| e == "at").unwrap_or(false) {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                // Skip app.at and pac.at
                if file_name == "app.at" || file_name == "pac.at" {
                    continue;
                }

                match auto_lang::ui_build_shadcn_with_widgets_and_stores(path.to_str().unwrap(), None, Some(root_dir.to_str().unwrap())) {
                    Ok((vue_code, widgets, stores)) => {
                        collect_ext_import_files(&widgets, &mut ext_file_set);
                        let components = detect_shadcn_components(&vue_code);
                        for comp in &components {
                            all_shadcn_components.insert(comp.clone());
                        }
                        all_store_files.extend(stores);
                        // Extract store deps from this .at file so re-generated
                        // components get their store import + `const store = ...`
                        let file_store_deps = auto_lang::extract_store_deps_from_file(
                            path.to_str().unwrap()
                        );
                        for widget in &widgets {
                            if let Some(ref routes) = widget.routes {
                                all_routes.extend(routes.routes.clone());
                            }
                            // Generate each widget as an independent Vue component
                            let mut gen = VueGenerator::new_shadcn()
                                .with_sub_widgets(sub_widget_names.clone());
                            if !widget.api_imports.is_empty() {
                                gen = gen.with_project_api_functions(widget.api_imports.clone());
                            }
                            if !file_store_deps.is_empty() {
                                gen = gen.with_store_deps(file_store_deps.clone());
                            }
                            match gen.generate(widget) {
                                Ok(widget_code) => {
                                    let comp_names = detect_shadcn_components(&widget_code);
                                    for comp in &comp_names {
                                        all_shadcn_components.insert(comp.clone());
                                    }
                                    auto_lang::ui_gen::validators::print_warnings_once(
                                        &path.display().to_string(),
                                        &gen.last_validation_warnings,
                                    );
                                    let stem = path.file_stem()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("component");
                                    all_components.push(("".to_string(), stem.to_string(), widget_code, widget.name.clone()));
                                }
                                Err(e) => {
                                    println!("{} Failed to generate widget {}: {}", "Warning:".bright_yellow(), widget.name, e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if auto_lang::ui_gen::validators::strict_enabled() {
                            return Err(format!("Failed to compile {}: {}", path.display(), e).into());
                        }
                        println!("{} Failed to compile {}: {}", "Warning:".bright_yellow(), path.display(), e);
                    }
                }
            }
        }

        let shadcn_components: Vec<String> = all_shadcn_components.into_iter().collect();
        let has_routes = !all_routes.is_empty();

        // Get App.vue code
        let app_vue_code = all_components.iter()
            .find(|(_, name, _, _)| name == "app")
            .map(|(_, _, code, _)| code.clone())
            .ok_or_else(|| "app.at not found or failed to compile".to_string())?;

        Ok(Self {
            root_dir: root_dir.to_path_buf(),
            output_dir,
            name,
            front_dir,
            public_dir,
            shadcn_components,
            has_routes,
            app_vue_code,
            components: all_components,
            routes: all_routes,
            npm_deps: parse_npm_deps(&pac_content),
            style_files: parse_style_files(&pac_content),
            i18n: parse_i18n(&pac_content),
            ext_files: ext_file_set.into_iter().collect(),
            store_files: all_store_files,
        })
    }

    /// Check if the project structure already exists
    pub fn exists(&self) -> bool {
        self.output_dir.exists() && self.output_dir.join("package.json").exists()
    }

    /// Copy pac.at `styles:` CSS files into `src/styles/` (byte-for-byte)
    /// and return the copied file names for `main.ts` imports.
    ///
    /// Files are flattened to their file name — if two declared files share
    /// a file name, the later one wins.
    fn copy_style_files(&self) -> AutoResult<Vec<String>> {
        let mut copied = Vec::new();
        if self.style_files.is_empty() {
            return Ok(copied);
        }
        let styles_dir = self.output_dir.join("src").join("styles");
        fs::create_dir_all(&styles_dir)
            .map_err(|e| format!("Failed to create src/styles: {}", e))?;
        for rel in &self.style_files {
            let src = self.root_dir.join(rel);
            let file_name = Path::new(rel)
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| format!("Invalid styles path in pac.at: {}", rel))?
                .to_string();
            fs::copy(&src, styles_dir.join(&file_name)).map_err(|e| {
                format!("Failed to copy style file {}: {}", src.display(), e)
            })?;
            copied.push(file_name);
        }
        Ok(copied)
    }

    /// Plan musk-022 Phase 2: copy pac.at `i18n:` locale files into
    /// `src/locales/` (byte-for-byte) and return the copied file names for
    /// `main.ts` imports. Mirrors `copy_style_files`.
    fn copy_locale_files(&self) -> AutoResult<Vec<String>> {
        let mut copied = Vec::new();
        if !self.i18n.enabled || self.i18n.locale_files.is_empty() {
            return Ok(copied);
        }
        let locales_dir = self.output_dir.join("src").join("locales");
        fs::create_dir_all(&locales_dir)
            .map_err(|e| format!("Failed to create src/locales: {}", e))?;
        for rel in &self.i18n.locale_files {
            let src = self.root_dir.join(rel);
            let file_name = basename(rel);
            fs::copy(&src, locales_dir.join(&file_name))
                .map_err(|e| format!("Failed to copy locale file {}: {}", src.display(), e))?;
            copied.push(file_name);
        }
        Ok(copied)
    }

    /// Copy project-local files referenced by widget `use { ... }` imports
    /// into `src/ext/`, preserving their root-relative layout (so sibling
    /// relative imports between copied files keep resolving). Generated
    /// SFCs import them as `@/ext/<root-relative-path>`.
    fn copy_ext_files(&self) -> AutoResult<Vec<String>> {
        let mut copied = Vec::new();
        if self.ext_files.is_empty() {
            return Ok(copied);
        }
        let ext_dir = self.output_dir.join("src").join("ext");
        for rel in &self.ext_files {
            // Reject paths that escape the project root (`..` segments):
            // the generated `@/ext/...` specifier could not reach them.
            let mut normalized = std::path::PathBuf::new();
            for comp in Path::new(rel).components() {
                match comp {
                    std::path::Component::ParentDir => {
                        return Err(format!(
                            "Widget use-block import path '{}' escapes the project root; \
                             move the file into the project or consume it via pac.at npm_deps (link:) instead",
                            rel
                        )
                        .into());
                    }
                    std::path::Component::CurDir => {}
                    std::path::Component::Normal(c) => normalized.push(c),
                    _ => {}
                }
            }
            let src = self.root_dir.join(&normalized);
            let dst = ext_dir.join(&normalized);
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
            }
            fs::copy(&src, &dst).map_err(|e| {
                format!(
                    "Failed to copy use-block import {} → {}: {}",
                    src.display(),
                    dst.display(),
                    e
                )
            })?;
            copied.push(rel.clone());
        }
        Ok(copied)
    }

    /// Generate the Vue project structure
    pub fn generate(&self) -> AutoResult<()> {
        println!("{}", "─────────────────────────────────".bright_yellow().bold());
        println!("{}", "  AURA Workspace → Vue + shadcn-vue".bright_yellow().bold());
        println!("{}", "─────────────────────────────────".bright_yellow().bold());
        println!();

        println!("{} {}", "Output:".bright_cyan(), self.output_dir.display());
        println!("{} {}", "Name:".bright_cyan(), self.name);

        if !self.shadcn_components.is_empty() {
            println!("{} {}", "shadcn-vue:".bright_cyan(), self.shadcn_components.join(", "));
        }

        if self.has_routes {
            println!("{} {}", "Routes:".bright_cyan(), self.routes.len());
        }
        println!();

        // Create output directory
        fs::create_dir_all(&self.output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        // Create src directory structure
        let src_dir = self.output_dir.join("src");
        let components_dir = src_dir.join("components");
        let lib_dir = src_dir.join("lib");
        let assets_dir = src_dir.join("assets");

        fs::create_dir_all(&components_dir)
            .map_err(|e| format!("Failed to create src/components: {}", e))?;
        fs::create_dir_all(&lib_dir)
            .map_err(|e| format!("Failed to create src/lib: {}", e))?;
        fs::create_dir_all(&assets_dir)
            .map_err(|e| format!("Failed to create src/assets: {}", e))?;

        println!("{}", "✓ Created directory structure".bright_green());

        // Copy pac.at `styles:` CSS files (byte-for-byte) before writing
        // project files so main.ts can import them.
        let style_copies = self.copy_style_files()?;
        if !style_copies.is_empty() {
            println!("{} {}", "Styles:".bright_cyan(), style_copies.join(", "));
        }

        // Copy widget `use { ... }` local import files into src/ext/.
        let ext_copies = self.copy_ext_files()?;
        if !ext_copies.is_empty() {
            println!("{} {}", "Ext imports:".bright_cyan(), ext_copies.join(", "));
        }

        // Plan musk-022 Phase 2: copy pac.at `i18n:` locale files into src/locales/.
        let locale_copies = self.copy_locale_files()?;
        if self.i18n.enabled {
            println!(
                "{} {}",
                "i18n:".bright_cyan(),
                if locale_copies.is_empty() {
                    "enabled (inline messages)".to_string()
                } else {
                    locale_copies.join(", ")
                }
            );
        }

        // Write project files
        write_project_files(
            &self.output_dir,
            &self.name,
            &self.app_vue_code,
            &self.shadcn_components,
            self.has_routes,
            &self.npm_deps,
            &style_copies,
            &self.i18n,
            &locale_copies,
        )?;

        // Generate router files if routes detected
        if self.has_routes {
            let router_dir = self.output_dir.join("src/router");
            fs::create_dir_all(&router_dir)
                .map_err(|e| format!("Failed to create src/router: {}", e))?;

            let router_content = self.generate_router_file();
            fs::write(router_dir.join("index.ts"), router_content)
                .map_err(|e| format!("Failed to write router/index.ts: {}", e))?;

            println!("{}", "  Generated src/router/index.ts".bright_green());
        }

        // Write all components
        for (relative_dir, name, code, widget_name) in &self.components {
            if name != "app" {
                let output_subdir = if relative_dir.is_empty() || relative_dir == "components" {
                    components_dir.clone()
                } else if relative_dir == "pages" || relative_dir.starts_with("pages/") {
                    let pages_dir = src_dir.join("pages");
                    let sub_path = relative_dir.strip_prefix("pages/").unwrap_or(relative_dir);
                    if sub_path.is_empty() || sub_path == "pages" {
                        pages_dir
                    } else {
                        pages_dir.join(sub_path)
                    }
                } else if relative_dir.starts_with("components/") {
                    let sub_path = relative_dir.strip_prefix("components/").unwrap_or(relative_dir);
                    components_dir.join(sub_path)
                } else {
                    components_dir.join(relative_dir)
                };

                fs::create_dir_all(&output_subdir)
                    .map_err(|e| format!("Failed to create {}: {}", output_subdir.display(), e))?;

                let vue_file_name = if relative_dir == "pages" || relative_dir.starts_with("pages/") {
                    name.clone()
                } else {
                    widget_name.clone()
                };

                let component_file = output_subdir.join(format!("{}.vue", vue_file_name));
                fs::write(&component_file, code)
                    .map_err(|e| format!("Failed to write {}: {}", component_file.display(), e))?;
            }
        }

        // Plan 043 store-codegen: write store composable files (explicit, not
        // thread-local). Mirrors prepare_vue_sources' drain logic.
        if !self.store_files.is_empty() {
            let stores_dir = src_dir.join("stores");
            fs::create_dir_all(&stores_dir)
                .map_err(|e| format!("Failed to create src/stores: {}", e))?;
            for (filename, code) in &self.store_files {
                let clean_name = filename.strip_prefix("stores/").unwrap_or(filename);
                let path = stores_dir.join(clean_name);
                fs::write(&path, code)
                    .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
                println!("  {} Store composable: {}", "✓".bright_green(), path.display());
            }
        }

        println!("{}", "✓ Generated project files".bright_green());

        Ok(())
    }

    /// Generate scaffolding only (package.json, vite.config, tsconfig, etc.)
    /// WITHOUT overwriting component .vue files that were already written incrementally.
    /// Write src/router/index.ts when routes exist. main.ts unconditionally
    /// imports './router' for routed projects, so every scaffold/run path
    /// must leave this file in place — the incremental scaffolding path
    /// historically skipped it, breaking vite import resolution.
    pub fn ensure_router_file(&self) -> AutoResult<()> {
        if !self.has_routes {
            return Ok(());
        }
        let router_dir = self.output_dir.join("src/router");
        fs::create_dir_all(&router_dir)
            .map_err(|e| format!("Failed to create src/router: {}", e))?;
        let router_content = self.generate_router_file();
        fs::write(router_dir.join("index.ts"), router_content)
            .map_err(|e| format!("Failed to write router/index.ts: {}", e))?;
        println!("{}", "  ✓ Generated src/router/index.ts".bright_green());
        Ok(())
    }

    pub fn generate_scaffolding_only(&self) -> AutoResult<()> {
        let output_path = &self.output_dir;
        let src_dir = output_path.join("src");
        let components_dir = src_dir.join("components");
        let lib_dir = src_dir.join("lib");
        let assets_dir = src_dir.join("assets");

        fs::create_dir_all(output_path)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
        fs::create_dir_all(&src_dir)
            .map_err(|e| format!("Failed to create src: {}", e))?;
        fs::create_dir_all(&components_dir)
            .map_err(|e| format!("Failed to create src/components: {}", e))?;
        fs::create_dir_all(&lib_dir)
            .map_err(|e| format!("Failed to create src/lib: {}", e))?;
        fs::create_dir_all(&assets_dir)
            .map_err(|e| format!("Failed to create src/assets: {}", e))?;

        // Copy pac.at `styles:` CSS files (byte-for-byte) so main.ts can
        // import them.
        let style_copies = self.copy_style_files()?;

        // Copy widget `use { ... }` local import files into src/ext/.
        self.copy_ext_files()?;

        // Plan musk-022 Phase 2: copy i18n locale files.
        let locale_copies = self.copy_locale_files()?;

        // Scaffolding files (not component .vue files)
        write_project_files(
            output_path,
            &self.name,
            &self.app_vue_code,
            &self.shadcn_components,
            self.has_routes,
            &self.npm_deps,
            &style_copies,
            &self.i18n,
            &locale_copies,
        )?;

        // Write App.vue (the root component)
        let app_vue_path = src_dir.join("App.vue");
        if !app_vue_path.exists() {
            fs::write(&app_vue_path, &self.app_vue_code)
                .map_err(|e| format!("Failed to write App.vue: {}", e))?;
        }

        // Write main.ts
        let uses_autodown = self.npm_deps.iter().any(|(name, _)| name == "@autodown/editor");
        let main_ts_content = generate_main_ts(self.has_routes, uses_autodown, &style_copies, &self.i18n, &locale_copies);
        fs::write(src_dir.join("main.ts"), &main_ts_content)
            .map_err(|e| format!("Failed to write main.ts: {}", e))?;

        // Write index.css
        let index_css_content = generate_index_css();
        fs::write(assets_dir.join("index.css"), &index_css_content)
            .map_err(|e| format!("Failed to write src/assets/index.css: {}", e))?;

        // Write tsconfig.json
        let tsconfig = generate_tsconfig();
        fs::write(output_path.join("tsconfig.json"), &tsconfig)
            .map_err(|e| format!("Failed to write tsconfig.json: {}", e))?;

        // Router file — main.ts imports './router' whenever routes exist.
        self.ensure_router_file()?;

        println!("{}", "✓ Generated scaffolding (preserved incremental components)".bright_green());

        Ok(())
    }

    /// Regenerate only source files (App.vue, pages, components, router)
    /// This preserves node_modules, package.json, and installed shadcn components
    pub fn regenerate_source_files(&self) -> AutoResult<()> {
        println!("{}", "Regenerating source files...".bright_cyan());

        let src_dir = self.output_dir.join("src");
        let components_dir = src_dir.join("components");

        // Regenerate App.vue
        let app_vue_path = src_dir.join("App.vue");
        fs::write(&app_vue_path, &self.app_vue_code)
            .map_err(|e| format!("Failed to write App.vue: {}", e))?;
        println!("{}", "  ✓ Regenerated App.vue".bright_green());

        // Regenerate main.ts (re-copy pac.at `styles:` CSS files first so
        // the imports below resolve)
        let style_copies = self.copy_style_files()?;
        // Re-copy widget `use { ... }` local import files into src/ext/.
        self.copy_ext_files()?;
        // Plan musk-022 Phase 2: re-copy i18n locale files.
        let locale_copies = self.copy_locale_files()?;
        let uses_autodown = self.npm_deps.iter().any(|(name, _)| name == "@autodown/editor");
        let main_ts_content = generate_main_ts(self.has_routes, uses_autodown, &style_copies, &self.i18n, &locale_copies);
        let main_ts_path = src_dir.join("main.ts");
        fs::write(&main_ts_path, &main_ts_content)
            .map_err(|e| format!("Failed to write main.ts: {}", e))?;
        println!("{}", "  ✓ Regenerated main.ts".bright_green());

        // Regenerate src/assets/index.css
        let assets_dir = src_dir.join("assets");
        fs::create_dir_all(&assets_dir)
            .map_err(|e| format!("Failed to create src/assets: {}", e))?;
        let index_css_content = generate_index_css();
        let index_css_path = assets_dir.join("index.css");
        fs::write(&index_css_path, &index_css_content)
            .map_err(|e| format!("Failed to write src/assets/index.css: {}", e))?;
        println!("{}", "  ✓ Regenerated src/assets/index.css".bright_green());

        // Regenerate tsconfig.json
        let tsconfig_path = self.output_dir.join("tsconfig.json");
        let tsconfig = generate_tsconfig();
        fs::write(&tsconfig_path, &tsconfig)
            .map_err(|e| format!("Failed to write tsconfig.json: {}", e))?;
        println!("{}", "  ✓ Regenerated tsconfig.json".bright_green());

        // Regenerate index.html (Plan 043 M5: carries `class="dark"` so the
        // shadcn `.dark` tokens in index.css actually apply; without it the
        // app renders light). Previously only written on the initial scaffold,
        // so a fresh index.html (or a generator fix) never took effect.
        let index_html_path = self.output_dir.join("index.html");
        let index_html = generate_index_html(&self.name);
        fs::write(&index_html_path, &index_html)
            .map_err(|e| format!("Failed to write index.html: {}", e))?;
        println!("{}", "  ✓ Regenerated index.html".bright_green());

        // Regenerate package.json if outdated (e.g., missing @types/prismjs,
        // or missing vue-i18n when i18n is enabled — Plan musk-022 Phase 2)
        let pkg_path = self.output_dir.join("package.json");
        if pkg_path.exists() {
            let existing_pkg = fs::read_to_string(&pkg_path)
                .map_err(|e| format!("Failed to read package.json: {}", e))?;
            let needs_i18n = self.i18n.enabled && !existing_pkg.contains("vue-i18n");
            if !existing_pkg.contains("@types/prismjs")
                || !existing_pkg.contains("onlyBuiltDependencies")
                || needs_i18n
            {
                let new_pkg = generate_package_json(&self.name, self.has_routes, self.i18n.enabled, &self.npm_deps);
                fs::write(&pkg_path, &new_pkg)
                    .map_err(|e| format!("Failed to write package.json: {}", e))?;
                println!("{}", "  ✓ Updated package.json".bright_green());
            }
        }

        // Regenerate router if routes exist
        if self.has_routes {
            let router_dir = self.output_dir.join("src/router");
            fs::create_dir_all(&router_dir)
                .map_err(|e| format!("Failed to create src/router: {}", e))?;

            let router_content = self.generate_router_file();
            fs::write(router_dir.join("index.ts"), router_content)
                .map_err(|e| format!("Failed to write router/index.ts: {}", e))?;

            println!("{}", "  ✓ Regenerated router/index.ts".bright_green());
        }

        // Regenerate all components and pages
        let mut pages_count = 0;
        let mut components_count = 0;

        for (relative_dir, name, code, widget_name) in &self.components {
            if name != "app" {
                let output_subdir = if relative_dir.is_empty() || relative_dir == "components" {
                    components_dir.clone()
                } else if relative_dir == "pages" || relative_dir.starts_with("pages/") {
                    let pages_dir = src_dir.join("pages");
                    let sub_path = relative_dir.strip_prefix("pages/").unwrap_or(relative_dir);
                    if sub_path.is_empty() || sub_path == "pages" {
                        pages_dir
                    } else {
                        pages_dir.join(sub_path)
                    }
                } else if relative_dir.starts_with("components/") {
                    let sub_path = relative_dir.strip_prefix("components/").unwrap_or(relative_dir);
                    components_dir.join(sub_path)
                } else {
                    components_dir.join(relative_dir)
                };

                fs::create_dir_all(&output_subdir)
                    .map_err(|e| format!("Failed to create {}: {}", output_subdir.display(), e))?;

                let vue_file_name = if relative_dir == "pages" || relative_dir.starts_with("pages/") {
                    pages_count += 1;
                    name.clone()
                } else {
                    components_count += 1;
                    widget_name.clone()
                };

                let component_file = output_subdir.join(format!("{}.vue", vue_file_name));
                fs::write(&component_file, code)
                    .map_err(|e| format!("Failed to write {}: {}", component_file.display(), e))?;
            }
        }

        // Plan 043 store-codegen: regenerate store composable files explicitly.
        if !self.store_files.is_empty() {
            let stores_dir = src_dir.join("stores");
            fs::create_dir_all(&stores_dir)
                .map_err(|e| format!("Failed to create src/stores: {}", e))?;
            for (filename, code) in &self.store_files {
                let clean_name = filename.strip_prefix("stores/").unwrap_or(filename);
                let path = stores_dir.join(clean_name);
                fs::write(&path, code)
                    .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
                println!("  {} Store composable: {}", "✓".bright_green(), path.display());
            }
        }

        if pages_count > 0 {
            println!("{}", format!("  ✓ Regenerated {} pages", pages_count).bright_green());
        }
        if components_count > 0 {
            println!("{}", format!("  ✓ Regenerated {} components", components_count).bright_green());
        }

        Ok(())
    }

    /// Run package manager install
    pub fn npm_install(&self) -> AutoResult<()> {
        let pm = crate::pkg::display_name();

        // Ensure package.json has pnpm onlyBuiltDependencies for esbuild/vue-demi
        // Plan 328: Ensure .npmrc has correct pnpm v10+ build approvals.
        // pnpm v10+ reads build approvals from .npmrc (only-built-dependencies[]).
        if ensure_pnpm_build_approvals(&self.output_dir) {
            // yaml written (or already correct)
        }

        if !crate::pkg::command_exists(crate::pkg::install_cmd()) {
            println!("{}", format!("⚠ {} not found. Please install it or Node.js.", pm).bright_yellow());
            return Err(format!("{} not found", pm).into());
        }

        // pnpm 10/11 reads build approvals from .npmrc (only-built-dependencies[]),
        // not package.json. Write the correct list before install so it doesn't
        // exit 1 with ERR_PNPM_IGNORED_BUILDS.
        if pm == "pnpm" && ensure_pnpm_build_approvals(&self.output_dir) {
            println!("{}", "  ✓ Wrote .npmrc (build approvals for esbuild/vue-demi)".bright_green());
        }

        println!();
        println!("{} {}", "▶".bright_cyan(), "Installing dependencies...".bright_white());
        println!("{}", format!("  Running: {} install", pm).bright_black());

        match crate::pkg::install(&self.output_dir) {
            Ok(_) => {
                println!("{}", "  ✓ Dependencies installed".bright_green());
                Ok(())
            }
            Err(e) => {
                // pnpm v11 fails with ERR_PNPM_IGNORED_BUILDS when postinstall
                // builds (esbuild/vue-demi) are un-approved, OR when node_modules
                // already contains the unbuilt packages (pnpm then reports
                // "Already up to date", skips the build step, and re-emits the
                // error). Auto-recover by re-asserting the build approvals and
                // wiping node_modules + lockfile so the next install runs the
                // (now-approved) builds from scratch.
                if pm == "pnpm" {
                    println!("{}", "  ⚠ Retrying: rebuilding from clean node_modules...".bright_yellow());
                    let lockfile = self.output_dir.join("pnpm-lock.yaml");
                    if lockfile.exists() {
                        let _ = fs::remove_file(&lockfile);
                    }
                    let node_modules = self.output_dir.join("node_modules");
                    if node_modules.exists() {
                        let _ = fs::remove_dir_all(&node_modules);
                    }
                    // Re-assert build approvals in case pnpm clobbered the file
                    // with a broken scaffold during the failed attempt.
                    if ensure_pnpm_build_approvals(&self.output_dir) {
                        println!("{}", "  ✓ Re-wrote .npmrc".bright_green());
                    }
                    match crate::pkg::install(&self.output_dir) {
                        Ok(_) => {
                            println!("{}", "  ✓ Dependencies installed (retry)".bright_green());
                            Ok(())
                        }
                        Err(e2) => {
                            println!("{} {}", "  ✗ Failed:".bright_red(), e2);
                            Err(format!("{} install failed: {}", pm, e2).into())
                        }
                    }
                } else {
                    println!("{} {}", "  ✗ Failed:".bright_red(), e);
                    Err(format!("{} install failed: {}", pm, e).into())
                }
            }
        }
    }

    /// Fix known compatibility issues in shadcn-vue installed components
    fn fix_shadcn_compatibility_issues(&self) {
        // Fix Sonner.vue: lucide-vue-next icon naming changed in newer versions
        let sonner_path = self.output_dir.join("src/components/ui/sonner/Sonner.vue");
        if sonner_path.exists() {
            if let Ok(content) = fs::read_to_string(&sonner_path) {
                let fixed = content
                    .replace("CircleCheckIcon", "CheckCircle")
                    .replace("OctagonXIcon", "XOctagon")
                    .replace("TriangleAlertIcon", "AlertTriangle");
                if fixed != content {
                    let _ = fs::write(&sonner_path, fixed);
                    println!("{}", "  ✓ Fixed Sonner.vue icon names for lucide-vue-next compatibility".bright_green());
                }
            }
        }
    }

    /// Install shadcn-vue components
    pub fn install_shadcn_components(&self) -> AutoResult<()> {
        if self.shadcn_components.is_empty() {
            println!("{} {}", "▶".bright_cyan(), "No shadcn-vue components needed".bright_white());
            return Ok(());
        }

        // Fix known compatibility issues regardless of whether components are already installed
        self.fix_shadcn_compatibility_issues();

        // Check if already installed
        if are_shadcn_components_installed(&self.output_dir, &self.shadcn_components) {
            println!("{} {}", "▶".bright_cyan(), "shadcn-vue components already installed (skipping)".bright_white());
            return Ok(());
        }

        println!();
        println!("{} {}", "▶".bright_cyan(), format!("Adding shadcn-vue components ({})...", self.shadcn_components.join(", ")).bright_white());

        let mut pkg_args: Vec<&str> = vec!["add"];
        pkg_args.extend(self.shadcn_components.iter().map(|s| s.as_str()));
        pkg_args.push("--yes");  // shadcn-vue uses --yes for non-interactive

        println!("{}", format!("  Running: {} shadcn-vue@latest add {}", crate::pkg::exec_cmd(), self.shadcn_components.join(" ")).bright_black());

        match crate::pkg::exec("shadcn-vue@latest", &pkg_args, &self.output_dir) {
            Ok(_) => {
                println!("{}", "  ✓ shadcn-vue components added".bright_green());
                // Fix known compatibility issues in installed components
                self.fix_shadcn_compatibility_issues();
                // shadcn-vue add runs `pnpm install`/`pnpm add` internally. pnpm may
                // leave a scaffolded pnpm-workspace.yaml behind (activating workspace
                // mode) or clobber .npmrc. Re-assert the correct build approvals so
                // subsequent pnpm invocations don't fail.
                ensure_pnpm_build_approvals(&self.output_dir);
                Ok(())
            }
            Err(e) => {
                println!("  ✗ shadcn-vue add failed: {}", e.to_string().bright_red());
                println!("  You may need to run '{} shadcn-vue@latest add {} -y' manually.", crate::pkg::exec_cmd(), self.shadcn_components.join(" "));
                // shadcn-vue add may have partially run pnpm and left a stray
                // pnpm-workspace.yaml / clobbered .npmrc — re-assert approvals.
                ensure_pnpm_build_approvals(&self.output_dir);
                // Don't fail - user can install manually
                Ok(())
            }
        }
    }

    /// Copy public assets
    pub fn copy_public_assets(&self) -> AutoResult<()> {
        if !self.public_dir.exists() || !self.public_dir.is_dir() {
            println!("{} {}", "▶".bright_cyan(), "No public assets to copy".bright_white());
            return Ok(());
        }

        let dest_public = self.output_dir.join("public");
        if dest_public.exists() && dest_public.is_dir() {
            println!("{} {}", "▶".bright_cyan(), "Public assets already copied (skipping)".bright_white());
            return Ok(());
        }

        println!();
        println!("{} {}", "▶".bright_cyan(), "Copying public assets...".bright_white());

        copy_dir_all(&self.public_dir, &dest_public)
            .map_err(|e| format!("Failed to copy public folder: {}", e))?;

        println!("{}", "  ✓ Public assets copied".bright_green());
        Ok(())
    }

    /// Run package manager build
    pub fn npm_build(&self) -> AutoResult<()> {
        let pm = crate::pkg::display_name();
        println!();
        println!("{} {}", "▶".bright_cyan(), "Building Vue project...".bright_white());
        println!("{}", format!("  Running: {} run build", pm).bright_black());

        match crate::pkg::run_script("build", &[], &self.output_dir) {
            Ok(_) => {
                println!();
                println!("═════════════════════════════════");
                println!("{}", "  Vue project built successfully!".bright_green().bold());
                println!("═════════════════════════════════");
                Ok(())
            }
            Err(e) => {
                Err(format!("{} run build failed: {}", pm, e).into())
            }
        }
    }

    /// Run package manager dev server
    pub fn npm_run_dev(&self, args: Vec<String>) -> AutoResult<()> {
        let pm = crate::pkg::display_name();
        println!();
        println!("{} {}", "▶".bright_cyan(), "Starting dev server...".bright_white());
        println!();
        println!("═════════════════════════════════");
        println!("{}", "  Starting Vue dev server...".bright_green().bold());
        println!("═════════════════════════════════");
        println!();

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        match crate::pkg::run_script("dev", &args_str, &self.output_dir) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("{} run dev failed: {}", pm, e).into())
        }
    }
}

/// Build Vue project (auto build command)
///
/// Steps:
/// 1. Generate/regenerate project sources (see `prepare_vue_sources`)
/// 2. npm install
/// 3. Install shadcn-vue components
/// 4. Copy public assets
/// 5. npm run build
pub fn build_vue_project(root_dir: &Path) -> AutoResult<()> {
    println!("{}", "Building Vue project (backend: vue)".bright_cyan());
    let project = prepare_vue_sources(root_dir)?;

    // Step 3: npm install
    println!();
    println!("▶ Installing dependencies...");
    project.npm_install()?;

    // Step 4: Install shadcn-vue components
    println!();
    println!("▶ Installing shadcn-vue components...");
    project.install_shadcn_components()?;

    // Step 5: Copy public assets
    println!();
    println!("▶ Copying public assets...");
    project.copy_public_assets()?;

    // Step 6: npm run build
    println!();
    println!("▶ Building Vue project...");
    project.npm_build()?;

    Ok(())
}

/// Generate-only build for the Vue backend (`auto build --gen-only`).
///
/// Runs the full .at → Vue SFC generation pipeline (parse, ui_gen,
/// post-generation validators, style/use-block asset copies) but stops
/// before any npm/pnpm step. Used by CI to regression-guard the generator
/// without paying for npm install + vite build per example.
pub fn gen_vue_project(root_dir: &Path) -> AutoResult<()> {
    println!(
        "{}",
        "Generating Vue project (backend: vue, gen-only)".bright_cyan()
    );
    let project = prepare_vue_sources(root_dir)?;
    println!(
        "{}",
        format!(
            "✓ Generation complete ({} component(s)); npm steps skipped (--gen-only)",
            project.components.len()
        )
        .bright_green()
    );
    Ok(())
}

/// Shared generation phase of the Vue build: compile all .at sources into
/// SFCs (running the ui_gen validators), write/regenerate the project
/// sources under gen/front/vue, and copy handmade/style/use-block assets.
/// Returns the loaded project so callers can continue with npm steps.
fn prepare_vue_sources(root_dir: &Path) -> AutoResult<VueProject> {
    // Pre-load API function names BEFORE creating VueProject (which instantiates VueGenerator)
    let api_fns_path = root_dir.join("dist").join(".api_functions");
    if api_fns_path.exists() {
        if let Ok(content) = fs::read_to_string(&api_fns_path) {
            let fns: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
            if !fns.is_empty() {
                // SAFETY: Setting a process-wide env var before VueGenerator::new() reads it
                unsafe { std::env::set_var("AUTO_API_FUNCTIONS", fns.join(",")); }
            }
        }
    }

    // Load project context
    let project = VueProject::from_workspace(root_dir)?;

    // Plan 351: drain stashed store composable files and write them
    for (filename, content) in auto_lang::drain_store_extra_files() {
        let stores_dir = project.output_dir.join("src").join("stores");
        fs::create_dir_all(&stores_dir).ok();
        let clean_name = filename.strip_prefix("stores/").unwrap_or(&filename);
        let path = stores_dir.join(clean_name);
        fs::write(&path, &content).ok();
        println!("  ✓ Store composable: {}", path.display());
    }

    // Step 1: Generate project structure if not exists, or regenerate source files if exists
    if !project.exists() {
        println!();
        println!("▶ Generating Vue project...");
        project.generate()?;
    } else {
        // Regenerate source files even if project exists
        println!();
        println!("▶ Regenerating source files...");
        project.regenerate_source_files()?;
    }

    // Step 2: Generate API client code (if api.at exists)
    println!();
    println!("▶ Generating API client...");
    if let Err(e) = crate::api_gen::generate_api(root_dir, "vue") {
        // API generation is optional - only warn on failure
        println!("  ⚠ API generation skipped: {}", e);
    }

    // Refresh API function names after generating (for next run)
    let api_fns_path = root_dir.join("dist").join(".api_functions");
    if api_fns_path.exists() {
        if let Ok(content) = fs::read_to_string(&api_fns_path) {
            let fns: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
            if !fns.is_empty() {
                // SAFETY: Setting a process-wide env var for downstream use
                unsafe { std::env::set_var("AUTO_API_FUNCTIONS", fns.join(",")); }
            }
        }
    }

    // Copy handmade theme assets if available
    let handmade_css = root_dir.join("vue").join("src").join("assets").join("index.css");
    let gen_css = root_dir.join("gen").join("front").join("vue").join("src").join("assets").join("index.css");
    if handmade_css.exists() && gen_css.exists() {
        if let Ok(content) = fs::read_to_string(&handmade_css) {
            fs::write(&gen_css, content)
                .map_err(|e| format!("Failed to copy handmade index.css: {}", e))?;
            println!("{}", "  ✓ Copied handmade theme CSS".bright_green());
        }
    }
    let handmade_theme_toggle = root_dir.join("vue").join("src").join("components").join("ThemeToggle.vue");
    let gen_components_dir = root_dir.join("gen").join("front").join("vue").join("src").join("components");
    if handmade_theme_toggle.exists() {
        let gen_theme_toggle = gen_components_dir.join("ThemeToggle.vue");
        if let Ok(content) = fs::read_to_string(&handmade_theme_toggle) {
            fs::write(&gen_theme_toggle, content)
                .map_err(|e| format!("Failed to copy ThemeToggle.vue: {}", e))?;
            println!("{}", "  ✓ Copied ThemeToggle.vue".bright_green());
        }
    }

    // Plan 234: Copy all handmade Vue components from vue/src/components/
    let handmade_components_dir = root_dir.join("vue").join("src").join("components");
    if handmade_components_dir.exists() && handmade_components_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&handmade_components_dir) {
            let mut copied_count = 0;
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = entry.file_name();
                // Skip ThemeToggle.vue (already handled above)
                if file_name == "ThemeToggle.vue" {
                    continue;
                }
                let dest = gen_components_dir.join(&file_name);
                if path.is_dir() {
                    // Copy subdirectories recursively (e.g. a2ui-renderers/)
                    if let Err(e) = copy_dir_all(&path, &dest) {
                        println!("  ⚠ Failed to copy handmade component dir {}: {}", file_name.to_string_lossy(), e);
                    } else {
                        copied_count += 1;
                    }
                } else if path.extension().map(|e| e == "vue").unwrap_or(false) {
                    if let Err(e) = fs::copy(&path, &dest) {
                        println!("  ⚠ Failed to copy handmade component {}: {}", file_name.to_string_lossy(), e);
                    } else {
                        copied_count += 1;
                    }
                }
            }
            if copied_count > 0 {
                println!("{}", format!("  ✓ Copied {} handmade component(s)", copied_count).bright_green());
            }
        }
    }

    // Plan 234: Copy handmade composables
    let handmade_composables_dir = root_dir.join("vue").join("src").join("composables");
    let gen_composables_dir = root_dir.join("gen").join("front").join("vue").join("src").join("composables");
    if handmade_composables_dir.exists() && handmade_composables_dir.is_dir() {
        fs::create_dir_all(&gen_composables_dir).ok();
        if let Ok(entries) = fs::read_dir(&handmade_composables_dir) {
            let mut copied_count = 0;
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = entry.file_name();
                let dest = gen_composables_dir.join(&file_name);
                if path.is_dir() {
                    if let Err(e) = copy_dir_all(&path, &dest) {
                        println!("  ⚠ Failed to copy composable dir {}: {}", file_name.to_string_lossy(), e);
                    } else {
                        copied_count += 1;
                    }
                } else {
                    if let Err(e) = fs::copy(&path, &dest) {
                        println!("  ⚠ Failed to copy composable {}: {}", file_name.to_string_lossy(), e);
                    } else {
                        copied_count += 1;
                    }
                }
            }
            if copied_count > 0 {
                println!("{}", format!("  ✓ Copied {} composable(s)", copied_count).bright_green());
            }
        }
    }

    // Plan 234: Copy handmade types
    let handmade_types_dir = root_dir.join("vue").join("src").join("types");
    let gen_types_dir = root_dir.join("gen").join("front").join("vue").join("src").join("types");
    if handmade_types_dir.exists() && handmade_types_dir.is_dir() {
        fs::create_dir_all(&gen_types_dir).ok();
        if let Ok(entries) = fs::read_dir(&handmade_types_dir) {
            let mut copied_count = 0;
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = entry.file_name();
                let dest = gen_types_dir.join(&file_name);
                if path.is_file() {
                    if let Err(e) = fs::copy(&path, &dest) {
                        println!("  ⚠ Failed to copy type file {}: {}", file_name.to_string_lossy(), e);
                    } else {
                        copied_count += 1;
                    }
                }
            }
            if copied_count > 0 {
                println!("{}", format!("  ✓ Copied {} type file(s)", copied_count).bright_green());
            }
        }
    }

    Ok(project)
}

/// Incremental compile phase of `auto run`: compiles every changed (or
/// output-missing) .at file through the UI cache, writes the changed SFCs
/// plus the store composables collected from each compiled file, and returns
/// the number of changed SFCs written.
///
/// Plan 012 Batch B: extracted from run_vue_project so the store-emission
/// (gap 9a) and parse-failure semantics (gap 9b) are unit-testable without
/// npm. Parse failures print the same "Warning: Failed to compile ..." line
/// as the fresh path (see `handle_compile_error`) and fail the build under
/// `auto build --strict`.
fn incremental_compile_changed(root_dir: &Path) -> AutoResult<usize> {
    // Pre-load API function names BEFORE any VueGenerator::new() calls
    let api_fns_path = root_dir.join("dist").join(".api_functions");
    if api_fns_path.exists() {
        if let Ok(content) = fs::read_to_string(&api_fns_path) {
            let fns: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
            if !fns.is_empty() {
                // SAFETY: Setting a process-wide env var before VueGenerator::new() reads it
                unsafe { std::env::set_var("AUTO_API_FUNCTIONS", fns.join(",")); }
            }
        }
    }

    // Resolve front directory using same logic as VueProject::from_workspace
    let front_dir = resolve_front_dir(root_dir);
    let output_dir = root_dir.join("gen").join("front").join("vue");

    // Load cache for incremental compilation
    let mut cache = UICache::load(root_dir);

    // Invalidate cache if .api_functions changed (API imports may be different)
    if cache.invalidate_if_api_functions_changed(&api_fns_path) {
        println!("  {} (API config changed, regenerating all)", "cache".bright_yellow());
    }

    let mut changed_files: Vec<(PathBuf, String, String)> = Vec::new(); // (output_path, vue_code, widget_name)
    // Plan 012 Batch B (gap 9a): store composables are collected explicitly
    // from every compiled .at file. The STORE_EXTRA_FILES thread-local is
    // cleared per generate_component_from_file call, so it only ever holds
    // the LAST compiled file's stores — unusable for multi-store workspaces.
    let mut store_files: Vec<(String, String)> = Vec::new();

    // Phase 1: Scan sub-widget .at files in front_dir (e.g. editor.at, sidebar.at)
    // Collect their names for app.at compilation and generate their .vue files
    let mut sub_widget_names: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir(&front_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "at").unwrap_or(false) {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                // Skip app.at, pac.at, types.at, mod.at
                if file_name == "app.at" || file_name == "pac.at" || file_name == "types.at" || file_name == "mod.at" {
                    continue;
                }
                let file_stem = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("component");

                if let Ok(content) = fs::read_to_string(&path) {
                    let hash = hash_string(&content);
                    let source_changed = cache.is_dirty(&path, hash);
                    let widget_output = output_dir.join("src").join("components").join(format!("{}.vue", file_stem));
                    let output_missing = !widget_output.exists();

                    if source_changed || output_missing {
                        println!("  {} (changed)", file_name.bright_yellow());
                        match compile_at_to_vue(&path, &content, root_dir) {
                            Ok((vue_code, widgets, stores)) => {
                                store_files.extend(stores);
                                for widget_name in &widgets {
                                    sub_widget_names.push(widget_name.clone());
                                    // Also generate with widget name as fallback
                                    let output_path = output_dir.join("src").join("components").join(format!("{}.vue", widget_name));
                                    changed_files.push((output_path, vue_code.clone(), widget_name.clone()));
                                }
                                let artifacts: Vec<UIArtifact> = widgets.iter().map(|w| {
                                    UIArtifact {
                                        source_path: path.clone(),
                                        widget_name: w.clone(),
                                        output_path: PathBuf::from(format!("src/components/{}.vue", w)),
                                        source_hash: hash,
                                        content_hash: hash_string(&vue_code),
                                        backend: UIBackend::Vue,
                                    }
                                }).collect();
                                cache.update(path.clone(), hash, artifacts);
                            }
                            Err(e) => handle_compile_error(&path, &e)?,
                        }
                    } else {
                        // Cached: still need sub-widget names for app.at compilation
                        match compile_at_to_vue(&path, &content, root_dir) {
                            Ok((_vue_code, widgets, _stores)) => {
                                for widget_name in &widgets {
                                    sub_widget_names.push(widget_name.clone());
                                }
                            }
                            Err(e) => handle_compile_error(&path, &e)?,
                        }
                        println!("  {} (cached)", file_name.bright_green());
                    }
                }
            }
        }
    }

    // Phase 2: Check app.at for changes (with sub-widget names known)
    let app_at = front_dir.join("app.at");
    let app_output_path = output_dir.join("src").join("App.vue");
    if app_at.exists() {
        if let Ok(content) = fs::read_to_string(&app_at) {
            let hash = hash_string(&content);
            let source_changed = cache.is_dirty(&app_at, hash);
            let output_missing = !app_output_path.exists();

            if source_changed || output_missing {
                if source_changed {
                    println!("  {} (changed)", "app.at".bright_yellow());
                } else {
                    println!("  {} (output missing)", "app.at".bright_yellow());
                }
                match compile_at_to_vue_with_sub_widgets(&app_at, &content, sub_widget_names.clone(), root_dir) {
                    Ok((vue_code, widgets, stores)) => {
                        store_files.extend(stores);
                        let content_hash = hash_string(&vue_code);
                        if let Some(widget_name) = widgets.first() {
                            changed_files.push((app_output_path, vue_code, widget_name.clone()));
                        }
                        let artifacts: Vec<UIArtifact> = widgets.iter().map(|w| {
                            UIArtifact {
                                source_path: app_at.clone(),
                                widget_name: w.clone(),
                                output_path: PathBuf::from(format!("src/App.vue")),
                                source_hash: hash,
                                content_hash: content_hash.clone(),
                                backend: UIBackend::Vue,
                            }
                        }).collect();
                        cache.update(app_at.clone(), hash, artifacts);
                    }
                    Err(e) => handle_compile_error(&app_at, &e)?,
                }
            } else {
                println!("  {} (cached)", "app.at".bright_green());
            }
        }
    }

    // Phase 3: Check widgets/ directory for changes
    let widgets_dir = front_dir.join("widgets");
    if widgets_dir.exists() {
        if let Ok(entries) = fs::read_dir(&widgets_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "at").unwrap_or(false) {
                    let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                    if let Ok(content) = fs::read_to_string(&path) {
                        let hash = hash_string(&content);
                        // For widgets, we need to compile first to get widget name for output path
                        // So we check cache first, then verify output exists
                        let source_changed = cache.is_dirty(&path, hash);

                        if source_changed {
                            println!("  widgets/{} (changed)", file_name.bright_yellow());
                            match compile_at_to_vue(&path, &content, root_dir) {
                                Ok((vue_code, widgets, stores)) => {
                                    store_files.extend(stores);
                                    if let Some(widget_name) = widgets.first() {
                                        let output_path = output_dir.join("src").join("components").join(format!("{}.vue", widget_name));
                                        changed_files.push((output_path, vue_code, widget_name.clone()));
                                    }
                                    let artifacts: Vec<UIArtifact> = widgets.iter().map(|w| {
                                        UIArtifact {
                                            source_path: path.clone(),
                                            widget_name: w.clone(),
                                            output_path: PathBuf::from(format!("src/components/{}.vue", w)),
                                            source_hash: hash,
                                            content_hash: hash_string(&changed_files.last().map(|f| f.1.as_str()).unwrap_or("")),
                                            backend: UIBackend::Vue,
                                        }
                                    }).collect();
                                    cache.update(path.clone(), hash, artifacts);
                                }
                                Err(e) => handle_compile_error(&path, &e)?,
                            }
                        } else {
                            println!("  widgets/{} (cached)", file_name.bright_green());
                        }
                    }
                }
            }
        }
    }

    // Phase 4: Check pages/ directory for changes
    let pages_dir = front_dir.join("pages");
    if pages_dir.exists() {
        if let Ok(entries) = fs::read_dir(&pages_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "at").unwrap_or(false) {
                    let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                    // Use file stem (e.g., "index") as the output file name, matching VueProject::generate behavior
                    let file_stem = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("page");
                    // Pre-compute output path for existence check
                    let output_path = output_dir.join("src").join("pages").join(format!("{}.vue", file_stem));

                    if let Ok(content) = fs::read_to_string(&path) {
                        let hash = hash_string(&content);
                        // Check if source changed OR output file is missing
                        let source_changed = cache.is_dirty(&path, hash);
                        let output_missing = !output_path.exists();

                        if source_changed || output_missing {
                            if source_changed {
                                println!("  pages/{} (changed)", file_name.bright_yellow());
                            } else {
                                println!("  pages/{} (output missing)", file_name.bright_yellow());
                            }
                            match compile_at_to_vue(&path, &content, root_dir) {
                                Ok((vue_code, widgets, stores)) => {
                                    store_files.extend(stores);
                                    // Use file_stem for output path (matching VueProject::generate behavior)
                                    let widget_name = widgets.first().cloned().unwrap_or_else(|| file_stem.to_string());
                                    changed_files.push((output_path, vue_code, widget_name.clone()));
                                    let artifacts: Vec<UIArtifact> = widgets.iter().map(|w| {
                                        UIArtifact {
                                            source_path: path.clone(),
                                            widget_name: w.clone(),
                                            output_path: PathBuf::from(format!("src/pages/{}.vue", file_stem)),
                                            source_hash: hash,
                                            content_hash: hash_string(&changed_files.last().map(|f| f.1.as_str()).unwrap_or("")),
                                            backend: UIBackend::Vue,
                                        }
                                    }).collect();
                                    cache.update(path.clone(), hash, artifacts);
                                }
                                Err(e) => handle_compile_error(&path, &e)?,
                            }
                        } else {
                            println!("  pages/{} (cached)", file_name.bright_green());
                        }
                    }
                }
            }
        }
    }

    // Save cache
    cache.save(root_dir).ok();

    // Write changed files
    let changed_count = changed_files.len();
    if changed_count > 0 {
        println!("{} files changed, writing...", changed_count.to_string().bright_yellow());
        for (output_path, vue_code, _widget_name) in changed_files {
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::write(&output_path, &vue_code)
                .map_err(|e| format!("Failed to write {}: {}", output_path.display(), e))?;
            // Extract file name from output path for logging
            let file_name = output_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            println!("  ✓ Wrote {}.vue", file_name.bright_green());
        }
    } else {
        println!("{}", "No changes detected, using cached files".bright_green());
    }

    // Plan 012 Batch B (gap 9a): write store composables collected explicitly
    // from every compiled .at file above. Any residual STORE_EXTRA_FILES
    // entries (e.g. stashed by the Phase-1 cached-branch recompile) merge in
    // first; the explicitly-collected content wins on name conflicts.
    let mut store_map: std::collections::BTreeMap<String, String> =
        auto_lang::drain_store_extra_files().into_iter().collect();
    store_map.extend(store_files);
    if !store_map.is_empty() {
        let stores_dir = output_dir.join("src").join("stores");
        fs::create_dir_all(&stores_dir).ok();
        for (filename, content) in store_map {
            let clean_name = filename.strip_prefix("stores/").unwrap_or(&filename);
            let path = stores_dir.join(clean_name);
            fs::write(&path, &content).ok();
            println!("  ✓ Store composable: {}", path.display());
        }
    }

    Ok(changed_count)
}

/// Run Vue dev server (auto run command)
///
/// Steps:
/// 1. Incrementally compile changed .at files (see `incremental_compile_changed`)
/// 2. Generate project structure if not exists
/// 3. Generate API client code (if api.at exists)
/// 4. npm install
/// 5. Install shadcn-vue components
/// 6. Copy public assets
/// 7. npm run dev
pub fn run_vue_project(root_dir: &Path, args: Vec<String>) -> AutoResult<()> {
    println!("{}", "Running Vue dev server (backend: vue)".bright_cyan());

    let changed_count = incremental_compile_changed(root_dir)?;

    // Load project context
    let project = VueProject::from_workspace(root_dir)?;

    // Determine total steps based on whether project exists
    let total_steps = 6;
    let mut current_step = 0;

    // Step 1: Generate project structure if not exists, or regenerate source files
    current_step += 1;
    println!();
    if !project.exists() && changed_count == 0 {
        // Fresh project: no incremental changes were written, so generate everything
        println!("▶ Step {}/{}: Generating Vue project...", current_step, total_steps);
        project.generate()?;
    } else if !project.exists() && changed_count > 0 {
        // Project dir was removed but we already wrote files incrementally.
        // Only generate scaffolding (package.json, vite.config, etc), don't
        // overwrite the incrementally-written component .vue files.
        println!("▶ Step {}/{}: Generating project scaffolding...", current_step, total_steps);
        project.generate_scaffolding_only()?;
    } else if changed_count == 0 {
        println!("▶ Step {}/{}: Checking source files...", current_step, total_steps);
        // Self-heal router/index.ts — older scaffolding-only runs left it
        // missing, which breaks `import router from './router'` in main.ts.
        project.ensure_router_file()?;
    }

    // Copy handmade theme assets if available
    let handmade_css = root_dir.join("vue").join("src").join("assets").join("index.css");
    let gen_css = root_dir.join("gen").join("front").join("vue").join("src").join("assets").join("index.css");
    if handmade_css.exists() && gen_css.exists() {
        if let Ok(content) = fs::read_to_string(&handmade_css) {
            fs::write(&gen_css, content)
                .map_err(|e| format!("Failed to copy handmade index.css: {}", e))?;
            println!("{}", "  ✓ Copied handmade theme CSS".bright_green());
        }
    }
    let handmade_theme_toggle = root_dir.join("vue").join("src").join("components").join("ThemeToggle.vue");
    let gen_components_dir = root_dir.join("gen").join("front").join("vue").join("src").join("components");
    if handmade_theme_toggle.exists() {
        let gen_theme_toggle = gen_components_dir.join("ThemeToggle.vue");
        if let Ok(content) = fs::read_to_string(&handmade_theme_toggle) {
            fs::write(&gen_theme_toggle, content)
                .map_err(|e| format!("Failed to copy ThemeToggle.vue: {}", e))?;
            println!("{}", "  ✓ Copied ThemeToggle.vue".bright_green());
        }
    }

    // Step 2: Generate API client code (if api.at exists)
    current_step += 1;
    println!();
    println!("▶ Step {}/{}: Generating API client...", current_step, total_steps);
    if let Err(e) = crate::api_gen::generate_api(root_dir, "vue") {
        // API generation is optional - only warn on failure
        println!("  ⚠ API generation skipped: {}", e);
    }

    // Load API function names for Vue generator (dynamic detection)
    let api_fns_path = root_dir.join("dist").join(".api_functions");
    if api_fns_path.exists() {
        if let Ok(content) = fs::read_to_string(&api_fns_path) {
            let fns: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
            if !fns.is_empty() {
                // SAFETY: Setting a process-wide env var before spawning Vue generation
                unsafe { std::env::set_var("AUTO_API_FUNCTIONS", fns.join(",")); }
            }
        }
    }

    // Step 3: npm install
    current_step += 1;
    println!();
    println!("▶ Step {}/{}: Installing dependencies...", current_step, total_steps);
    project.npm_install()?;

    // Step 4: Install shadcn-vue components
    current_step += 1;
    println!();
    println!("▶ Step {}/{}: Installing shadcn-vue components...", current_step, total_steps);
    project.install_shadcn_components()?;

    // Step 5: Copy public assets
    current_step += 1;
    println!();
    println!("▶ Step {}/{}: Copying public assets...", current_step, total_steps);
    project.copy_public_assets()?;

    // Step 5.5: Start API backend server.
    // Plan 346: --server=vm starts AutoVM HTTP server; --server=rust (default)
    // starts the a2r-generated Rust axum server.
    let mut _api_child: Option<std::process::Child> = None;
    let backend_impl = std::env::var("AUTO_BACKEND_IMPL").unwrap_or_else(|_| "rust".to_string());
    if backend_impl == "vm" {
        // Vue+VM: AutoVM HTTP server as backend.
        crate::rust_ui::start_vm_server(root_dir);
    } else {
        // Vue+Rust: a2r-generated Rust axum server.
        if let Some(child) = crate::rust_ui::start_api_server(root_dir) {
            _api_child = Some(child);
        }
    }

    // Step 6: npm run dev
    current_step += 1;
    println!();
    println!("▶ Step {}/{}: Starting dev server...", current_step, total_steps);
    project.npm_run_dev(args)?;

    // Cleanup: stop API backend server when dev server exits
    if let Some(mut child) = _api_child {
        let _ = child.kill();
        println!("  ✓ API server (Rust) stopped");
    } else if backend_impl == "vm" {
        // VM server runs on a background thread — process exit cleans it up.
        println!("  ✓ API server (VM) stopped");
    }

    Ok(())
}

/// Check if a `use` statement imports from the API module (`back.api`)
fn is_api_use(use_stmt: &auto_lang::ast::Use) -> bool {
    // Check legacy paths: ["back", "api"]
    if use_stmt.paths.len() == 2
        && use_stmt.paths[0].as_str() == "back"
        && use_stmt.paths[1].as_str() == "api"
    {
        return true;
    }
    // Check Plan 131 module_path: display == "back.api"
    if let Some(ref mp) = use_stmt.module_path {
        if mp.display() == "back.api" {
            return true;
        }
    }
    false
}

/// Validate that imported API function names exist in the API manifest
fn validate_api_imports(imports: &[String], root_dir: &Path) -> Result<(), String> {
    let manifest_path = root_dir.join("dist").join(".api_functions");
    if !manifest_path.exists() {
        // No API module exists yet; skip validation
        return Ok(());
    }
    let known = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read .api_functions: {}", e))?;
    let known_names: Vec<&str> = known.lines().filter(|l| !l.trim().is_empty()).collect();
    for import in imports {
        let lower = import.to_lowercase();
        if !known_names.iter().any(|k| k.eq_ignore_ascii_case(&lower)) {
            return Err(format!(
                "Unknown API function '{}' in use statement. Available: {}",
                import,
                known_names.join(", ")
            ));
        }
    }
    Ok(())
}

/// Compile a .at file to Vue component
/// Returns (vue_code, widget_names)
/// Resolve streaming API endpoints (`#[api] fn` returning `~Stream<T>`) from the
/// project's `back/api.at`, so the store composable can wire type-driven SSE.
/// (Plan 043 stream phase.) Delegates to the auto-lang regex-based resolver.
fn resolve_stream_endpoints(root_dir: &Path) -> Vec<auto_lang::aura::StreamEndpoint> {
    auto_lang::ui_gen::api::resolve_stream_endpoints_for_project(
        &root_dir.to_string_lossy(),
    )
}

/// Compile an .at file to Vue SFC (Plan 361 §3: uses generate_component_from_file).
/// Plan 012 Batch B (gap 9b): unified parse-failure semantics for the
/// incremental build path. Prints the same "Warning: Failed to compile ..."
/// line the fresh path (`VueProject::from_workspace`) prints — jade's regen
/// flow greps for that string — and, under `auto build --strict`, escalates
/// to a hard build failure (non-zero exit), matching from_workspace.
fn handle_compile_error(path: &Path, e: &str) -> Result<(), String> {
    if auto_lang::ui_gen::validators::strict_enabled() {
        return Err(format!("Failed to compile {}: {}", path.display(), e));
    }
    println!("{} Failed to compile {}: {}", "Warning:".bright_yellow(), path.display(), e);
    Ok(())
}

/// Compile an .at file to Vue SFC (Plan 361 §3: uses generate_component_from_file).
///
/// Returns (vue_code, widget_names, store_composables). The store composables
/// are returned explicitly — the STORE_EXTRA_FILES thread-local is cleared at
/// the start of every generate_component_from_file call, so draining it after
/// compiling several files only ever yields the LAST file's stores
/// (Plan 012 Batch B, gap 9a).
fn compile_at_to_vue(at_path: &Path, _content: &str, root_dir: &Path) -> Result<(String, Vec<String>, Vec<(String, String)>), String> {
    use auto_lang::ui_gen::{generate_component_from_file, ComponentGenOptions};

    let opts = ComponentGenOptions {
        root_dir_for_validation: Some(root_dir.to_path_buf()),
        stream_endpoints: Some(resolve_stream_endpoints(root_dir)),
        ..Default::default()
    };
    let result = generate_component_from_file(at_path, opts)
        .map_err(|e| format!("{}", e))?;

    // Plan 012 Batch A: surface codegen validation warnings (deduplicated).
    auto_lang::ui_gen::validators::print_warnings_once(
        &at_path.display().to_string(),
        &result.validation_warnings,
    );

    // Validate API imports against the manifest (auto-man-specific)
    if !result.detected_api_imports.is_empty() {
        validate_api_imports(&result.detected_api_imports, root_dir)?;
    }

    let names: Vec<String> = result.widgets.iter().map(|w| w.name.clone()).collect();
    Ok((result.vue_code, names, result.store_composables))
}

/// Compile an .at file to Vue SFC with known sub-widget names (Plan 361 §3: uses generate_component_from_file).
/// See `compile_at_to_vue` for the return contract.
fn compile_at_to_vue_with_sub_widgets(at_path: &Path, _content: &str, sub_widget_names: Vec<String>, root_dir: &Path) -> Result<(String, Vec<String>, Vec<(String, String)>), String> {
    use auto_lang::ui_gen::{generate_component_from_file, ComponentGenOptions};

    let opts = ComponentGenOptions {
        sub_widgets: Some(sub_widget_names),
        root_dir_for_validation: Some(root_dir.to_path_buf()),
        stream_endpoints: Some(resolve_stream_endpoints(root_dir)),
        ..Default::default()
    };
    let result = generate_component_from_file(at_path, opts)
        .map_err(|e| format!("{}", e))?;

    auto_lang::ui_gen::validators::print_warnings_once(
        &at_path.display().to_string(),
        &result.validation_warnings,
    );

    if !result.detected_api_imports.is_empty() {
        validate_api_imports(&result.detected_api_imports, root_dir)?;
    }

    let names: Vec<String> = result.widgets.iter().map(|w| w.name.clone()).collect();
    Ok((result.vue_code, names, result.store_composables))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_style_files_inline_array() {
        let content = r#"name: "demo"
version: "1.0.0"
styles: ["src/front/autodown-editor.css", "src/front/theme.css"]
"#;
        let files = parse_style_files(content);
        assert_eq!(
            files,
            vec![
                "src/front/autodown-editor.css".to_string(),
                "src/front/theme.css".to_string()
            ]
        );
    }

    #[test]
    fn test_parse_style_files_single_string() {
        let content = "name: \"demo\"\nstyles: \"src/front/autodown-editor.css\"\n";
        let files = parse_style_files(content);
        assert_eq!(files, vec!["src/front/autodown-editor.css".to_string()]);
    }

    #[test]
    fn test_parse_style_files_absent() {
        let content = "name: \"demo\"\nrender: \"vue\"\n";
        assert!(parse_style_files(content).is_empty());
    }

    #[test]
    fn test_generate_main_ts_imports_style_files() {
        let styles = vec!["autodown-editor.css".to_string(), "theme.css".to_string()];
        let main_ts = generate_main_ts(false, false, &styles, &I18nConfig::default(), &[]);
        assert!(main_ts.contains("import './styles/autodown-editor.css'"), "main.ts:\n{}", main_ts);
        assert!(main_ts.contains("import './styles/theme.css'"), "main.ts:\n{}", main_ts);

        // Without styles: no imports, same as before.
        let plain = generate_main_ts(false, false, &[], &I18nConfig::default(), &[]);
        assert!(!plain.contains("./styles/"), "main.ts:\n{}", plain);
    }

    // ====================================================================
    // Plan musk-022 Phase 2: i18n (vue-i18n) support
    // ====================================================================

    #[test]
    fn test_parse_i18n_true() {
        let content = "name: \"demo\"\ni18n: true\n";
        let cfg = parse_i18n(content);
        assert!(cfg.enabled);
        assert!(cfg.locale_files.is_empty());
    }

    #[test]
    fn test_parse_i18n_locale_files() {
        let content = "name: \"demo\"\ni18n: [\"src/i18n/locales/en.json\", \"src/i18n/locales/zh.json\"]\n";
        let cfg = parse_i18n(content);
        assert!(cfg.enabled);
        assert_eq!(
            cfg.locale_files,
            vec!["src/i18n/locales/en.json".to_string(), "src/i18n/locales/zh.json".to_string()]
        );
    }

    #[test]
    fn test_parse_i18n_single_locale() {
        let content = "name: \"demo\"\ni18n: \"src/locales/en.json\"\n";
        let cfg = parse_i18n(content);
        assert!(cfg.enabled);
        assert_eq!(cfg.locale_files, vec!["src/locales/en.json".to_string()]);
    }

    #[test]
    fn test_parse_i18n_absent() {
        let content = "name: \"demo\"\nrender: \"vue\"\n";
        let cfg = parse_i18n(content);
        assert!(!cfg.enabled);
    }

    #[test]
    fn test_generate_main_ts_injects_i18n() {
        let cfg = I18nConfig {
            enabled: true,
            locale_files: vec!["src/i18n/locales/en.json".to_string(), "src/i18n/locales/zh.json".to_string()],
        };
        let locales = vec!["en.json".to_string(), "zh.json".to_string()];
        let main_ts = generate_main_ts(false, false, &[], &cfg, &locales);
        // createI18n + vue-i18n import.
        assert!(main_ts.contains("import { createI18n } from 'vue-i18n'"), "main.ts:\n{}", main_ts);
        assert!(main_ts.contains("const i18n = createI18n("), "main.ts:\n{}", main_ts);
        // Locale imports keyed by filename stem.
        assert!(main_ts.contains("import en from './locales/en.json'"), "main.ts:\n{}", main_ts);
        assert!(main_ts.contains("import zh from './locales/zh.json'"), "main.ts:\n{}", main_ts);
        // messages object includes both locales.
        assert!(main_ts.contains("messages: {"), "main.ts:\n{}", main_ts);
        // app.use(i18n) before mount.
        assert!(main_ts.contains("app.use(i18n)"), "main.ts:\n{}", main_ts);
    }

    #[test]
    fn test_generate_main_ts_no_i18n_when_disabled() {
        let main_ts = generate_main_ts(false, false, &[], &I18nConfig::default(), &[]);
        assert!(!main_ts.contains("vue-i18n"), "main.ts should not mention i18n:\n{}", main_ts);
        assert!(!main_ts.contains("createI18n"), "main.ts:\n{}", main_ts);
    }

    #[test]
    fn test_generate_package_json_includes_vue_i18n() {
        let pkg = generate_package_json("demo", false, true, &[]);
        assert!(pkg.contains("\"vue-i18n\": \"^9.14.0\""), "package.json:\n{}", pkg);
    }

    #[test]
    fn test_generate_package_json_no_vue_i18n_when_disabled() {
        let pkg = generate_package_json("demo", false, false, &[]);
        assert!(!pkg.contains("vue-i18n"), "package.json should not mention i18n:\n{}", pkg);
    }


    #[test]
    fn test_copy_style_files_byte_for_byte() {
        // CSS content with bytes that must survive verbatim: CSS variables,
        // pseudo-classes, comments, CRLF-free newlines, non-ASCII.
        let css: &[u8] = b"/* autodown editor theme */\n:root {\n  --ad-bg: #1e1e1e;\n}\n.autodown-editor:hover {\n  border-color: var(--ad-bg);\n}\n.autodown-editor .c\xC3\xA9 {\n  color: red;\n}\n";

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("proj");
        let front = root.join("src/front");
        fs::create_dir_all(&front).unwrap();
        fs::write(front.join("autodown-editor.css"), css).unwrap();

        let project = VueProject {
            root_dir: root.clone(),
            output_dir: root.join("gen/front/vue"),
            name: "demo".to_string(),
            front_dir: front.clone(),
            public_dir: front.join("public"),
            shadcn_components: vec![],
            has_routes: false,
            app_vue_code: String::new(),
            components: vec![],
            routes: vec![],
            npm_deps: vec![],
            style_files: vec!["src/front/autodown-editor.css".to_string()],
            ext_files: vec![],
            store_files: vec![],
            i18n: I18nConfig::default(),
        };

        let copied = project.copy_style_files().unwrap();
        assert_eq!(copied, vec!["autodown-editor.css".to_string()]);

        let out = project.output_dir.join("src/styles/autodown-editor.css");
        let bytes = fs::read(&out).unwrap();
        assert_eq!(bytes, css, "copied CSS must be byte-for-byte identical");
    }

    #[test]
    fn test_copy_style_files_missing_file_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("proj");
        fs::create_dir_all(&root).unwrap();

        let project = VueProject {
            root_dir: root.clone(),
            output_dir: root.join("gen/front/vue"),
            name: "demo".to_string(),
            front_dir: root.clone(),
            public_dir: root.join("public"),
            shadcn_components: vec![],
            has_routes: false,
            app_vue_code: String::new(),
            components: vec![],
            routes: vec![],
            npm_deps: vec![],
            style_files: vec!["src/front/nope.css".to_string()],
            ext_files: vec![],
            store_files: vec![],
            i18n: I18nConfig::default(),
        };

        assert!(project.copy_style_files().is_err());
    }

    #[test]
    fn test_is_local_ext_path() {
        // Project-local files (copied into src/ext/)
        assert!(is_local_ext_path("src/front/utils/greet.ts"));
        assert!(is_local_ext_path("src/front/components/FancyBadge.vue"));
        assert!(is_local_ext_path("./utils/x.tsx"));
        assert!(is_local_ext_path("../shared/y.mjs"));
        assert!(is_local_ext_path("src/front/lib/z.js"));
        // npm package specifiers (left as-is)
        assert!(!is_local_ext_path("lucide-vue-next"));
        assert!(!is_local_ext_path("@autodown/editor"));
        assert!(!is_local_ext_path("marked"));
    }

    #[test]
    fn test_collect_ext_import_files() {
        let src = r#"
widget App {
    use {
        fn: greet from "src/front/utils/greet.ts"
        fn: marked from "marked"
        component: FancyBadge from "src/front/components/FancyBadge.vue"
        composable: useClock from "./src/front/composables/useClock.ts"
    }
    view { div { "hi" } }
}
"#;
        let session = auto_lang::session::CompilerSession::ui();
        let mut parser = auto_lang::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let widgets: Vec<AuraWidget> = ast
            .stmts
            .iter()
            .filter_map(|s| match s {
                auto_lang::ast::Stmt::WidgetDecl(d) => {
                    auto_lang::aura::extract_widget_from_decl(d).ok()
                }
                _ => None,
            })
            .collect();
        assert_eq!(widgets.len(), 1);

        let mut set = std::collections::BTreeSet::new();
        collect_ext_import_files(&widgets, &mut set);
        let files: Vec<String> = set.into_iter().collect();
        // npm specifier excluded; "./" normalized away; deduped + sorted.
        assert_eq!(
            files,
            vec![
                "src/front/components/FancyBadge.vue".to_string(),
                "src/front/composables/useClock.ts".to_string(),
                "src/front/utils/greet.ts".to_string(),
            ]
        );
    }

    #[test]
    fn test_copy_ext_files_preserves_layout() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("proj");
        let utils = root.join("src/front/utils");
        fs::create_dir_all(&utils).unwrap();
        fs::write(utils.join("greet.ts"), "export const greet = 1\n").unwrap();

        let project = VueProject {
            root_dir: root.clone(),
            output_dir: root.join("gen/front/vue"),
            name: "demo".to_string(),
            front_dir: root.join("src/front"),
            public_dir: root.join("public"),
            shadcn_components: vec![],
            has_routes: false,
            app_vue_code: String::new(),
            components: vec![],
            routes: vec![],
            npm_deps: vec![],
            style_files: vec![],
            ext_files: vec!["src/front/utils/greet.ts".to_string()],
            store_files: vec![],
            i18n: I18nConfig::default(),
        };

        let copied = project.copy_ext_files().unwrap();
        assert_eq!(copied, vec!["src/front/utils/greet.ts".to_string()]);
        let out = project.output_dir.join("src/ext/src/front/utils/greet.ts");
        assert_eq!(fs::read_to_string(&out).unwrap(), "export const greet = 1\n");
    }

    #[test]
    fn test_copy_ext_files_rejects_escaping_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("proj");
        fs::create_dir_all(&root).unwrap();

        let project = VueProject {
            root_dir: root.clone(),
            output_dir: root.join("gen/front/vue"),
            name: "demo".to_string(),
            front_dir: root.clone(),
            public_dir: root.join("public"),
            shadcn_components: vec![],
            has_routes: false,
            app_vue_code: String::new(),
            components: vec![],
            routes: vec![],
            npm_deps: vec![],
            style_files: vec![],
            ext_files: vec!["../outside/x.ts".to_string()],
            store_files: vec![],
            i18n: I18nConfig::default(),
        };

        let err = project.copy_ext_files().unwrap_err().to_string();
        assert!(err.contains("escapes the project root"), "err: {}", err);
    }

    // --- Plan 012 Batch B: incremental store emission + failure semantics ---

    const BATCH_B_APP_AT: &str = r#"
widget App {
    view {
        col {
            text "hello"
        }
    }
}
"#;

    const BATCH_B_ALPHA_STORE: &str = r#"
store AlphaStore {
    model {
        var items []str = []
    }
    msg Msg { Touch }
    on {
        .Touch -> { }
    }
}
"#;

    const BATCH_B_BETA_STORE: &str = r#"
store BetaStore {
    model {
        var count int = 0
    }
    msg Msg { Bump }
    on {
        .Bump -> { .count = .count + 1 }
    }
}
"#;

    /// Create a minimal two-store workspace in a temp dir.
    fn make_multi_store_workspace(root: &Path) {
        fs::write(root.join("pac.at"), "name: \"multistore\"\n").unwrap();
        let front = root.join("src").join("front");
        fs::create_dir_all(&front).unwrap();
        fs::write(front.join("app.at"), BATCH_B_APP_AT).unwrap();
        fs::write(front.join("alpha_store.at"), BATCH_B_ALPHA_STORE).unwrap();
        fs::write(front.join("beta_store.at"), BATCH_B_BETA_STORE).unwrap();
    }

    /// Gap 9a: with TWO store .at files changed in one incremental build,
    /// BOTH store composables must be (re-)emitted. Before Batch B the
    /// incremental path drained the STORE_EXTRA_FILES thread-local, which is
    /// cleared per compiled file — only the LAST store survived.
    #[test]
    fn test_incremental_build_emits_all_store_composables() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        make_multi_store_workspace(root);
        let stores_dir = root.join("gen").join("front").join("vue").join("src").join("stores");

        // First incremental pass: everything is dirty, both stores emitted.
        let changed = incremental_compile_changed(root).expect("first pass must succeed");
        assert!(changed > 0);
        assert!(
            stores_dir.join("useAlphaStoreStore.ts").exists(),
            "alpha composable after first pass"
        );
        assert!(
            stores_dir.join("useBetaStoreStore.ts").exists(),
            "beta composable after first pass"
        );

        // Delete the generated stores and touch BOTH store sources: the
        // incremental path must re-emit BOTH, not just the last one.
        fs::remove_dir_all(&stores_dir).unwrap();
        fs::write(
            root.join("src").join("front").join("alpha_store.at"),
            format!("{}\n// touched\n", BATCH_B_ALPHA_STORE),
        )
        .unwrap();
        fs::write(
            root.join("src").join("front").join("beta_store.at"),
            format!("{}\n// touched\n", BATCH_B_BETA_STORE),
        )
        .unwrap();

        incremental_compile_changed(root).expect("second pass must succeed");
        assert!(
            stores_dir.join("useAlphaStoreStore.ts").exists(),
            "alpha composable re-emitted by incremental pass"
        );
        assert!(
            stores_dir.join("useBetaStoreStore.ts").exists(),
            "beta composable re-emitted by incremental pass"
        );
    }

    /// Gap 9b: a parse failure in the incremental path must NOT be swallowed.
    /// Non-strict: the build continues (Warning printed, matching the fresh
    /// path) and the broken file's composable is not written. Strict
    /// (`auto build --strict`): the build fails. Both assertions live in ONE
    /// test because strict mode is a process-wide flag and cargo runs tests
    /// in parallel.
    #[test]
    fn test_incremental_parse_failure_semantics() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        make_multi_store_workspace(root);
        // Break the beta store (unterminated block → parse error).
        fs::write(
            root.join("src").join("front").join("beta_store.at"),
            "store BetaStore {\n    model {\n        var count int = \n",
        )
        .unwrap();

        // Non-strict: warning + build continues.
        auto_lang::ui_gen::validators::set_strict(false);
        let ok = incremental_compile_changed(root);
        assert!(ok.is_ok(), "non-strict build must continue: {:?}", ok.err());
        let stores_dir = root.join("gen").join("front").join("vue").join("src").join("stores");
        assert!(
            stores_dir.join("useAlphaStoreStore.ts").exists(),
            "healthy store still emitted"
        );
        assert!(
            !stores_dir.join("useBetaStoreStore.ts").exists(),
            "broken store must not emit a composable"
        );

        // Strict: the same parse failure fails the build.
        struct StrictGuard;
        impl StrictGuard {
            fn on() -> Self {
                auto_lang::ui_gen::validators::set_strict(true);
                StrictGuard
            }
        }
        impl Drop for StrictGuard {
            fn drop(&mut self) {
                auto_lang::ui_gen::validators::set_strict(false);
            }
        }
        let _guard = StrictGuard::on();
        let err = match incremental_compile_changed(root) {
            Ok(_) => panic!("strict build must fail on parse error"),
            Err(e) => e,
        };
        assert!(
            err.to_string().contains("Failed to compile"),
            "error should name the compile failure: {}",
            err
        );
    }
}
