# plan-430 A1/A2: rustdoc JSON 可行性验证与元信息格式 v1

- 日期：2026-08-23；环境：stable 1.97.0 / nightly（本机双工具链）

## 验证结果

1. **stable 1.97 不支持** `--output-format json`（报错要求 `-Zunstable-options`）。
2. **nightly 可用**：`rustdoc +nightly -Zunstable-options --output-format json` 输出
   **format_version 53**，微型探针 crate 解析成功。
3. **v53 结构要点**（A2 解析子集已验证）：
   - 顶层 `{root, index, paths, external_crates, format_version}`，扁平 index；
   - item 的 kind = `inner` 字典的唯一键（function/method/struct/enum/impl/...）；
   - 函数签名在 `inner.function.sig`：`inputs: [(name, type_repr)]`、`output`、
     `header.is_const/is_async/abi`、`generics.params`、`has_body`；
   - 类型表示为嵌套字典（`primitive:"u64"`、`borrowed_ref{is_mutable,type}`、`generic:"Self"`），
     self 的 `&`/`&mut` 可直接判定——**分类器所需信息齐备**。
4. **三方 crate 路径打通**：在其项目内 `cargo +nightly rustdoc -Zunstable-options
   --output-format json` 即可（40 个默认 crate 均适用）。
5. **std 本体暂未打通**：`cargo +nightly rustdoc -p std --lib -Z build-std=std ...`
   在本机报 "package ID specification `std` did not match any packages"
   （已试 config 表/CLI flag/RUSTC_BOOTSTRAP 三变体；cargo-public-api 项目有专门处理，未复刻）。
   **决策**：std 起步不走 rustdoc——429-B1 已精确枚举 std 方法面（66 个），元信息 v1 对 std
   直接手写例外表/半自动生成；后续打通配方后再自动化。std 源码已确认在机
   （`rustc --print sysroot` 下 `library/std/src`），syn 扫描作为备选。

## 元信息格式 v1（定稿）

shim 包目录（每 crate 一个）：
```
<crate>.shimpack/
  manifest.json     # crate 名/版本/工具链指纹/生成器版本
  signatures.json   # rustdoc 提取的方法签名子集(见下)
  rules.json        # 分类器默认规则版本号 + 该 crate 的例外条目
  src/              # 生成的 shim Rust 源码(std:编入 auto-lang 的追加段;三方:cdylib crate)
```

signatures.json 条目（从 v53 投影）：
```json
{"type":"Vec","method":"push","self":"&mut","params":["i32"],"ret":"void",
 "generic":false,"call":"method"}
```
marshalling 计划由规则层推导后并入条目：`{"marshal":{"ret":"scalar_i32"|"opaque",...}}`。

## 风险与决策记录

- format_version 53 为不稳定版本号 → manifest.json 记录 nightly 工具链指纹，跨版本重新生成；
- stable 不支持 → 元信息工具锁定 nightly（CI 预生成规避普通用户依赖 nightly）；
- std rustdoc 配方 → 挂账，v1 用 B1 报告 + 手写例外表替代，不影响 Phase E 交付。
