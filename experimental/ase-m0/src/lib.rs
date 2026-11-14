//! ASE M0 headless experiment harness (PLAN-1021): resolve two registered
//! `app://` identities strictly, read their registered source closures
//! under the allowed root, and execute each once through the public ACE
//! packages chain (`compile_package_product` -> `PackageAvTarget::cold` ->
//! `run_value`). Bounded experiment: no UI, no daemon, no installer, no
//! disk zero-compile cache, no warm body-edit path, no persistent session.
//! The URI is not authorization to execute arbitrary files — only the two
//! explicitly registered local trusted programs are loadable.

pub mod error;
pub mod registry;
pub mod run;
pub mod source;
pub mod uri;

pub use error::{AseError, Kind, Phase};
pub use registry::{load_registry, Registry, RegistryApp};
pub use run::{run_app, RunOptions, RunReport};
pub use uri::parse_app_uri;
