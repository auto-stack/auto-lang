//! Jetpack Compose Code Generator (a2jet)
//!
//! Generates Jetpack Compose Kotlin code from AURA widgets, producing
//! modern Android apps with Material3 design.
//!
//! ## Architecture
//!
//! ```text
//! AuraWidget → JetGenerator → Kotlin/Compose Code
//!                 │
//!                 ├── Material3Registry (component mappings)
//!                 ├── FormGenerator (inputs, buttons)
//!                 ├── LayoutGenerator (Column, Row, Box)
//!                 ├── ListGenerator (LazyColumn, Grid)
//!                 ├── NavigationGenerator (NavHost)
//!                 ├── ModifierDsl (Tailwind → Compose)
//!                 ├── StateConverter (model → mutableStateOf)
//!                 └── ProjectGenerator (full Android project)
//! ```
//!
//! ## Features
//!
//! - **Form Components**: Input, Checkbox, Switch, Slider, Textarea
//! - **Layout Components**: Column, Row, Box, Card, Scroll
//! - **List Components**: LazyColumn, LazyRow, LazyVerticalGrid, FlowRow
//! - **Navigation**: NavHost with type-safe routes
//! - **Modifier DSL**: Tailwind classes → Compose Modifiers
//! - **Project Generation**: Complete Android project structure
//!
//! ## Quick Start
//!
//! ### Generate a Widget
//!
//! ```rust,ignore
//! use auto_lang::ui_gen::jet::JetGenerator;
//! use auto_lang::ui_gen::BackendGenerator;
//!
//! let mut gen = JetGenerator::new();
//! let kotlin_code = gen.generate(&aura_widget)?;
//! ```
//!
//! ### Generate a Full Project
//!
//! ```rust
//! use auto_lang::ui_gen::jet::JetGenerator;
//!
//! let gen = JetGenerator::new();
//!
//! // With defaults
//! let files = gen.generate_project_default("MyApp");
//!
//! // With custom package
//! let files = gen.generate_project_with_package("MyApp", "com.company.myapp");
//!
//! // With custom theme
//! let files = gen.generate_project_with_theme("MyApp", "#6750A4", "#625B71");
//! ```
//!
//! ## Output Format
//!
//! Generated Kotlin code follows Android best practices:
//!
//! ```kotlin
//! package com.example.widgets
//!
//! import androidx.compose.foundation.layout.*
//! import androidx.compose.material3.*
//! import androidx.compose.runtime.*
//! import androidx.compose.ui.Modifier
//! import androidx.compose.ui.unit.dp
//!
//! @Composable
//! fun MyWidget(modifier: Modifier = Modifier) {
//!     var count by remember { mutableStateOf(0) }
//!
//!     Column(modifier = modifier) {
//!         Button(onClick = { count++ }) {
//!             Text("Click: $count")
//!         }
//!     }
//! }
//!
//! @Preview(showBackground = true)
//! @Composable
//! fun MyWidgetPreview() {
//!     MyWidget()
//! }
//! ```
//!
//! ## Project Structure
//!
//! Full project generation creates:
//!
//! ```text
//! myapp/
//! ├── app/
//! │   ├── src/main/
//! │   │   ├── java/com/example/myapp/
//! │   │   │   ├── MainActivity.kt
//! │   │   │   └── ui/
//! │   │   │       ├── theme/
//! │   │   │       │   ├── Theme.kt
//! │   │   │       │   ├── Color.kt
//! │   │   │       │   └── Type.kt
//! │   │   │       └── widgets/
//! │   │   └── AndroidManifest.xml
//! │   └── build.gradle.kts
//! ├── build.gradle.kts
//! ├── settings.gradle.kts
//! └── gradle/libs.versions.toml
//! ```

mod generator;
mod components;
mod form;
mod layout;
mod list;
mod modifier;
mod navigation;
mod project;
mod state;
mod theme;

// Re-export main types
pub use generator::JetGenerator;
#[allow(deprecated)]
pub use components::Material3Registry;
pub use form::FormGenerator;
pub use layout::LayoutGenerator;
pub use list::ListGenerator;
pub use modifier::{ModifierDsl, ModifierResult};
pub use navigation::NavigationGenerator;
pub use project::{JetProjectConfig, ProjectGenerator, ThemeColors};
pub use state::StateConverter;
// pub use theme::ThemeConfig; // TODO: implement in future phase
