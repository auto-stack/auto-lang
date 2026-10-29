"""Original child observations. Wrapper success never creates child PASS rows."""
import hashlib
import json


def collect(directory,module):
    groups={}
    for path in sorted(directory.glob("*.json")):
        value=json.loads(path.read_bytes())
        if value.get("schema")!="test-execution-trace/1" or value.get("module")!=module:
            raise ValueError("child trace schema/module mismatch")
        group=groups.setdefault(value["invocation"],{})
        phase=value["phase"]
        if phase not in ("start","finish","incomplete") or phase in group:
            raise ValueError("child trace duplicate/unknown phase")
        group[phase]=value
    invocations=[]
    for identity,group in sorted(groups.items()):
        if "start" not in group:raise ValueError("child trace has no invocation start")
        start=group["start"]
        finished=group.get("finish")
        invocations.append({"invocation":identity,"metadata":start["data"],
            "rust_test":start["rust_test"],"complete":finished is not None,
            "observation":finished["data"] if finished else {"status":"ERROR","detail":"invocation did not complete"}})
    return {"schema":"test-execution-traces/1","scope":"original-child-observations",
            "invocations":invocations,"incomplete":sum(not row["complete"] for row in invocations),
            "note":"Statuses are original observations, including deliberate negative controls; no inferred child pass."}


def validate_bindings(rows,traces):
    expected={r['test']:r['legacy_input'] for r in rows if r.get('legacy_input')}
    observed={}
    errors=[]
    for invocation in traces['invocations']:
        metadata=invocation['metadata']
        if metadata.get('family')!='legacy-vm-cache':continue
        name=invocation['rust_test']
        observed.setdefault(name,[]).append(invocation)
    for name,binding in expected.items():
        events=observed.get(name,[])
        if len(events)!=1:
            errors.append(f'{name}: expected exactly one original cache invocation, observed {len(events)}')
            continue
        event=events[0];metadata=event['metadata']
        if not event['complete'] or event['observation'].get('status')!='PASS':
            errors.append(f'{name}: original comparison did not complete successfully')
        if any(metadata.get(key)!=binding[key] for key in ('id','source_sha256','oracle_sha256')):
            errors.append(f'{name}: original input/oracle binding differs')
    if expected and observed.keys()-expected.keys():errors.append('unexpected cache invocation outside the module')
    return {'valid':not errors,'errors':errors,'required_bound_invocations':len(expected)}
