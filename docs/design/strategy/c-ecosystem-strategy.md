# C 生态战略：一个底座、三个消费面

**日期**: 2026-09-06
**状态**: Draft（战略纲领，作为 GOAL-013 / GOAL-006 后续拆 plan 的锚）
**关联**: [ecosystem-portfolio-strategy](ecosystem-portfolio-strategy.md)（伞形总战略，本文档为
其 §2.6 消费面）、[GOAL-013](../../specs/goals.md)（C 生态与嵌入式）、[GOAL-006](../../specs/goals.md)（consumer-mode parity）、
[consumer-parity-strategy](consumer-parity-strategy.md)（消费者 parity 母篇）、
[auto-as-rust-script-strategy](auto-as-rust-script-strategy.md)（a2r 轨母纲）、
[Design 06](../06-code-generation.md)（a2c/FFI 现状）、`raw/a2c-lvgl-analysis.md`（LVGL 素材）、
`raw/c.md`（C 互操作素材）

---

## 0. 战略一页（结论先行）

**一个底座 + 三个消费面**：

```
                ┌──────────────────────────────────────────┐
                │ 底座：a2c 全语法 + C 互操作（#[c] 绑定、   │
                │ 头文件摄入、FFI 边界所有权约定、freestanding）│
                └──────────────┬───────────────────────────┘
                               │（底座被三个消费面全量继承）
        ┌──────────────────────┼──────────────────────────┐
        ▼                      ▼                          ▼
 消费面一（先行）          消费面二                      消费面三（远期）
 Linux 系统库消费者         嵌入式（MCU）                 Linux 内核
 libc/POSIX/redis/SDL…     驱动/RTOS/LVGL UI            新驱动/新模块（LKM）
 旗舰：APUE/UNP 语料       旗舰：AutoUI × LVGL          定位：产普通 C 落 Kbuild
      → unix/net 标准库
      → WebServer
```

**排序**：消费面一先行（hosted Linux 方言最干净、反馈环最快、受众最广），消费面二分叉，
消费面三等前两条线磨出发射质量后进入。上一轮讨论曾裁定"嵌入式先行"，本文档以
"系统库消费者先行"取代之——理由见 §3.1。

**一条路线铁律**：C 是目标代码（source-to-source），不建 LLVM 后端、不做直接机器码发射；
任何消费面都不引入运行时/GC——编译期零开销安全是三个消费面共同的价值前提。

---

## 1. 现状盘点（2026-09-06）

### 1.1 已有资产

| 资产 | 现状 | 战略角色 |
|---|---|---|
| a2c 后端（`trans/c.rs`，4534 行） | C99 发射；函数/struct/enum/`use c <header>` 导入；`test/a2c/` 套件（Design 06 定性 "Mature" 指基础形态完整） | 底座主体；**深度约为 a2r（23472 行）的 1/5，未经历过 a2r 级多轮 parity 深化** |
| `stdlib/c/`（stdio.c.at 等） | 手写 `#[c]` 绑定 + 配套 .c/.h | 绑定层雏形与约定样板 |
| `auto-bindgen` | **Auto→C 方向**：从 Auto extern 声明生成 C 头文件/manifest；范围明确排除 C→Auto | 需扩展或另立工具补 C→Auto 摄入方向（§6.2） |
| 内存模型（move/hold/view） | 编译期、零开销导向（Design 04） | 三个消费面的核心价值主张 |
| a2r-std（http/json/task/str） | a2r 轨运行时库，plan 442 打通 serve/HTTP/SSE | 消费面一三臂 parity 的 a2r 臂依托 |
| parity 机器（`parity/`，30+ 库语料三方对拍） | 成熟方法论 | APUE/UNP 语料工程直接复用 |
| `raw/a2c-lvgl-analysis.md` + Design 06 未实现清单 | a2c+LVGL 已有分析素材与占位 | 消费面二旗舰的前期输入 |
| GOAL-013 / roadmap C 节 | "部分达成"；条目：全语法 a2c、宏/预处理、CTE-C、Linker 接管、OpenOCD、MCU 热重载、SDL3 | 本文将其重组为三消费面结构（§8） |

### 1.2 关键缺口

- a2c 语法覆盖未对标 a2r 的 parity 深度；无 freestanding/no-heap 模式；无 `#line` 调试发射。
- C→Auto 头文件摄入不存在（auto-bindgen 明确不做此方向）——消费面一的规模化前置。
- FFI 边界的所有权约定（谁分配/谁释放/borrow 表达）、errno→`May` 错误映射未成文。
- VM 侧 `#[c]` 绑定需要 host shim 双臂纪律——"脚本开发→转译发布"双形态在 C 轨的成立前提。

---

## 2. 路线判断：为什么"C 作为目标代码"

1. **工具链生态免费获得**：每颗 MCU 都有厂商 GCC/Clang 工具链、SDK、链接脚本。产出 C =
   第一天支持整个目标矩阵，并继承厂商工具链的功能安全认证（ISO 26262/IEC 61508 认的是 C
   编译器）。未来若产出 C 需过 MISRA-C 检查，可直接成为认证卖点。
2. **内核方向的政治豁免**：Rust-in-Linux 花了五年仍在为"工具链进内核"作战，因为 Rust 必须
   成为内核的真实依赖。Auto 源到源转译的产物就是普通 C——maintainer 用现有 gcc/clang 编译、
   checkpatch 审查。**这是 Auto 相对 Rust 在内核场景的结构性优势**。
3. **可读 C 是采用桥梁**：嵌入式团队可逐行检查产出、在仓库混放 `.auto` 与 `.c` 渐进采用。
   与 a2r 在 Rust 生态"脚本迭代、Rust 发布"的打法完全同构，组织记忆可复用。
4. **寄生期不建 LLVM 后端**：C 就是 IR，一个发射后端换整个目标矩阵；优化交给 C 编译器，
   Auto 只对发射质量（可读性/确定性/调试信息）负责。（阶段裁决——终局 native 直发必做，
   见 [ecosystem-portfolio-strategy](ecosystem-portfolio-strategy.md) §2.9 与
   [native-backend-strategy](native-backend-strategy.md)。）

---

## 3. 消费面一（先行）：Linux 系统库消费者

### 3.1 定位与排序理由

**定位**："像 Python 一样胶水消费 C 生态（libc/POSIX、hiredis、SDL3、ffmpeg…），但性能接
近 C"——三条消费面里受众最广的一张牌（Linux 服务端开发者 >> 嵌入式 >> 内核模块），且与
"AI 原生 + 脚本开发→转译发布"教义严丝合缝：开发期 VM 快迭代，发布期 a2c 出原生 C。

**排序理由（取代此前"嵌入式先行"的裁定）**：

- hosted Linux 的 libc/POSIX 是三种 C 方言里最干净的（比厂商 HAL 友好、远比内核 GNU 扩展
  全家桶友好）；无交叉编译/烧录/板级模拟，反馈环最快。
- 可直接复用 parity 机器，且能打出**三臂故事**：同一份 Auto 网络库代码，VM 直跑 /
  a2r+Tokio+Axum / a2c+libc+epoll 三后端对拍——多目标战略第一次在系统级负载上兑现。
- 产出的绑定层/绑定纪律/所有权约定/auto-bindgen 升格，被消费面二、三全量继承。
- 与既有 GOAL 的结构最顺：GOAL-006（consumer-mode parity，Draft）的 C 轨实例化，
  与 GOAL-003（Rust 消费）/GOAL-005（Python 消费）形成消费者三轨对称。

### 3.2 旗舰工程 A：APUE/UNP 语料复刻

复刻《UNIX 环境高级编程》（APUE）与《UNIX 网络编程》（UNP）两书的代码为 Auto——本质不是
"复刻两本书"，而是把 **POSIX 系统面当成系统性课程大纲**：文件 IO、进程/线程、信号、
管道/FIFO/共享内存/信号量 IPC（APUE）→ socket、TCP/UDP 服务器、epoll（UNP）。按章节推进
即强制成熟：libc 绑定层、errno→`May` 映射、struct 布局约定、指针生命周期约定。

- **产出物**：Auto 的 `unix` / `net` 标准库（对标 Python 的 os/socket、Go 的 os/net）——
  a2c 轨自己的标准库层（a2r 轨是 a2r-std，C 轨的优势是库层之下直调 libc、无运行时依赖）。
- **"复刻"的定义是现代化重实现**：以 epoll 为主轴（非 select/poll 时代顺序）、线程 API
  取舍一次定死、陈旧 API 的跳过清单在实施 plan 明确标注。不做逐行翻译。
- **方法论沿用 parity 套件**：语料即 `parity/` 风格的用例库，oracle 是等价 C（或 Rust）程序
  的行为对拍（应用语义层，见 §3.4）。

### 3.3 旗舰工程 B：net 网络库 + WebServer

分层定义，防范围失控（"类似 nginx 但比它丰富"是范围失控的经典入口）：

| 层 | 内容 | 来源 |
|---|---|---|
| `net` 层 | socket 封装、epoll 事件循环、定时器；暴露两个面——阻塞客户端 API（易用）+ 事件驱动服务端核心（性能） | UNP 收口产出 |
| HTTP 层 | HTTP/1.1 解析、静态文件、反向代理、TLS（OpenSSL 绑定） | `net` 层之上 |
| 运维面 | AutoUI 监控/管理台（QPS、连接、热点路由实时面板）——"比 nginx 丰富"的差异化打这里，不打 HTTP 功能堆叠 | UI 轨资产复用 |

**v1 明确不做**：HTTP/2、动态模块系统、配置 DSL 之外的可扩展机制。

**验收闸门用数字说话**：wrk 对拍 nginx 与 a2r/Axum 臂，性能预算写进 plan 验收标准——
"性能比 Python 好很多"必须落为可复现基准数字，旗舰才算完成证明任务。

### 3.4 三臂 parity 的口径

同一份 Auto 应用代码（用 `use auto.net` / `use auto.http`）：VM 臂调 native shim（Rust 实现）、
a2r 臂转译链 a2r-std（Tokio/Axum 底层）、a2c 臂直调 libc/epoll。三臂**底层库天然不一致**，
因此沿用 consumer-parity-strategy 的策略 B：**测应用语义层**（状态码/body/文件副作用），
不测传输字节级一致。这是对 consumer-parity-strategy §2.1"同一底层库"原则的三臂扩展口径，
需在其母篇补记。

### 3.5 双臂绑定纪律（规模化的生死线）

每个 `#[c]` 绑定要维持双臂：VM 侧 host shim（开发期 VM 直跑）+ a2c 侧直调。绑定规模上百
之后，这套纪律的手工成本决定项目可持续性——所以 **auto-bindgen 的 C→Auto 方向升格必须排
在语料工程最前面**（§6.2），先手写约定样板、再机器复制。

---

## 4. 消费面二：嵌入式（MCU）

**技术难点是"广"而非"深"**：厂商 HAL 多为标准 C99，难在目标矩阵分散、FFI 面大、调试链路长。

- **旗舰场景：AutoUI × LVGL**。"用 Auto 写声明式 UI，跑在 MCU 上"市面上没有第二家，直接
  复用 UI 轨最大投资；`raw/a2c-lvgl-analysis.md` 为前期输入，Design 06 未实现清单已占位。
  优先级高于 SDL3（SDL3 归属消费面一，见 §8）。
- **VM 即嵌入式模拟器**："脚本开发→转译发布"双形态映射到嵌入式：开发期宿主 VM 1 秒周转，
  板级补 QEMU（Cortex-M/RISC-V virt），CI 用 QEMU/Renode，发布期 a2c 交叉编译烧录。
  这是别的语言给不了的 dev-loop。
- **RTOS 与驱动**：FreeRTOS/Zephyr/RT-Thread（国内生态加分）经 `#[c]` 互操作接入；绑定由
  auto-bindgen 从厂商头文件生成（消费面一养出的产能）。
- **内存**：no_std/no-heap 模式——静态分配分析、显式 arena，绝不引入运行时。这是 Auto 对 C
  的核心价值主张，丢了它嵌入式就没意义。
- **调试链路**：`#line` 发射 + 名字映射使 gdb/OpenOCD 直接可用（roadmap 已列 OpenOCD 对接）；
  这是源到源路线必须提前做对的口碑项。MCU 热重载（GOAL-013 既有条目）在此消费面归属。
- **桥接市场**：嵌入式 Linux（buildroot/Yocto——有 C 编译器、没有 Rust 工具链的环境）是
  消费面一与二的天然交汇点，Auto 可作该市场的脚本层+应用层。

---

## 5. 消费面三（远期）：Linux 内核

**技术难点是"深"而非"广"**：单一目标（x86-64/aarch64），但内核 C 是地球上最难啃的方言——
GNU 扩展全家桶、inline asm、`__attribute__`、`container_of` 宏魔法、Kbuild 集成。

- **定位**：只做"新驱动/新模块的编写语言"，产出普通 C 落进 Kbuild（`.c` + Makefile 片段）；
  不做"用 Auto 重写内核"。切入点选 char device、sysfs、platform driver 这类小而标准的模块。
- **难点队列**（按依赖序）：GNU C 方言摄入（消费内核头文件）→ inline asm / `container_of`
  类宏的 Auto 侧建模（候选：trait/泛型映射）→ Kbuild 发射 → 内核编码风格合规的 C 产出。
- **向 Rust-in-Linux 学习**：他们被迫证明的每件事——无运行时开销、与内核内存/锁惯例互操作、
  长期维护承诺——Auto 同样要证明。唯一豁免是工具链接受度（§2.2），但"生成的 C 谁来维护"
  的质疑依然会来。对策：产出**惯用、可读、checkpatch 合规**的 C，模块保持小尺寸。
- **启动判据**：消费面一达到"WebServer 旗舰 benchmark 闸门过线"、消费面二达到"驱动 + LVGL
  应用"级别后再立项；两条线练出的方言覆盖/发射质量/调试链路对内核线复用率约八成。

---

## 6. 底座：三个消费面共享的关键工程

### 6.1 a2c 全语法 parity 补课

对标 a2r 曾享受的多轮深化待遇（plans 400/415/442 序列的 C 轨对应物），以 parity 方法论驱动：
语法覆盖差距清单 → 语料驱动的逐块收敛。这是 Stage 0 的主体。

### 6.2 C 互操作三层演进

| 层 | 形态 | 时机 |
|---|---|---|
| L1 手写 `#[c]` 绑定 | 现状（stdlib/c 样板）——继续作为约定样板 | 已有 |
| L2 头文件自动生成绑定 | **auto-bindgen 扩展 C→Auto 方向**（其当前范围声明"不做 C→Auto"需修订）或另立工具；产出 `#[c]` 声明 + VM shim 骨架 | Stage 0 末/Stage 1 初，**语料工程前置** |
| L3 GNU 方言摄入 | 内核头文件所需扩展集 | 消费面三 |

### 6.3 FFI 边界所有权约定（成文）

- errno → `May<T>` 错误映射规则；struct 布局/对齐声明；变长参数边界。
- 所有权跨边界：哪侧分配/释放、borrowed buffer 用 `view`/`str`（len+data）表达的约定——
  这是 Auto 区别于一切 C 绑定方案的差异化点，须与 Design 04 ownership 体系衔接成文。

### 6.4 freestanding 模式与调试发射

no_std/no-heap 语义档位（消费面二/三需要，消费面一不阻塞）；`#line` + 名字映射发射规范。

---

## 7. 排序与里程碑

| 阶段 | 内容 | 闸门 | 产出 |
|---|---|---|---|
| **Stage 0 底座** | a2c parity 补课；FFI 所有权约定成文（§6.3）；auto-bindgen C→Auto 升格（§6.2 L2）；freestanding 骨架 | a2c 语料套件绿；从 `<stdio.h>` 生成绑定并双臂跑通 hello | 绑定机器 + 约定文档 |
| **Stage 1 消费面一** | APUE 语料 → `unix` 库；UNP 语料 → `net` 库；WebServer 旗舰（v1 范围 §3.3）；三臂 parity 套件 | **wrk 对拍 nginx 与 a2r/Axum 臂的预算线**；APUE/UNP 章节覆盖矩阵 | `unix`/`net` 标准库 + WebServer + 基准报告 |
| **Stage 2 消费面二** | 厂商 HAL 自动绑定；FreeRTOS/RT-Thread 示例；AutoUI×LVGL 后端；QEMU/Renode CI 档位；OpenOCD/#line | QEMU Cortex-M/RISC-V 点灯→LVGL demo；测试档位（如 `tc`）入库 | 嵌入式工具链 + LVGL 旗舰 demo |
| **Stage 3 消费面三** | GNU 方言摄入；Kbuild 发射；首个 LKM（char device） | LKM 在真内核树外编译加载运行 | 内核模块样板 + 方言覆盖清单 |

Stage 1 与 Stage 2 可部分并行（底座就绪即分叉），Stage 3 严格后置（§5 启动判据）。

---

## 8. 与目标账本 / roadmap 的映射

- **GOAL-013** 重组为本文档的三消费面结构：roadmap C 节条目重新归属——"全语法 a2c、宏/预处
  理、CTE-C"→底座；"Linker 接管、OpenOCD、MCU 热重载"→消费面二；"SDL3 图形框架"→消费面
  一（桌面 SDL 游戏/图形，与嵌入式线无关）。GOAL-013 的描述文字待 Stage 0 立项 plan 时更新。
- **GOAL-006**（consumer-mode parity，Draft）：本文档 §3 是其 C 轨实例化内容；其"同一底层库"
  原则在三臂下按 §3.4 口径扩展。
- **GOAL-003**（a2r 轨）：WebServer 旗舰的 a2r 臂与 benchmark 对照直接反哺该线；三臂 parity
  是 GOAL-003 与本文档的交汇点。
- 消费者三轨对称成型：Rust 消费（GOAL-003/004）/ Python 消费（GOAL-005）/ C 消费（本文档）。

---

## 9. 风险与不做的事

**风险**：

1. 绑定规模爆炸——双臂纪律手工成本（对策：§6.2 L2 前置，见 §3.5）。
2. FFI 边界安全空洞——raw C 调用击穿所有权保证（对策：§6.3 约定 + 检查器规则）。
3. WebServer 范围失控（对策：§3.3 v1 边界 + 数字闸门）。
4. 内核线"生成代码维护性"质疑（对策：§5 惯用 C + 小模块）。
5. 三臂 parity 底层不一致被误读为"行为不一致"（对策：§3.4 口径在母篇成文）。

**不做**：并行 LLVM 后端；第一天消费任意内核头文件（先手写声明、后自动生成——Rust 早期
同路径）；任何场景引入运行时/GC；双垂直同时推进（底座先行、分叉有闸门）。

---

## 10. 待决策点

1. **`unix`/`net` 库层归属**：纯 Auto 写（复刻语料沉淀，双臂共享语义）+ 底层薄 `#[c]` 绑定
   ——本文档倾向此案；备选是各臂独立实现（快但 parity 漂移）。
2. **auto-bindgen 扩展 vs 另立工具**：C→Auto 方向放 auto-bindgen（改其范围声明）还是新
   crate。Stage 0 立项时定。
3. **WebServer TLS 选型**：a2c 臂 OpenSSL 绑定 vs 裸 TLS 库；a2r 臂维持 rustls/reqwest 生态。
4. **三臂 parity 基线**：以现有 `parity/` runner 扩第三臂，还是新 runner。建议扩展现有。
5. **消费面三启动判据的量化**（§5）在 Stage 1/2 收官时回填。

---

## 11. 下一步

> **优先级裁决（2026-09-06）**：近期交付主线为 Rust 生态（GOAL-003/004）与 AutoUI
> （GOAL-007/009/010），C 方向**暂缓启动工程**——本文档以 Draft 状态作为定向锚留存，
> 不立项、不占用 trans/UI 轨带宽。重启触发条件（满足其一即在季度规划时重新评估）：
> ① Rust 线到达稳定闸门（GOAL-003 消费者侧收口或 GOAL-004 语料里程碑）；
> ② 出现真实外部拉力（具体嵌入式/内核项目或客户需求）；
> ③ UI 轨产生 MCU/嵌入式 Linux 落地需求（AutoUI×LVGL 交点）。

1. 触发条件满足后，本文档过用户终审，Stage 0 立项（L1 plan：a2c parity 补课清单 +
   FFI 约定成文 + auto-bindgen 升格裁决）。
2. consumer-parity-strategy 母篇补记 §3.4 三臂口径。
3. GOAL-013 描述文字随 Stage 0 立项 plan 更新为新结构。
