# 动态类型擦除 MIR（av.dynamic/1）

> 来源 PLAN-824；实际实现 `experimental/ac-av-probe/core/src/{mir,mir_verify,instr_spec,abc}.rs`。

`av.dynamic/1` 在已交付集合 profile 上增加 Dyn 槽和 `DYN_PACK`、`DYN_IS`、`DYN_TAKE`。显式擦除只接受 [动态值合同](../auto-av/dynamic-portable-values.md) 的递归 owned 白名单；无效类型引用、非法嵌套类型及错误槽类型在 verify/ABC 准入门拒绝。准入错误不能借最终执行失败冒充消费隔离；受控错型 take 则属于执行期失败并保持源值。名义类型身份和声明字段、分支及其类型不因相同布局擦除。

语义 MIR、受检 ABC 和执行态布局分层；类型、owner、字段/分支及语义/版本身份不是可剥离的 debug 数据。能力请求与当前宿主授权分开，载荷不能授予自身权限。类型解析、复制、比较和恢复采用共享资源及取消合同；执行器在构造类型之前计量其展开与名称，便携恢复重验当前依赖和结构。源码/native 扩面及任意反射不在此 profile 内。
