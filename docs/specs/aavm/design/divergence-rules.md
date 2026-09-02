# AAVM v2 divergence 规则与编码规范（plan-431 Phase C + E4）

## 1. 总原则

语句级 1:1 优先;确无 Auto 对应时允许结构性改写,但**每处必须记录**。
433 四向对比(Rust 参考 / AAVM-VM / a2r 产物 / 旧 AAVM)时,diff 必须能由
`docs/specs/aavm/design/divergences.md` 的登记逐条解释。

## 2. 允许的改写模式清单（C1 首版）

| # | Rust 惯用法 | Auto 改写 | 备注 |
|---|---|---|---|
| D1 | `Peekable<Chars>` | `Vec<char>` + 下标游标 | lexer 主结构;peek=读 vec[i],advance=i+=1 |
| D2 | `Rc`/`Arc` | 直接值 / `Box` | AAVM 单线程域内;引用计数语义不计 |
| D3 | `Box<dyn HeapObject>` 等 trait 对象 | type struct + kind 鉴别器字段 | Auto 无 trait 对象;switch on kind |
| D4 | 迭代器链（map/filter/collect） | 显式 for 循环 | 保留语义,逐元素 append |
| D5 | Rust 宏（matches!/vec!/write! 等） | 展开后的等价代码 | 以基线快照展开为准 |
| D6 | `?` 传播 | 显式 match(或 Auto `.?`,以 429-B3 盘点可用性为准) | B3:a2r 侧 `?` 直通已有 |
| D7 | `&mut` 借用切片/窗口 | 索引区间 (start, end) 传参 | 避免双借用检查 |
| D8 | 模式匹配中的绑定@模式/guard 复杂形态 | 拆为嵌套 match + 临时变量 | 仅在 Auto 模式语法不支持时 |
| D9 | `usize` 算术 | Auto int(i64) | 边界处 as usize(与 plan-430 规则 6 对齐,不做有损截断) |

新发现的模式照此表追加编号,不改已定条目。

## 3. 强制记录格式（C2）

1. `.at` 文件内:`// DIVERGE(D3): Box<dyn Any> → kind 鉴别器,heap 对象三形态`
   ——编号引用 §2 表,冒号后写"原 → 改 + 一句理由";
2. 汇总登记:`docs/specs/aavm/design/divergences.md`(432 起随移植进度维护),
   按文件分节,含计数与理由;433 对比 runner 把它作为"合法差异白名单"输入。

## 4. 纯 Rust 模式编码规范（C3）

1. **只准 `use.rs` 直调 std/三方**,禁用 Auto 自身 stdlib(auto.xxx/Result.*
   /List.*/Map.* 等 v1 便利层);
2. 函数名、结构体字段名**保留 Rust 原名**(snake_case 原样),便于逐函数对照;
   类型名 PascalCase 原样;
3. 文件头用 §5 Snapshot 模板;文件内函数顺序与基线快照一致(对照 review 友好);
   修正(plan-432 S2 考古):**List.* 为 sanctioned**(a2r 直译 List→Vec,
   trans/rust.rs:1888;VM 侧 auto.list.* 可装结构体)——禁用清单收窄为
   auto.*/Result.*/Map.* 便利层;
4. **违规检测(lint)**:v2 runner(test_aavm2)在拼接语料后追加一个检查 pass
   ——对 `auto.`/`Result.`/`List.`/`Map.` 前缀调用做正则扫描,命中即 fail
   (实现挂 432 的 runner 迭代,规则先在此定稿)。

### 4a. is-match / 枚举载荷写法规范(plan-447 部分③ 起,γ 系列改写适用)

> 依据 idiom-upgrade-prereqs.md §2/§7 实证(2026-08-25)+ 部分① 执行记录
> (H1-H6 修复后复验)。lib 七文件从此允许并鼓励 is/on-enum 载荷写法,
> 但以下边界逐条强制:

5. **单行枚举声明需逗号 + 显式值**,无值变体必须逐行:
   `enum X { A = 1, B = 2 }` 合法;`enum X { A, B }`(无值单行)非法。
   无载荷变体逐行声明是 lib 既有形态,不因风格升级改变;
   载荷变体(元组/结构形)同样逐行。
6. **Auto 无"模式 + guard"组合臂**:Rust `VI(n) if n > 10 =>` 必须拆为
   独立卫语臂(`if n > 10 ->`)或调整臂序(先载荷绑定臂内 if)。
   **永久 DIVERGE(D41)**,登记于 divergences.md;宿主 H4 修复保证
   独立卫语臂的 a2r 发射正确(`<绑定> if <cond> =>`)。
7. **同函数对同一枚举值多次 is**:VM 侧无碍;a2r 侧依赖 H5 修复
   (≥2 次同标识符 scrutinee 时自动发 `match &v`)。允许直接使用,
   不再要求"多次传参或局部复制"的旧规避(prereqs §7-3 作废)。
8. **枚举构造参数位内联运行期计算值**:H3 复现件在 plan-442 RC 死区修复
   (f81e18c8e)合入后已不复发(p09h/p13b 不提升局部也正确),
   prereqs §7-4 的"先提升局部"规范**解除**;Val 枚举化(Phase 10)前
   仍以 repro_242_string_pool_uaf 等 RC 回归护航。
9. **is 值语义仅限函数尾位**:函数尾直接 `is x {...}` 作返回值 ✅
   (p01/p02b 实证);`let r = is x {...}` 位当前返回 0(部分① 观察项 1,
   KNOWN-DEBT 登记在册)——lib 内 is 一律语句位书写,不用 let 位值语义。

## 5. `.at` 文件头 Snapshot 模板（E4 定稿）

```auto
// Rust ref: crates/auto-lang/src/token.rs @ b3bd64f5 (v0.5 临时基线)
// Baseline: b3bd64f5   (v0.5 tag 落地后重锚定,见 429-c1-baseline.md)
// Coverage: 138/140 TokenKind   (未覆盖: COMMENT_RAW, XXX)
// Missing:  <无 | 逐条列函数/分支>
// DIVERGE:  D1 ×2, D5 ×1        (明细见文件内注释 + divergences.md)
```

五个字段全部必填;Coverage 用"已移植/基线总量"计数;tag 重锚定时
Baseline 行更新并重跑对照。

### 4b. 方法化写法规范（plan-514 W2/W3 起，γ4 适用）

> 依据 plan-514 W3 六文件裁定（2026-09-02）+ W2 方法发射批。lib 从此
> 允许并鼓励 type 体方法写法，边界逐条强制：

10. **状态类型自有操作进 `type` 体**：游标/发射器状态（P/CG/Ar 族）的
    c-only 操作函数入 type 体作方法（`fn cg_emit(c, idx)` → `c.emit(idx)`）；
    产生式（带 p 游标的 parse_*/cg_*/ar_* 步行族）与纯表函数
    （keyword_kind/op_name/i_size 族）**保留自由函数**——映射以"一对一
    Rust 对译可读性"为准，不设 100% 消灭。
11. **方法名与字段撞名改写**：`p_err→fail`（P/CG/Ar 均有 err 字段）同款
    规则；构造 `cg_new()→static fn new()`（`Type.new(x)` 双侧发射
    `Type::new(x)`）。
12. **方法体内前导点**：块首语句豁免；**非块首语句位的 `.method()` 必须
    显式 `self.`**（流式链糖已移除，P514-W3-1；换行句首点报语法错）；
    表达式位/赋值位前导点（`.field`/`.method(args)` 读位）自由。
13. **跨方法 &mut 传递**：方法体内调用同类型可变方法（expect→next 族）
    接收者自动 &mut（主 a2r compute_type_mut_methods 传递闭包 + AA2R
    ar_fixpoint_mutates 镜像）——书写无需标注，行为两侧一致。
14. **多行管道用 `|>`**：`x |> .m() |> .f`（方法形+字段投影形）；
    函数形二期、is 臂内不支持（D-pipe-first 登记 divergences.md）。
