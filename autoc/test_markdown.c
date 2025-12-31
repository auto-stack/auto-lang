/**
 * Markdown Test Framework for autoc
 * Common functions for running tests from markdown files
 */

#include "test.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

char* read_file(const char* filename, size_t* out_size) {
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

MarkdownTestCase* parse_markdown_tests(const char* content, size_t* out_count) {
    MarkdownTestCase* cases = NULL;
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
                MarkdownTestCase* new_cases = (MarkdownTestCase*)realloc(cases, capacity * sizeof(MarkdownTestCase));
                if (!new_cases) {
                    fprintf(stderr, "Error: Out of memory\n");
                    free(cases);
                    return NULL;
                }
                cases = new_cases;
            }

            MarkdownTestCase* tc = &cases[count];
            memset(tc, 0, sizeof(MarkdownTestCase));

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

bool compare_exact(const char* actual, const char* expected) {
    return strcmp(actual, expected) == 0;
}

bool compare_ignore_ws(const char* actual, const char* expected) {
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

int run_markdown_test_suite(
    const char* test_filename,
    const char* suite_name,
    markdown_test_func_t test_func
) {
    printf("=============================================================\n");
    printf("  %s\n", suite_name);
    printf("=============================================================\n\n");

    fflush(stdout);

    // Try multiple possible test file paths
    const char* test_file_paths[] = {
        test_filename,
        "../../tests",
        "../tests",
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
    MarkdownTestCase* test_cases = parse_markdown_tests(content, &test_count);

    printf("Found %zu test cases\n\n", test_count);
    fflush(stdout);
    free(content);

    if (!test_cases || test_count == 0) {
        fprintf(stderr, "No test cases found\n");
        return 1;
    }

    // Initialize statistics
    TestStatistics stats = {0, 0, 0};

    // Run all tests
    for (size_t i = 0; i < test_count; i++) {
        MarkdownTestCase* tc = &test_cases[i];
        printf("Running %-50s...", tc->name);
        fflush(stdout);
        test_func(tc, &stats);
    }

    // Summary
    printf("\n=============================================================\n");
    printf("  Test Summary\n");
    printf("=============================================================\n");
    printf("Total:   %d\n", stats.run);
    printf("Passed:  %d\n", stats.passed);
    printf("Failed:  %d\n", stats.failed);
    printf("=============================================================\n");

    // Cleanup
    free(test_cases);

    return stats.failed > 0 ? 1 : 0;
}
