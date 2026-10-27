# PLAN-842 T-02 工作中：第一批与第二批消费者

计划 r2；T-01 冻结决策见 [矩阵](../retirement-matrix.md)。UI 基线 `49aa54d6e18e8d893be69b9430022aea14624ce7` → 提交 `96cd9f91d736913fae4412eeb8b674140c2f66e3`；语言基线 `ecc78701124fe6e0ab6f0e262148a1d853ba2af5` → 提交 `ee18e8b0b5d828b12b1536814ca528d36222b1ea`、`de258c3762ddc0bf6bd456cb7fc72640da7c8301`、`f43bab5b00833add2a98e86e9c8d35b8c5431b33`、`0d96d596c60c0374ab95a3ab1eb537aa0186f925`；Man `7cc37fa364a2f59bedc2c920638f8007cd0075ce`、down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。T-02 consumer 迁移完成；生成器/安装在 T-03，过渡声明删除在 T-04。

- UI grammar 内部及轻消费者使用 `auto_parser_api::DomainParseContext`，旧公开别名保留待 T-04。`ui-grammar-consumer --test v01_light_grammar` 6/6。真实 `auto-lang --features test-cases --lib plan810_full_ui_grammar` 21 passed、1 ignored（未终止 EOF 旧探针，仅隔离 deadline 下运行），2164 filtered。未指定 test-cases 的首次编译因原测试组未 gate 的 `auto_test_case`/`ac_av_probe_tests` 依赖失败；启用原测试 feature 后定向成功，诊断见日志。
- `AuraViewBuilder::with_registry` 与四参旧构造器逐字段相同；4 个生产点和 10 个测试点迁至现有三参入口，旧构造器保留至 T-04。UI runtime 实际注册图标 1/1，outlet 3/3；多余测试 filtered。
- LSP 生产路径由旧 facade 切到 `auto-ast`、`auto-lang-core`、`auto-diagnostics`，Cargo.lock 增加真实直接依赖；`cargo check -p auto-lsp` 与语义标记 1/1 成功。
- 外部 UI 方言消费者改直连 SDK 和 owner；保留非 UI 旧 Dialect/ParserBridge 实际注册与值/错误断言。七个受影响 test target 共 48/48：scope 12、dispatch 4、legacy 3、delegation 6、bare-field 8、external dialect 12、snapshot 3。日志分两份，第二份使用 --locked。
- 第二批已提交：native frame 6/6、clipboard 2/2、dialog/dnd 3/3，均真实 VM/provider；诊断互操作 7/7；AST identity 2/2 和 glob 1/1。对应日志在本目录；另 `cargo check` 验证样例与三个受影响集成目标，SDK 文档测试 1/1；diagnostics 的原示例测试为 1 ignored、0 passed，不冒称通过。
- 各命令均 offline 且 scoped；已存在的 Rust warnings 仍有输出，不称零 warnings。当前 UI/语言源码已变，Man 发行驱动 UI pin 和之前 T-00 的最终安装绑定已经过时；T-03 更新，T-07 在最终源码上重建安装并重放。完整独立 review 和 merge-owned 档位 pending。

- 轻文法原 `verify-deps.sh` 在当前 bash 指向 WSL 时 exit 1 且将 cargo 的 stderr 吞掉，未作为通过依据；使用 Windows 原生 `cargo tree --locked --offline --edges normal,build,dev --prefix none` 复核实际同一依赖闭包：无 `auto-lang`/`auto-man`/`auto-vm`/`auto-ui`，存在 `auto-ui-dialect`，原始树见 `ui-grammar-cargo-tree.log.txt`。
- 领域组合 helper 的误导性 `parse_override` 名称迁为 `parse_with_external_ui`，保留原测试 ID 与断言并重放 12/12；实际非 UI `Override`/`Append` 公开合同与测试仍保留。
- 语言树 6 个 fixture Cargo.lock 为测试产生的未跟踪文件，保留；最终验收前需按实际锁策略处理。
