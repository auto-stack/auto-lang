<p align="center"><img src="docs/icon.png" alt="Auto logo" width="96"></p>

# Auto · AI × Lang × OS

**From instant scripts to interfaces, desktops, and applications.**

[中文](README.cn.md) · [Language guide](docs/language/overview.md) · [v0.5 notes](docs/releases/v0.5.md) · [Website sources](website/)

Auto is a language and application ecosystem built around **AutoLang**. Run scripts immediately on
AutoVM and build programs through Rust transpilation. **AutoUI** brings shared interface definitions
to the web and native desktop; **AutoOS** organizes the desktop and system applications; **AutoAI**
supplies shared model services and Agent tools.

AutoEdit, AutoShell, AutoMusk, and JadeEdit put these foundations to work in editing, command
automation, AI-assisted development, and knowledge management. This repository is the ecosystem's
entry point and maintains the language implementation, UI framework, and developer toolchain.

[![AutoOS dark desktop with app icons, taskbar, and clock, Todo and music widgets](website/public/desktop-showcase/02-desktop-dark.png)](website/public/desktop-showcase/02-desktop-dark.png)

*AutoOS native desktop, captured on 2026-10-01. The widgets belong to apps that have been started
and minimized. This original image is shared with the website; click to view it at full size.*

## The ecosystem

| Project | Role | Explore |
|---|---|---|
| **AutoLang** | A dynamic-meets-static language: AutoVM for scripting, Rust for the main static shipping path, and bridges to Rust, Python, and C | [Language overview](docs/language/overview.md) · [Compiler and runtime](crates/auto-lang/) |
| **AutoUI** | State, events, and views in Auto; Vue for web interfaces and Rust/iced for native desktop, with hot reload, DevTools, and MCP inspection | [UI introduction](website/ui/index.md) · [Examples](examples/ui/README.md) |
| **AutoOS** | A hosted desktop with windows, workspaces, Launcher, settings, and system applications | [Desktop introduction](website/autoos/index.md) · [Source](https://github.com/auto-stack/auto-os) |
| **AutoAI** | Shared model clients and the `aaid` service, plus Agent roles, skills, and tools used by applications | [AI introduction](website/ai.md) · [Source](https://github.com/auto-stack/auto-ai) |

The working idea is **script now, ship as Rust**: keep the iteration loop short, then use the Rust
ecosystem for native builds. AutoUI extends that workflow to interfaces; AutoOS gives applications
a shared desktop; AutoAI connects applications to model services and Agent workflows.

AutoOS currently runs within a host operating system. **Language as OS (LaOS)** is the implementation
direction, and **OS over OS** describes that current hosted form. Independently deployable systems
and deeper AI-assisted knowledge work remain longer-term directions. See the [OS introduction](website/os.md).

## Applications

| Application | What it is for | Learn more / source |
|---|---|---|
| **AutoEdit** | Browse projects, edit code and text, search files, and compare files, directories, or editing buffers | [Introduction](website/apps/autoedit/index.md) · [auto-edit](https://github.com/auto-stack/auto-edit) |
| **AutoShell** | An interactive command session, structured file/JSON pipelines, and multiline AutoScript; CLI and GUI forms | [Introduction](website/apps/autoshell/index.md) · [Usage guide](website/apps/autoshell/guide/index.md) · [auto-shell](https://github.com/auto-stack/auto-shell) |
| **AutoMusk** | A coding-agent workspace connecting project conversations, tools, Plans, Specs, execution, and review | [Introduction](website/apps/automusk/index.md) · [auto-musk](https://github.com/auto-stack/auto-musk) |
| **JadeEdit** | AutoDown documents and local knowledge bases: editing, page links, backlinks, search, and organization | [Introduction](website/apps/jadeedit/index.md) · [jade-edit](https://github.com/auto-stack/jade-edit) |

**AutoDown** supplies the document format and editing foundations. [AutoDown and Jade Garden](website/apps/autodown/index.md)
introduces the related document and knowledge workspace; [auto-down](https://github.com/auto-stack/auto-down)
is its source repository.

System applications and demos include file management, a terminal, settings, system monitoring,
notes, calendars, charts, media tools, and games. Browse the [application catalog](website/apps.md)
and [UI example projects](examples/ui/README.md). These applications combine Auto and Rust foundations;
their features and readiness are described individually.

### AutoEdit: code and text

[![AutoEdit dark native workspace with a project tree and an Auto source file](website/public/apps/autoedit/overview-dark.png)](website/public/apps/autoedit/overview-dark.png)

*Native AutoEdit workspace. User-provided capture from 2026-10-01, shared with the website.*

### AutoShell: commands and structured data

[![Native ash terminal showing a colored ls file table and a field-based pipeline](website/public/apps/autoshell/ash-01.png)](website/public/apps/autoshell/ash-01.png)

*Original `ash` terminal capture used by the website's AutoShell preview and guide, from the
September 2026 capture set. The full session shows both structured output and AutoScript.*

## The v0.5 milestone

[v0.5](docs/releases/v0.5.md) brings the language, interfaces, desktop, and applications together:

- **Language and services:** module loading, Actor state and scheduling, lazy generators, streaming
  IO, and HTTP APIs, alongside the main Rust transpilation path.
- **Ecosystem bridges:** Rust natives and `dep`/`use.rs` shims; CPython through `use.py`; `.as` scripting
  and Python transpilation, with real-library and PyTorch parity cases.
- **Interfaces and desktop:** Vue and iced, themes, editors, charts and media integrations;
  hosted desktop windows, Launcher, settings, and system apps.
- **Developer tools:** LSP, browser Playground, debugging, F12 DevTools, MCP tree/layout/screenshot
  inspection, and shared build caches.
- **Experimental self-hosting:** Auto-written VM and transpiler implementations close a verified
  Auto → Rust → Rust compiler loop. The Rust implementation remains the reference toolchain.

The release notes describe a rolling milestone, with the audited history through 2026-09-30;
release-candidate freezing is tracked separately. Backend coverage and application maturity vary.
Web and native desktop are the main UI paths; Android/HarmonyOS are feasibility/demo paths,
and broader mobile, MCU, and independent-OS support remain roadmap work.

## Start with a script

With an `auto` CLI available, run the existing [Hello World example](docs/tour/ch01-hello/01_hello.at):

```sh
auto docs/tour/ch01-hello/01_hello.at
```

```auto
fn main() {
    print("Hello, World!")
}
```

To inspect generated Rust from the same source:

```sh
auto trans --path docs/tour/ch01-hello/01_hello.at rust
```

For an interface, the existing counter project has both web and native development paths:

```sh
auto run examples/ui/002-counter -r vue
auto run examples/ui/002-counter -r vm
```

Vue development also needs the project's Node.js/package-manager dependencies. Native UI uses iced;
media examples have additional runtime requirements. See the [counter guide](examples/ui/002-counter/README.md).

**Building from source:** this workspace currently has a sibling `auto-down` path dependency.
Clone [auto-lang](https://github.com/auto-stack/auto-lang) and [auto-down](https://github.com/auto-stack/auto-down)
as adjacent directories, then build `cargo build -p auto --release` from `auto-lang`.
The default CLI includes Python and AutoDown integration: prepare the matching CPython development
and native build environment. The binary is `target/release/auto` (`auto.exe` on Windows).
See the [language guide](docs/language/overview.md#running-and-building) for the feature choices.

## Learn and explore

| Goal | Start here |
|---|---|
| Learn the current language | [Language overview](docs/language/overview.md) · [Hands-on Tour](docs/tour/ch01-hello.md) |
| Develop a script and ship it | [From Script to Ship](docs/script-to-ship/README.md) · [Parity examples](parity/README.md) |
| Build an interface | [AutoUI](website/ui/index.md) · [UI projects](examples/ui/README.md) · [Blueprints](blueprints/README.md) |
| Explore the desktop and apps | [AutoOS](website/autoos/index.md) · [Application catalog](website/apps.md) |
| Understand shared AI services | [AutoAI](website/ai.md) · [AutoMusk](website/apps/automusk/index.md) |
| Explore browser examples | [Playground page](website/playground.md) · [Run the website locally](website/README.md#development) |
| Read the books | [Auto and companion books](https://github.com/auto-stack/books) |

Website links here point to the authored pages in this repository. The [website](website/) builds
those introductions and the shared screenshots into a bilingual documentation site.

## Repository and contributions

- [`crates/`](crates/): language, AutoVM, transpilers, UI, CLI, LSP, package/build tools, and Playground.
- [`stdlib/`](stdlib/), [`schema/`](schema/), [`packages/`](packages/), [`blueprints/`](blueprints/): standard libraries, UI contracts, components, and reusable UI blueprints.
- [`examples/`](examples/) and [`parity/`](parity/): runnable projects and behavior comparisons.
- [`docs/`](docs/) and [`website/`](website/): learning material, design knowledge, release notes, and site sources.
- [`auto/`](auto/): experimental self-hosted toolchain.

For development, follow [AGENTS.md](AGENTS.md) and the [Plan/Spec workflow](docs/specs/README.md).
Work in an isolated worktree and use the checks appropriate to the change.

[MIT license](LICENSE).
