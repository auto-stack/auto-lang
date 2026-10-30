"""Distinguish actual compiler/test jobs from an unchanged idle loopback service."""
import math
from pathlib import PureWindowsPath
import re

ACTIVE = {'rustc.exe', 'cargo-nextest.exe', 'av_probe_worker.exe', 'ac-case-worker.exe',
          'mn-worker.exe', 'mn-collections-worker.exe', 'test-case-pilot.exe', 'rust-lld.exe', 'link.exe', 'cl.exe'}


def classify(snapshot, previous=None):
    rows = snapshot['processes']
    current = {r['pid']: r for r in rows}
    old = {r['pid']: r for r in (previous or {}).get('processes', [])}
    if len(current) != len(rows):
        raise ValueError('duplicate process identity in workload observation')
    busy, services = [], []
    for row in rows:
        name = row['name'].lower()
        reason = 'compiler or test workload'
        details = []
        if name == 'cargo.exe':
            command = row.get('command') or ''
            head = re.match(r'^(?:"[^"]+"|\S+)\s+(\S+)', command)
            manifests = re.findall(r'--manifest-path\s+(?:"([^"]+)"|(\S+))', command)
            descendants, pending = {}, [row['pid']]
            while pending:
                parent = pending.pop()
                for child in rows:
                    if child['parent_pid'] == parent:
                        if child['pid'] == row['pid']:
                            raise ValueError('cycle in external process observation')
                        if child['pid'] not in descendants:
                            descendants[child['pid']] = child
                            pending.append(child['pid'])
            path = PureWindowsPath(next((a or b for a, b in manifests), '').removeprefix('\\\\?\\'))
            expected = path.parent.name.lower() + '.exe'
            backends = [p for p in descendants.values() if p['name'].lower() == expected]
            members = [row, *descendants.values()]
            candidate = (head is not None and head[1] == 'run' and len(manifests) == 1
                and path.is_absolute() and path.name.lower() == 'cargo.toml'
                and path.parent.name.endswith('-back') and len(backends) == 1
                and all(p['name'].lower() in ('cargo.exe', expected) for p in members))
            listeners = ([p for p in snapshot['listeners'] if p['pid'] == backends[0]['pid']]
                         if candidate else [])
            stable = candidate and listeners and all(p['address'] in ('127.0.0.1', '::1')
                and isinstance(p['port'], int) and 0 < p['port'] < 65536 for p in listeners)
            for member in members:
                prior = old.get(member['pid'])
                value = member.get('cpu')
                stable = stable and (prior is not None and member.get('created') is not None
                    and member['created'] == prior.get('created')
                    and member['name'] == prior.get('name')
                    and member.get('command') == prior.get('command')
                    and isinstance(value, (int, float)) and math.isfinite(value) and value >= 0
                    and value == prior.get('cpu'))
            if candidate:
                details = [{'pid': p['pid'], 'name': p['name'], 'created': p.get('created'),
                            'cpu': p.get('cpu'), 'previous_cpu': old.get(p['pid'], {}).get('cpu'),
                            'previous_created': old.get(p['pid'], {}).get('created')} for p in members]
            if stable:
                services.append({'cargo_pid': row['pid'], 'backend_pid': backends[0]['pid'],
                    'backend_name': backends[0]['name'], 'created': backends[0]['created'],
                    'cpu': backends[0]['cpu'], 'unchanged_cpu': True, 'loopback_listeners': listeners,
                    'members': details})
                continue
            reason = 'Cargo workload or service not proven unchanged and idle'
        elif name not in ACTIVE:
            continue
        observation = {'Id': row['pid'], 'ProcessName': row['name'], 'reason': reason}
        if details:
            observation['service_candidate_members'] = details
        busy.append(observation)
    return busy, services


QUERY = r'''
$ErrorActionPreference='Stop'
$all=@(Get-CimInstance Win32_Process)
$names=@('cargo.exe','rustc.exe','cargo-nextest.exe','av_probe_worker.exe','ac-case-worker.exe','mn-worker.exe','mn-collections-worker.exe','test-case-pilot.exe','rust-lld.exe','link.exe','cl.exe')
$selected=@{}; $pending=[Collections.Generic.Queue[int]]::new()
foreach($p in $all){if($names -contains $p.Name){$selected[[int]$p.ProcessId]=$p; if($p.Name -eq 'cargo.exe'){$pending.Enqueue([int]$p.ProcessId)}}}
while($pending.Count){$parent=$pending.Dequeue(); foreach($p in $all){if($p.ParentProcessId -eq $parent -and -not $selected.ContainsKey([int]$p.ProcessId)){$selected[[int]$p.ProcessId]=$p; $pending.Enqueue([int]$p.ProcessId)}}}
$records=@(); $listeners=@()
foreach($p in $selected.Values){
  $live=Get-Process -Id $p.ProcessId -ErrorAction SilentlyContinue
  $records+=[pscustomobject]@{pid=[int]$p.ProcessId;parent_pid=[int]$p.ParentProcessId;name=$p.Name;created=$p.CreationDate.ToUniversalTime().ToString('o');cpu=if($live){$live.CPU}else{$null};command=if($p.Name -eq 'cargo.exe'){$p.CommandLine}else{$null}}
  if($p.Name -like '*-back.exe'){$listeners+=@(Get-NetTCPConnection -OwningProcess $p.ProcessId -State Listen -ErrorAction SilentlyContinue | ForEach-Object {[pscustomobject]@{pid=[int]$p.ProcessId;address=$_.LocalAddress;port=[int]$_.LocalPort}})}
}
@{processes=@($records);listeners=@($listeners)} | ConvertTo-Json -Depth 5 -Compress
'''
