# P572 T2: 最小触发集二分与根因单点指认(2026-09-06)

## 实验记录(exe¹ = aavm2-bin-106d6ff148612181,T1 基线同源)

### 阶段1:28 方法逐一减一(bisect.py --phase 1)
唯一必要成分 = m15 `emit`(剔除即 0.13s 修复);其余 26 个单独均可剔(仍爆)。
N=27 快(无 emit_store)→ emit_store 亦必要。

### 种子对测试
- {emit, emit_store} 二方法单独:**TIMEOUT**(最小引爆集=恰此二方法)
- {emit, line} / {emit_store, line}:0.05s / 0.03s OK(对照)

### 语句级形态(shapes.py + v2 变体组)
| 变体 | emit 体 | store 体 | 结果 |
|---|---|---|---|
| orig | push(I)+len-1 | 完整 if/else if/else 5 调用 | TIMEOUT |
| emit_bare | `return 0` | 完整 | OK |
| emit_push_only | 仅 `self.ins.push(I(...))` | 完整 | TIMEOUT |
| emit_len_only | 仅 `return self.ins.len()-1` | 完整 | OK |
| emit_1param | push(I) 1 参数 | 完整 | TIMEOUT |
| store_one_call | push | 单条 `self.emit(...)` | TIMEOUT |
| store_first_if / call_var / call_arith / ifelse2 / 3seq | push | 各形态 | TIMEOUT |
| c_fieldasn | `self.n_args = n`(字段写) | 单条 self.emit | TIMEOUT |
| b_callfoo | push(I) | 调**非突变**方法 foo | OK |
| d_store_noop | push(I) | 空 | OK |
| e_emit_only | push(I) | (无 B) | OK |
| f_foo_noop | (无 A) | 调 foo | OK |
| g_B_writes_too | push(I) | self 写 + self.emit | **OK** |
| h_chain3 | push(I) | mid(调用)→top(调用 mid) | TIMEOUT |
| i_B_calls_A_twice | push(I) | 两条 self.emit | TIMEOUT |

### 触发面(token 形态归因)
**类型内存在 (A: mutates=1 方法,A 有任意 self 写——`self.f = x` 或
`self.x.push(...)` 内建突变名单命中) + (B: 自身无直接写(mutates=0)且
体内有 `self.A(...)` 调用)** → 死循环。调用图形态:B→A(任意深度链
h_chain3 同爆);字段写/内建 push 可互换(c vs a);方法名无关(b 的
foo 证明:调非突变方法不爆)。

## 根因单点指认

**`auto/lib/a2r.at` L1082-1084 `ar_fixpoint_mutates`:链式写回作用于
值拷贝临时,mutates 位永不持久 → `while grew` 死循环。**

```auto
if ar_scan_self_calls(p, names) == 1 {
    var m3 = a.tys.get(ti).methods.get(k2)
    m3.mutates = 1
    a.tys.get(ti).methods.set(k2, m3)   // ← 丢失:临时副本上的 set
    grew = 1
}
```

- `a.tys.get(ti)` 返回值拷贝;链式 `.methods.set(...)` 落在临时副本上,
  原表不变 → B 的 mutates 位每轮被重新翻转 → `grew` 恒 1 → 无限循环。
- **同型坑在案**:ar_prescan_ext L1288-1294 注释明确记录链式
  `a.tys.get(ti).methods.push` 在主 a2r 转译下成 `.clone().methods.push`
  (变更丢失),修法=显式写回(get→改→set 回,D25 范式);全文件唯一
  违例即 L1084(grep `\.tys\.get.*\.methods\.(set|push)` 验证)。
- 悬崖解释:m1..m27 中凡调用突变方法者自身均有直接写(如 line 调
  self.emit 且写 self.cur_line)→ 无 0→1 翻转;emit_store 是 type CG 内
  首个"自身零写但调用 mutates=1 方法"的方法 → N=28 引爆。
- "超线性"实为**无限循环**:⑤腿语料 58/58 与 a2r.at 自转译(4.3s)不含
  该形态故从未暴露;完成过的源在其上零翻转 → 修复不改变任何既往产物
  (golden 前后逐字节一致的构造性证明)。

## 修法(T3)
按 D25 范式显式写回(镜像 ar_prescan_ext 正例):

```auto
var mty = a.tys.get(ti)
var m3 = mty.methods.get(k2)
m3.mutates = 1
mty.methods.set(k2, m3)
a.tys.set(ti, mty)
```

翻转持久后轮数 ≤ 方法数+1(mutates 位单调),不做轮数上限 band-aid
(上限会截断传递闭包产生错误签名,产物漂移)。
