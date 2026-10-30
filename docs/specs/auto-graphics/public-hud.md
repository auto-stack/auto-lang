# Finite public HUD and shared presentation state

The independent Windows authoring engine provides a finite public text HUD.
Explicit `autographics.hud2d/1` (`hud2d`, version113/profile24) and
`autographics.hud3d/1` (`hud3d`, 114/25) extend delivered audio2d/audio3d.
Scene8 carries one fixed font and exactly four HUD slots. Each slot persists
`visible`, `text`, `anchor`, logical-pixel `offset`, `size` and `rgba`; Auto
emits the complete visible/text state of all four slots, never a partial delta.

Public `ag_hud_set(slot,visible,text)` costs one shared credit. New-profile
`ag_present2d` and `ag_present3d` keep the audio visual+sound prefix and append
four `(visible int,text str)` pairs. One bundle costs one credit and includes at
most one visual, four sounds and all four HUD slots. Standalone HUD writes mixed
with a bundle, duplicate bundles and the existing mixed-write forms reject the
whole callback. The full event branch remains exactly32 credits; a 33rd charged
call rejects. Old17 profiles, signatures, prices and assembled bytes remain
unchanged.

Text is0..32 visible ASCII bytes32..126 per slot and at most128 total. Hidden
and visible-empty states are explicit. Fixed four34-scalar rows follow the
16-scalar audio block: visible,length and32 integer codes, with canonical zero
padding. Host independently validates types, lengths, domains and the accepted
font/layout before common `Play.accept`. Auto produces gameplay and HUD state;
Host never derives them from events or scores.

Only audited Lilita One TTF (26828bytes, SHA256
`4444c23631884f411aa7f2fee35061b1756a9927940c62d2a5572acf470b3ffa`) is admitted.
Scene8 embeds the exact OFL/RFN license (4391bytes, SHA256
`31d45880a38de46bc81835329c0c7d964609751e65add92769ea0095ad92c05b`). No system
font, arbitrary TTF, silent missing-glyph replacement, CJK/IME/rich text or
production UI component is added.

Atlas keys are `(font SHA, rounded logical px16..48, device ordinal)`, at most
four across accepted/history/pending/external owners. Equal sizes share one
atlas. Atlas sides are at most1024 and4MiB; overcapacity and missing glyphs are
named errors. Warm text/visibility changes do not rerasterize/reupload. All raw,
layout, glyph, vertex, atlas, PCM and service owners share the unchanged64MiB
CPU/GPU and8-identity/32MiB raw budgets.

HUD draws last on the game surface, in logical coordinates with a physical
scissor. World cameras do not move HUD anchors. Two-dimensional letterbox and
all four anchors use the accepted viewport; resize/DPI maps that logical space
without creating new font sizes/atlases. HUD does not enter letterbox bars,
editor panels or3D depth. Preaccept failure preserves the complete last-good
visual/HUD and inserts no new voice. Postaccept native output failure follows
012: named Fault, last accepted visual/HUD retained, already emitted samples
not withdrawn and no automatic replay.

Pause dispatches no candidate. Neutral Step publishes one visual/HUD tick and
consumes sound ordinals as `suppressed_by_step`, without a new voice or Resume
backlog. Stop restores editing defaults; fresh Run starts from document defaults
then `app_start`. Twenty-cycle owner/warm evidence distinguishes healthy Host
reuse from VM/Run release and legitimate final external pins.

Executable directed evidence: `public_hud`17, `public_audio`10,
`public_audio_codec`6, `public_audio_resources`9, `public_audio_host`12,
`public_audio_compat`2, `public_hud_consumer`2 and5 selected audio units. Final
`tools/check_public_hud.py --controls` binds source/runners, actual GPU/PCM,
owners and fresh independent consumer; partial/zero-selected/stale reports are
rejected. `verify_plan013.py --controls` protects complete source identity.
The frozen gate is merge-owned; this document does not treat a scoped work or
review run as the complete gate.

Engine fixtures `samples/public_hud/project2d` and `project3d` are finite bound
projects with real Auto scripts, visual assets, a three-cue PCM bank and the
fixed font. The scene embeds the font license and each project also retains
`FONT-LICENSE.txt`; other art/sound provenance and upstream code license remain
separate. A normal entry is `autographics-authoring --open <scene.agscene>` then
Run.

These fixtures are not a full two-level game, production HUD/UI, arbitrary-font
support or a complete AG milestone. ag-examples/006 G12b may bind the actual
delivered engine revision and must independently verify real glyphs,
event/selected-cue/gain/once-only audio and Stop behavior after its own content
changes.

Provenance: [auto-ui PLAN-013:r1](../../../../auto-ui/docs/plans/archived/013-autographics-public-hud-and-presentation-state.md),
independent final R2 pass on `52f370a4af1a10d1af7ac4dff1e53aba105050d1`; two
range-diff-equal rebase hops map the reviewed implementation to `b3ebe42` and
the ff-only delivery is `8b1e799aaa158c8ffed12f95173a859be590ceb6` with an
identical experimental tree. Frozen SDK
`7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`, both lock baselines unchanged.
Complete directed gate, 1577-hash source identity and the fresh offline A→B
consumer passed with controls on the delivery tip; full AG/game/Gallery
milestones remain pending.
