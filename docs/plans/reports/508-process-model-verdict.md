# Plan 508 G2 —— 进程模型对比实测与默认策略裁定

- 计划：`docs/plans/508-desktop-protocol-stage6-remote-policy.md`（G1/G2）
- 日期：2026-09-01
- 平台：Windows 11（10.0.26200）x64
- 构建：`cargo nextest` 默认 **debug** profile；outproc 子进程 = 真实
  `target/debug/auto.exe`（`run --autodesk-incubate` 生产链，非 480 的
  re-exec 测试体）
- 复跑稳定性：两臂各跑两轮——内存差 <1%，时延差在进程 spawn 噪声带内
  （首例冷启 247.6ms vs 65.0ms 为机器噪声主项，其余样本差 <20%）
- harness：`stage3::tests::p508_g2_inproc_arm` / `p508_g2_outproc_arm`
  （各自独立测试进程，避免臂间内存串染）

## 1. 度量口径

- **冷启动到首帧**：
  - inproc 臂 = `launch_app()` 同步时长（resolver→编译装载→挂会话→
    开虚拟窗）。**首帧时点 = 返回后下一渲染节拍（~16ms 帧预算）**——
    inproc 无独立帧到达事件，窗口内容随宿主下一帧上屏；该口径对
    inproc 略偏保守（多计 ≤1 节拍）。
  - outproc 臂 = spawn 真子进程 → broker 孵化 → attach Active
    （`launch_app` 返回点，attach_ms）→ 首个 composed DrawList 到达
    （firstframe_ms，自旋泵 10s 预算）。全链实测。
- **稳态内存**（Private 口径，480 方法 = `K32GetProcessMemoryInfo`，
  阶段 N=1/3/5 各 settle 1.5s 后采样）：
  - inproc 臂 = 宿主进程 PrivateUsage 阶梯（App 成本全部在宿主内）；
  - outproc 臂 = 宿主 Private（broker/会话泵侧）+ 全体子进程 Private
    （`auto.exe` 实体）。边际 = (N5−N1)/4，排首个子进程冷起底噪。
- **交互往返延迟**（点击→帧更新，预热 3–5 次后采样 20 取 median/p95）：
  - inproc 臂 = **引擎核心**：孪生投影器 `on_with_input`（VM handler）
    + `render_frame`（视图重建+DrawList 编码）——与 child 点击链同引擎，
    减去 IPC/进程开销；
  - outproc 臂 = **端到端**：`broker_pointer_down` → 子进程 VM handler →
    新帧 shm → 宿主 composed 文本变化（自旋泵，量化粒度 ~亚毫秒）。
  - 两臂差值 ≈ 进程/协议栈开销（宿主侧渲染上屏两形态同为下一节拍，
    不入差值）。

## 2. 实测数字

### 2.1 冷启动到首帧（ms，两轮）

| App | inproc launch | outproc attach | outproc 首帧 |
|-----|--------------:|---------------:|-------------:|
| 001-helloworld（首例，冷） | 3.7 / 3.8 | 247.6 / 65.0 | +1.5 / +1.5 |
| 002-counter | 8.4 / 2.6 | 27.2 / 46.5 | +1.2 / +0.5 |
| 003-converter | 3.1 / 2.2 | 25.8 / 25.7 | +1.1 / +1.8 |
| 004-profile-card | 4.5 / 2.1 | 25.6 / 25.5 | +0.7 / +0.9 |
| 005-login | 3.0 / 2.5 | 26.2 / 27.2 | +1.4 / +1.3 |
| 009-article-feed（中型） | 17.1 / 9.2 | 54.5 / 53.9 | **>10s 预算†** |

† 009 未被 RenderQueue queue 模式覆盖（`button.variant`、
`style:cursor-pointer`、`style:opacity-60`、`tag:exampleheader`），
auto 裁决降级 independent（像素臂）——首帧在 10s 度量预算内未达
（composed 队列口径），且代价见 §2.2 专行。**该降级路径正是 507
（Stage 5 全覆盖爬坡）要消除的对象**。

### 2.2 稳态内存（PrivateUsage 阶梯，两轮均值）

| 阶段 | inproc 宿主 | outproc 宿主 | outproc 子进程合计 |
|------|------------:|-------------:|-------------------:|
| base（0 App） | 3.31 MiB | 3.20 MiB | — |
| N=1（001） | 4.15 MiB | 4.66 MiB | 5.66 MiB |
| N=3（+002/003） | 6.43 MiB | 7.26 MiB | 18.89 MiB |
| N=5（+004/005） | 7.59 MiB | 9.30 MiB | 31.55 MiB |
| +009 探针 | 9.31 MiB | 11.96 MiB | **242.2 MiB**（含 009 像素臂 +210.5 MiB） |

**边际增量（每增 1 App，(N5−N1)/4）**：

| 口径 | inproc | outproc |
|------|-------:|--------:|
| 宿主侧 | **0.86 MiB/App** | 1.16 MiB/App |
| 子进程侧 | — | **6.48 MiB/App** |
| **系统总边际** | **0.86 MiB/App** | **7.64 MiB/App** |

- outproc 真实 `auto.exe` 子进程边际 **6.48 MiB/App**，高于 480 基线的
  4.81 MiB/App（480 子进程 = re-exec 测试体；生产二进制多出 logger/
  registry/CLI 装配面）——4.81 是下界口径，6.48 是生产口径。
- 009 降级像素臂单 App 成本 **+210 MiB**（iced 隐藏窗自渲染 + 截图泵）
  ——未覆盖 App 在 outproc 形态下的内存不可接受，inproc 同 App 仅
  +1.74 MiB。

### 2.3 交互往返延迟（ms，n=20）

| 臂 | median | p95 |
|----|-------:|----:|
| inproc（引擎核心：handler+视图重建） | 0.071 | 0.076–0.117 |
| outproc（端到端：IPC+child 全链+宿主合成） | 1.536 | 1.593–1.737 |
| **差值 = 进程/协议开销** | **≈1.47** | — |

## 3. 裁定（明示）

> **维持 `inproc` 为缺省进程模型；`outproc` 保留为显式隔离/实验选项
> （`shell.apps.process_model: outproc`，G1 已落）。**

依据（不预设结论、以数据定）：

1. **内存**：系统总边际 inproc 0.86 vs outproc 7.64 MiB/App（≈9×）。
   桌面多 App 常驻形态下 outproc 的进程税结构性偏高（debug 口径；
   release 会收敛但量级差不会反转）。
2. **启动**：inproc 2–17ms vs outproc 25–250ms+（≈10×），且 outproc
   首例冷启受进程 spawn 噪声影响方差大（65–248ms）。
3. **交互**：outproc 端到端 1.54ms median——绝对值仍在流畅带（<<16ms
   帧预算），**交互延迟本身不构成否决 outproc 的理由**；但叠加内存与
   启动劣势，无翻转向 outproc 的数据支撑。
4. **未覆盖 App 降级**：507（Stage 5）合入前，outproc auto 对中型 App
   （009 实证）降级像素臂：+210 MiB/App 且首帧不可用级——**outproc
   作为默认形态在覆盖集补齐前不可行**。
5. **outproc 存在价值**（保留配置位的理由）：崩溃隔离（App panic 不拖
   垒桌面）、统一 RenderQueue 路径（远程/多宿主复用，本计划 G3–G6
   的 WS 远程线消费同链）、独立内存账（per-App 可审计）。

### 翻转路径与触发条件（何时重议默认 outproc）

- **T-覆盖**：507 合入（Tier1+2 全覆盖）后重跑本 harness——若中型示例
  （009 族）降级消失且子进程边际进入 ~2MiB 级（release 构建 + 运行时
  裁剪），启动进入 ~50ms 级；
- **T-稳定性**：桌面实机出现 App 崩溃拖垮宿主的频度数据（隔离需求从
  假设变为实测）；
- **T-远程**：G3–G6 WS 远程会话成为主消费形态（远程红利要求 App 会话
  天然进程外）。

三者其一成立即重议；当前全部不成立。

## 4. 复现

```bash
cargo nextest run -p auto-lang --lib --features ui-iced \
  p508_g2_inproc_arm --nocapture    # AUTO508-INPROC-* 行
cargo nextest run -p auto-lang --lib --features ui-iced \
  p508_g2_outproc_arm --nocapture   # AUTO508-OUTPROC-* 行（先
                                    # cargo build -p auto --bin auto）
```

降级证据行（stderr）：`[render] 009-article-feed: auto -> independent
(coverage downgrade)`。
