# 025-sys-monitor — regeneration SPEC

> Purpose: Windows 11 任务管理器式系统监视器（Plan 541·025-dashboard 全面重构升级）。
> **真实系统数据源（sysinfo 后端 + auto.sys.* 原生函数族），双端对齐（Vue / VM 原生桌面）。**
> 架构模式：平铺多模块 Front (`app.at` + `sys_store.at` + `processes.at` + `performance.at` + `details_users.at` + `types.at`) + Back API (`api.at` + `sys_info.at`)。

---

## 1. 架构总览

```
                ┌───────────────────────────────────────────────┐
                │             auto.sys.* 原生函数族               │
                │   (crates/auto-lang/src/libs/sys.rs: sysinfo) │
                └───────────────────────┬───────────────────────┘
                                        │ (natives)
                                        ▼
                ┌───────────────────────────────────────────────┐
                │          后端服务 (src/back/sys_info.at)        │
                │        GET  /api/system/snapshot              │
                │        POST /api/system/kill                  │
                └───────────────────────┬───────────────────────┘
                                        │ (use back.api)
                                        ▼
                ┌───────────────────────────────────────────────┐
                │         SharedStore (src/front/sys_store.at)  │
                │  - 30 点滑窗历史 (CPU/Mem/Disk/Net)            │
                │  - SVG 面积图/折线几何自适应计算                 │
                │  - 多列排序与选择视图 (procsView)               │
                │  - 进程终止二次确认弹窗状态驱动                  │
                └───────────────────────┬───────────────────────┘
                                        │ (use sys_store)
        ┌───────────────┬───────────────┴───────────────┬───────────────┐
        ▼               ▼                               ▼               ▼
┌───────────────┐┌───────────────┐              ┌───────────────┐┌───────────────┐
│     壳应用     ││    进程页     │              │    性能页     ││ 详细/用户页   │
│ (src/front/   ││ (src/front/   │              │ (src/front/   ││ (src/front/   │
│   app.at)     ││ processes.at) │              │performance.at)││details_users) │
│ NavRail+Tick  ││ KPI+表+Kill确认│              │MiniRail+SVG图 ││ PID表+用户聚合│
└───────────────┘└───────────────┘              └───────────────┘└───────────────┘
```

---

## 2. 后端数据契约 (`src/back/api.at`)

### 2.1 类型定义
```auto
pub type SysSummary = {
    cpu float,              // 全机总体 CPU 使用率 (0.0 - 100.0)
    cpu_count int,          // 逻辑核心总数
    cpu_brand str,          // 处理器型号名称 (如 AMD Ryzen / Intel Core)
    mem_used_mb int,        // 物理内存已用量 (MB)
    mem_total_mb int,       // 物理内存总量 (MB)
    net_sent_kbs float,     // 发送速率 (KB/s，宿主差分采样)
    net_recv_kbs float,     // 接收速率 (KB/s，宿主差分采样)
    proc_count int,         // 当前系统总进程数
    os_name str,            // 操作系统名称 (如 Windows / Linux)
    os_version str,         // 操作系统版本号
    kernel str,             // 内核版本
    hostname str,           // 主机名
    uptime_s int            // 系统运行时间 (秒)
}

pub type ProcInfo = {
    pid int,                // 进程标识符
    name str,               // 进程可执行文件名
    cpu float,              // 进程 CPU 占用率 (0.0 - 100.0)
    mem_mb int,             // 进程物理内存常驻量 (MB)
    disk_kbs float,         // 进程磁盘吞吐速率 (KB/s)
    net_kbs float,          // 进程网络速率 (KB/s)
    status str,             // 运行状态 ("running" / "sleeping" / "stopped" / "idle")
    user str                // 启动用户
}

pub type DiskInfo = {
    name str,               // 磁盘名称
    mount str,              // 挂载点 (如 C:\)
    total_mb int,           // 磁盘总容量 (MB)
    avail_mb int            // 剩余可用空间 (MB)
}

pub type Snapshot = {
    summary SysSummary,     // 系统总览
    procs []ProcInfo,       // 实时进程列表 (降序前 512 条)
    disks []DiskInfo,       // 磁盘卷挂载列表
    users []str,            // 系统活动用户列表
    cores []float           // 各逻辑核心独立 CPU 使用率
}
```

### 2.2 接口声明
```auto
#[api(method="GET", path="/api/system/snapshot")]
pub fn system_snapshot() Snapshot

#[api(method="POST", path="/api/system/kill")]
pub fn kill_process(pid int) bool
```

---

## 3. 前端功能与页面架构

### 3.1 壳应用 (`src/front/app.at`)
- **左侧导航栏 (Nav Rail)**:
  - 品牌区：⚡ 图标 + "系统监视器" + "Task Manager"。
  - 4 个功能页切换项：进程 (`processes`)、性能 (`performance`)、详细信息 (`details`)、用户 (`users`)。
  - 底部状态灯：绿色小圆点 + "后端正常 (sysinfo)"；后端异常时红点 + "后端离线"。
- **顶栏控制**:
  - 当前页面名称与副标题。
  - 刷新间隔档位切换：250ms (快速) / 1s (常规) / 2.5s (省电)。
  - 暂停 / 继续实时轮询控制按钮。
- **Tick 分频机制**:
  - 基准 250ms 触发一次 `.Tick`，根据 `speedDiv` (1 / 4 / 10) 分频触发 `store.Refresh()`。
  - 声明主题播种状态 `dark_mode bool = true` 与 `accent_color str = "indigo"`，接收桌面及 os-config 注入。

### 3.2 进程页 (`src/front/processes.at`)
- **顶部 4 张 KPI 概览卡片 (Win11 风格)**:
  - CPU: 总使用率 (0.0 %) + 核心数与 CPU 型号。
  - 内存: `已用 / 总量 GB` + 物理内存 MB。
  - 磁盘: 实时磁盘吞吐总速率 (KB/s) + 挂载驱动器数。
  - 网络: 总吞吐 (KB/s) + 上行/下行独立速率。
- **操作栏 & 二次确认 (AlertDialog)**:
  - 选中状态栏：展示选中进程的 PID 与名称。
  - 「结束任务」按钮：未选中时禁用置灰；选中时高亮红色。
  - 二次确认弹窗 (AlertDialog)：标题提示 "结束任务确认"，描述明确目标进程名称与 PID。用户点击「结束进程」才执行 `kill_process(pid)`，点击「取消」或遮罩外点不杀。
- **6 列表格**:
  - 列：名称 / CPU % / 内存 / 磁盘 / 网络 / 状态。
  - 表头支持点击切换排序（名称、CPU、内存、磁盘、网络），附带 `↑` / `↓` 箭头指示。
  - 点击整行选中该进程。

### 3.3 性能页 (`src/front/performance.at`)
- **左侧 Mini Rail (1/4 宽)**:
  - 4 张点选卡片：CPU、内存、磁盘、网络，带指示色条与实时高亮边框。
- **右侧主图表区 (3/4 宽)**:
  - 大 SVG 实时面积曲线图 (560 × 134 视口，30 点历史滑窗)。
  - 刻度轴与虚线网格：100 / 75 / 50 / 25 / 0。
  - 4 图自适应定标：CPU (0-100%)、内存 (0-总物理内存)、磁盘 (自适应标尺)、网络 (自适应标尺)。
  - 子面板联动：
    - CPU 模式：展示所有逻辑核心 (如 16/20/32 核) 独立实时使用率网格与进度条。
    - 内存模式：物理内存总量、已用量、可用量构成详情。
    - 磁盘模式：各分区卷挂载点、容量、可用空间柱状展示。
    - 网络模式：网络接口、上行与下行实时速率。
  - 底部硬件规格卡片：系统名称、OS 版本、内核版本、计算机名、运行时间。

### 3.4 详细信息 & 用户页 (`src/front/details_users.at`)
- **详细信息页**:
  - 聚焦全量进程列表，默认按 PID 排序（支持 PID、名称、内存多维排序）。
  - 清晰展示 PID、进程名称、状态 Badge、所属用户、CPU 占用、常驻内存。
- **用户页**:
  - 用户聚合卡片：展示当前活跃登录系统用户。
  - 关联统计：统计各用户名下拥有的进程数量与内存消耗汇总。

---

## 4. 状态管理与持久化 (`sys_store.at`)

### 4.1 数据滑窗与几何
- 维护定长 30 点的历史滑窗：`cpuHist`、`memHist`、`diskHist`、`netHist`。
- 每次 `Refresh` 采样推进滑窗（先进先出），并实时构造 SVG Path：
  - 折线 Path: `M x1 y1 L x2 y2 ...`
  - 面积 Path: 折线 + `L 540 120 L 36 120 Z` (底闭合)

### 4.2 本地持久化 (Storage)
- 支持键名定版：
  - `sysmon.speed`: 刷新速度 ("fast" / "normal" / "slow")
  - `sysmon.sort_column`: 排序排序列 ("cpu" / "mem" / "disk" / "net" / "name" / "pid")
  - `sysmon.sort_dir`: 排序方向 ("asc" / "desc")
  - `sysmon.active_tab`: 当前导航页 ("processes" / "performance" / "details" / "users")
  - `sysmon.perf_tab`: 性能页子项 ("cpu" / "mem" / "disk" / "net")
- **向下兼容**: `.Init` 读取 `sysmon.*` 键；若不存在则回退读取旧版 `dash.*` 对应键，平滑无感升级。

---

## 5. 双端差异与特性

| 维度 | Vue 模式 (`auto run`) | VM 原生桌面模式 (`auto run -r vm`) |
|---|---|---|
| **执行引擎** | Vite + Vue 3 + Tailwind + shadcn-vue | AutoVM 字节码解释器 + Iced GUI 渲染引擎 |
| **数据通讯** | Vite 代理 `/api/*` 请求至 AutoVM HTTP 服务 (8025) | 进程内单体/IPC 调用本地 `auto.sys.*` |
| **表格悬浮态** | shadcn `hover:bg-muted/50` 浏览器原生 CSS | 明确点击选中高亮 (`bg-accent`) |
| **弹窗行为** | shadcn AlertDialog 规范浮层 | Aura ViewBuilder Popover (Modal placement) 二次确认 |
| **主题响应** | Tailwind `dark` class 根节点响应 | Iced 调色板 Palette + `dark_mode` 播种 |

---

## 6. 退役与升级说明

- **Mock 数据全面退役**：早期版本的随机游走算术生成函数及 12 条固定 Mock 假进程数组已全量删除。
- **真实系统守护**：通过 `sysinfo 0.33` 提供的原生采样及宿主差分，所有 CPU/内存/磁盘/网络数值与 Windows 任务管理器真实指标同量级对应。
