# base64_real — 真三方 base64 **全红样本**（PLAN-594，不注册进 phase p10）

动态加载真实 crates.io base64 0.22.1 的调用面**三轨无一致绿面**——本库是
PLAN-594 命中率报告的全红样本（验收标准 #1 注记豁免，见计划待澄清 #2）。
因此**不含 tests/ 目录**：放进矩阵只会产出恒分歧 case，无对拍价值。

## 实勘证据（2026-09-09 探针，PLAN-594 T7）

| 调用面 | VM（pack/native） | a2r | oracle 可写 | 裁定 |
|---|---|---|---|---|
| `STANDARD.encode("hello")`（直呼） | ✅ `aGVsbG8=`（真 crate pack 构建，T7 实证） | ❌ 发射 `STANDARD::encode`（大写接收者被当类型，E0224 级编译失败） | ✅ | 三轨断 |
| `let eng = STANDARD` + `eng.encode(..)` | ❌ 绑定拿回 None（常量不落 VM 绑定面） | ❌ 转译即失败 | ✅ | 双断 |
| `use.rs base64::{engine::general_purpose::STANDARD, Engine}` 嵌套路径 | ❌ E0099 语法错（20 errors，嵌套 brace 路径不解析） | — | ✅ | VM 语法面拒收 |
| 两行 use.rs（`use.rs base64::engine::general_purpose::STANDARD` + `use.rs base64::Engine`） | ✅ 语法通过且 encode 绿 | ✅ `use base64::Engine;` 发射正确 | ✅ | 仅此形态可用（上两行即此形态） |
| `STANDARD.decode(..)` → `Vec<u8>` | （未钉） | ❌ Vec<u8> 无 Display，print 位编译失败 | ✅ | 红面 |

- 根因归类：**trait 面**（`Engine` trait 方法经常量接收者调用）+ **发射器大小写
  启发式**（`Type.method` 与 `CONST.method` 不可分辨，恒发射 `::`）。根治归属
  PLAN-591 V2（trait 单态转发）/发射器元数据接入。
- 离线可复现：本表各面用 `dep base64(version: "0.22.1")` 语料 + 上述形态即可重钉。
