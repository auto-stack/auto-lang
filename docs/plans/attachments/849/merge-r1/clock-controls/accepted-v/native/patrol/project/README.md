# Patrol

Run `autographics-authoring --open <this-directory>/scene.agscene --behavior main.at`.
The3D parent patrols automatically, reversing every160 logical ticks. Its child
world coordinate is read after the write and sets the2D quad's vertical position;
2D focus/arrows/V separately control horizontal motion/visibility. Pause and Step
show the private timer progressing only on actual ticks. Stop restores editing.

Copy scene.agscene, assets and main.at together to a regular local directory;
relative main.at resolves at that scene root. This is a reusable single-file
project, not an automatic script-package export. Bundled geometry is original
MIT-licensed PLAN-838 test geometry. See ../../../api/README.md for the API.
