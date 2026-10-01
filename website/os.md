---
title: 'AutoOS: architecture and current progress'
description: 'AutoOS: architecture and current progress'
---

<script setup>
import OSIntroduction from './.vitepress/theme/components/OSIntroduction.vue'
import DesktopShowcase from './.vitepress/theme/components/DesktopShowcase.vue'
import IntroductionRelation from './.vitepress/theme/components/IntroductionRelation.vue'
</script>

<OSIntroduction />

<DesktopShowcase preview embedded />

## From Language as OS to OS over OS

**Language as OS (LaOS)** is the original implementation idea: use Auto to organize and implement system components, adapting the language and runtime to different software and hardware environments while maintaining consistent public interfaces and user experiences.

The scope includes UI libraries, compositors, system applications, configuration, launchers, monitoring, command shells, AI services, and the desktop. A kernel is also part of the long-term vision, but an AutoOS kernel has not been implemented. The current runtime and underlying infrastructure rely substantially on Rust and existing ecosystems. A self-implemented native language foundation may also be explored in the future.

**OS over OS** describes the same direction from the product's current operating model. AutoOS runs alongside the host desktop, with its own applications inside a virtual desktop and platform-specific integration with host applications and services. It is the current product form of LaOS, carrying forward the same architecture.

## How the system is organized

<IntroductionRelation kind="os" />

| Part | Responsibility |
|---|---|
| AutoLang and runtime | Language, execution, code generation, and access to system capabilities |
| AutoUI and rendering/composition hosts | Component definitions and interfaces for browser and native environments |
| Desktop and window management | Application windows, workspaces, taskbar, launch entry points, and notifications |
| Configuration and system services | Shared settings, resources, and capabilities for application coordination |
| System applications and work tools | Terminal, editing, monitoring, boards, and knowledge work |
| Platform adapters | Host window systems, processes, files, devices, and existing kernels |

`auto-lang` owns the language and framework implementation. `auto-os` organizes the desktop surfaces, system applications, and product integration. Configuration, shell, terminal, AI, and work tools live in their respective projects and connect through shared interfaces and integration rules.

Client/Daemon is one pattern for shared resources and services; it does not require every component to be a separate daemon. UI definitions, window semantics, and platform rendering/composition also have distinct boundaries.

## Operating forms

**A virtual desktop within a host system** is the main current form. It reuses the host kernel, drivers, and system capabilities, runs AutoOS applications in its own desktop, and coexists with the host desktop. Host interoperability is developed per platform, with support depending on the window system and integration method.

**An independent system using an existing kernel** is a future direction. On suitable devices, AutoOS could provide the desktop and system shell, connect to the Linux kernel and required services, and form an independent distribution. OpenHarmony integration is another long-term path. Early Linux composition-host work exists; these complete product forms have not yet been delivered.

**A complete system with its own kernel** is a longer-term research direction. No AutoOS kernel or independent OS based on it is currently available.

These forms share language, component, and interface design. Platform integration and deployment still require separate implementation work. One codebase adapting to different ecosystems is a continuing goal. Current support is described for implemented and verified environments.

## Current progress

| Area | Current state |
|---|---|
| UI and desktop | Vue and native desktop paths, virtual windows, workspaces, launcher, and desktop surfaces exist, with feature-specific limits |
| Applications and configuration | System applications and separate tools have an integration manifest; the settings UI uses Auto sources for Web and desktop forms |
| Shell and terminal | AutoShell provides structured cross-platform commands and scripting; AutoTerm supplies terminal infrastructure. Common commands use POSIX-style interfaces; compatibility is described per command |
| AI services | Shared model services, clients, and Agent application integrations exist; applications integrate AI incrementally as needed |
| Host interoperability | Native-window and clipboard integration exists for specific platforms; support is assessed per platform and feature |
| Independent systems and kernel | Linux composition-host work is at an early stage; complete distributions, an OpenHarmony-based independent system, and an AutoOS kernel remain future directions |

The consistency goal spans components, commands, service interfaces, notifications, launch, and communication. Coverage is implemented and verified incrementally. UI consistency focuses on layout, interaction, and themes; platform text rendering can still differ.

## Longer-term direction: AI + Lang + OS

AutoOS aims to become a human-centered environment for knowledge management, work, and everyday life. AI has participated deeply in Auto's development and is intended to become a shared service and a means of application coordination. AutoLang supplies the expressive and executable foundation, while the OS and applications carry knowledge, tasks, and work processes.

Existing editors, knowledge tools, boards, shells, and Agent applications provide different starting points. Connecting personal knowledge, tasks, and actions across applications remains work to be done. The aim is to help people understand, choose, and control the system's capabilities, without assuming that every task should be delegated to AI.
