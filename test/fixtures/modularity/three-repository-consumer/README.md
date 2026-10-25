# three-repository-consumer

PLAN-832 的外部消费探针，独立 workspace。当前从固定语言仓消费 `auto-man-core`、
`auto-lang-core`、`auto-tool-protocol`、`auto-project-model` 和 `auto-val`；T-08 只切换
Man 所属包的固定 source，不改变断言或引入第二份类型。

真实判据：

- 管理端口选择规则在 crate 外执行；
- SDK 真实 parse/compile/VM 执行并输出 42；
- I-PROJECT 请求经协议单源 codec 和 framed envelope 往返；
- `verify-deps.sh` 要求四个关键包各只有一个 source，且闭包无 UI、旧兼容包、
  `auto-vm`/`auto-gen` 或第二实现。

入口：

```bash
cargo test --manifest-path test/fixtures/modularity/three-repository-consumer/Cargo.toml
bash test/fixtures/modularity/three-repository-consumer/verify-deps.sh
python scripts/three-repository-fixture.py run --phase pre-migration
python scripts/three-repository-fixture.py selftest
```

安装、升级/失败和三仓完整运行由同一 driver 的后续阶段承载；未执行的阶段在收据中
显式为 pending，不能用本 fixture 的通过代替完整三仓发行验收。
