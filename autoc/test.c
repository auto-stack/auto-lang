/**
 * Test Cases for autoc
 * Ported from Rust auto-lang test suite
 */

#include "autoc.h"
#include "test.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Helper function to run code and get result
static const char* run_code(const char* code) {
    static char* result = NULL;
    if (result) {
        free(result);
        result = NULL;
    }

    AutoRunResult r = autoc_run(code);
    if (r.result == AUTOC_OK && r.value) {
        const char* repr = value_repr(r.value);
        result = strdup(repr);
    } else if (r.error_msg) {
        result = strdup(r.error_msg);
    } else {
        result = strdup("");
    }
    autorun_free(&r);
    return result;
}

// ============================================================================
// Basic Arithmetic Tests
// ============================================================================

bool test_uint(void) {
    const char* result = run_code("1u+2u");
    TEST_ASSERT_EQ_STR(result, "3u", "uint addition");

    result = run_code("25u+123u");
    TEST_ASSERT_EQ_STR(result, "148u", "uint addition larger");

    return true;
}

bool test_arithmetic(void) {
    const char* result = run_code("1+2*3");
    TEST_ASSERT_EQ_STR(result, "7", "arithmetic precedence");

    result = run_code("(2+3.5)*5");
    TEST_ASSERT_EQ_STR(result, "27.5", "float arithmetic");

    return true;
}

bool test_unary(void) {
    const char* result = run_code("-2*3");
    TEST_ASSERT_EQ_STR(result, "-6", "unary minus");

    return true;
}

bool test_group(void) {
    const char* result = run_code("(1+2)*3");
    TEST_ASSERT_EQ_STR(result, "9", "grouping");

    return true;
}

bool test_comp(void) {
    const char* result = run_code("1 < 2");
    TEST_ASSERT_EQ_STR(result, "true", "less than");

    result = run_code("1 > 2");
    TEST_ASSERT_EQ_STR(result, "false", "greater than");

    result = run_code("1 == 1");
    TEST_ASSERT_EQ_STR(result, "true", "equal");

    result = run_code("1 != 2");
    TEST_ASSERT_EQ_STR(result, "true", "not equal");

    return true;
}

// ============================================================================
// Variable Tests
// ============================================================================

bool test_var_assign(void) {
    const char* result = run_code("var a = 1; a = 2; a");
    TEST_ASSERT_EQ_STR(result, "2", "variable assignment");

    return true;
}

bool test_var_arithmetic(void) {
    const char* result = run_code("var a = 12312; a * 10");
    TEST_ASSERT_EQ_STR(result, "123120", "variable arithmetic");

    return true;
}

bool test_var(void) {
    const char* result = run_code("var a = 1; a+2");
    TEST_ASSERT_EQ_STR(result, "3", "variable usage");

    return true;
}

bool test_var_mut(void) {
    const char* result = run_code("var x = 1; x = 10; x+1");
    TEST_ASSERT_EQ_STR(result, "11", "mutable variable");

    return true;
}

// ============================================================================
// Control Flow Tests
// ============================================================================

bool test_if(void) {
    const char* result = run_code("if true { 1 } else { 2 }");
    TEST_ASSERT_EQ_STR(result, "1", "if true");

    result = run_code("if false { 1 } else { 2 }");
    TEST_ASSERT_EQ_STR(result, "2", "if false");

    return true;
}

bool test_for_range(void) {
    const char* result = run_code("var sum = 0; for i in 0..10 { sum = sum + i }; sum");
    TEST_ASSERT_EQ_STR(result, "45", "for range sum");

    return true;
}

bool test_for_range_eq(void) {
    const char* result = run_code("var sum = 0; for i in 0..=10 { sum = sum + i }; sum");
    TEST_ASSERT_EQ_STR(result, "55", "for range eq sum");

    return true;
}

// ============================================================================
// Advanced Tests
// ============================================================================

bool test_array(void) {
    const char* result = run_code("[1, 2, 3]");
    TEST_ASSERT_EQ_STR(result, "[1, 2, 3]", "array literal");

    result = run_code("var a = [1, 2, 3]; a[0]");
    TEST_ASSERT_EQ_STR(result, "1", "array index");

    result = run_code("var a = [1, 2, 3]; a[-1]");
    TEST_ASSERT_EQ_STR(result, "3", "array negative index");

    return true;
}

bool test_object(void) {
    const char* result = run_code("{ name: \"auto\", age: 18 }");
    TEST_ASSERT(strstr(result, "auto") != NULL && strstr(result, "18") != NULL, "object literal");

    result = run_code("var a = { name: \"auto\", age: 18 }; a.name");
    TEST_ASSERT_EQ_STR(result, "auto", "object field access name");

    result = run_code("var a = { name: \"auto\", age: 18 }; a.age");
    TEST_ASSERT_EQ_STR(result, "18", "object field access age");

    return true;
}

// ============================================================================
// Main Test Runner
// ============================================================================

int main(int argc, char* argv[]) {
    printf("========================================\n");
    printf("autoc Test Suite\n");
    printf("========================================\n\n");

    // Basic arithmetic tests
    printf("=== Basic Arithmetic Tests ===\n");
    RUN_TEST(test_uint);
    RUN_TEST(test_arithmetic);
    RUN_TEST(test_unary);
    RUN_TEST(test_group);
    RUN_TEST(test_comp);

    // Variable tests
    printf("\n=== Variable Tests ===\n");
    RUN_TEST(test_var_assign);
    RUN_TEST(test_var_arithmetic);
    RUN_TEST(test_var);
    RUN_TEST(test_var_mut);

    // Control flow tests
    printf("\n=== Control Flow Tests ===\n");
    RUN_TEST(test_if);
    RUN_TEST(test_for_range);
    RUN_TEST(test_for_range_eq);

    // Advanced tests
    printf("\n=== Advanced Tests ===\n");
    RUN_TEST(test_array);
    RUN_TEST(test_object);

    // Print summary
    PRINT_TEST_SUMMARY();

    return tests_failed > 0 ? 1 : 0;
}
