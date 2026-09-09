# P594 skip 面命中率报告——真三方五库调用面三轨实勘

**计划**: PLAN-594（dep-parity-real-crates） · **日期**: 2026-09-09 ·
**证据**: parity 工具三轨跑批（`phase p10` 4 库全绿）+ 起草期/T5..T9 形态探针；
登记条目 `parity/docs/known-divergences.md` DIV-DEP-8..14。

## 目的

为 PLAN-591（use-rust-any-crate-direct，能力计划）的 T2/T3 优先级裁定提供
**真实 crates.io 库调用面**的 skip 面分布数据——回答「V1 的 Option/Result 语义
收口」与「V2 的 trait/泛型单态化」哪个解锁面更大。

## 口径

- **三面**: VM 腿按路径分两半——`VM-native`（类型构造器/方法走 native_catalog
  手写分发，零构建）与 `VM-pack`（自由函数/构造器走 430/212 wrapper，真编译
  三方 crate）；`a2r` = 转译编译轨。oracle（手写 Rust 真库）恒可表达，不计红。
- **绿** = 三轨 TAP 可一致断言的调用面（已进 `libs/dep/*_real` 语料）。
- **实证红** = 探针实跑钉死的红面（DIV 编号锚定）；**推定红** = 按签名/已知
  分歧规则推定、未单独实钉（报告中标注，不进 known-divergences）。
- 命中率 = 绿面数 / 调用面总数（每库清单见下）。

## 五库调用面清单（25 面）

### serde_json 1.0.145（6 面，绿 1 → 17%）

| 调用面 | VM-pack | a2r | 裁定 |
|---|---|---|---|
| `from_str::<Value>(&str) -> Result<Value>` + unwrap + `print` 直出 | ✅ | ✅ | **绿**（语料 `from_str_unwrap_print`；print 行三轨逐字节一致） |
| `Value.to(str)` 字符串化 | 紧凑 JSON | Debug 转储 | 实证红 DIV-DEP-8 |
| `let n i64 = from_str("42")` 标注 | `4294967295` | `42` | 实证红 DIV-DEP-9 |
| `from_str(bad).is_err()` | 静默 `None` | `true` | 实证红 DIV-DEP-12 |
| `to_string(42)`（&T: Serialize） | RuntimeError | 缺 `&` E0308 | 实证红 DIV-DEP-10 |
| `to_string(&Value)` 借用参 | — | DIV-DEP-7 家族 | 推定红 |

### regex 1.11.3（4 面，绿 2 → 50%）

| 调用面 | VM | a2r | 裁定 |
|---|---|---|---|
| `Regex.new(&str)` + unwrap | ✅ | ✅ | **绿**（`new_unwrap_is_match`） |
| `re.is_match(&str) -> bool` | ✅ | ✅ | **绿**（`is_match_negative`） |
| `re.find(&str) -> Option<Match>` 句柄导航 | 无 Match 面 | Option 链 | 推定红（Option 族） |
| `captures`/`replace_all`（Cow/借用返回） | 无 | — | 推定红 |

### base64 0.22.1（4 面，绿 0 → 0%，**全红样本**）

| 调用面 | VM-pack | a2r | 裁定 |
|---|---|---|---|
| `STANDARD.encode(&str) -> String`（直呼） | ✅ `aGVsbG8=` | `STANDARD::encode` 编译失败 | 实证红 DIV-DEP-13 |
| `let eng = STANDARD` 常量落绑定 | `None` | 转译失败 | 实证红 |
| `use.rs` 嵌套路径一行形态 | E0099 | — | 实证红 DIV-DEP-14（accepted，两行形态缓解） |
| `STANDARD.decode -> Vec<u8>` | — | Vec 无 Display | 推定红 |

### url 2.5.4（6 面，绿 3 → 50%）

| 调用面 | VM-native | a2r | 裁定 |
|---|---|---|---|
| `Url.parse` + unwrap | ✅ | ✅ | **绿** |
| `u.scheme()` / `u.path() -> &str` | ✅ | ✅ | **绿**（`parse_scheme`/`parse_path`） |
| `u.host_str().unwrap()` | ✅ | ✅ | **绿**（`host_str_unwrap`） |
| `u.host_str()` 直出不 unwrap | 裸文本 | `Some(..)` | 实证红 DIV-DEP-11 |
| `u.port() -> Option<u16>` | 未钉 | Option 链 | 推定红 |
| `u.to(str)` | `<url::Url>` | Debug 转储 | 实证红 DIV-DEP-8 |

### semver 1.0.26（5 面，绿 2 → 40%）

| 调用面 | VM-native | a2r | 裁定 |
|---|---|---|---|
| `Version.parse` + unwrap | ✅ | ✅ | **绿** |
| `.major/.minor/.patch -> u64`（数值比较） | ✅ | ✅ | **绿**（`parse_major/minor/patch`） |
| `print(v)`（Display 面） | `<semver::Version>` 占位 | `"1.2.3"` | 实证红 DIV-DEP-8 家族 |
| `VersionReq`/`Comparator` 构造器面 | 未钉 | 未钉 | 推定红 |
| `bump_*`（&mut self 变异接收者） | 无 | — | 推定红 |

## 汇总

| 库 | 面 | 绿 | 实证红 | 推定红 | 命中率 |
|---|---|---|---|---|---|
| serde_json | 6 | 1 | 4 | 1 | 17% |
| regex | 4 | 2 | 0 | 2 | 50% |
| base64 | 4 | 0 | 3 | 1 | 0%（全红样本） |
| url | 6 | 3 | 2 | 1 | 50% |
| semver | 5 | 2 | 1 | 2 | 40% |
| **合计** | **25** | **8** | **10** | **7** | **32%** |

红面按主因分类（DIV-DEP-8..14 + 推定）：

| 主因分类 | 面数 | 解锁归属 |
|---|---|---|
| Option/Result 语义（DIV-DEP-1/11/12 + find/port/captures） | ~5 | **591 T2（最高杠杆）** |
| 格式化/字符串化·槽转换·语法（DIV-DEP-8/9/14） | ~7 | 发射器 Display 对齐 + wrapper 返回码（建议并入 T2 范围一并裁定） |
| trait/常量接收面（DIV-DEP-13、Origin/Serde） | ~3 | **591 T3（第二优先，base64 整库解锁）** |
| generic/借用（DIV-DEP-7/10 家族） | ~2 | A2R_EXTERN_SIGS / dep 元数据接入 |
| by-value-self（DIV-DEP-2） | **0** | 五库调用面零命中；V2 move 语义优先级可后置 |

## 对 591 待澄清的结论（已回填 #4）

1. **T2（Option/Result 语义）是最高杠杆**——解锁 url `host_str`/`port` 直出、
   regex `find`/`captures`、serde 错误路径谓词，且能把「格式化/字符串化面」
   一并收口（DIV-DEP-8 影响**所有** rust-typed 值的断言表达力，是隐性天花板）。
2. **T3（trait 单态化）第二**——直接解锁 base64 整库（当前全红样本），并覆盖
   Display/ToString 家族（semver/url 的 print 面随 T2/T3 联动翻绿）。
3. **by-value-self 零命中**——五库读侧 API 不触 move 语义，V2 的 by-value
   收口不因本数据提级。
4. 方法论注记：五库全在 `BUILTIN_OPAQUE_CRATES`——「真三方库动态加载」在
   类型面实为 VM 内置实现 vs 真库对拍；自由函数面才走真编译 pack。后续
   非 builtin 库（非白名单 crate）的 pack 面命中率应单独立项勘测。
