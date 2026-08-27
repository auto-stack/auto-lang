---
plan_id: PLAN-454
status: drafting
feature_name: 447 尾巴清偿——D40 续修复/f-string 全量还原 + 宿主 2.4 加固
author: [zhaopuming]
created_at: 2026-08-27T01:00:00+08:00
updated_at: 2026-08-27T01:00:00+08:00

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 0
total_steps: 12
---

# [PLAN-454] 447 尾巴清偿:D40 续修复 + f-string 全量还原 + 宿主 2.4 加固

## 变更摘要

清偿 447 归档时登记的两项真实未竟:①AA2R FStrPart 直通三缺口
(D40 续:E0599 方法改写/E0308 借用适配/E0382 last-use 失明)修复后,
重启 lib 的 f-string 批量还原(约 330 行拼接);②宿主 2.4 可选加固
(IS_VARIANT 非实例静默回退、CONSTRUCT_INSTANCE 载荷回退改显式错误)。

## 目标 / 架构方案

**B 段(AA2R 收敛)核心设计**:
- 缺口③(last-use 失明):ar_lu_after / ar_scan_mutations 扫描器在遇到
  FStrPart token 时,解析其文本内的 $引用标识符并计入使用——变量可见性
  恢复,前行赋值的 clone 注入重新生效;
- 缺口①②(方法改写/借用适配):ar_fstr_parse 的 ${expr} 段不再原文直通,
  改为子 tokenize + ar_expr 翻译(a.ty/a.strk 快照恢复);纯标识符段保持
  直通(format!/println! 按借用语义,无移动面);
- 三形态语料 g09(method-call 插值/参数借用插值/先行 move+插值复用)
  进 corpus_a2r 文本对齐闸门,钉死能力;
- 转换器(%TEMP%/p447/fstr_conv.py 迁入工作区 tools)分批应用:
  先 a2r.at 单文件 + 手工⑤自举 rustc 验证,再其余六文件,每批全闸门。

**A 段(宿主加固)**:engine.rs IS_VARIANT 对不可能构成 Option 载荷的
输入(Tag 域外/负哨兵域)改显式 RuntimeError;CONSTRUCT_INSTANCE 未知
载荷 tag 回退改报错。gating:若全量/矩阵暴露既有合法消费面依赖宽松
行为,则收窄条件而非强推。

## 测试设计与验收标准

- aavm2 13+2 闸门每批保持绿;g09 文本对齐绿;
- 批量后:②列产物 cargo build 零错;⑤列自举产物 rustc 零错;
- 五方矩阵 ×5 全绿;G2 双演示不变;.expected.out 零变化;
- 宿主加固:A 段探针(vm_file_tests 或单元)常驻;全量测试相对基线
  零新增失败(musk 联动风险在提交说明中声明)。
- 收账:D40 续条目翻转为已闭合(附 g09 引用)、2.4 完成、计划归档。

## 执行步骤

### Phase A:宿主 2.4 加固(auto-lang 主仓直接小步)
- [ ] A1 读定 IS_VARIANT 当前回退条件与合法消费面(Option 编码路径/
  corpus/musk 常驻探针),确定收窄判定式。
- [ ] A2 实现 IS_VARIANT 显式错误化 + 探针;全量回归。
- [ ] A3 CONSTRUCT_INSTANCE 未知载荷 tag 回退改报错 + 探针;全量回归。

### Phase B:AA2R 发射侧收敛(worktree)
- [ ] B1 扫描器扩展(ar_lu_after/ar_scan_mutations 的 FStrPart 文本计数)。
- [ ] B2 ar_fstr_parse 的 ${expr} 子翻译(a.ty 快照恢复;print/value 两路)。
- [ ] B3 g09 语料三形态 + 文本对齐闸门绿。

### Phase C:f-string 全量还原
- [ ] C1 转换器入库(tools/ 或 scripts/)并按新能力校准规则。
- [ ] C2 a2r.at 单文件批量 → aavm2 全量 + ②产物编译 + ⑤自举手工 rustc。
- [ ] C3 其余六文件分批 → 每批 aavm2 闸门。
- [ ] C4 终验:矩阵 ×5 + G2 双演示 + .expected.out 零变化。

### Phase D:收账
- [ ] D1 divergences D40 续翻转闭合(引 g09)/2.4 注记/Snapshot 更新;
  计划归档。

## 待澄清事项
- IS_VARIANT 收窄判定若遇 Option 小整数载荷编码冲突,按"A 段 gating"
  降级为最窄条件并在注记说明。
