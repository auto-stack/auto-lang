# 原语：`auto.type_of`（运行时类型谓词）

> 来源：PLAN-781 阶段 E（T-25 R05-r6/r7，SD-12）。状态：已交付（VM nat#1527；PLAN-956 增加原生 Rust）。

## 合同

```
auto.type_of(x) -> str      # .at 调用形：type_of(x)（裸名注册）
```

返回运行时值的类型标签：

| 返回 | 判定（nanbox tag） |
|---|---|
| `"int"` | i32 / i64 / u64 / BigInt 整数 tag（PLAN-956 补齐宽整数谓词） |
| `"float"` | f32 tag 或 f64（tag 7 / 非装箱形态归一） |
| `"str"` | 字符串池引用 |
| `"bool"` / `"null"` | 布尔 / null |
| `"list"` | 专用列表引用形态 |
| `"object"` | 堆对象（json 对象/数组——**数组字面量为堆对象**） |
| `"future"` | future 引用 |

## 关键语义：0.0 与 0 的可靠分离

json.parse 对 `0.0` 产生 float、对 `0` 产生 int——`type_of` 是唯一可靠判别面：
字符串形态判别（`"" + x` 位模式 vs `x.to_string()` 截断）对浮点 0.0 失真（位模式
恰为 0），且受池/槽混叠（F-781-E02 家族）波及；比较判别（`> -1`）对字符串/负值
有效但放行一切正浮点。消费方（如 Reader meta 数字域 `int && >= 0`）应以本原语
为类型权威。

## 所有权约定（F-781-R07）

多态参数消费必须配平：`pop_arg_nv`（弹出+清槽）+ `StakeGuard::nv`（Drop 恰一次
释放堆 id 与池串份额），同 `shim_json_from_value` 先例。裸 `pop_nv` 只读 tag 会
每调用泄漏一份输入引用。

## 注册边界

- nat#1527 + 裸名 `type_of` 双注册；白名单避让行防动态 `#[vm]` id 撞号
  （T-03 教训：动态计数器爬 15xx/99xx 带，撞号会静默覆盖无关绑定）；
- Rust A2R 裸名映射到 `a2r_std::type_of(&value)`；已实现可序列化标量与
  JSON 值，包含 int/float（保留 0.0）、str/bool/null/object。JSON 数组及生成
  Vec 对应 VM 堆对象，返回 `object`，不改旧数组标签；VM 的专用 list tag
  与 future tag 保留，Rust 本次不宣称支持不透明 Future/任意宿主类型。
- Rust 借用输入，VM 继续 pop_arg_nv + StakeGuard；两轨直接值域测试覆盖
  JSON 标量/数组/对象以及完整 i64/u64 范围，不能用转成字符串后的值代替原值。
