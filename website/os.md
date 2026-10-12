---
title: 'AutoOS: architecture and current progress'
description: 'AutoOS: architecture and current progress'
---

<script setup>
import OSIntroduction from './.vitepress/theme/components/OSIntroduction.vue'
import DesktopShowcase from './.vitepress/theme/components/DesktopShowcase.vue'
import EvidenceImage from './.vitepress/theme/components/EvidenceImage.vue'
import IntroductionRelation from './.vitepress/theme/components/IntroductionRelation.vue'
</script>

<OSIntroduction />

## The current virtual desktop

<DesktopShowcase embedded />

## Windows, workspaces, and launch

AutoUI applications can run directly on Windows/Linux native desktops or inside an AutoOS virtual desktop hosted on Windows/Linux. The virtual desktop is the current **OS over OS** form: AutoOS applications run inside a desktop hosted by the existing system, alongside its desktop. The host continues to provide the kernel, drivers, and system services.

The desktop shell itself uses AutoUI. It organizes virtual windows, focus, dragging, resizing, workspaces, the taskbar, launcher, and notifications. Application windows and desktop surfaces share window semantics; rendering and host integration have separate responsibilities.

The captures above move from the desktop to Launcher, games, and a working arrangement. Widgets come from applications that have been started and minimized. Static captures show their demonstration state, rather than certifying every platform or application behavior.

## Applications and work tools

`auto-os` organizes the desktop surfaces and application integration. Its launch manifest declares the current application paths, primarily VM execution. Compiled shell and native application generation are separate paths with their own integration requirements.

The gallery shows AutoEdit, Todo, Calendar, and three games. Separate projects also supply tools such as [AutoShell](/apps/autoshell/), [AutoMusk](/apps/automusk/), and Kanban. Integration and maturity vary by application; a source example is not automatically a fully integrated system application.

Kanban illustrates a data-driven board with Web and desktop views. The following existing project captures document those interfaces; they are not part of the October 1 desktop capture sequence.

<EvidenceImage src="/v05/kanban-web.png" alt="Kanban board in its Web interface" caption="Existing project capture: Kanban Web interface." zoom-label="View full size" close-label="Close" original-label="Open original" />

<EvidenceImage src="/v05/kanban-desktop.png" alt="Kanban board in its desktop interface" caption="Existing project capture: Kanban desktop interface." zoom-label="View full size" close-label="Close" original-label="Open original" />

## Shared settings

`auto-os-config` uses Auto sources to organize its settings UI for Web and desktop. Shared configuration includes applications, roles, skills, and models. The generic editor derives forms from supported data shapes; particular modules can still require their own rules and integration.

<EvidenceImage src="/v05/autoos-config-agents.png" alt="Settings Center showing Agent configuration" caption="Existing project capture: Agent settings." zoom-label="View full size" close-label="Close" original-label="Open original" />

<EvidenceImage src="/v05/autoos-config-skills.png" alt="Settings Center showing skill configuration" caption="Existing project capture: skill settings." zoom-label="View full size" close-label="Close" original-label="Open original" />

## Framework, product, and host

`auto-lang` owns the language, UI framework, execution, and rendering contracts. `auto-os` owns desktop surfaces, application assembly, and product integration. Underlying execution and system facilities currently combine Auto sources, generated code, Rust infrastructure, and existing ecosystems.

Host applications and services are connected per platform. Shared UI definitions aim for consistent layout, interactions, and themes; platform adapters and text rendering can differ. The Windows/Linux virtual desktop uses the existing host system; complete independent distributions, OpenHarmony integration, and an AutoOS kernel remain future directions.

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
