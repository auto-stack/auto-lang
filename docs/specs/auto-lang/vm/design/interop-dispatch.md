# 互操作分发契约（ForeignObject 协议 + 分发组合子）

> 来源：[PLAN-555](../../../../plans/archive/555-script-mode-w1-dispatch-foundation.md)
> （2026-09-05，脚本模式 W1 动态分派地基；设计源
> `docs/design/strategy/script-mode-interop.md` §1/§8/§9）。
> 对应代码：`vm/interop.rs`（协议 + 组合子）、`vm/heap_object.rs`
> （as_foreign_object 钩子）、`py_ffi.rs`（PyObjectHandle 适配器 +
> py 三桥 467-469）、`vm/native_catalog.rs`（ID 段注册）、
> `vm/codegen.rs`（发射形态）。

## 范围

跨宿主对象协议与正常模式分发函数层：脚本模式（`.as`）W2 糖批的
lowering 目标面、跨语言矩阵（JS/GDScript/ArkTS）的承接位。**不进
engine 热臂**（§10 裁决"架构"行：组合子分派，s2s 工具先行）。

## ForeignObject 协议（宿主接入位）

- **接入机制**：`HeapObject::as_foreign_object() -> Option<&dyn ForeignObject>`
  默认 `None`；宿主句柄类型覆写为 `Some(self)`。理由：`as_any` 只能
  downcast 具体类型，无法跨 trait 对象——默认钩子是协议分派的收口点。
- **方法面（六操作）**：`obj_get / obj_set / obj_call / obj_len /
  obj_iter / obj_type_name`，签名统一 `(key/value nvs, &mut AutoTask,
  &AutoVM) -> Result<(), VMError>`，结果推回 task 栈（复用各宿主
  marshal 回程）。
- **预留位**：`obj_send / obj_contains`（W2 挂 B7/C 族时定签名）。
- **首实现**：`py_ffi::PyObjectHandle`（getattr/setattr/call_method/
  len/type_name/iter，GIL 闭包；obj_call 含 BridgeGuard 桥窗口——
  539 T21 回调重入约束）。
- **约定细节**：mutating 操作（set）语句形态推 null 保栈平衡
  （py_setitem 约定）；`obj_call` 的 `pending_native_arg_count` 含已被
  组合子消费的 receiver（实现方按 n-2 实参处理）；obj_set 值封送借道
  push/pop_auto_py_arg（裸 f64 单槽罕见面 → None，P555-D3）。

## 分发组合子（六件，native ID 1860-1865）

| ID | 组合子 | 外对象臂（协议） | Auto 臂（原生方法表） |
|---|---|---|---|
| 1860 | `obj_get(x, key)` | py_getattr 语义 | str[i] 码点 / list[i]（负索引，OOB→IndexError 对标 550）/ map["k"]（缺字段 0 哨兵） |
| 1861 | `obj_set(x, key, v)` | py_setattr（467） | list[i]=v / map["k"]=v（ListData\<i32\>/\<Value\> + ObjectData） |
| 1862 | `obj_call(x, m, ...)` | GIL call_method | **响亮拒绝** "object is not callable"（Auto 闭包动态调用面归 W2，P555-D1） |
| 1863 | `obj_len(x)` | GIL len（468） | ARRAY_LEN 语义（str 字符数 / ListData 长 / ObjectData 字段数） |
| 1864 | `obj_iter(x)` | py_iter 物化迭代器 | list/str 原样回推（array 通道 for-in 消费，设计 E3 Auto 侧） |
| 1865 | `obj_type_name(x)` | type(x).__name__（469） | nv_py_type_name 家族（550）+ "list"/"str"/"map" 容器名 |

- 预留：1866 `obj_send` / 1867 `obj_contains`。
- **注册面**：native_catalog 双条目（限定名 `interop.obj_*` + 裸名
  `obj_*`，惰性命中）；engine `AutoVM::new` 经
  `register_interop_natives` 挂运行时 shim。
- **发射形态**：复用 `is_py_ffi_call → CALL_PY`（带调用点实参数字节，
  shim 按 `pending_native_arg_count` 弹参）——该 flag 实为"计数原生
  调用"通用约定，命名与 py 耦合属历史包袱（P555-D5，W2 顺手改名
  CALL_NAT_COUNTED 类）。

## py 三桥（467-469，539 桥型）

`py_setattr(467)`（B2 桥半）/ `py_len(468)`（B6）/ `py_type_name(469)`
（D8）——GIL 闭包 + `pop_auto_py_arg` + marshal 标准回程；与组合子的
协议臂共享语义（PyObjectHandle 适配器与桥 shim 镜像实现）。

## 测试载具

- 单测：`tests_script_mode`（模式八格）、`tests_s2s`（改写器三件）、
  python 档 `test_w1_dispatch_bridges_registered` /
  `test_w1_foreign_object_protocol_adapter`。
- 探针：`scratch/p555/p5-p7`（三桥端到端 / 组合子 Auto 全矩阵 /
  组合子 py 句柄通道）。
- 门禁基线：py 五套件三方 64/64 零回归（零行为变更红线）。

## 已知边界（债务 P555-D1..D5）

obj_call Auto 臂仅守卫错误 / s2s W1 token 帧形 / obj_set 借道封送罕见面 /
CALL_PY 命名清理 / master charts 既有红甄别——详见 KNOWN-DEBT P555 节。
