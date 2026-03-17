# Migration Guide: Unified Backend Output Structure

This guide explains how to migrate from the old directory structure to the new unified backend output structure introduced in Plan 129.

## What Changed

### Old Structure (Before Plan 129)

```
my-project/
├── pac.at
├── source/
│   ├── front/
│   │   ├── app.at
│   │   └── pages/
│   └── back/
└── dist/                    # Generated Vue project (or Jet at root)
```

### New Structure (After Plan 129)

```
my-project/
├── pac.at
├── source/
│   ├── front/
│   │   ├── app.at
│   │   └── pages/
│   └── back/
├── vue/                     # Generated Vue project
├── jet/                     # Generated Jetpack project
├── tauri/                   # Generated Tauri project
└── back/                    # Generated Rust backend
```

## Key Changes

### 1. Output Directory Names

| Backend | Old Directory | New Directory |
|---------|---------------|---------------|
| Vue     | `dist/`       | `vue/`        |
| Jetpack | project root  | `jet/`        |
| Tauri   | `dist/`       | `tauri/`      |
| Rust    | N/A           | `back/`       |

### 2. New Commands

Old commands (deprecated):
```bash
auto vue [input.at] -o ./output
auto jet [input.at]
```

New commands:
```bash
auto build              # Build all backends
auto build --target vue # Build only Vue
auto run                # Run dev server for all backends
auto run --target vue   # Run only Vue dev server
```

### 3. pac.at Configuration

New `backend` field in `pac.at`:

```auto
// Single backend
backend: "vue"

// Or split frontend/backend
backend: {
    front: ["vue", "jet"]
    back: "rust"
}
```

## Migration Steps

### For Vue Projects

1. **Backup your generated code** (if needed):
   ```bash
   mv dist dist-backup
   ```

2. **Run the new build command**:
   ```bash
   auto build --target vue
   ```

3. **Verify output in `vue/` directory**:
   ```bash
   ls vue/
   ```

4. **Update .gitignore**:
   ```gitignore
   # Old
   # dist/

   # New
   vue/
   ```

### For Jetpack Projects

1. **Generated code now goes to `jet/`** instead of project root

2. **Run the new build command**:
   ```bash
   auto build --target jet
   ```

3. **Verify output in `jet/` directory**:
   ```bash
   ls jet/
   ```

### For Multi-Backend Projects

1. **Update pac.at** with backend configuration:
   ```auto
   backend: {
       front: ["vue", "jet"]
       back: "rust"
   }
   ```

2. **Run build for all backends**:
   ```bash
   auto build
   ```

3. **Output will be in**:
   - `vue/` - Vue frontend
   - `jet/` - Jetpack frontend
   - `back/` - Rust backend

## Troubleshooting

### Q: My old `dist/` directory is still there

A: The old `dist/` directory is not automatically removed. You can safely delete it:
```bash
rm -rf dist/
```

### Q: Build command says "target not found"

A: Make sure you have the `backend` field in your `pac.at` file, or specify the target explicitly:
```bash
auto build --target vue
```

### Q: Can I still use `auto vue` command?

A: The `auto vue` and `auto jet` commands are deprecated but still work. They will be removed in a future version. Please migrate to `auto build` and `auto run`.

## Need Help?

If you encounter any issues with migration, please open an issue on GitHub with:
- Your `pac.at` file contents
- The error message you're seeing
- Your project structure
