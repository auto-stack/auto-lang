// HashSet Implementation using uthash
// Phase 3: HashMap/HashSet (Plan 027)

#include "hashset.h"
#include "may.h"
#include <stdlib.h>
#include <string.h>

// ============================================================================
// API - Creation
// ============================================================================

May* HashSet_new() {
    HashSet* set = (HashSet*)malloc(sizeof(HashSet));
    if (!set) {
        return May_error("out of memory");
    }

    set->entries = NULL;
    set->count = 0;

    return May_value(set);
}

void HashSet_drop(HashSet* set) {
    if (!set) {
        return;
    }

    // Clear all entries first
    HashSet_clear(set);

    // Free the set structure itself
    free(set);
}

// ============================================================================
// API - Operations
// ============================================================================

May* HashSet_insert(HashSet* set, const char* value) {
    if (!set || !value) {
        return May_error("null argument");
    }

    // Check if value already exists
    HashSetEntry* entry = NULL;
    HASH_FIND_STR(set->entries, value, entry);

    if (entry) {
        // Value already exists: do nothing
        return May_value(set);
    }

    // Value doesn't exist: create new entry
    entry = (HashSetEntry*)malloc(sizeof(HashSetEntry));
    if (!entry) {
        return May_error("out of memory");
    }

    // Duplicate the value (HashSet owns the string)
    entry->value = strdup(value);
    if (!entry->value) {
        free(entry);
        return May_error("out of memory");
    }

    // Add to hash table
    HASH_ADD_STR(set->entries, value, entry);
    set->count++;

    return May_value(set);
}

bool HashSet_contains(HashSet* set, const char* value) {
    if (!set || !value) {
        return false;
    }

    HashSetEntry* entry = NULL;
    HASH_FIND_STR(set->entries, value, entry);

    return entry != NULL;
}

May* HashSet_remove(HashSet* set, const char* value) {
    if (!set || !value) {
        return May_error("null argument");
    }

    HashSetEntry* entry = NULL;
    HASH_FIND_STR(set->entries, value, entry);

    if (!entry) {
        return May_nil();
    }

    // Remove from hash table
    HASH_DEL(set->entries, entry);
    set->count--;

    // Free the value and entry
    free(entry->value);
    free(entry);

    return May_value((void*)1);  // Return true (value was found)
}

// ============================================================================
// API - Utilities
// ============================================================================

size_t HashSet_len(HashSet* set) {
    return set ? set->count : 0;
}

void HashSet_clear(HashSet* set) {
    if (!set) {
        return;
    }

    HashSetEntry* entry;
    HashSetEntry* tmp;

    // Iterate over all entries
    HASH_ITER(hh, set->entries, entry, tmp) {
        // Free the value
        free(entry->value);

        // Remove from hash table and free entry
        HASH_DEL(set->entries, entry);
        free(entry);
    }

    set->count = 0;
}

void HashSet_iter(HashSet* set, bool (*callback)(const char* value, void* user_data), void* user_data) {
    if (!set || !callback) {
        return;
    }

    HashSetEntry* entry;
    HashSetEntry* tmp;

    HASH_ITER(hh, set->entries, entry, tmp) {
        bool should_continue = callback(entry->value, user_data);
        if (!should_continue) {
            break;
        }
    }
}
