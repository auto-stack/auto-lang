// units.mjs — bp-gate 单元台账配置（PLAN-075 T-03，jade gallery units.mjs 同形；
// PLAN-665 T-02 增第四单元 sandwich）。
//
// 每单元双臂断言面：
//   vue —— playwright：单页同定（无深链），全页 needle（fixture 刻意四单元
//          互斥词表）+ clip 区域截图基线（e2e/baselines/<id>.png，固定高度
//          堆叠 h-52=208px·sandwich 416px，viewport 800x1040；
//          --update-snapshots 刷新）。
//   vm  —— 单次 boot：root 投影字段等值断言（autoui_state）+ root 单元标记行
//          snapshot needle（sandwich 另含 slot 填充 needle——T-00 实证
//          snapshot v2(rendered) 可见 bp 子件子树）。
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
]

// 视口与单元几何（clip 计算源；playwright.config.ts 对齐）。
// PLAN-665：视口高 640→1040（第四单元 624+416——前三单元 clip 绝对坐标
// 不变，既有基线零扰动；sandwich 单元底缘=视口底，AC-04 贴底断言用）。
export const GEOMETRY = { viewport: { width: 800, height: 1040 }, unitHeight: 208 }
