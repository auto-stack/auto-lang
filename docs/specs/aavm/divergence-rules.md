# AAVM v2 divergence 规则与编码规范（plan-431 Phase C + E4）

## 1. 总原则

语句级 1:1 优先;确无 Auto 对应时允许结构性改写,但**每处必须记录**。
433 四向对比(Rust 参考 / AAVM-VM / a2r 产物 / 旧 AAVM)时,diff 必须能由
`docs/specs/aavm/divergences.md` 的登记逐条解释。

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
2. 汇总登记:`docs/specs/aavm/divergences.md`(432 起随移植进度维护),
   按文件分节,含计数与理由;433 对比 runner 把它作为"合法差异白名单"输入。

## 4. 纯 Rust 模式编码规范（C3）

1. **只准 `use.rust` 直调 std/三方**,禁用 Auto 自身 stdlib(auto.xxx/Result.*
   /List.*/Map.* 等 v1 便利层);
2. 函数名、结构体字段名**保留 Rust 原名**(snake_case 原样),便于逐函数对照;
   类型名 PascalCase 原样;
3. 文件头用 §5 Snapshot 模板;文件内函数顺序与基线快照一致(对照 review 友好);
4. **违规检测(lint)**:v2 runner(test_aavm2)在拼接语料后追加一个检查 pass
   ——对 `auto.`/`Result.`/`List.`/`Map.` 前缀调用做正则扫描,命中即 fail
   (实现挂 432 的 runner 迭代,规则先在此定稿)。

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
