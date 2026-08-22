# 交接:Plan 417-E2 追随批 —— a2r 关联类型体内引用 + 组合盲区

> **状态(2026-08-22 追随批完成)**:F1 ✅ F3 ✅ F4 ✅ 已全部落地(分支
> plan-fix/417e2-followup);F2 保持设计边界登记;F5 已由并行会话补齐。
> 下文保留原始调查记录供溯源,勿重做。

## F1. ✅ 已修:a2r 实现者方法体内/签名里的关联类型引用替换

VM 侧动态类型天然宽容(实测体内 `var x Item`、E4 合成默认方法体内调用链均
正确,探针输出 8/7/7);**a2r 侧静态类型必坏**:

```auto
type IntBox as Container<Item=int> {
    fn first() int {
        var x Item = self.data[0]   // a2r 发射 let mut x: Item —— Rust 编不过
        return x
    }
    fn get(i int) Item { ... }      // 实现者签名写 Item → impl 块 -> Item 同样坏
}
```

- 复现:`auto trans --path <探针> rust`,产物含裸 `Item`。
- **修复方案(已设计已验证受阻)**:RustTrans 加字段
  `current_assoc_bindings: HashMap<AutoStr, Type>`;在 Stmt::TypeDecl 调用点
  (:9266 一带)从 `type_decl.spec_impls` 收集 assoc_bindings 填入、发射完
  恢复(用 mem::take 保存/回写,防 type_decl 内部早退泄漏);
  `rust_type_name` User 臂在 `dyn `/`Self::` 检查之后、spec_decls 之前加
  `else if let Some(bound_ty) = self.current_assoc_bindings.get(name.as_str())
  { self.rust_type_name(bound_ty) }`。绑定是类型级的,整个 TypeDecl 发射期间
  生效(签名/局部标注/字段类型统一覆盖)。注意与 E3 在同臂新增的
  `current_fn_type_params` 分支共存(E3 在前,assoc 在后均可,语义不冲突)。
- 本次实施细节:字段+两处构造器初始化已写入后因主 worktree 进入 E3 合并
  UU 现场而**精确回滚**(规范:并行 WIP 勿动)。
- 验收:上方探针产物 `let mut x: i64`;golden 011 扩一体内引用用例;
  trait_advanced 三方保持 14/14。

## F2. has-Spec 委托 + 关联类型:语言无绑定入口(设计边界,登记即可)

`type X has SomeSpec` 的 has 子句只接受类型列表(parser :9234 一带),无法
携带 `Item=int` 绑定。若 spec 声明了关联类型而实现者走 has 委托:
AutoVM 正常(动态);a2r 的 `impl Trait for X` 缺 `type Item = ...;` 时 Rust
编不过。处置:文档化为已知限制;若未来需要,给 has 子句扩展命名绑定语法。

## F3. ✅ 已补:混合"泛型 spec + 关联类型" golden 012

`spec Store<T> { type Item ... }` + `type S as Store<int, Item=str>`:
四层代码路径均支持(位置式 int→type_args、Item=str→assoc_bindings 分流,
trait_checker 分别校验,a2r 分别发射 `impl Store<int>` + `type Item =
str;`),但 golden/单测/parity 全部只测了纯关联类型场景。追随批可加一个
混合用例锁定。(注:spec 签名里 T 本身的替换是既有 TODO,E3 领域,勿混。)

## F4. ✅ 已补:parity trait_advanced wrapper 变量名 `data` 规避原因已入注

KNOWN-DEBT-AND-RISKS 已登记根因(a2r `fix_vec_i32_index` Pattern 2 对任意
`xxx.get(i)` 盲重写),但 lib 源文件里 wrapper 局部变量叫 `data` 的原因
只有 commit message 知道——**改名(如改回 bx)会立刻踩坑**。追随批在
lib 注释与 README gotchas 段补一句(prose-only,README 有注释规范)。
根治方向:该正则启发式感知 receiver 类型(local_var_types)。

## F5. ✅ 已消:master 预存 E0583(tests_string_pool 已由并行会话补齐)

`e8672fdd`(字符串池批)在 `vm.rs:19` 声明 `#[cfg(test)] mod
tests_string_pool;` 但**文件未进提交**(提交信息称已新增回归锁)。
影响:`cargo test -p auto-lang --lib` 编译失败(E0583),非测试构建不受
影响。本审查批验证时曾以未提交的 placeholder 垫过。归该并行会话补齐。

## 审查批已修(7b7ced31,勿重做)

- `SpecDecl::write_atom`:assoc_type 段括号多一个右括号 → 平衡;空
  associated_types 原会多输出 `assoc_types([])` 改变全部现存 spec 的 atom
  形状 → 非空才输出(bounds 同例);+2 单测锁定两种形状(括号平衡断言)。
