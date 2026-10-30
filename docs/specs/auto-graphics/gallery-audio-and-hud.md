# G12 Gallery — real actions and one-shot audio (audio delivered; HUD pending)

Status: **delivered** content knowledge for the audio half (ag-examples
PLAN-005, consumer of the auto-ui/PLAN-012 public-audio delivery
`78632af39de28881618f9219d07e20a9b7a5b56f`). This document records the
enduring rules the G12 gallery audio item relies on; it does not restate the
engine implementation. G12b (public HUD and presentation state) is a separate
pending delivery (ag-examples PLAN-006) and is intentionally **not** declared
here — no HUD subentry, no scene-less placeholder.

## Scope

Capability C12, first stage: exactly one new logical item `g12` (slug
`g12-services`) with exactly one subentry `g12/audio` — public
`autographics.audio2d/1` over `autographics.scene/7`, catalog 12 logical
items / 21 subentries. The gallery grows only this audio item; the logical
title, subentry notes, README and catalog platform note all state that G12b
HUD is pending.

## Frozen audio material

- One derived PCM16 bank, byte-identical to the frozen PLAN-012 derivation:
  `audio/assets/b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d.wav`
  (RIFF PCM16 stereo, 44100 Hz, 22504 frames, 90060 bytes). Recipe:
  Kenney `Starter-Kit-3D-Platformer` @
  `3fa8a04b1c01ab23db43123d4ce814a34c3fc7f0`, channel-duplicate to stereo,
  PCM16 quantize, no resampling, no amplitude editing; max quantization error
  ≤ 1.53e-5 (1/65536, inside the plan's 1/32768 bound). The conversion
  audit ships beside the bank (`audio/bank-audit.json`); the three original
  Ogg sources (5805/6620/5885 bytes, total 18310) are retained unchanged
  under `audio/sources/` with their CC0 provenance texts. The bank license is
  bound root-relative and byte-exact via `license_sources`.
- Scene cues: three non-overlapping spans of that one bank — `jump`
  [0,7403), `coin` [7403,20046), `land` [20046,22504), gains 1.0
  (`main-alt.at` changes only the coin gain to 0.5, so the real mixed PCM
  differs). No fictional hashes or sizes: every identity above is measured.

## Auto-owned cues and pricing

- The real Auto decides every branch, ordinal and cue; the Host never replays
  cues from motion state. Grounded jump edge → one `jump` cue; leaving the
  air → one `land` cue; the first pickup zone of each collectible → one
  `coin` cue plus a true `ag_object_despawn` (a stay can never repeat it).
  Held keys cannot re-fire (pressed bits are consumed once per read); the
  ordinal series strictly increases per Run and resets only on a new Run.
- Measured pricing on the audio2d profile: the character sweep costs 26
  credits in total (ag_char_move 1 + ag_move_and_slide 24 + grounded scan 1 —
  the "sweep is 26" lesson from G11), so the shipped Auto reads at most ONE
  trigger event per callback and its worst accepted callback is exactly 32
  (snapshot 1 + move 26 + grounded 1 + event 1 + despawn 1 + two emits 2).
  A 33rd charged call is a named `QueryBudget` fault; the acceptance drives
  both sides.
- Pickup triggers use shape 1 (circle radius 0.4): `ag_move_and_slide` rests
  exactly `clearance` (0.002) short of an obstacle AABB touch, so a shape-0
  box overlap is unreachable by walking; the closest-point radius test fires
  through that gap (the same option the G11 hazard uses). Scene templates
  that spawn live instances must be parked off the play line — a static
  template node at the spawn site keeps blocking the sweep even after the
  instance despawns (the G11 parking pattern).

## Strict binding and acceptance

- `engine-locks/auto-ui-012.json` pins the delivered consumer: commit
  `78632af…`, workspace Cargo.lock git blobs
  `d7738d27…`/`bc7287b5…`, SDK `7b987ca8…` (681 files / 17 local packages /
  479 metadata / 29 path edges), identity manifest
  `research/plan012-identity.json` (1536 files, SHA `64012630…`), runner
  `autographics-authoring` SHA `b089d10e…` built `--locked --offline`.
  Users pass the runner explicitly; no producer absolute paths are stored.
- Gallery validation treats audio assets as strictly as PNG/GLB: declared
  rate/channels/frames must match a canonical 44-byte RIFF PCM16 header and
  the actual frame count, container identity is checked before anything
  trusts the bytes (Ogg-masquerading-as-WAV refused by name), cue spans must
  sit inside the declared frames with finite gains 0..1, and every asset
  carries its content-addressed license. `verify_packages.py --only g12`
  routes to the G12a runner/consumer explicitly; a missing runner is a
  non-zero refusal, never a silent skip.
- Windowless acceptance (`checks/g12a/gallery_runtime.rs` via
  `tools/verify_g12a.py`): the real Auto drives the public Host; accepted
  packets are rendered through this service's mixer and compared against an
  independent PCM16 read of the shipped bank (≤ 2e-5 max sample diff, no
  production codec involved); the real default-output stream must observe
  nonzero frames during a live Run and run=0 after Stop; lifecycle covers
  Pause, neutral Step (ordinals consumed with `SuppressedByStep`, no new
  voice, nothing replayed on resume), Stop and a fresh Run with a stale-run
  packet rejected. Eight required oracle names are enforced by the validator.

## Offline single package

`gallery.py package g12` ships 34 files: the audio project, complete
resources/licenses, the new lock, a filtered catalog (exactly this logical
ID), the launch tools and the shipped acceptance. Offline A→B relocation
re-validates in both locations, compares `--inspect` identity hashes, runs
the real audio acceptance in A and B, and refuses then repairs bank, license,
source and lock tampering with zero-engine byte restoration.

## Out of scope here

No HUD, no font, no music loops, no spatial audio, no full game (enemies,
win/lose), no production UI. G12b pending; full Gallery and the AG milestone
remain pending.

Provenance: ag-examples [PLAN-005](../../../../ag-examples/docs/plans/archived/005-autographics-gallery-audio-actions.md)
(archived at merge), independent final review pass on `bcc45a1`, delivered
ag-examples master `5996d8f`. Engine: auto-ui PLAN-012 at `78632af…`; SDK
`7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`.
