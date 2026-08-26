# 025-dashboard — regeneration SPEC

> Purpose: 系统监视器 dashboard——KPI 卡行 + CPU/内存/网络实时面积曲线 +
> 可排序进程表（Design 21 App 轨道填洞 ②，Plan 438 M1）。
> **Frontend-only, mock 数据源（前端随机游走），no backend。**
> 暗色对齐：shadcn 语义 token（bg-background/bg-card/text-muted-foreground），
> 脚手架 index.html 默认 `class="dark"`（Design 19 theming）。

## Functional spec (regenerate from this, no code)

单视图系统监视器（三区纵向布局）：

- **头部**: 标题 + 副标题；右侧刷新控制——三个间隔预设按钮
  （250ms / 1s / 2.5s，当前项高亮）+ 暂停/继续按钮。
- **KPI 卡行**: 4 张卡（grid-cols-4）——CPU（%）、内存（x.x / 32 GB +
  百分比）、网络（x.x MB/s）、进程数。值随刷新更新。
- **曲线区**: 3 张面积图卡片纵向堆叠（CPU 使用率 0-100%、内存 0-30 GB、
  网络 0-8 MB/s），每张卡片头有色块 + 标题 + 量程说明 + checkbox 独立开关
  （关闭即隐藏整卡）。图 = SVG 直通（442 A4）：静态网格线 + 轴线 +
  area path（fill-opacity 0.25）+ 顶边 line path。左侧固定刻度标签列。
- **进程表**: table 原语族（table/table-header/table-row/table-head/
  table-body/table-cell + badge）。列：名称 / CPU % / 内存 / 状态。
  名称、CPU、内存三列点击列头排序（升降切换，列头箭头指示）；
  状态列不排序（badge 展示：running=default / sleeping=secondary /
  stopped=outline）。

## Data model

```
interval int = 250        // 被 .Tick 机制取走作 setInterval 周期（基准拍）
running str ∈ {"true","false"}   // watch 门控启停；Init 置 "true" 触发启动
speedDiv int ∈ {1,4,10}   // 分频 → 有效刷新间隔 250ms/1s/2.5s
subTick int, tickN int

cpu int (3-97), memT int (80-300, 十分位 GB), netT int (0-80, 十分位 MB/s),
procCount int (142-158)
cpuLabel/memLabel/netLabel/procLabel str

cpuHist/memHist/netHist: List[int]，定长滑窗 30 点（Init 预填 → 步长恒定）
cpuLine/cpuArea/memLine/memArea/netLine/netArea str（path d 几何）

procs: List of { name str, cpu int (0-62), mem int (十分位 GB, 4-320), status str }
// mem 十分位存储：显示 /10 为 JS 精确浮点（"12.4"），<100 显示 *10 MB
procsView: 排序后视图（含显示串 cpuL/memL）
sortColumn str ∈ {"name","cpu","mem"}, sortDir str ∈ {"asc","desc"}
nameH/cpuH/memH str（列头标签 + ↑↓ 指示）
```

## 机制要点（可再生时必须保留）

1. **单 Tick 源分发**（plan §6 风险缓解）：一个 `.Tick`（250ms 基准）+
   `subTick >= speedDiv` 分频后统一推进全部数据源——不允许多 timer。
2. **随机游走为确定性算术**（`((tickN * k) % r) - d` 漂移 + clamp），
   vue 轨无 rand 映射（024 同款决策）；换真数据源后此节整体删除。
3. **几何/排序为 handler 内联重算**（模块级 fn 不进 vue SFC，
   437 §0.6.E-3）——与 024-charts 同款已知重复模式，445 同笔债务
   （"几何重算三份内联"，435 组件化后收敛）。
4. **排序 = 选择式重建**（扫描剩余取最优 + 重建剩余，无索引写）；
   name 列与值漂移无关 → Tick 内跳过重排。
5. **watch 门控启动**：`running` 初始 "false"、`.Init` 末尾置 "true"
   （生成的 watch(running) 无 immediate，需一次变更启动定时器）。
6. float 纪律（437 §0.6.D）：几何中间量显式 `float` 声明。

## Mock → 真数据源替换（接口形状，plan DoD）

换真后端时**前端零改动**（只换数据生产段）。`.Tick` 的 apply 分支中
"随机游走" 节替换为对下列 API 的调用（形状即当前 model 字段）：

```
// 单次轮询返回（一次请求带全部数据源 → 保持单 Tick 分发语义）
poll_system() -> {
    cpu: int,             // 聚合 CPU 百分比 0-100
    mem_used_tenths: int, // 已用内存，十分位 GB（142 = 14.2GB）
    net_tenths: int,      // 吞吐，十分位 MB/s
    proc_count: int,
    procs: [ { name: str, cpu: int, mem_mb: int, status: str } ]
}
```

- 进程 `mem_tenths` 为十分位 GB 整数（前端 memL 格式化：<100 显示
  `*10` MB，否则 `/10` GB）；`status ∈ {"running","sleeping","stopped"}`。
- 滑窗/几何/排序/显示串逻辑不变（消费同一形状）。
- M2 若引入 storage 持久化（018 先例），键名建议：
  `dash.speed_div` / `dash.sort_column` / `dash.sort_dir` /
  `dash.show_{cpu,mem,net}`。

## 已知边界（M1 记账，vue 轨生成器缺口）

1. **f-string 模型引用直插**：`f"${.cpu}"` 发出 `` `${cpu}` `` 而非
   `` `${cpu.value}` ``（vue-tsc TS2362）。规避：先提升局部变量
   （`var cL = .cpu`）。
2. **any/int 除法的 Math.trunc 发射不稳定**：record 字段（any）参与
   `/`、`%` 不加 `Math.trunc`（12.109375 直出）；且观察到同源代码
   不同构建间局部 int 推断的截断行为翻转。规避：**数值显示一律
   "十分位整数存储 + 单次 /10"**（JS 浮点精确到一位小数）或纯整数
   运算；不做 `%.十` 拼接小数。

## Verification

- `auto build --gen-only`（strict，零 S002）；
- gen 树 `npx vue-tsc --noEmit` 零错 + `pnpm build` 绿；
- 实机（浏览器/桌面）：KPI 随 Tick 变化、间隔预设生效、暂停冻结、
  三曲线独立开关、三列排序升降切换；
- M2（vm 模式）补 `tests/desktop_mcp.py`（013/038 惯例）。
