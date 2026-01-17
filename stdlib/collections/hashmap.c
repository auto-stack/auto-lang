// HashMap Implementation using uthash
// Phase 3: HashMap/HashSet (Plan 027)

#include "hashmap.h"
#include "may.h"
#include <stdlib.h>
#include <string.h>

// ============================================================================
// API - Creation
// ============================================================================

May* HashMap_new() {
    HashMap* map = (HashMap*)malloc(sizeof(HashMap));
    if (!map) {
        return May_error("out of memory");
    }

    map->entries = NULL;
    map->count = 0;

    return May_value(map);
}

void HashMap_drop(HashMap* map, void (*value_drop)(void*)) {
    if (!map) {
        return;
    }

    // Clear all entries first
    HashMap_clear(map, value_drop);

    // Free the map structure itself
    free(map);
}

// ============================================================================
// API - Operations
// ============================================================================

May* HashMap_insert(HashMap* map, const char* key, void* value) {
    if (!map || !key) {
        return May_error("null argument");
    }

    // Check if key already exists
    HashMapEntry* entry = NULL;
    HASH_FIND_STR(map->entries, key, entry);

    if (entry) {
        // Key exists: update value
        entry->value = value;
        return May_value(map);
    }

    // Key doesn't exist: create new entry
    entry = (HashMapEntry*)malloc(sizeof(HashMapEntry));
    if (!entry) {
        return May_error("out of memory");
    }

    // Duplicate the key (HashMap owns the key string)
    entry->key = strdup(key);
    if (!entry->key) {
        free(entry);
        return May_error("out of memory");
    }

    entry->value = value;

    // Add to hash table
    HASH_ADD_STR(map->entries, key, entry);
    map->count++;

    return May_value(map);
}

May* HashMap_get(HashMap* map, const char* key) {
    if (!map || !key) {
        return May_error("null argument");
    }

    HashMapEntry* entry = NULL;
    HASH_FIND_STR(map->entries, key, entry);

    if (entry) {
        return May_value(entry->value);
    }

    return May_nil();
}

bool HashMap_contains(HashMap* map, const char* key) {
    if (!map || !key) {
        return false;
    }

    HashMapEntry* entry = NULL;
    HASH_FIND_STR(map->entries, key, entry);

    return entry != NULL;
}

May* HashMap_remove(HashMap* map, const char* key) {
    if (!map || !key) {
        return May_error("null argument");
    }

    HashMapEntry* entry = NULL;
    HASH_FIND_STR(map->entries, key, entry);

    if (!entry) {
        return May_nil();
    }

    // Remove from hash table
    HASH_DEL(map->entries, entry);
    map->count--;

    // Get the value before freeing entry
    void* value = entry->value;

    // Free the key and entry
    free(entry->key);
    free(entry);

    return May_value(value);
}

// ============================================================================
// API - Utilities
// ============================================================================

size_t HashMap_len(HashMap* map) {
    return map ? map->count : 0;
}

void HashMap_clear(HashMap* map, void (*value_drop)(void*)) {
    if (!map) {
        return;
    }

    HashMapEntry* entry;
    HashMapEntry* tmp;

    // Iterate over all entries
    HASH_ITER(hh, map->entries, entry, tmp) {
        // Call value_drop if provided
        if (value_drop && entry->value) {
            value_drop(entry->value);
        }

        // Free the key
        free(entry->key);

        // Remove from hash table and free entry
        HASH_DEL(map->entries, entry);
        free(entry);
    }

    map->count = 0;
}

void HashMap_iter(HashMap* map, bool (*callback)(const char* key, void* value, void* user_data), void* user_data) {
    if (!map || !callback) {
        return;
    }

    HashMapEntry* entry;
    HashMapEntry* tmp;

    HASH_ITER(hh, map->entries, entry, tmp) {
        bool should_continue = callback(entry->key, entry->value, user_data);
        if (!should_continue) {
            break;
        }
    }
}
