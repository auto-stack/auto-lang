"""Scoped legacy ledger projection; canonical Markdown remains authoritative.

The prepared owned view uses current musk SpecsDocument/SpecItem fields. The
existing repository projection uses lowercase statuses/date compatibility keys;
adapt only owned records, preserve unrelated records and unknown fields exactly.
"""
import copy
import ctypes
import hashlib
import json
import os
import re
import sys
import subprocess
import time
import uuid
from datetime import datetime, timezone
from pathlib import Path

main_root = Path('D:/autostack/auto-lang').resolve()
root = Path(sys.argv[2]).resolve() if len(sys.argv) > 2 else main_root
assert root in (main_root, Path('D:/autostack/.wt/lang-745/auto-lang').resolve())
packet = root/'docs/plans/attachments/745/merge'
ledger = root/'.autoos/specs.json'
mode = sys.argv[1]
assert mode in ('active', 'archive')
baseline = json.loads((packet/'baseline.json').read_text(encoding='utf-8-sig'))
landed = json.loads((packet/'landed.json').read_text(encoding='utf-8-sig'))
archive = 'docs/plans/archive/745-ac-source-checked-hir-native.md'
history_file = archive if mode == 'archive' else 'docs/plans/745-ac-source-checked-hir-native.md'
assert (root/history_file).is_file()
guard_path = root/'.autoos/plan-745-ledger.lock'
lock_fd = os.open(guard_path, os.O_CREAT|os.O_EXCL|os.O_RDWR)
k32 = ctypes.WinDLL('kernel32', use_last_error=True)
k32.CreateFileW.argtypes = [ctypes.c_wchar_p,ctypes.c_uint32,ctypes.c_uint32,ctypes.c_void_p,ctypes.c_uint32,ctypes.c_uint32,ctypes.c_void_p]
k32.CreateFileW.restype = ctypes.c_void_p
k32.CloseHandle.argtypes = [ctypes.c_void_p]
handle = None
temporary = None
try:
    # Deny write opens while allowing our atomic replacement. No Musk writer is
    # active (process inspection is separately recorded by the merge owner).
    handle = k32.CreateFileW(str(ledger),0x80000000,1|4,None,3,0x80,None)
    assert handle not in (None,ctypes.c_void_p(-1).value), f'ledger lock failed: {ctypes.get_last_error()}'
    before_bytes = ledger.read_bytes()
    before_hash = hashlib.sha256(before_bytes).hexdigest()
    if mode == 'active':
        assert before_hash == baseline['ledger_before_sha256'], 'runtime ledger changed since baseline'
    else:
        prior = json.loads((packet/'ledger-active.json').read_text(encoding='utf-8'))
        assert before_hash == prior['after_sha256'], 'runtime ledger drift before archive reference update'
    data = json.loads(before_bytes)
    original = copy.deepcopy(data)
    sections = {s['id']:s for s in data['sections']}
    assert set(sections) == {'goals','architecture','designs','tests','reviews','reports'}
    all_ids = {i['id'] for s in data['sections'] for i in s['items']}
    now = int(time.time())
    day = datetime.now(timezone.utc).date().isoformat()
    source_specs = {
        'P741-2': ('designs','docs/specs/auto-ac/project.md','current-state','AC 原生AOT：HIR及有限源码profile', '当前 Checked→native/事务/工具链见canonical；PLAN-745新增显式core-i32源码→Checked→Windows PE闭环。完整语言、生产ABI与ACC native自举仍open。'),
        'P743-1': ('designs','docs/specs/auto-acc/project.md','current-state','ACC 主体与能力边界', 'ACC主体/Gen1–3判据与四态现状见canonical；PLAN-745仅补有限Rust source profile数值证据。MD408/409 required与完整int/char open维持，不把原型当Auto主体/自举。'),
        'P743-2': ('designs','docs/specs/auto-hir/stage-contract.md','current-state','公共HIR阶段与Checked/pass边界', 'S0仅auto.source.core-i32.draft接入；生成Atom/来源表→bind→全Bundle来源投影→verify→Checked。HIR三重身份/Schema/运算面/pass状态不扩面；阶段与效果/trap合同见canonical。'),
        'P745-1': ('designs','docs/specs/auto-ac/source-core-i32.md','profile','Auto source core-i32 profile', 'PLAN-745 r1已审有限源码profile的数值/支持/拒绝/CLI/来源/嵌套与运行期语义；当前规则仅引用canonical，不复制Plan过程。'),
        'P745-2': ('tests','docs/specs/auto-ac/source-core-i32.md','验收与范围来源','源码→Checked→native 验收', '源码74例（21PE/53拒绝）、114测试与独立M/N块流/未闭合/深输入、Bundle/事务/执行器控制；可执行入口与边界见canonical验收节，最终证据见PLAN-745第七轮报告。'),
        'P745-3': ('reviews',history_file,'review-r7','PLAN-745 r1 第七轮最终复审 pass', '最终独立复审绑定bb8cb4b4330d505059ea20e7bfe561a93c1051f3；AC01..08 pass。24提交rebase补丁全等价，交付'+landed['delivery_commit']+'。原三项发现清偿，根历史红公开，详细证据见review-r7/REVIEW.md与Plan收据。'),
        'P745-4': ('reports',history_file,'delivery','PLAN-745 源码计算子集交付', '有限Auto源码→Checked HIR→Windows PE已落v0.6-dev；Specs与派生投影沉淀。归档/清理实际状态见Plan的PLAN-745:r1 merge收据；完整语言/ACC自举未完成。'),
    }
    deps = {'P745-2':['P745-1','P741-2'],'P745-3':['P745-1','P741-2','P743-1','P743-2'],'P745-4':['P745-2','P745-3']}
    owned = {}
    native = {'project':'auto-lang','version':int(data.get('version',0))+(len(source_specs) if mode=='active' else 2),'sections':[]}
    for ident,(section,file,part,title,content) in source_specs.items():
        matches = [i for i in sections[section]['items'] if i.get('id') == ident]
        by_target = [i for i in sections[section]['items'] if i.get('file') == file and (i.get('canonical_section',part) == part)]
        if ident in ('P741-2','P743-1','P743-2'):
            assert len(matches) == 1 and matches[0]['file'] == file
        elif mode == 'active':
            assert ident not in all_ids, f'ID collision: {ident}'
            assert not by_target, f'Canonical target already projected: {file}/{part}'
        else:
            assert len(matches) == 1, f'Missing retry ID: {ident}'
        item = matches[0] if matches else {'id':ident,'created':day}
        if not matches:
            sections[section]['items'].append(item)
        created = item.get('created_at')
        if created is None:
            created = int(datetime.fromisoformat(item.get('created',day)).replace(tzinfo=timezone.utc).timestamp())
        changed = mode=='active' or section in ('reviews','reports')
        fields = {'id':ident,'title':title,'content':content,'status':'Published','depends_on':deps.get(ident,item.get('depends_on',[])), 'related':[], 'priority':item.get('priority'), 'assignee':item.get('assignee'), 'test_file':None, 'file':file, 'milestone':None,'module':'auto-ac' if ident.startswith('P745') else item.get('module'), 'tags':['plan-745','derived-reference'],'created_at':created,'modified_at':now if changed else item['modified_at'],'completed_at':now if changed else item['completed_at']}
        compat = copy.deepcopy(fields)
        compat['status'] = 'published'
        compat['last_modified'] = day if changed else item['last_modified']
        compat['canonical_section'] = part
        compat['provenance'] = {'plan':'PLAN-745','revision':1,'reviewed_commit':landed['reviewed_commit'],'delivery_commit':landed['delivery_commit'],'frozen_delta_sha256':baseline['frozen_delta_sha256'],'review_report':'docs/plans/attachments/745/review-r7/REVIEW.md','review_report_sha256':hashlib.sha256((root/'docs/plans/attachments/745/review-r7/REVIEW.md').read_bytes()).hexdigest(),'archive':archive}
        if section not in ('reviews','reports'):
            compat['source_sha256'] = hashlib.sha256((root/file).read_bytes()).hexdigest()
        item.update(compat)
        owned[ident] = (section,item,fields)
    # Rebuild owned reverse edges using the store's declared deps + known-ID
    # content scan. Unrelated legacy records are not migrated or rewritten.
    known = {i['id'] for s in data['sections'] for i in s['items']}
    reverse = {i:[] for i in owned}
    for section in data['sections']:
        for item in section['items']:
            forward = set(item.get('depends_on',[]))
            forward |= set(re.findall(r'\b(?:[A-Za-z]+-)?[GADPSVXTIR]\d+(?:\.\d+)?\b',item.get('content',''))) & known
            for target in forward & reverse.keys():
                reverse[target].append(item['id'])
    for ident,(_,item,fields) in owned.items():
        item['related'] = fields['related'] = sorted(set(reverse[ident]))
    titles = {'goals':'Goals','architecture':'Architecture','designs':'Designs','tests':'Tests','reviews':'Reviews','reports':'Reports'}
    for section in data['sections']:
        fields = [x[2] for x in owned.values() if x[0] == section['id']]
        native['sections'].append({'id':section['id'],'section_type':titles[section['id']],'title':titles[section['id']],'items':fields,'status':'Published' if fields else 'Empty','content':'','depends_on':[],'last_modified':now,'last_verified':now if fields else None})
    # Current Rust schema keys/types/statuses for the complete owned view.
    required = {'id','title','content','status','depends_on','related','priority','assignee','test_file','file','milestone','module','tags','created_at','modified_at','completed_at'}
    for section in native['sections']:
        for item in section['items']:
            assert set(item) == required and item['status'] == 'Published'
            assert all(isinstance(item[k],int) for k in ('created_at','modified_at','completed_at'))
            assert all(isinstance(item[k],list) for k in ('depends_on','related','tags'))
            assert (root/item['file']).is_file()
    for section in original['sections']:
        current = sections[section['id']]
        for old in section['items']:
            if old['id'] not in owned:
                assert next(i for i in current['items'] if i['id'] == old['id']) == old, f'Unrelated item changed: {old["id"]}'
    data['project'] = data.get('project','auto-lang')
    data['version'] = native['version']
    assert data['project'] == 'auto-lang'
    payload = (json.dumps(data,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
    json.loads(payload)
    intended = {'schema_source':'D:/autostack/auto-musk/backend/crates/musk/src/specs.rs','adapter':'existing local lowercase/date compatibility format; no whole-ledger migration','native_owned_view':native,'owned_items':[{'section':s,'item':i} for s,i,_ in owned.values()]}
    (packet/f'projection-{mode}.json').write_text(json.dumps(intended,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    backup = root/f'.autoos/specs.745-r1.{mode}.before.json'
    if backup.exists():
        assert backup.read_bytes() == before_bytes, 'Recovery snapshot differs; reconcile before retry'
    else:
        backup.write_bytes(before_bytes)
    temporary = root/'.autoos'/f'specs.745.{uuid.uuid4().hex}.tmp'
    temporary.write_bytes(payload)
    assert hashlib.sha256(ledger.read_bytes()).hexdigest() == before_hash, 'Concurrent ledger drift'
    # Windows MoveFileEx replacement cannot proceed while this read guard is
    # open on this host. Release after preparing/validating the complete JSON;
    # the exclusive operation lock and verified absence of service writers
    # remain in force. Recheck the exact source immediately before replacement.
    k32.CloseHandle(handle)
    handle = None
    assert hashlib.sha256(ledger.read_bytes()).hexdigest() == before_hash, 'Ledger drift at publish boundary'
    os.replace(temporary,ledger)
    temporary = None
    readback = ledger.read_bytes()
    assert readback == payload
    after = json.loads(readback)
    for ident,(s,item,_) in owned.items():
        observed = next(i for sec in after['sections'] if sec['id']==s for i in sec['items'] if i['id']==ident)
        assert observed == item
    receipt = {'checkpoint':'ledger_refreshed' if root==main_root else 'ledger_prepared','mode':mode,'key':'PLAN-745:r1','target':str(main_root/'.autoos/specs.json'),'preparation_path':str(ledger),'before_sha256':before_hash,'after_sha256':hashlib.sha256(readback).hexdigest(),'before_version':original.get('version',0),'after_version':data['version'],'owned_ids':list(owned),'changed_ids':list(owned) if mode=='active' else ['P745-3','P745-4'],'added_ids':[] if mode=='archive' else ['P745-1','P745-2','P745-3','P745-4'],'unrelated_entries_preserved':True,'sections_preserved':True,'schema_owned_view_valid':True,'atomic_replace_verified':True,'recovery_snapshot':str(backup),'recovery_commit':subprocess.run(['git','rev-parse','HEAD'],cwd=root,capture_output=True,text=True,check=True).stdout.strip(),'history_file':history_file,'canonical_items_resolve':True,'archive_pending':mode=='active','storage':'Git tracked; prepared worktree changes publish via ff-only'}
    (packet/f'ledger-{mode}.json').write_text(json.dumps(receipt,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(f'Ledger {mode}: seven scoped upserts verified; unrelated entries preserved; version {data["version"]}.')
finally:
    if handle not in (None,ctypes.c_void_p(-1).value):
        k32.CloseHandle(handle)
    if temporary and temporary.exists():
        temporary.unlink()
    os.close(lock_fd)
    guard_path.unlink()
