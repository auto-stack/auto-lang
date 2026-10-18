//! PLAN-811 media-core consumer probe (light skeleton; the full V-01..V-06
//! acceptance face lands with T-07).
//!
//! External-crate position: this fixture depends on `auto-media-core` ONLY
//! and exercises its public surface the way a downstream service would —
//! no auto-lang/auto-man/auto-ui/auto-vm anywhere in the closure.

/// Compile-and-run smoke over the public data plane: registry lifecycle,
/// ticket addressing, and the transport-neutral HTTP rules must be reachable
/// from outside the language crate with unchanged observable behavior.
pub fn data_plane_smoke() -> bool {
    use auto_media_core::image_pipeline::{
        media_http_response, MediaAssetKey, MediaAssetRegistry, MediaAssetState, MediaMetadata,
        MediaOrientation, RenditionSpec,
    };
    use std::sync::Arc;
    use std::time::Duration;

    let registry = MediaAssetRegistry::new(Duration::from_secs(1));
    let ticket = registry.queue(
        MediaAssetKey {
            source_fingerprint: "consumer-smoke".into(),
            orientation: MediaOrientation::Normal,
            rendition: RenditionSpec::viewport(8, 8),
            revision: 3,
        },
        MediaMetadata::default(),
    );
    let uri = format!("/api/__auto/media/{}/{}", ticket.id, ticket.revision);
    // Pending rendition answers 503 through the shared HTTP rules.
    if media_http_response(&registry, "GET", &uri, None).status != 503 {
        return false;
    }
    registry.transition(ticket.id, MediaAssetState::Reading).is_ok()
        && registry
            .transition(ticket.id, MediaAssetState::Decoding)
            .is_ok()
        && registry
            .transition(ticket.id, MediaAssetState::Transforming)
            .is_ok()
        && registry
            .publish_ready(ticket.id, ticket.revision, Arc::<[u8]>::from([1, 2, 3]))
            .is_ok()
        && media_http_response(&registry, "GET", &uri, None).status == 200
}

#[cfg(test)]
mod tests {
    #[test]
    fn external_consumer_reaches_the_public_media_surface() {
        assert!(super::data_plane_smoke());
    }
}
