# 原语：`auto.str.byte_len`（UTF-8 字节长度）

> 来源：PLAN-781 阶段 E（T-24，SD-10）。状态：已交付（VM nat#1526 + a2r 双轨）。

## 合同

```
auto.str.byte_len(s str) -> int      # .at 方法形：s.byte_len()
```

返回 `s` 的 **UTF-8 字节数**（Rust `String::len()` 口径）。

- `str.len()` 是 PLAN-055 的**字符数**语义，两者不可互换；
- `str.bytes()` 返回 VM 列表，在大文本（10MiB 书）不可用；
- `str.char_at` 在 VM 轨索引 ≥1 恒 0（已知缺陷），不可作字节寻址。

## 动机与游标空间

Reader 章 meta 索引需要**字节偏移累计**：章窗读用
`File.read_text_range(path, offset, limit)`（nat#1016，PLAN-687——真 seek 窗读，
mid-char 回退、EOF 信封 `{text,total,next_offset}`）。`byte_len` 的字节口径与该
游标空间一致（单测钉死：total/next_offset/mid-char 回退三形态一致）。

## 双轨交付

| 轨 | 形态 |
|---|---|
| VM | nat#1526（1500–1525 实勘占用后取 1526），catalog 三表接线（签名/白名单/别名镜像 is_digit 形态） |
| a2r（vue 转译后端） | `a2r_std::str::str_byte_len` 镜像 + `trans/rust.rs` 降表/类型推断臂；`stdlib/auto/str.rs.at` 声明同 步（`a2r_std_signature_parity` 双侧校验） |

## 边界

- 入参为非 str 时按 VM 动态类型强转语义（与 `str.len()` 一致）；
- a2r 轨无 `char_at` 缺陷影响，但口径以 Rust `String::len()` 为双轨共同基准。
