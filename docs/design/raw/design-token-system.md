# Design Token 系统

## 概述

本文档定义 AutoLang 的跨平台 Design Token 系统，用于实现 UI 视觉一致性。

### 背景

传统 Design Token 工作流：
```
设计师 → Figma → 导出 Token → 各平台实现
```

AI 时代的新工作流：
```
AI → 直接生成 Token → Token Compiler → 各平台代码
```

### 目标

1. **AI 友好**：AI 可直接生成 Token 定义
2. **跨平台一致**：同一 Token 编译到不同平台
3. **类型安全**：使用 Auto 语言定义，编译时检查
4. **易于维护**：单一数据源，自动生成

---

## 架构设计

### 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                  Design Tokens (tokens.at)                  │
│                    AI 直接生成                               │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   Token Compiler                            │
│              (crates/auto-lang/src/tokens/)                 │
│                                                             │
│  - 解析 Auto 类型定义 / JSON                                │
│  - 提取 Token 值                                            │
│  - 转换为目标平台代码                                       │
└─────────────────────────────────────────────────────────────┘
                              │
       ┌──────────────────────┼──────────────────────┐
       │                      │                      │
       ▼                      ▼                      ▼
┌─────────────┐      ┌─────────────┐       ┌─────────────┐
│   Vue/CSS   │      │  Rust/gpui  │       │Kotlin/Compose│
│             │      │             │       │             │
│ CSS Vars    │      │ const Token │       │ Material    │
│ Tailwind    │      │             │       │ Theme       │
└─────────────┘      └─────────────┘       └─────────────┘
       │                      │                      │
       ▼                      ▼                      ▼
┌─────────────┐      ┌─────────────┐       ┌─────────────┐
│  ArkTS      │      │   LVGL/C    │       │   iOS*      │
│             │      │             │       │             │
│ @Styles     │      │ #define     │       │ SwiftUI     │
└─────────────┘      └─────────────┘       └─────────────┘

*iOS: 未来通过 Jetpack Compose Multiplatform 或 SwiftUI 实现
```

### 工作流程

```
1. AI 生成 Design Tokens
   输出: tokens.at (Auto 类型定义) 或 tokens.json

2. Token Compiler 解析
   输入: tokens.at / tokens.json
   输出: DesignTokens 结构体

3. 编译到各平台
   ├─→ Vue: tokens.css + tailwind.config.js
   ├─→ Rust: tokens.rs
   ├─→ Kotlin: Color.kt + Theme.kt
   ├─→ ArkTS: tokens.ets
   └─→ C: design_tokens.h

4. 组件引用 Tokens
   Button.bg(Colors.Primary)
   Container.padding(Spacing.Md)
```

---

## Token 定义格式

### 格式选择

| 格式 | 优点 | 缺点 | 推荐场景 |
|-----|------|------|---------|
| **Auto 类型定义** | 类型安全，可编程 | 需要解析 | 默认推荐 |
| **JSON** | 通用，工具支持好 | 无类型检查 | 外部工具集成 |

### Auto 类型定义格式

```auto
// styles/tokens.at

// ═══════════════════════════════════════════════════════════
// Design Tokens - 设计令牌定义
// ═══════════════════════════════════════════════════════════

// ───────────────────────────────────────────────────────────
// 颜色系统
// ───────────────────────────────────────────────────────────

type ColorTokens = {
    // 语义色（组件直接使用）
    primary: string      // 主色
    secondary: string    // 辅助色
    accent: string       // 强调色

    // 功能色
    success: string      // 成功
    warning: string      // 警告
    error: string        // 错误
    info: string         // 信息

    // 中性色
    background: string   // 背景色
    foreground: string   // 前景色（文字）
    muted: string        // 次要文字
    border: string       // 边框

    // 调色板（用于生成色阶）
    slate: {
        50: string
        100: string
        200: string
        300: string
        400: string
        500: string
        600: string
        700: string
        800: string
        900: string
    }
}

// ───────────────────────────────────────────────────────────
// 间距系统（基于 4px 基准）
// ───────────────────────────────────────────────────────────

type SpacingTokens = {
    0: int       // 0px
    px: int      // 1px
    xs: int      // 4px  (0.25rem)
    sm: int      // 8px  (0.5rem)
    md: int      // 16px (1rem)
    lg: int      // 24px (1.5rem)
    xl: int      // 32px (2rem)
    "2xl": int   // 48px (3rem)
    "3xl": int   // 64px (4rem)
}

// ───────────────────────────────────────────────────────────
// 圆角系统
// ───────────────────────────────────────────────────────────

type RadiusTokens = {
    none: int    // 0px
    sm: int      // 4px
    md: int      // 8px
    lg: int      // 12px
    xl: int      // 16px
    "2xl": int   // 24px
    full: int    // 9999px (完全圆形)
}

// ───────────────────────────────────────────────────────────
// 字体大小系统
// ───────────────────────────────────────────────────────────

type FontSizeTokens = {
    xs: int      // 12px
    sm: int      // 14px
    base: int    // 16px
    lg: int      // 18px
    xl: int      // 20px
    "2xl": int   // 24px
    "3xl": int   // 30px
    "4xl": int   // 36px
}

// ───────────────────────────────────────────────────────────
// 阴影系统
// ───────────────────────────────────────────────────────────

type ShadowTokens = {
    sm: string
    md: string
    lg: string
    xl: string
}

// ───────────────────────────────────────────────────────────
// 动画时长系统
// ───────────────────────────────────────────────────────────

type DurationTokens = {
    fast: int     // 100ms
    normal: int   // 200ms
    slow: int     // 300ms
}

// ───────────────────────────────────────────────────────────
// 断点系统（响应式）
// ───────────────────────────────────────────────────────────

type BreakpointTokens = {
    sm: int      // 640px
    md: int      // 768px
    lg: int      // 1024px
    xl: int      // 1280px
    "2xl": int   // 1536px
}

// ═══════════════════════════════════════════════════════════
// 完整 Token 集合
// ═══════════════════════════════════════════════════════════

type DesignTokens = {
    color: ColorTokens
    spacing: SpacingTokens
    radius: RadiusTokens
    fontSize: FontSizeTokens
    shadow: ShadowTokens
    duration: DurationTokens
    breakpoint: BreakpointTokens
}

// ═══════════════════════════════════════════════════════════
// Token 实例（AI 生成时填充实际值）
// ═══════════════════════════════════════════════════════════

const TOKENS DesignTokens = {
    color: {
        primary: "#3B82F6",
        secondary: "#6366F1",
        accent: "#F59E0B",
        success: "#10B981",
        warning: "#F59E0B",
        error: "#EF4444",
        info: "#3B82F6",
        background: "#FFFFFF",
        foreground: "#0F172A",
        muted: "#64748B",
        border: "#E2E8F0",
        slate: {
            50: "#F8FAFC",
            100: "#F1F5F9",
            200: "#E2E8F0",
            300: "#CBD5E1",
            400: "#94A3B8",
            500: "#64748B",
            600: "#475569",
            700: "#334155",
            800: "#1E293B",
            900: "#0F172A",
        },
    },
    spacing: {
        0: 0,
        px: 1,
        xs: 4,
        sm: 8,
        md: 16,
        lg: 24,
        xl: 32,
        "2xl": 48,
        "3xl": 64,
    },
    radius: {
        none: 0,
        sm: 4,
        md: 8,
        lg: 12,
        xl: 16,
        "2xl": 24,
        full: 9999,
    },
    fontSize: {
        xs: 12,
        sm: 14,
        base: 16,
        lg: 18,
        xl: 20,
        "2xl": 24,
        "3xl": 30,
        "4xl": 36,
    },
    shadow: {
        sm: "0 1px 2px 0 rgb(0 0 0 / 0.05)",
        md: "0 4px 6px -1px rgb(0 0 0 / 0.1)",
        lg: "0 10px 15px -3px rgb(0 0 0 / 0.1)",
        xl: "0 20px 25px -5px rgb(0 0 0 / 0.1)",
    },
    duration: {
        fast: 100,
        normal: 200,
        slow: 300,
    },
    breakpoint: {
        sm: 640,
        md: 768,
        lg: 1024,
        xl: 1280,
        "2xl": 1536,
    },
}
```

### JSON 格式（可选）

```json
{
  "color": {
    "primary": "#3B82F6",
    "secondary": "#6366F1",
    "accent": "#F59E0B"
  },
  "spacing": {
    "xs": 4,
    "sm": 8,
    "md": 16
  }
}
```

---

## Token Compiler 设计

### 模块结构

```
crates/auto-lang/src/tokens/
├── mod.rs           # 模块入口，TokenCompiler
├── parser.rs        # 解析 Auto/JSON Token 定义
├── types.rs         # Token 数据结构定义
└── targets/
    ├── mod.rs       # Target trait
    ├── vue.rs       # CSS Variables + Tailwind
    ├── rust.rs      # Rust const
    ├── kotlin.rs    # Material Theme
    ├── arkts.rs     # @Styles
    └── c.rs         # #define macros
```

### 核心 API

```rust
// tokens/mod.rs

pub mod parser;
pub mod types;
pub mod targets;

use types::DesignTokens;

#[derive(Debug, Clone, Copy)]
pub enum Target {
    Vue,
    Rust,
    Kotlin,
    ArkTS,
    C,
}

pub struct TokenCompiler {
    tokens: DesignTokens,
}

impl TokenCompiler {
    /// 从 Auto 源文件解析 Tokens
    pub fn from_auto(source: &str) -> Result<Self> {
        let tokens = parser::parse_auto_tokens(source)?;
        Ok(Self { tokens })
    }

    /// 从 JSON 解析 Tokens
    pub fn from_json(json: &str) -> Result<Self> {
        let tokens = parser::parse_json_tokens(json)?;
        Ok(Self { tokens })
    }

    /// 获取 Tokens 引用
    pub fn tokens(&self) -> &DesignTokens {
        &self.tokens
    }

    /// 编译到指定目标平台
    pub fn compile(&self, target: Target) -> Result<String> {
        match target {
            Target::Vue => targets::vue::compile(&self.tokens),
            Target::Rust => targets::rust::compile(&self.tokens),
            Target::Kotlin => targets::kotlin::compile(&self.tokens),
            Target::ArkTS => targets::arkts::compile(&self.tokens),
            Target::C => targets::c::compile(&self.tokens),
        }
    }

    /// 编译到所有平台，返回 (target, output) 列表
    pub fn compile_all(&self) -> Result<Vec<(Target, String)>> {
        let targets = [Target::Vue, Target::Rust, Target::Kotlin, Target::ArkTS, Target::C];
        targets.iter()
            .map(|&t| Ok((t, self.compile(t)?)))
            .collect()
    }
}
```

### Token 数据结构

```rust
// tokens/types.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    pub v50: String,
    pub v100: String,
    pub v200: String,
    pub v300: String,
    pub v400: String,
    pub v500: String,
    pub v600: String,
    pub v700: String,
    pub v800: String,
    pub v900: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorTokens {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub success: String,
    pub warning: String,
    pub error: String,
    pub info: String,
    pub background: String,
    pub foreground: String,
    pub muted: String,
    pub border: String,
    pub slate: ColorPalette,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacingTokens {
    pub none: u32,
    pub px: u32,
    pub xs: u32,
    pub sm: u32,
    pub md: u32,
    pub lg: u32,
    pub xl: u32,
    pub x2l: u32,
    pub x3l: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadiusTokens {
    pub none: u32,
    pub sm: u32,
    pub md: u32,
    pub lg: u32,
    pub xl: u32,
    pub x2l: u32,
    pub full: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSizeTokens {
    pub xs: u32,
    pub sm: u32,
    pub base: u32,
    pub lg: u32,
    pub xl: u32,
    pub x2l: u32,
    pub x3l: u32,
    pub x4l: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowTokens {
    pub sm: String,
    pub md: String,
    pub lg: String,
    pub xl: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurationTokens {
    pub fast: u32,
    pub normal: u32,
    pub slow: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakpointTokens {
    pub sm: u32,
    pub md: u32,
    pub lg: u32,
    pub xl: u32,
    pub x2l: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignTokens {
    pub color: ColorTokens,
    pub spacing: SpacingTokens,
    pub radius: RadiusTokens,
    pub font_size: FontSizeTokens,
    pub shadow: ShadowTokens,
    pub duration: DurationTokens,
    pub breakpoint: BreakpointTokens,
}
```

---

## 各平台输出格式

### 1. Vue (CSS Variables + Tailwind)

**输出文件**: `tokens.css`

```css
/* Auto-generated by Token Compiler */

:root {
  /* Colors */
  --color-primary: #3B82F6;
  --color-secondary: #6366F1;
  --color-accent: #F59E0B;
  --color-success: #10B981;
  --color-warning: #F59E0B;
  --color-error: #EF4444;
  --color-info: #3B82F6;
  --color-background: #FFFFFF;
  --color-foreground: #0F172A;
  --color-muted: #64748B;
  --color-border: #E2E8F0;

  /* Spacing */
  --spacing-xs: 4px;
  --spacing-sm: 8px;
  --spacing-md: 16px;
  --spacing-lg: 24px;
  --spacing-xl: 32px;
  --spacing-2xl: 48px;
  --spacing-3xl: 64px;

  /* Border Radius */
  --radius-sm: 4px;
  --radius-md: 8px;
  --radius-lg: 12px;
  --radius-xl: 16px;
  --radius-2xl: 24px;
  --radius-full: 9999px;

  /* Font Size */
  --font-size-xs: 12px;
  --font-size-sm: 14px;
  --font-size-base: 16px;
  --font-size-lg: 18px;
  --font-size-xl: 20px;
  --font-size-2xl: 24px;

  /* Shadows */
  --shadow-sm: 0 1px 2px 0 rgb(0 0 0 / 0.05);
  --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.1);
  --shadow-lg: 0 10px 15px -3px rgb(0 0 0 / 0.1);

  /* Duration */
  --duration-fast: 100ms;
  --duration-normal: 200ms;
  --duration-slow: 300ms;
}

/* Dark mode override */
@media (prefers-color-scheme: dark) {
  :root {
    --color-background: #0F172A;
    --color-foreground: #F8FAFC;
    /* ... */
  }
}
```

**输出文件**: `tailwind.config.js`

```javascript
/** @type {import('tailwindcss').Config} */
module.exports = {
  theme: {
    extend: {
      colors: {
        primary: 'var(--color-primary)',
        secondary: 'var(--color-secondary)',
        accent: 'var(--color-accent)',
        success: 'var(--color-success)',
        warning: 'var(--color-warning)',
        error: 'var(--color-error)',
      },
      spacing: {
        'xs': 'var(--spacing-xs)',
        'sm': 'var(--spacing-sm)',
        'md': 'var(--spacing-md)',
        'lg': 'var(--spacing-lg)',
        'xl': 'var(--spacing-xl)',
      },
      borderRadius: {
        'sm': 'var(--radius-sm)',
        'md': 'var(--radius-md)',
        'lg': 'var(--radius-lg)',
      },
    },
  },
}
```

### 2. Rust (const tokens)

**输出文件**: `tokens.rs`

```rust
//! Design Tokens - Auto-generated by Token Compiler
//! Do not edit manually

pub mod color {
    pub const PRIMARY: &str = "#3B82F6";
    pub const SECONDARY: &str = "#6366F1";
    pub const ACCENT: &str = "#F59E0B";
    pub const SUCCESS: &str = "#10B981";
    pub const WARNING: &str = "#F59E0B";
    pub const ERROR: &str = "#EF4444";
    pub const INFO: &str = "#3B82F6";
    pub const BACKGROUND: &str = "#FFFFFF";
    pub const FOREGROUND: &str = "#0F172A";
    pub const MUTED: &str = "#64748B";
    pub const BORDER: &str = "#E2E8F0";

    /// Parse color to RGBA
    pub fn parse(hex: &str) -> [f32; 4] {
        // #3B82F6 -> [0.231, 0.510, 0.965, 1.0]
        let hex = hex.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap() as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap() as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap() as f32 / 255.0;
        [r, g, b, 1.0]
    }
}

pub mod spacing {
    pub const NONE: f64 = 0.0;
    pub const PX: f64 = 1.0;
    pub const XS: f64 = 4.0;
    pub const SM: f64 = 8.0;
    pub const MD: f64 = 16.0;
    pub const LG: f64 = 24.0;
    pub const XL: f64 = 32.0;
    pub const X2L: f64 = 48.0;
    pub const X3L: f64 = 64.0;
}

pub mod radius {
    pub const NONE: f64 = 0.0;
    pub const SM: f64 = 4.0;
    pub const MD: f64 = 8.0;
    pub const LG: f64 = 12.0;
    pub const XL: f64 = 16.0;
    pub const X2L: f64 = 24.0;
    pub const FULL: f64 = 9999.0;
}

pub mod font_size {
    pub const XS: f64 = 12.0;
    pub const SM: f64 = 14.0;
    pub const BASE: f64 = 16.0;
    pub const LG: f64 = 18.0;
    pub const XL: f64 = 20.0;
    pub const X2L: f64 = 24.0;
    pub const X3L: f64 = 30.0;
    pub const X4L: f64 = 36.0;
}

pub mod duration {
    pub const FAST: u64 = 100;
    pub const NORMAL: u64 = 200;
    pub const SLOW: u64 = 300;
}
```

### 3. Kotlin (Material Theme)

**输出文件**: `Color.kt`

```kotlin
// Auto-generated by Token Compiler
package ui.theme

import androidx.compose.ui.graphics.Color

// Semantic Colors
val Primary = Color(0xFF3B82F6)
val Secondary = Color(0xFF6366F1)
val Accent = Color(0xFFF59E0B)
val Success = Color(0xFF10B981)
val Warning = Color(0xFFF59E0B)
val Error = Color(0xFFEF4444)
val Info = Color(0xFF3B82F6)

// Neutral Colors
val Background = Color(0xFFFFFFFF)
val Foreground = Color(0xFF0F172A)
val Muted = Color(0xFF64748B)
val Border = Color(0xFFE2E8F0)

// Slate Palette
val Slate50 = Color(0xFFF8FAFC)
val Slate100 = Color(0xFFF1F5F9)
val Slate200 = Color(0xFFE2E8F0)
val Slate300 = Color(0xFFCBD5E1)
val Slate400 = Color(0xFF94A3B8)
val Slate500 = Color(0xFF64748B)
val Slate600 = Color(0xFF475569)
val Slate700 = Color(0xFF334155)
val Slate800 = Color(0xFF1E293B)
val Slate900 = Color(0xFF0F172A)
```

**输出文件**: `Theme.kt`

```kotlin
// Auto-generated by Token Compiler
package ui.theme

import androidx.compose.material3.*

private val LightColorScheme = lightColorScheme(
    primary = Primary,
    secondary = Secondary,
    tertiary = Accent,
    background = Background,
    surface = Background,
    error = Error,
    onPrimary = Background,
    onSecondary = Background,
    onBackground = Foreground,
    onSurface = Foreground,
    onError = Background,
)

private val DarkColorScheme = darkColorScheme(
    primary = Primary,
    secondary = Secondary,
    tertiary = Accent,
    background = Slate900,
    surface = Slate900,
    error = Error,
    onPrimary = Slate900,
    onSecondary = Slate900,
    onBackground = Slate50,
    onSurface = Slate50,
    onError = Slate50,
)

@Composable
fun AppTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    content: @Composable () -> Unit
) {
    val colorScheme = if (darkTheme) DarkColorScheme else LightColorScheme

    MaterialTheme(
        colorScheme = colorScheme,
        typography = Typography,
        content = content
    )
}
```

**输出文件**: `Spacing.kt`

```kotlin
// Auto-generated by Token Compiler
package ui.theme

import androidx.compose.ui.unit.dp

object Spacing {
    val None = 0.dp
    val Px = 1.dp
    val Xs = 4.dp
    val Sm = 8.dp
    val Md = 16.dp
    val Lg = 24.dp
    val Xl = 32.dp
    val X2l = 48.dp
    val X3l = 64.dp
}

object Radius {
    val None = 0.dp
    val Sm = 4.dp
    val Md = 8.dp
    val Lg = 12.dp
    val Xl = 16.dp
    val X2l = 24.dp
    val Full = 9999.dp
}
```

### 4. ArkTS (@Styles)

**输出文件**: `tokens.ets`

```typescript
// Auto-generated by Token Compiler

// ═══════════════════════════════════════════════════════════
// Colors
// ═══════════════════════════════════════════════════════════

export class Colors {
  // Semantic
  static readonly Primary: string = '#3B82F6'
  static readonly Secondary: string = '#6366F1'
  static readonly Accent: string = '#F59E0B'
  static readonly Success: string = '#10B981'
  static readonly Warning: string = '#F59E0B'
  static readonly Error: string = '#EF4444'
  static readonly Info: string = '#3B82F6'

  // Neutral
  static readonly Background: string = '#FFFFFF'
  static readonly Foreground: string = '#0F172A'
  static readonly Muted: string = '#64748B'
  static readonly Border: string = '#E2E8F0'

  // Slate Palette
  static readonly Slate50: string = '#F8FAFC'
  static readonly Slate100: string = '#F1F5F9'
  static readonly Slate200: string = '#E2E8F0'
  static readonly Slate300: string = '#CBD5E1'
  static readonly Slate400: string = '#94A3B8'
  static readonly Slate500: string = '#64748B'
  static readonly Slate600: string = '#475569'
  static readonly Slate700: string = '#334155'
  static readonly Slate800: string = '#1E293B'
  static readonly Slate900: string = '#0F172A'
}

// ═══════════════════════════════════════════════════════════
// Spacing
// ═══════════════════════════════════════════════════════════

export class Spacing {
  static readonly None: number = 0
  static readonly Px: number = 1
  static readonly Xs: number = 4
  static readonly Sm: number = 8
  static readonly Md: number = 16
  static readonly Lg: number = 24
  static readonly Xl: number = 32
  static readonly X2l: number = 48
  static readonly X3l: number = 64
}

export class Radius {
  static readonly None: number = 0
  static readonly Sm: number = 4
  static readonly Md: number = 8
  static readonly Lg: number = 12
  static readonly Xl: number = 16
  static readonly X2l: number = 24
  static readonly Full: number = 9999
}

// ═══════════════════════════════════════════════════════════
// Common Styles
// ═══════════════════════════════════════════════════════════

@Styles function buttonPrimary() {
  .backgroundColor(Colors.Primary)
  .borderRadius(Radius.Md)
  .padding({
    left: Spacing.Md,
    right: Spacing.Md,
    top: Spacing.Sm,
    bottom: Spacing.Sm
  })
}

@Styles function buttonSecondary() {
  .backgroundColor(Colors.Secondary)
  .borderRadius(Radius.Md)
  .padding({
    left: Spacing.Md,
    right: Spacing.Md,
    top: Spacing.Sm,
    bottom: Spacing.Sm
  })
}

@Styles function cardStyle() {
  .backgroundColor(Colors.Background)
  .borderRadius(Radius.Lg)
  .padding(Spacing.Md)
  .shadow({
    radius: 8,
    color: 'rgba(0, 0, 0, 0.1)',
    offsetX: 0,
    offsetY: 2
  })
}
```

### 5. C (LVGL)

**输出文件**: `design_tokens.h`

```c
/**
 * Design Tokens - Auto-generated by Token Compiler
 * Do not edit manually
 */

#ifndef DESIGN_TOKENS_H
#define DESIGN_TOKENS_H

#include "lvgl.h"

/* ═══════════════════════════════════════════════════════════
 * Colors
 * ═══════════════════════════════════════════════════════════ */

/* Semantic Colors */
#define COLOR_PRIMARY       lv_color_hex(0x3B82F6)
#define COLOR_SECONDARY     lv_color_hex(0x6366F1)
#define COLOR_ACCENT        lv_color_hex(0xF59E0B)
#define COLOR_SUCCESS       lv_color_hex(0x10B981)
#define COLOR_WARNING       lv_color_hex(0xF59E0B)
#define COLOR_ERROR         lv_color_hex(0xEF4444)
#define COLOR_INFO          lv_color_hex(0x3B82F6)

/* Neutral Colors */
#define COLOR_BACKGROUND    lv_color_hex(0xFFFFFF)
#define COLOR_FOREGROUND    lv_color_hex(0x0F172A)
#define COLOR_MUTED         lv_color_hex(0x64748B)
#define COLOR_BORDER        lv_color_hex(0xE2E8F0)

/* Slate Palette */
#define COLOR_SLATE_50      lv_color_hex(0xF8FAFC)
#define COLOR_SLATE_100     lv_color_hex(0xF1F5F9)
#define COLOR_SLATE_200     lv_color_hex(0xE2E8F0)
#define COLOR_SLATE_300     lv_color_hex(0xCBD5E1)
#define COLOR_SLATE_400     lv_color_hex(0x94A3B8)
#define COLOR_SLATE_500     lv_color_hex(0x64748B)
#define COLOR_SLATE_600     lv_color_hex(0x475569)
#define COLOR_SLATE_700     lv_color_hex(0x334155)
#define COLOR_SLATE_800     lv_color_hex(0x1E293B)
#define COLOR_SLATE_900     lv_color_hex(0x0F172A)

/* ═══════════════════════════════════════════════════════════
 * Spacing
 * ═══════════════════════════════════════════════════════════ */

#define SPACING_NONE    0
#define SPACING_PX      1
#define SPACING_XS      4
#define SPACING_SM      8
#define SPACING_MD      16
#define SPACING_LG      24
#define SPACING_XL      32
#define SPACING_X2L     48
#define SPACING_X3L     64

/* ═══════════════════════════════════════════════════════════
 * Border Radius
 * ═══════════════════════════════════════════════════════════ */

#define RADIUS_NONE     0
#define RADIUS_SM       4
#define RADIUS_MD       8
#define RADIUS_LG       12
#define RADIUS_XL       16
#define RADIUS_X2L      24
#define RADIUS_FULL     9999

/* ═══════════════════════════════════════════════════════════
 * Font Size (in LVGL, fonts are pre-defined)
 * ═══════════════════════════════════════════════════════════ */

#define FONT_SIZE_XS    12
#define FONT_SIZE_SM    14
#define FONT_SIZE_BASE  16
#define FONT_SIZE_LG    18
#define FONT_SIZE_XL    20
#define FONT_SIZE_X2L   24

/* ═══════════════════════════════════════════════════════════
 * Helper Functions
 * ═══════════════════════════════════════════════════════════ */

/**
 * Apply primary button style
 */
static inline void style_button_primary(lv_obj_t* btn) {
    lv_obj_set_style_bg_color(btn, COLOR_PRIMARY, 0);
    lv_obj_set_style_radius(btn, RADIUS_MD, 0);
    lv_obj_set_style_pad_all(btn, SPACING_SM, 0);
}

/**
 * Apply card style
 */
static inline void style_card(lv_obj_t* card) {
    lv_obj_set_style_bg_color(card, COLOR_BACKGROUND, 0);
    lv_obj_set_style_radius(card, RADIUS_LG, 0);
    lv_obj_set_style_pad_all(card, SPACING_MD, 0);
    lv_obj_set_style_border_width(card, 1, 0);
    lv_obj_set_style_border_color(card, COLOR_BORDER, 0);
}

#endif /* DESIGN_TOKENS_H */
```

---

## AI 生成 Token 的 Prompt 模板

### 基础模板

```
为 [应用类型] 应用生成 Design Tokens。

应用类型: [Web App / Mobile App / Desktop App / Embedded App]
设计风格: [Modern / Minimal / Corporate / Playful / Dark Theme]
主色调: [如蓝色、绿色等]

输出格式: Auto 语言类型定义

需要定义:
1. 颜色系统
   - 语义色 (primary, secondary, accent)
   - 功能色
   - 中性色 (background, foreground, muted, border)
   - 调色板 (slate 色阶)

2. 间距系统 (基于 4px 基准)
   - xs: 4px, sm: 8px, md: 16px, lg: 24px, xl: 32px

3. 圆角系统
   - sm: 4px, md: 8px, lg: 12px

4. 字体大小系统
   - xs: 12px, sm: 14px, base: 16px, lg: 18px, xl: 20px

5. 阴影系统
   - sm, md, lg (CSS box-shadow 格式)

请输出完整的 Auto 语言类型定义，包括类型声明和 TOKENS 常量。
```

### 示例 Prompt

```
为任务管理应用生成 Design Tokens。

设计风格: 现代、简洁、专业
主色调: 蓝色 (#3B82F6)
支持暗色模式

输出 Auto 语言格式的 Design Tokens 定义。
```

---

## 与 AURA 的集成

### 组件中使用 Token

```rust
// AURA 组件定义
Button::new("Submit")
    .style(Style::new()
        .background(TOKENS.color.primary)      // 使用 Token
        .padding(TOKENS.spacing.md)
        .radius(TOKENS.radius.md)
        .text_color(TOKENS.color.background)
    )
```

### 编译时解析

```rust
// Token 引用在编译时被解析为具体值
Button::new("Submit")
    .style(Style::new()
        .background("#3B82F6")    // TOKENS.color.primary 被替换
        .padding(16)              // TOKENS.spacing.md 被替换
        .radius(8)                // TOKENS.radius.md 被替换
        .text_color("#FFFFFF")    // TOKENS.color.background 被替换
    )
```

---

## 实施计划

### Phase 1: 基础设施
- [ ] 创建 `tokens/` 模块结构
- [ ] 定义 `DesignTokens` 数据结构
- [ ] 实现 JSON 解析器

### Phase 2: Auto 解析器
- [ ] 解析 Auto 类型定义
- [ ] 提取 Token 值
- [ ] 验证 Token 完整性

### Phase 3: 平台 Target
- [ ] Vue/CSS Target
- [ ] Rust Target
- [ ] Kotlin Target
- [ ] ArkTS Target
- [ ] C Target

### Phase 4: CLI 集成
- [ ] `auto tokens compile --target vue tokens.at`
- [ ] `auto tokens compile --all tokens.at`
- [ ] 监听文件变化自动编译

### Phase 5: AURA 集成
- [ ] 组件引用 Token 语法
- [ ] 编译时 Token 解析
- [ ] Token 变更触发重新编译

---

## 相关文档

- [Plan 100: a2js → a2ts 移植计划](../plans/100-a2js-to-a2ts.md)
- [Plan 101: DevTools 与热重载](../plans/101-devtools-hotreload.md)
- [a2c + LVGL 架构分析](./a2c-lvgl-analysis.md)
