# Gotchas — navigation/sidebar-nav

### Not handling empty folders / tags

**Wrong**
Rendering `for f in .folders` with no else branch, so when the data source
returns `[]` the section header sits above a blank gap.

**Why**
An empty section looks broken, not empty. Users assume the sidebar failed to
load rather than that they have no folders/tags yet.

**Right**
Guard each region: `if .folders.len() == 0 { text "No folders." }` (and the
same for tags) before the `for` loop. Each empty branch is its own small EDIT
decision the consumer can restyle.

### Hardcoding folder names instead of using dataSource

**Wrong**
```auto
folders: list of Folder = [
    { name: "Inbox", ... },
    { name: "Work", ... }
]
```
and never replacing the sample data.

**Why**
The block now only ever shows those two folders. It can't reflect a user's
actual folder set or be reused for another app's taxonomy.

**Right**
Treat the sample list as reference-only seed data; populate `.folders` from
`dataSource.folders()` in `.Init`. The block owns the rendering chrome, not the
folder definitions.

### Missing active-state highlighting

**Wrong**
Rendering every shortcut / folder button with the same `variant: "ghost"` and
no notion of `.active`, so all rows look identical.

**Why**
With no selection affordance the user can't tell which shortcut, folder, or tag
is currently driving the list — navigation becomes guesswork.

**Right**
Track `.active` and vary the row on it: `variant: if .active == f.name { "secondary" }
else { "ghost" }`. Update `.active` in every selection msg
(`SelectShortcut` / `SelectFolder` / `SelectTag`).
