# AAC 完整语义控制面 (aac-complete-semantics)

Spec 归 PLAN-1049 (SD-01) 交付。描述已落地的 Auto 侧控制效果分类、资源
审计与领域节点分类;闭包/动态/异步与完整资源体系的执行尾为具名后继
(1050/1051/1053)。

## 1. 控制效果分类

- `aac/semantics/control/control.at` 对函数体内的短路 `&&`/`||` 位点给出
  静态分类(`CTL-LAZY <op> <span> RHS=<trap|pure>`):RHS 含除法/取余/调用等
  trap 能力的位点标 `trap`,否则 `pure`。
- 短路右侧不得成为无条件效果;1055 基础 MIR 已按 CFG 惰性执行布尔控制流。
- 分支/循环/早退/错误传播、`?T`/`!T` 区分、闭包 capture 与函数值为具名
  后继(1050/1053)。

## 2. 资源审计(有界)

- `aac/semantics/resources/resources.at` 在绑定前审计函数体的 `.move`
  转移后对同一局部的裸引用,以 `resource.use-after-move` 具名拒绝
  (后端前拒绝);合法近邻(无转移或无转移后读取)通过。
- 完整 borrow/move/drop/alias 事实、跨分支合流、循环回边、foreign 句柄/
  回调/future 限制为具名后继(1050/1051)。

## 3. 领域节点分类

- `aac/semantics/domains/domains.at` 对 Schema/AutoUI/AGE `node { view {
  text ... } }` 声明给出模块/名字/span 分类(`DOMAIN-NODE`),不把领域节点
  静默当作标量运行时代码;无 renderer 时本阶段只签语义,真实 host 执行
  为具名后继(1053)。
