//! Vue project generation and build utilities
//!
//! This module provides the complete Vue + shadcn-vue project workflow:
//! 1. Generate project structure (package.json, vite.config.ts, etc.)
//! 2. bun install (or npm install as fallback)
//! 3. Install shadcn-vue components
//! 4. Build (bun run build) or Run dev server (bun run dev)

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use colored::Colorize;
use auto_lang::aura::AuraRoute;
use auto_lang::database::{UIArtifact, UIBackend, UICache};
use auto_lang::ui_gen::VueGenerator;

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

fn generate_package_json(name: &str, has_routes: bool) -> String {
    let router_dep = if has_routes {
        r#"    "vue-router": "^4.2.0",
"#
    } else {
        ""
    };

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
    "vue": "^3.4.0",
{}    "@vueuse/core": "^10.7.0",
    "reka-ui": "^2.0.0",
    "class-variance-authority": "^0.7.0",
    "clsx": "^2.1.0",
    "tailwind-merge": "^2.2.0",
    "lucide-vue-next": "^0.312.0",
    "prismjs": "^1.29.0",
    "embla-carousel-vue": "^8.5.1",
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
"#, name, router_dep)
}

fn generate_vite_config() -> String {
    r#"import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
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
    port: 3000,
    // Only auto-open browser when NOT running under Tauri
    // Tauri sets TAURI_ENV before running vite
    open: !process.env.TAURI_ENV,
    // Proxy API requests to Rust backend
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      }
    }
  }
})
"#.to_string()
}

fn generate_tsconfig() -> String {
    r#"{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
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
    format!(r#"<!DOCTYPE html>
<html lang="en">
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

fn generate_main_ts(has_routes: bool) -> String {
    let base = r#"import { createApp } from 'vue'
import App from './App.vue'
import './assets/index.css'
import 'prismjs/themes/prism-tomorrow.css'
import Prism from 'prismjs'

// Define custom 'auto' language for Prism
Prism.languages.auto = {
  'comment': /\/\/.*|\/\*[\s\S]*?\*\//,
  'string': {
    pattern: /f?"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'/,
    greedy: true
  },
  'keyword': /\b(?:widget|view|model|msg|fn|let|mut|const|if|else|for|in|return|use|type|spec|import|export|struct|enum|interface|extends|implements|new|true|false|null)\b/,
  'function': /\b[a-z_][a-z0-9_]*(?=\s*\()/i,
  'number': /\b\d+\.?\d*\b/,
  'operator': /[+\-*/%=<>!&|^~?:]+/,
  'punctuation': /[{}[\]();,.]/,
  'property': /\.[a-z_][a-z0-9_]*/i,
  'element': /\b(?:col|row|button|text|input|card|link|div|span|p|h1|h2|h3|h4|h5|h6|ul|ol|li|table|thead|tbody|tr|td|th|form|label|checkbox|switch|select|option|dialog|modal|toast|dropdown|menu|tab|tabs|accordion|badge|avatar|progress|slider|scroll|codeblock|pre|code|img|video|audio|canvas|svg|path|rect|circle|ellipse|line|polyline|polygon|header|footer|nav|main|aside|section|article|header|footer|sidebar|outlet|slot)\b/,
  'attr': /\([^)]*\)/,
};
"#;
    if has_routes {
        format!("{}\nimport router from './router'\n\nconst app = createApp(App)\napp.use(router)\napp.mount('#app')\n", base)
    } else {
        format!("{}\ncreateApp(App).mount('#app')\n", base)
    }
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

    --primary: 222.2 47.4% 11.2%;
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
    --ring: 222.2 84% 4.9%;

    --radius: 0.5rem;
  }

  .dark {
    --background: 222.2 84% 4.9%;
    --foreground: 210 40% 98%;

    --card: 222.2 84% 4.9%;
    --card-foreground: 210 40% 98%;

    --popover: 222.2 84% 4.9%;
    --popover-foreground: 210 40% 98%;

    --primary: 210 40% 98%;
    --primary-foreground: 222.2 47.4% 11.2%;

    --secondary: 217.2 32.6% 17.5%;
    --secondary-foreground: 210 40% 98%;

    --muted: 217.2 32.6% 17.5%;
    --muted-foreground: 215 20.2% 65.1%;

    --accent: 217.2 32.6% 17.5%;
    --accent-foreground: 210 40% 98%;

    --destructive: 0 62.8% 30.6%;
    --destructive-foreground: 210 40% 98%;

    --border: 217.2 32.6% 17.5%;
    --input: 217.2 32.6% 17.5%;
    --ring: 212.7 26.8% 83.9%;
  }
}

@layer base {
  * {
    @apply border-border;
  }
  body {
    @apply bg-background text-foreground;
  }
}
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

/// Write all project files
fn write_project_files(
    output_path: &Path,
    name: &str,
    vue_code: &str,
    _components: &[String],
    has_routes: bool,
) -> Result<(), String> {
    // package.json
    let package_json = generate_package_json(name, has_routes);
    fs::write(output_path.join("package.json"), package_json)
        .map_err(|e| format!("Failed to write package.json: {}", e))?;

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
    let main_ts = generate_main_ts(has_routes);
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
}

impl VueProject {
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
        let output_dir = root_dir.join("gen").join("vue");
        let public_dir = front_dir.join("public");

        // Compile .at files
        let mut all_components: Vec<(String, String, String, String)> = Vec::new();
        let mut all_shadcn_components = HashSet::new();
        let mut all_routes: Vec<AuraRoute> = Vec::new();

        // Process app.at
        if app_at.exists() {
            match auto_lang::ui_build_shadcn_with_widgets(app_at.to_str().unwrap(), None) {
                Ok((vue_code, widgets)) => {
                    let components = detect_shadcn_components(&vue_code);
                    for comp in &components {
                        all_shadcn_components.insert(comp.clone());
                    }
                    for widget in &widgets {
                        if let Some(ref routes) = widget.routes {
                            all_routes.extend(routes.routes.clone());
                        }
                    }
                    let widget_name = widgets.first().map(|w| w.name.as_str()).unwrap_or("App");
                    all_components.push(("".to_string(), "app".to_string(), vue_code, widget_name.to_string()));
                }
                Err(e) => {
                    println!("{} {}", "Warning: Failed to compile app.at:".bright_yellow(), e);
                }
            }
        }

        // Process pages/ directory
        let pages_dir = front_dir.join("pages");
        if pages_dir.exists() {
            for entry in fs::read_dir(&pages_dir)
                .map_err(|e| format!("Failed to read pages directory: {}", e))?
            {
                let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                let path = entry.path();

                if path.extension().map(|e| e == "at").unwrap_or(false) {
                    let file_stem = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("page");

                    match auto_lang::ui_build_shadcn_with_widgets(path.to_str().unwrap(), None) {
                        Ok((vue_code, widgets)) => {
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
                            all_components.push(("pages".to_string(), file_stem.to_string(), vue_code, widget_name.to_string()));
                        }
                        Err(e) => {
                            println!("{} Failed to compile {}: {}", "Warning:".bright_yellow(), path.display(), e);
                        }
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
        })
    }

    /// Check if the project structure already exists
    pub fn exists(&self) -> bool {
        self.output_dir.exists() && self.output_dir.join("package.json").exists()
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

        // Write project files
        write_project_files(
            &self.output_dir,
            &self.name,
            &self.app_vue_code,
            &self.shadcn_components,
            self.has_routes,
        )?;

        // Generate router files if routes detected
        if self.has_routes {
            let router_dir = self.output_dir.join("src/router");
            fs::create_dir_all(&router_dir)
                .map_err(|e| format!("Failed to create src/router: {}", e))?;

            let router_content = VueGenerator::generate_router_file(&self.routes);
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

        println!("{}", "✓ Generated project files".bright_green());

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

        // Regenerate main.ts
        let main_ts_content = generate_main_ts(self.has_routes);
        let main_ts_path = src_dir.join("main.ts");
        fs::write(&main_ts_path, &main_ts_content)
            .map_err(|e| format!("Failed to write main.ts: {}", e))?;
        println!("{}", "  ✓ Regenerated main.ts".bright_green());

        // Regenerate tsconfig.json
        let tsconfig_path = self.output_dir.join("tsconfig.json");
        let tsconfig = generate_tsconfig();
        fs::write(&tsconfig_path, &tsconfig)
            .map_err(|e| format!("Failed to write tsconfig.json: {}", e))?;
        println!("{}", "  ✓ Regenerated tsconfig.json".bright_green());

        // Regenerate package.json if outdated (e.g., missing @types/prismjs)
        let pkg_path = self.output_dir.join("package.json");
        if pkg_path.exists() {
            let existing_pkg = fs::read_to_string(&pkg_path)
                .map_err(|e| format!("Failed to read package.json: {}", e))?;
            if !existing_pkg.contains("@types/prismjs") {
                let new_pkg = generate_package_json(&self.name, self.has_routes);
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

            let router_content = VueGenerator::generate_router_file(&self.routes);
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
        if !crate::pkg::command_exists(crate::pkg::install_cmd()) {
            println!("{}", format!("⚠ {} not found. Please install it or Node.js.", pm).bright_yellow());
            return Err(format!("{} not found", pm).into());
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
                println!("{} {}", "  ✗ Failed:".bright_red(), e);
                Err(format!("{} install failed: {}", pm, e).into())
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
                Ok(())
            }
            Err(e) => {
                println!("{} {}", "  ✗ Failed:".bright_red(), e);
                println!("  You may need to run '{} shadcn-vue@latest add {} -y' manually.", crate::pkg::exec_cmd(), self.shadcn_components.join(" "));
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
/// 1. Generate project structure if not exists
/// 2. npm install
/// 3. Install shadcn-vue components
/// 4. Copy public assets
/// 5. npm run build
pub fn build_vue_project(root_dir: &Path) -> AutoResult<()> {
    println!("{}", "Building Vue project (backend: vue)".bright_cyan());

    // Load project context
    let project = VueProject::from_workspace(root_dir)?;

    // Step 1: Generate project structure if not exists, or regenerate source files if exists
    let total_steps = if project.exists() { 5 } else { 6 };
    let mut current_step = 0;

    if !project.exists() {
        current_step += 1;
        println!();
        println!("▶ Step {}/{}: Generating Vue project...", current_step, total_steps);
        project.generate()?;
    } else {
        // Regenerate source files even if project exists
        current_step += 1;
        println!();
        println!("▶ Step {}/{}: Regenerating source files...", current_step, total_steps);
        project.regenerate_source_files()?;
    }

    // Step 2: Generate API client code (if api.at exists)
    current_step += 1;
    println!();
    println!("▶ Step {}/{}: Generating API client...", current_step, total_steps);
    if let Err(e) = crate::api_gen::generate_api(root_dir, "vue") {
        // API generation is optional - only warn on failure
        println!("  ⚠ API generation skipped: {}", e);
    }

    // Copy handmade theme assets if available
    let handmade_css = root_dir.join("vue").join("src").join("assets").join("index.css");
    let gen_css = root_dir.join("gen").join("vue").join("src").join("assets").join("index.css");
    if handmade_css.exists() && gen_css.exists() {
        if let Ok(content) = fs::read_to_string(&handmade_css) {
            fs::write(&gen_css, content)
                .map_err(|e| format!("Failed to copy handmade index.css: {}", e))?;
            println!("{}", "  ✓ Copied handmade theme CSS".bright_green());
        }
    }
    let handmade_theme_toggle = root_dir.join("vue").join("src").join("components").join("ThemeToggle.vue");
    let gen_components_dir = root_dir.join("gen").join("vue").join("src").join("components");
    if handmade_theme_toggle.exists() {
        let gen_theme_toggle = gen_components_dir.join("ThemeToggle.vue");
        if let Ok(content) = fs::read_to_string(&handmade_theme_toggle) {
            fs::write(&gen_theme_toggle, content)
                .map_err(|e| format!("Failed to copy ThemeToggle.vue: {}", e))?;
            println!("{}", "  ✓ Copied ThemeToggle.vue".bright_green());
        }
    }

    // Step 3: npm install
    current_step += 1;
    println!();
    println!("▶ Step {}/{}: Installing dependencies...", current_step, total_steps);
    project.npm_install()?;

    // Step 3: Install shadcn-vue components
    current_step += 1;
    println!();
    println!("▶ Step {}/{}: Installing shadcn-vue components...", current_step, total_steps);
    project.install_shadcn_components()?;

    // Step 4: Copy public assets
    current_step += 1;
    println!();
    println!("▶ Step {}/{}: Copying public assets...", current_step, total_steps);
    project.copy_public_assets()?;

    // Step 5: npm run build
    current_step += 1;
    println!();
    println!("▶ Step {}/{}: Building Vue project...", current_step, total_steps);
    project.npm_build()?;

    Ok(())
}

/// Run Vue dev server (auto run command)
///
/// Steps:
/// 1. Generate project structure if not exists, or regenerate source files if exists
/// 2. Generate API client code (if api.at exists)
/// 3. npm install
/// 4. Install shadcn-vue components
/// 5. Copy public assets
/// 6. npm run dev
pub fn run_vue_project(root_dir: &Path, args: Vec<String>) -> AutoResult<()> {
    println!("{}", "Running Vue dev server (backend: vue)".bright_cyan());

    // Resolve front directory using same logic as VueProject::from_workspace
    let front_dir = resolve_front_dir(root_dir);
    let output_dir = root_dir.join("gen").join("vue");

    // Load cache for incremental compilation
    let mut cache = UICache::load(root_dir);
    let mut changed_files: Vec<(PathBuf, String, String)> = Vec::new(); // (output_path, vue_code, widget_name)

    // Check app.at for changes
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
                if let Ok((vue_code, widgets)) = compile_at_to_vue(&app_at, &content) {
                    if let Some(widget_name) = widgets.first() {
                        changed_files.push((app_output_path, vue_code, widget_name.clone()));
                    }
                    let artifacts: Vec<UIArtifact> = widgets.iter().map(|w| {
                        UIArtifact {
                            source_path: app_at.clone(),
                            widget_name: w.clone(),
                            output_path: PathBuf::from(format!("src/App.vue")),
                            source_hash: hash,
                            content_hash: hash_string(&changed_files.first().map(|f| f.1.as_str()).unwrap_or("")),
                            backend: UIBackend::Vue,
                        }
                    }).collect();
                    cache.update(app_at.clone(), hash, artifacts);
                }
            } else {
                println!("  {} (cached)", "app.at".bright_green());
            }
        }
    }

    // Check widgets/ directory for changes
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
                            if let Ok((vue_code, widgets)) = compile_at_to_vue(&path, &content) {
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
                        } else {
                            println!("  widgets/{} (cached)", file_name.bright_green());
                        }
                    }
                }
            }
        }
    }

    // Check pages/ directory for changes
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
                            if let Ok((vue_code, widgets)) = compile_at_to_vue(&path, &content) {
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

    // Load project context
    let project = VueProject::from_workspace(root_dir)?;

    // Determine total steps based on whether project exists
    let total_steps = 6;
    let mut current_step = 0;

    // Step 1: Generate project structure if not exists, or regenerate source files
    current_step += 1;
    println!();
    if !project.exists() {
        println!("▶ Step {}/{}: Generating Vue project...", current_step, total_steps);
        project.generate()?;
    } else if changed_count == 0 {
        // Only regenerate if no incremental changes were detected
        // This handles the case where output files are missing but source hasn't changed
        println!("▶ Step {}/{}: Checking source files...", current_step, total_steps);
        // Skip regeneration if we already did incremental updates
    }

    // Copy handmade theme assets if available
    let handmade_css = root_dir.join("vue").join("src").join("assets").join("index.css");
    let gen_css = root_dir.join("gen").join("vue").join("src").join("assets").join("index.css");
    if handmade_css.exists() && gen_css.exists() {
        if let Ok(content) = fs::read_to_string(&handmade_css) {
            fs::write(&gen_css, content)
                .map_err(|e| format!("Failed to copy handmade index.css: {}", e))?;
            println!("{}", "  ✓ Copied handmade theme CSS".bright_green());
        }
    }
    let handmade_theme_toggle = root_dir.join("vue").join("src").join("components").join("ThemeToggle.vue");
    let gen_components_dir = root_dir.join("gen").join("vue").join("src").join("components");
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

    // Step 6: npm run dev
    current_step += 1;
    println!();
    println!("▶ Step {}/{}: Starting dev server...", current_step, total_steps);
    project.npm_run_dev(args)?;

    Ok(())
}

/// Compile a .at file to Vue component
/// Returns (vue_code, widget_names)
fn compile_at_to_vue(_at_path: &Path, content: &str) -> Result<(String, Vec<String>), String> {
    use auto_lang::Parser;
    use auto_lang::session::CompilerSession;
    use auto_lang::ui_gen::BackendGenerator;
    use auto_lang::aura::extract_widget_from_decl;

    let session = CompilerSession::ui().with_backend("vue");
    let mut parser = Parser::from(content);
    parser = parser.with_session(session);

    let ast = parser.parse().map_err(|e| format!("Parse error: {:?}", e))?;

    let mut widgets = Vec::new();
    for stmt in &ast.stmts {
        if let auto_lang::ast::Stmt::WidgetDecl(widget_decl) = stmt {
            let aura_widget = extract_widget_from_decl(widget_decl)
                .map_err(|e| e.to_string())?;
            widgets.push(aura_widget);
        }
    }

    if widgets.is_empty() {
        return Err("No widgets found".to_string());
    }

    // Use shadcn mode for proper component generation
    let mut generator = VueGenerator::new().with_mode(auto_lang::ui_gen::VueMode::Shadcn);
    let vue_code = generator.generate(&widgets[0])
        .map_err(|e| e.to_string())?;

    let names: Vec<String> = widgets.iter().map(|w| w.name.clone()).collect();
    Ok((vue_code, names))
}
