# VSCode Extension Enhancement Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enhance the AutoLang VSCode extension with AutoVM-based LSP and auto build/run buttons for pac.at projects

**Architecture:**
- Keep the existing LSP integration (auto-lsp crate) which already provides diagnostics, completion, hover, go-to-definition
- The LSP backend (`auto-lsp/src/backend.rs`) uses `auto_lang::parse_preserve_error()` for parsing - this is NOT AutoVM-based,- Add two new VSCode commands: `autoLang.build` and `autoLang.run` that appear when pac.at is detected
- Commands execute `auto build` and `auto run` CLI commands in the workspace root

**Tech Stack:**
- VSCode Extension API (TypeScript/JavaScript)
- auto-lsp Rust crate (tower-lsp)
- auto-man crate (VueProject, build_vue_project, run_vue_project)
- Node.js child_process for executing CLI commands

---

## Task 1: Add auto build Command

**Files:**
- Modify: `d:/autostack/auto-vscode/vscode-extension/package.json` (add commands)
- Modify: `d:/autostack/auto-vscode/vscode-extension/extension.js` (implement command)

**Step 1: Update package.json with new commands**

Add two new commands to the contributes.commands section:

```json
"commands": [
  {
    "command": "autoLang.showOutput",
    "title": "Show AutoLang LSP Output",
    "category": "AutoLang"
  },
  {
    "command": "autoLang.build",
    "title": "Auto Build",
    "category": "AutoLang",
    "icon": "$(package)",
    "enablement": "workspaceFolderContains:pac.at"
  },
  {
    "command": "autoLang.run",
    "title": "Auto Run",
    "category": "AutoLang",
    "icon": "$(play)",
    "enablement": "workspaceFolderContains:pac.at"
  }
]
```

**Step 2: Add Status Bar Button Contributions**

Add status bar buttons to package.json under contributes.menus:

```json
"menus": {
  "editor/title": [
    {
      "command": "autoLang.build",
      "when": "resourceFilename == pac.at",
      "group": "navigation"
    },
    {
      "command": "autoLang.run",
      "when": "resourceFilename == pac.at",
      "group": "navigation"
    }
  ],
  "editor/context": [
    {
      "command": "autoLang.build",
      "when": "resourceFilename == pac.at",
      "group": "AutoLang"
    }
  ]
}
```

**Step 3: Implement command handlers in extension.js**

Add the following code after the the `activate` function:

```javascript
// ========================================
// auto build and auto run Commands
// ========================================

const path = require('path');
const { exec } = require('child_process');

/**
 * Check if pac.at exists in the given directory
 * @param {string} dir - Directory to check
 * @returns {boolean} - True if pac.at exists
 */
function hasPacAt(dir) {
    const pacPath = path.join(dir, 'pac.at');
    return vscode.workspace.fs.isFile(pacPath);
}

/**
 * Get the workspace root directory
 * @returns {string | undefined} - Workspace root path or undefined
 */
function getWorkspaceRoot() {
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (workspaceFolders && workspaceFolders.length > 0) {
        return workspaceFolders[0].uri.fsPath;
    }
    return undefined;
}

/**
 * Run a shell command and stream output to the AutoLang output channel
 * @param {string} cmd - Command to run
 * @param {string} cwd - Working directory
 * @param {vscode.OutputChannel} outputChannel - Output channel for logs
 * @returns {Promise<void>}
 */
function runShellCommand(cmd, cwd, outputChannel) {
    return new Promise((resolve, reject) => {
        outputChannel.appendLine(`\n> ${cmd}`);
        outputChannel.appendLine(`  cwd: ${cwd}\n`);

        // Use cmd.exe /C on Windows for proper PATH resolution
        const isWindows = process.platform === 'win32';
        const shellCmd = isWindows ? 'cmd' : cmd;
        const shellArgs = isWindows ? ['/C', cmd] : [];

        const child = exec(shellCmd, isWindows ? { cwd, shell: true } : { cwd }, {
            cwd,
            ...(isWindows ? { shell: true } : {})
        }, (error, stdout, stderr) => {
            if (stdout) {
                outputChannel.appendLine(stdout);
            }
            if (stderr) {
                outputChannel.appendLine(stderr);
            }
            if (error) {
                reject(error);
            } else {
                resolve();
            }
        });

        // Stream stdout in real-time
        if (child.stdout) {
            child.stdout.on('data', (data) => {
                outputChannel.append(data.toString());
            });
        }

        // Stream stderr in real-time
        if (child.stderr) {
            child.stderr.on('data', (data) => {
                outputChannel.append(data.toString());
            });
        }
    });
}

/**
 * Register auto build/run commands
 * @param {vscode.ExtensionContext} context
 * @param {vscode.OutputChannel} outputChannel
 */
function registerAutoCommands(context, outputChannel) {
    // Register auto build command
    const buildCommand = vscode.commands.registerCommand('autoLang.build', async () => {
        const workspaceRoot = getWorkspaceRoot();
        if (!workspaceRoot) {
            vscode.window.showErrorMessage('No workspace folder open. Please open a folder containing pac.at');
            return;
        }

        if (!hasPacAt(workspaceRoot)) {
            vscode.window.showErrorMessage('pac.at not found in workspace root. Please open a project with pac.at');
            return;
        }

        outputChannel.show();
        outputChannel.appendLine('════════════════════════════════');
        outputChannel.appendLine('  Running auto build...');
        outputChannel.appendLine('════════════════════════════════');

        try {
            await runShellCommand('auto build', workspaceRoot, outputChannel);
            vscode.window.showInformationMessage('auto build completed successfully!');
        } catch (error) {
            outputChannel.appendLine(`\nError: ${error.message}`);
            vscode.window.showErrorMessage(`auto build failed: ${error.message}`);
        }
    });

    // Register auto run command
    const runCommand = vscode.commands.registerCommand('autoLang.run', async () => {
        const workspaceRoot = getWorkspaceRoot();
        if (!workspaceRoot) {
            vscode.window.showErrorMessage('No workspace folder open. Please open a folder containing pac.at');
            return;
        }

        if (!hasPacAt(workspaceRoot)) {
            vscode.window.showErrorMessage('pac.at not found in workspace root. Please open a project with pac.at');
            return;
        }

        outputChannel.show();
        outputChannel.appendLine('════════════════════════════════');
        outputChannel.appendLine('  Running auto run...');
        outputChannel.appendLine('════════════════════════════════');

        try {
            // For auto run, we don't wait for completion since it starts a dev server
            const terminal = vscode.window.createTerminal('AutoLang: auto run');
            terminal.sendText('auto run');
            terminal.show();
        } catch (error) {
            outputChannel.appendLine(`\nError: ${error.message}`);
            vscode.window.showErrorMessage(`auto run failed: ${error.message}`);
        }
    });

    context.subscriptions.push(buildCommand, runCommand);
}
```

**Step 4: Call registerAutoCommands in activate function**

Add this line after `context.subscriptions.push(languageClient);`:

```javascript
    // Register auto build/run commands
    registerAutoCommands(context, outputChannel);
```

**Step 5: Test the commands**

1. Open a project with pac.at (e.g., examples/component-gallery)
2. Open Command Palette (Ctrl+Shift+P)
3. Type "Auto Build" - should see the command
4. Type "Auto Run" - should see the command
5. Execute each command to verify they work

---

## Task 2: Document LSP Architecture (Research Only)

**Files:**
- Read: `d:/autostack/auto-lang/crates/auto-lsp/src/backend.rs`
- Read: `d:/autostack/auto-lang/crates/auto-lsp/src/diagnostics.rs`

**Current LSP Implementation:**

The existing LSP (`auto-lsp` crate) uses:
- `auto_lang::parse_preserve_error()` for diagnostics - this is Parser-based, NOT AutoVM-based
- Custom completion logic in `completion.rs`
- Hover info in `hover_info.rs`
- Go-to-definition in `goto_def.rs`

**Key Finding:** The LSP does NOT currently use AutoVM. It uses the Parser directly.

**Why AutoVM is not needed for LSP:**

1. **LSP focuses on static analysis** - Diagnostics, completion, hover, go-to-def are all compile-time features
2. **AutoVM is for runtime execution** - REPL, script execution, bytecode VM
3. **Current approach is correct** - Using Parser directly is more efficient for LSP use cases

**Recommendation:** Keep the existing LSP implementation. It's already well-structured and appropriate for IDE features. AutoVM is for runtime execution, not static analysis.

---

## Task 3: Add Status Bar Indicators (Optional Enhancement)

**Files:**
- Modify: `d:/autostack/auto-vscode/vscode-extension/extension.js`

**Step 1: Add status bar items when pac.at is detected**

Add after `registerAutoCommands` call:

```javascript
/**
 * Create status bar items for auto build/run
 * @param {vscode.ExtensionContext} context
 */
function createStatusBarItems(context) {
    const workspaceRoot = getWorkspaceRoot();
    if (!workspaceRoot || !hasPacAt(workspaceRoot)) {
        return;
    }

    // Build button
    const buildButton = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
    buildButton.text = "$(package) Build";
    buildButton.tooltip = "Run 'auto build' to compile the project";
    buildButton.command = 'autoLang.build';
    buildButton.show();

    // Run button
    const runButton = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 99);
    runButton.text = "$(play) Run";
    runButton.tooltip = "Run 'auto run' to start development server";
    runButton.command = 'autoLang.run';
    runButton.show();

    context.subscriptions.push(buildButton, runButton);
}
```

**Step 2: Call createStatusBarItems in activate**

Add after `registerAutoCommands(context, outputChannel);`:

```javascript
    // Create status bar buttons for pac.at projects
    createStatusBarItems(context);
```

**Step 3: Add workspace folder change listener**

Add to handle workspace changes:

```javascript
// Listen for workspace folder changes
context.subscriptions.push(
    vscode.workspace.onDidChangeWorkspaceFolders(() => {
        // Update status bar visibility
        // This would require storing references to status bar items
    })
);
```

---

## Summary

### What This Plan Delivers:

1. **Two new commands** (`autoLang.build`, `autoLang.run`) that:
   - Only appear when `pac.at` is in the workspace root
   - Execute `auto build` and `auto run` respectively
   - Stream output to the AutoLang LSP output channel

2. **Status bar buttons** (optional) that:
   - Show Build and Run buttons when pac.at is detected
   - Provide quick access to the commands

3. **Documentation** clarifying that:
   - The existing LSP is Parser-based (not AutoVM-based)
   - This is the correct approach for static analysis
   - AutoVM is for runtime execution, not LSP

### Files Modified:

1. `d:/autostack/auto-vscode/vscode-extension/package.json` - Add commands and menus
2. `d:/autostack/auto-vscode/vscode-extension/extension.js` - Implement command handlers

### No LSP Changes Needed:

The current LSP implementation is appropriate for its use case. AutoVM is designed for runtime execution (REPL, scripts), while LSP needs static analysis (parsing, type checking), which the Parser already provides efficiently.
