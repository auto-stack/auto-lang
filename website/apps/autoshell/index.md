---
title: AutoShell — Commands, data pipelines, and AutoScript
description: Everyday shell commands, structured data, and multiline Auto scripts in one execution engine.
outline: [2, 3]
editLink: false
---

# AutoShell: commands, data pipelines, and AutoScript

AutoShell, invoked as `ash`, brings everyday commands, structured pipelines, and AutoScript into one execution engine. Keep an interactive session open, or use the CLI to execute a command or script file.

It connects exploration with repeatable work: inspect files or data, filter and transform fields, then turn useful operations into a script.

[All applications](/apps) · [Usage guide and recorded examples](/apps/autoshell/guide/)

<!-- Screenshot slot: ash-01 native colored ls table is the approved main capture. Main introduction remains image-free for this writing phase; existing captures are retained in guide. -->

## Keep working in one session

Directories, variables, and aliases persist between commands. Completion, history search, and inline suggestions reduce repeated typing. Pipelines, redirection, command chains, and background jobs help compose daily work.

For example, browse a directory with `ls`, filter relevant files, and export results to another program. Interactive use and `ash -c` share the engine, making explored commands reusable in automation.

## Pass structured records between commands

Some built-in commands return records with fields rather than only a text stream. File records from `ls` and parsed JSON can be filtered, sorted, projected, and converted.

```sh
cat users.json | from_json | .age > 30 | select .name .age
```

This reads JSON, filters by age, and retains names and ages. The [pipeline examples](/apps/autoshell/guide/#data-pipelines) provide data, copyable commands, and recorded output. External-command output and structured records are not always the same data kind; handle them according to the command.

## Extend a command into a script

F2 opens the multiline AutoScript editor for functions, conditions, and loops. Enter inserts a line; F5 executes and returns to the prompt. Ctrl+Enter depends on the terminal reporting its modifiers.

Capture query results in a script, then calculate, iterate, or summarize. Save a `.ash` file to repeat the operation through the CLI. The guide contains complete classification, amount-summary, and JSON-query examples.

<!-- Screenshot slot: F2 script input followed by its real output; existing recorded examples stay in guide. -->

## Commands, scripts, and the AI entry

| Entry | Purpose |
|---|---|
| F1 | Lock shell command mode |
| F2 | Edit and execute multiline AutoScript |
| F3 | Enter AI chat input; a working model service configuration is required |

F3 opens an AI session. An input-area screenshot does not prove an external model call completed. [AutoMusk](/apps/automusk/) organizes more complex development tasks; [AutoAI](/ai) explains shared model services.

## Automation and execution scope

CLI options provide JSON output, read-only operation, execution previews, path and command restrictions, and audit records. Their coverage differs: policy applies to file operations managed by ash, while external programs require OS isolation for their own file access.

Current agent integration can use command calls, JSON results, and execution policy. Some dedicated Agent subcommands in older documentation are not implemented and are not current entry points.

## Getting started and current boundaries

Building requires Rust and matching repository dependencies. Add the built `ash` to PATH, then use:

```sh
ash
ash --json -c 'ls'
ash user-report.ash
```

Recorded runs cover the Windows CLI and interactive shell on 2026-09-29/30, using ash v0.1.0. Other platforms and standalone GUI/TUI coverage need separate confirmation. AutoLang's website release v0.5 and the ash executable version are identified separately.

The [usage guide](/apps/autoshell/guide/#quick-start) contains build instructions, current query syntax, script downloads, and observed limitations such as CSV output. This introduction was organized on **2026-10-01**.

[Real terminal and operation examples](/apps/autoshell/guide/#interface-overview) · [AutoOS](/os) · [Application overview](/apps)
