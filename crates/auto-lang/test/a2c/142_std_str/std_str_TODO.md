# TODO: printf 格式字符串类型错误

## 问题
字符串类型变量使用了错误的 printf 格式说明符。

当前输出: `printf("%d\n", s1);`
期望输出: `printf("%s\n", s1);`

## 根因
C transpiler 在生成 printf 调用时，没有根据变量类型选择正确的格式说明符。
`str` 类型应使用 `%s`，而非 `%d`。
