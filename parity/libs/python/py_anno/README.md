# py_anno — W3 注解预言机套件（Plan 567 T20）

验证 `use.py` 导入函数返回注解的**预言机消费链**：

- 注册期 `typing.get_type_hints` 内省（`inspect_return_annotation`）→
  `PySignature.returns` + 全局 `PY_RETURN_ANNOTATIONS` 表；
- **D4 授权强制**：`-> float` / `-> int` 承诺的 shim 出口走 GIL
  `float()` / `int()` 强制（撒谎注解退回 Python 行为，不做正确性地基）；
- **nullability**：`-> int | None` / `Optional[T]` —— None 是值封 null；
  未判空直用时编译期 W0010 lint（log::warn，不阻断）；
- 无注解函数行为零变化（backward 兼容——其余 19 个 py 套件即回归面）。

## 本地模块

`tests/python/anno_mod.py` 为注解夹具模块——parity runner 对 Python 模式
套件注入 `PYTHONPATH=tests/python`（VM/a2py 两个后端），编译期
`use.py anno_mod: ...` 直接种入预言机。
