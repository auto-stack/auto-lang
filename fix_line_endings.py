#!/usr/bin/env python3
"""
Fix line endings in expected files to match generated C code (LF only).
The C transpiler generates code with LF line endings, but files created on Windows
may have CRLF line endings, causing test failures.
"""

import os
import glob

# Fix all expected.c and expected.h files in tests 046-070
test_dirs = []
for i in range(46, 71):
    test_dirs.extend(glob.glob(f"crates/auto-lang/test/a2c/{i}_*"))

for test_dir in test_dirs:
    for ext in ['.c', '.h']:
        expected_files = glob.glob(os.path.join(test_dir, f"*{ext}"))

        for expected_file in expected_files:
            # Read file as text
            with open(expected_file, 'r', newline='') as f:
                content = f.read()

            # Write back with LF line endings only
            with open(expected_file, 'w', newline='\n') as f:
                f.write(content)

            print(f"Fixed: {expected_file}")

print(f"\nFixed {len(test_dirs)} test directories")
