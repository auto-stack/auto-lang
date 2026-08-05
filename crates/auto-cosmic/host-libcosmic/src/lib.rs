//! Plan 365 W3: libcosmic host backend (VTree → libcosmic Element).
//!
//! This is Host ② (the COSMIC deliverable) from Plan 365 D2. It lowers the
//! platform-neutral `VTree` to libcosmic/iced `Element`s, producing a real
//! Wayland citizen (layer-shell, xdg activation, cosmic-protocols). Replicated
//! components can be dropped into a real COSMIC session and verified against
//! the upstream compositor.
//!
//! ## Status
//!
//! **Scaffold** — the crate structure and design are in place; the actual
//! VTree→Element lowering requires `libcosmic` (Linux/Wayland-only) and is
//! implemented incrementally as COSMIC components are replicated. See the
//! module-level docs of each function for the lowering strategy.
//!
//! ## Build
//!
//! Linux/WSL2 only. Excluded from the default workspace build on Windows.
//! On Linux, uncomment the deps in `Cargo.toml` and:
//! ```sh
//! cargo build -p auto-cosmic-host-libcosmic
//! ```

// Placeholder: the real impl is gated on `target_os = "linux"` once libcosmic
// is wired. On non-Linux, this crate compiles to an empty dylib so it doesn't
// break the workspace if accidentally included.

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(not(target_os = "linux"))]
compile_error!(
    "auto-cosmic-host-libcosmic is Linux-only (libcosmic/Wayland). \
     It is excluded from the workspace by default; do not build on Windows."
);
