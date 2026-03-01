//! `auto vue` command - Generate complete Vue + shadcn-vue project from AURA file
//!
//! Usage:
//!   auto vue input.at -o ./my-app
//!   auto vue input.at -o ./my-app --name MyApp
//!
//! This command:
//! 1. Parses the AURA file
//! 2. Detects required shadcn-vue components
//! 3. Generates a complete Vite + Vue + TypeScript project
//! 4. Runs npm install
//! 5. Runs npx shadcn-vue add to add components

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use colored::Colorize;

/// Check if a command exists
fn command_exists(cmd: &str) -> bool {
    #[cfg(windows)]
    let check = Command::new("where").arg(cmd).output();
    #[cfg(not(windows))]
    let check = Command::new("which").arg(cmd).output();

    check.map(|o| o.status.success()).unwrap_or(false)
}

/// Run a command with live output (inherits stdout/stderr)
/// On Windows, uses cmd.exe /C to properly resolve commands in PATH
fn run_command_live(cmd: &str, args: &[&str], cwd: &Path) -> Result<(), String> {
    use std::process::Stdio;

    #[cfg(windows)]
    let status = {
        // On Windows, use cmd.exe /C to properly resolve npm/npx from PATH
        let mut full_args = vec!["/C", cmd];
        full_args.extend(args);
        Command::new("cmd")
            .args(&full_args)
            .current_dir(cwd)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| format!("Failed to run {}: {}", cmd, e))?
    };

    #[cfg(not(windows))]
    let status = {
        Command::new(cmd)
            .args(args)
            .current_dir(cwd)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| format!("Failed to run {}: {}", cmd, e))?
    };

    if status.success() {
        Ok(())
    } else {
        Err(format!("{} exited with code {:?}", cmd, status.code()))
    }
}

/// Generate Vue project from AURA file
pub fn generate_vue_project(
    input_path: &str,
    output_dir: Option<&str>,
    project_name: Option<&str>,
    no_install: bool,
    yes: bool,
) -> Result<(), String> {
    // Determine output directory
    let input = Path::new(input_path);
    let input_stem = input.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("aura-app");

    let output = output_dir.unwrap_or(input_stem);
    let output_path = Path::new(output);

    // Determine project name
    let name = project_name.unwrap_or_else(|| {
        output_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(input_stem)
    });

    println!("{}", "─────────────────────────────────".bright_yellow().bold());
    println!("{}", "  AURA → Vue + shadcn-vue".bright_yellow().bold());
    println!("{}", "─────────────────────────────────".bright_yellow().bold());
    println!();

    // Check prerequisites
    if !command_exists("npm") {
        return Err("npm not found. Please install Node.js from https://nodejs.org/".to_string());
    }

    println!("{} {}", "Input:".bright_cyan(), input_path);
    println!("{} {}", "Output:".bright_cyan(), output);
    println!("{} {}", "Name:".bright_cyan(), name);
    println!();

    // Create output directory
    if output_path.exists() {
        return Err(format!("Output directory '{}' already exists", output));
    }

    fs::create_dir_all(output_path)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    // Create src directory structure
    let src_dir = output_path.join("src");
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

    // Parse AURA and generate Vue component with shadcn-vue mode
    let vue_code = auto_lang::ui_build_shadcn(input_path, None)
        .map_err(|e| format!("Failed to generate Vue code: {:?}", e))?;

    // Detect required shadcn components
    let components = detect_shadcn_components(&vue_code);
    println!("{} {}", "✓ Detected shadcn-vue components:".bright_green(), components.join(", "));

    // Write project files
    write_project_files(output_path, name, &vue_code, &components)?;

    println!("{}", "✓ Generated project files".bright_green());

    if no_install {
        println!();
        println!("{}", "Project created successfully!".bright_green().bold());
        println!();
        println!("Next steps:");
        println!("  cd {}", output);
        println!("  npm install");
        println!("  npx shadcn-vue@latest add {} --yes", components.join(" "));
        println!("  npm run dev");
    } else {
        // Check if npm exists
        if !command_exists("npm") {
            println!();
            println!("{}", "⚠ npm not found".bright_yellow());
            println!("Please install Node.js from https://nodejs.org/");
            println!();
            println!("Then run:");
            println!("  cd {}", output);
            println!("  npm install");
            println!("  npx shadcn-vue@latest add {} --yes", components.join(" "));
            println!("  npm run dev");
            return Ok(());
        }

        // Step 1: npm install
        println!();
        println!("{} {}", "▶".bright_cyan(), "Step 1/3: Installing dependencies...".bright_white());

        let npm_install_args = if yes {
            println!("{}", "  Running: npm install -y".bright_black());
            vec!["install", "-y"]
        } else {
            println!("{}", "  Running: npm install".bright_black());
            vec!["install"]
        };

        match run_command_live("npm", &npm_install_args, output_path) {
            Ok(_) => println!("{}", "  ✓ Dependencies installed".bright_green()),
            Err(e) => {
                println!("{} {}", "  ✗ Failed:".bright_red(), e);
                println!("  You may need to run 'npm install' manually.");
            }
        }

        // Step 2: shadcn-vue add
        if !components.is_empty() {
            println!();
            println!("{} {}", "▶".bright_cyan(), format!("Step 2/3: Adding shadcn-vue components ({})...", components.join(", ")).bright_white());

            // Build args: npx --yes shadcn-vue@latest add button --yes
            // First --yes is for npx (auto-install package)
            // Second --yes is for shadcn-vue (skip prompts)
            let mut args = if yes {
                println!("{}", format!("  Running: npx --yes shadcn-vue@latest add {} --yes", components.join(" ")).bright_black());
                vec!["--yes", "shadcn-vue@latest", "add"]
            } else {
                println!("{}", format!("  Running: npx shadcn-vue@latest add {} --yes", components.join(" ")).bright_black());
                vec!["shadcn-vue@latest", "add"]
            };
            args.extend(components.iter().map(|s| s.as_str()));
            args.push("--yes");

            match run_command_live("npx", &args, output_path) {
                Ok(_) => println!("{}", "  ✓ shadcn-vue components added".bright_green()),
                Err(e) => {
                    println!("{} {}", "  ✗ Failed:".bright_red(), e);
                    println!("  You may need to run 'npx shadcn-vue@latest add {} --yes' manually.", components.join(" "));
                }
            }
        } else {
            println!();
            println!("{} {}", "▶".bright_cyan(), "Step 2/3: No shadcn-vue components needed".bright_white());
        }

        // Step 3: Ask if user wants to run dev server
        println!();
        println!("{} {}", "▶".bright_cyan(), "Step 3/3: Ready to start dev server".bright_white());
        println!();
        println!("{}", "═════════════════════════════════".bright_green().bold());
        println!("{}", "  Project created successfully!".bright_green().bold());
        println!("{}", "═════════════════════════════════".bright_green().bold());
        println!();
        println!("Starting dev server...");
        println!();

        // Run npm run dev
        let _ = run_command_live("npm", &["run", "dev"], output_path);
    }

    Ok(())
}

/// Detect which shadcn-vue components are needed from generated Vue code
fn detect_shadcn_components(vue_code: &str) -> Vec<String> {
    let mut components = HashSet::new();

    // Map import patterns to component names
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
        // Feedback & Overlay
        ("@/components/ui/alert", "alert"),
        ("@/components/ui/sonner", "sonner"),
        ("@/components/ui/dropdown-menu", "dropdown-menu"),
        ("@/components/ui/popover", "popover"),
        ("@/components/ui/sheet", "sheet"),
        ("@/components/ui/breadcrumb", "breadcrumb"),
        // High Priority Components
        ("@/components/ui/accordion", "accordion"),
        ("@/components/ui/alert-dialog", "alert-dialog"),
        ("@/components/ui/command", "command"),
        ("@/components/ui/form", "form"),
        ("@/components/ui/navigation-menu", "navigation-menu"),
        ("@/components/ui/sidebar", "sidebar"),
        ("@/components/ui/stepper", "stepper"),
    ];

    for (pattern, component) in component_patterns {
        if vue_code.contains(pattern) {
            components.insert(component.to_string());
        }
    }

    // Sort for consistent output
    let mut result: Vec<String> = components.into_iter().collect();
    result.sort();
    result
}

/// Write all project files
fn write_project_files(
    output_path: &Path,
    name: &str,
    vue_code: &str,
    components: &[String],
) -> Result<(), String> {
    // package.json
    let package_json = generate_package_json(name);
    fs::write(output_path.join("package.json"), package_json)
        .map_err(|e| format!("Failed to write package.json: {}", e))?;

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

    // tailwind.config.cjs (use .cjs for ES module compatibility)
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
    let main_ts = generate_main_ts();
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

    // Write Vue component(s)
    write_vue_components(&output_path.join("src/components"), vue_code)?;

    Ok(())
}

/// Write Vue component files
fn write_vue_components(components_dir: &Path, vue_code: &str) -> Result<(), String> {
    // For now, we write the entire generated code as a single component
    // The vue generator already produces proper component code

    // Extract widget name from the generated code
    let widget_name = extract_widget_name(vue_code).unwrap_or_else(|| "Widget".to_string());

    // Write the component file
    fs::write(components_dir.join(format!("{}.vue", widget_name)), vue_code)
        .map_err(|e| format!("Failed to write component: {}", e))?;

    Ok(())
}

/// Extract widget name from generated Vue code
fn extract_widget_name(vue_code: &str) -> Option<String> {
    // Look for <!-- WidgetName component --> comment
    for line in vue_code.lines() {
        if line.starts_with("<!--") && line.contains("component") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                return Some(parts[1].to_string());
            }
        }
    }
    None
}

// Template generators

fn generate_package_json(name: &str) -> String {
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
    "@vueuse/core": "^10.7.0",
    "radix-vue": "^1.4.0",
    "class-variance-authority": "^0.7.0",
    "clsx": "^2.1.0",
    "tailwind-merge": "^2.2.0",
    "lucide-vue-next": "^0.312.0"
  }},
  "devDependencies": {{
    "@vitejs/plugin-vue": "^5.0.0",
    "vite": "^5.0.0",
    "typescript": "^5.3.0",
    "vue-tsc": "^1.8.0",
    "tailwindcss": "^3.4.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0",
    "tailwindcss-animate": "^1.0.7"
  }}
}}
"#, name)
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
  server: {
    port: 3000,
    open: true
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
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "baseUrl": ".",
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
    "allowSyntheticDefaultImports": true,
    "strict": true
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
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
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
          to: { height: "var(--radix-accordion-content-height)" },
        },
        "accordion-up": {
          from: { height: "var(--radix-accordion-content-height)" },
          to: { height: 0 },
        },
      },
      animation: {
        "accordion-down": "accordion-down 0.2s ease-out",
        "accordion-up": "accordion-up 0.2s ease-out",
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

fn generate_main_ts() -> String {
    r#"import { createApp } from 'vue'
import App from './App.vue'
import './assets/index.css'

createApp(App).mount('#app')
"#.to_string()
}

fn generate_app_vue(vue_code: &str) -> String {
    // Extract the widget name to create the import
    let widget_name = extract_widget_name(vue_code).unwrap_or_else(|| "Widget".to_string());

    format!(r#"<script setup lang="ts">
import {0} from './components/{0}.vue'
</script>

<template>
  <div class="min-h-screen bg-background">
    <{0} />
  </div>
</template>
"#, widget_name)
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
