#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "builder.h"

int main() {
    printf("Testing StringBuilder...\n");

    // Test 1: Create and append
    printf("\nTest 1: Create and append strings\n");
    StringBuilder* sb = (StringBuilder*)StringBuilder_new(1024);
    if (!sb) {
        printf("FAIL: Could not create StringBuilder\n");
        return 1;
    }

    StringBuilder_append(sb, "Hello, ");
    StringBuilder_append(sb, "World!");

    char* result = StringBuilder_build(sb);
    printf("Result: %s\n", result);
    free(result);

    // Test 2: Append integer
    printf("\nTest 2: Append integer\n");
    StringBuilder_clear(sb);
    StringBuilder_append(sb, "Count: ");
    StringBuilder_append_int(sb, 42);
    result = StringBuilder_build(sb);
    printf("Result: %s\n", result);
    free(result);

    // Test 3: Append character
    printf("\nTest 3: Append character\n");
    StringBuilder_clear(sb);
    StringBuilder_append(sb, "Char: ");
    StringBuilder_append_char(sb, 'A');
    result = StringBuilder_build(sb);
    printf("Result: %s\n", result);
    free(result);

    // Test 4: Length
    printf("\nTest 4: Get length\n");
    StringBuilder_clear(sb);
    StringBuilder_append(sb, "Test");
    size_t len = StringBuilder_len(sb);
    printf("Length: %zu\n", len);

    // Cleanup
    StringBuilder_drop(sb);

    printf("\nAll tests passed!\n");
    return 0;
}
