#!/usr/bin/env python3
"""PLAN-941 read-only evidence admission primitives.

This first implementation slice does not issue an acceptance verdict. Domain
readers and the complete frozen obligation registry must precede that reducer.
No tools, tests, compilers, guests, or meters are started here.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import stat
import math
import sys

DOCUMENT_BYTES = 4 << 20
LIVE_BYTES = 48 << 20
U64_MAX = (1 << 64) - 1
HASH = re.compile(r"[0-9a-f]{64}\Z")


class AdmissionError(ValueError):
    """Malformed, inapplicable, or stale evidence; never a language failure."""


class ResourceError(AdmissionError):
    """Admission exceeded the composer's own budget; never truncate to PASS."""


class LiveBudget:
    """One aggregate reservation shared across all authorized roots."""
    def __init__(self):
        self.reserved = 0
        self.peak_reserved = 0

    def reserve(self, amount):
        if type(amount) is not int or amount < 0:
            raise AdmissionError('invalid live-data reservation')
        if amount > LIVE_BYTES - self.reserved:
            raise ResourceError('composer live-data reservation exceeds 48 MiB')
        self.reserved += amount
        self.peak_reserved = max(self.peak_reserved, self.reserved)

    def release_transient(self, amount):
        """Caller must have dropped all input documents covered by this amount.

        Returned compact summaries keep their own reservation. A lifetime
        release never changes the aggregate peak or grants another live cap.
        """
        if type(amount) is not int or not 0 <= amount <= self.reserved:
            raise AdmissionError('invalid transient release')
        self.reserved -= amount


def exact_keys(value, keys, where):
    if type(value) is not dict or value.keys() != set(keys):
        raise AdmissionError(f'{where}: expected exact keys {sorted(keys)}')
    return value


def counter(value, where):
    if type(value) is not int or not 0 <= value <= U64_MAX:
        raise AdmissionError(f'{where}: expected an unsigned 64-bit integer')
    return value


def hash_value(value, where):
    if type(value) is not str or HASH.fullmatch(value) is None:
        raise AdmissionError(f'{where}: expected a lowercase SHA256')
    return value


def _object(pairs):
    value = {}
    for key, item in pairs:
        if key in value:
            raise AdmissionError(f'duplicate JSON key: {key}')
        value[key] = item
    return value


def _constant(value):
    raise AdmissionError(f'non-finite JSON number: {value}')


def _integer(value):
    # Bound before constructing an arbitrarily large Python integer. Fields
    # requiring counters additionally call counter(), which refuses booleans.
    if len(value.lstrip('-')) > 20:
        raise AdmissionError('JSON integer exceeds the evidence integer domain')
    parsed = int(value)
    if not -(1 << 63) <= parsed <= U64_MAX:
        raise AdmissionError('JSON integer exceeds the evidence integer domain')
    return parsed


def _float(value):
    parsed = float(value)
    if not math.isfinite(parsed):
        raise AdmissionError('non-finite JSON number')
    return parsed


def _no_reparse(path):
    # Inspect lexical ancestors BEFORE resolve/open: resolve alone would erase
    # the evidence that an authorized-looking path crossed a junction.
    for parent in reversed((path, *path.parents)):
        try:
            info = parent.lstat()
        except OSError as error:
            raise AdmissionError(f'unreadable evidence path: {parent}') from error
        if stat.S_ISLNK(info.st_mode) or getattr(info, 'st_file_attributes', 0) & 0x400:
            raise AdmissionError(f'reparse point refused: {parent}')


class EvidenceScope:
    """Explicit root, no directory discovery, bounded JSON and streaming hash.

    Reservations accumulate for the lifetime of this scope, conservatively
    covering retained raw/decoded documents and their indexes. Callers must not
    reset a reservation while retaining a document. A fresh scope per child
    does not replace the composer's aggregate live-data reservation.
    """

    def __init__(self, root, budget=None):
        self.root = Path(root).absolute()
        _no_reparse(self.root)
        if not self.root.is_dir():
            raise AdmissionError('evidence scope must be an existing directory')
        self.budget = budget if budget is not None else LiveBudget()
        self.reserve(256 << 10)  # streaming hash/read buffers, reused

    @property
    def reserved(self):
        return self.budget.reserved

    def path(self, relative):
        if type(relative) is not str or not relative or '\\' in relative or ':' in relative:
            raise AdmissionError('artifact path must be a normalized relative POSIX path')
        if any(ord(c) < 32 or c in '<>"|?*' or 0xD800 <= ord(c) <= 0xDFFF for c in relative):
            raise AdmissionError('invalid artifact path character')
        parts = relative.split('/')
        if (PurePosixPath(relative).is_absolute() or len(relative.encode('utf-8')) > 4096
                or any(p in ('', '.', '..') or p.rstrip(' .') != p for p in parts)):
            raise AdmissionError('artifact path escapes or aliases its scope')
        # Windows device paths remain special even with a filename extension.
        for part in parts:
            if part.split('.')[0].upper() in {'CON', 'PRN', 'AUX', 'NUL',
                    *(f'COM{i}' for i in range(1, 10)), *(f'LPT{i}' for i in range(1, 10))}:
                raise AdmissionError('device artifact path refused')
        path = self.root.joinpath(*parts)
        _no_reparse(path)
        if not stat.S_ISREG(path.stat().st_mode):
            raise AdmissionError('artifact must be a regular file')
        return path

    def reserve(self, amount):
        self.budget.reserve(amount)

    def read_bytes(self, relative):
        path = self.path(relative)
        size = path.stat().st_size
        if size > DOCUMENT_BYTES:
            raise ResourceError('input document exceeds 4 MiB bytes')
        self.reserve(size + 1)
        # Also cap the actual read, in case a regular file grew after stat.
        with path.open('rb') as stream:
            raw = stream.read(size + 1)
        if len(raw) > size:
            raise ResourceError('input document grew after budget reservation')
        return raw

    def read_json(self, relative):
        raw = self.read_bytes(relative)
        return self.decode_json(raw)

    def decode_json(self, raw):
        """Decode an already admitted raw frame under the same shared budget."""
        if type(raw) is not bytes:
            raise AdmissionError('JSON frame must be bytes')
        if len(raw) > DOCUMENT_BYTES:
            raise ResourceError('input document exceeds 4 MiB bytes')
        # Pre-decode conservative reservation: UTF-8/Unicode, raw and decoder
        # buffers plus Python nodes/index capacity. Counting punctuation inside
        # strings over-reserves intentionally; it never understates live data.
        nodes = sum(raw.count(bytes([c])) for c in b'{}[],:')
        self.reserve(7 * len(raw) + 256 * nodes + 4096)
        try:
            text = raw.decode('utf-8', errors='strict')
            value = json.loads(text, object_pairs_hook=_object,
                               parse_constant=_constant, parse_int=_integer,
                               parse_float=_float)
        except (UnicodeError, json.JSONDecodeError) as error:
            raise AdmissionError(f'invalid UTF-8 JSON: {error}') from error
        except (RecursionError, MemoryError) as error:
            raise ResourceError('JSON decoder resource limit') from error
        return value, hashlib.sha256(raw).hexdigest()

    def sha256(self, relative, expected):
        hash_value(expected, relative)
        path = self.path(relative)
        value = hashlib.sha256()
        with path.open('rb') as stream:
            for block in iter(lambda: stream.read(64 << 10), b''):
                value.update(block)
            if value.hexdigest() != expected:
                raise AdmissionError(f'STALE artifact content: {relative}')
        return path


# PLAN-941 T-04 (§5.2 total predicate): the acceptance reducer. Pure over
# named domain gates — the four input classes (domain gates, accounting,
# evidence completeness, system delivery) NEVER derive the final claim on
# their own: can_claim_expanded_m1 requires every domain gate AND valid
# accounting AND complete evidence AND zero legacy cert-gaps AND zero
# awaiting-D promises AND a delivered acceptance system, each supplied
# independently. A red or named blocker is a correct result; the reducer
# never converts a missing input into a pass.
ACCEPTANCE_GATES = ('legacy_current_gate', 'legacy_required_execution', 'c_core_and_resource',
                    'q4_oracle_certification', 'syntax_architecture', 'precise_incremental_restore',
                    'budgets_time', 'expanded_registry_valid')


def reduce_acceptance(gates, accounting_valid, evidence_complete, cert_gaps, awaiting_d, system_delivered, missing_inputs):
    """§5.2: legacy-current-gate AND legacy-required-execution-evidence AND
    C-core/resource AND Q4-oracle/certification AND SYN/ARCH-01..08 AND
    precise-incremental/restore AND budgets/time AND
    expanded-registry/evidence-valid."""
    exact_keys(gates, ACCEPTANCE_GATES, 'acceptance domain gates')
    for name in ACCEPTANCE_GATES:
        if type(gates[name]) is not bool:
            raise AdmissionError(f'acceptance gate {name} must be a bool')
    for name, value in (('accounting_valid', accounting_valid), ('evidence_complete', evidence_complete),
                        ('system_delivered', system_delivered)):
        if type(value) is not bool:
            raise AdmissionError(f'{name} must be a bool')
    for name, value in (('cert_gaps', cert_gaps), ('awaiting_d', awaiting_d)):
        if type(value) is not int or value < 0:
            raise AdmissionError(f'{name} must be a non-negative integer')
    if type(missing_inputs) is not list or not all(type(item) is str and item for item in missing_inputs):
        raise AdmissionError('missing_inputs must be a list of non-empty strings')
    blockers = [name for name in ACCEPTANCE_GATES if not gates[name]]
    if accounting_valid is not True:
        blockers.append('accounting_invalid')
    if evidence_complete is not True:
        blockers.append('evidence_incomplete')
    if cert_gaps:
        blockers.append(f'legacy_cert_gaps:{cert_gaps}')
    if awaiting_d:
        blockers.append(f'awaiting_d:{awaiting_d}')
    if not system_delivered:
        blockers.append('acceptance_system_not_delivered')
    for item in missing_inputs:
        blockers.append('missing_input:' + item)
    return {'schema': 'ace.m1.acceptance-reducer/1',
            'domain_gates': dict(gates),
            'accounting_valid': accounting_valid,
            'evidence_complete': evidence_complete,
            'legacy_cert_gaps': cert_gaps,
            'awaiting_d': awaiting_d,
            'acceptance_system_delivered': system_delivered,
            'can_claim_expanded_m1': not blockers,
            'blockers': blockers}


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description='Read-only bounded M1 evidence preview; full acceptance remains incomplete')
    parser.add_argument('--first-slice-preview', action='store_true')
    parser.add_argument('--repo')
    parser.add_argument('--evidence')
    parser.add_argument('--bindings')
    args = parser.parse_args()
    if not args.first_slice_preview or not all((args.repo, args.evidence, args.bindings)):
        parser.error('only --first-slice-preview with explicit --repo/--evidence/--bindings is available')
    try:
        from ace_m1_first_slice import compose_first_slice
        report = compose_first_slice(args.repo, args.evidence, args.bindings)
        raw = json.dumps(report, ensure_ascii=False, separators=(',', ':'), allow_nan=False).encode('utf-8')
        if len(raw) > DOCUMENT_BYTES:
            raise ResourceError('preview report exceeds 4 MiB')
        sys.stdout.buffer.write(raw + b'\n')
    except (ValueError, OSError, KeyError, TypeError, RecursionError, MemoryError) as error:
        print('PLAN-941 evidence refused: ' + str(error), file=sys.stderr)
        raise SystemExit(2)
    # Preview is always ineligible, including a fully green bounded first slice.
    raise SystemExit(1)
