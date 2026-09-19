# Gotchas — layout/status-bar

## 1. 同名不同物：不要把 app 的实体 status_bar 收编成本 bp 的变体（jade 三版实证）

**wrong**: 把 jade web 的 ext-composable 版 StatusBar（store 直绑 + 光标位）
或 desktop 值 props 版当作本 bp 的第二 variant 收编。

**why**: 三版同名不同物（design 30 §6）：数据流（store 直绑 vs 值 props vs
静态）、运行时惯用（vue 专属 composable vs 双轨兼容）、功能面互异——硬收编
违反契约 Q6 双形态同语义（filetree gotcha#2 web 谱系同型裁定）。

**right**: 三版共用本**骨架**（布局+节奏），内容做各自的 slot 变体；实体
组件保持各 app 自持身份。出现真实跨 app 收敛需求时先做变体提升评审。

## 2. 内容 slot 的行派生 fn 归消费方自备（.at 无跨文件导入）

**wrong**: 在骨架里放模块 fn 供消费方复用，或指望消费方 import 骨架侧
helper。

**why**: .at 模块间 import 缺位（074-sink-mode G-3；links.at 头注在案）；
骨架纪律 = fn-free 纯布局。

**right**: 派生逻辑（字数统计/链接计数等）沉在消费方自己的 widget 文件
顶层模块 fn（Plan 367 P2-4），传值进 slot。

## 3. VM 轨消费方断言走 root 投影（子件子树对 MCP 快照不可见）

**wrong**: VM 轨把 StatusBarSkeleton 挂进组件后，靠 MCP snapshot 断言骨架
内部的 text 行。

**why**: vm 组件子树对 MCP 快照不可见（041 README「vm 组件边界」②；
filetree gotcha#1 同型）。

**right**: 断言面走 root 投影字段（消费方把要断言的值同时投到 root
state），渲染视觉断言走 vue 轨。
