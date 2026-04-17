# 185: VSCode Extension Reuses Vue Build Output

## Context

The VSCode extension generator (`a2vscode`) currently creates a complete duplicate Vue project inside `gen/vscode/webview-ui/`. This means a second `npm install` and a second Vite build for what is essentially the same frontend code that `gen/vue/` already produces. The user must wait for two full frontend builds when targeting both backends.

## Goal

Eliminate the duplicate webview project. The VSCode extension loads its webview UI from `gen/vue/dist/` instead of maintaining its own `webview-ui/` subfolder.

## Current Structure

```
gen/
  vue/                   # Full Vue project (npm install, shadcn, vite build)
    dist/                # Built output
  vscode/
    src/                 # Extension host code
      extension.ts
      panels/AppPanel.ts
    webview-ui/          # DUPLICATE — separate npm install, separate build
      src/App.vue
      src/main.ts
      package.json
      vite.config.ts
      ...12 more files
    package.json         # Extension manifest
    webpack.config.js
```

## Target Structure

```
gen/
  vue/                   # Full Vue project (single npm install + build)
    dist/                # Built output — shared with VSCode
  vscode/                # Extension scaffold only
    src/
      extension.ts       # Extension activation
      panels/AppPanel.ts # Webview host — loads from ../vue/dist/
    package.json         # Extension manifest
    webpack.config.js
    tsconfig.json
    .vscodeignore
    .vscode/
```

## Changes

### 1. AppPanel.ts resource paths

Change webview asset resolution from `webview-ui/dist/` to `../vue/dist/`:

```ts
// Before
vscode.Uri.joinPath(this._extensionUri, 'webview-ui', 'dist', 'assets', 'index.js')
// After
vscode.Uri.joinPath(this._extensionUri, '..', 'vue', 'dist', 'assets', 'index.js')
```

Same change for the CSS URI.

### 2. localResourceRoots

Allow the extension to load resources from the sibling `vue/dist/` directory:

```ts
// Before
localResourceRoots: [
    vscode.Uri.joinPath(extensionUri, 'webview-ui'),
    vscode.Uri.joinPath(extensionUri, 'media'),
]
// After
localResourceRoots: [
    vscode.Uri.joinPath(extensionUri, '..', 'vue', 'dist'),
    vscode.Uri.joinPath(extensionUri, 'media'),
]
```

### 3. Build flow in build_vscode_project

New build sequence:

1. **Ensure gen/vue/ is built**: Call the Vue generator and run `npm install && npm run build` in `gen/vue/` to produce `gen/vue/dist/`.
2. **Generate extension scaffold**: Only generate extension-specific files into `gen/vscode/` (no webview-ui/).
3. **Install extension deps**: Run `npm install` in `gen/vscode/` (for the extension's own TypeScript compilation deps).
4. **Compile extension**: Run `npm run compile` (webpack) to bundle extension.ts and AppPanel.ts.

### 4. Remove from VSCode generator

Delete all `webview-ui/*` file generation:

- `webview-ui/src/App.vue`
- `webview-ui/src/main.ts`
- `webview-ui/src/env.d.ts`
- `webview-ui/src/assets/index.css`
- `webview-ui/package.json`
- `webview-ui/vite.config.ts`
- `webview-ui/tsconfig.json`
- `webview-ui/tailwind.config.js`
- `webview-ui/postcss.config.js`
- `webview-ui/index.html`

Remove from `package.json` scripts:
- `webview:install`
- `webview:build`

Remove from `.vscodeignore`:
- `webview-ui/src/**`
- `webview-ui/node_modules/**`
- `webview-ui/index.html**`

### 5. compile_at_to_vue removal

The `compile_at_to_vue()` function in vscode.rs is no longer needed. The VSCode generator no longer generates Vue code — it reuses `gen/vue/dist/`.

## Files Modified

- `crates/auto-man/src/vscode.rs` — remove webview-ui generation, update build flow, update template strings for AppPanel.ts

## Verification

1. `AUTO_BACKEND=vscode auto run` from a UI example directory
2. Verify `gen/vue/dist/` is built first
3. Verify `gen/vscode/` contains only extension files (no webview-ui/)
4. Verify VSCode opens and the webview renders correctly
5. Verify `auto gen` for vue backend still works independently
