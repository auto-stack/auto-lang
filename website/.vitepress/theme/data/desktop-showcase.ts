// Original AutoOS captures: desktop-shots/showcase, 2026-10-01, 2560×1600.
// Shared by the release page, virtual desktop page and OS overview.
import type { GalleryShot } from '../components/ScreenshotGallery.vue'

export function desktopShots(zh: boolean): GalleryShot[] {
  const rows = zh ? [
    ['01-desktop-light.png', '桌面 · 浅色', '浅色 AutoOS 桌面，图标、壁纸与右上角小组件，无展开的应用窗口', '先看桌面：浅色壁纸、应用图标与任务栏。右上角是时钟、待办和音乐小组件；对应应用已启动并最小化。'],
    ['02-desktop-dark.png', '桌面 · 深色', '深色 AutoOS 桌面，图标、壁纸与右上角小组件，无展开的应用窗口', '同一桌面的深色主题：图标、任务栏和小组件随主题呈现。此处没有展开的应用窗口，应用已在后台运行。'],
    ['03-launcher-light.png', 'Launcher · 浅色', '浅色 Launcher 的应用图标网格', '接着打开应用：Launcher 的图标网格列出应用，支持搜索与键盘选择；Tab 可切换展示形式。'],
    ['04-launcher-dark.png', 'Launcher · 深色', '深色 Launcher 的应用图标网格', '深色主题下的同一 Launcher 图标网格，桌面保留在面板后方。'],
    ['05-games-dark.png', '小游戏 · 多窗口', '深色桌面中并排的俄罗斯方块、纸牌接龙和扫雷窗口', '再看多窗口：俄罗斯方块、纸牌接龙和扫雷同时打开，窗口拖动后并排摆放，主要游戏区域都可见。'],
    ['06-productivity-dark.png', '工作 · 编辑与待办', '编辑器占左半屏，待办清单位于右上，日历位于右下', '最后安排工作：AutoEdit 占左半屏，待办清单在右上、日历在右下，代码、任务和日期可以同时查看。'],
  ] : [
    ['01-desktop-light.png', 'Desktop · Light', 'Light AutoOS desktop with wallpaper, app icons and upper-right widgets, without expanded app windows', 'Start at the desktop: light wallpaper, app icons and taskbar. Clock, Todo and music widgets appear at the upper right after their apps have been started and minimized.'],
    ['02-desktop-dark.png', 'Desktop · Dark', 'Dark AutoOS desktop with wallpaper, app icons and upper-right widgets, without expanded app windows', 'The same desktop in its dark theme: icons, taskbar and widgets share the appearance. No app windows are expanded here; the widget apps are running in the background.'],
    ['03-launcher-light.png', 'Launcher · Light', 'Light Launcher showing its application icon grid', 'Next, open an app: the Launcher icon grid lists applications, with search and keyboard selection. Tab switches its presentation.'],
    ['04-launcher-dark.png', 'Launcher · Dark', 'Dark Launcher showing its application icon grid', 'The same Launcher icon grid in the dark theme, with the desktop still visible behind the panel.'],
    ['05-games-dark.png', 'Games · Multiple windows', 'Tetris, Solitaire and Minesweeper arranged side by side on the dark desktop', 'Then try multiple windows: Tetris, Solitaire and Minesweeper are open together. Their windows have been dragged side by side so the main game areas remain visible.'],
    ['06-productivity-dark.png', 'Work · Editor and tasks', 'AutoEdit on the left half, Todo at the upper right and Calendar at the lower right', 'Finally, arrange a workspace: AutoEdit takes the left half, with Todo above Calendar on the right, keeping code, tasks and dates in view together.'],
  ]
  return rows.map(([file, label, alt, caption]) => ({ src: `/desktop-showcase/${file}`, label, alt, caption, width: 2560, height: 1600 }))
}

export function desktopSteps(zh: boolean) {
  return zh ? [
    { title: '从桌面开始', desc: '图标、壁纸与任务栏组成桌面。时钟、待办和音乐小组件来自已启动并最小化的应用。' },
    { title: '用 Launcher 找到应用', desc: '打开启动器，以 Tab 切到图标网格，再通过搜索或键盘选择应用。' },
    { title: '试试多个窗口', desc: '同时打开俄罗斯方块、纸牌接龙和扫雷；拖动窗口，让主要内容并排可见。' },
    { title: '安排一个工作场景', desc: '编辑器占左半屏，右侧上下放置待办清单和日历，同时查看代码、任务与日期。' },
  ] : [
    { title: 'Start at the desktop', desc: 'Icons, wallpaper and taskbar form the desktop. Clock, Todo and music widgets come from apps that have been started and minimized.' },
    { title: 'Find an app in Launcher', desc: 'Open Launcher, use Tab to switch to its icon grid, then choose an app through search or keyboard navigation.' },
    { title: 'Try multiple windows', desc: 'Open Tetris, Solitaire and Minesweeper together, then drag the windows to keep their main content visible side by side.' },
    { title: 'Arrange a workspace', desc: 'Give the editor the left half and stack Todo above Calendar on the right, keeping code, tasks and dates in view.' },
  ]
}
