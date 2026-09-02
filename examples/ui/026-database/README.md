# 026-database — Database Studio (SQLite 可视化客户端)

> **AutoOS 默认应用矩阵 · App 轨道填洞 ③ (Plan 439)**
> 端口: `4026` (Vue 前端) / `8026` (API) | 场景: `ui` | 渲染引擎: `vue` / `vm`

---

## 概述

**Database Studio** 是 AutoOS 默认应用矩阵中的 SQLite 可视化管理客户端，提供轻量、现代且响应迅速的数据库管理体验。

### 核心特性

1. **数据库对象树导航 (Database Object Tree)**
   - 树状层级浏览 Tables (7), Views (2), Indexes (3)。
   - 对象名与行数徽章实时展示，支持实时搜索过滤。
2. **DataTable 深水区能力**
   - **分页控制**：支持首页、上一页、下一页、末页快速跳转，可自由切换页容量 (10/25/50/100)。
   - **列排序**：点击表头即时升降序重排，表头带方向指示。
   - **数据检索**：支持快速多字段模糊过滤。
   - **类型徽章与样式**：NULL 值斜体弱化展示，数值右对齐。
   - **大数据集验证**：内置 10,000 行 `audit_logs` 表用于检验切片性能。
3. **表结构与 DDL 检视 (Schema Inspector)**
   - 字段规格明细清单（CID、字段名、数据类型、Not Null、Default Value、主键 PK 标识）。
   - 索引清单（索引名、字段、唯一性约束）。
   - 格式化 `CREATE TABLE ...` DDL 源码卡片。
4. **SQL 查询控制台 (SQL Console)**
   - 快捷预设 SQL 模板（Customers、High-Value Products、Orders、Error Test）。
   - SQL 执行耗时 (ms) 与返回行数统计。
   - 结果集 DataTable 呈现。
   - 详尽的错误路径反馈面板。
5. **写操作与事务模拟 (Transaction / Dirty Tracking)**
   - 新增记录表单与行就地删除。
   - 事务脏状态计数 (`Uncommitted` 徽章)。
   - `Commit` 提交与 `Rollback` 撤销操作闭环。

---

## 快速运行与构建

### 1. Vue 模式
```bash
cd examples/ui/026-database
auto run
```
在浏览器中打开 `http://localhost:4026`。

### 2. 构建与类型检查
```bash
auto build
```

### 3. VM 模式与自动化 MCP 测试
```bash
cd examples/ui/026-database/tests
python desktop_mcp.py
```
