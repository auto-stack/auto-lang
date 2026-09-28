# diff 消费端点契约（VM/back envelope 面）

> PLAN-703 供⑤（2026-09-27）。natives 9915/9916/9917——**envelope 逐字
> 段同形**下游 auto-edit `fsys.diff_files_json`/`diff_dirs_json` 契约
> （diff-view.md SD-01，frozen 输入）：下游消费件仅换 back 实现体，
> envelope/视图/矩阵零改动（PLAN-011 替换缝声明、PLAN-015 二次消费实
> 证）。错误=值不 raise（669/687 端点族惯例）。

## 端点总表

| native id | catalog 名 | 裸名 | 签名 | 返回 |
|---|---|---|---|---|
| 9915 | `auto.diff_files` | `diff_files` | `(path_a str, path_b str, ctx int) → str` | 文件 envelope JSON |
| 9916 | `auto.diff_snapshots` | `diff_snapshots` | `(key_a str, key_b str) → str` | 缓冲区 envelope JSON |
| 9917 | `auto.diff_dirs` | `diff_dirs` | `(path_a str, path_b str) → str` | 目录 envelope JSON |

位次注记：9900+ 高段续段；**9914 已被 PLAN-701 供⑥
`auto.shell.add_recent` 占用**——计划草案「9914 起」前提漂移修正（§9
入场记录）。五面注册：for_each_native!（catalog+shim 绑定单源）+
for_each_bigvm_native!（返回类型 String）+ codegen intrinsics（裸名）+
ui_gen/rust.rs（a2r 臂——实现体单源 `diff::envelope`）+ 静态绑定表。

## diff_files envelope（`diff_files_envelope_from_paths`）

```json
{"hunks":[{"a1,a2,b1,b2"}],
 "rows":[{12 字段渲染就绪行}],
 "adds":int, "dels":int, "truncated":false, "degraded":false, "err":""}
```

- **hunks** 0 基半开 `[a1,a2)`，ctx 上下文扩张（默认 3；`ctx<=0` 落
  3；扩张钳位文件界）；归组=非 keep 间距 ≤2·ctx 双坐标同时满足。
- **rows** 12 字段：`lo/ro` 1 基行号（缺席侧 0）；`ln/rn` 全行文本；
  `lk/rk` ∈ {ctx, del, add, ""}（配对行 lk=del/rk=add；单侧行缺席侧
  空串）；三段标记 `lpre/lmid/lpost/rpre/rmid/rpost`——配对行公共前后
  缀裁剪（**refine 恒开**：下游 100k 字符预算系 VM 步墙，Rust 侧无此
  约束）；ctx 行 `lpre`=全文、mid 空；不成对行整行入 mid（Q-4 已接受
  形）。
- **CR 容忍**：universal-newlines 行切分——`\r\n` 剥尾 `\r` 与 `\n`
  等价；行尾孤立 `\r` 同剥（split-on-LF 语义）；尾空元素吸收（尾换行
  不生幻行）。
- **rows 面语义锚（PLAN-704 D-1 清偿成文）**：rows 投影=下游 011
  过渡参考实现逐字段对齐——逐 hunk 切片无重复前导 ctx、变更行必在
  位（纯增形 adds=N ⇔ rows 恰含 N 条 rk="add" 行；纯删同理）、切片
  域不越 hunk 窗；多 hunk/纯增/纯删/删多增少四族与下游参考零漂移
  （对账镜面=auto-edit `tests/evidence-p016-recon.json`，缺陷期漂移
  证据在档）。**换位族=引擎时代语义**（非漂移）：degraded 退场、
  锚点对齐多 hunk 正确脚本（620 行换位=310/310 双向对称+kept 记账
  `len−dels==len−adds`），非过渡参考的降级单 replace 形（610/610）。
  下游 golden 重定轮以此为期望基准。
- **truncated 恒 false**（v1 保留字段）；**degraded 恒 false**（引擎
  时代无降级语义——下游 DP-200/桶积护栏整块 replace 形态随之退场）。
- **错误形**（值不 raise）：文件不存在/读取失败 → `hunks:[], rows:[]`、
  计数 0、`err` 带消息（「文件不存在: …」/「读取失败: …」）。

## diff_snapshots envelope（`diff_snapshots_envelope`）

`key_a/key_b` 直读 buffer registry（`CodeEditorCore::doc_snapshot`——
快照隔离读取面，编辑器继续编辑零锁竞争）；引擎与文件面同管线
（subtree_equal O(1) 快路+prune 包络+单次全局行 diff）。返回
hunk **净形**（`rows:[]`——rows 是文件面的渲染投影；缓冲区面供 hunk
导航，三段标记不涉及）。缺键 → err 形（「编辑器不存在: …」）。

注册惯例勘定：`code_editor()` 按**给定键**插入，payload 访问面按
`storage_key(key)`（`__code_editor_` 前缀）查——VM 轨调用自动经
normalize 对齐。

## diff_dirs envelope（`diff_dirs_envelope`）

```json
{"entries":[{"rel,status,size_a,size_b,is_dir,note"}],
 "counts":{same,added,deleted,modified,binary}, "truncated", "err"}
```

- **左右语义**（BC 同款）：左=path_a=旧（`deleted`=只在左）、右=
  path_b=新（`added`=只在右）。
- **五态分类序**：①存在性（单侧→deleted/added）→②kind 冲突
  （dir↔file→modified）→③二进制启发式（size>0 且 ≤2MB 域读失败/空
  →任一侧命中整条 binary）→④尺寸差→modified→⑤≤2MB 字节等→same /
  \>2MB 同尺寸→`same`+`note="uncompared"`（下游可见注记态保持——引
  擎时代 >2MB 字节比对的矩阵行为变更为后续件）。
- **skip-list**=`fif_skipped` 同语义（`.` 前缀段+target/node_modules/
  gen/dist/build/__pycache__）；rel 统一 `/` 分隔剥根；每目录排序定序；
  metadata 竞态跳条目不炸端点；空目录不产生条目（经 is_dir 条目参与
  存在性）。
- **cap 5000**：超限 `truncated=true` 不静默；**counts 与 entries 同
  域**（惰性迭代器逐条分类计数——截断后计数=已处理域，AC-03）。
- **错误形**：根缺失 → `entries:[]`、`err:"目录不存在: …"` 不静默。
- Rust 面 `DirDiffIter`：惰性分类（per-next 读盘），大目录增量返回、
  consumer 可早停；`DirEntryDiff::hunks(opts)` 惰性内容级（两级模型第
  二级，按需调引擎）。`mtime_fast_path` opt-in 默认关（同尺寸同 mtime
  异内容边缘=下游无此行为的 parity 风险）。

## 三面同步

VM shim（`vm/native.rs` shim_diff_files/snapshots/dirs，`code-editor`
feature 双臂）+ catalog 9915-9917 + a2r/merged 臂（实现体单源
`diff::envelope`，非 Windows 形不适用——纯计算无 FFI）。

## 验证锚

`cargo test -p auto-lang plan703`：探针 7 个（time 族形态）——字段逐
一断言（hunks 形/配对行三段/ctx 行/counts/err 形/CR 容忍/degraded=
false）+ registry 直读 + 缺键/缺根/缺文件错误形。
**PLAN-704 回归锚**：`plan704_rows` 三测试（多 hunk 21 行参考对照/
删多增少 7 行序列/纯增删行存在性——D-1 四漂移族钉死）+
`plan704_d2` 两测试（锚集单调/换位 310/310 对称——D-2）；703 探针
盲区机理注记（八形态 golden=hunks/counts 面无换位形、探针 rows 面
用 modify 形[变更前零 keep 域]——D-1/D-2 恰互漏，故回归组直接钉
漂移族形状）。
