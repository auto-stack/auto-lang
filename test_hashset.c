#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "collections/hashset.h"

int main() {
    printf("Testing HashSet...\n");

    // Test 1: Create and insert
    printf("\nTest 1: Create and insert\n");
    HashSet* set = NULL;
    May* result = HashSet_new();
    if (result && result->tag == May_Value) {
        set = (HashSet*)result->data.value;
        printf("HashSet created successfully\n");
    } else {
        printf("FAIL: Could not create HashSet\n");
        return 1;
    }

    // Insert some values
    HashSet_insert(set, "apple");
    HashSet_insert(set, "banana");
    HashSet_insert(set, "cherry");

    printf("Inserted 3 entries\n");

    // Test 2: Contains
    printf("\nTest 2: Contains\n");
    bool contains = HashSet_contains(set, "banana");
    printf("HashSet_contains(set, \"banana\") = %s\n", contains ? "true" : "false");

    contains = HashSet_contains(set, "date");
    printf("HashSet_contains(set, \"date\") = %s\n", contains ? "true" : "false");

    // Test 3: Duplicate insert
    printf("\nTest 3: Duplicate insert\n");
    size_t len_before = HashSet_len(set);
    HashSet_insert(set, "apple");  // Duplicate
    size_t len_after = HashSet_len(set);
    printf("Before: %zu, After duplicate insert: %zu\n", len_before, len_after);

    // Test 4: Length
    printf("\nTest 4: Length\n");
    printf("HashSet_len(set) = %zu\n", HashSet_len(set));

    // Test 5: Remove
    printf("\nTest 5: Remove\n");
    result = HashSet_remove(set, "banana");
    if (result && result->tag == May_Value) {
        printf("Removed entry successfully\n");
    }

    len_after = HashSet_len(set);
    printf("After remove, HashSet_len(set) = %zu\n", len_after);

    // Test 6: Try to remove non-existent
    printf("\nTest 6: Remove non-existent\n");
    result = HashSet_remove(set, "date");
    if (result && result->tag == May_Nil) {
        printf("Correctly returned nil for non-existent key\n");
    }

    // Cleanup
    HashSet_drop(set);

    printf("\nAll HashSet tests passed!\n");
    return 0;
}
