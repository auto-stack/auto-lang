# P572 T5:P532 步骤 9 解锁实证(2026-09-06)

## 解锁面(达成)
- `bash scripts/aavm_native_gen_check.sh --skip-gen1`(exe¹=修复版
  ce20d8ee)成功推进到 exe² 构建段——此前(exe¹=106d6ff)该段
  因 exe¹ --trans 拼合源挂死(3h+ 被杀)不可达。
- 一代判定表:8/8 PASS(b01/b07/b08/b13/b27/b30/b46/b58,
  exe¹ vs 宿主 oracle)。
- exe¹ --trans 拼合源(470230B)完成:**61.29s**,产物 514979B
  (scratch 外部存档 $TEMP/p572_gen2/gen2_body.rs)。

## 阻断面(新暴露,非 572 引入)
exe² cargo build 失败,权威清单(`--message-format=short`,见
errors_short.txt)恰 5 错:

| 位置 | 错误 | 归因 |
|---|---|---|
| 14183:39 | E0308 expected &str found String(`ev_run_files(argv[1].clone())` 实参强转缺) | AA2R 发射缺口(aavm.at main 切片) |
| 14186:20 / 14191:20 | E0423 module `IO` used as value(发射 `IO.read_line()` 点号形态 vs gen2 prelude `mod IO` 需 `::`) | gen2 harness prelude 形态不匹配(P532 脚本侧) |
| 14245:1 | E0428 `main` 重复(拼合源 aavm.at main 已发射,harness 又追加 main) | gen2 harness 设计(P532 脚本侧;⑤腿 merge 模式不双 main) |
| 8576:26 | E0382 borrow of moved value `mc`(cgs.push(mc) 后 return mc,clone 注入缺口) | AA2R 发射缺口(engine.at 切片) |

**非 572 回归的三重证据**:
1. 7 个 lib 单文件 + aavm.at 新旧 exe¹ `--trans` 输出**逐字节一致**
   (token/lexer/parser/typeinfo/engine/a2r/aavm);
2. codegen.at 旧 exe¹ 从未产出(挂死)——其切片缺陷不可能由修复引入;
3. 完成过的源零翻转(t2_findings 构造性证明),修复只改终止性不改发射。

## 二代对拍状态
被 exe² 构建失败阻塞(上表 5 缺陷)。修复后管道可复跑:
`bash scripts/aavm_native_gen_check.sh --skip-gen1`。
缺陷处置归属(572 续修 vs 回 P532)→ 572 待澄清③。

---

# P572 T5b 收口追记(2026-09-06,用户裁定本计划续修)

## 修复面(4 处 a2r.at + 3 处管道脚本)
| 缺陷 | 修复 | 证据 |
|---|---|---|
| E0308 &str 强转缺 | ar_method_call 增 process.args() 原生臂(ty=List<str>,镜像主 a2r 10075/P531 P523-2①) | t5b_repro R1 GREEN(argv[1].as_str()) |
| E0382 push 克隆缺 | push 臂增非 Copy(struct/enum)ident 实参 .clone()(镜像主 a2r 8659) | R2 GREEN;lib 内 11 处结构 push 全获 .clone() |
| E0423 IO 点号形 | 增 IO.read_line() → IO::read_line() 臂(镜像 531 at_mode 形态) | R3 GREEN |
| (双 main/IO 入构建) | 管道:concat 去 aavm.at(镜像⑤腿 lib-only+harness main);harness 补 --files 臂 | exe² 构建绿 |
| **固定点大括号逐代翻倍**(第 6 缺陷,固定点判据暴露) | ar_fstr_parse 字面大括号改直通(源内已是 format 转义形;旧实现二次转义,自举链逐代漂移) | corpus 76/76 不变;宿主形 3 处命中、双重转义 0 |
| exe¹ 选取不确定 | 管道 find|head-1 改 xargs ls -t(曾误选修复前缓存致固定点误判) | 本轮 exe1=71b25d8 确定性 |

## 最终判定(P532 步骤 9 两判据)
- 一代(exe¹ vs 宿主 oracle)8/8 PASS
- **exe² 构建绿**(body 514856B)
- 二代(exe² vs exe¹)8/8 PASS
- **转译固定点 PASS**(exe¹ --trans == exe² --trans,自编译转译≥两代一致
  = 自举闭合)——scratch/p572/gen2/native_gen_table.md 留档

## 闸门(T5b 后)
- ⑤腿 compile_corpus 58/58 ✓(重建 exe¹ 时);compile_use_corpus ✓;
  aavm2_a2r 2/2 ✓;corpus golden 76/76 逐字节不变(brace 修复前后双验)
- 裸 taa 3612/3625,13 失败与基点逐名一致(零回归);tf 3460/3461
  (charts_gallery 预存同款)
