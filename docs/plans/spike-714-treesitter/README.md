# PLAN-714 spike——tree-sitter 首批烟测（源档）

spike-428-bench 同形：独立 scratch crate，**不入主线 workspace/crates**，
构建在仓外执行（本目录只存源档与收据）。

## 复现

```bash
cp -r docs/plans/spike-714-treesitter /d/autostack/tmp/spike-714
cd /d/autostack/tmp/spike-714
cargo run --release   # 依赖经 aliyun sparse 镜像拉取
```

## 收据（2026-09-30 实测，MSVC 工具链）

- 依赖：tree-sitter 0.27.0 + tree-sitter-highlight 0.27.0 +
  tree-sitter-rust 0.24.2 + tree-sitter-python 0.25.0。
- 版本兼容：runtime ABI v15 / 最低接受 v13 / 两 grammar v15——混搭绿。
- parse：rust source_file 52 节点、python module 41 节点——绿。
- 增量：tree.edit 标记 warm=true/cold=false；`changed_ranges`=[44..59)
  ⊂ 字节域 [40..60)（token 级差异域）。
- 查询：捆绑 highlights.scm 直载 rust 103 事件/10 类、python 76 事件/7
  类；最小内联查询类别断言绿。
- 体积初值：release exe 3,578,368B（3.41MB，全栈未 strip）。
- 冷构建：cargo clean 后 3.0s（Rust 侧 sccache 命中；C 侧 cc 真冷）。
- 出口：**SMOKE-OK**（exit 0）。

烟测 API 面注记（0.27 形）见勘定报告 §3.2 末段；证据链=
`docs/plans/reports/714-treesitter-survey.md` §3。
