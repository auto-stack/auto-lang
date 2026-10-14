# PLAN-782 T-01 探针用例索引（probe-cases）

> oracle 注册表：三部分——(A) 当前基线组合边界探针（已实跑收据）、(B) 前置正控
> （已实跑收据）、(C) 组合正反例冻结 oracle（T-02..T-06 物化为永久测试；实现须命中，
> 不得反读输出改预期）。全部输入字节存于 `probe/baseline-boundary/`（A/B）或随
> fixtures/tests 物化（C）。运行环境：worktree `D:/autostack/.wt/lang-782/auto-lang`，
> 基线 `4568e4ea7`，rustc 1.98.0。

## A. 当前基线组合边界探针（负探针；`negative-run.txt` 全文收据）

| ID | 输入 | 命令 | 当前结果（冻结） |
|---|---|---|---|
| N01 | `type Tok { text Str }`（record 域） | check-source record | **F-A 缺陷**：panic exit 101（`parser.rs:985 unreachable`）。修复后 oracle：`error[text/source.unsupported]: type \`Str\` is not part of this source profile`，exit 1（T-03） |
| N02 | `type Tok { text str }`（record 域） | 同上 | 同 F-A（str 变体） |
| N03 | `type Tok { c char }`（record 域） | 同上 | 同 F-A（char 变体） |
| N04 | `type Pair { left int }`（text 域） | check-source text | `error[text/source.unsupported]: \`type\` is not part of this source profile`，exit 1（受控，冻结） |
| N05r | `fn eat(move p Pair)` + `eat(p.move)`（record 域） | check-source record | `error[text/source.unsupported]: keyword \`move\` cannot be used as a name`，exit 1（受控；782 后该拼写合法化于新域，record 旧域行为不变） |
| N06 | `p.clone()`（record 域） | check-source record | `error[bind/source.undefined]: type \`Pair\` has no method named \`clone\"`（+后续级联 type-mismatch），exit 1（受控；782 后新域合法，旧域不变） |
| N07 | atom 头 `auto.hir.text-record.draft/1/text-record-i32-draft` | check | `error[bind/bind.descriptor-unknown]`，消息列名四旧三元组，exit 1 —— **新身份无碰撞**；注册后此探针反转为 V-01 正例 |
| N08 | `let s Str = "x"`（scalar 域） | check-source scalar | `error[text/source.unsupported]: type \`Str\` is not part of this source profile`，exit 1（受控；F-A 的对照组——证明守卫模式已存在于 param/let 位） |
| N09 | `fn f(s str)`（core 域） | check-source core | 同 N08 形态（str 参数位），exit 1 |

## B. 前置正控（`positive-controls.txt` 收据）

| ID | 内容 | 结果 |
|---|---|---|
| P-ctl-1 | record 旗舰 `fixtures/record-source/valid/p01-records.at` check-source（record 域） | `OK source profile=auto.source.record.draft …` exit 0 |
| P-ctl-2 | `cargo test --locked --test record_hir --test record_source --test text_hir --test text_source` | 21+21+16+10 = **68/68 绿** |

## C. 组合正反例冻结 oracle（实现后物化；区分断言=退出码/stdout/stderr/trace 字节）

### C-01 身份（→V-01，text_record_hir）

- 正：新三元组 CLI check/Atom bind/API bind_source 过；writer roundtrip 语义等。
- 反：四旧域各投一件组合面文档 → `verify.domain`/`verify.profile-op`；三元组改任一维
  → `bind.descriptor-unknown`；writer 对未注册/错域 → `canonical.identity-conflict`。

### C-02 构造与类型（→V-02）

- `Tok { text Str; kind int; c char; at int; len int }` 位置/命名（逆声明序命名）构造；
  trace 按源码序（`ba` 沿 771）；字段哨兵不互换。
- 嵌套 `Wrap { t Tok; n int }`、copyable `Meta { a int; b bool }` 混合。
- 反：同形异名 `Tok2{text,kind,c,at,len}` 互换 → `source.type-mismatch`（两侧具名）；
  `str` 字段 → `verify.field-borrow`；缺/重/未知字段沿 771 码。

### C-03 资源分类（→V-02/V-08）

- 派生断言：`Meta`=copyable（赋值=拷贝、p=q 合法、771 全套回归保持）；`Tok`/`Wrap`
  =资源（p=q 拒 `verify.resource-assign`）。
- 变异：Bundle 改字段 TypeId 造"自称 copyable" → verify 重推导拒，无 Checked。

### C-04 移动与深 clone（→V-03）

- `let t = Tok{...}; eat(t.move)`：move 零新增 buffer 分配（SUMMARY allocs 不增）；
  t 后读 → `verify.use-after-move`；二次 move → `verify.double-move`。
- `let c = t.clone()`：两份独立（不同 id）、`c.text = c.text.to(Str)` 后 t.text 不变；
  正常出口 alloc==free、live=0。
- 反：`p = q`（资源）→ `verify.resource-assign`；`p = q.text.move`（字段 move）
  → `verify.partial-move`；被拒操作 native 调用计数=0。

### C-05 字段借用来源（→V-04）

- 正：`fn head(v Tok) str { return v.text }`（view 参数字段）+ 静态字面量返回。
- 反：局部字段/拥有临时字段/move-in 参数字段/mut 参数字段返回 str →
  `verify.borrow-escape`；两层 wrapper：同源 view+mut → `verify.mut-conflict`（参数对
  fixpoint 传播），异源 → 过。

### C-06 替换与重叠（→V-05）

- `t.text = t.text.to(Str)`（RHS 读旧产新）：新值精确、旧缓冲恰一次 FREE、trace 序
  =RHS 完成后 FREE（mark 先于 FREE）。
- `t = t.clone()`、`t = fresh(t)`（clone 返回）、`t = eat(t.move)`（消费重建）：值正确
  + 旧缓冲恰一次 free + fresh destination 兄弟对照（不重叠时不重复释放）。
- 反：let/view 根替换 → `verify.immutable-write`；字段 view 活跃期间替换/移根 →
  `verify.mut-conflict`；同一表达式先 move 后读 → `verify.use-after-move`。

### C-07 控制流清理（→V-06；source 与直接 HIR 各一腿）

- 动态单臂/双臂局部构造：出口一次清理；终止臂不污染 join（770 R4-03 保持）。
- 嵌套 return move、break/continue 目标出口（loop_exits/RE-ENTRY fixpoint/after-loop
  三态沿 770——owner 扩至资源根）。
- 0 次循环：alloc==free==0。

### C-08 故障与部分输出（→V-07）

- 两 Str 构造 fail_nth=1/2、深 clone fail_nth=1/2、账本 fail_ledger_nth：status 路径
  部分输出逆序回收（FREE 可观察）、旧目标保全、live=0；trap 路径 RT-FAIL 行+exit 70。
- 零长 Str、预算临界（恰容纳 vs 差 1 字节 → `res.-budget` 前/后行为）。

### C-09 顺序与临时（→V-09）

- `&&/||` 跳过含拥有字段临时：跳过侧 alloc/free 均 0；执行侧一次配对；语句尾
  marker 先于 FREE。receiver/命名实参求值序断言。

### C-10 规模与静态（→V-10）

- 256 层含资源嵌套构造→读→Drop：全链完成、live=0、无栈崩；4096 生命周期循环配对。
- malformed partial Drop（构造遗留未评估表达式）→ eval-position 拒（沿 771 R11 教训）；
  CRLF/span、未执行分支非法类型前门拒并原 token 定位。

### C-11 发布与能力（→V-11）

- 三件旧成功制品 + 新域制品：拒绝前不覆盖旧 obj/exe/receipt（741 事务合同）；
  缺库/错 target/API/hash → `runtime.contract-*` 前门；故意失败 fixture 令 gate 失败
  且 exit 区分成功。

### C-12 Token 形状与联合门禁（→V-12）

- Token 微程序（text Str+kind/line/at/len int）：source/HIR 双腿真实 PE，值/trace/
  计数达 C-04..C-09 oracle；enum/List/use 拒；新 gate + 770/771/741/745/767 全绿；
  scope diff（四旧域测试计数与行为零变化）。

## 收据绑定

- `files.sha256`：whole_path_probe.rs + run-output.txt（PROBE-OK exit 0 零警告）。
- `baseline-boundary/negative-run.txt`：N01..N09 原文（含 F-A panic 现场）。
- `baseline-boundary/positive-controls.txt`：68/68。
