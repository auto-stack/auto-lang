// units.mjs — bp-gate 单元台账配置（PLAN-075 T-03，jade gallery units.mjs 同形）。
//
// 每单元双臂断言面：
//   vue —— playwright：单页同定（无深链），全页 needle（fixture 刻意三单元
//          互斥词表）+ clip 区域截图基线（e2e/baselines/<id>.png，固定高度
//          堆叠 h-52=208px，viewport 800x640；--update-snapshots 刷新）。
//   vm  —— 单次 boot：root 投影字段等值断言（autoui_state；F-1：bp 子件
//          子树对快照不可见——内容断言不依赖子树）+ root 单元标记行
//          snapshot needle。
//
// 新单元流程：host/src/front/app.at 加单元区（固定高度+标记行+互斥词表
// fixture）→ 本表登记（clip y=208*n）→ pnpm exec playwright test
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
]

// 视口与单元几何（clip 计算源；playwright.config.ts 对齐）。
export const GEOMETRY = { viewport: { width: 800, height: 640 }, unitHeight: 208 }
