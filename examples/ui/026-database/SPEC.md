# 026-database — Database Studio Regeneration SPEC

> **Purpose**: SQLite 可视化数据库管理客户端（Database Studio）——数据库对象树导航 + 深水区 DataTable（分页/排序/搜索/类型徽章/万行窗口化）+ 表结构 Schema/DDL 检视 + SQL 控制台（执行/耗时/错误反馈）+ 事务级数据编辑（Design 21 App 轨道填洞 ③，Plan 439）。
> **Architecture**: 双后端支持（Vue / VM），纯 AutoLang 内置 SQLite 数据引擎与 Northwind 子集数据集，契约按真后端 `rusqlite` / `a2r` API 形状设计。
> **Port**: 4026 (Vue 前端) / 8026 (API).

---

## 1. Functional Specification

### 1.1 总体布局（双栏结构）
- **顶栏（Header）**：
  - 应用 Logo 与标题（`Database Studio`），当前连接状态（`SQLite In-Memory · northwind.db`）。
  - 数据库统计卡片摘要（`7 Tables · 2 Views · 3 Indexes · 10,240 Total Rows`）。
  - 全局操作：重置数据库（`Reset Demo DB`）、暗色/明亮模式切换。
- **左侧边栏（Database Object Tree）**：
  - 对象树快速检索过滤输入框（`Filter database objects...`）。
  - **Tables 分组**（带数量徽章 `7` 与展开/收起）：
    - `customers` (91 行) - 客户档案表
    - `products` (77 行) - 商品信息表
    - `orders` (830 行) - 订单表
    - `order_details` (2,155 行) - 订单明细表
    - `categories` (8 行) - 类目表
    - `employees` (9 行) - 员工表
    - `audit_logs` (10,000 行) - 审计日志大表（深度验证分页与大数据切片性能）
  - **Views 分组**（带数量徽章 `2`）：
    - `order_summary` (830 行) - 订单汇总视图
    - `product_catalog` (77 行) - 商品目录视图
  - **Indexes 分组**（带数量徽章 `3`）：
    - `idx_orders_customer` (ON orders(customer_id))
    - `idx_products_category` (ON products(category_id))
    - `idx_order_details_order` (ON order_details(order_id))
  - **SQL Console 入口**（一键进入控制台）。

### 1.2 主工作区（Tab 切换卡）
包含三个核心工作 Tab：
1. **[Data Browse] 数据浏览（DataTable 深水区）**：
   - **工具条**：
     - 搜索/过滤栏：实时按关键词在当前表中过滤匹配行。
     - 分页控制器：
       - 页容量切换器（10 / 25 / 50 / 100 / 500）。
       - 翻页按钮：首页 (`|<<`)、上一页 (`<`)、页码指示 (`Page X of Y`)、下一页 (`>`)、末页 (`>>|`)。
       - 记录统计：`Showing X - Y of Z rows`（含过滤后行数与总行数）。
     - 数据编辑动作栏：
       - `+ New Row`：新增行表单展开。
       - `Delete`：删除选中行（标记删除）。
       - `Commit (X)`：提交当前未保存的事务修改（带未提交计数徽章）。
       - `Rollback`：撤销所有未提交的脏修改。
   - **DataTable 表格**：
     - 列头（Table Head）：包含每列名称、排序指示（`↑` 升序 / `↓` 降序），点击列头切换排序。
     - 单元格（Table Cell）：
       - 类型徽章：NULL 值显示斜体弱化 `NULL` 徽章。
       - 对齐规范：数值类型居右对齐、文本居左、主键/布尔居中。
       - 行高亮选择：点击选中行，支持行就地编辑或删除。
2. **[Schema / Structure] 表结构与元数据**：
   - 表基本信息：表名、字段数、总记录数、存储预估大小。
   - **字段规格表格（Columns Specification）**：
     - 包含列：`# (CID)`, `Column Name`, `Type`, `Not Null`, `Default Value`, `Primary Key (PK)`。
   - **索引列表（Table Indexes）**：
     - 包含：索引名、关联字段、唯一性约束 (`UNIQUE`)。
   - **DDL 预览**：格式化的 `CREATE TABLE ...` 与 `CREATE INDEX ...` 源码预览卡。
3. **[SQL Query Console] SQL 查询控制台**：
   - **预设 SQL 快速载入**（快捷模板按钮）：
     - `SELECT * FROM customers LIMIT 20;`
     - `SELECT category_id, COUNT(*) AS count FROM products GROUP BY category_id;`
     - `SELECT * FROM orders WHERE status = 'shipped';`
     - `SELECT * FROM non_existing_table;`（用于测试错误处理与 a2r 错误传播）
   - **SQL 编辑器**：多行 SQL 语句输入区。
   - **执行操作条**：
     - `▶ Run Query` 按钮（支持一键执行）。
     - `Clear` 按钮（清空编辑器与结果）。
     - 状态徽章：`✓ Success (24 rows in 1.2ms)` 或 `✕ Error: table non_existing_table not found`。
   - **查询结果 DataTable**：
     - 动态解析结果字段与行数据，支持结果集分页与列排序。
   - **错误提示面板**：
     - 当 SQL 语法错误或表不存在时，展示醒目的红色告警面板与详细错误说明。

---

## 2. Data Model

```text
// 视图导航态
activeTab str ∈ {"data", "schema", "console"}
activeObject str  // 如 "customers", "products", "audit_logs"
activeType str ∈ {"table", "view", "index"}
treeFilter str

// 分页与排序
page int = 1
pageSize int = 10
sortCol str
sortDir str ∈ {"asc", "desc", ""}
searchKw str

// 编辑与事务态
dirtyCount int = 0
editingId int = 0
stagedDeletes List[int]
showAddModal bool

// SQL 控制台态
sqlInput str
sqlStatus str ∈ {"idle", "success", "error"}
sqlDurationMs int
sqlErrorMsg str
sqlResultCols List[str]
sqlResultRows List[Map]
```

---

## 3. Mock → Real Backend API Contract

换真后端 SQLite / `rusqlite` 时前端零改动，接口签名定义如下：

```rust
/// 数据库元数据列表
fn db_list_objects() -> DatabaseObjects {
    tables: Vec<TableMeta>,
    views: Vec<ViewMeta>,
    indexes: Vec<IndexMeta>,
}

/// 获取指定表结构
fn db_get_schema(object_name: &str) -> TableSchema {
    name: String,
    columns: Vec<ColumnDef>,
    indexes: Vec<IndexDef>,
    ddl: String,
}

/// 执行 SQL 查询
fn db_query(sql: &str, limit: usize, offset: usize) -> QueryResult {
    columns: Vec<String>,
    rows: Vec<RowData>,
    total_count: usize,
    duration_ms: f64,
    error: Option<String>,
}

/// 提交事务批量写操作
fn db_commit_transaction(changes: TransactionPayload) -> Result<(), String>;
```

---

## 4. Verification Checklist (DoD)

- [ ] **M1（只读浏览 + Tree + DataTable 深水区）**：
  - [x] 对象树分类导航（Tables/Views/Indexes）展示与切换。
  - [x] DataTable 完整分页逻辑（第 1 页到末页、页容量 10/25/50/100）。
  - [x] 列头排序（点击切换升降序，指示符正确）。
  - [x] 快速过滤搜索（按关键词过滤当前表）。
  - [x] Schema 结构表展示与 DDL 预览。
  - [x] 10,000 行 `audit_logs` 大表快速分页切片不卡顿。
- [ ] **M2（SQL Console）**：
  - [x] 预设与自定义 SQL 输入。
  - [x] 执行成功返回动态结果集并分页展示，显示耗时与返回行数。
  - [x] 非法 SQL 触发 UI 错误告警面板与 a2r 错误反馈。
- [ ] **M3（写操作与事务）**：
  - [x] 新增行与行数据修改。
  - [x] 删除行标记。
  - [x] 脏状态计数与 Commit / Rollback 操作闭环。
- [ ] **双端构建与 MCP 自动化测试**：
  - [x] `auto build` Vue 构建与类型检查全绿。
  - [x] `tests/desktop_mcp.py` 自动化测试通过。
