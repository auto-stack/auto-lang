# 16 - Shell & Agent Tools

## Overview
Plans covering the AutoShell cross-platform shell environment, its command enhancements, structured data pipelines, and AI agent capabilities. These plans build AutoLang's interactive tooling layer from basic shell commands to a full AI-powered coding agent.

## Plan Summary

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 017 | AutoShell Design | ✅ | Cross-platform shell with structured data pipelines, 155 tests, all 10 phases complete |
| 046 | AutoShell ls Flags | ✅ | Coreutils flags (-a, -l, -h, -t, -r, -R) for ls command |
| 047 | AutoShell Value Pipelines | ✅ | Zero-copy structured data pipelines using Auto Value system (176 tests) |
| 153 | AutoShell AI Agent Design | ⏳ | Multi-granularity AI agent with LLM providers, tools, MCP, and multi-agent coordination |
| 159 | AutoCode Coding Agent | ⏳ | AI-powered coding agent integrated into AutoShell |

## Status Summary
- Completed: 3 | Partial: 0 | Planned: 2 | Deprecated: 0

## Key Achievements
- AutoShell (Plan 017) fully implemented with REPL, pipelines, 20+ built-in commands, auto-completion, and history (155 tests)
- ls command (Plan 046) supports all 6 coreutils flags with cross-platform permission handling
- Value Pipelines (Plan 047) transformed AutoShell from string-based to Nushell-style structured data with `get`, `where`, `select` commands (176 tests)

## Remaining Work
- AI Agent (Plan 153) is a large undertaking with 8 phases covering LLM providers, tool system, MCP client, multi-agent coordination, and Process-granularity agents
- AutoCode (Plan 159) extends the agent concept into a full coding assistant
- AutoShell configuration system, I/O redirection, and job control remain as future enhancements
- Reedline Tab integration and history expansion activation are pending UI polish items
