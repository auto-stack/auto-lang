# auto-parser-api：解析推进性与坏源受控拒绝

> **Status**: active
> 路径：`crates/auto-parser-api`（合同面）+ Core/方言实现约束
> 来源：[PLAN-940](../../plans/archive/940-parser-progress-and-malformed-source-recovery.md)；
> 验收与监督探针证据见该归档计划 §9。

## Core 保证（现行行为，PLAN-940 控制反例钉住）

- `Parser::parse` / `Parser::parse_domain` 主循环以 EOF 为终止条件；语句错误经
  `add_error`（`error_limit` 有界）+ `synchronize` 恢复，`synchronize` 在 EOF
  立即停止——坏源有限终止、诊断不无限增殖。
- `parse_body` 以 `expect(LBrace)` 开头；`parse_expr`/`parse_type` 在 EOF 受控
  报错（"Expected term"/"Expected type"），不静默返回空产物。
- 方言 `Err` → 核心收集后恢复继续，方言注册不丢；`Ok(None)` 让行的 checkpoint
  回滚保持；坏源 Err 后同一公开消费流程的下一次合法 parse（新 Parser）成功。

## 端口语义（方言作者义务：推进不变量）

- `advance()` 在 EOF **不推进**（游标停在 EOF token）；EOF token 文本为空串。
- 任何"收集循环"（等待闭合符 `}`/`)`/`]` 的 while/loop）**必须显式守卫 EOF**：
  到达 EOF 时受控拒绝（仓内惯例 `if is_kind(EOF) { expect(closer)?; }`，错误
  带位置与期望闭合符），不得假设循环体必然消费游标或必然报错。
- 依据：EOF 上若干文法臂可 `Ok` 返回且游标不动（如 UI `parse_view_node` 的
  空元素、裸 token 收集臂），只看闭合符的循环会无限收集——UI042 截断 widget
  实测 ~51GB 病理分配（PLAN-940 修复前监督探针 11 形态 resource-stop
  >512MiB/10s；修复后执行方 16 + 复审方 12 形态全部 0ms 受控 Err）。
- 检查面惯例：合法语料逐字节前缀扫（每个前缀 Ok 或受控 Err、完成即 oracle）
  与"坏源后下一份合法源码可消费"恢复腿，随本合同入各 owner 测试面。

## 边界

- 不承诺任意输入的无限资源保证，也不附加新的固定语法长度/深度上限；本合同
  只覆盖推进、终止与受控拒绝语义。
- PLAN-940 语言仓零生产差额：端口行为未变，[project.md](project.md) 的
  841/842 合同仍准确、未随本包改动；本文件将上述现行合同显式化为方言作者
  的推进不变量依据。
