---
title: AutoMusk — A coding agent built around Plans and Specs
description: Conversations, tools, development Plans, review, project knowledge, and current automation boundaries.
outline: [2, 3]
editLink: false
---

# AutoMusk: a coding agent built around Plans and Specs

AutoMusk brings project conversations, tool use, and development activity into one workspace. A Plan organizes one change; Specs preserve the project's existing knowledge and constraints.

The practical goal is continuity: requirements, decisions, code changes, and verification results should remain accessible after many rounds of work, providing a reliable starting point for the next task.

[All applications](/apps) · [AutoAI architecture](/ai)

<!-- Screenshot slot: overview from the agreed sample-project walkthrough, after AutoMusk preparation is complete. -->

## Start a conversation in a project

Choose a working directory, ask the agent to read and explain the project, then discuss a specific change. Messages, tool events, Plan progress, confirmation gates, and reports make activity visible. People inspect the results and remain involved where decisions are needed.

A representative walkthrough follows one small project: understand it, request a change, check the Plan, execute, and inspect review and results. This introduction explains that sequence; captures will follow it when the application is ready.

## What Plans and Specs record

| Artifact | Purpose |
|---|---|
| Plan | One change's goals, design, tasks, acceptance criteria, progress, and review |
| Module Specs | Agreed current behavior, interfaces, and design knowledge |
| Spec Ledger | Derived indexes, relationships, and history; it does not replace module specifications |
| Conversations and reports | Execution, tool activity, and matters requiring human attention |

A completed Plan should satisfy its acceptance criteria. Knowledge consolidation updates specifications to reflect delivered work. Neither is established simply by a run finishing.

## Planning, execution, review, and consolidation

The four-stage method forms a reviewable Plan, implements the change, checks actual code and test results, and consolidates knowledge. The built-in Relay `plan` flow currently runs serially using `plan-dev`, with a human confirmation gate before execution.

Repository development skills and the application's Relay flow are separate entry points. The skills' worktree, review, landing, and consolidation rules are not automatically enforced by every application run. Failed review currently requires follow-up repair and another review. Restarting the service does not resume active Relay runs held in memory.

<!-- Screenshot slot: the same sample project's Plan, confirmation gate, tool activity, and reviewed result. -->

## Models, tools, and application responsibilities

AutoMusk owns development tasks, project artifacts, local tools, API, and UI. AutoAI supplies the common agent loop, roles, skills, and orchestration capabilities. Clients connect model requests to aaid, which handles providers, concurrency, and usage. File and command operations still execute in the application or agent tool layer.

Models, roles, and skills are configurable. Available models depend on configured providers, account access, and the runtime environment. The ecosystem offers shared settings; the [AI introduction](/ai) explains the model-service relationships.

## Current implementation and prerequisites

Auto sources generate the primary Vue frontend. The default HTTP backend combines Rust infrastructure and Rust generated from Auto. AutoVM backend and native UI paths also exist; this does not make the entire application fully VM-native or exclusively implemented in Auto.

Usage requires matching Auto and Rust toolchains, Node.js/pnpm, repository dependencies, and configured aaid access for model-backed tasks. Common CLI entry points include:

```sh
musk chat
musk run "Read the current project and summarize its structure"
musk serve
```

Start CLI tasks from the target project directory. Prepare the application and frontend according to the version's build instructions before serving, and set the intended working directory. The website Playground runs language examples; it is not a hosted AutoMusk service.

The application and its demonstration workflow remain in development. Durable runs, restart recovery, automatic repair loops, and full runtime-contract alignment are future work.

This introduction reflects current README and architecture material as of **2026-10-01**.

[AI history and outlook](/articles/auto-ai-history) · [AutoEdit](/apps/autoedit/) · [Application overview](/apps)
