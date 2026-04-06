# TODO: 头文件名称生成错误

## 问题
生成的 C 头文件 include 路径使用了错误的文件名。

当前输出: `#include "tag_types.h"`
期望输出: `#include "hetero_enum_types.h"`

## 根因
`trans/c.rs` 中头文件名称生成逻辑没有使用正确的测试用例名称。
