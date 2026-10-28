# Runtime Game Cameras (PLAN-875)

Agreed contract: the public runtime game cameras are a Run-owned finite
service over the existing render surfaces. One Run supports exactly one
camera dimension (2D follow, or 3D orbit) and one character target.

## Identity

| dimension | version | wire number | profile | trusted unit |
|---|---|---|---|---|
| 2D follow | `autographics.runtime_camera2d/1` | 103 | 14 | `<runtime_camera2d_v1.at>` (v5 chain + character/1 unit + camera2d unit) |
| 3D orbit | `autographics.runtime_camera3d/1` | 104 | 15 | `<runtime_camera3d_v1.at>` (v5 chain + character3d/1 unit + camera3d unit) |

Old assemblies (`autographics.behavior/1..5`, `autographics.character/1`,
`autographics.character3d/1`) are byte-identical to their PLAN-874 delivery.
No camera API exists on those profiles; the 858-style default presentation
(2D pixel view, frozen 3D sample camera) is preserved for them unchanged.

## Public inputs

- 2D: `ag_camera_follow_2d(stable_id, min_x, min_y, max_x, max_y, view_w,
  view_h)` once in `app_start`; the target must be this Run's bound
  character; the viewport must fit the rectangle.
- 3D: `ag_camera_orbit_3d(stable_id, distance, pitch_deg, projection,
  ortho_size)` once in `app_start`; `distance` in [4,12] (initial 8),
  `pitch` in [15°,75°] (initial 45°), `projection` 0 = perspective
  (fov 60°, near 0.05, far 256) or 1 = orthographic (size in [1,64]);
  yaw always starts normalized at 0.
- 3D per-tick reads (free, no spatial-charge cost): `ag_cam3_forward_x/z`,
  `ag_cam3_right_x/z`, `ag_cam3_yaw/pitch/distance`, mirroring the host
  candidate: horizontal basis `F=(-sin yaw,0,-cos yaw)`, `R=(cos yaw,0,
  -sin yaw)`; movement is user-owned (`F*forward + R*right`, user speed/dt,
  constant-factor diagonal normalization).

## Fault codes

27 CameraConfiguration (invalid domain / tick-0 missing), 29
CameraAlreadyConfigured (duplicate or post-freeze), 30 CameraTargetInvalid
(unmapped or not this Run's bound character).

## System inputs and presentation (host-owned)

- LMB drag on the focused 3D game surface while Running: yaw += 0.25° per
  logical pixel, pitch += 0.25° (down positive); wheel: distance ×=
  1.1^(-notches) clamped to [4,12], with the 856 physical→logical rule (40
  logical px per notch, ±8 notches per event). Fixed scalar accumulators
  with saturation caps (|yaw|,|pitch| ≤ 720°, |notches| ≤ 64) and a
  saturation diagnostic counter; no queues.
- Up/blur/leave/Pause/Stop/Fault/resize/DPI/modal clear the capture and the
  unconsumed delta; Resume requires a fresh press; a neutral Step moves the
  camera with the character but never orbits.
- One orbit/zoom candidate folds per dispatch, is sent into the tick
  (`_ag_cam3_basis` + `_ag_cam3_orbit`), echoed in the camera output block,
  and checked bit-exactly by the frame admission; configuration echoes are
  drift-checked against the tick-0 freeze.
- 2D presentation: follow center = clamp(actor, min+view/2, max-view/2), no
  interpolation; equal-ratio centered letterbox (scale = min(surface/view);
  the frozen sample rect x[0,30] y[-4,12] with viewport 12×8 letterboxes to
  570×380 with 115 logical px side bars). Bars are opaque draws in the clear
  color and never receive the game pointer.
- 3D presentation: eye = accepted-character center + distance ×
  (sin yaw cos pitch, sin pitch, cos yaw cos pitch), up (0,1,0), aspect 2.0.

## Transaction and lifecycle

The camera outputs ride the Play frame: frame and camera pass
prepare/accept/publish as one transaction; a preparation failure keeps the
last-good pair (previous frame + previous camera) under the existing
Fault/VM-close rules, and a mid-transaction Pause does not decouple the
in-flight echo check. Camera state never enters the Document, scene
serialization, dirty tracking or Undo/Redo. Stop restores the editing
Prepared and View2d/View3d without re-uploading static meshes; a new Run
restarts from the configured initial values.

## Envelope

The 128-node denominator stays the PLAN-874 spatial-scene ruling (wide/sparse
128 nodes, dense-depth 64), with collision membership of the character plus
structure solids; binding 127 solids exceeds the frozen 100000-step
initialization budget (permanent negative case). Fresh-process sampling
(50 warm steps): camera2d-regular ≈3.9ms P50, camera2d-128 ≈42ms,
camera3d-regular ≈4.2ms, camera3d-128 ≈42ms — all far inside the 2s
whole-tick budget; camera reads cost zero spatial credits (the full
controller path stays at the 32-credit cap).
