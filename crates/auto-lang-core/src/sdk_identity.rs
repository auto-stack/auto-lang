//! PLAN-982 A slice (route C, 2026-11-17 ruling): the **single SDK identity**
//! that product legs bind to.
//!
//! The A slice proves that the two legs a product exercises — (1) project
//! configuration reading and (2) the minimal CLI script execution — come from
//! **one SDK**, not from two different builds/engines:
//!
//! - the config leg runs on this crate's designated config owner. PLAN-969's
//!   verified spec (`docs/specs/ace-migration/m2-facade.md`, §边界) froze
//!   `eval_config_with_vm` as the **legacy owner** for `pac.at` documents; the
//!   ACE door deliberately does not copy that surface. "Legacy owner" is the
//!   *designated* owner, not a fallback: selecting the ACE candidate never
//!   means silently switching engines, and never means the legacy owner is
//!   replaced by a second interpreter.
//! - the execution leg is the explicit ACE door
//!   ([`crate::ace_entry`] / `run_ace`), served by the host-installed
//!   executor.
//!
//! Both legs are linked from this one crate graph, so a product receipt that
//! records [`sdk_identity`] plus the serving engine identity proves the legs
//! share a single SDK. This module is pure data (no evaluation, no IO, no
//! ACE dependency) — it only names the identity axes.
//!
//! Naming note: this "SDK identity" is about **product legs sharing one SDK
//! build**. It is a different axis from the assembly-manifest "SDK identity"
//! (the build-input/source closure asserted in
//! `stdlib_assembly::manifest::plan814_build_input_boundary`) — that one names
//! which sources go into an assembly, this one names which legs a product
//! exercised.
//!
//! **Not in this module**: capability evidence. The configuration *engine*
//! migration to ACE is a named, unowned capability gap (`GAP-982-CONFIG-ACE`,
//! PLAN-982 §5/§10) that stays with the M2-B slice; this identity makes the
//! current split explicit instead of pretending it is already one engine.

/// Receipt schema for the identity line a product records per run.
pub const SDK_IDENTITY_SCHEMA: &str = "auto.sdk/1";

/// The crate that owns both legs in this SDK (one package, one version).
pub const SDK_PACKAGE: &str = "auto-lang-core";

/// Who evaluates `pac.at` / `am.at` configuration documents today. Frozen by
/// PLAN-969's verified facade spec; migrating it is `GAP-982-CONFIG-ACE`.
pub const CONFIG_OWNER_ID: &str = "legacy-owner(config)";

/// The executing engine identity the ACE door serves (`AceEngineChoice::AceAV`).
pub const EXECUTION_ENGINE_ID: &str = "av";

/// This SDK build's package version.
pub fn sdk_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// The identity of the configuration owner leg.
pub fn config_owner_identity() -> String {
    format!(
        "{SDK_IDENTITY_SCHEMA} package={SDK_PACKAGE} version={} leg=config owner={CONFIG_OWNER_ID}",
        sdk_version()
    )
}

/// The identity of the execution leg.
pub fn execution_owner_identity() -> String {
    format!(
        "{SDK_IDENTITY_SCHEMA} package={SDK_PACKAGE} version={} leg=execution engine={EXECUTION_ENGINE_ID}",
        sdk_version()
    )
}

/// One-line SDK identity shared by every leg of this build.
pub fn sdk_identity() -> String {
    format!(
        "{SDK_IDENTITY_SCHEMA} package={SDK_PACKAGE} version={} config_owner={CONFIG_OWNER_ID} engine={EXECUTION_ENGINE_ID}",
        sdk_version()
    )
}

/// True when the two legs reported by [`config_owner_identity`] and
/// [`execution_owner_identity`] belong to the same SDK package+version. This
/// is the machine-checkable "same SDK" predicate: it never inspects engine
/// semantics, only identity.
pub fn same_sdk(config_leg: &str, execution_leg: &str) -> bool {
    let key = |s: &str| -> Option<(String, String)> {
        let package = s
            .split_whitespace()
            .find(|t| t.starts_with("package="))?
            .to_string();
        let version = s
            .split_whitespace()
            .find(|t| t.starts_with("version="))?
            .to_string();
        Some((package, version))
    };
    match (key(config_leg), key(execution_leg)) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn p982_identity_names_one_package_and_version() {
        let id = sdk_identity();
        assert!(id.starts_with(SDK_IDENTITY_SCHEMA), "schema prefix: {id}");
        assert!(id.contains(&format!("package={SDK_PACKAGE}")), "{id}");
        assert!(id.contains(&format!("version={}", sdk_version())), "{id}");
        assert!(
            id.contains(&format!("config_owner={CONFIG_OWNER_ID}")),
            "{id}"
        );
        assert!(
            id.contains(&format!("engine={EXECUTION_ENGINE_ID}")),
            "{id}"
        );
    }

    #[test]
    fn p982_legs_share_one_sdk() {
        assert!(same_sdk(
            &config_owner_identity(),
            &execution_owner_identity()
        ));
        assert!(same_sdk(&config_owner_identity(), &sdk_identity()));
        // A foreign/different-version leg is NOT the same SDK.
        assert!(!same_sdk(
            &config_owner_identity(),
            "auto.sdk/1 package=auto-lang-core version=9.9.9 leg=execution engine=av"
        ));
        assert!(!same_sdk(
            &config_owner_identity(),
            "auto.sdk/1 package=other-sdk version=0.1.0 leg=execution engine=av"
        ));
        // Malformed identities never pass (no silent equality).
        assert!(!same_sdk("", &execution_owner_identity()));
        assert!(!same_sdk(&config_owner_identity(), "garbage"));
    }
}
