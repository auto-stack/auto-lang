# TAPL 全书 Run 问题分析与修复边界

2026-10-19；PLAN-813 r1；这是**静态全量普查和实施建议，不是逐块运行验收报告**。

## 核查范围与数量

实际作者源在 `D:/autostack/book/tapl/`，网站 `website/books/` 和 `website/zh/books/` 是生成物。
book 基线 `d7a71a7fb1fa42ddd26d6cec859715ed98161f7a`；当前 book WIP 是另一本 Rust 教程的 9 个金样，不能覆盖。
auto-lang 取材基线 `99d1bda7ea5e8371555a1ff5dc915000dbd5ebf1`；主检出另有其他会话文档 WIP。

- 27 个页面（ch00–22、附录 A–D）× 中英 = **54 页**。
- 全部原始围栏 **1829 个**，包含 Auto、其他目标语言、命令、配置与输出。
- 网站将 **305 个 Listing** 整段替换成 CodeView；其中 303 个包含 Auto 围栏，2 个是配置清单。
- 替换后还剩 **96 个裸 Auto 围栏**，故共有 **401 个 Run 位置**（EN 201、ZH 200）。
- 实际独立 `main.at` 为 **152 个**，中英共用。不能用 399 个 Auto 原始围栏数代替网站实际入口数。
- 发现 **17 处**正文 Auto 与实际 Listing 字节内容不同；其中可能包括注释或排版差异，必须逐项语义核对，不能全部算作代码错误。
- 本轮 **0 次运行**；所有 Auto 记录的 `runtime_status=not_run`。没有静态风险标签的块也不能算通过。

[逐块清单](static-inventory.md) 给出所有 Run 的源位置；[CSV](static-inventory.csv) / [JSON](static-inventory.json) 同时保留其余所有围栏、hash 和被替换关系。[扫描脚本](static_inventory.py) 只读书稿，在指定证据目录写出清单。

## 已由代码直接确认的共性根因

| ID | 证据与影响 | 处理建议 |
|---|---|---|
| F-01 | `CodeView.vue::runCode` 默认 POST `http://localhost:3030/api/run`。305 个 Listing 入口都走此组件；线上读者访问的是自己电脑。`runnable` prop 也未参与按钮条件，模板只检查 `isAuto`。 | 共用同源 `/api` 运行客户端，显式支持既有配置地址；正确处理 HTTP 错误、超时和离线。真正使用执行策略决定按钮。 |
| F-02 | `listingToCodeView` 把所选文件放入 `auto` prop 并无条件设 runnable。ch01 Listing 1-2 的 `file-name="auto.toml"` 被当 Auto 程序。 | 根据作者标注和文件语言渲染配置；TOML/输出/命令只复制和阅读，不显示程序 Run。 |
| F-03 | `AutoFence` 只把当前 `source` 传给 SnippetRunner；`CodeView` 也只发当前编辑器源码。跨块定义完全没有组装。 | 给依赖片段配置显式、最小的 setup 和项目文件；运行时让读者能看到完整程序。不能把整章前文自动拼接，避免重复 main、类型重名和副作用重演。 |
| F-04 | `AutoFence.locked` 用 `use/import` 行正则决定不能运行；CodeView 则完全不做这个检查。且正则不识别 `use.rs`、`use.rust`。 | 统一两套入口的执行描述；标准库可解析不等于多文件缺失；项目示例携带真实 files/entry；Rust/Python 外部依赖需明确执行后端和准备方式。 |
| F-05 | `RunRequest` 没有 stdin；`shim_io_read_line` 直接读服务端 `std::io::stdin()`。ch05 调用未实现的裸 `input()`。 | 先用既有 `auto.io.read_line` 规范 API；浏览器提供每次运行的输入文本、显式有限 EOF。不能仅补一个阻塞服务器 stdin 的 input 别名就声称已修好。 |
| F-06 | ch03 条件与循环片段依赖 `n/count/i`；ch05 `for true { // ... }` 可能无限循环，另有脱离循环的 break/continue。 | 逐块补最小上下文；错误演示保留错误并标注预期诊断；占位无限循环不能提供普通成功 Run。 |
| F-07 | TAPL 的 `main.expected.out` / `.error` 都为 0。auto-lang `book_listing_tests.rs` 默认路径是 **book/rust/listings**，只对拍 a2r；不覆盖 TAPL 网站所有围栏。book 自身 VM harness 仅在存在 out/result/error 时自动登记该路径。 | 为 TAPL 网站实际有效源建立定向验证入口，执行后比较输出或具体诊断；不能把现有 `cargo tb` 绿色作为全书 Run 可用的证据。 |
| F-08 | ch14 `read_file` 返回固定文本；ch09 自定义 `parse_int` 恒返回 42；ch16 的所谓超时示例注释为 `Could implement timeout pattern`，没有超时逻辑。 | 核对教学承诺；文件项目和错误处理改为真实行为并提供 fixture/失败例；纯消息示例可保留，但必须准确命名说明，不能把模拟行为当对应能力验收。 |
| F-09 | ch18 几乎全为 `#[test]` 函数；一般 run 路径与 `auto test` 的测试收集路径不是同一入口。 | 接入现有测试收集执行入口或清楚显示的完整测试调用包装；断言执行次数必须 >0，并用故意失败断言验证不能假绿。 |
| F-10 | ch21 把未限定 `to_json/base64_encode/path_join/parse_toml` 等写成默认 API；现有标准库通常使用模块 API。还混有 use.rust serde_json/regex/sha2。 | 先逐个匹配现有声明、VM native 和真实调用。已有功能优先改真实 API 加 use；确有通用缺口再补最小实现/声明/绑定/错误路径，不全局注册书上每个拼写。 |
| F-11 | ch22 的 EN 有 Listing 22-6，ZH 没有同号完整服务器。正文与实际 22-7/8/9 也有差异。 | 同步中英章节完整度和实际执行源，既保留教学步骤也提供完整工程。 |

### “增加实现和 use”怎样做才有效

现有 API 是优先候选，不代表已由本次实跑证明所有装配链都通：

| 书中写法 | 现有事实 | 建议 |
|---|---|---|
| `input()` | `stdlib/auto/io.at` 有 `read_line()`；VM 有 `auto.io.read_line`，现 shim 读进程 stdin | 改标准 I/O 用法并解决网页输入通道；若保留 input 便利函数，清楚决定它属于模块还是核心，不能凭教程偷偷扩大全局内建。 |
| `random_int(1,100)` | `auto/random.at` 有 `rand_int(max)`，约定 `[0,max)`；有 seeded Random | 优先采用现有 API，并检查 1–100 的端点换算。测试注入确定性随机源，网页实际运行仍随机。 |
| `parse_int(text)` | 有 `Str.parse_int` / `auto.str.parse_int` 的 native，但声明的错误/空值语义须再核验 | 按真实转换契约写有效/非法输入处理；不能在整数字符串失败时假设自动返回 none。 |
| `to_json/from_json` | `auto/json.at` 有 encode/decode/try_decode/parse | 用实际 API，核对泛型和 JsonValue 的真实返回形态。 |
| `base64_encode/decode`、`hex/unhex` | `auto/encoding/{base64,hex}.at` 有 encode/decode | 补正确导入与调用，保留非法输入例验证。 |
| `now()` 做相减计时、`sleep(100)` | `auto/time.at` 的 now 返回 str；now_ms 返回 i64；sleep_ms 参数是毫秒 | 计时用数值时间并写对单位；不把 now 的字符串时间戳当整数。 |
| `Time.now().format(...)`、`Duration.hours` | native `Time.now` 的返回类型在 catalog 为 String；已有 `auto/datetime.at::DateTime` 提供 now/format/timestamp，但不等于书中 Time/Duration API | 优先复用 DateTime/time 并验证真实导入和调用；缺失日期运算明确实现或修订教学主张，不能增加同名空壳。 |
| `parse_toml` | 未在本次读取的标准库/VM catalog 中找到对应公共入口 | work 先确认支持库与解析链；若无可复用实现，补真正模块能力及有效/非法配置用例，或经明确裁定改写本节；不能换成固定 map 冒充解析。 |
| `read_file/write_file/path_join` | 有 File/fs/path 等模块，书上裸名与实际命名并不一致 | 用真实 API 和隔离 fixture；调用签名、返回值/错误和目录创建顺序都需验证。 |

## 逐章处理矩阵

“入口”按网站将呈现的 Run 位置计数，包含目前错误给出的 TOML Run；中英共享源码的 Listing 仍在每个页面各算一次。
除直接证实的源问题外，其余列是必须实跑的验证重点，**不把风险推测当错误结论**。

| 章节 | EN/ZH 入口 | 源码核查结果与实施重点 |
|---|---:|---|
| 00 导言 | 0/0 | 无代码围栏；保持阅读与链接正确。 |
| 01 入门 | 4/4 | 修掉 TOML 的 Run 和 localhost；空 main 示例与 Hello World 分开验收，不把“无输出”算坏。更新实际可用 CLI/包配置指引。 |
| 02 变量运算 | 16/16 | 全查声明、重绑定、除法、小数和多行字符串；纯声明允许无 stdout；需要演示结果时提供可见值，不伪造输出。 |
| 03 函数控制流 | 19/19 | greet/add 调用块缺定义，n/count/i 等缺初始化，continue 例缺递增；补独立程序或显式 setup，验证终止及分支输出。 |
| 04 集合节点 | 16/16 | 原始节点声明可以只编译/无输出；检查 set/map/node 与数组真实语义；ZH 4-9..11 与实际源不同须对齐。 |
| 05 猜数 | 13/13 | 4 个 Listing 全调用 input；还有 parse_int、random_int、依赖 guess/secret 的片段和占位死循环。补 I/O、解析失败、随机范围、输入耗尽与完整运行上下文。 |
| 06 类型 | 11/11 | let 约束、类型名和类型别名按真实语言验收；错误示范须区别可运行示例，ZH 6-1 源差异核对。 |
| 07 枚举 | 7/7 | 全部为 Listing；验证每个变体、payload 和模式分支，不能仅凭生成 Rust 判断 VM 可用。 |
| 08 OOP | 8/8 | 类型/spec/ext/组合语义；8-4 正文与实际源不同；区分声明无输出和未调用业务方法。 |
| 09 错误处理 | 10/10 | 9-3 恒返 42、9-4 未定义 read_file；裸 alice.address 依赖前文；9-8 Rust 导入，9-9 环境修改。补真实成功/失败分支、上下文和运行隔离。 |
| 10 模块 | 6/6 | 10-5 引用 utils/network.server 却只提交单源；装配真实多文件项目。更正过时项目描述，但不把注释中的示意目录当模块测试通过。 |
| 11 引用 | 8/8 | `bad() view String` 是生命周期错误教学；按检查/预期错误执行，不加空壳绕过所有权。借用可运行例验证真实结果。 |
| 12 内存 | 6/6 | 独立验证 move/clone/销毁和错误例的教学主张；VM 与静态检查语义需明确，不把 VM 偶然可跑外推所有权安全。 |
| 13 泛型 | 8/8 | Pair 定义片段、泛型函数及约束/Result 逐块验证，补演示调用；区分类型声明与运行结果。 |
| 14 文件项目 | 10/10 | 14-2/14-5 的读取是固定串；9 个 Listing 的 File/CSV/配置入口都需真实 fixture、错误分支与隔离输出，ZH 14-6..9 对齐实际源。 |
| 15 Actor | 7/7 | 验证 actor 收发、状态、监督/错误处理与有限退出；异步输出按语义核验，不固定无关线程顺序。 |
| 16 异步 | 6/6 | 16-2/16-6 有模拟工作/未实现 timeout 的注释；核对正文承诺，真正验证 await/消息关联/超时；所有网页示例可结束。 |
| 17 智能转换 | 5/5 | 17-3 Triangle 分支恒返 0 的简化需审正文含义；全部类型窄化/空值路径按结果验证，不伪称实现了完整面积公式。 |
| 18 测试 | 5/5 | 采用真实测试入口，修复断言写法或导入，输出实际执行数与通过数；用失败对照证明断言真的执行。 |
| 19 闭包迭代器 | 7/7 | 全查闭包捕获、返回闭包、filter/map/fold 等回调与集合结果；不能只验证可解析。 |
| 20 编译期 | 5/5 | 20-1 含 #[test]，但此章目标为 comptime；逐例核对 cfg/编译期断言与 Fibonacci 的实际语义，不能只跑 main 的常量输出。 |
| 21 标准库 | 14/14 | 数学、文件、集合、时间、JSON、TOML、Base64、Hex、regex、SHA 的逐项 API 装配与依赖审计；真实失败例/单位/返回类型；优先复用现有模块，补通用缺口。 |
| 22 聊天项目 | 10/9 | 整理 std::net/Rust url 等依赖；服务长驻不直接走有限单次 Run，给完整工程与本地启动入口，另给有限消息交换演示；补 ZH 缺失的 22-6 和 testing 入口；22-9 是模拟捕获输出，须准确说明。 |
| 附录 A | 0/0 | 无围栏；关键词表与正文语法更正同步。 |
| 附录 B | 0/0 | 无围栏；运算符和例子中的语义描述同步。 |
| 附录 C | 0/0 | 无围栏；后端支持与命令指引不能超出实测。 |
| 附录 D | 0/0 | 无围栏；大量表格内的 std::core/API 名称必须随第 21 章统一，不能只修按钮。 |

## 执行后的完成标准

每个有效入口都有明确的 standalone / with-setup / project / test / expected-error / local-only / display-only 处置。
既有例子缺能力先修源或实现，不能以批量 display-only、ignore 或固定输出“清零错误”。本地服务和故意错误例是合理分类，须逐项有语义理由。

逐块实跑表须记下：章节/语言/稳定 ID、实际源 hash、运行方式/文件/输入、预期输出或具体诊断、实际输出、退出与时限、修复处置和最终证据。
中英共享 Listing 可以复用相同源码/依赖/输入的 VM 执行证据，但两套生成页面的入口映射仍需逐块核对。
书页与 Playground Notes 消费同一执行描述；不能一处修复另一处继续发布坏示例。
输入、文件、环境变量和长驻服务不得污染下一次运行；请求 stdin 不能使用进程全局替换以致串线。

本次没有编辑书稿、语言实现或网站运行器，也未运行 Cargo 或网站构建。下一步为确认 Plan 后在专用兄弟 worktree 中完成真实逐块运行与修复。
