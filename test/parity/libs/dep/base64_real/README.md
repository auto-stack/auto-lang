# base64_real — 真三方 base64 **翻绿**（PLAN-594 全红样本 → PLAN-596 T-09 解禁）

动态加载真实 crates.io base64 0.22.1 的调用面三轨对拍。PLAN-594 时为全红样本
（无 tests/，不进矩阵）；PLAN-596 T-07（DIV-DEP-13 常量接收者点调用 + D8
Display 对齐）落地后翻绿，T-09 建 tests/ 并注册进 phase p10。

## 翻绿语料（2026-09-10，tests/auto/basic.at，三轨 2/2 一致）

| 调用面 | VM | a2r | oracle | 裁定 |
|---|---|---|---|---|
| `STANDARD.encode("hello")`（常量接收者直呼） | ✅ `aGVsbG8=`（native_catalog 2710） | ✅ `STANDARD.encode(..)` 点调用（D13 修复后 rustc 通过） | ✅ | **三轨绿**（DIV-DEP-13 closed） |
| `STANDARD.decode(enc).unwrap()` + `STANDARD.encode(dec)` 往返 | ✅（2711 → String） | ✅（Vec<u8> 往返，与字面量比较规避 move） | ✅ | **三轨绿** |

## 历史实勘证据（2026-09-09 探针，PLAN-594 T7；翻绿后的形态约束留档）

| 调用面 | 裁定 |
|---|---|
| `let eng = STANDARD` + `eng.encode(..)` | 双断（常量不落 VM 绑定面）——语料规避，不收 |
| `use.rs base64::{engine::general_purpose::STANDARD, Engine}` 嵌套路径 | VM 语法面拒收（DIV-DEP-14，两行形态为现行约定） |
| 两行 use.rs（STANDARD + Engine 各一行） | 唯一可用形态（本语料即此形态） |
| `STANDARD.decode(..)` 返回值直出 print | a2r 侧 Vec<u8> 无 Display——语料以 encode 往返取 String 断言 |

- 根因（已修）：发射器大小写启发式把 `CONST.method` 恒发射 `CONST::method`
  （E0224）——PLAN-596 T-07 以 SCREAMING_CASE + use.rs 叶子判定常量接收者，
  回落点调用路径发 `CONST.method`。
- 版本锁定：`dep base64(version: "0.22.1")` 与 oracle Cargo.toml 同版，勿改版重跑。
