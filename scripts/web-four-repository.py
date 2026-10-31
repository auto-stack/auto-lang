#!/usr/bin/env python3
"""Run the explicit PLAN-890 fixed language groups and native Web/UI/Man owners."""
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent / 'test-governance'))
from four_repository import main

if __name__ == '__main__':
    sys.exit(main())
