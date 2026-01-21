#!/bin/bash
# Test script for ls command flags

cd auto-shell

echo "=== Testing ls command with flags ==="
echo

# Create test directory structure
mkdir -p /tmp/ls_test/subdir
echo "test content" > /tmp/ls_test/visible.txt
echo "hidden content" > /tmp/ls_test/.hidden.txt
echo " subdir content" > /tmp/ls_test/subdir/nested.txt

echo "Test directory created at /tmp/ls_test"
echo "Files:"
echo "  - visible.txt (visible file)"
echo "  - .hidden.txt (hidden file)"
echo "  - subdir/ (directory)"
echo "    - nested.txt"
echo

# Note: For actual testing, user needs to run:
# cargo run
# Then try commands like:
# ls /tmp/ls_test
# ls -a /tmp/ls_test
# ls -l /tmp/ls_test
# ls -R /tmp/ls_test
