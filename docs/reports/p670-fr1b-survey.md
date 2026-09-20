# P670 F-R1-B 勘定决策件：契约 fn 空体桩三态分类

- 日期：2026-09-21（PLAN-670 T-01）
- 输入证据：auto-edit 契约实物（`D:/autostack/.wt/edit-003/auto-edit/specs/auto-edit/src/back/{api.at,fsys.at}`，PLAN-003 T-01 产出）+ auto-edit 003 计划 F-R1 定性（六契约 fn 空体 + E0432）+ 本仓 api_gen.rs / trans/rust.rs / a2r-std 源码勘定。
- 结论：**S-重活** → 按 PLAN-670 §5.2 处置：本批缩面只交付 F-R1-A（`use api::Db` 条件化）；B 半登记 KNOWN-DEBT **P670-D1**，拆独立计划呈报用户裁定。

## 1. auto-edit 契约命中支路

契约形状（api.at）：六端点**全标量返回**（str/bool）、**无 `pub type` 类型块**、`use fsys` 伴生模块、**无 db.at**。

生成器判定链（auto-man/src/api_gen.rs）：

1. `primary_type_name_pub(api_module)` → **None**（无类型块）；
2. `has_db=false`（src/back/db.at 缺席，:657 只认 db.at）→ `db_fns=None` → `db_active=false`；
3. `generate_api_rs` 走 :1590 None 支路 → db_active=false → **命中支路②：骨架 fallback（:1693-1701 "No types defined, generating skeleton handlers"）**——六 fn 全部 `pub async fn name() { // TODO: Implement }`；
4. `generate_main_rs`：`db_full_cover=false`（db_fns None）→ legacy 种子路径无条件 `use api::Db`（:2637）→ api.rs 无 `pub type Db` 定义（:1705 只在有 primary 类型时发出）→ **E0432**（auto-edit 003 review 独立源级核对在案：main.rs:199 use api::Db + api.rs 无 Db）。

支路①（:1625，db.at 名单未覆盖端点的 fallback）与支路③（:2354，State<Db> CRUD 模板非 CRUD 方法 default 臂）本次未命中；三支路同根（见 §3）。

## 2. 真体需要什么（S-重活判据）

契约 fn 体形如 `return fsys.root_dir()`——实现体在伴生模块 fsys.at（Env/fs/File 内建薄封装）。真降体 = 三层接线：

| # | 缺口 | 现状 | 尺寸 |
|---|---|---|---|
| 1 | **伴生模块通用转译**：生成器只认 `src/back/db.at`（:657 硬编码），fsys.at 不进转译面 | db.at 单点特判 | api_gen.rs 通用化 + mod 声明 + Cargo 依赖门 ~60-100 行 |
| 2 | **Route A 接线**：端点 fn 体直接降体进 handler 从未实现——`ApiEndpoint.body` 字段生产不读（api/types.rs 注记 "Currently unused in production… route B shipped"，仅 `test_extract_endpoint_captures_body` 读） | 零接线 | no-types 有体契约 → transpile api.at（剥离 #[api] 属性行）→ 委派/直体 ~100-150 行 |
| 3 | **trans/rust.rs 内建面扩展**：`Env.get`（大写 VM 内建名）**零覆盖**（小写 `env.get` 有映射 :5687）；`fs.tree(path, depth)` 转译器无 ("fs","tree") 映射 | 部分缺席 | 别名 + 映射 ~20-40 行，**动 trans/ 本体** |
| 4 | **a2r-std 新宿主函数**：`a2r_std::fs` 无 tree（读目录树→JSON 串），需新增宿主实现且 **JSON 形状与 VM 内建逐字节对齐**（front 侧 json.to_value 解析，vm/rust 双轨分叉风险） | 双双缺席 | a2r-std/src/fs.rs ~30-60 行 + 对齐验证 |

已覆盖项（无需动）：`File.read_text/write_text/exists` 转译映射在档（trans/rust.rs:6504-6525 std::fs 直映射）；`File.write_text` a2r 形态返回 int 0/-1 但 fsys 用法丢弃结果、仅语句位，无碍。

**合计 ~210-350 行，跨 auto-man / auto-lang trans / a2r-std 三 crate；§5.2 S-重活判据双条全中（"trans/rust.rs 接线" + ">~300 行"量级），另含新宿主运行面（fs.tree）与双轨语义对齐风险。**

## 3. 三支路同根注记

支路①②③共享同一根因：生成器只会两种真体来源——db.rs 委托（route B，fn 名单匹配）与 State<Db> CRUD 惯用法模板（:2331 邻域 toggle 推断是其内先例）；**端点自身 .at fn 体从未被降体**（route A 未建）。B 半修复因此是"route A + 伴生模块通用化"一件事，不是三个孤立模板缺口。

## 4. 处置

- 本批（PLAN-670）只交付 F-R1-A：无 Db 契约生成工程可编译（E0432 消失）——修复后 auto-edit rust 轨可编译，六端点为空体桩（HTTP 200/空返回）但不再卡构建。
- B 半登记 KNOWN-DEBT-AND-RISKS.md **P670-D1**，呈报用户裁定拆独立计划（建议输入=本决策件 §2 四缺口表）。
