# PLAN-813 T-01 决定与基线探针

日期：2026-10-19  
执行树：`D:/autostack/.wt/lang-813/auto-lang` (`plan-813-dev`)  
作者树：`D:/autostack/.wt/lang-813/book` (`plan-813-book`)  
auto-lang 计划基线：`30c1b9a7edc3112cac99089d14169d9f9bdeaeb6`  
book 基线：`d7a71a7fb1fa42ddd26d6cec859715ed98161f7a`  
auto-down 只读基线：`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`

## 页面映射

在执行树通过 `AUTO_BOOK_ROOT` 指向同组 book 树运行 `website/scripts/prepare-content.js`，随后统计生成页：

| 语言 | 有效章节/附录页 | CodeView | AutoFence | Run 总入口 |
|---|---:|---:|---:|---:|
| 英文 | 27 | 152 | 48 | 200 |
| 中文 | 27 | 151 | 48 | 199 |
| 合计 | 54 | 303 | 96 | 399 |

与静态原稿中的 401 个 Run/锁定位置相差两处。英文 `tapl/ch01-getting-started.md:319` 和中文 `tapl/ch01-getting-started.cn.md:242` 都是 `<Listing number="1-2" file-name="auto.toml">`，来源 `tapl/listings/ch01/listing-01-02/auto.toml`。生成器把其内容当 Auto 代码查找，输出 `Listing source not found`，所以并未生成 Run。此例应保留 TOML 静态展示，元数据声明 `display-only`；页面门禁需断言 Run 不存在。TAPL 原稿已列出静态风险和源 hash，见同目录 `static-inventory.csv`。

## 公共入口探针

| 探针 | 入口和输入 | 观察结果 | 修复约束 |
|---|---|---|---|
| ch03 上下文 | CLI 运行 `tapl/listings/ch03/listing-03-01/main.at` | 退出码 0，输出 `Hello, Alice!` 与 `Hello, Bob!` | 公共 CLI 能运行完整 Listing；展示页只含 `greet("Alice")` 时因 `greet` 未定义失败。网页需传显式完整 setup，同时实际执行编辑后的片段。 |
| ch05 输入 | CLI 编译 `tapl/listings/ch05/listing-05-01/main.at` | `input` 未定义；ch05 `listing-05-04/main.at` 先因 `none` 未定义终止 | 修复真实调用/类型并用请求输入跑通；验证合法、非法、空行和 EOF。 |
| ch10 多文件 | CLI 运行 `tapl/listings/ch10/listing-10-05/main.at` | `Module not found: utils`；目录仅有 main.at、pac.at，无 `utils`/network 文件 | 在作者源中补齐完整工程文件，页面和 Notes 都运行同一份文件树。已知 `examples/playground-demo/09-multi-module` 经 CLI 可成功解析兄弟模块。 |
| stdin 行为 | 用公共 CLI 跑 `use auto.io; io.read_line()` 探针，分别输入空 LF 行、CRLF `hello`、立即 EOF | CRLF 转为 `hello`；空行与 EOF 都得到 `""` | 不替换全局 stdin；请求输入在 VM/请求级隔离，LF/CRLF 去行尾；新 Optional API 区分空行与 EOF；旧 CLI `read_line` 保持兼容。 |

输入探针的临时源码/结果保存在 auto-lang 执行树忽略的 `target/plan813-probes/`，不进入交付。第一次 CLI 编译使用同树中的 ignored `Cargo.lock`（从主检出复制 Oct-18 锁文件），后续 Rust 检查均须 `--locked`。

## 接口与元数据决定

1. 执行合同作者源：`book/tapl/examples.json`。各入口用 locale 范围内稳定 `id` 与共享 `pair_id` 对应中英版本，记录书页路径、Listing/围栏锚点、模式、完整项目文件、入口、输入、预期类别与值/诊断、超时。Listing 使用 `example-id` 属性；裸 `auto` 围栏使用 Markdown info 属性 `example-id`。同一作者合同驱动页面转换、Run UI、Playground Notes 和全源 runner。
2. 模式显式为 `run`、`test` 或 `display-only`。未登记或重复 ID 是构建/审计错误；没有因 `use`/`import` 正则推断运行资格的回退。TOML 属于 `display-only`。负控删除一条 descriptor 或改坏一个 oracle，必须让审计失败。
3. Run 页面请求增加输入字段。每次请求拥有独立的输入行队列并绑定对应 AutoVM；无输入也作为显式 EOF，不得落回 Playground 进程 OS stdin。`io.read_line_opt() -> Option<str>` 返回空行 `Some("")`，EOF 返回 `None`。CLI 旧 `io.read_line()` 继续读 OS stdin。Playground run 默认超时 10 秒并使用既有 1–60 秒后端限幅；资源超限返回清楚错误而不静默截断。
4. 新增 Playground `/api/test`，真实运行 `#[test]` 并返回 executed/pass/fail 明细；测试 VM 必须使用同一请求截止时间。原 `test_code` 公共调用保留既有无截止语义，新增带 deadline 入口供受限宿主使用。
5. 官网 CodeView 使用同源 `/api/run`，使用 RunResponse 的结构化结果并展示 HTTP/API 错误；不将 `localhost:3030` 编进静态站点。项目文件继续复用每请求临时目录并清理；执行路径不改全局 cwd/env。

## 现存证据边界

本记录只证明上述路径、现状和设计决定，不证明 399 个入口已修复。具体源/页面代码 hash、oracle、执行结果、超时和清理证据由 T-02..06 新增全源清单和 runner 生成；生成网站目录是忽略产物，不作为作者源修改。
