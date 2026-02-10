#!/bin/bash
# Plan 087 Integration Tests - Run All Tests

echo "=========================================="
echo "Plan 087: Generic Type Integration Tests"
echo "=========================================="
echo ""

# Test 1: Multi-instance
echo "Test 1: Multiple Instances"
echo "----------------------------"
../../target/release/auto.exe run multi_instances.at
echo ""

# Test 2: Nested Generic
echo "Test 2: Nested Generic Types"
echo "----------------------------"
../../target/release/auto.exe run nested_generic.at
echo ""

# Test 3: Edge Cases
echo "Test 3: Edge Cases"
echo "----------------------------"
../../target/release/auto.exe run edge_cases.at
echo ""

# Test 4: Type Modification
echo "Test 4: Type Modification"
echo "----------------------------"
../../target/release/auto.exe run type_modification.at
echo ""

# Test 5: Advanced Generic
echo "Test 5: Advanced Generic Types"
echo "----------------------------"
../../target/release/auto.exe run advanced_generic.at
echo ""

# Test 6: Generic Constraints
echo "Test 6: Generic Constraints"
echo "----------------------------"
../../target/release/auto.exe run generic_constraints.at
echo ""

# Test 7: Mixed Syntax
echo "Test 7: Mixed Syntax"
echo "----------------------------"
../../target/release/auto.exe run mixed_syntax.at
echo ""

echo "=========================================="
echo "All Integration Tests Complete!"
echo "=========================================="
