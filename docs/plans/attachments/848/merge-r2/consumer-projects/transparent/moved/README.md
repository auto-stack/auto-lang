# Bound move project

Copy this entire directory anywhere on a supported local Windows drive. Start the same runner with `autographics-authoring --open scene.agscene`, then click Run. No --behavior or original source path is needed. scripts/<sha256>.at is the exact accepted single-file source; assets/<sha256>.glb is the declared model. Keep all declared files with the scene.

To edit the behavior, copy its script to a user .at file, select the file in Editing, click Bind/update script, then Save. Undo/Redo restores accepted byte snapshots. Apply bound actors or No bound 2D/No bound 3D changes persistent roles. Use project default runs the saved binding; external source/built-in selection is a temporary override. Stop restores the editing document. This is an experimental trusted local project, not an executable installer.
