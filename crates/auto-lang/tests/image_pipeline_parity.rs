#![cfg(feature = "ui-iced")]

use auto_lang::ui::image_pipeline::{
    MediaAssetKey, MediaAssetRegistry, MediaAssetState, MediaMetadata, RenditionSpec,
};
use std::sync::Arc;
use std::time::Duration;

fn run_sequence(registry: &MediaAssetRegistry) -> (MediaAssetState, usize, u64) {
    let ticket = registry.queue(
        MediaAssetKey {
            source_fingerprint: "parity-fixture".into(),
            orientation: Default::default(),
            rendition: RenditionSpec::viewport(32, 32),
            revision: 4,
        },
        MediaMetadata::default(),
    );
    registry.transition(ticket.id, MediaAssetState::Reading).unwrap();
    registry.transition(ticket.id, MediaAssetState::Decoding).unwrap();
    registry.transition(ticket.id, MediaAssetState::Transforming).unwrap();
    registry
        .publish_ready(ticket.id, ticket.revision, Arc::<[u8]>::from([1, 2, 3, 4]))
        .unwrap();
    let state = registry.wait_for_terminal(ticket.id, Duration::from_millis(1)).unwrap();
    let stats = registry.stats();
    (state, stats.encoded_bytes, stats.completed)
}

#[test]
fn vm_and_a2r_image_pipeline_sequence_has_matching_observable_state() {
    let vm = MediaAssetRegistry::new(Duration::ZERO);
    let rust = MediaAssetRegistry::new(Duration::ZERO);
    assert_eq!(run_sequence(&vm), run_sequence(&rust));
}
