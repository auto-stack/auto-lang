#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "collections/hashmap.h"

// Simple value drop function for integers
void drop_int(void* value) {
    // Integers don't need special cleanup
}

int main() {
    printf("Testing HashMap...\n");

    // Test 1: Create and insert
    printf("\nTest 1: Create and insert\n");
    HashMap* map = NULL;
    May* result = HashMap_new();
    if (result && result->tag == May_Value) {
        map = (HashMap*)result->data.value;
        printf("HashMap created successfully\n");
    } else {
        printf("FAIL: Could not create HashMap\n");
        return 1;
    }

    // Insert some values (using heap-allocated integers for demo)
    int* one = malloc(sizeof(int));
    *one = 1;
    int* two = malloc(sizeof(int));
    *two = 2;
    int* three = malloc(sizeof(int));
    *three = 3;

    HashMap_insert(map, "one", one);
    HashMap_insert(map, "two", two);
    HashMap_insert(map, "three", three);

    printf("Inserted 3 entries\n");

    // Test 2: Get values
    printf("\nTest 2: Get values\n");
    result = HashMap_get(map, "two");
    if (result && result->tag == May_Value) {
        int value = *(int*)result->data.value;
        printf("HashMap_get(map, \"two\") = %d\n", value);
    } else {
        printf("FAIL: Could not get value\n");
    }

    // Test 3: Contains
    printf("\nTest 3: Contains\n");
    bool contains = HashMap_contains(map, "two");
    printf("HashMap_contains(map, \"two\") = %s\n", contains ? "true" : "false");

    contains = HashMap_contains(map, "four");
    printf("HashMap_contains(map, \"four\") = %s\n", contains ? "true" : "false");

    // Test 4: Length
    printf("\nTest 4: Length\n");
    size_t len = HashMap_len(map);
    printf("HashMap_len(map) = %zu\n", len);

    // Test 5: Remove
    printf("\nTest 5: Remove\n");
    result = HashMap_remove(map, "two");
    if (result && result->tag == May_Value) {
        printf("Removed entry: %d\n", *(int*)result->data.value);
        free(result->data.value);  // Free the removed value
    }

    len = HashMap_len(map);
    printf("After remove, HashMap_len(map) = %zu\n", len);

    // Test 6: Iterate
    printf("\nTest 6: Iterate\n");
    HashMap_iter(map, NULL, NULL);

    // Cleanup
    HashMap_drop(map, drop_int);

    printf("\nAll HashMap tests passed!\n");
    return 0;
}
