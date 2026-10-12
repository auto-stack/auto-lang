# jade 供料包草稿：vue 轨草稿结算竞态根因与修复范式（PLAN-750 产出）

> 落点建议：jade-edit `docs/upstream/2026-10-10-draft-settle-race-rootcause.md`
> （本文件为 auto-lang PLAN-750 交接件，jade 会话取用/改写后入库）。

## 一、根因定谳（auto-lang 侧三证，2026-10-10）

上游载体无回归，竞态一直潜伏：

1. **codegen 零 diff**：同一份 jade 源分别用 v0.4.2-2747（ab7a650bd，绿
   时代载体）与现役 exe（v0.4.2-2853 域）执行 `build -r vue --gen-only`，
   34 组件 src 树**逐字节恒等**——转译产物（含 useEditorStore.ts 的
   Edit/DraftDrainOf/DraftSettleOf 全链）无任何变化。
2. **后端恒等**：两载体作 `--server vm` 后端，draft API 短往返恒等
   （静载 p50 1-5ms）；matrix 忠实 [4edit]→[5save] 弧在两载体全协议链
   健康（checkpoint→write→read-verify→settle saved 全绿）。
3. **老载体同败**：v2747 干净构建在最小复现场景（D 场景，见下）复现
   与现役载体逐字段同构的完整失败签名。

## 二、机制（与你方 d0003/d0004 一手证据逐点对账）

`Edit` 处理器的分配臂（`doc_id == ""` → `await draft_begin()` /
`await draft_alloc()`）存在 **await 悬窗重入**缺口：

1. 首挂载编辑器**逐键发射存活**（D-17「重挂载后逐键发射死」仅覆盖切档
   重挂载态）——matrix 打字弧 `keyboard.type(..., {delay:25})` 每键一个
   input 事件；
2. 键 N 的 Edit 在 draft_begin/draft_alloc 往返内挂起；键 N+1 的 Edit
   重入：erev 递增、doc_id 仍空 → **再次分配**；
3. alloc#1 返回 dA → Edit#N 续体排水：快照 erev 已含键 N+1 → checkpoint
   (dA, erev=2)（= 你方 c00001.erev=2 之谜）；alloc#2 返回 dB →
   `tab.doc_id = dB` **覆写**（= 你方 d0004 **空目录**签名——draft_alloc
   建目录后永无检查点）；
4. checkpoint 回执按 doc_id==dA 重扫 tabs 失配（已变 dB）→ 静默停，
   draft_seq 永不落账（tab 账面 dB，seq=0）；
5. 保存：`draft_seq > 0` 为假 → **settle 静默跳过**（磁盘无墓碑——
   d_settle 即使 verify 失败也会先写墓碑，无墓碑=从未调用）；文件照存、
   脏标照清；
6. 删除弧：tab 卡 `draft_state='sending'`（同运行）+ dA 检查点无墓碑
   （跨运行 draft_pending）→「草稿检查点在途」永久误阻。

**触发条件**：两键间隔 < 分配往返（vite 代理下静载 ~15ms、批量负载
25-60ms）。你们 4/4 确定性 = 全套件批跑自加载使窗口稳定 >25ms 键间隔；
历史静载单跑（含 10-09「修测后 1.3m 干净绿」）窗口 <25ms 故绿；
vm 轨 MCP 往返串行故免疫。同运行 d0001（单键弧 erev=1）健康结算与
d0003（双键弧 erev=2）卡死并存即此分界。

最小复现（干净工作区，任一载体）：
`auto run --server vm` + vite 前端 → 打开 Hello World →
`page.keyboard.type('XY', {delay:0})` → 中性 blur →
`.auto-jade/drafts/v1/`：`dA/c00001.meta[event=checkpoint,erev=2]` +
`dB/`（空）。完整仪器化脚本：auto-lang
`docs/plans/evidence/p750/repro.mjs`（场景 A/B/C/D，fetch 钩子记录
API 时间线与请求体）。

## 三、修复范式（.at 分配臂 in-flight 门）

editor_store.at `.Edit` 分配臂（两处同形态：~L414 与 ~L1122）加
in-flight 门；`draft_allocating` 为 App 态单标量（与 `draft_epoch`
同域）：

```text
if .tabs[j].dirty {
    if .tabs[j].doc_id == "" {
        if .draft_allocating {
            // 悬窗重入：只更新正文/erev（同步段已做），分配由在途
            // 调用收尾——跳过整个分配臂
        } else {
            .draft_allocating = true
            if .draft_epoch == 0 {
                .draft_epoch = draft_begin().parse_int()
            }
            .tabs[j].doc_id = draft_alloc()
            .draft_allocating = false
        }
    }
    if .tabs[j].doc_id != "" {
        if .tabs[j].draft_state != "sending" {
            store.DraftDrainOf(.tabs[j].doc_id)
        }
    } else {
        if !.draft_allocating {   // 真·分配失败才提示（重入期不误报）
            .save_note = "草稿身份分配失败——正文未保护"
            ...
        }
    }
}
```

正确性论证：Edit#N 续体在 alloc 返回后必然走 `doc_id != ""` 分支排水
（此时 state 非 sending——首个排水才置位），快照自动吸收重入键的正文
（erev 已前进）——**重入键的检查点保护由在途排水的续发语义覆盖**
（DraftDrainOf 既有 `draft_erev > erev → 续发最新` 机制），无保护空窗。
异常面：draft_begin/alloc 抛错时 `draft_allocating` 复位（try/catch 或
既有容错路径）需保证，否则永久卡门。

验收口径（jade 侧）：
- D 场景复现脚本改后：单 alloc、checkpoint 含 erev 前进、保存后
  `event=saved` 墓碑落、无孤儿空 doc 目录；
- matrix.spec 全绿（批量负载窗跑 ≥2 轮——负载是触发条件，静载单跑
  不构成否证）；vm_matrix merged 20/20 维持；
- 已污染工作区（在途 d0003 等）恢复面：草稿恢复面板可按既有
  RecoverDiscard 消费，或手清 `.auto-jade/drafts/v1/` 对应 doc 目录。

## 四、边界与不负项

- 上游不改：转译产物零 diff（证据一），后端零回归（证据二）——无需
  auto-lang 侧行为变更；PLAN-750 已将「vue 轨 handler 事件重入与
  await 悬窗」契约成文（auto-lang `docs/specs/auto-lang/ui/overview.md`
  SD-01，PLAN-750 节），重入是双轨统一模型、身份分配单飞责任在宿主。
- 引擎逐键发射（首挂载存活）为既有 D-17 边界族，不在本供料改动面；
  matrix 三处 `keyboard.type` 弧如需去脆弱化，可循 helpers
  appendToEditor 的 insertText 先例（单事件化）——与分配臂守卫二选一
  或双做，均成立（守卫为根修，弧线单事件化为减载）。
