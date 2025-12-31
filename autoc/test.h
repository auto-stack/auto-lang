/**
 * Test Framework for autoc
 */

#ifndef TEST_H
#define TEST_H

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

// Test statistics
static int tests_run = 0;
static int tests_passed = 0;
static int tests_failed = 0;

// Test assertion macros
#define TEST_ASSERT(cond, msg) \
    do { \
        if (!(cond)) { \
            fprintf(stderr, "  \033[31mFAILED\033[0m: %s\n", msg); \
            return false; \
        } \
    } while(0)

#define TEST_ASSERT_EQ_STR(actual, expected, name) \
    do { \
        if (strcmp((actual), (expected)) != 0) { \
            fprintf(stderr, "  \033[31mFAILED\033[0m: %s\n", name); \
            fprintf(stderr, "    Expected: %s\n", expected); \
            fprintf(stderr, "    Actual:   %s\n", actual); \
            return false; \
        } \
    } while(0)

#define TEST_ASSERT_EQ_INT(actual, expected, name) \
    do { \
        if ((actual) != (expected)) { \
            fprintf(stderr, "  \033[31mFAILED\033[0m: %s\n", name); \
            fprintf(stderr, "    Expected: %d\n", expected); \
            fprintf(stderr, "    Actual:   %d\n", actual); \
            return false; \
        } \
    } while(0)

// Test case definition
typedef bool (*test_func_t)(void);

typedef struct {
    const char* name;
    test_func_t func;
} TestCase;

// Test runner macros
#define RUN_TEST(test) \
    do { \
        tests_run++; \
        printf("Running %-37s...", #test); \
        if (test()) { \
            tests_passed++; \
            printf("  \033[32mPASSED\033[0m\n"); \
        } else { \
            tests_failed++; \
        } \
    } while(0)

// Print test summary
#define PRINT_TEST_SUMMARY() \
    do { \
        printf("\n========================================\n"); \
        printf("Test Summary:\n"); \
        printf("  Total:   %d\n", tests_run); \
        printf("  Passed:  %d\n", tests_passed); \
        printf("  Failed:  %d\n", tests_failed); \
        printf("========================================\n"); \
    } while(0)

// ============================================================================
// Test Cases
// ============================================================================

// Basic arithmetic tests
bool test_uint(void);
bool test_arithmetic(void);
bool test_unary(void);
bool test_group(void);
bool test_comp(void);

// Variable tests
bool test_var_assign(void);
bool test_var_arithmetic(void);
bool test_var(void);
bool test_var_mut(void);

// Control flow tests
bool test_if(void);
bool test_if_else(void);
bool test_for_range(void);
bool test_for_range_eq(void);

// Advanced tests
bool test_array(void);
bool test_object(void);

#endif // TEST_H
