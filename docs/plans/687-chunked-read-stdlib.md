---
plan_id: PLAN-687
status: executing             # drafting → executing → execution_done → reviewed → archived
feature_name: chunked-read-stdlib
author: [zcode]
created_at: 2026-09-22
updated_at: 2026-09-22

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [SD-01 read_text_range 真分块 IO 契约（流式语义注记：UTF-8 校验限块区）]
touched_goals: []

affects: [auto-lang/vm, a2r-std]
current_step: 0
total_steps: 4
---

# [PLAN-687] read_text_range 真分块 IO（供料 auto-edit PLAN-007 §10 观察 B②）

## 变更摘要

`File.read_text_range`（nat#1016 VM shim + a2r-std 镜像）现实现为
`fs::read` **全量读盘 + from_utf8 全文验证 + 切片**——分块读端点在
IO 层并未分块（多块调用 = IO 平方；全文件反复过内存）。本计划改
**真分块**：`std::fs::File` + `metadata`（总长）+ `seek` + 精确窗读
（借 Rust std 现成实现——用户 2026-09-22 方针：「标准库的一些函数
也可以借助 rust 的实现来加速（未来再自己写）」）。envelope 契约
（`{text,total,next_offset}` serde 形状与字段序）byte-identical 不变；
双轨（VM shim / a2r-std）继续逐字节一致（P670-D1）。

## 目标

1. **单块调用只读块区**：IO 与内存成本 = O(limit)（+ ≤4B 边界余量）
   而非 O(file)——大文件多块装载的 IO 平方与全文过载随之消除。
2. **契约保形**：边界矩阵（offset 回退/limit 回退/真 EOF/空文件/
   IO 错误/CJK 边）逐字节等值（既有 parity 测试守门）。
3. **语义注记（流式收窄）**：UTF-8 校验从「全文一次性」收窄为
   「块区」——非法字节在所在块读取时报 total:-1，未读区不再提前
   暴露（流式语义合理化；既有测试无 invalid-UTF-8 全文用例，不踩）。

## 架构方案

- 窗读算法（两轨同形，各自内联——两 crate 无共享依赖面）：
  1. `total = metadata(path).len()`（缺席/错误 → 错误形 total:-1）；
  2. 参数校验层不动（shim：offset<0||limit<=0；a2r-std：limit==0）；
  3. `start = min(offset, total)`，`start >= total` → EOF 形（真 total）；
  4. 读窗 `W = [start.saturating_sub(3), min(total, end+1))`——
     前向 3B 余量供 offset 回退（UTF-8 最长 4B），尾向 1B 余量供
     「end 是否落在续字节上」判定；`seek + read_exact`；
  5. 窗内回退：`s` 回退到非续字节（0xC0 掩码判 0x80 续字节族）；
     `e`（若窗含 end 处字节）回退到非续字节——与旧版全文
     `is_char_boundary` 双 walk 等价；
  6. `String::from_utf8(W[s..e])` 失败 → 错误形（块区校验语义）；
  7. `next_offset = e_abs >= total ? null : Some(e_abs)`（真 EOF 判定
     同旧：end 到字节尾才 null，短读不伪造）。
- envelope 序列化结构（serde 字段序 text,total,next_offset）原样保留。

## 需求分析与背景调查

（从 docs/specs/overview.md 与相关 module spec 取材）

- **供料归因**：auto-edit PLAN-007（store 去文本化）T-03 实测——
  分块装载 100MB 时 RSS 729-1356MB vs 旧 src 镜像路径 521MB 锚，
  归因三件之一即本件（docs/upstream/2026-09-m1-supply.md §10 观察
  B②，auto-edit 仓）。用户 2026-09-22 裁定走上游修复路线。
- **现实现锚**：`a2r-std/src/fs.rs:129` 与
  `auto-lang/src/vm/ffi/stdlib.rs:367`（shim）——两处同构
  `fs::read → from_utf8(全文) → 切片`；envelope helper
  `read_text_range_json`（stdlib.rs:352）。
- **测试守门面**：`tests/read_text_range_parity.rs`（双轨 14 例边界
  矩阵逐字节等值）；`test/vm/18_ffi/058_read_text_range/`（VM 语义
  五例+CJK）；a2r-std 单元测试。均小文件（块区=全文），流式收窄
  不触碰。
- **越界不修**：edit O(n) 全文重写=S2 债（P673-D1 在册维持）；
  renderer 首建推送落空（观察 A）=下游已有防御 + 主检出并行 WIP
  冲突面，维持登记。

## 详细设计

文件两处（函数体替换，签名/参数校验/envelope 结构不动）：

1. `crates/a2r-std/src/fs.rs::read_text_range(path, offset: usize,
   limit: usize) -> String`
2. `crates/auto-lang/src/vm/ffi/stdlib.rs::shim_file_read_text_range(
   path: String, offset: i32, limit: i32) -> String`

均按「架构方案」窗读算法重写；stdlib 侧复用既有
`read_text_range_json` helper。

## 测试设计

- 既有守门：`cargo test -p auto-lang read_text_range`（parity）+
  a2r-std fs 测试 + VM 058（`auto test` 或既有门禁路径）。
- 新增（a2r-std 单测）：大文件多块拼接回读——8MB 临时文件按 3MB
  块循环读，拼接 text == 原文（真分块的端到端等值性）；以及块区
  校验收窄注记例（invalid UTF-8 落在第二块——首块正常返回，第二块
  错误形——语义注记的构造性红证）。

## 验收标准

- **AC-1**：双轨 parity 测试 + VM 058 + a2r-std 测试全绿（byte-identical
  守门零回退）。
- **AC-2**：真分块实证——多块拼接单测绿 + 新增收窄注记例绿。
- **AC-3**（合作件，auto-edit 侧执行）：下游 100MB 装载锚点复跑
  内存较 521MB 锚下降（PLAN-007 AC-02 复验；工具链含本计划构建）。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- **T-01** a2r-std `read_text_range` 真分块重写 + 新增两单测 →
  `cargo test -p a2r-std fs`
- **T-02** VM shim 同形重写 → `cargo test -p auto-lang
  read_text_range`（parity 守门）
- **T-03** 工具链构建（worktree 内 `cargo build --features ui-iced
  --bin auto`）+ VM 058 语义例复跑
- **T-04** 下游复跑（auto-edit edit-007 工作树：AUTO_BIN=新工具链 +
  装载回多块 + L0 锚点）→ 收据回写 PLAN-007

## 复审记录

- 2026-09-22 · stage: work entry · PLAN-687 · r1 · 供料 auto-edit
  PLAN-007 §10-5 用户裁定（上游修复路线）；起草即执行授权=用户
  会话指令原文（「应该修改上游标准库的实现……借助 rust 的实现来
  加速（未来再自己写）」）。

## 待澄清事项

（无——范围已裁：仅 read_text_range 真分块；edit O(n) 与观察 A 维持
登记不修。）
