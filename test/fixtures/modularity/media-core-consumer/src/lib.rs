//! PLAN-811 media-core consumer probe — full external-consumer face
//! (T-02 skeleton extended to the T-07 V-01..V-06 acceptance surface).
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

/// Bounded caches/scheduler invariants reachable outside the language crate.
/// Asset ids come from real registry tickets — the raw u128 field stays
/// crate-private (no construction API is opened for external consumers).
pub fn scheduler_cache_smoke() -> bool {
    use auto_media_core::image_pipeline::{
        DecodedPixelCache, EncodedByteLru, MediaAssetKey, MediaAssetRegistry, MediaLatestWins,
        MediaMetadata, MediaOrientation, MediaPin, MediaPriority, MediaPriorityQueue,
        MediaWork, RenditionSpec,
    };
    use std::sync::Arc;
    use std::time::Duration;

    let registry = MediaAssetRegistry::new(Duration::from_secs(60));
    let id_of = |fingerprint: &str| {
        registry
            .queue(
                MediaAssetKey {
                    source_fingerprint: fingerprint.into(),
                    orientation: MediaOrientation::Normal,
                    rendition: RenditionSpec::original(),
                    revision: 1,
                },
                MediaMetadata::default(),
            )
            .id
    };
    let a = id_of("consumer-a");
    let b = id_of("consumer-b");
    let c = id_of("consumer-c");

    let mut encoded = EncodedByteLru::new(4);
    encoded.insert(a, Arc::<[u8]>::from([1, 2]));
    encoded.insert(b, Arc::<[u8]>::from([3, 4]));
    let _ = encoded.get(a);
    encoded.insert(c, Arc::<[u8]>::from([5, 6]));
    if encoded.get(b).is_some() {
        return false; // LRU evicted the cold entry
    }

    let mut decoded = DecodedPixelCache::new(4);
    decoded.insert(a, Arc::<[u8]>::from([5, 6, 7, 8, 9, 10]), MediaPin::Hard);
    // Hard-pinned current may temporarily exceed budget.
    if decoded.used_bytes() != 6 {
        return false;
    }
    decoded.set_pin(a, MediaPin::None);
    decoded.collect();
    if decoded.used_bytes() > 4 {
        return false;
    }

    let mut queue = MediaPriorityQueue::new();
    if !queue.push(MediaWork::new(10, MediaPriority::Current))
        || !queue.push(MediaWork::new(11, MediaPriority::Thumbnail))
        || queue.push(MediaWork::new(10, MediaPriority::Current))
    {
        return false; // duplicate rejected
    }
    if queue.pop().map(|w| w.id) != Some(10) {
        return false; // priority order
    }

    let mut gate = MediaLatestWins::new(1, 1);
    gate.advance(2, 2);
    !gate.accept_publish(1, 1) && gate.dropped() == 1
}

/// The local media file service (index/tokens/Range) reachable standalone.
pub fn media_service_smoke() -> bool {
    use auto_media_core::media_service::{content_range, parse_range, StreamPlan};

    match parse_range(Some("bytes=10-"), 100) {
        StreamPlan::Partial { start, end } if start == 10 && end == 99 => {}
        _ => return false,
    }
    if parse_range(Some("bytes=100-"), 100) != StreamPlan::Unsatisfiable {
        return false;
    }
    content_range(0, 99, 100) == "bytes 0-99/100"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_consumer_reaches_the_public_media_surface() {
        assert!(data_plane_smoke());
    }

    #[test]
    fn external_consumer_reaches_caches_scheduler_and_gates() {
        assert!(scheduler_cache_smoke());
    }

    #[test]
    fn external_consumer_reaches_media_file_service_rules() {
        assert!(media_service_smoke());
    }
}
