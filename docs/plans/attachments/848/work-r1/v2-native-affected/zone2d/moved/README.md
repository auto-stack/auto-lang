# zone2d

Open scene.agscene in autographics-authoring, then Run and click the upper 2D canvas. Hold Right to enter, stay in and leave the region. The bound behavior/2 source implements all trigger state in Auto. The cyan marker x advances10 per enter, y advances10 per exit; the door is visible while inside and its x advances0.1 per stay tick. app_start establishes a baseline without an enter event. Pause produces no events, Step one neutral update, Stop restores editing. New Run resets private state.

The circle uses a rectangular AABB proxy. Its yellow corner marker is intentionally a hit at the AABB corner even though that point is outside the rendered circle.

Hidden/transparent objects remain queryable. This source does not filter the region by visibility. No gravity, blocking, rigid bodies or engine enter/exit event queue is added. Copy this complete directory and open its scene in another cwd with the same runner; the original paths are unnecessary.
