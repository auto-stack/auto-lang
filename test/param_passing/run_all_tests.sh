#!/bin/bash
# Plan 088 Phase 7: Run all parameter passing mode tests

echo "======================================"
echo "Plan 088 Phase 7: Integration Tests"
echo "======================================"
echo ""

TEST_DIR="test/param_passing"
AUTO_EXE="./target/release/auto.exe"

# 检查 auto.exe 是否存在
if [ ! -f "$AUTO_EXE" ]; then
    echo "Error: $AUTO_EXE not found"
    echo "Please build the project first: cargo build --release"
    exit 1
fi

# 计数器
TOTAL=0
PASSED=0
FAILED=0

# 运行测试函数
run_test() {
    local test_file=$1
    local test_name=$(basename "$test_file" .at)

    TOTAL=$((TOTAL + 1))
    echo "--------------------------------------"
    echo "Test $TOTAL: $test_name"
    echo "File: $test_file"
    echo "--------------------------------------"

    if $AUTO_EXE run "$test_file"; then
        echo "✅ PASSED"
        PASSED=$((PASSED + 1))
    else
        echo "❌ FAILED"
        FAILED=$((FAILED + 1))
    fi
    echo ""
}

# 遍历测试目录
for test_file in $TEST_DIR/*.at; do
    if [ -f "$test_file" ]; then
        run_test "$test_file"
    fi
done

# 输出总结
echo "======================================"
echo "Test Summary"
echo "======================================"
echo "Total:  $TOTAL"
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "🎉 All tests passed!"
    exit 0
else
    echo "⚠️  Some tests failed"
    exit 1
fi
