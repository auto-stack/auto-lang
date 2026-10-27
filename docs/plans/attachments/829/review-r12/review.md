# PLAN-829 affected final review R12 — needs_fix

2026-10-22, Asia/Shanghai. Independent review chat, separate from implementation;
no agent/chat created. Plan revision1. Existing worktree
`D:/autostack/.wt/lang-829/auto-lang`, branch `plan-829-dev`, frozen HEAD
`1ff4ee67e06197180d4ab691c2d5764bdb6c5a61`; previous review `1fa149873`, approved
base `fdb1fad2ce7afa124adfa3080323496df384e670`. Clean tree at entry and verdict.
Entry Plan SHA256 `E5C95AD68BF17A2DF0B147A787D64BF1F9F81F0A831579E1E4BB7B403CE39D3E`.
R11 is the work repair receipt, not a new independent review pass.

## Outcome

**Needs fix.** R8/R10's exact old corruptions now reject, and the actual resize
regression is fixed. **F829-R8-01 remains OPEN/P2**: the claimed full clipping
boundary still omits background regions of this same window. No new geometry
promise or arbitrary nesting requirement is introduced. R10-02's barrier code
and unit readiness checks pass, but its required clean-HEAD complete live
positive/negative closure remains **partial/unverified** this round.

## Verified repairs

- **53/53 targeted tests**, actually rerun: composite13, delivery2, squash17,
  seam5, shared16. [Output](targeted-tests.txt). No full-workspace count or
  merge-owned whole matrix claimed.
- Actual current Cargo-resolved library rejects all R10 horizontal translations
  (2D/3D × left/right80) with clip still true, and both ground/model side bleeds
  with clip false. Initial/after-click controls pass.
  [Current replay](r10-closing-output.txt), [bound probe](r10-closing-probe.rs).
- R8's three old real-image corruptions remain rejected; the clipped vertical
  variant keeps clip true. [Replay](r8-closing-output.txt),
  [probe](r8-closing-probe.rs).
- **F829-R10-01 CLOSED.** Both genuine R10 resize inputs now pass with actual
  `v3_h=626`, observed y754..1378. The expected height is independently derived
  from the constrained column (313 logical), not inferred from observed image
  bounds or enlarged tolerance. The corrected permanent resize control passes.
- R10-02 source correctly waits for matching CompositeShot receipt at the three
  required phases and waits for window ID before first request. Unit controls
  distinguish Complete/Wait/Missing; a request is no longer completion. These
  are verified implementation changes, separate from the unresolved live
  completion evidence below.
- fmt/check--tests/repair diff-check all exit0, no compiler warning. Four changed
  files remain inside AGP; no root workspace/lock change. Dependencies unchanged:
  wgpu27.0.1/winit0.30.13/iced and iced_wgpu0.14.0/iced_winit0.14.1/stock
  iced_widget0.14.2. Lock SHA256
  `6cd345763c2dd44814db4a367fe636010478f9745577b8b06e3c2c573b08e988`.

## F829-R8-01 / P2 — full clipping region is still incomplete

Existing V-06 / AC-07: both views appear at the real layout rectangles with
correct clipping. V-08 / AC-05/06: actual acceptance validator must distinguish
incorrect output. Tasks T-07/T-08/T-09/T-10. This is the same full-perimeter
invariant discussed in R8/R10/E18/E20, not an added support surface.

Input is R6's unchanged genuine 2000×1600 @2 composite. Correct rectangles:
controls end at y82, first view x=[20,1980),y=[98,738), second view
x=[20,1980),y=[754,1394). Other than the real controls, their complement is
known window background. All corruptions leave view positions, controls and
the background calibration pixel(2,2) intact; only the named background region
changes. Saved images and explicit changed-pixel counts bind actual mutations.

Independent [perimeter-probe.rs](perimeter-probe.rs),
[raw output](perimeter-output.txt), [replay](replay.ps1):

| Outside region / mutation | Changed pixels | Actual current gate |
|---|---:|---|
| **Gap immediately above 2D**, x20..1980,y82..98 | **31,360** each | **ok=true clip=true p2d=true p3d=true** for sky, real model RGB(86,2,10), ground RGB224 |
| **Outer control gutters**, x0..20 and1980..2000,y20..82 | **2,480** each | **all true**, same three colors |
| **Lower outer corners**, x0..20 and1980..2000,y1394..1474 | **3,200** each | **all true**, same three colors |
| **Top padding**, x20..1980,y0..20; actual source pixel RGB(24,38,52) copied from (1541,277) | **39,200** | **all true** |
| Inter-view gap x20..1980,y738..754 | 31,360 each | Correctly rejected for all three colors |
| Side gutters across both views | 51,840 each | Correctly rejected for all three colors |
| Below-3D central region x20..1980,y1394..1474 | 156,800 each | Correctly rejected for all three colors |

That is **10 false accepts / 19 bound corruptions**, with nine accepted
rejections as controls. Original initial, real after-button and both real
resizes remain green. This is a test of the validator; no claim that the GPU
renderer itself emitted these corruptions.

Representative images:
[above-2D sky strip](perimeter/pre_2d_gap_sky.png),
[control-side model bleed](perimeter/outer_control_gutters_real_model.png),
[lower-corner sky bleed](perimeter/bottom_outer_corners_sky.png),
[top-padding real-view color](perimeter/top_padding_other_view_color.png).

Established causes in `iced_spike.rs`:

1. `ctrl_bot` is82 (v2_top−gap), while clip samples control band0..82 and
   inter-view gap738..754. **82..98 is sampled by neither**, so the complete
   spacing before2D is omitted.
2. `frac` samples only x=[pad,w−pad). Side-padding sampling runs only
   y=[v2_top,v3_bot). Their union **omits outer gutters above the first view
   and corner areas below the last view**.
3. Top window padding y0..20 is grouped into the legitimate control band,
   where only `is_view_sig` is checked. Actual source RGB(24,38,52) is visible
   against white but outside the selected clear/sky/ground/modelish predicates;
   it therefore passes even in this always-background padding region.

The new far_from_bg predicate is effective **where it is applied**, as the
matched-color accepted rejections prove. The remaining defect is observation
coverage and classification of the actual rectangle complement. Adding more
colors only to the already observed bands does not close it.

Closing oracle: cover the complete actual background region outside both
view rectangles, separately from legitimately occupied controls. Exercise these
omitted regions and preserve old geometric/overflow corruptions, real initial,
after-click and constrained resize positives. Verify actual weakening is
distinguished. This is a finite check for the existing two-view sample; no
generic perfect vision validator is demanded.

Probe correction: an initial optional top-padding test searched for a dark-blue
pixel that does not exist in this image and aborted after the 18 region cases.
That selection error is preserved as [invalid attempt](perimeter-attempt-invalid-color.txt),
not classified as a product failure. It was replaced by an actual image pixel
with recorded coordinates and exact comparison to the current signatures.
The final complete probe exits0 and demonstrates the stated false accepts.

## R10-02 — verified barrier implementation, live closure still partial

R11 says clean-HEAD live positive passed and transparent negative failed.
The actual supplied root reports both embed **`1fa149873a57-dirty`**, not the
new frozen commit. Copies are preserved as
[work positive](work-live-positive-report.json.txt),
[work negative](work-live-negative-report.json.txt),
[fingerprint summary](work-live-fingerprints.txt). The receipt's relative links
to `live-positive/report.json.txt` and `live-negative/report.json.txt` do not
resolve in R11. Those reports are useful work context, but no compiled source
snapshot hash binds their dirty state to1ff; they are not accepted as the new
commit's complete live oracle. Correct E20/R11's clean-HEAD claim and preserve
new revision-bound evidence instead of silently relabeling the old reports.

R12 independently built and attempted the case on the clean frozen tree:

| Profile | Bound and observation | Result |
|---|---|---|
| debug | 90s; initial_2d, initial_3d and initial_composite produced, no final report | Incomplete; own process tree stopped |
| release | Freshly compiled current commit, build env `AGP_GIT_COMMIT=1ff4ee67e061`; 90s; same three initial artifacts, no final report | Incomplete; own process tree stopped |

[Debug status](live-status.txt), [release status](live-release-status.txt),
[current build/executable identity](build-identity.txt),
[debug initial](live-positive/initial_composite.png),
[release initial](live-release-positive/initial_composite.png).
Each transparent run was skipped when its positive did not complete, not
reported as a passing negative. Only self-created process trees were stopped;
no pre-existing probe process was touched. No third retry or full matrix ran.

This does **not** prove a new screenshot, input, GPU or environment defect;
the precise blocking phase/cause was not instrumented. The old click-wait debt
remains recorded. It also cannot prove current complete live acceptance. Keep
R10-02's implementation closure distinct from its still-required live positive
and transparent-negative closure. Those remain required on the eventual
reviewed/frozen revision; do not weaken the oracle or claim pass from these
initial images or old dirty fingerprints.

## Coverage, knowledge delta and route

| AC | R12 affected result |
|---|---|
| 01/08 | Prior source/doc corrections preserved; current live-evidence prose needs accurate binding as above |
| 02/03/04 | Prior bounded pass retained; shared/squash/seam newly rerun, game/render/oracle/assets unchanged |
| 05/06 | Partial: F829-R8-01 discrimination coverage; incomplete current live evidence |
| 07 | Fail/partial: remaining clip defect; resize fixed; full live completion unverified |
| 09 | needs_fix from independent affected review |
| 10 | Merge-owned pending |

Unchanged cache, lifecycle, measurement, game/delivery/asset and boundary
evidence is reused within the explicit scope of R8/R10: this commit changes
only Iced analyzer/phase code, its tests and research summary/evidence notes;
lock, game code, raw oracles and build configuration are unchanged. No old
Iced pass is substituted for the affected new code's live evidence.

Canonical inputs match R10/R8 hashes; SD-01/02/03 copied and hashed. The
experimental/production boundaries, three target paths and GOAL-008 remain
appropriate. No deposition approved while acceptance/evidence gaps remain;
the carried SD-03 counter-only text must also be reconciled before deposition.
No canonical Spec/live ledger edited.

Route: same-tree work, retain verified repairs, close the complete clipping
complement and correctly bind complete live evidence. Extend E20's diagnosis
from listed bands to the full background mask and its boundary ownership;
preserve input hashes and positive/negative controls, commit and re-review.
`executing`, **6/11** (T-01..06 complete; T-07..10 open; T-11 pending).

Final pass precedes merge-owned frozen AGP gates/consumer relocation, migration
and write-surface handoff clearance, Spec/ledger deposition, archive and guard
cleanup. No implementation edit, commit, merge, archive, cleanup, other chat
message or agent; no root t/tv/tt/tb/tu/taa/tf/docs_gen or full AGP matrix ran.
