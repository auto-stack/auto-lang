# TODO: 方法调用未转换为 C 函数调用

## 问题
C transpiler 没有将 AutoLang 的方法调用语法 `obj.method()` 转换为 C 函数调用语法 `Type_method(&obj)`。

当前输出: `b.fly()`
期望输出: `int_fly(b)`

## 根因
`trans/c.rs` 中缺少 method call 到 C 函数调用的转换逻辑。需要在 call() 方法中
检测 `Expr::Dot` 模式并将其转换为 `Type_method(obj, args)` 格式。
