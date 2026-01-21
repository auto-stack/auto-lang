#!/bin/bash
# Test ls combined flags

echo "Testing ls command with combined flags..."

cd auto-shell

# Build first
cargo build --release

# Create test directory
mkdir -p /tmp/ls_test/subdir
echo "visible file" > /tmp/ls_test/visible.txt
echo "hidden file" > /tmp/ls_test/.hidden.txt
echo "nested file" > /tmp/ls_test/subdir/nested.txt

echo ""
echo "Test 1: ls -al (combined all + long)"
echo "-----------------------------------"
cargo run -- ls -al /tmp/ls_test

echo ""
echo "Test 2: ls -lh (long + human-readable)"
echo "-----------------------------------"
cargo run -- ls -lh /tmp/ls_test

echo ""
echo "Test 3: ls -ltr (long + time + reverse)"
echo "-----------------------------------"
cargo run -- ls -ltr /tmp/ls_test

echo ""
echo "Test 4: ls -aR (all + recursive)"
echo "-----------------------------------"
cargo run -- ls -aR /tmp/ls_test

echo ""
echo "Test 5: ls -alhR (all + long + human + recursive)"
echo "-----------------------------------"
cargo run -- ls -alhR /tmp/ls_test
