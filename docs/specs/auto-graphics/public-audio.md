# Finite public audio and accepted one-shot playback

The independent Windows authoring engine admits finite public one-shot sound
resources. Explicit `autographics.audio2d/1` (`--behavior-api audio2d`) and
`autographics.audio3d/1` (`audio3d`) profiles add public audio to the delivered
sprite2d and node-animation3d compositions respectively. A normal bound-project
Run opens the default native CPAL stream automatically. Unsupported/no output
device/config/format/build/play fails as `audio_unavailable`; it never selects a
silent substitute. Native output supports F32, I16, U16; mono output averages the
two mixer channels, stereo keeps them, additional device channels receive zero.
PCM observations describe this service's submitted samples after channel routing
and quantization; they are not system recording or physical speaker measurements.

Input is Ogg/Vorbis or little-endian RIFF/WAVE PCM16, mono/stereo,
22050/44100/48000Hz, <=8MiB raw, <=10s, <=16MiB decoded normalized stereo f32 PCM.
RIFF/container/CRC/packet spans, dimensions, multiplication, setup/VQ/VLC and
predicted working frames are checked before decoder construction/allocation.
Decode is bounded block-by-block and actual lengths/PCM are checked afterwards.
Pinned Vorbis lookup1 setup arithmetic not representable in checked integer
prediction is `audio_unsupported`; invalid data and exceeded budgets are
`audio_invalid` and `audio_resource_limit`. The bounded first/last Ogg valid
interval removes declared padding, including one-page first+last audio data;
meaningful samples are retained. No arbitrary Vorbis configuration is promised.
Import/decode is synchronous and timed separately from warm VM/native mixing.

Scene7 declares <=8 named cues, each referencing an accepted audio identity and
nonempty half-open `[start_frame,end_frame)` interval, finite gain `[0,1]`.
Names are unique/nonempty/<=128bytes. Multiple cues share one immutable PCM owner.
`ag_audio_emit(ordinal int,cue int,gain float) int` costs1credit, permits <=4requests
per callback and <=8simultaneous voices/Run. Ordinals are exact signed integers
in1..2147483646, strictly increasing across the Run and in each packet. Exhaustion,
duplicate/stale ordinal, bad identity/span/gain or the ninth voice refuses the
whole candidate, with no stealing/dropping/unbounded queue.

Wire Run/tick/bundle/count/ordinal/cue are typed integers (not round-tripped through
f32). Four fixed request rows accompany the whole logical candidate; unused rows
are canonical zero. Host independently validates identity/domain/capacity and
prepares the complete visual/resource candidate before common logical accept.
Only that accept inserts voices once. Exact latest transport replay is acknowledged
without reaccepting or inserting voices; older/replaced tick or old Run is rejected
without effects. Render/capture/multiple portals never resubmit sound.

The native callback and offline oracle share the same bounded mixer, linear
resampling, multiplied cue/request gain, summed/clamped stereo mix. Effect begins
at the next available service quantum after acceptance. Hardware already-buffered
sound cannot be withdrawn; visual/audio hardware simultaneity is not promised.
Preaccept failure keeps last accepted Frame/GPU image and adds no voice; Fault
cancels the Run's already-existing voices. Already emitted samples are not rolled
back. Postaccept native output error produces `audio_fault`, clears voices/queue,
preserves last accepted visual, and cannot retry/replay automatically.

Host owns one native manager, retained across healthy Stop and replaced (old
stream dropped before opening a new stream) after output fault, including a round
restart. Host final close drops it. Pause freezes voice cursor; Resume continues
existing valid tails. One already-dispatched candidate may finish under Pause and
its new voices remain frozen. Neutral Step is silent, marks `suppressed_by_step`
and consumes ordinals with no new voices/backlog. Stop/Fault/new Run yields no old
Run PCM from the next service quantum; new Run clears input/request/ordinal state
and old controls reject. Legitimate Editor/history/external static raw/PCM pins can
outlive a Run; they do not retain a playable Run owner. Actual VM/voice/manager/PCM
weak probes, 20cycles per dimension and warm decode/upload0 distinguish lifetimes.

Executable evidence: `public_audio`10, `public_audio_codec`6,
`public_audio_resources`9, `public_audio_host`12, `public_audio_consumer`2,
`public_audio_compat`1 and5selected `--lib audio` units. The fixed source/runner
gate is `tools/check_public_audio.py --controls` (final review/merge-owned), with
`verify_plan012.py --controls` and an exported A→B cold SDK/target offline consumer.
Reports bind actual executable/source hashes, counts, observations and both
dimensions. Empty/partial/old runner proof is rejected. Per-module scoped work
receipts and the final gate must not be confused.

Engine fixtures `samples/public_audio/project2d/scene.agscene` and `project3d/...`
are ordinary bound projects. Normal entry:
`cargo run --locked --offline --bin autographics-authoring -- --open <scene.agscene>` then Run;
temporary override uses `--behavior <audio2d.at|audio3d.at> --behavior-api audio2d|audio3d`.
The platformer art/sounds are CC0 per retained upstream README; upstream code MIT
is retained separately. The generated two-color2D PNG is original CC0 test art.
No font license is inferred from either. Projects, bank, scripts, image/GLB blobs
and provenance travel with the independent engine source consumer.

These are finite engine fixtures, not Gallery content, product ZIP, complete
G2-09/G3-08 games, G12 registration, public HUD or production/multiplatform support.
ag-examples005 must bind this actual delivered engine revision and independently
verify real level event/selected-cue/gain/once-only/Stop behavior after001/002/003
content changes, together with013font/HUD delivery. Do not upgrade011or the full
AG milestone from this engine gate alone. Merge records the actual delivery SHA
and canonical paths; this proposal contains no speculative delivery hash.

Provenance: [auto-ui PLAN-012:r1](../../../../auto-ui/docs/plans/archived/012-autographics-public-audio-and-accepted-cues.md), independent final R3 pass on `7e9802bb55961bf1d22be77892e568f15c9437bf`; equivalent ff-only delivery `78632af39de28881618f9219d07e20a9b7a5b56f` (eight equal patches, identical experimental tree). Frozen SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`, both lock baselines unchanged. Final45case gate and fresh offline consumer passed with controls; full AG/game/Gallery milestones remain pending.
