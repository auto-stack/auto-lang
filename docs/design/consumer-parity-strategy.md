# Consumer-Mode Parity Strategy — Auto 作为库消费者

**日期**: 2026-07-21
**状态**: Draft（设计文档，待实施）
**关联**: `docs/design/auto-as-rust-script-strategy.md`（母纲）、`docs/plans/359-auto-as-rust-script-rollout.md`（Phase E/F）

---

## 0. 为什么需要这份文档

现有 parity 套件（`parity/libs/`，241/241 L1）验证的是 **"Auto 作为库实现者"** 的能力——Auto 自己重实现 base64/serde_json/regex 等库的逻辑，对比原生 Rust crate。这证明"Auto 能写出和 Rust 等价的库代码"。

但开发者和 AI **最常用的模式不是写库，而是用库做应用**：用 Auto 调 HTTP 客户端下载文件、调 JSON 库处理数据、调文件系统读写、甚至调 sqlite/redis 做后端。这是 **"Auto 作为库消费者"** 的能力——验证"Auto 调用外部能力的应用代码，行为是否与等价 Rust 应用一致"。

这两种模式的 parity 含义不同：

| | 实现者模式（现有 241 用例） | 消费者模式（本文档规划） |
|---|---|---|
| Auto 角色 | 重实现库的内部逻辑 | 调用库的 API 做应用 |
| 代码形态 | `use auto.<mylib>: fn`（本地重实现） | `use auto.http: get` / `use auto.fs: read`（调 stdlib 能力） |
| Rust oracle | 调真实 crate（base64 crate、serde_json crate） | 调同一底层库（std::fs、serde_json） |
| 验证点 | Auto 的计算逻辑 == Rust crate 的计算逻辑 | Auto 应用的 IO/副作用行为 == Rust 应用的 IO/副作用行为 |
| 典型场景 | 重写 base64 编解码 | 写一个文件下载器、JSON 处理器、CLI 工具 |

**消费者模式才是普通开发者和 AI 最常用的**——它直接关系"用 Auto 写应用是否可靠"。

---

## 1. 调研结论：当前能力图景

### 1.1 Auto 调用外部能力的机制

Auto 有三条路径调用 Rust 能力：

| 路径 | 机制 | 状态 |
|---|---|---|
| **`use auto.<module>`**（主路径） | Auto stdlib 声明 → VM native shim 实现 / a2r 转 `a2r_std::` | VM 端覆盖广（~200 函数），a2r 端部分覆盖 |
| **`use.rust <crate>::<path>`**（动态 FFI） | `RustFfiBridge` + libloading 加载 cdylib | **严重受限**：仅 marshal 原语类型，无法传 opaque handle（Connection/Statement 等） |
| **a2r 转译内联** | a2r 把 `auto.X` 转 `a2r_std::X`，链接 `a2r-std` crate | 独立 crate（ureq）与内联（reqwest）**底层库不一致** |

### 1.2 能力域 × 双端底层库一致性（parity 可行性的决定因素）

| 能力域 | VM 底层 | a2r 底层 | 一致？ | 消费者 parity 可行性 |
|---|---|---|---|---|
| **fs（文件/目录）** | std::fs | std::fs | ✅ | **高** |
| **json（编解码/查询）** | serde_json | serde_json | ✅ | **高** |
| **time / env / math / str** | std | std | ✅ | **高** |
| **process（spawn/exec）** | std::process | std::process（shell） | ✅ | **中高** |
| **http client** | reqwest 0.12 | ureq 2（独立 crate）/ reqwest（内联） | ❌ | **中**（需统一底层或限语义层） |
| **http server** | 自研（tokio+手写） | 缺失 | ❌ | 低 |
| **net/tcp（裸 socket）** | std::net | 缺失 | ❌ | 低（a2r 缺） |
| **sqlite** | 未实现 | 未实现 | — | **不可行**（双端缺） |
| **redis** | 未实现 | 未实现 | — | **不可行** |
| **crypto/hash** | 未实现 | 未实现 | — | 只能走复刻模式 |

### 1.3 关键阻塞

- **DIV-HTTP-LANG-1**（stdlib `auto/http.at` 解析 bug）：任何 `use auto.http` 在 VM 和 a2r 都失败。修复计划在 Plan 359 Phase E Task E4。**所有 HTTP 消费者场景的前置依赖。**
- **opaque-handle 库**（sqlite/redis/hyper）：`use.rust` 无法传 Connection/Statement 等 opaque handle（DIV-RUSQLITE-1）。消费者 parity 不可行，除非先实现 `auto.sqlite`/`auto.redis` 的纯 Auto 包装（大工程）。
- **a2r 独立 crate 用 ureq，VM 用 reqwest**：HTTP 传输层细节（重定向/cookie/TLS/chunked）可能分歧。消费者 parity 需把范围限在"应用语义层"（状态码/body 内容），或统一底层库。

---

## 2. 消费者 parity 套件设计

### 2.1 设计原则

1. **三方对比同一底层库**：AutoVM 调 `auto.X`、a2r 转译调 `a2r_std::X`、Rust oracle 直调底层 crate。三者底层应是**同一个 Rust 库**（如都用 std::fs），否则传输层差异污染对比。
2. **测应用语义，不测传输细节**：消费者 parity 关注"调用是否正确达成应用目的"（文件写进去了？JSON 解析对了？状态码对？），而非"传输字节是否逐位一致"。
3. **确定性优先**：用例必须避免外网、时间戳、随机等非确定性。网络用 in-process mock server，时间用相对值，文件用临时目录。
4. **按可行性分层**：先做底层库已一致、无阻塞的（fs/json/process），再做需修 bug 的（http），最后是有重大缺口的（db）。

### 2.2 三层用例规划

#### Layer 1: 立即可行（底层库一致，无阻塞）

这层用例 VM 和 a2r 底层都是 std/serde_json，无需任何前置修复。

| 用例 ID | 应用场景 | 调用能力 | 对标 Rust | 确定性策略 |
|---|---|---|---|---|
| **C-fs** | 文件读写器：写文本→读回→统计行数 | `auto.fs`（read_text/write_text/exists） | std::fs | 临时目录，固定内容 |
| **C-json-app** | JSON 配置处理器：解析→查询→修改→序列化 | `auto.json`（parse/encode/get/keys） | serde_json | 固定 JSON 输入 |
| **C-cli** | CLI 参数解析器：读 argv→分派命令 | `auto.process`（args）+ `auto.fs` | std::env::args + std::fs | 固定 argv 模拟 |
| **C-env** | 环境变量工具：get/set/遍历 | `auto.env`（get/set/get_or） | std::env | 固定 env 值 |
| **C-text** | 文本批处理器：读文件→正则替换→写文件 | `auto.fs` + Auto 自实现 regex（复用 parity/libs/regex） | std::fs + regex crate | 临时文件，固定模式 |

**用例数量估计**：每个 10-20 个 TAP 断言，5 个用例 ≈ 60-100 个新 L1 用例。

#### Layer 2: 需修 DIV-HTTP-LANG-1 后可行

这层依赖 Phase E Task E4（修 `Type.method` 解析）完成。修复后 `use auto.http` 可用。

| 用例 ID | 应用场景 | 调用能力 | 对标 Rust | 确定性策略 | 风险 |
|---|---|---|---|---|---|
| **C-http-get** | HTTP GET 客户端：请求 mock server→读 body | `auto.http`（get） | reqwest/ureq | in-process mock server，固定响应 | 中（VM reqwest vs a2r ureq） |
| **C-http-post** | HTTP POST 客户端：发 JSON→读响应 | `auto.http`（post） | reqwest/ureq | mock server echo | 中 |
| **C-wget** | 简易文件下载器（wget 风格）：URL→存文件 | `auto.http`（download）+ `auto.fs` | reqwest + std::fs | mock server 提供固定文件 | 中 |
| **C-crawler** | 简易 web 爬虫：GET→解析链接→递归 | `auto.http` + `auto.json`/文本 | reqwest + 解析 | mock server 固定页面 | 中高 |

**用例数量估计**：每个 8-15 个断言，4 个用例 ≈ 40-60 个新用例。**但受 HTTP 底层库不一致影响，可能只能达到"应用语义层一致"而非字节一致。**

#### Layer 3: 需重大前置工程（暂列路线图）

这层底层实现缺失，需先补齐 `auto.X` 模块。暂不实施，记录为 L3 路线图。

| 用例 ID | 应用场景 | 缺失能力 | 前置工程 |
|---|---|---|---|
| **C-sqlite-client** | sqlite 客户端：open→exec→query | `auto.sqlite` 模块（VM+a2r 都缺） | 实现 `auto.sqlite` 纯 Auto 包装（调 rusqlite via FFI 或重实现） |
| **C-redis-client** | redis 客户端：connect→set/get | `auto.redis` 模块（双端缺） | 实现 `auto.redis` 模块 |
| **C-http-server** | HTTP 服务器：路由→handler→响应 | a2r 侧 `http.server` 缺失 | a2r_std 补 server 模块（或用 axum 生成） |

---

## 3. 技术方案：消费者 parity 怎么测

### 3.1 复用现有 parity 框架

`parity/crates/auto-parity/` 的三方对比模型（AutoVM / a2r / native Rust）直接适用。每个消费者用例就是一个 `parity/libs/<name>/` 库，结构不变：

```
parity/libs/c_fs_app/
├── README.md
├── auto/c_fs_app.at          # Auto 应用代码：use auto.fs，做文件操作
├── tests/auto/*.at           # Auto 测试：TAP 断言文件操作结果
├── tests/rust/               # Rust oracle：用 std::fs 做同样操作，产相同 TAP
│   ├── Cargo.toml
│   └── tests/c_fs_app.rs
└── mock/ (若需)              # in-process mock（HTTP server 等）
```

### 3.2 确定性策略（关键）

消费者用例涉及 IO/网络/进程，必须保证三方确定性：

- **文件操作**：每方用独立临时目录（`std::env::temp_dir()` + 唯一子目录），写入固定内容，读回断言。不依赖共享文件系统状态。
- **HTTP**：in-process mock server（固定端口如 18081，回固定响应）。三方依次跑（parity runner 已是顺序），共享同一 mock server 实例。需给 runner 加 setup/teardown hook 启停 mock（http_client_sync 骨架已设计此机制）。
- **进程/环境**：避免真 spawn 外部进程。测 `process.args` 时由 parity runner 传入固定 argv；测 env 时 set 固定值后读回。

### 3.3 Rust oracle 的写法

消费者用例的 Rust oracle **不调 Auto stdlib**，而是直调底层 Rust 库。例如 C-fs 的 oracle：

```rust
// parity/libs/c_fs_app/tests/rust/tests/c_fs_app.rs
fn write_and_read(path: &str, content: &str) -> String {
    std::fs::write(path, content).unwrap();
    std::fs::read_to_string(path).unwrap()
}
// 产出与 Auto 侧相同的 TAP：ok N - test_write_read
```

三方对比的是 **TAP 输出**（应用行为结果），不是字节级源码。

### 3.4 HTTP 底层不一致的处理

C-http-* 系列面临 VM(reqwest) vs a2r(ureq) 的底层差异。两种策略：

- **策略 A（推荐）**：统一 a2r-std 的 HTTP 底层为 reqwest（与 VM 一致）。需改 `crates/a2r-std/src/http.rs` 从 ureq 换 reqwest。这是根本解法，但影响所有 a2r HTTP 转译代码。
- **策略 B（折中）**：parity 范围限"应用语义层"——mock server 回固定 200 + 固定 body，三方都断言状态码和 body 内容，不测重定向/cookie/chunked 等传输细节。当前 http_client_sync 骨架用的就是此策略。

建议先走策略 B（快速出 L1），策略 A 作为后续优化。

---

## 4. 与 Phase E 的依赖关系

```
Phase E (修复缺口)          Phase F (消费者 parity)
─────────────────────       ─────────────────────────
E4: DIV-HTTP-LANG-1 ──────► Layer 2 (C-http-*/C-wget/C-crawler)
                              解锁
E5: DIV-A2R-CHAR-AT-1        (无直接依赖，但 Layer 1 的文本处理若用 char_at 会遇)
(独立)                        
Layer 1 (C-fs/C-json/...) ◄── 无前置，立即可做
```

**Layer 1 完全独立**，可在 Phase E 之前/并行启动。Layer 2 必须等 E4。

---

## 5. 实施路线图

| 阶段 | 内容 | 前置 | 产出 |
|---|---|---|---|
| **F1** | Layer 1: C-fs + C-json-app | 无 | ~30-40 新 L1 用例（fs+json 消费者） |
| **F2** | Layer 1: C-cli + C-env + C-text | 无 | ~30 新 L1 用例（process/env/文本消费者） |
| **F3** | Phase E Task E4（修 DIV-HTTP-LANG-1） | — | 解锁 HTTP 消费者 |
| **F4** | Layer 2: C-http-get + C-http-post | F3 + http_client_sync 激活 | ~20-30 新用例（HTTP 客户端语义） |
| **F5** | Layer 2: C-wget + C-crawler | F4 | ~20-30 新用例（组合应用） |
| **F6** | Layer 3（路线图） | sqlite/redis 模块实现 | 远期 |

**关键路径**：F1→F2（立即可做）‖ F3（Phase E）→ F4→F5。

---

## 6. 价值论证

### 为什么消费者 parity 比实现者 parity 更重要

现有 241 用例证明"Auto 能写出等价 Rust 的库代码"。但开发者和 AI 90% 的时间是**调用**库做应用，不是**重写**库。消费者 parity 直接回答："我用 Auto 写一个文件下载器/JSON 处理器/CLI 工具，a2r 转译后的 Rust 行为和我 VM 跑的一致吗？"——这才是"Auto 是 Rust 脚本层"宣传点的**日常验证**。

### 对宣传叙事的补充

母纲的三段式叙事（Dev/Ship/Bridge）在消费者模式下更有说服力：

> "AI 用 Auto 写了一个文件下载器（`use auto.http` + `use auto.fs`），VM 里秒级验证下载逻辑正确。发布时 a2r 转成 Rust，调 reqwest + std::fs，行为完全一致。这就是'脚本迭代、Rust 发布'的日常形态。"

这比"Auto 重实现了 base64 编解码"更贴近开发者直觉。

---

## 7. 待决策点

1. **Layer 2 的 HTTP 底层不一致**：走策略 A（统一 reqwest，根本解）还是策略 B（限语义层，快速出 L1）？建议先 B 后 A。
2. **Layer 3 是否在本文档范围**：sqlite/redis 消费者需要先实现底层模块（大工程），建议列为远期路线图，不在当前 Plan 359 范围。
3. **用例颗粒度**：每个消费者用例是做成"单一应用"（如完整 wget）还是"能力矩阵"（fs 的每个方法一组断言）？建议前者——更贴近真实应用，宣传价值更高。
4. **mock server 基础设施**：http_client_sync 已设计 runner setup/teardown hook，但未实现。Layer 2 需要先实现这个 hook（~20 行 runner 改动）。

---

## 8. 下一步

本文档作为设计纲领。实施时：
1. 先在 `docs/plans/359-auto-as-rust-script-rollout.md` 加 **Phase F**（引用本文档），按 F1→F2→...→F5 拆 task。
2. F1/F2 可立即用 worktree 方式实施（无前置依赖）。
3. F3 与 Phase E Task E4 合并（同一修复）。
4. F4/F5 等 F3 完成后实施。
