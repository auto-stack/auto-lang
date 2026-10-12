# c_json_app — JSON 消费者应用

组合 `auto.json`（parse / get / as_int / as_string / len / is_valid）做 JSON 解析与提取。
验证 Auto 消费 json stdlib 能力的行为，与 Rust oracle（serde_json）三方一致。

## API

| 函数 | 说明 |
|------|------|
| `get_int(json_str, key)` | 解析 JSON，提取 key 的整数值 |
| `get_str(json_str, key)` | 解析 JSON，提取 key 的字符串值 |
| `get_bool(json_str, key)` | 解析 JSON，提取 key 的布尔值 (0/1) |
| `get_array_len(json_str)` | 解析 JSON 数组，返回长度 |
| `check_valid(json_str)` | 检查 JSON 是否合法 (0/1) |

## 测试用例 (7)

1. `test_get_int` — 从对象提取整数
2. `test_get_str` — 从对象提取字符串
3. `test_get_bool` — 从对象提取布尔值
4. `test_array_len` — 数组长度
5. `test_valid_json` — 合法 JSON 检测
6. `test_invalid_json` — 非法 JSON 检测
7. `test_missing_key` — 缺失 key 返回 0
