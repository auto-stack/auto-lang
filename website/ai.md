---
title: 'AI: shared capabilities and work tools'
description: 'AI: shared capabilities and work tools'
---

<script setup>
import AIIntroduction from './.vitepress/theme/components/AIIntroduction.vue'
</script>

<AIIntroduction />

## Why a shared system layer

Applications using models face common questions: which provider to connect to, how to configure models and credentials, how to manage concurrency, and how to inspect usage. AutoAI puts these shared capabilities into a service so integrated applications can focus on their tasks.

This also allows AI capabilities to extend beyond one chat interface. Command shells, development tools, and future knowledge applications can reuse the infrastructure. Applications integrate the service incrementally according to their tasks and tools.

## Applications, Agents, and services

| Layer | Current responsibility |
|---|---|
| Applications and user interfaces | User tasks, results, progress, operations, and confirmation entry points |
| Agent execution | Roles, skills, tool calls, and workflows |
| Shared client | Normalized requests and responses for the model service |
| `aaid` model service | Configured provider access, concurrency, routing, and usage records |
| Configuration service | Application, role, skill, and model settings |

A typical path is application → Agent/client → aaid → configured model provider. Tool execution belongs to the relevant application or Agent tool layer; the model daemon focuses on model communication and resource coordination.

## Existing applications and integrations

- **AutoMusk** is a development work tool organizing chat, tool activity, and project materials such as Plans, Specs, and Wiki. Planning, execution, review, and knowledge consolidation are separate stages with human confirmation. Its implementation combines Auto sources, generated code, and Rust infrastructure.
- **AutoAI CLI and AutoShell** provide terminal Agent and AI interaction entry points. AutoAI CLI prefers AutoShell for command execution, with explicit handling of failures and policy denials.
- **The configuration center** manages roles, skills, applications, and model settings through a shared configuration entry point.

Capabilities depend on each integration’s tools and workflows, with the corresponding human collaboration and confirmation steps.

## Current capabilities and boundaries

| Area | Capabilities and boundaries |
|---|---|
| Model access | Requests to configured providers with implemented adapters; model availability depends on configuration, credentials, and provider services |
| Request coordination | Shared concurrency management, model routing, and usage records; routing follows configuration and candidate rules, without guaranteeing the “best model” |
| Agent tools | Roles, skills, tool calls, and workflows exist; capabilities depend on application integration and tool configuration |
| System integration | Development tools, shell, and configuration integrations exist; knowledge coordination across applications and a broader personal work environment remain under development |

The model gateway and tools for command execution or file operations are separate layers. Interfaces should make tasks, status, output, verification results, and required confirmation understandable.

## From development tools to a knowledge and work environment

The long-term AI + Lang + OS direction connects language, services, and applications: describe data and operations with the language, connect capabilities through services, and preserve knowledge, tasks, and outcomes in applications.

This remains a long-term product direction. Existing tools provide part of the foundation. A unified personal knowledge system, lasting collaboration across applications, and broader work and everyday-life scenarios still require explicit data, permission, and interaction designs, followed by implementation.

[Read the AutoOS overview](/os) · [Read the AI ecosystem history](/articles/auto-ai-history)
