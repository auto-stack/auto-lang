---
plan_id: PLAN-458
status: in-progress
feature_name: parity 测试用例分类目录化（libs/<category>/<name>）+ 分类索引 README
author: [zhaopuming]
created_at: 2026-08-27T21:00:00+08:00
updated_at: 2026-08-27T21:00:00+08:00

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 0
total_steps: 7
---

# [PLAN-458] parity 测试用例分类目录化 + 分类索引 README

## 变更摘要

`parity/libs/` 目前平铺 35 个用例目录，靠命名前缀（`py_*` / `c_*` / 无前缀）
隐式区分类别，可发现性差：唯一权威分类是 `main.rs` 硬编码的 phase 表，且已
与磁盘漂移（8 个 `py_*` 用例不在任何 phase；p4 列的 `reqwest` 磁盘上已不存在）；
`parity/docs/parity-guide.md` 的 phase 表停在 Plan 347 时代；30/35 有 per-lib
README 但无法被一览。

本计划把用例重组为 `libs/<category>/<name>/` 两级结构，并新增
`parity/README.md` 分类索引，趁规模小把组织架构定下来，为后续更多 parity
类别（python roadmap 等）预留生长空间。

## 目标 / 架构方案

### 分类体系（5 类，覆盖全部 35 个用例）

| 目录 | 类别意图 | 用例 |
|---|---|---|
| `libs/framework/` | runner 框架冒烟 | `_dummy` |
| `libs/lang/` | Auto 语言特性三方对比（纯计算、无 IO） | `cli_app`, `generators`, `string_utils`, `trait_advanced` |
| `libs/python/` | Python 标准库三方对比（AutoVM vs a2py vs Python oracle） | `py_configparser`, `py_datetime`, `py_hashlib`, `py_json`, `py_list`, `py_math`, `py_os`, `py_random`, `py_re`, `py_string`, `py_struct`, `py_sys`, `py_uuid` |
| `libs/consumer/` | 消费级应用对比（Plan 367/368 d5/d6，Rust oracle 直调底层 crate） | `c_crawler`, `c_env_app`, `c_fs_app`, `c_http_get`, `c_json_app`, `c_process_app`, `c_text_app`, `c_wget`, `http_client_sync` |
| `libs/rust/` | Rust crate 复刻对比（Plan 347 p1-p4） | `base64`, `regex`, `rusqlite`, `serde_json`, `sha2`, `tokio`, `tokio_stream`, `url` |

### 核心设计决策：库身份串不变

**库名 = 叶子目录名**（如 `py_math`），分类只存在于路径层级。CLI 参数
（`run py_math`）、phase 表字符串、TAP 名、各用例内部 `use auto.<name>` 导入、
`auto/<name>.at` 文件命名**全部不动**。前缀去重（如 `python/py_math` 改名
`python/math`）会波及模块名生成 `auto_{name}` 与全部导入，属可分离的后续
计划，本计划不做。

### auto-parity 工具改造（crates/auto-parity）

- 新增 `resolve_lib_dir(parity_root, library)`：扫描 `libs/<category>/<library>`，
  恰好一个目录命中才返回（0 或多命中返回 None，防未来跨类重名歧义）；
- `RunConfig::lib_dir()`：解析失败时回退旧平铺路径 `libs/<library>`（保底 +
  错误信息仍指向可读路径）；
- `discover_all_libraries`：两级扫描 `libs/*/*` 收集叶子名（跳过 `_dummy`）；
  同时兼容旧平铺（`libs/<x>/auto` 存在则视为未迁移库，stderr 警告并纳入），
  保证过渡期与健壮性；
- `discover_libraries_by_phase` 的存在性过滤（`main.rs:401`）改走
  `resolve_lib_dir`；
- 单测夹具从 `libs/<name>` 更新为 `libs/<category>/<name>`，新增解析命中/
  歧义/回退三个用例。

### phase 表卫生（顺带清偿）

- p4 移除已不存在的 `reqwest`；
- 新增 `p7` 收编 8 个游离的 `py_*` 用例（configparser/hashlib/json/list/os/
  re/string/sys），使 phase 视角与磁盘全集一致；
- `parity/.gitignore` 的 `libs/*/...` 通配升级为 `libs/*/*/...`（build_a2r、
  build_a2py、tests/rust/target、c_*_tmp 四组）。

### 文档

- 新增 `parity/README.md`：五类意图说明 + 全量用例索引表（用例 × 一句话
  用意 × README 链接）+ 新增用例的目录规范；
- `parity/docs/parity-guide.md`：路径改两级、phase 表补全至 p0-p7/d1-d6；
- 补缺失的 per-lib README：`tokio_stream`、`c_crawler`、`c_http_get`、
  `c_wget`（`_dummy` 在总 README 标注为框架冒烟，豁免）。

## 测试设计与验收标准

- parity workspace `cargo test -p auto-parity` 全绿（含新增解析单测）；
- `auto-parity list`（--root 指向 worktree parity/）恰好列出 35 个叶子名；
- `phase p0`（_dummy）与每类至少 1 个用例实际跑通（复用 master 已构建的
  auto.exe，避免 worktree 全量构建）；
- `git status` 干净、`cargo check` 零警告；无遗留 TODO/debug print；
- 合并前独立复审：对照本清单逐项核销 + 全库 grep `libs/` 确认无漏网引用。

## 执行步骤

- [x] T1 计划文档 + worktree（本文件，branch plan-458）
- [x] T2 auto-parity 代码改造 + 单测（resolve_lib_dir 唯一命中/歧义拒绝；
      lib_dir 平铺回退；discover 两级扫描 + 未迁移平铺警告 + 纳入；phase
      过滤改走 resolve_lib_dir）
- [x] T3 `git mv` 35 个用例入分类目录 + `.gitignore` 通配升级
- [x] T4 phase 表修正（p4 去 reqwest、新增 p7 收编 8 个 py_*）
- [x] T5 `parity/README.md` + `parity-guide.md` 更新
- [x] T6 补 4 个 per-lib README（tokio_stream、c_crawler、c_http_get、c_wget）
- [x] T7 验证（2026-08-27）：
  - parity workspace `cargo test`：33 passed / 0 failed（含新增 5 个
    解析/发现单测）
  - `auto-parity list`：恰好 34 个叶子名（_dummy 按语义排除），无平铺警告
  - 端到端抽样（worktree 真实目录树 + master 构建 auto.exe）：
    - `phase p0`（framework/_dummy）：5/5 三方一致 ✓
    - `run py_string`（python）：8/8 三方一致 ✓（a2py/Python oracle 路径）
    - `run string_utils`（lang）：22/22 三方一致 ✓
    - `run c_text_app`（consumer）：6/6 三方一致 ✓
    - `run base64`（rust）：33/33 三方一致 ✓
  - 运行产物（build_a2r 等）确认被升级后的 .gitignore 两级通配覆盖
  - 备注：首次验证中 p0 的 a2r 列 missing 系验证手段副作用（共享
    CARGO_TARGET_DIR 把产物重定向出 runner 硬编码的
    build_a2r/<bin>/target/release/ 路径），与布局改造无关；去除该环境
    变量重跑后三方全绿（见上）
- [x] T7b 独立复审：全库 grep 确认活文档/代码无漏网平铺路径引用（归档
      plans 与历史 handoff 除外——后者 2 处亦已同步）；println! 均为工具
      正常输出；5 个编译警告全部为 master 既有死代码警告（aavm.rs
      golden_is_file、compare.rs Backend、tap.rs tap_map、runner.rs
      MockServer mut），位于本次未触碰的代码，登记 KNOWN-DEBT 不在本计划
      处理
