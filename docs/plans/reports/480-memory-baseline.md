# Plan 480 —— 内存实测基线报告（Stage 3 验收：内存实测验收）

- 计划：`docs/plans/480-desktop-protocol-stage3.md`（S5 采样 + S6 本报告）
- 日期：2026-08-29
- 平台：Windows 11（10.0.26200）x64
- 构建：`cargo nextest` 默认 **debug** profile（测试宿主 = 未优化测试二进制）
- 复跑稳定性：两次运行差 < 1%（22.6/22.6、115.1/115.2 MiB）

## 1. 度量口径

- **采样点**：child 进程（`auto --autodesk-incubate` 等价物——压测中为
  re-exec 的真协议 client，`request_incubation` → `ClientPump::run`），
  宿主侧以 `K32GetProcessMemoryInfo` FFI 逐 pid 采样（零新依赖，Plan 480 S5）。
- **双字段**：
  - `WorkingSet`（物理驻留，含共享运行时页面）
  - `PrivateUsage`（提交私有字节，更贴近"每增一个 App 的净成本"）
- **口径 = 边际增量**（计划待澄清①）：`(总量(N=5) − 总量(N=1)) / 4`，
  排除首个 child 摊入的进程冷起底噪。
- **阶段化**：N=1 → N=3 → N=5 分批 spawn+attach，每阶段 settle 1.5s 后
  采样（`stage3_memory_baseline_n1_3_5`；早批次多出的 settle 时间使估计
  偏保守，对判定方向安全）。

## 2. 实测数字

### 2.1 全体 child 总量

| 阶段 | WorkingSet | PrivateUsage |
|------|-----------:|-------------:|
| N=1  | 22.6 MiB (23,658,496 B)  | 4.8 MiB (4,988,928 B) |
| N=3  | 68.8 MiB (72,130,560 B)  | 14.3 MiB (15,007,744 B) |
| N=5  | 115.2 MiB (120,823,808 B) | 24.0 MiB (25,161,728 B) |

### 2.2 边际增量（每增 1 个 child 的均摊）

| 口径 | 边际增量/App | 对 1-5MB/App 目标 |
|------|-------------:|-------------------|
| **PrivateUsage（净成本口径）** | **4.81 MiB** | **落入目标带上沿** ✅（临界） |
| WorkingSet（物理驻留口径） | 23.17 MiB | 超出约 4.6×（见 §3 归因） |

## 3. 判定结论（明示）

> 对 386 §0 的 "1-5MB/App" 目标：**PrivateUsage 口径判定为"临界达标"
> （4.81 MiB/App，处于目标带 5MB 上沿）**；**WorkingSet 口径判定为
> "未达标"（23.17 MiB/App）**。整体结论：**非硬达标，实测数字如实
> 呈现**——这正是计划把 386 复活条件从"硬达标"改写为"度量 + 判定"
> 的预期形态（待澄清①）。

**底噪归因**：

1. **Rust runtime 底噪**（计划 §目标 3 预告）：每个 child = 一个独立
   `auto` 运行时进程——tokio runtime（命名管道读写线程 + 1 worker）、
   AutoVM、编译器数据结构（AST/泛型注册表等 Rc 池）。N=1 的 PrivateUsage
   底噪即 4.8 MiB，几乎全部来自该层，与 App 负载（一个计数器 widget）
   无关。
2. **WorkingSet 放大因子**：debug 构建未做符号/页面裁剪，且 OS 以共享
   页计 WorkingSet——多个 child 共享同一 DLL/静态页使 WS 边际远高于
   Private 边际（23.2 vs 4.8 MiB）。WS 口径对"进程外 App 的内存成本"
   是高估口径。
3. **测试宿主偏差**：采样宿主为 nextest debug 测试二进制（非 release
   `auto` 产品二进制）。release + strip 预计显著下调两个口径；此为
   后续产品化时的复核点，不改变本次"度量 + 判定"的验收形态。

## 4. 复现

```bash
cargo nextest run -p auto-lang --lib --features ui-iced \
  desktop_protocol::stage3::tests::stage3_memory_baseline_n1_3_5 --nocapture
# 输出行：AUTO480-MEM stage=Nx ... / AUTO480-MEM marginal-per-app ...
```

压测路径（稳定窗末采样，N=3/N=5 档）：
`desktop_protocol::stage3::tests::stage3_multi_app_stress_n3/n5`，
输出行 `AUTO480-MEM stage=Nn ...`。
