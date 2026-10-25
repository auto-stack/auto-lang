# PLAN-848 r3 native backlog repair

Frozen repair `dd7076a7f40b35ffda6b876c36f826af3443fb97`, parent6bc5abc60.
Only four driver files changed: native Play verifier, its new discriminating
control script, handoff and source receipt374. Production Play/clock/Host,
Auto APIs/segment math, SDK,AGP and three locks are unchanged. Same Plan r3.

Cause: the original frame-delayed native verifier could continue sending input
after the legitimate backlog/display-gap pause cleared input. It then tested
motion/visibility before the stimulus was accepted. A copied native entry with
one100ms redraw stall reproduced Patrol phase5/no-actions at the original
clock threshold (diagnostic-before). The same stall passes with the repaired
verifier (diagnostic-after). This explains the failure path; no claim is made
that the original uncontrolled stalls had a uniquely identified external cause.

Repair: live input phases observe clock-pause state and use the actual OS/Iced
Resume button, restore the original target, re-arm held motion, and re-post an
unaccepted V batch. Accepted toggle reports prevent double application. A single
V must appear in an accepted report before its visibility assertion. Motion
waits for the original minimum5 real2D/90 real3D actions. All original independent
f32 traces/pixels/Step/Stop/focus-loss assertions remain. Recovery cannot reset
the360-frame phase deadline; focus-loss/explicit Step phases are excluded from
the new clock recovery. After the intentional OS focus-loss witness, the fixture
requests real window focus before its OS Stop click; pointer/PID reachability
checks still reject occlusion. No direct Host replacement, new runtime hook,
threshold change or output/credit waiver is introduced.

Checks on final repair inputs:

- clock-controls-verified:5 actual OS/Auto/GPU positives with one100ms stall
  before motion, during held motion, before unaccepted hide/restore and after
  an already accepted V. Re-posted-unaccepted witnesses exist; accepted-V has
  no duplicate phase17 replay. The no-recovery copy fails at the original
  phase4 deadline. All6 named controls meet their original strict outcomes.
- native-repair:Patrol and relocated source complete phase27; wrong-focus
  remains rejected at phase5 with no real2D actions. Same final runner hash
  `07893426fd483e85583e717a7bc197b529212149f2d86163bc615ae08820e7a4`.
- step:normal/delayed complete phase27; dropped Step times out at phase6;
  duplicate Step fails exactly-one-actual-tick at phase6. Original OS click
  counts, accepted tick observations and all6pixel stages checked by script.
- cargo check for authoring binary, formatting and Git whitespace passed;
  verify_source reports374source/681SDK/17local/29edges/479metadata. Only the
  fixed SDK146 pre-existing warnings remain; no added driver warnings.

Historical failed setup records remain: an overlapping build briefly removed a
probe rlib; the final tool captures and guards library identities before cases.
A real pointer occlusion failed one trial; an attempted has_focus wait after the
fixture's synthetic focus-loss message timed out and was removed. Neither is
counted as pass; all final6 controls above ran sequentially on final verifier.

Work-owned closure: M848-NATIVE-BACKLOG fixed pending independent affected
review. T07/T08 complete again; execution_done8/10, T09 review pending. Original
F-848-R1-01 math/128/32/CPU closure remains unchanged. No whole package/root
tier was run during repair. Final merge-owned gates remain pending, including
the complete user/project/Play/Step matrix in both producer and consumer.
