# PLAN-750 T-01/T-02 证据档案：vue 轨草稿结算竞态定位（2026-10-10）

载体与依赖：
- 主检出现役 exe：`auto 0.1.0+v0.4.2-2853-gc72ff953b-dirty`（10-10 11:18 构建
  = jade 4/4 失败运行所用失败工件；供料包称 v2835 域）。
- 老载体：ab7a650bd = `v0.4.2-2747`（10-08 16:49；jade 10-09 绿跑时代域），
  干净构建于 `D:/autostack/.wt/lang-750-m4/auto-lang`（配 auto-down 895f8d0
  兄弟 + auto-edit specs/stylekit 副本）。
- jade 源：HEAD（e949b62 后，身份改名批——`.jade`→`.auto-jade` 草稿根）；
  e2e 并行会话活跃（同机负载窗）。

## 一、机制定谳（动态复现，D 场景）

场景 D = 打开 Hello World → `keyboard.type('XY', {delay:0})` → 中性 blur。
新载体与老载体**同等复现完整失败签名**（新载体日志 3788ms 段 / 老载体
11216ms 段，形态逐字段同构）：

```
draft_begin × 2        ← 两次 Edit 在对方 draft_begin 往返内重入（11ms/…间隔）
draft_alloc × 2        → d0001 + d0002（后者建目录后永远空——供料包 d0004 同构）
draft_checkpoint × 1   → doc=d0001, erev=2（两键均已落账后才快照）
（无 draft_settle —— 后续保存时 tab 账面=d0002/draft_seq=0 → Save 的
  `draft_seq > 0` 守卫静默跳过 settle；tab 卡 state='sending'）
```

磁盘终态：`d0001/: c00001[erev=2]`（卡死）+ `d0002/: (EMPTY)` ——与 jade
供料包一手证据 `d0003 卡 checkpoint + d0004 空目录` 逐字段同构。

## 二、归因矩阵（T-02 定谳：RC-C 潜伏竞态，非 auto-lang 回归）

| 判别 | 方法 | 结果 |
|---|---|---|
| RC-A 前端 codegen 回归 | 同份 jade 源分别用 2747 老 exe 与现役 exe `build -r vue --gen-only`，全树 diff | **零 diff**（34 组件 src 树逐字节恒等）→ 出局 |
| RC-B 后端 RTT 回归 | 老/新 exe 作 `--server vm` 后端，node 直连采样 draft_begin ×20 | 老新恒等（静载 p50 1-5ms/max≤21ms；B 场景全协议链两载体均健康 settle）→ 出局 |
| RC-C 潜伏竞态 | D 场景双载体对照 | **两载体同败**（含绿时代载体 ab7a650bd）→ 确证 |
| 负载敏感性 | 同场景不同负载窗采样 | 静载 RTT p50=1ms（A 场景 50ms 轮间距健康）；负载窗 p50=24-61ms（jade 会话活跃期）→ 窗口越键间隔即触发 |

对照场景（全部健康，机制反面印证）：
- A（两轮 insertText+blur，轮间距 ~50ms > 窗口）：单 alloc、两 checkpoint
  （erev=1→2）顺序排水，state='saved'，console「草稿已保护」；
- B（matrix 忠实 [4edit]→[5save]：insertText+poll+保存）：checkpoint→
  write_wiki→read_wiki→**settle(saved, seq=1)** 全链绿（老/新载体均然）；
- C（insertText 后立即保存）：同 B 绿——Save 流自身的 write_wiki+read_wiki
  两个 await 构成天然屏障，settle 检查前 checkpoint 回执必已落账。

## 三、触发条件与历史绿/红解释

- 引擎发射粒度：**首挂载编辑器逐键发射存活**（D-17「重挂载后逐键发射死、
  blur 冲刷」仅覆盖切档重挂载态）；delay:0 两键间隔 ~5ms。
- 分配悬窗宽度 = draft_begin+draft_alloc 经 vite 代理往返：静载 ~15ms，
  批量负载 25-60ms。
- jade e2e matrix 打字弧（`keyboard.type(..., {delay:25})` ×3 处）在批量
  套跑（自加载）下窗口稳定 >25ms → 第二键落进窗口 → 双分配 → 4/4 确定性；
  历史静载单跑窗口 <25ms → 绿；10-09 单 spec「修测后 1.3m 干净绿」与此自洽。
- vm 轨免疫：MCP type_text 事件经 MCP 往返串行，间隔 >> 窗口。
- jade .at 自身证据（editor_store.at 排水循环的身份重查模式）表明
  「await 悬窗内事件可重入」是**双轨统一**的设计模型——分配臂
  （`doc_id==""` → draft_begin/alloc）缺 in-flight 守卫是违反该模型的
  宿主缺口；runtime 侧（转译产物、后端）与模型一致，无回归。

## 四、结论与修复归属

- **归因 = RC-C**：vue 轨「引擎逐键发射 × 分配悬窗重入」潜伏竞态，jade
  侧 .at 分配臂守卫缺失为主体修复面（jade 仓）；auto-lang 无行为回归
  （codegen 零 diff + 后端恒等 + 老载体同败三证）。
- auto-lang 交付：SD-01 契约成文（ui/overview.md）+ 本证据档案 + 复现
  工具（repro.mjs）+ jade 供料包草稿（含 .at 守卫范式）。
- 复现配方：`cd D:/autostack/jade-edit && node <本目录>/repro.mjs D`
  （A/B/C/D 场景；AUTO_EXE 可换载体）。
