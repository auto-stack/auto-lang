---
title: AutoOS virtual desktop
description: The current AutoOS desktop, applications, settings, and host integration.
---

<script setup>
import IntroductionFrame from '../.vitepress/theme/components/IntroductionFrame.vue'
import DesktopShowcase from '../.vitepress/theme/components/DesktopShowcase.vue'
import EvidenceImage from '../.vitepress/theme/components/EvidenceImage.vue'
</script>

<IntroductionFrame kind="desktop" />

<DesktopShowcase style="padding: 0" />

## Windows, workspaces, and launch

The virtual desktop is the current **OS over OS** form: AutoOS applications run inside a desktop hosted by the existing system, alongside its desktop. The host continues to provide the kernel, drivers, and system services.

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

Host applications and services are connected per platform. Shared UI definitions aim for consistent layout, interactions, and themes; platform adapters and text rendering can differ. Early Linux composition-host work exists, while complete independent distributions, OpenHarmony integration, and an AutoOS kernel remain future directions.

[Read the OS architecture and operating forms](/os) · [Read the AutoOS history and outlook](/articles/autoos-history) · [Explore desktop execution paths](/ui-desktop) · [Back to v0.5](/v05/)
