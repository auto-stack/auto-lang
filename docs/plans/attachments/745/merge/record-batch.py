"""Record completed root batches without relabeling unmatched failures as green."""
import hashlib
import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path

root = Path('D:/autostack/auto-lang')
packet = root/'docs/plans/attachments/745/merge'
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
write = lambda p,v: p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
baseline = read(packet/'baseline.json')
landed = read(packet/'landed.json')
attribution = read(packet/'batch-attribution.json')
for control in read(packet/'isolated-controls.json'):
    raw = (packet/(control['test']+'.stderr.txt')).read_text(encoding='utf-8-sig')
    assert control['exit']==0 and re.search(r'Summary .*1 test run: 1 passed',raw)
book = Path('D:/autostack/book')
def bookgit(*args):
    return subprocess.run(['git',*args],cwd=book,capture_output=True,text=True,check=True).stdout
tb = (packet/'batch-tb.stderr.txt').read_text(encoding='utf-8-sig')
book_failures = [n for n in attribution['legs']['tb']['failures'] if 'tests::book_listing_tests::book_ch' in n]
assert len(book_failures)==55
assert 'generate_book_expected' in tb
assert re.search(r'PASS .*generate_book_expected',tb)
audit = {'book_root':str(book),'head':bookgit('rev-parse','HEAD').strip(),
         'status_after_batch':bookgit('status','--short').splitlines(),
         'generator_test':'tests::book_listing_tests::generate_book_expected',
         'generator_passed':True,'source':'crates/auto-lang/src/tests/book_listing_tests.rs:69',
         'external_writes':'unignored generator writes main.expected.rs; mismatch tests write main.wrong.rs',
         'book_failure_count':55,'book_failures':book_failures,
         'pre_batch_external_snapshot_available':False,
         'limits':'No pre-batch book snapshot: cannot attribute every external modification to this run or restore prior WIP. Preserve current external state; no external reset/commit or golden regeneration used as acceptance.',
         'log_sha256':hashlib.sha256((packet/'batch-tb.stderr.txt').read_bytes()).hexdigest()}
write(packet/'external-book-audit.json',audit)
observations = [
    {'id':'P745-MERGE-ENV-01','test':'tests::a2r_tests::temp_plan009_t8_transpile_at_app','result':'fails: fixed D:/autostack/auto-term/at/autoterm.at missing; unchanged root test, no external path creation/repair'},
    {'id':'P745-MERGE-VERIFY-01','tests':book_failures,'result':'55 golden mismatches; unignored golden generator also passed in same parallel batch. Oracle is mutable/external. Not treated as old seven parse failures or green; external state preserved.'},
    {'tests':['stage3_memory_baseline_n1_3_5','plan707_client_sse_poll_close_error_surface'],'result':'failed under batch load; each isolated single-job run passed 1/1; root sources/config unchanged by 745'},
]
write(root/'docs/plans/.last-batch-regression.json',{
    'last_covered_plan_id':745,'timestamp':datetime.now(timezone.utc).isoformat(),
    'covered_commit':landed['delivery_commit'],'previous_covered_commit':baseline['previous_batch']['covered_commit'],
    'legs':['tf','tt','tb'],'gates':read(packet/'batch-gates.json'),
    'taa_skipped_reason':'3668-path window touches zero aavm trigger paths; see merge/baseline.json',
    'new_reds':[],'new_reds_scope':'No failure attributable to PLAN-745 production changes: independent experimental workspace; production sources/config/manifests unchanged from pre-landing main. This is not an all-green or same-input baseline-run claim.',
    'new_observations':observations,
    'known_reds_seen':{k:list(v['historical_matches']) for k,v in attribution['legs'].items()},
    'supplemental_known_red':'a2vue desktop golden is explicitly recorded in prior Plan715 batch receipt; no raw-name match in review-log index',
    'counts':{'tf':{'run':5994,'passed':5963,'failed':31,'skipped':512},'tt':{'run':5342,'passed':5308,'failed':34,'skipped':1782},'tb':{'run':5027,'passed':4951,'failed':76,'skipped':1534}},
    'evidence':'docs/plans/attachments/745/merge/MERGE.md',
    'regress_skill':baseline['regress_skill'],
    'verdict':'Completed with known root failures and explicit environment/oracle observations; no all-green claim. PLAN-745 scoped acceptance remains verified.'})
debt = root/'docs/plans/KNOWN-DEBT-AND-RISKS.md'
text = debt.read_text(encoding='utf-8')
assert '## PLAN-745 合入批量回归观察' not in text
debt.write_text(text+'''\n## PLAN-745 合入批量回归观察（2026-10-08）

745 的独立源码 profile 合同已通过；以下属于根测试/外部输入的观察，未静默缩减验收或把失败报成全绿。完整原始日志与逐名归因见[合入报告](attachments/745/merge/MERGE.md)。根生产源、配置及依赖清单在落地前后无745变更。

| ID | 观察 | 后续处理边界 |
|---|---|---|
| P745-MERGE-ENV-01 | temp_plan009_t8_transpile_at_app 硬编码外仓 auto-term 输入路径不存在，tt失败 | 后续独立计划修测试环境合同；本次不创建或写外仓路径 |
| P745-MERGE-VERIFY-01 | tb 的55个book金样差异，且未ignore的generate_book_expected在同批成功写外仓金样；并发验证oracle会变化 | 后续独立计划隔离生成与只读验证；无外仓运行前快照，保留现存修改，不重生成/回滚来制造绿色，不等同旧七个parse失败 |

stage3_memory_baseline_n1_3_5 与 plan707_client_sse_poll_close_error_surface 批量失败后，单实例、单job各1/1通过；按负载时序观察记录。tf/tt/tb分别31/34/76失败均保留原始证据，新增未匹配名没有自动归为历史红。
''',encoding='utf-8')
print('Batch receipt and external-state audit recorded; no external mutations.')
