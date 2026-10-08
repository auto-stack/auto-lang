"""Apply only PLAN-745's reviewed knowledge delta in its original worktree."""
import copy
import hashlib
import json
import re
import sys
from pathlib import Path

root = Path(sys.argv[1]).resolve()
assert root == Path('D:/autostack/.wt/lang-745/auto-lang').resolve()
packet = Path(__file__).resolve().parent

def edit(path, before, after):
    target = root / path
    text = target.read_text(encoding='utf-8')
    assert before in text, path
    target.write_text(text.replace(before, after, 1), encoding='utf-8', newline='\n')

source_spec = '''# Auto 源码 core-i32 profile

> **Status**: experimental；`auto.source.core-i32.draft` revision 1（PLAN-745）。
> 实现：`experimental/ac-core/src/source/`；消费端见 [project.md](project.md)。

本 profile 提供单模块 Auto 计算源码 → Checked HIR → Windows x64 PE 的有限域闭环。
它不替代生产 parser/VM/A2X，不证明完整 Auto 语言支持或 ACC native 自举。

## 数值与支持面

- `int`/`i32` 同义，精确有符号32位；Bool 与 I32 不隐式互转。
- 无后缀十进制字面量域 [-2147483648,2147483647]；越界报
  `text/source.literal-range`。`-2147483648`/`- 2147483648` 合法；负号只用于
  直接后随数字的字面量，`-x`/`-(expr)` 排除，不提供通用取负。
- 运算仅 `*` > `+` > `<`，左结合，lt 为 signed；加乘 signed 溢出运行期
  ExitProcess(70)，包括普通常量表达式；不编译期求值，不提升未执行分支 trap。
  70 是测试 profile 约定，不是全语言异常 ABI；完整 int 宽度继续 open。
- 函数参数/返回必须注解 int/i32/bool；let/var 始终初始化，可省类型注解；
  let 不可写，var 可写；同块重名拒绝，内层初始化 RHS 先读原环境，再绑定新名字。
- 有值 return、if/else、while、loop、无标签 break/continue，目标为最近外层循环。
  全文件函数均检查，含未调用函数/未执行分支；允许前向/直接/相互递归。
- 调用只允许全 positional 或全 named；实参严格按源码顺序求值，参数绑定单独
  建立双射。用户函数与局部绑定优先，不能被测试 intrinsic 劫持。
- ASCII 名字；完整 Auto 关键词不可作名字。语句用换行/分号，支持 `//`；
  未括起表达式的相邻 token 不跨语句换行；括号/实参表内换行是空白，闭括号
  后可在其所在行续算。callee 与 `(` 不跨换行。
- 没有固定嵌套计数排除：表达式/调用/块解析、Drop、resolve/typecheck、adapter
  （终止分类 memo）、verify 数据流、native 发射均显式迭代。合法深输入与未闭合
  深输入的正反验收保持；这不承诺无限内存或任意规模性能。

## 排除与拒绝

char/str/float、其它整数宽度/后缀/进制、聚合/泛型/方法/多模块、for、标签、
全序整数/逻辑运算、无值表达式语句不支持。识别但排除的 token 为
`text/source.unsupported`；坏 token 为 source.lex，坏语法/未闭合/剩余输入为
source.syntax。名字错误在 bind（undefined/duplicate/argument/not-callable），精确
类型错误在 verify/source.type-mismatch；后段保留既有 bind.*/verify.* 诊断。
verify 保守初始化、块唯一角色/可达/无环、全路径返回、终结后死语句规则仍适用。

## 来源与可信边界

源码 profile 显式选择；source AST/签名/词法与类型解析 → 生成 Atom 文本和来源表
→ 真正 bind_source → 全 Bundle span 投影 → verify 返回 CheckedModule → 共享
finish_build → native/暂存链接/事务发布。后端只接受本进程 verify 凭证，无旧 AST、
VM/A2R 回退，不补名字/类型检查；HIR 三重身份
`auto.hir.core.draft / 1 / core-i32-draft`、descriptor/运算面不变。
生成条目与 token 映射覆盖所有来源；bool 合成比较和 while 降低指回原 literal/条件。
范围/UTF8边界/无生成位置残留检查失败受控拒绝，缺映射不伪造位置、不 panic。
所有错误指向原 .at 冒犯 token/EOF，保留 UTF8/CRLF 行列与阶段/code。
未闭合输入不能产生 Checked 或发布制品；不能静默丢弃未闭合的调用帧。

## CLI、制品与测试能力

`check-source <file> --source-profile auto.source.core-i32.draft`；
`build-source <file> --source-profile auto.source.core-i32.draft --entry <源码函数名>
--output <exe> [--capability N]... [--support-lib L]...`。
profile 漏/未知是用法错2；成功/流水线拒绝/用法错为0/1/2，输入模式不按扩展名猜测。
build 入口要求零参数 int/i32；源码函数 native 符号加 sf_，允许用户使用 ac_start/
ExitProcess 名字，绑定仍走 DefId。源码 profile/输入 hash 与 exe/obj/link 收据同事务
发布；所有失败保留原三件，事务与硬截止进程约束见 [project.md](project.md)。
mark_a/mark_b 仅显式 `hir.test.trace.v1` 激活，nullary→int；用户声明优先。
原生 trace 支持库须显式链接，不是生产 I/O/运行时契约。

## 验收与范围来源

`scripts/verify-ac-source-745.ps1`：13步/74例（21真实PE，53阶段/code反例）；
source_frontend/hir/native/cli、deep_regression 覆盖 token、语义、求值顺序、trap、
真实Bundle变异、事务SENTINEL及块/表达式/调用深输入。
最终独立复审另含逗号/未闭合帧与24项块流控制：
[报告](../../plans/attachments/745/review-r7/REVIEW.md)。
数值取证：[numeric-boundary.md](../../reports/745-source-core-i32/numeric-boundary.md)；
实现收据：[verification.md](../../reports/745-source-core-i32/verification.md)；
过程：[PLAN-745](../../plans/archive/745-ac-source-checked-hir-native.md)。
完整宽度/char语义及ACC主体/Gen1–3仍 open，未重开 GOAL-017。
'''
path = root / 'docs/specs/auto-ac/source-core-i32.md'
assert not path.exists()
path.write_text(source_spec, encoding='utf-8', newline='\n')

edit('docs/specs/auto-hir/stage-contract.md',
     'S0 源码工作态（未实现）→ S1 Schema-bound Atom 文本（已实现）→ S2 bound Bundle',
     'S0 源码工作态（仅 PLAN-745 的 auto.source.core-i32.draft 已实现；源码 →\nspan token/AST → 词法/类型解析）→ S1 Schema-bound Atom 文本（已实现；\n该源码 profile 生成文本及来源表）→ S2 bound Bundle')
edit('docs/specs/auto-hir/stage-contract.md',
     '规则：R1 阶段身份随行、禁止跨阶段取用；',
     '源码入口与全 Bundle 来源投影见 [source-core-i32.md](../auto-ac/source-core-i32.md)。\n' 
     'verify 仍为 Checked 唯一构造路径；HIR 三重身份、Schema/运算面与 pass 未实现状态不变。\n\n'
     '规则：R1 阶段身份随行、禁止跨阶段取用；')

edit('docs/specs/auto-ac/project.md',
     '**"HIR → 原生执行"已完成；"Auto 源码 → 原生执行"与 AAC 自举未完成**，由 A1/B/C 后续计划承接。',
     '**HIR → 原生执行与有限源码 profile → 原生执行均已完成**；PLAN-745 的\n'
     '[source-core-i32.md](source-core-i32.md) 提供 A1–A3 源码闭环。完整语言面、AAC/ACC\n'
     'native 自举与其它能力仍由后续计划承接。')
edit('docs/specs/auto-ac/project.md',
     '- 模块内函数名即 COFF 符号：重名在 backend 阶段拒绝；intrinsic 按声明 symbol 文本作 import。',
     '- HIR 模式模块内函数名即 COFF 符号：重名在 backend 阶段拒绝；intrinsic 按声明 symbol 文本作 import。\n'
     '- 源码模式入口按源码函数名查 DefId；源码 native 符号加 sf_ 前缀，用户可合法声明\n'
     '  ac_start/ExitProcess；不改变 HIR 身份/绑定与 OS 入口包装。')
edit('docs/specs/auto-ac/project.md',
     '## CLI 与制品\n',
     '## CLI 与制品\n\n'
     '- 源码模式 check-source/build-source 必须显式 source profile；源码 → Atom/span表\n'
     '  → 真正 bind/verify → Checked → 与 HIR 模式共用 finish_build/native/link/发布尾部。\n'
     '  支持/拒绝、0/1/2、trace、来源映射与无固定嵌套排除见 [源码规范](source-core-i32.md)。\n'
     '  一键 scripts/verify-ac-source-745.ps1，21原生/53拒绝及独立深块/帧控制已验；\n'
     '  源码 profile/输入hash与三制品在同一事务写入收据。\n')
edit('docs/specs/auto-ac/project.md',
     '- 不宣称：Auto 源码编译（旧 AST adapter 未接）、生产 ABI 冻结、DLL/热重载、Linux 发布验收、',
     '- 不宣称：完整 Auto 源码编译（仅独立 source profile，旧 AST 未接）、生产 ABI 冻结、DLL/热重载、Linux 发布验收、')

edit('docs/specs/auto-acc/project.md',
     '## 锚点/候选 owner 一致性（QA-05）',
     '## 有限源码 profile 取证（PLAN-745）\n\n'
     'Rust AC 原型已有 [source-core-i32](../auto-ac/source-core-i32.md) 的源码 →\n'
     'Checked → native 闭环；它不等于 Auto 实现的 ACC 主体迁移、Gen1/2/3或完整 required 能力。\n'
     'MD-409 仅收敛该 profile 的 int/i32=有符号32位、add/mul/lt与runtime trap；\n'
     '全语言 int 宽度与全序运算仍 open/required。MD-408 的 char 字面量/注解/转换\n'
     '在 profile 内受控拒绝；三种可见宿主表示不能证明统一 char↔int 映射。\n'
     '两决定追加取证 note，required conclusion及原绑定保持；unknown族 owner/探针不变。\n'
     '数值证据见 [numeric-boundary.md](../../reports/745-source-core-i32/numeric-boundary.md)。\n\n'
     '## 锚点/候选 owner 一致性（QA-05）')

for module, summary in {
    'auto-hir': '仅source profile S0接入；生成Atom+来源投影经真bind/verify到Checked，HIR身份/Schema/pass边界保持',
    'auto-ac': '单模块Auto计算源码→Checked→Windows PE；精确I32/Bool、求值序/runtime trap、事务发布与迭代深块正反验收',
    'auto-acc': 'MD408/409追加有限profile数值取证；完整int/char与ACC主体/Gen1–3仍open，不升级required清单',
}.items():
    p = root / f'docs/specs/{module}/plans.md'
    text = p.read_text(encoding='utf-8')
    assert '| 745 |' not in text
    text += f'| 745 | ac-source-checked-hir-native | ✅（delivered，r1；第七轮最终复审pass） | [745-ac-source-checked-hir-native.md](../../plans/archive/745-ac-source-checked-hir-native.md) | {summary} |\n'
    p.write_text(text, encoding='utf-8', newline='\n')
edit('docs/specs/overview.md',
     'PLAN-741；HIR→exe 完成，源码→exe 未完成',
     'PLAN-741/745；HIR与显式core-i32源码profile→exe完成，完整语言与自举仍open')

# Preserve all manual semantic fields/bindings and unknown-family consumers.
p = root / 'docs/reports/743-acc-hir-contract/manual-decisions.json'
data = json.loads(p.read_text(encoding='utf-8'))
old = copy.deepcopy(data)
frozen = (packet.parent / 'review-r7/proposed-spec-delta.frozen.md').read_text(encoding='utf-8-sig')
notes = re.findall(r'`(〔745[^\n]+?〕)`', frozen)
assert len(notes) == 2
for ident, note in zip(('MD-408', 'MD-409'), notes):
    rec = next(r for r in data['decisions'] if r['id'] == ident)
    assert note not in rec['note']
    rec['note'] += note
for a, b in zip(old['decisions'], data['decisions']):
    if a['id'] in ('MD-408', 'MD-409'):
        assert {k:v for k,v in a.items() if k != 'note'} == {k:v for k,v in b.items() if k != 'note'}
    else:
        assert a == b
assert {k:v for k,v in old.items() if k != 'decisions'} == {k:v for k,v in data.items() if k != 'decisions'}
p.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
assert hashlib.sha256((root/'docs/specs/auto-hir/project.md').read_bytes()).hexdigest() == '8ada42d7024af2c8104733a31ab7c857b0a79d2646228bcd73e12d89e9f1a5ab'
(packet/'manual-note-audit.json').write_text(json.dumps({'ids':['MD-408','MD-409'], 'only_notes_changed':True,'bindings_unchanged':True,'consumers_unchanged':True}, indent=2)+'\n', encoding='utf-8')
print('Reviewed SD01..04 + module navigation/overview prepared; only two manual notes changed.')
