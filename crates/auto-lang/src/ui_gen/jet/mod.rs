//! Jetpack Compose Code Generator
//!
//! Generates Jetpack Compose Kotlin code from AURA widgets.
//!
//! ## Architecture
//!
//! ```text
//! AuraWidget → JetGenerator → Kotlin/Compose Code
//!                 │
//!                 ├── Material3Registry (component mappings)
//!                 ├── ModifierDsl (Tailwind → Compose)
//!                 └── StateConverter (state management)
//! ```
//!
//! ## Output Format
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

mod generator;
mod components;
mod modifier;
mod state;
mod theme;

// Re-export main types
pub use generator::JetGenerator;
pub use components::Material3Registry;
pub use modifier::{ModifierDsl, ModifierResult};
pub use state::StateConverter;
// pub use theme::ThemeConfig; // TODO: implement in future phase
