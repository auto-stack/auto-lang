这是一个极具里程碑意义的重构！将 `am.exe` 融入 `auto.exe`，标志着 Auto 语言正式从一个“附带工具链的语言”蜕变成了一个**“全天候、多范式的超级工程中枢”**。

在设计支持多后端的通用 CLI 时，最忌讳的就是**“命令的笛卡尔积爆炸”**。如果我们保留 `capp`、`clib` 的设计思路，那么未来有了 Rust、Vue 和 Python，你的帮助菜单里就会塞满 `rsapp`, `rslib`, `vueapp`, `pyapp`……这会让用户感到极其混乱。

最优雅的现代 CLI 架构哲学是：**“让 CLI 变笨（只保留动词），让配置文件 `pac.at` 变聪明（决定名词和后端）”**。

下面是我为你重新设计的 `auto.exe` 命令行交互规范。它完美吸收了 Cargo 和 Go CLI 的工业级审美，同时兼容了 Auto 特有的跨语言编译与硬件调度能力。

---

### 全新的 `auto.exe` CLI 架构图谱

在终端输入 `auto --help` 后，用户将看到这样一个极度克制、语义清晰的界面：

```text
-------------------------------------------------------
AutoNexus / Auto CLI v1.0.0
The Universal Build Coordinator & Language Environment
-------------------------------------------------------
Usage: auto [OPTIONS] <COMMAND> [ARGS]

Execution:
  (no args)    Enter ASH / Auto REPL interactive mode
  <file.at>    Run an Auto script directly via AutoVM

Project Creation:
  new          Create a new Auto project (app, lib, gear, gadget)
  init         Initialize an Auto project in the current directory

Build & Run:
  build        Compile the project based on pac.at backend (Rust/C/Vue/etc.)
  run          Build and run the executable/dev-server
  clean        Remove the .auto/build directory and artifacts

Dependencies:
  add          Add a dependency to pac.at
  fetch        Fetch and resolve all dependencies (Replaces scan/pull)
  deps         Show the dependency graph

Hardware & Embedded:
  device       Manage connected hardware devices and ports (List/Select)

Project Utils:
  info         Show package, backend, and target information
  open         Open the current project in the default IDE
  
Environment:
  upgrade      Upgrade auto.exe toolchain to the latest version
  env          Manage global AutoMan configurations and cache (Replaces reset/install)

Options:
  -h, --help     Print help
  -V, --version  Print version

```

---

### 核心重构逻辑与设计亮点

让我们来拆解一下，那些旧的命令是如何被完美收编的，以及新范式下应该如何操作。

#### 1. 收编 `capp`, `clib`：模板化 `auto new`

绝不能把后端语言写在顶级命令里。我们引入 `--template` (或 `-t`) 机制。
当你输入 `auto new my_project` 时，默认创建一个纯 Auto 的应用。如果需要混合工程：

* **C 嵌入式工程**：`auto new sysmon_cdd -t c-app` (生成包含 C 编译配置的 `pac.at`)
* **Rust 桌面工程**：`auto new my_desktop -t rs-app`
* **Vue Web 工程**：`auto new web_dash -t vue-app`
* **ASH 智能体组件**：`auto new my_tool -t gadget`

#### 2. 收编 `auto vue` 和 `am build`：大一统的 `auto build`

以前你需要手动敲 `auto vue`，这破坏了“一次编写，多端转译”的承诺。
现在，所有的构建命令只有一句：**`auto build`**。
`auto.exe` 会读取当前目录的 `pac.at`：

* 如果 `backend: "rust"` $\rightarrow$ 静默生成 `.rs`，并在后台调起 `cargo build`。
* 如果 `backend: "c"` $\rightarrow$ 静默生成 `.c`，并在后台调起 `ninja`。
* 如果 `backend: "vue"` $\rightarrow$ 将 AURA 视图树转化为 Vue 组件，并调起 `vite build`。

#### 3. 收编 `scan` 和 `pull`：合并为 `auto fetch`

依赖管理的原则是“要么不理我，要么一次性搞定”。
`scan`（扫描依赖）和 `pull`（下载依赖）在用户的实际心智中是一回事：“把我的工程跑起来需要的库弄好”。因此统一合并为 `auto fetch`。并且，在执行 `auto build` 时，系统应当静默执行一次轻量级的 `fetch`。

#### 4. 收编 `devices` 和 `port`：收口到 `auto device` 子命令

因为你的生态横跨了“高层次的 Web/桌面”和“底层的嵌入式 C”，如果把硬件端口相关的命令放在顶级，会让 Web 开发者感到困惑。
将其收口到子命令是最优雅的：

* `auto device list` (替代原 `devices`)
* `auto device select COM3` (替代原 `port`)

---

### 旧命令到新命令的无缝映射表 (Migration Guide)

| 以前的调用方式 (am.exe / auto.exe) | 现在的优雅调用方式 (auto.exe) | 备注说明 |
| --- | --- | --- |
| `auto.exe` (无参数) | **`auto`** | 完美保留，进入超级终端 ASH |
| `auto.exe script.at` | **`auto script.at`** | 完美保留，JIT 运行脚本 |
| `auto.exe vue` | **`auto run`** 或 **`auto build`** | 取决于 `pac.at` 中的 `backend: "vue"` 配置。`run` 会启动 Vite 的 Dev Server。 |
| `am app <name>` | **`auto new <name>`** | 默认 template 为 auto application |
| `am capp <name>` | **`auto new <name> -t c-app`** | 使用 C 混合工程模板 |
| `am scan` / `am pull` | **`auto fetch`** | 统一获取所有依赖库，包括 C/Rust 的 FFI 库 |
| `am devices` / `am port` | **`auto device`** | 将硬件相关操作收口到子命令体系 |
| `am reset` / `am install` | **`auto env reset` / `install**` | 将环境清理和缓存安装归档到 `env` 子命令 |

### 总结：大道至简

这套 CLI 设计让你彻底摆脱了“每支持一种新语言/新框架，就要修改 CLI 顶级菜单”的噩梦。未来的 `a2py` (转 Python) 只需要在 `auto new` 里加一个模板，并在编译器的内部加一个 Router 即可，对外暴露的命令行界面（CLI）永远保持这一份极简和稳定。

结合我们刚才聊的 ASH（超级终端）和 Gadget，这个 `auto.exe` 实际上就是开启整个 Agent OS 生态的一把“万能钥匙”。