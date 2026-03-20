//! ArkTS (HarmonyOS) UI Generator
//!
//! Transpiles AURA widgets to ArkTS code for HarmonyOS applications.
//!
//! # Architecture
//!
//! ```text
//! AURA Widget → ArkGenerator → ArkTS Code (.ets)
//! ```
//!
//! # Generated Project Structure
//!
//! ```text
//! arkts/
//! ├── build-profile.json5
//! ├── oh-package.json5
//! ├── entry/src/main/ets/pages/Index.ets
//! └── ...
//! ```

mod generator;
mod components;
mod state;
mod project;
mod modifier;

pub use generator::ArkGenerator;
pub use components::ArkComponentRegistry;
pub use project::ArkProjectGenerator;
