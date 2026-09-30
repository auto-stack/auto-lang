<script setup lang="ts">
import { computed, nextTick, onUnmounted, ref } from 'vue'
import FeatureCard from './FeatureCard.vue'
import ShowcaseSection from './ShowcaseSection.vue'
import EvidenceImage from './EvidenceImage.vue'

const props = withDefaults(defineProps<{ lang?: 'zh' | 'en' }>(), { lang: 'en' })
const zh = computed(() => props.lang === 'zh')
const prefix = computed(() => zh.value ? '/zh' : '')
const repo = 'https://github.com/auto-stack/auto-shell'
const assets = '/apps/autoshell/'
const selected = ref(0)
const tabList = ref<HTMLDivElement>()
const copied = ref('')
let copyTimer: ReturnType<typeof setTimeout> | undefined
onUnmounted(() => clearTimeout(copyTimer))

const t = computed(() => zh.value ? {
  badge: 'AutoOS 应用 · Beta', title: '结构化 Shell',
  intro: '熟悉的日常命令，按字段处理的数据管道，还有可直接编辑运行的 Auto 脚本。交互式会话和 CLI 共用同一套执行引擎。',
  start: '开始使用', source: '查看源码', meta: 'ash v0.1.0 · Windows 实测 · 2026-09-29/30',
  terminal: 'ash · 示例数据 users.json', zoom: '放大查看', close: '关闭', original: '打开原图', copy: '复制代码', copied: '已复制', copyFailed: '请选中代码复制',
  nav: ['界面与模式', '日常 Shell', '数据管道', '多行脚本', '自动化', '开始使用'],
  facts: [
    { icon: '⌨️', title: '持续的交互会话', description: '目录、变量和别名在会话中延续；补全、历史和行内建议帮助你少打字。' },
    { icon: '↗', title: '按字段处理数据', description: '查询文件和 JSON，筛选、排序、投影；用 --json 将结果交给其他程序。' },
    { icon: '▤', title: 'F2 编辑，F5 运行', description: '把一行命令扩展成函数、循环和条件；在终端里完成多行 AutoScript。' },
  ],
  shellTitle: '像日常 Shell 一样连续工作', shellDesc: '进入 ash 后，命令运行完会回到提示符。你可以在同一会话里切换目录、保留变量、加载脚本和管理后台作业。',
  shellItems: ['cd、pwd 与 shell 变量：跨命令保持会话状态。', 'alias 与 abbr：把常用命令变成容易输入的简称。', '管道、重定向、&& 命令链和多行续写：组合日常工作。', 'jobs：查询后台任务；source：把配置或脚本加载进当前会话。'],
  shellCaption: '补全、历史与行内建议的实际交互记录。图片按 Windows PTY 输入与输出排版重绘。',
  shortcutTitle: '常用交互键', keys: ['命令模式锁', '多行脚本编辑器', 'AI 对话入口', '补全', '历史搜索', '接受行内建议', '编辑当前行'],
  aiNote: 'F3 需要模型服务配置；本次展示没有调用外部模型。F4 已退役。',
  pipeTitle: '命令之间，可以直接传记录', pipeDesc: 'ls 返回文件记录，from_json 解析 JSON。筛选与排序访问字段，不需要先拆分文本列。',
  pipeItems: ['按 .name、.size、.type 等字段筛选和排序。', 'select 选择字段，first N 取前 N 条结构化记录。', '查询结果可继续捕获为 AutoScript 变量，参与计算和循环。'],
  pipeCaption: '文件列表排序、字段选择与 JSON 查询的实跑结果；点击可以查看完整命令。',
  pipeCodeTitle: '当前可用的字段查询写法', pipeHint: '把 users.json 保存到当前目录即可运行下面的查询。', downloadData: '下载 users.json',
  scriptTitle: '从一行命令，走到一段脚本', scriptDesc: 'F2 打开多行编辑器，Enter 换行，F5 或 Ctrl+Enter 运行。每次运行结束，回到普通提示符。',
  scriptStep: '输入函数、条件与循环', scriptOutput: '实际输出', scriptDownload: '下载这段脚本',
  scriptNote: 'F5 是终端可稳定发送的运行键；Ctrl+Enter 的识别取决于终端是否传递修饰键。三张图均来自 F2 编辑器中通过 F5 的实际运行，并按 PTY 记录重绘。',
  scriptLabels: ['函数与条件', '循环与汇总', 'JSON 查询与循环'],
  scriptDescriptions: ['定义分类函数，对数组中的每个分数调用函数。', '累计金额，并统计达到阈值的项目数量。', '将 shell 查询结果捕获成记录数组，再按年龄条件筛选并计数。'],
  scriptCaption: 'F2 → 输入多行脚本 → F5 → 输出与返回提示符。', jsonPathNote: '可下载示例使用同目录的 users.json；原实跑图保留了演示目录的文件路径。',
  automationTitle: '交互探索，也能放进自动化', automationDesc: '交互会话里的命令可以用 ash -c 执行；多行代码可以保存为 .ash 文件。CLI 启动参数决定 JSON 输出和执行策略。',
  policies: [
    { icon: '◉', title: '先看执行意图', description: '--dry-run 展示待执行操作；--read-only 拒绝写操作。' },
    { icon: '▣', title: '限定执行范围', description: '--sandbox 限制内置文件命令与重定向路径；--allow 和 --no-exec 控制命令执行。' },
    { icon: '≡', title: '留下执行记录', description: '--audit 输出 JSONL 审计日志；--no-network 拦截相应网络命令。' },
  ],
  policyNote: '路径策略覆盖 ash 管理的文件操作。外部程序自身的文件访问需要操作系统层面的隔离；使用外部程序时，应结合命令限制。',
  statusTitle: '当前版本的使用边界',
  statusItems: ['Agent 集成目前可使用 -c、--json 和执行策略。旧文档中的 ash agent describe-tools/check/run 尚未实现。', '本次验证覆盖 Windows CLI 与交互式 Shell；其他平台和独立 GUI/TUI 的功能覆盖需要单独确认。', '已观察到 to_csv 导出部分数值字段为空；本次隔离配置中的 .ashrc 自动加载未确认，手动 source 已验证可用。'],
  quickTitle: '从一个会话开始', quickDesc: '将构建出的 ash 加入 PATH 后，选择适合你的入口。',
  launchLabels: ['交互式会话', '单条命令', '运行脚本'],
  launchHints: ['运行 ash，然后直接输入命令。', '用于脚本和其他程序的命令调用。', '将示例和 users.json 保存在同一目录。'],
  buildTitle: '从源码构建', buildNote: '构建需要 Rust 工具链，以及同级目录中的 auto-lang、auto-ai 和 auto-shell 仓库。请按源码仓库说明准备依赖；在 auto-shell/ash 中构建 CLI：',
  evidenceNote: '本页展示依据 2026-09-29/30 的源码构建与实跑记录。AutoLang 网站的 v0.5 发布号与 ash v0.1.0 的程序版本分别标注。',
  back: '全部应用', release: 'v0.5 发布专题',
} : {
  badge: 'AutoOS application · Beta', title: 'The structured shell',
  intro: 'Everyday commands, pipelines that work with fields, and Auto scripts you can edit and run in your terminal. The interactive session and CLI share the same execution engine.',
  start: 'Get started', source: 'View source', meta: 'ash v0.1.0 · Verified on Windows · 2026-09-29/30',
  terminal: 'ash · sample data in users.json', zoom: 'Enlarge image', close: 'Close', original: 'Open original', copy: 'Copy code', copied: 'Copied', copyFailed: 'Select the code to copy',
  nav: ['Interface & modes', 'Daily shell', 'Data pipelines', 'Multiline scripts', 'Automation', 'Get started'],
  facts: [
    { icon: '⌨️', title: 'A persistent session', description: 'Keep directory, variables and aliases between commands. Completion, history and inline suggestions reduce typing.' },
    { icon: '↗', title: 'Work with fields', description: 'Query files and JSON, filter, sort and select fields. Use --json to pass results to another program.' },
    { icon: '▤', title: 'F2 to edit, F5 to run', description: 'Extend a command into functions, loops and conditions. Write multiline AutoScript in your terminal.' },
  ],
  shellTitle: 'Keep working in one shell session', shellDesc: 'Each command returns to the prompt. Change directories, keep variables, load scripts and manage background jobs in the same session.',
  shellItems: ['cd, pwd and shell variables preserve session state between commands.', 'alias and abbr give frequently used commands short names.', 'Pipelines, redirection, && chains and multiline continuation combine everyday tasks.', 'jobs inspects background work; source loads configuration or scripts into the session.'],
  shellCaption: 'Actual completion, history and inline suggestion interactions, redrawn from Windows PTY input and output.',
  shortcutTitle: 'Common interaction keys', keys: ['Lock command mode', 'Multiline script editor', 'AI chat entry', 'Completion', 'Search history', 'Accept inline suggestion', 'Edit the current line'],
  aiNote: 'F3 requires model service configuration. These demonstrations made no external model calls. F4 is retired.',
  pipeTitle: 'Pass records between commands', pipeDesc: 'ls returns file records; from_json parses JSON. Filter and sort by fields without splitting text columns.',
  pipeItems: ['Filter and sort fields such as .name, .size and .type.', 'select projects fields; first N takes the first N structured records.', 'Capture query results in AutoScript variables, then calculate and loop over them.'],
  pipeCaption: 'Observed file sorting, field selection and JSON queries. Enlarge the image to read the complete commands.',
  pipeCodeTitle: 'Field queries you can run today', pipeHint: 'Save users.json in the current directory to run the query below.', downloadData: 'Download users.json',
  scriptTitle: 'Turn a command into a script', scriptDesc: 'F2 opens the multiline editor. Enter inserts a newline; F5 or Ctrl+Enter runs the script. Each run returns to the normal prompt.',
  scriptStep: 'Write functions, conditions and loops', scriptOutput: 'Observed output', scriptDownload: 'Download this script',
  scriptNote: 'F5 has a reliable terminal key sequence. Ctrl+Enter depends on your terminal reporting its modifiers. All three images record actual F2 editor runs submitted with F5, redrawn from PTY output.',
  scriptLabels: ['Functions & conditions', 'Loops & totals', 'JSON & iteration'],
  scriptDescriptions: ['Define a classification function and call it for each score in an array.', 'Add up amounts and count the entries that reach a threshold.', 'Capture shell query results as records, then filter by age and count matches.'],
  scriptCaption: 'F2 → multiline input → F5 → output and return to the prompt.', jsonPathNote: 'The downloadable sample uses users.json in the same directory. The original run image retains its demonstration directory path.',
  automationTitle: 'Explore interactively, reuse in automation', automationDesc: 'Run session commands with ash -c, or save multiline code as a .ash file. CLI startup options control JSON output and execution policy.',
  policies: [
    { icon: '◉', title: 'Preview operations', description: '--dry-run describes intended operations; --read-only rejects writes.' },
    { icon: '▣', title: 'Limit execution', description: '--sandbox confines built-in file operations and redirection. --allow and --no-exec control command execution.' },
    { icon: '≡', title: 'Record activity', description: '--audit writes JSONL audit records; --no-network blocks the corresponding network commands.' },
  ],
  policyNote: 'Path policy applies to file access managed by ash. External programs need OS isolation for their own file access; combine policy with command restrictions.',
  statusTitle: 'Current version boundaries',
  statusItems: ['For agent integration, use -c, --json and execution policies. The old ash agent describe-tools/check/run commands are not implemented.', 'These runs cover the Windows CLI and interactive shell. Other platforms and standalone GUI/TUI coverage require separate verification.', 'to_csv was observed dropping some numeric values. Automatic .ashrc loading was not confirmed in the isolated profile; manual source worked.'],
  quickTitle: 'Start with a session', quickDesc: 'Add the built ash executable to PATH, then choose your entry point.',
  launchLabels: ['Interactive session', 'One command', 'A script file'],
  launchHints: ['Start ash and type commands at its prompt.', 'For command calls from scripts and other programs.', 'Save the script and users.json in the same directory.'],
  buildTitle: 'Build from source', buildNote: 'You need the Rust toolchain and sibling auto-lang, auto-ai and auto-shell checkouts. Follow the repository instructions for dependencies; build the CLI from auto-shell/ash:',
  evidenceNote: 'This page uses source builds and observed runs from 2026-09-29/30. The website release, AutoLang v0.5, and the executable version, ash v0.1.0, are identified separately.',
  back: 'All applications', release: 'v0.5 release highlights',
})

const examples = [
  {
    file: 'classify.ash', image: '18_f2-script-classify.png',
    code: 'fn bucket(n) {\n    if n >= 80 { return "high" }\n    return "low"\n}\n\nvar values = [95, 82, 61]\nfor v in values {\n    print(v.str() + ":" + bucket(v))\n}',
    output: '95:high\n82:high\n61:low',
  },
  {
    file: 'summary.ash', image: '19_f2-script-summary.png',
    code: 'var amounts = [120, 85, 210, 40]\nvar total = 0\nvar large = 0\n\nfor amount in amounts {\n    total = total + amount\n    if amount >= 100 {\n        large = large + 1\n    }\n}\n\nprint("items: " + amounts.len().str())\nprint("total: " + total.str())\nprint("amounts >= 100: " + large.str())',
    output: 'items: 4\ntotal: 455\namounts >= 100: 2',
  },
  {
    file: 'user-report.ash', image: '20_f2-script-json-capture.png',
    code: 'var users = > cat users.json | from_json\nvar adults = 0\n\nfor user in users {\n    if user.age >= 30 {\n        print(user.name + " (" + user.age.str() + ")")\n        adults = adults + 1\n    }\n}\n\nprint("matching users: " + adults.str())',
    output: 'Lin (35)\nNoah (42)\nmatching users: 2',
  },
]
const nativeShots = computed(() => zh.value ? [
  { image: 'ash-01.png', title: '文件列表与文档查看', description: 'ls 用彩色表格展示文件名、类型、大小和修改时间；管道筛选文件，show 展示文档内容。提示符同时给出当前目录与 Git 状态。', caption: '原生终端截图 ash-01。原图中的命令反映截图当时的版本。' },
  { image: 'ash-2.png', title: 'Shell 会话与磁盘用量', description: 'Shell 提示符下连续查看目录和磁盘用量，再用 where、select 处理字段，to_json 输出结果。它展示了普通终端操作与结构化数据处理的结合。', caption: '原生终端截图 ash-02（实际文件名 ash-2.png）；下方复制示例使用本次验证过的写法。' },
] : [
  { image: 'ash-01.png', title: 'File tables and document viewing', description: 'ls displays names, types, sizes and timestamps in a colored table. A pipeline filters files, and show displays a document. The prompt also shows the directory and Git status.', caption: 'Original interface capture ash-01, preserved as supplied. Its command syntax reflects the version captured.' },
  { image: 'ash-2.png', title: 'A shell session and disk usage', description: 'The Shell prompt keeps a sequence of directory and disk-usage commands. where and select work with fields, then to_json exports the result: terminal work and structured data in one session.', caption: 'Original ash-02 capture (actual filename ash-2.png). Copyable examples below use the syntax verified in these demonstrations.' },
])
const modeShots = computed(() => zh.value ? [
  { key: 'F1', image: '21_f1-command-mode.png', title: '锁定 Shell 命令模式', description: 'F1 将输入锁定为 Shell 命令，右侧显示 Shell，命令提示符变为蓝色。连续运行命令时保留该模式，再按 F1 可解除锁定。', caption: '实际按 F1 并执行 pwd 的记录重绘。' },
  { key: 'F2', image: '22_f2-editor-mode.png', title: '多行 AutoScript 编辑器', description: 'F2 打开带行号的编辑框。Enter 换行，F5 或 Ctrl+Enter 执行，Esc 取消；运行结束回到普通提示符。下面还有三个完整脚本实例。', caption: '实际打开 F2，输入并用 F5 运行；按 PTY 记录重绘。' },
  { key: 'F3', image: '23_f3-ai-entry.png', title: 'AI 对话入口', description: 'F3 打开 AI 会话，右侧显示 AI，输入提示符为 ?。它需要可用的模型服务；Esc 可离开并回到 Shell。', caption: '实际进入 F3 的输入区记录重绘；未提交问题或触发模型回复。' },
] : [
  { key: 'F1', image: '21_f1-command-mode.png', title: 'Lock shell command mode', description: 'F1 locks input to shell commands. The right prompt reads Shell and the command prompt turns blue. Commands retain this mode; press F1 again to unlock.', caption: 'Redrawn from an actual F1 switch and pwd command.' },
  { key: 'F2', image: '22_f2-editor-mode.png', title: 'Multiline AutoScript editor', description: 'F2 opens a numbered editor. Enter adds a line, F5 or Ctrl+Enter runs, and Esc cancels. Execution returns to the normal prompt. Three complete scripts follow below.', caption: 'Actual F2 input and F5 execution, redrawn from PTY output.' },
  { key: 'F3', image: '23_f3-ai-entry.png', title: 'AI chat entry', description: 'F3 enters an AI session. The right prompt reads AI and input uses ?. A model service is required; Esc returns to the shell.', caption: 'Actual F3 input area, redrawn from PTY output. No question was submitted or model reply requested.' },
])
const example = computed(() => examples[selected.value])
const heroCommand = 'cat users.json | from_json | .age > 30 | select .name .age'
const pipeline = 'ls | .type == "dir" | select .name | first 5\n' + heroCommand
const policies = "ash --dry-run -c 'touch output.txt'\nash --read-only -c 'touch output.txt'\nash --audit ash-audit.jsonl -c 'pwd'"
const build = 'cargo build --release -p ash\ncargo run --release -p ash'
const launches = ['ash', "ash --json -c 'ls | sort .size desc | select .name .size | first 5'", 'ash user-report.ash']
const sections = ['interface-overview', 'daily-shell', 'data-pipelines', 'scripts', 'automation', 'quick-start']
const keyNames = ['F1', 'F2', 'F3', 'Tab', 'Ctrl+R', 'Ctrl+F', 'Ctrl+O']
async function copyCode(code: string) {
  clearTimeout(copyTimer)
  try { await navigator.clipboard.writeText(code); copied.value = code }
  catch { copied.value = 'failed' }
  copyTimer = setTimeout(() => copied.value = '', 2500)
}
async function handleTabs(event: KeyboardEvent, index: number) {
  let next = index
  if (event.key === 'ArrowRight') next = (index + 1) % examples.length
  else if (event.key === 'ArrowLeft') next = (index + examples.length - 1) % examples.length
  else if (event.key === 'Home') next = 0
  else if (event.key === 'End') next = examples.length - 1
  else return
  event.preventDefault()
  selected.value = next
  await nextTick()
  tabList.value?.querySelectorAll<HTMLButtonElement>('[role="tab"]')[next]?.focus()
}
</script>

<template>
  <div class="landing-page autoshell-page">
    <div class="landing-hero autoshell-hero">
      <div class="hero-grid">
        <div class="hero-copy">
          <a class="hero-back" :href="prefix + '/apps'">← {{ t.back }}</a>
          <div class="badge">{{ t.badge }}</div>
          <h1 class="title">AutoShell<br /><span class="accent">{{ t.title }}</span></h1>
          <p class="description">{{ t.intro }}</p>
          <div class="actions">
            <a href="#quick-start" class="btn btn-primary">{{ t.start }} <span aria-hidden="true">↗</span></a>
            <a :href="repo" class="btn btn-secondary">{{ t.source }}</a>
          </div>
          <p class="hero-meta">{{ t.meta }}</p>
        </div>
        <div class="hero-terminal code-window">
          <div class="code-header">
            <div class="code-dots" aria-hidden="true"><span></span><span></span><span></span></div>
            <span class="code-title">{{ t.terminal }}</span>
          </div>
          <pre class="code-body" tabindex="0"><code><span class="prompt">❯ </span>{{ heroCommand }}

<span class="output">name   age
Lin    35
Noah   42</span>

<span class="prompt">❯ </span><span class="comment">F2 → AutoScript → F5</span></code></pre>
          <div class="terminal-tags"><span>Shell</span><span>JSON</span><span>AutoScript</span></div>
        </div>
      </div>
    </div>

    <nav class="section-nav" :aria-label="zh ? '本页导航' : 'On this page'">
      <a v-for="(label, i) in t.nav" :key="sections[i]" :href="'#' + sections[i]">{{ label }}</a>
    </nav>

    <div class="features-section overview-section">
      <div class="features-grid">
        <FeatureCard v-for="fact in t.facts" :key="fact.title" v-bind="fact" color="color-mix(in srgb, var(--page-accent-1) 12%, transparent)" />
      </div>
    </div>

    <section id="interface-overview" class="content-section interface-section" aria-labelledby="ash-interface-title">
      <div class="section-heading">
        <span class="section-eyebrow">Interface</span>
        <h2 id="ash-interface-title">{{ zh ? '真实终端界面' : 'The terminal interface' }}</h2>
        <p>{{ zh ? '从最初的两张界面截图认识 AutoShell：熟悉的提示符、可读的表格和连续的命令记录。' : 'Two original captures show the prompt, readable tables and a continuous record of commands.' }}</p>
      </div>
      <div class="native-gallery">
        <article v-for="shot in nativeShots" :key="shot.image" class="interface-example">
          <h3>{{ shot.title }}</h3><p>{{ shot.description }}</p>
          <EvidenceImage :src="assets + shot.image" :alt="shot.title" :caption="shot.caption" :zoom-label="t.zoom" :close-label="t.close" :original-label="t.original" />
        </article>
      </div>
    </section>

    <section class="content-section modes-section" aria-labelledby="ash-modes-title">
      <div class="section-heading">
        <span class="section-eyebrow">F1 / F2 / F3</span>
        <h2 id="ash-modes-title">{{ zh ? '三种常见交互形式' : 'Three common interaction forms' }}</h2>
        <p>{{ zh ? '用快捷键在命令、脚本编辑与 AI 对话之间切换。每种形式都保留独立图示，点击可放大查看。' : 'Switch between commands, script editing and AI chat. Each form has a separate image you can enlarge.' }}</p>
      </div>
      <div class="mode-gallery">
        <article v-for="shot in modeShots" :key="shot.key" class="mode-example">
          <div class="mode-copy"><h3><kbd>{{ shot.key }}</kbd> {{ shot.title }}</h3><p>{{ shot.description }}</p></div>
          <EvidenceImage :src="assets + shot.image" :alt="shot.key + ' / ' + shot.title" :caption="shot.caption" :zoom-label="t.zoom" :close-label="t.close" :original-label="t.original" />
        </article>
      </div>
      <p class="small-note">{{ zh ? '上方两张为原生终端截图；本区三张按 2026-09-30 的实际 PTY 交互记录排版重绘。F3 只展示输入区，历史内容未纳入图片。' : 'The two captures above are original interface screenshots. These three images are redrawn from actual PTY interactions on 2026-09-30. F3 shows only its input area, excluding conversation history.' }}</p>
    </section>

    <section id="daily-shell" class="content-section">
      <ShowcaseSection :title="t.shellTitle" :description="t.shellDesc" badge="01 / Shell">
        <ul><li v-for="item in t.shellItems" :key="item">{{ item }}</li></ul>
        <template #visual>
          <EvidenceImage :src="assets + '12_shell-editing-history.png'" :alt="t.shortcutTitle" :caption="t.shellCaption" :zoom-label="t.zoom" :close-label="t.close" :original-label="t.original" />
        </template>
      </ShowcaseSection>
      <div class="shortcut-panel">
        <h3>{{ t.shortcutTitle }}</h3>
        <div class="shortcut-grid"><div v-for="(key, i) in keyNames" :key="key"><kbd>{{ key }}</kbd><span>{{ t.keys[i] }}</span></div></div>
        <p class="small-note">{{ t.aiNote }}</p>
      </div>
    </section>

    <section id="data-pipelines" class="content-section">
      <ShowcaseSection :title="t.pipeTitle" :description="t.pipeDesc" badge="02 / Pipeline" reverse>
        <ul><li v-for="item in t.pipeItems" :key="item">{{ item }}</li></ul>
        <template #visual>
          <EvidenceImage :src="assets + '02_structured-pipeline.png'" :alt="t.pipeTitle" :caption="t.pipeCaption" :zoom-label="t.zoom" :close-label="t.close" :original-label="t.original" />
        </template>
      </ShowcaseSection>
      <div class="code-window query-code">
        <div class="code-header"><span class="code-title">{{ t.pipeCodeTitle }}</span><button class="copy-button" type="button" @click="copyCode(pipeline)">{{ copied === pipeline ? t.copied : t.copy }}</button></div>
        <pre class="code-body" tabindex="0"><code>{{ pipeline }}</code></pre>
      </div>
      <p class="small-note">{{ t.pipeHint }} <a :href="assets + 'examples/users.json'" download>{{ t.downloadData }} ↗</a></p>
    </section>

    <section id="scripts" class="content-section scripts-section">
      <div class="section-heading"><span class="section-eyebrow">03 / AutoScript</span><h2>{{ t.scriptTitle }}</h2><p>{{ t.scriptDesc }}</p></div>
      <div class="script-steps"><kbd>F2</kbd><span aria-hidden="true">→</span><span>{{ t.scriptStep }}</span><span aria-hidden="true">→</span><kbd>F5 / Ctrl+Enter</kbd></div>
      <div ref="tabList" class="script-tabs" role="tablist" :aria-label="zh ? '多行脚本示例' : 'Multiline script examples'">
        <button v-for="(item, i) in examples" :id="'ash-tab-' + i" :key="item.file" role="tab" type="button" :aria-selected="selected === i" aria-controls="ash-script-panel" :tabindex="selected === i ? 0 : -1" @click="selected = i" @keydown="handleTabs($event, i)">{{ t.scriptLabels[i] }}</button>
      </div>
      <div id="ash-script-panel" class="script-panel" role="tabpanel" :aria-labelledby="'ash-tab-' + selected">
        <div class="script-code">
          <p class="example-description">{{ t.scriptDescriptions[selected] }}</p>
          <div class="code-window">
            <div class="code-header"><span class="code-title">{{ example.file }}</span><button type="button" class="copy-button" @click="copyCode(example.code)">{{ copied === example.code ? t.copied : t.copy }}</button></div>
            <pre class="code-body" tabindex="0"><code>{{ example.code }}</code></pre>
          </div>
          <div class="observed-output"><span>{{ t.scriptOutput }}</span><pre><code>{{ example.output }}</code></pre></div>
          <a class="text-link" :href="assets + 'examples/' + example.file" download>{{ t.scriptDownload }} ↗</a>
        </div>
        <div class="script-evidence">
          <EvidenceImage :src="assets + example.image" :alt="'F2 / ' + t.scriptLabels[selected]" :caption="t.scriptCaption" :zoom-label="t.zoom" :close-label="t.close" :original-label="t.original" />
          <p v-if="selected === 2" class="small-note">{{ t.jsonPathNote }}</p>
        </div>
      </div>
      <p class="small-note evidence-note">{{ t.scriptNote }}</p>
    </section>

    <section id="automation" class="content-section automation-section">
      <div class="section-heading"><span class="section-eyebrow">04 / CLI</span><h2>{{ t.automationTitle }}</h2><p>{{ t.automationDesc }}</p></div>
      <div class="features-grid"><FeatureCard v-for="item in t.policies" :key="item.title" v-bind="item" color="color-mix(in srgb, var(--page-accent-2) 12%, transparent)" /></div>
      <div class="code-window query-code"><div class="code-header"><span class="code-title">ash · CLI</span><button type="button" class="copy-button" @click="copyCode(policies)">{{ copied === policies ? t.copied : t.copy }}</button></div><pre class="code-body" tabindex="0"><code>{{ policies }}</code></pre></div>
      <p class="small-note">{{ t.policyNote }}</p>
      <details class="version-boundaries"><summary>{{ t.statusTitle }}</summary><ul><li v-for="item in t.statusItems" :key="item">{{ item }}</li></ul></details>
    </section>

    <section id="quick-start" class="content-section quick-start-section">
      <div class="section-heading"><span class="section-eyebrow">05 / Get started</span><h2>{{ t.quickTitle }}</h2><p>{{ t.quickDesc }}</p></div>
      <div class="launch-grid">
        <div v-for="(code, i) in launches" :key="i" class="launch-card">
          <h3>{{ t.launchLabels[i] }}</h3><p>{{ t.launchHints[i] }}</p>
          <div class="code-window"><div class="code-header"><span class="code-title">ash</span><button class="copy-button" type="button" @click="copyCode(code)">{{ copied === code ? t.copied : t.copy }}</button></div><pre class="code-body" tabindex="0"><code>{{ code }}</code></pre></div>
          <div v-if="i === 2" class="sample-downloads"><a :href="assets + 'examples/user-report.ash'" download>user-report.ash ↗</a><a :href="assets + 'examples/users.json'" download>users.json ↗</a></div>
        </div>
      </div>
      <details class="build-details"><summary>{{ t.buildTitle }}</summary><p>{{ t.buildNote }}</p><div class="code-window"><div class="code-header"><span class="code-title">auto-shell/ash</span></div><pre class="code-body" tabindex="0"><code>{{ build }}</code></pre></div><a :href="repo">{{ t.source }} ↗</a></details>
      <p class="small-note evidence-note">{{ t.evidenceNote }}</p>
    </section>

    <div class="cta-section"><div class="cta-actions"><a :href="repo" class="btn btn-primary">{{ t.source }} ↗</a><a :href="prefix + '/v05/'" class="btn btn-secondary">{{ t.release }}</a></div></div>
    <span class="sr-only" aria-live="polite">{{ copied === 'failed' ? t.copyFailed : copied ? t.copied : '' }}</span>
  </div>
</template>

<style scoped>
.autoshell-page { --page-accent-1: #b45309; --page-accent-2: #2563eb; }
:global(.dark .autoshell-page) { --page-accent-1: #f59e0b; --page-accent-2: #60a5fa; }
.autoshell-page .btn-primary { background: linear-gradient(135deg, #b45309, #2563eb); color: #fff; }
.autoshell-hero { padding: 64px 32px 56px; }
.hero-grid { display: grid; grid-template-columns: 1.05fr 1fr; align-items: center; gap: 56px; max-width: 1120px; margin: auto; text-align: left; }
.hero-grid > *, .script-panel > * { min-width: 0; }
.hero-back { display: block; margin-bottom: 24px; color: hsl(var(--muted-foreground)); font-size: 13px; }
.autoshell-hero .badge { margin-bottom: 20px; }
.autoshell-hero .title { font-size: clamp(38px, 4.2vw, 58px); line-height: 1.18; }
.autoshell-hero .description { margin: 0 0 24px; font-size: 17px; line-height: 1.9; }
.autoshell-hero .actions { justify-content: flex-start; }
.hero-meta { margin: 24px 0 0; color: hsl(var(--muted-foreground)); font-size: 12px; line-height: 1.7; }
.autoshell-page .code-window { max-width: none; box-shadow: 0 16px 40px rgb(0 0 0 / 10%); }
.hero-terminal { transform: rotate(-1deg); border: 1px solid #313244; }
.hero-terminal .code-body { font-size: 13px; padding: 28px 24px; line-height: 1.9; }
.terminal-tags { display: flex; gap: 8px; padding: 0 24px 20px; }
.terminal-tags span { border: 1px solid #41445b; color: #bac2de; font-size: 11px; border-radius: 5px; padding: 2px 8px; }
.section-nav { position: sticky; top: 56px; z-index: 20; display: flex; gap: 8px; justify-content: center; padding: 12px 24px; border-bottom: 1px solid hsl(var(--border)); background: hsl(var(--background) / 95%); backdrop-filter: blur(12px); overflow-x: auto; }
.section-nav a { white-space: nowrap; padding: 8px 14px; border-radius: 6px; color: hsl(var(--muted-foreground)); font-size: 13px; font-weight: 600; }
.section-nav a:hover { background: hsl(var(--secondary)); color: hsl(var(--foreground)); }
.overview-section { max-width: 1184px; padding-top: 40px; padding-bottom: 0; }
.content-section { max-width: 1184px; margin: auto; padding: 56px 32px; scroll-margin-top: 128px; }
.content-section + .content-section { border-top: 1px solid hsl(var(--border)); }
.content-section :deep(.showcase-section) { padding: 0; gap: 40px; }
.native-gallery { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 32px; }
.mode-gallery { display: grid; gap: 32px; }
.interface-example, .mode-example { min-width: 0; }
.interface-example h3, .mode-copy h3 { font-size: 19px; font-weight: 600; margin: 0 0 12px; line-height: 1.5; }
.interface-example > p, .mode-copy > p { color: hsl(var(--muted-foreground)); font-size: 14px; line-height: 1.8; margin: 0 0 20px; }
.mode-example { display: grid; grid-template-columns: 1fr 1.55fr; column-gap: 32px; align-items: start; }
.mode-copy h3 { margin-top: 12px; }
.shortcut-panel { margin-top: 32px; padding: 24px; border: 1px solid hsl(var(--border)); border-radius: 12px; background: hsl(var(--secondary) / 40%); }
.shortcut-panel h3, .launch-card h3 { margin: 0 0 16px; font-size: 16px; font-weight: 600; }
.shortcut-grid { display: flex; flex-wrap: wrap; gap: 18px 32px; font-size: 13px; }
.shortcut-grid > div { display: flex; align-items: center; gap: 10px; }
kbd { display: inline-block; padding: 3px 8px; font: 12px var(--vp-font-family-mono); border: 1px solid hsl(var(--border)); border-bottom-width: 2px; border-radius: 5px; color: hsl(var(--foreground)); background: hsl(var(--background)); white-space: nowrap; }
.small-note { margin: 16px 0 0; font-size: 13px; line-height: 1.8; color: hsl(var(--muted-foreground)); }
.small-note a, .text-link, .sample-downloads a, .build-details > a { color: var(--vp-c-brand-1); text-decoration: underline; text-underline-offset: 3px; }
.section-heading { max-width: 780px; margin-bottom: 28px; }
.section-eyebrow { color: var(--page-accent-1); font-size: 12px; font-weight: 700; letter-spacing: .06em; }
.section-heading h2 { font-size: clamp(25px, 3vw, 34px); line-height: 1.35; font-weight: 700; margin: 12px 0 16px; }
.section-heading > p { font-size: 16px; line-height: 1.85; color: hsl(var(--muted-foreground)); }
.query-code { margin-top: 28px; }
.code-header { justify-content: space-between; gap: 12px; }
.copy-button { flex-shrink: 0; margin-left: auto; padding: 3px 8px; font-size: 12px; color: #cdd6f4; border: 1px solid #41445b; border-radius: 5px; }
.copy-button:hover { background: #313244; }
.code-body { font-size: 13px; }
.script-steps { display: flex; gap: 12px; flex-wrap: wrap; align-items: center; margin: 0 0 24px; font-size: 13px; color: hsl(var(--muted-foreground)); }
.script-tabs { display: flex; gap: 8px; padding: 5px; width: fit-content; max-width: 100%; margin-bottom: 24px; background: hsl(var(--secondary)); border: 1px solid hsl(var(--border)); border-radius: 9px; overflow-x: auto; }
.script-tabs button { padding: 10px 18px; border-radius: 6px; white-space: nowrap; font-size: 14px; color: hsl(var(--muted-foreground)); }
.script-tabs button[aria-selected="true"] { background: hsl(var(--background)); color: hsl(var(--foreground)); box-shadow: 0 2px 5px rgb(0 0 0 / 8%); font-weight: 600; }
.script-panel { display: grid; grid-template-columns: 1fr 1fr; gap: 32px; align-items: start; }
.example-description { margin: 0 0 16px; font-size: 14px; line-height: 1.8; color: hsl(var(--muted-foreground)); }
.script-code .code-body { padding: 20px; line-height: 1.65; }
.script-evidence { padding-top: 40px; }
.observed-output { padding: 16px 20px; margin: 16px 0; border-left: 3px solid var(--page-accent-1); background: hsl(var(--secondary) / 55%); border-radius: 0 8px 8px 0; }
.observed-output > span { font-size: 12px; color: hsl(var(--muted-foreground)); }
.observed-output pre { margin-top: 8px; font: 13px/1.8 var(--vp-font-family-mono); overflow-x: auto; }
.evidence-note { max-width: 860px; }
.version-boundaries, .build-details { margin-top: 24px; padding: 20px 24px; border: 1px solid hsl(var(--border)); border-radius: 10px; background: hsl(var(--secondary) / 35%); font-size: 14px; line-height: 1.85; }
summary { cursor: pointer; font-weight: 600; }
.version-boundaries ul { list-style: disc; padding-left: 20px; margin: 16px 0 0; color: hsl(var(--muted-foreground)); }
.version-boundaries li + li { margin-top: 12px; }
.launch-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 24px; }
.launch-card { min-width: 0; }
.launch-card > p { min-height: 52px; font-size: 13px; line-height: 1.8; color: hsl(var(--muted-foreground)); margin-bottom: 12px; }
.launch-card .code-body { min-height: 100px; }
.sample-downloads { display: flex; flex-wrap: wrap; gap: 16px; margin-top: 12px; font-size: 12px; }
.build-details > p { color: hsl(var(--muted-foreground)); margin: 16px 0; }
.build-details > a { display: inline-block; margin-top: 16px; }
.autoshell-page :is(a, button, summary, pre):focus-visible { outline: 3px solid var(--vp-c-brand-1); outline-offset: 4px; }
.sr-only { position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip-path: inset(50%); white-space: nowrap; }
@media (max-width: 960px) { .hero-grid { gap: 28px; } .launch-grid { grid-template-columns: 1fr; } .launch-card > p { min-height: 0; } .launch-card .code-body { min-height: 0; } }
@media (max-width: 768px) { .native-gallery, .mode-example { grid-template-columns: 1fr; } .mode-example { row-gap: 0; } .autoshell-hero { padding: 40px 24px; } .hero-grid, .script-panel { grid-template-columns: 1fr; } .hero-terminal { transform: none; margin-top: 12px; } .section-nav { justify-content: flex-start; padding: 10px 16px; } .section-nav a { padding: 8px 10px; } .content-section { padding: 40px 24px; } .overview-section { padding: 32px 24px 0; } .script-evidence { padding-top: 0; } .shortcut-panel { padding: 20px; } .script-tabs button { padding: 8px 12px; font-size: 13px; } }
@media (prefers-reduced-motion: reduce) { .autoshell-page * { scroll-behavior: auto !important; transition: none !important; } }
</style>
