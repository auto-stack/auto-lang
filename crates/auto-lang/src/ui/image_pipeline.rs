//! Shared, renderer-neutral image asset pipeline.
//!
//! The public media types, bounded scheduler, cache registry, and HTTP helper
//! live here so VM, generated Rust, and an HTTP backend use identical asset
//! identities and lifetimes.  Task 1 deliberately provides only the module
//! seam; its types and behavior are added in the following tasks.

