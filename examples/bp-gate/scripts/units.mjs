// units.mjs — bp-gate 单元台账配置（PLAN-075 T-03，jade gallery units.mjs 同形；
// PLAN-665 T-02 增第四单元 sandwich；PLAN-676 T-05 增第五单元 gallery_shell）。
//
// 每单元双臂断言面：
//   vue —— playwright：单页同定（无深链），全页 needle（fixture 刻意单元
//          互斥词表）+ clip 区域截图基线（e2e/baselines/<id>.png，固定高度
//          堆叠 h-52=208px·sandwich/gallery_shell 416px，viewport 800x1456；
//          --update-snapshots 刷新）。
//   vm  —— 单次 boot：root 投影字段等值断言（autoui_state）+ root 单元标记行
//          snapshot needle（sandwich/gallery_shell 另含 slot 填充 needle——
//          T-00 实证 snapshot v2(rendered) 可见 bp 子件子树）。
//
// 新单元流程：host/src/front/app.at 加单元区（固定高度+标记行+互斥词表
// fixture）→ 本表登记（clip y 按单元顶缘）→ pnpm exec playwright test
// --update-snapshots 建基线 → node scripts/gate.mjs 全绿。
export const UNITS = [
  {
    id: 'status_bar',
    title: 'status-bar 骨架（layout·fn-free，PLAN-075 T-02）',
    vue: {
      needles: ['3 backlinks', 'saved', '1,024 words'],
      clip: { x: 0, y: 0, width: 800, height: 208 },
    },
    vm: {
      state: { sb_left: '3 backlinks', sb_center: 'saved', sb_right: '1,024 words' },
      snapshot: ['unit: status_bar'],
    },
  },
  {
    id: 'row_list',
    title: 'row-list 骨架（data-display·fn-free，PLAN-075 T-02）',
    vue: {
      needles: ['alpha row', 'beta row', 'gamma row'],
      clip: { x: 0, y: 208, width: 800, height: 208 },
    },
    vm: {
      state: { rl_count: '3' },
      snapshot: ['unit: row_list'],
    },
  },
  {
    id: 'filetree',
    title: 'filetree 组合形态（navigation·跨文件 fn 携带件，645 转译）',
    vue: {
      needles: ['wiki', '方法', '引言.ad', '另页.ad'],
      clip: { x: 0, y: 416, width: 800, height: 208 },
    },
    vm: {
      state: { ft_count: '4' },
      snapshot: ['unit: filetree'],
    },
  },
  {
    // PLAN-665 T-02：fn-free slot 形态首证单元。416px（2×208 节奏）——
    // AC-04 几何断言（content ≥300px）要求。snapshot needles 除 root 标记
    // 行外含 slot 填充文本（T-00 实证 snapshot v2(rendered) 可见 bp 子树，
    // F-1 观察修正；root 投影字段叠加不替换）。
    id: 'sandwich',
    title: 'sandwich 骨架（layout·fn-free slot 四出口，PLAN-665；full 嵌 content 组合）',
    vue: {
      needles: ['sw toolbar', 'sw sidebar', 'sw status', 'swf toolbar', 'swf full content', 'swf status'],
      clip: { x: 0, y: 624, width: 800, height: 416 },
    },
    vm: {
      state: { sw_slots: 'toolbar|sidebar|content|statusbar', sw_content_rows: '1', swf_slots: 'toolbar|content|statusbar' },
      snapshot: ['unit: sandwich', 'sw toolbar', 'sw sidebar', 'sw status', 'swf toolbar', 'swf full content', 'swf status'],
    },
  },
  {
    // PLAN-676 T-05：gallery-shell 骨架单元（三画廊框架综合；非受控
    // popover 设置弹层 + on_* msg-ref 回调契约 + 值 props 首证）。
    // VM snapshot 含 bp 子树 slot 填充 needle（sandwich T-00 同款双保险）；
    // 弹层内容（Theme/Accent 按钮）触发后才入树，静态断言不含（gotchas#3）。
    id: 'gallery_shell',
    title: 'gallery-shell 骨架（layout·三画廊综合，PLAN-676）',
    vue: {
      needles: ['gs brand', 'gs pill one', 'gs pill two', 'gs content body', 'gs/one', 'gs title two', 'Settings'],
      clip: { x: 0, y: 1040, width: 800, height: 416 },
    },
    vm: {
      state: { gs_active: 'gs/one', gs_dark: 'true', gs_accent: 'indigo' },
      snapshot: ['unit: gallery_shell', 'gs brand', 'gs pill one', 'gs content body', 'gs/one', 'gs title one'],
    },
  },
]

// 视口与单元几何（clip 计算源；playwright.config.ts 对齐）。
// PLAN-665：视口高 640→1040（第四单元 624+416——前三单元 clip 绝对坐标
// 不变，既有基线零扰动；sandwich 单元底缘=视口底，AC-04 贴底断言用）。
// PLAN-676：视口高 1040→1456（第五单元 1040+416——前四单元 clip 绝对
// 坐标不变，既有基线零扰动）。
export const GEOMETRY = { viewport: { width: 800, height: 1456 }, unitHeight: 208 }
