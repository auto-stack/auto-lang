"""Short runtime paths with a verified artifact copy at the requested output."""
from dataclasses import dataclass
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import tempfile
import uuid


@dataclass(frozen=True)
class CaseRuntime:
    path: Path
    base: Path
    nonce: str


def create_runtime():
    base = Path(tempfile.gettempdir()).resolve()
    path = Path(tempfile.mkdtemp(prefix='tmcase-', dir=base)).resolve()
    nonce = uuid.uuid4().hex
    (path / '.tmcase-owner').write_text(nonce, encoding='utf-8')
    return CaseRuntime(path, base, nonce)


def _reparse(path):
    info = path.lstat()
    return path.is_symlink() or bool(getattr(info, 'st_file_attributes', 0) & getattr(stat, 'FILE_ATTRIBUTE_REPARSE_POINT', 0x400))


def check_output_path(path):
    lexical = Path(path).absolute()
    for current in reversed([lexical, *lexical.parents]):
        if os.path.lexists(current) and _reparse(current):
            raise ValueError('output reparse ancestor refused: ' + str(current))
    return lexical.resolve()


def _owned(runtime):
    path = runtime.path
    if (_reparse(path) or path.resolve().parent != runtime.base
            or not path.name.startswith('tmcase-') or _reparse(path / '.tmcase-owner')
            or (path / '.tmcase-owner').read_text(encoding='utf-8') != runtime.nonce):
        raise ValueError('case runtime ownership changed; runtime retained: ' + str(path))
    return path.resolve()


def _files(root):
    stack, files = [root], []
    while stack:
        current = stack.pop()
        for path in current.iterdir():
            # Check before descending. Never follow or create a reparse point
            # in the worktree, including when a guest produced it outside it.
            if _reparse(path):
                raise ValueError('artifact reparse point refused; runtime retained: ' + str(path))
            if path.is_dir():
                stack.append(path)
            elif path.is_file():
                files.append(path)
            else:
                raise ValueError('unsupported artifact node; runtime retained: ' + str(path))
    return sorted(files)


def _sha(path):
    value = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1 << 20), b''):
            value.update(block)
    return value.hexdigest()


def archive_runtime(runtime, output, index):
    source = _owned(runtime)
    files = _files(source)
    output = check_output_path(output)
    output.mkdir(parents=True, exist_ok=True)
    destination = Path(tempfile.mkdtemp(prefix=f'{index}-case-', dir=output)).resolve()
    manifest = []
    for path in files:
        relative = path.relative_to(source)
        target = destination / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        digest = _sha(path)
        shutil.copy2(path, target)
        if _sha(target) != digest:
            raise ValueError('artifact copy changed bytes; runtime retained: ' + str(source))
        manifest.append({'path': relative.as_posix(), 'bytes': target.stat().st_size, 'sha256': digest})
    document = {'schema': 'test-case-artifact-copy/1', 'original_runtime_root': str(source),
                'files': manifest, 'runtime_cleanup_complete': False,
                'note': 'Original receipt bytes and their runtime paths are retained; resolve copied artifacts under this directory.'}
    receipt = destination / 'artifact-copy.json'
    receipt.write_text(json.dumps(document, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    # The full copy is verified before removal. The resolved target is the
    # explicitly created temporary directory, with its ownership nonce intact.
    _owned(runtime)
    _files(source)
    shutil.rmtree(source)
    document['runtime_cleanup_complete'] = True
    receipt.write_text(json.dumps(document, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    return {'root': str(destination), 'report': str(destination / 'module-case.json'),
            'manifest': str(receipt), 'original_runtime_root': str(source), 'runtime_cleanup_complete': True}
