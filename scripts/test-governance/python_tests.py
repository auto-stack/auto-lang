#!/usr/bin/env python3
"""Discover/run real unittest cases through the standard loader, with exact IDs."""
import argparse
import contextlib
import json
from pathlib import Path
import sys
import unittest


def flatten(suite):
    for test in suite:
        if isinstance(test, unittest.TestSuite):
            yield from flatten(test)
        else:
            yield test


def load_tests(path):
    path = Path(path).resolve(strict=True)
    loader = unittest.TestLoader()
    suite = loader.discover(str(path.parent), pattern=path.name, top_level_dir=str(path.parent))
    if loader.errors:
        raise ValueError("unittest discovery failed: " + "; ".join(loader.errors))
    tests = list(flatten(suite))
    if not tests or len(tests) != len({t.id() for t in tests}):
        raise ValueError("empty or duplicate unittest discovery")
    return tests


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--file", required=True, type=Path)
    parser.add_argument("--list", action="store_true")
    parser.add_argument("--case", action="append", default=[])
    args = parser.parse_args()
    tests = load_tests(args.file)
    if args.list:
        if args.case:
            raise ValueError("list enumerates the whole file; case filtering is execution-only")
        print(json.dumps({"schema": "python-unittest-discovery/1", "tests": [
            {"id": t.id(), "ignored": bool(getattr(t, "__unittest_skip__", False)
                or getattr(type(t), "__unittest_skip__", False)
                or getattr(getattr(t, t._testMethodName), "__unittest_skip__", False))}
            for t in tests]}))
        return 0
    if not args.case or len(args.case) != len(set(args.case)):
        raise ValueError("execution requires nonempty, distinct exact --case IDs")
    known = {t.id(): t for t in tests}
    if set(args.case) - known.keys():
        raise ValueError("unknown unittest identity: " + str(sorted(set(args.case) - known.keys())))
    suite = unittest.TestSuite(known[name] for name in args.case)
    with contextlib.redirect_stdout(sys.stderr):
        result = unittest.TextTestRunner(stream=sys.stderr, verbosity=2).run(suite)
    report = {"schema": "python-unittest-result/1", "selected": args.case,
              "executed": result.testsRun, "skipped": [(t.id(), reason) for t, reason in result.skipped],
              "failures": [t.id() for t, _ in result.failures], "errors": [t.id() for t, _ in result.errors]}
    print(json.dumps(report))
    # A dynamic skip is evidence of an unmet condition, never a module pass.
    return 0 if result.wasSuccessful() and not result.skipped and result.testsRun == len(args.case) else 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (ValueError, OSError) as error:
        print(f"python tests ERROR: {error}", file=sys.stderr)
        sys.exit(2)
