# Auto as Rust's Script Layer — 宣传与文档策略纲领

**日期**: 2026-07-17
**状态**: Draft（待用户 review）
**作者**: ZCode brainstorming session
**性质**: 高层策略纲领（high-level strategy framework）。本文档是后续 4 个子项目各自详细 spec 的**母纲**，不直接进入实施。

---

## 0. TL;DR（一页纸摘要）

下一版本的核心宣传点定为：

> **"Auto 是 Rust 的脚本层。AI 生成 Auto 脚本做秒级迭代（跳过编译），发布时 `a2r` 转译成 Rust 短代码拿到原生性能与内存安全。编译器保证两种模式行为一致。开发效率、运行效率、安全——一个语言全占。"**

本纲领围绕这个宣传点，规划 4 个相互咬合的子项目：

| 子项目 | 代号 | 核心问题 | 角色 |
|---|---|---|---|
| 英雄演示 + 核心叙事 | **A** | 如何在 10 秒内让人看懂"脚本即 Rust" | 门面 |
| "From Script to Ship" 互动教程 | **B** | 如何系统性地教会用户这套工作流 | 旗舰 |
| VM↔A2R 一致性验证体系 | **C** | 如何证明"一致性"不是空头支票 | 地基 |
| Rust 生态用例库 | **D** | 如何覆盖真实世界的 Rust 用例 | 弹药 |

**推荐产出顺序**：C（地基）→ D（弹药）→ B（旗舰）→ A（门面）。但 A 可先做最小版探路。

**核心基调（本次 brainstorming 已确认）**：
1. **主角是 AI-Native 开发流**，不是语言本身。AI 不是"生成工具"配角，而是闭环核心。
2. **只承诺现有证据**，不超前宣传。发现 `conformance_tests.rs` 名不副实（只跑 VM 一侧），真正的三向对比只在早期 `parity/` workspace。这一点必须诚实处理。
3. **受众优先级：Rust 开发者 > AI 应用开发者 > Python/动态语言开发者 > 企业决策者**。Rust 开发者最难说服但也最有杠杆。
4. **产出形态：完整策略纲领**。后续 4 个子项目各出独立 spec。

---

## 1. 背景与现状盘点

### 1.1 我们手里已经有什么（别从零开始）

| 已有资产 | 位置 | 对本纲领的意义 |
|---|---|---|
| **12 章 Language Tour + 57 个可运行 `.at`** | `docs/tour/ch01..ch12` + 每章同名目录 | 旗舰教程的**形式骨架**已存在。但**主题是"Auto 语言入门"（hello/types/functions/.../async/interop），与"Script→Ship 工作流"主题不同**。形式可复用，内容需新建。 |
| **`<Listing>` → `<CodeView>` 构建管线** | `website/scripts/prepare-content.js` | 网页内每个代码块可直接 Run。但 **`CodeView` 当前只接 `/api/run`（VM 执行），从未接 `/api/trans`（Rust 转译那一侧）**——这正是 B 子项目要补的核心缺口。 |
| **Playground 后端能力齐全** | `crates/auto-playground/src/routes/` | `/api/run`、`/api/trans`（rust/c/python/js/ts）、`/api/agent-debug/*`、多文件项目执行。**后端很强，但 tour 几乎没用上**。 |
| **a2r 转译器覆盖面广** | `crates/auto-lang/src/trans/rust.rs` (~12,400 行) + `test/a2r/`（~159 例 + 34 例运行时） | 能产出真正的 `trait`/`impl`/`Box<dyn>`/泛型/const 泛型/所有权借用/`?`/`!` 错误传播/多文件模块。覆盖面是宣传的底气。 |
| **`parity/` 三向对比框架** | `parity/crates/auto-parity/` | 真正的"VM↔a2r↔原生 Rust"对比，分 p1-p4 阶段（base64/url → serde_json/regex → sha2/rusqlite → reqwest/tokio）。但**仍处早期**：带 `DBG355` 调试打印、只 ~8 个库、p3/p4 不完整。 |
| **`a2r-std` 运行时库** | `crates/a2r-std/` | 转译后 Rust 代码链靠它（serde/serde_json/ureq）。是 Ship 一侧的"标准库桥"。 |
| **8 个 playground-demo 示例** | `examples/playground-demo/` | VM/a2r/a2c 全 ✅ 的最小矩阵，可做 A 的起点。 |
| **bilingual website** | `website/`（VitePress, EN + 简体中文） | 国际化基础已具备，纲领要求所有产出做 EN/CN 双语。 |

### 1.2 必须先正视的两个"可信度漏洞"

宣传越响，这两个洞越致命。**纲领要求在 C 子项目里先堵上，再大范围宣传。**

#### 漏洞 1：`conformance_tests.rs` 名不副实

文件 `crates/auto-lang/src/tests/conformance_tests.rs` 头部注释写着：
> "Tests that AutoVM and a2r produce identical output"

但实现（`run_conformance_test`，22-45 行）**只跑 AutoVM 一侧对比 golden 文件**，a2r 那一侧被明确标注 `"(Future)"`，**根本没有实现**。

**影响**：如果有人 grep 仓库找"一致性证据"，会发现这个测试，以为已经验证了 VM↔a2r 一致性，结果一读代码发现是空的。这对 Rust 社区的杀伤力极大——会被当成"虚假宣传"的实锤。

**处理**（C 子项目核心任务）：
- 短期：要么补全 a2r 一侧（transpile → cargo build → run → 对比），要么**修改文件头注释**，诚实标注当前只验证 VM-vs-golden。
- 中期：把真正的三向对比收敛到 `parity/` 框架，让 `conformance_tests.rs` 要么退役、要么明确降级为"VM 输出回归测试"。

#### 漏洞 2：a2r 测试覆盖的薄弱区

a2r 测试总量可观，但有几个关键 Rust 模式**覆盖稀薄**，正是 Rust 开发者会盯着问的地方：

| 薄弱区 | 现状 | 风险 |
|---|---|---|
| **async/await** | 仅 3 个 trivial 案例（`~T` → `async fn`），无 `.await` 链、无 async trait、无 `select!` | tokio 生态是 Rust 半壁江山，这是最显眼的短板 |
| **generators** | 仅 1 个案例（`yield` → `~Iter<int>`） | |
| **trait 高级特性** | 仅 3 个 spec 测试，无 trait 默认方法、无关联类型、无带 bound 的泛型 trait impl | trait 是 Rust 灵魂，覆盖不足会被质疑 |
| **API server 路由** | 仅 2 个案例（parse + 最小 server） | axum/actix 生态 |
| **纯 Rust 输出（无 a2r-std 依赖）** | 仅 1 个案例 | 影响"产出的 Rust 够不够 Rusty"的观感 |

**处理**（D 子项目核心任务）：把这些薄弱区变成 `parity/` 的 p1-p4 用例库，既是测试用例，也是教程素材，也是宣传弹药。

### 1.3 与 Python+C/C++ 的对比论点（宣传的核心对照）

这是说服 Rust 开发者之外的受众（尤其是 Python 开发者）的关键论点。要在 A/B 子项目里反复强化：

| 维度 | Python + C/C++ | Auto + Rust（a2r） |
|---|---|---|
| 生态关系 | **两个分裂的生态**（PyPI vs C/C++ 生态，FFI 是桥但断层明显） | **一个生态**（Auto 完整支持 Rust 所有编程模式 + std + 三方库，a2r-std 做桥） |
| 语言能力对等性 | Python 缺类型/缺所有权/缺零成本抽象，C/C++ 缺 GC/缺 ergonomics | Auto 与 Rust **能力对等**（同样的 trait/泛型/所有权/async） |
| 迁移成本 | Python → C/C++ 是**完整重写工程**，需 AI 大量介入 | Auto → Rust 是**机械转译**，编译器保证行为一致 |
| 行为一致性保证 | 无（Python 和 C 的行为经常不一致，尤以数值/并发/内存为甚） | **有**（编译器层面，C 子项目提供证据） |
| AI 辅助 | Python 好生成，但 C/C++ 重写是另一座山 | Auto 好生成（脚本模式可丢弃=适合 AI 试错），转 Rust 是确定性步骤 |

**核心 punchline**（A 子项目的 hero 文案候选）：
> "Python taught the world that fast iteration wins. Rust taught the world that safety wins. Auto refuses to choose."

---

## 2. 叙事定位（AI-Native 开发流为主角）

### 2.1 一句话定位

> **Auto 是 AI-Native 时代的 Rust 脚本层。同一份代码，开发期当脚本秒级迭代，发布期 `a2r` 转译成 Rust。AI 在闭环里做生成与验证，编译器在转译时保证行为一致。**

### 2.2 三段式故事线（贯穿所有文档/教程/演示）

```
        ┌─────────────────────────────────────────────────────────────┐
        │   AI-Native 开发闭环（主角）                                 │
        │                                                             │
        │   ┌─────────┐    秒级反馈    ┌──────────┐                   │
        │   │ AI 生成  │ ────────────> │ AutoVM   │                   │
        │   │ Auto 脚本│ <──────────── │ 执行脚本  │                   │
        │   └─────────┘   修正/迭代    └──────────┘                   │
        │         │                              │                    │
        │         │   脚本模式 = 可丢弃 = 适合 AI 试错                   │
        │         ▼                              ▼                    │
        │   ┌──────────────────────────────────────┐                  │
        │   │  a2r 转译（确定性、机械）              │  <── Bridge      │
        │   │  Auto → Rust + a2r-std                │      编译器保证   │
        │   └──────────────────────────────────────┘      行为一致    │
        │         │                                                   │
        │         ▼                                                   │
        │   ┌──────────────┐                                          │
        │   │  Rust 发布版  │  <── 性能 + 内存安全                      │
        │   │  (cargo build)│                                          │
        │   └──────────────┘                                          │
        └─────────────────────────────────────────────────────────────┘

        Dev（开发效率） ∩ Ship（运行效率 + 安全） ∩ Bridge（一致性保证）
                          = "不可能三角"
```

- **Dev 段**：AI 生成 Auto，AutoVM 解释执行，改完即刷新，跳过编译。AI 在闭环里通过 Playground/MCP 做生成-验证-修正。**卖点：开发效率**。
- **Ship 段**：`a2r` 把同一份代码转成 Rust 短代码，链上 `a2r-std`，cargo build 出原生性能 + 内存安全。**卖点：运行效率 + 安全**。
- **Bridge 段**：编译器层面保证 VM 输出 == Rust 输出。C 子项目提供自动化证据（parity 仪表盘）。对比 Python+C/C++ 的"两个生态、二次重写"困局。**卖点：一致性 = 不重写**。

"不可能三角"（开发效率 / 运行效率 / 安全）就是这三段的交集，是所有宣传的视觉锚点。

### 2.3 AI 为什么是闭环核心（而非"生成工具"配角）

这一点决定了 A/B 子项目的重心。AI 不是"帮你写第一版代码然后人接管"的工具，而是**持续运行在闭环里**：

1. **生成**：AI 写 Auto（脚本模式容忍 AI 的不完美——错了就丢，重新生成）。
2. **验证**：AI 通过 Playground API（`/api/run` 秒级执行）或 MCP（`auto mcp` 子命令）拿到执行结果，自己判断对错。
3. **迭代**：基于反馈重写。脚本模式的"无编译"特性让这个循环可以是秒级的，而 Rust 直接写的话每个循环都要等编译。
4. **冻结**：AI 和人都满意后，`a2r` 转译是确定性步骤（不需要 AI 介入，或仅需 AI 做格式微调），编译器兜底一致性。

**对 Rust 开发者的说服点**：这不是"用 AI 替代你写 Rust"，而是"用 AI 在脚本层快速试错，最终交付的还是你能审计的 Rust"。Rust 开发者最在意"交付物是否仍是可审计的、零成本抽象的 Rust"——答案是肯定的，a2r 产出就是 Rust。

### 2.4 为什么 Rust 开发者是首要受众

- **最难说服**：他们对"披着 X 皮的 Y 语言"高度警惕，会盯着问"产出的 Rust 够不够 Rusty"、"trait 动态分发是不是真的 Box<dyn>"、"所有权/借用有没有被偷偷退化"。
- **最有杠杆**：一旦他们认可，就是技术可信度的金字招牌。Rust 社区的 endorsement 会自动辐射到其他受众。
- **说服策略**：A/B/C/D 四个子项目都要为 Rust 开发者准备"硬核证据层"——不是营销话术，而是可 grep 的代码、可跑的 parity 测试、可审计的 a2r 输出。

---

## 3. 四个子项目的纲领

每个子项目后续会有独立 spec。这里只给定位、范围、关键决策点、依赖关系。

### 3.1 子项目 A：英雄演示 + 核心叙事（门面）

**定位**：10 秒内让人看懂"脚本即 Rust"，30 秒内让人想试。

**核心交付物**：
1. **一个杀手级 hero demo**：同一个 Auto 程序，左边"VM 跑（秒级）"，右边"a2r 转 Rust 后跑"，输出完全一致。视觉上并排、实时、可点。候选题材（D 子项目喂素材）：
   - HTTP 服务（最直观，Rust 开发者熟悉 axum 场景）
   - JSON 处理（serde 对标）
   - 并发任务（tokio/spawn）
2. **落地页叙事文案**：三段式（Dev/Ship/Bridge）+ "不可能三角"可视化 + 与 Python+C/C++ 的对比表（§1.3）。
3. **"为什么不是 Python"对照页**：核心论点展开（§1.3 的表 + punchline）。
4. **诚实的"现状声明"**：明确标注哪些场景已 parity 验证（链接到 C 的仪表盘），哪些是"路线图上"。

**关键决策点（留待 A 子项目 spec）**：
- hero demo 选哪个题材？（HTTP 服务 vs JSON 处理 vs 并发）——取决于 D 子项目先做通哪个 parity 用例。
- 是否做一个"AI 生成 Auto"的实时演示（比如在落地页放一个 AI 对话框，现场生成 Auto 并跑）？——AI 作为闭环核心的叙事需要这种"看得见的 AI"，但成本和稳定性风险高。
- 落地页是新建独立页还是改造现有 `website/index.md`？

**依赖**：吃 D 的最佳 parity 用例做 hero 题材；引用 C 的仪表盘做可信度链接。

### 3.2 子项目 B："From Script to Ship" 互动教程（旗舰）

**定位**：系统性地教会用户"AI 生成 → VM 迭代 → a2r 发布 → 一致性验证"这套工作流。这是教育资产，不是营销页。

**核心决策：新建主题化 tour，而非改造现有 `docs/tour/`**

现有 `docs/tour/` 是"Auto 语言入门"（12 章：hello/types/.../async/interop），**主题不同**。强行改造会破坏现有语言教程。纲领建议：
- **保留** `docs/tour/` 作为"语言参考教程"（它做得很好）。
- **新建** `docs/script-to-ship/`（或 `docs/workflow-tour/`）作为"工作流教程"，主题是 Dev/Ship/Bridge 三段式。
- 两者在导航上区分："语言是什么"（tour）vs"怎么用语言干活"（script-to-ship）。可交叉引用。

**核心交付物**：
1. **扩展 `<CodeView>` 组件**（关键技术债，B 的前置）：
   - 当前 `CodeView` 只调 `/api/run`，**必须接入 `/api/trans`**，让 Rust tab 里的代码是真实 a2r 转译的产物（而非手写 `.expected.rs`）。
   - 加一个"并排运行对比"模式：左 Auto VM 跑、右 Rust 编译跑，输出并排，一致则绿勾。这是 Bridge 段的视觉化。
   - 多文件支持（project_dir/files），让 modules 章节能真实跑成项目。
2. **Script-to-Ship tour 章节**（初版建议 6-8 章，素材来自 D）：
   - Ch1: Hello, Script & Ship（最小闭环：hello world 的 VM 跑 + a2r 转 + 一致）
   - Ch2: AI 在闭环里（演示 AI 生成-验证-迭代，强调脚本模式为何适合 AI）
   - Ch3: 类型与所有权（struct/enum/所有权/borrow，对应 a2r 的 `&`/`&mut`/move）
   - Ch4: 错误处理（`!`/`.?` → Rust `Result`/`?`）
   - Ch5: trait 与泛型（spec → trait/impl/Box<dyn>，const 泛型）
   - Ch6: 模块与项目（多文件，a2r 产出 Cargo.toml + 多 .rs）
   - Ch7: async 与并发（`~T` → async fn，tokio 场景）—— **依赖 D 的 async parity 用例**
   - Ch8: 发布（a2r 命令行、链 a2r-std、cargo build、性能对比）
3. **导航/进度系统**：上/下章、"在完整 Playground 打开"、进度追踪。
4. **双语**（EN + 简体中文）。

**关键决策点（留待 B 子项目 spec）**：
- `<CodeView>` 扩展的 API 设计：是加新 props（`transpile`/`compare`）还是新建 `<ScriptShipView>` 组件？
- "并排运行对比"的后端：是否需要新的 `/api/compare` 端点（一次调用同时跑 VM + a2r 编译 + 对比）？还是前端串行调两次？
- 是否在 tour 里嵌入"AI 对话框"让用户现场让 AI 改代码？（与 A 的决策呼应）
- 章节数与深度：初版 6-8 章是否够？哪些章节依赖 D 的 parity 用例必须等？

**依赖**：吃 D 的用例做章节素材；引用 C 的仪表盘做"一致性已验证"的背书；扩展 CodeView 需要后端 `/api/trans`（已存在）+ 可能新增 `/api/compare`。

### 3.3 子项目 C：VM↔A2R 一致性验证体系（地基）

**定位**：让"编译器保证行为一致"这句话有可审计的证据。这是整个宣传的可信度地基。**建议作为第一个深入的子项目。**

**核心交付物**：
1. **堵上漏洞 1**：诚实处理 `conformance_tests.rs`：
   - 选项 a：补全 a2r 一侧（transpile → cargo build → run → 对比 stdout）。
   - 选项 b：**改注释，降级为"VM 输出回归测试"**，把真正的三向对比统一收敛到 `parity/`。
   - 纲领倾向 b（避免两套并行的一致性框架），但具体由 C 子项目 spec 决定。
2. **强化 `parity/` 三向对比框架**：
   - 当前 p1-p4 阶段（base64/url → serde_json/regex → sha2/rusqlite → reqwest/tokio）。
   - 补全 p3/p4（sha2/rusqlite/reqwest/tokio 的 async 场景）。
   - 清理 `DBG355` 调试打印。
   - 明确"差分模糊测试"（differential fuzzing）策略：已有 `conformance_differential_*` 测试（50 seed 随机程序），需扩展到三向。
3. **公开 parity 仪表盘**（关键可信度资产）：
   - 一个网页（或 CI artifact），展示：通过用例数 / 总数、按 Rust 模式分类的覆盖面、已知 diverge 项（`parity/docs/known-divergences.md` 的可视化）。
   - 这是 A/B 子项目做"一致性已验证"背书的链接目标。
4. **CI 门禁**：parity 测试成为 PR 合并的硬门禁（至少 p1/p2 阶段）。

**关键决策点（留待 C 子项目 spec）**：
- `conformance_tests.rs` 的去留（补全 vs 降级退役）。
- parity 仪表盘的形态：CI artifact（静态 HTML）vs 实时网页 vs 嵌入 website 的页面。
- 差分模糊测试的 seed 策略和覆盖面目标。
- CI 门禁的渐进策略：先 p1/p2 硬门禁，p3/p4 警告？

**依赖**：无前置依赖（可立即开始），是 D/B/A 的地基。

### 3.4 子项目 D：Rust 生态用例库（弹药）

**定位**：一组真实场景例，每例都"VM 可跑 + a2r 可转 + parity 已验证"。一份代码三用：parity 测试用例 + tour 教程素材 + 宣传弹药。

**核心交付物**：覆盖真实 Rust 生态的用例矩阵。建议按 `parity/` 的 p1-p4 阶段组织，并针对 a2r 薄弱区（§1.2 漏洞 2）重点补强：

| 用例类别 | 候选例子 | 对标 Rust 生态 | 补强的薄弱区 |
|---|---|---|---|
| **HTTP 服务** | 最小 REST API、JSON endpoint | axum/actix | API 路由（仅 2 例） |
| **HTTP 客户端** | GET/POST、JSON 解析 | reqwest | async（仅 3 例） |
| **数据库** | sqlite CRUD | rusqlite/sqlx | p3 阶段 |
| **序列化** | JSON/serde、Atom 格式 | serde/serde_json | p2 阶段 |
| **async/并发** | tokio spawn、channel、select! | tokio | **async（最大短板）** |
| **正则** | 模式匹配、提取 | regex | p2 阶段 |
| **加密/哈希** | sha256、hmac | sha2/hmac | p3 阶段 |
| **CLI 工具** | 参数解析、文件处理 | clap/std::fs | 纯 Rust 输出（仅 1 例） |
| **trait 高级** | 默认方法、关联类型、带 bound 泛型 | std::iter 等 | **trait 高级（仅 3 例）** |
| **generators** | 迭代器、惰性序列 | std::iter | generators（仅 1 例） |

**每个用例的三重身份**：
1. **parity 测试用例**（喂给 C 的 `parity/` 框架）。
2. **tour 教程素材**（喂给 B 的章节）。
3. **宣传弹药**（喂给 A 的 hero demo 和对照页）。

**关键决策点（留待 D 子项目 spec）**：
- 用例库放哪？建议 `parity/libs/<name>/`（与现有 parity 框架一致）+ 在 `examples/` 下做面向用户的副本。
- 用例的"难度梯度"：每个类别是否都要有 minimal/realistic/full 三档？
- async 用例的 parity 对比策略（已有 sorted TAP 处理非确定完成顺序，是否够用）。
- 与 a2r-std 的关系：哪些用例需要扩展 a2r-std（比如 rusqlite 的绑定）？

**依赖**：与 C 并行（D 产出用例，C 提供验证框架）；D 的产出喂给 B 和 A。

### 3.5 子项目依赖图与产出顺序

```
    ┌─────────────────────────────────────────────┐
    │  C: 一致性验证体系（地基）  <── 无前置，最先开始  │
    │  堵漏洞1 + 强化 parity/ + 仪表盘 + CI 门禁    │
    └──────────────────────┬──────────────────────┘
                           │ 提供验证框架
                           ▼
    ┌─────────────────────────────────────────────┐
    │  D: Rust 生态用例库（弹药）  <── 与 C 并行/咬合   │
    │  HTTP/DB/serde/async/regex/crypto/CLI/trait  │
    └──────────────────────┬──────────────────────┘
                           │ 提供用例（测试+教程+弹药）
              ┌────────────┴────────────┐
              ▼                         ▼
    ┌─────────────────────┐   ┌─────────────────────┐
    │ B: 互动教程（旗舰）  │   │ A: 英雄演示（门面）  │
    │ script-to-ship tour │   │ hero demo + 落地页   │
    │ + 扩展 CodeView     │   │ + 叙事 + 对照页      │
    └─────────────────────┘   └─────────────────────┘
              ▲                         ▲
              └────────────┬────────────┘
                           │ A 可引用 B 的教程
                           │ B 可引用 A 的 hero
                           ▼
                    （互相引用，可并行收尾）
```

**推荐顺序**：
1. **C 先行**（堵漏洞 + 地基，1-2 个迭代周期）。
2. **D 与 C 并行**（D 产用例，C 验证；async/trait 等薄弱区是重点）。
3. **B 跟进**（CodeView 扩展是前置技术债；章节素材来自 D）。
4. **A 收尾**（hero demo 用 D 的最佳用例；引用 B 教程和 C 仪表盘）。
5. **A 最小版可前置探路**：在 C 还没全做完时，先用现有 8 个 playground-demo 做一个最小 hero，验证宣传点是否打动人。

---

## 4. 证据策略（只承诺现有证据）

这是本次 brainstorming 明确的基调，单列一节强调。

### 4.1 三级证据成熟度

所有对外文案（A/B 子项目产出）对"一致性"的措辞，必须按证据成熟度分级：

| 级别 | 含义 | 措辞模板 | 当前适用范围 |
|---|---|---|---|
| **L1 已验证** | 有 parity 测试通过（VM↔a2r↔原生 Rust 三向对比通过） | "已通过一致性验证" + 链接仪表盘 | C 完成后的 p1/p2 用例 |
| **L2 VM 稳定** | 有 VM-vs-golden 回归测试（当前 conformance/ 的 34 例） | "AutoVM 行为已回归测试覆盖" | 当前 conformance/ 全部 |
| **L3 路线图** | 计划支持但尚未验证 | "路线图上，预计 vX.Y" | async 高级、p3/p4 用例 |

**规则**：A/B 文案里**禁止**对 L3 场景使用 L1 措辞。这是诚实底线，也是规避"虚假宣传"反噬的护城河。

### 4.2 "现状声明"模块（强制）

A 的落地页和 B 的 tour 首页，必须有一个显眼的"现状声明"模块，明确：
- 哪些 Rust 模式已 L1 验证（列表 + 链仪表盘）。
- 哪些是 L2（VM 稳定但 a2r 一致性待验证）。
- 哪些是 L3（路线图）。
- `parity/docs/known-divergences.md` 的已知差异项（诚实公开）。

**这是对 Rust 社区的尊重**：他们能 grep、能读测试、能跑 parity。主动公开边界比被发现隐瞒强一百倍。

### 4.3 漏洞 1 的对外处理

在 C 子项目堵上 `conformance_tests.rs` 之前：
- **不在任何对外文案里引用 `conformance_tests.rs` 作为一致性证据**（避免误导）。
- 对外引用一致性证据时，只指向 `parity/` 框架（即使它早期），并诚实标注"框架建设中，p1/p2 已覆盖"。

---

## 5. 里程碑与节奏（建议）

| 里程碑 | 周期（建议） | 完成标志 | 解锁 |
|---|---|---|---|
| **M0: 纲领取认** | 本文档 review | 本纲领被接受 | 4 个子项目可各自立项 |
| **M1: C 地基** | 1-2 迭代 | 漏洞 1 处理完毕；parity p1/p2 CI 门禁；parity 仪表盘 v1 上线 | D 可大规模产用例；A/B 可引用 L1 证据 |
| **M2: A 最小版探路** | 与 M1 并行 | 用现有 playground-demo 做 hero + 落地页叙事 v1（只声明 L2） | 验证宣传点是否打动人 |
| **M3: D 用例库 v1** | 2-3 迭代 | HTTP/serde/regex/CLI 四类用例 L1 验证通过 | B 有素材；A 可升级 hero |
| **M4: B 旗舰 v1** | 2-3 迭代 | CodeView 扩展完成；script-to-ship tour 6-8 章 | 完整教育路径上线 |
| **M5: D 用例库 v2** | 与 M4 并行 | async/trait 高级/DB/crypto 补强 | 补齐薄弱区 |
| **M6: A 正式版** | 1 迭代 | hero 用 L1 用例；对照页 + 现状声明完整；双语 | 宣传物料就绪 |
| **M7: 发布** | — | 4 子项目齐备，parity 仪表盘公开，对外宣传 | — |

**关键路径**：M1（C）→ M3（D v1）→ M4（B）→ M6（A 正式版）。M2（A 最小版）是并行探路，不阻塞关键路径。

---

## 6. 风险与对策

| 风险 | 影响 | 对策 |
|---|---|---|
| **R1: Rust 社区质疑"披着皮的玩具"** | 高。Rust 开发者是首要受众，质疑杀伤力大 | C 的 parity 仪表盘 + 主动公开 a2r 输出（Rust 开发者能直接看转译出的 Rust 够不够 Rusty）；A 的对照页用真实 a2r 输出而非手写 |
| **R2: async 生态覆盖不足被发现** | 高。tokio 是 Rust 半壁江山 | D 子项目重点补 async parity（p4 阶段）；L3 场景诚实标注路线图 |
| **R3: parity 测试暴露大量 diverge** | 中。可能动摇"一致性"宣传 | 诚实公开 known-divergences；区分"行为一致"与"完全等价"（数值精度、panic 信息等次要差异可接受）；C 子项目定义"可接受差异"边界 |
| **R4: a2r-std 能力不足（如 rusqlite 绑定缺失）** | 中。限制 D 用例范围 | D 子项目评估每个用例对 a2r-std 的需求；必要时扩展 a2r-std（纳入 D spec） |
| **R5: AI 闭环演示不稳定** | 中。A/B 若嵌入实时 AI 演示，可能现场翻车 | 演示用预录 + 缓存策略；实时演示标注"实验性"；MVP 先用静态对照 |
| **R6: 双语工作量翻倍** | 低。拖慢节奏 | EN 先行，CN 跟进；机器翻译 + 人工校对；不阻塞 M0-M3 |
| **R7: CodeView 扩展引入前端复杂度** | 中。可能拖延 B | 新建 `<ScriptShipView>` 组件而非改 `CodeView`（隔离风险）；先做单文件，多文件后置 |

---

## 7. 待用户 review 的关键判断点

本纲领在以下点上做了判断，请 review 时确认或修正：

1. **主角是 AI-Native 开发流**（已 brainstorming 确认）——AI 是闭环核心，非配角。
2. **只承诺现有证据**（已 brainstorming 确认）——L3 不许用 L1 措辞。
3. **受众优先级 Rust 开发者第一**（已 brainstorming 确认）。
4. **新建 `docs/script-to-ship/` tour，而非改造现有 `docs/tour/`**（纲领建议）——避免破坏现有语言教程。
5. **推荐产出顺序 C→D→B→A，A 最小版可前置探路**（纲领建议）。
6. **`conformance_tests.rs` 倾向降级退役，真正一致性收敛到 `parity/`**（纲领倾向，具体由 C spec 定）。
7. **强制"现状声明"模块**（纲领要求）——A 落地页和 B 首页必须公开 L1/L2/L3 边界。
8. **`<CodeView>` 扩展建议新建 `<ScriptShipView>` 而非改原组件**（纲领倾向，具体由 B spec 定）。

---

## 8. 下一步

本纲领被接受后：
1. 选择第一个深入的子项目出独立 spec（建议从 **C** 开始，因为它是地基且无前置依赖）。
2. 每个子项目 spec 走标准 brainstorming → spec → writing-plans 流程。
3. 4 个子项目的 spec 全部就绪后，统一排期实施。
