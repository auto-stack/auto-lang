/**
 * Parser Test Runner
 * Reads test cases from parser_tests.md and runs them
 */

#include "autoc.h"
#include "test.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

#define MAX_LINE_LENGTH 4096
#define MAX_TEST_NAME 256
#define MAX_CODE_LENGTH 8192
#define MAX_EXPECTED_LENGTH 16384

// Test statistics
static int parser_tests_run = 0;
static int parser_tests_passed = 0;
static int parser_tests_failed = 0;

typedef struct {
    char name[MAX_TEST_NAME];
    char input[MAX_CODE_LENGTH];
    char expected[MAX_EXPECTED_LENGTH];
} ParserTestCase;

// Read the entire content of a file
static char* read_file(const char* filename, size_t* out_size) {
    FILE* file = fopen(filename, "r");
    if (!file) {
        return NULL;
    }

    fseek(file, 0, SEEK_END);
    size_t size = ftell(file);
    fseek(file, 0, SEEK_SET);

    char* content = (char*)malloc(size + 1);
    if (!content) {
        fprintf(stderr, "Error: Out of memory\n");
        fclose(file);
        return NULL;
    }

    size_t read = fread(content, 1, size, file);
    content[read] = '\0';
    fclose(file);

    if (out_size) *out_size = read;
    return content;
}

// Parse test cases from markdown content
static ParserTestCase* parse_test_cases(const char* content, size_t* out_count) {
    ParserTestCase* cases = NULL;
    size_t capacity = 0;
    size_t count = 0;

    if (!content || !*content) {
        *out_count = 0;
        return NULL;
    }

    const char* p = content;
    while (*p) {
        // Skip leading whitespace
        while (*p && (*p == '\n' || *p == '\r')) p++;
        if (!*p) break;

        // Look for "##" to start a test case
        if (p[0] == '#' && p[1] == '#') {
            // Ensure capacity
            if (count >= capacity) {
                capacity = capacity == 0 ? 16 : capacity * 2;
                ParserTestCase* new_cases = (ParserTestCase*)realloc(cases, capacity * sizeof(ParserTestCase));
                if (!new_cases) {
                    fprintf(stderr, "Error: Out of memory\n");
                    free(cases);
                    return NULL;
                }
                cases = new_cases;
            }

            ParserTestCase* tc = &cases[count];
            memset(tc, 0, sizeof(ParserTestCase));

            // Skip "## " and read test name
            p += 2;
            while (*p == ' ') p++;

            size_t name_len = 0;
            while (*p && *p != '\n' && *p != '\r' && name_len < MAX_TEST_NAME - 1) {
                tc->name[name_len++] = *p++;
            }
            tc->name[name_len] = '\0';

            // Skip to input section (after the empty line)
            while (*p && (*p == '\n' || *p == '\r')) p++;
            if (!*p) break;

            // Read input code until we hit "---"
            size_t input_len = 0;
            while (*p && !(p[0] == '-' && p[1] == '-' && p[2] == '-')) {
                if (input_len < MAX_CODE_LENGTH - 1) {
                    tc->input[input_len++] = *p;
                }
                p++;
            }
            tc->input[input_len] = '\0';

            // Skip "---" and surrounding empty lines
            if (p[0] == '-' && p[1] == '-' && p[2] == '-') {
                p += 3;
            }
            while (*p && (*p == '\n' || *p == '\r' || *p == ' ')) p++;
            if (!*p) break;

            // Read expected output until we hit next "##" or end of file
            size_t expected_len = 0;
            while (*p && !(p[0] == '\n' && p[1] == '#' && p[2] == '#')) {
                if (expected_len < MAX_EXPECTED_LENGTH - 1) {
                    tc->expected[expected_len++] = *p;
                }
                p++;
            }
            tc->expected[expected_len] = '\0';

            // Trim trailing whitespace from expected
            while (expected_len > 0 &&
                   (tc->expected[expected_len - 1] == '\n' ||
                    tc->expected[expected_len - 1] == '\r' ||
                    tc->expected[expected_len - 1] == ' ')) {
                tc->expected[--expected_len] = '\0';
            }

            count++;
        } else {
            p++;
        }
    }

    *out_count = count;
    return cases;
}

// Parse code and get AST representation
static char* get_ast_repr(const char* code) {
    if (!code) return strdup("Error: NULL code");

    Lexer* lexer = lexer_new(code);
    if (!lexer) return strdup("Error: Failed to create lexer");

    Parser* parser = parser_new(lexer);
    if (!parser) {
        lexer_free(lexer);
        return strdup("Error: Failed to create parser");
    }

    Code* ast = parser_parse(parser);
    if (!ast) {
        parser_free(parser);
        lexer_free(lexer);
        return strdup("Error: Failed to parse");
    }

    char* result = NULL;
    if (!ast->stmts) {
        result = strdup("Error: No statements");
    } else if (ast->count == 0) {
        result = strdup("Code(count: 0)");
    } else if (ast->count == 1) {
        // Single statement - use stmt or expr repr
        Stmt* stmt = ast->stmts[0];
        if (!stmt) {
            result = strdup("Error: NULL statement");
        } else if (stmt->kind == STMT_EXPR) {
            result = strdup(expr_repr(stmt->u.expr));
        } else {
            result = strdup(stmt_repr(stmt));
        }
    } else {
        // Multiple statements - use code repr
        result = strdup(code_repr(ast));
    }

    // Cleanup
    for (size_t i = 0; i < ast->count; i++) {
        if (ast->stmts[i]) {
            free(ast->stmts[i]);
        }
    }
    if (ast->stmts) free(ast->stmts);
    free(ast);
    parser_free(parser);
    lexer_free(lexer);

    return result ? result : strdup("Error: Unknown error");
}

// Compare strings ignoring whitespace differences
static bool compare_ignore_ws(const char* actual, const char* expected) {
    while (*actual && *expected) {
        // Skip whitespace in actual
        while (*actual && (*actual == ' ' || *actual == '\n' || *actual == '\r' || *actual == '\t')) actual++;
        // Skip whitespace in expected
        while (*expected && (*expected == ' ' || *expected == '\n' || *expected == '\r' || *expected == '\t')) expected++;

        if (*actual != *expected) return false;
        if (*actual) { actual++; expected++; }
    }

    // Check trailing whitespace
    while (*actual && (*actual == ' ' || *actual == '\n' || *actual == '\r' || *actual == '\t')) actual++;
    while (*expected && (*expected == ' ' || *expected == '\n' || *expected == '\r' || *expected == '\t')) expected++;

    return *actual == '\0' && *expected == '\0';
}

// Custom assertion macro for parser tests (ignores whitespace)
#define TEST_ASSERT_EQ_STR_IGNORE_WS(actual, expected, name) \
    do { \
        parser_tests_run++; \
        if (!compare_ignore_ws((actual), (expected))) { \
            fprintf(stderr, "  \033[31mFAILED\033[0m: %s\n", name); \
            fprintf(stderr, "    Expected: %s\n", expected); \
            fprintf(stderr, "    Actual:   %s\n", actual); \
            parser_tests_failed++; \
            return false; \
        } \
        parser_tests_passed++; \
        printf("  \033[32mPASSED\033[0m: %s\n", name); \
    } while(0)

// Test case runner function
static bool run_parser_test(ParserTestCase* tc) {
    char* actual = get_ast_repr(tc->input);
    TEST_ASSERT_EQ_STR_IGNORE_WS(actual, tc->expected, tc->name);
    free(actual);
    return true;
}

int main() {
    printf("=============================================================\n");
    printf("  Parser Test Runner\n");
    printf("=============================================================\n\n");

    fflush(stdout);

    // Try multiple possible test file paths
    const char* test_file_paths[] = {
        "tests/parser_tests.md",
        "../../tests/parser_tests.md",
        "../tests/parser_tests.md",
        NULL
    };

    const char* test_file = NULL;
    size_t content_size = 0;
    char* content = NULL;

    for (int i = 0; test_file_paths[i]; i++) {
        content = read_file(test_file_paths[i], &content_size);
        if (content) {
            test_file = test_file_paths[i];
            printf("Using test file: %s\n\n", test_file);
            fflush(stdout);
            break;
        }
    }

    if (!content) {
        fprintf(stderr, "Failed to read test file from any location\n");
        fprintf(stderr, "Tried:\n");
        for (int i = 0; test_file_paths[i]; i++) {
            fprintf(stderr, "  - %s\n", test_file_paths[i]);
        }
        return 1;
    }

    printf("Parsing test cases...\n");
    fflush(stdout);

    // Parse test cases
    size_t test_count = 0;
    ParserTestCase* test_cases = parse_test_cases(content, &test_count);

    printf("Found %zu test cases\n\n", test_count);
    fflush(stdout);
    free(content);

    if (!test_cases || test_count == 0) {
        fprintf(stderr, "No test cases found\n");
        return 1;
    }

    // Run all tests
    size_t max_tests = test_count;

    for (size_t i = 0; i < max_tests; i++) {
        ParserTestCase* tc = &test_cases[i];
        printf("Running %-50s...", tc->name);
        fflush(stdout);
        run_parser_test(tc);
    }

    // Summary
    printf("\n=============================================================\n");
    printf("  Test Summary\n");
    printf("=============================================================\n");
    printf("Total:   %d\n", parser_tests_run);
    printf("Passed:  %d\n", parser_tests_passed);
    printf("Failed:  %d\n", parser_tests_failed);
    printf("=============================================================\n");

    // Cleanup
    free(test_cases);

    return parser_tests_failed > 0 ? 1 : 0;
}
