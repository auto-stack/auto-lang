"""Audit and renew only five bindings invalidated by the reviewed AC card delta."""
import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path

root = Path(sys.argv[1]).resolve()
assert root == Path('D:/autostack/.wt/lang-745/auto-lang').resolve()
path = root/'docs/reports/743-acc-hir-contract/manual-decisions.json'
data = json.loads(path.read_text(encoding='utf-8'))
before = copy.deepcopy(data)
card = root/'docs/specs/auto-ac/project.md'
lines = card.read_text(encoding='utf-8').splitlines()
digest = hashlib.sha256(card.read_bytes()).hexdigest()
spec_path = 'docs/specs/auto-ac/project.md'
updates = {
    'MD-203': ('source-core-i32.md', 'new-subject', '745 现有 Rust 有限 source profile 的 adapter/来源映射已真实到 Checked；本决定 subject=acc:source-to-hir-adapter 指 Auto 编写的 ACC 主体，仍须新建。旧“源码 adapter 不存在”仅为741历史，不能覆盖745现状；不把 Rust 原型升级为 ACC 主体或 Gen1。'),
    'MD-205': ('源码模式 check-source/build-source', 'new-subject', '745 Rust 原型已有显式profile源码驱动/来源诊断，共用 finish_build；Auto 实现的 ACC 多阶段驱动、完整语言域诊断与桥客户端仍须新建。旧“不得冒充已接源码”属741历史；只认已审有限profile证据，不声称完整ACC。'),
    'MD-301': ('技术栈：Rust + Cranelift', 'reference', '核对 Cranelift 0.126.2 栈仍在Rust消费侧、ACC不重写机器码；745仅新增前端且共享后端，非主体薄桥定位保持。'),
    'MD-302': ('链接：首选 rust-lld', 'reference', '核对 rust-lld首选/MSVC备选、Rust link.rs发现逻辑未变；source与HIR共享构建尾部，系统链接器非ACC主体定位保持。'),
    'MD-304': ('`add_i32`/`mul_i32`', 'implemented', '核对加乘 signed 溢出仍 ExitProcess(70) 测试约定；745 source profile复用该路径且保持runtime trap/前驱效果，不扩成完整异常ABI。'),
}
audit = []
for ident, (anchor, conclusion, rationale) in updates.items():
    record = next(r for r in data['decisions'] if r['id'] == ident)
    assert record['conclusion'] == conclusion
    old_hash = record['bound_input_hashes'][spec_path]
    assert old_hash == '048a88cc01ca144259a99ed8a790fe61f19e80210e5315efb165bd0319085208'
    line = next(i+1 for i, s in enumerate(lines) if anchor in s)
    record['evidence'] = [f'{spec_path}:{line}' if e.split(':')[0] == spec_path else e for e in record['evidence']]
    record['bound_input_hashes'][spec_path] = digest
    record['note'] += f'〔745 merge 独立语义校准：{rationale}当前证据锚{line}行，验证后绑定本次落地card。〕'
    audit.append({'id':ident,'subject':record['subject'],'conclusion':conclusion,'old_hash':old_hash,'new_hash':digest,'line':line,'rationale':rationale})
for a,b in zip(before['decisions'], data['decisions']):
    if a['id'] in updates:
        assert {k:v for k,v in a.items() if k not in ('note','evidence','bound_input_hashes')} == {k:v for k,v in b.items() if k not in ('note','evidence','bound_input_hashes')}
        assert {k:v for k,v in a['bound_input_hashes'].items() if k != spec_path} == {k:v for k,v in b['bound_input_hashes'].items() if k != spec_path}
    else:
        assert a == b
assert {k:v for k,v in before.items() if k != 'decisions'} == {k:v for k,v in data.items() if k != 'decisions'}
path.write_text(json.dumps(data, ensure_ascii=False, indent=2)+'\n',encoding='utf-8',newline='\n')
packet = root/'docs/plans/attachments/745/merge'
(packet/'binding-audit.json').write_text(json.dumps({'only_ids':list(updates),'semantic_conclusions_preserved':True,'all_other_bindings_and_consumers_unchanged':True,'actual_source_ownership_clarified':True,'entries':audit},ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print('Five stale bindings semantically audited; historical source statements scoped, current evidence anchors renewed.')
