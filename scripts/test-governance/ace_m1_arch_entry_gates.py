"""PLAN-1000 T-06: architecture-row entry-gate admission.

Each obligation row in the frozen architecture contract names the evidence
entries that own it (case-suite groups, native modules, or delivered provider
plans). This reader admits, per entry, a green single-group fixed-test gate
document by content hash — group identity and outcome verified — and marks a
row PASS only when EVERY declared entry has a green admitted gate (the
contract's per-row AND). Provider-pending entries (PLAN-938/939) consume the
delivered plans' resident suite evidence (ace-abi), named by owner; a
delivered owner's rows are judged on that evidence, never on the stale
provider-pending label. No execution is inferred from declarations."""
from ace_m1_arch_acceptance import exact_keys
from ace_m1_case_evidence import require
from ace_m1_legacy import typed_equal

CONTRACT = 'crates/auto-lang/test/cases/ace-m1-expanded/architecture-obligations.json'

# delivered provider plans: entry -> (owner plan, resident evidence group)
DELIVERED_PROVIDERS = {
    'foreign': ('PLAN-938', 'ace-abi'),
    'foreign-resource': ('PLAN-939', 'ace-abi'),
}


def admit_arch_entry_gates(inputs, evidence, binding):
    """binding: {'entries': {entry-name: {'path','sha256}}} — every gate doc
    must be a green single-group run whose selected group matches the entry's
    declared group (or the delivered provider's resident group)."""
    exact_keys(binding, ('entries',), 'architecture entry-gate binding')
    require(type(binding['entries']) is dict and binding['entries'], 'entry gate map required')
    document, _ = inputs.read_json(CONTRACT)
    require(document.get('schema') == 'ace.m1.arch-obligation-contract/1', 'architecture contract schema')
    entries = document['entries']
    obligations = document['obligations']
    gates = {}
    for name, ref in sorted(binding['entries'].items()):
        exact_keys(ref, ('path', 'sha256'), f'entry gate reference {name}')
        require(name in entries, f'foreign entry family {name}')
        raw = evidence.read_bytes(ref['path'])
        nodes = sum(raw.count(bytes([c])) for c in b'{}[],:')
        transient = len(raw) + 1 + 7 * len(raw) + 256 * nodes + 4096
        doc, digest = evidence.decode_json(raw)
        if digest != ref['sha256']:
            raise ValueError(f'STALE artifact content: {ref["path"]}')
        groups = doc.get('selected_groups') or doc.get('groups')
        require(type(groups) is list and len(groups) == 1, f'single-group gate required {name}')
        declared = entries[name]
        if declared['kind'] in ('case-suite', 'native-module'):
            expected = declared['group']
        elif declared['kind'] == 'provider-pending':
            owner, resident = DELIVERED_PROVIDERS[name]
            require(declared.get('owner') == owner, f'delivered provider owner {name}')
            expected = resident
        else:
            raise ValueError(f'unknown entry kind {name}')
        typed_equal(groups[0], expected, f'entry gate group identity {name}')
        green = doc.get('gate_passed') is True or doc.get('outcome') == 'PASS'
        if not green:
            # nextest per-invocation doc: every executed event ok and only
            # ignored:true conditional exclusions (the repo's tiering policy)
            events = doc.get('results') or {}
            excluded = doc.get('excluded') or []
            green = (bool(events)
                     and all(v.get('event') == 'ok' for v in events.values())
                     and all(e.get('ignored') is True for e in excluded))
        require(green, f'entry gate not green {name}')
        gates[name] = {'path': ref['path'], 'group': groups[0]}
        # only the tiny {group} verdict survives; release the decoded gate
        evidence.budget.release_transient(transient)
    observations = {}
    for obligation in obligations:
        needed = obligation['entries']
        missing = [n for n in needed if n not in gates]
        if missing:
            continue  # row stays UNOBSERVED; never partially claimed
        observations[obligation['id']] = 'PASS'
    return {'schema': 'ace.m1.arch-entry-gates/1', 'observations': observations,
            'admitted_entries': sorted(gates),
            'source_evidence': {name: g['group'] for name, g in gates.items()}}
