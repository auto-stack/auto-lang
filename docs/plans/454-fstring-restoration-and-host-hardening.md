---
plan_id: PLAN-454
status: execution_done
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

### Phase A:宿主 2.4 加固(2026-08-27 完成,部分保留部分证伪)
- [x] A1 读定消费面:IS_VARIANT 回退臂承载 Option 原始载荷编码
  (CREATE_SOME 不包对象,int/str/bool/f64 载荷按 Some 命中是设计)。
- [x] A2 IS_VARIANT 用户变体×标量错误化:**已实施→实证否决→回退**。
  首版收窄(非 Option. 前缀+非对象非 null → RuntimeError)被
  p03_enum_payload 探针击穿——单元变体(Val.VN 无载荷)以裸判别值
  标量合法流入级联 payload 模式测试,"静默 false"是级联语义的一部分;
  运行时无类型元数据,与编码漂移不可区分。按本计划 gating 条款判定:
  **不存在安全收窄空间,维持宽松为终态**(结论与理由以注释固化于
  engine.rs 该臂处)。
- [x] A3 CONSTRUCT_INSTANCE 字符串载荷越界池索引:静默解成 i32 继续
  跑的回退改为显式 RuntimeError(engine.rs)。全量回归:相对当前
  master 基线(25 失败集,含其他会话新黄金漂移)**零新增**;唯一连带
  疑似项 cb_file_read_lines 经隔离复跑证实为并行 flake。

### Phase B:AA2R 发射侧收敛(worktree)(2026-08-27 完成)
- [x] B1 扫描器扩展:ar_scan_mutations 解析 FStrPart 文本内 $引用记入 lu 表。
- [x] B2 ar_fstr_parse 的 ${expr} 子 tokenize+ar_expr 翻译(a.ty 快照恢复);
  纯 $Ident 保持直通(format! 借用语义)。
- [x] B3 语料 g08_fstr_interp(更名自草稿 g09)三形态文本对齐绿;附带补齐
  ar_return 尾值 as-cast 防析括号镜像宿主;工具性新增 AA2R_DUMP 双向 dump。

### Phase C:f-string 全量还原(2026-08-27 完成,保守安全面)
- [x] C1 转换器入库 tools/fstr_conv.py。
- [x] C2 a2r.at 67 处先行,aavm2 全闸门绿 + ⑤自举产物 rustc 零错硬验证。
- [x] C3 parser39/typeinfo3/codegen15/lexer5/engine1 分批,每批闸门绿;
  七文件合计 130 处。
- [x] C4 终验:全量与基线零差;矩阵 36/36 ×5(②⑤转换后源重建);
  G2 冒烟正确;.expected.out 零变化。
  残余登记:多段方法链拼接需表达式级深度转换器、纯字面量拼接无迁移收益,
  如实留册不硬凑(见 divergences D40 续闭合条)。

### Phase D:收账(2026-08-27 完成)
- [x] D1 divergences D40 续翻转闭合 / A 段结论固化(engine.rs 注释 +
  本计划)/ 状态置 execution_done 待归档。

## 待澄清事项
- IS_VARIANT 收窄判定若遇 Option 小整数载荷编码冲突,按"A 段 gating"
  降级为最窄条件并在注记说明。
