// PLAN-719: shared summaries and explicitly labelled architecture illustrations.
type IntroKind = 'os' | 'ai' | 'desktop' | 'uiDesktop'
interface IntroSection {
  title: string
  intro: string
  links: { label: string; href: string }[]
  layers: string[]
  diagramLabel: string
  diagramNote: string
}
export function introCopy(zh: boolean): { home: { os: string; ai: string }; sections: Record<IntroKind, IntroSection> } {
  const prefix = zh ? '/zh' : ''
  const link = (label: string, href: string) => ({ label, href: prefix + href })
  return zh ? {
    home: {
      os: 'AutoOS 的虚拟桌面、系统应用与平台适配；从 LaOS 的实现理念走向独立系统的长期路线。',
      ai: '共享模型服务与 Agent 工具，连接语言、应用和实际工作；跨应用知识协作仍在建设。',
    },
    sections: {
      os: {
        title: 'AutoOS：以 Auto 语言为基础的操作系统环境',
        intro: 'AutoOS 正在构建一套跨平台的系统环境，包含桌面、应用、配置、命令接口和系统服务。当前可见的形态是运行在宿主系统中的虚拟桌面；长期方向是让这些组件根据设备与系统环境组合起来，逐步形成可以独立部署的操作系统。',
        links: [link('了解当前桌面', '/autoos/'), link('阅读桌面架构', '/docs/design/autoui/virtual-desktop'), link('AutoOS 的历史与展望', '/articles/autoos-history')],
        layers: ['应用与桌面 · 窗口、工作区和任务', 'AutoUI · 组件与交互定义', '渲染/合成宿主 · 浏览器、原生渲染与平台接入', 'AutoLang 与 Rust 基础设施 · 执行与系统能力', '宿主系统 · 内核、驱动、文件与进程'],
        diagramLabel: '当前宿主桌面的组成关系示意',
        diagramNote: '架构关系示意。当前形态复用宿主系统；各层的具体实现和平台覆盖不同，自有内核尚未实现。',
      },
      ai: {
        title: 'AI：Auto 体系中的共享能力与工作工具',
        intro: 'AI 参与了 Auto 的开发，也正在成为应用可以接入的系统能力。当前建设集中在共享模型服务、Agent 执行与开发工具；长期方向是让 AI 与语言、桌面和应用共同服务于人的知识管理和工作生活。',
        links: [link('了解 AutoMusk', '/apps/automusk/'), link('了解 AutoShell', '/apps/autoshell/'), link('AI 生态的演进', '/articles/auto-ai-history')],
        layers: ['应用与用户界面 · 任务、结果和确认', 'Agent 与工具 · 角色、技能和操作', '共享客户端 · 请求与响应', 'aaid 模型服务 · 接入、并发、路由与用量', '已配置的模型提供商'],
        diagramLabel: '应用、Agent 与模型服务的接入关系示意',
        diagramNote: '接入关系示意。工具在应用/Agent 层执行，模型服务负责模型通信与资源协调；配置服务提供应用、角色和模型设置。',
      },
      desktop: {
        title: '当前桌面：窗口、应用与系统入口',
        intro: 'AutoOS 在宿主系统中运行自己的桌面表面与应用，复用 AutoUI 的组件和交互定义。窗口、工作区、启动器、配置和应用共同组成当前的桌面体验。',
        links: [link('AutoOS 总览', '/os'), link('历史与展望', '/articles/autoos-history'), link('桌面架构', '/docs/design/autoui/virtual-desktop')],
        layers: [], diagramLabel: '', diagramNote: '',
      },
      uiDesktop: {
        title: 'AutoUI 的桌面运行路径',
        intro: '相同视图定义可用于 Vue 与原生桌面；运行、代码生成和平台适配有各自的支持范围。桌面表面由 AutoOS 产品仓组织，语言框架提供执行、渲染与系统接入。',
        links: [link('当前虚拟桌面', '/autoos/'), link('AutoUI 总览', '/ui'), link('桌面架构', '/docs/design/autoui/virtual-desktop')],
        layers: [], diagramLabel: '', diagramNote: '',
      },
    },
  } : {
    home: {
      os: 'The AutoOS virtual desktop, system applications, and platform integration; a long-term path from LaOS to independent systems.',
      ai: 'Shared model services and Agent tools connecting language, applications, and work; knowledge coordination across applications remains under development.',
    },
    sections: {
      os: {
        title: 'AutoOS: an operating environment built around Auto',
        intro: 'AutoOS is developing a cross-platform system environment spanning the desktop, applications, configuration, command interfaces, and services. Its current visible form is a virtual desktop running within a host system. The longer-term direction is to assemble these components for different devices and system environments, including independently deployable operating systems.',
        links: [link('Explore the current desktop', '/autoos/'), link('Desktop architecture', '/docs/design/autoui/virtual-desktop'), link('AutoOS history and outlook', '/articles/autoos-history')],
        layers: ['Applications and desktop · windows, workspaces and tasks', 'AutoUI · components and interaction definitions', 'Rendering/composition hosts · browser, native rendering and platform integration', 'AutoLang and Rust infrastructure · execution and system capabilities', 'Host system · kernel, drivers, files and processes'],
        diagramLabel: 'Illustration of the current hosted desktop architecture',
        diagramNote: 'Architecture illustration. The current form reuses the host system. Implementations and platform coverage differ by layer; an AutoOS kernel has not been implemented.',
      },
      ai: {
        title: 'AI: shared capabilities and work tools in the Auto ecosystem',
        intro: 'AI has participated in Auto’s development and is becoming a system capability that applications can use. Current work centers on shared model services, Agent execution, and development tools. The longer-term direction brings AI, language, desktop, and applications together for human-centered knowledge management, work, and everyday life.',
        links: [link('Explore AutoMusk', '/apps/automusk/'), link('Explore AutoShell', '/apps/autoshell/'), link('AI ecosystem history', '/articles/auto-ai-history')],
        layers: ['Applications and interfaces · tasks, results and confirmation', 'Agents and tools · roles, skills and operations', 'Shared client · requests and responses', 'aaid model service · access, concurrency, routing and usage', 'Configured model providers'],
        diagramLabel: 'Illustration of application, Agent and model service integration',
        diagramNote: 'Integration illustration. Tools execute in the application/Agent layer. The model service manages model communication and shared resources; configuration supplies application, role and model settings.',
      },
      desktop: {
        title: 'The current desktop: windows, applications and system entry points',
        intro: 'AutoOS runs its desktop surfaces and applications within a host system, reusing AutoUI component and interaction definitions. Windows, workspaces, Launcher, configuration and apps form the current desktop experience.',
        links: [link('AutoOS overview', '/os'), link('History and outlook', '/articles/autoos-history'), link('Desktop architecture', '/docs/design/autoui/virtual-desktop')],
        layers: [], diagramLabel: '', diagramNote: '',
      },
      uiDesktop: {
        title: 'AutoUI desktop paths',
        intro: 'Shared view definitions support Vue and native desktop environments, with separate coverage for execution, code generation and platform integration. The AutoOS product repository organizes desktop surfaces; the language framework supplies execution, rendering and system integration.',
        links: [link('Current virtual desktop', '/autoos/'), link('AutoUI overview', '/ui'), link('Desktop architecture', '/docs/design/autoui/virtual-desktop')],
        layers: [], diagramLabel: '', diagramNote: '',
      },
    },
  }
}
