"""Admit a fresh output before writing; delegate case-domain policy to its owner."""
import json
import os
from pathlib import Path

from case_artifacts import check_output_path
from process_capture import capture


def admit_output(path):
    path = check_output_path(Path(path))
    if path.exists() and (not path.is_dir() or any(path.iterdir())):
        raise ValueError('output directory must be fresh or empty; preserve previous results: ' + str(path))
    return path


def audit_case_outputs(catalog, selected, output, diagnostics, prepared):
    from governance_io import ROOT, load, write
    from prepare_tools import prepare
    names = set(selected)
    suites = sorted({catalog['recipes'][m['recipe']]['suite'] for m in catalog['modules']
                     if m['id'] in names and catalog['recipes'][m['recipe']].get('runner') == 'case-suite'})
    return audit_suite_outputs(ROOT.resolve(), suites, output, diagnostics, prepared)


def audit_suite_outputs(root, suites, output, diagnostics, prepared):
    """Original TestCase authority for the final output, independent of catalogs."""
    from governance_io import load, write
    from prepare_tools import prepare
    ROOT = Path(root)
    if not suites:
        return None
    diagnostics = Path(diagnostics).resolve()
    diagnostics.mkdir(parents=True, exist_ok=True)
    _, preparation = prepare(ROOT.resolve(), ['case-parent'], diagnostics / 'tools', prepared)
    tool = load(ROOT / 'scripts/test-governance/toolchains.json')['tools']['case-parent']
    parent = ROOT / tool['target'] / tool['profile'] / (tool['binaries'][0] + ('.exe' if os.name == 'nt' else ''))
    existed = output.exists()
    rows = []
    try:
        for index, suite in enumerate(suites):
            source = (ROOT / suite).resolve()
            command = [str(parent), 'audit-output', '--suite', str(source), '--out', str(output)]
            result = capture(command, cwd=ROOT, timeout=60)
            (diagnostics / f'{index}-stdout.json').write_bytes(result.stdout)
            (diagnostics / f'{index}-stderr.log').write_bytes(result.stderr)
            if result.measurement is not None:
                write(diagnostics / f'{index}-meter.json', result.measurement)
            if result.returncode:
                raise ValueError(f'case output rejected before execution: {suite}; {diagnostics / f"{index}-stderr.log"}')
            row = json.loads(result.stdout)
            if (row.get('schema') != 'test-case-output-audit/1' or row.get('scope') != 'output-admission'
                    or row.get('accepted') is not True or row.get('suite') != str(source)
                    or row.get('output') != str(output)):
                raise ValueError('case output admission identity/schema mismatch')
            rows.append(row)
    except Exception:
        # Only remove an empty directory that these admission calls created.
        # No receipt or source data has been written into it at this point.
        if not existed and output.is_dir() and not any(output.iterdir()):
            check_output_path(output).rmdir()
        raise
    return {'schema': 'test-module-output-admission/1', 'scope': 'output-admission',
            'preparation': preparation, 'suites': rows, 'diagnostics': str(diagnostics)}
