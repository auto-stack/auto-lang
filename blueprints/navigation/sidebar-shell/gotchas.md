# Gotchas — navigation/sidebar-shell

### Forking the shell per page

**Wrong**
Each route re-implements its own header + sidebar copy.

**Why**
The shell is the app's single frame. Per-page forks drift within weeks —
different badge counts, missing the new nav entry, inconsistent user menus
(the store-twin lesson at shell scale).

**Right**
Mount the shell once; routes render into the `content` EDIT region. Nav and
user identity come from props/dataSource, never from page-local copies.

### Duplicating user/session state inside the shell

**Wrong**
```auto
model {
    var user_name str = ""
    // …populated by a second session store owned by the shell
}
```

**Why**
Platform services (session, theme, workspace) are singleton-injected. A
shell-private copy drifts from the real session — the user menu shows a
stale identity after account switch.

**Right**
Take `user` via the prop contract from the app's single session source, and
route identity changes through `actions.sign_out` — never a private store.

### Building the content slot with bp-owned chrome

**Wrong**
Wrapping the `content` region in page padding, headings, or scroll
containers owned by the shell.

**Why**
The routed surface needs full layout control; shell-owned chrome fights
route-level layout and produces double scrollbars and misaligned headers.

**Right**
Keep `content` a single transparent slot; anything visual belongs to the
header/sidebar regions.
