// HashMap/HashSet C Implementation
// Phase 3: HashMap/HashSet (Plan 027)
//
// Simple linear-search implementation for now
// TODO: Optimize with proper hash table (separate phase)

#include "hashmap.h"
#include <stdlib.h>
#include <string.h>

#define INITIAL_CAPACITY 16

// ============================================================================
// HashMap Implementation
// ============================================================================

HashMap* HashMap_new() {
    HashMap* map = (HashMap*)malloc(sizeof(HashMap));
    if (!map) return NULL;

    map->entries = (HashMapEntry*)calloc(INITIAL_CAPACITY, sizeof(HashMapEntry));
    if (!map->entries) {
        free(map);
        return NULL;
    }

    map->capacity = INITIAL_CAPACITY;
    map->size = 0;

    return map;
}

void HashMap_drop(HashMap* map) {
    if (!map) return;

    // Free all keys
    for (size_t i = 0; i < map->size; i++) {
        if (map->entries[i].key) {
            free(map->entries[i].key);
        }
    }

    free(map->entries);
    free(map);
}

void HashMap_insert(HashMap* map, const char* key, void* value) {
    if (!map || !key) return;

    // Check if key already exists
    for (size_t i = 0; i < map->size; i++) {
        if (strcmp(map->entries[i].key, key) == 0) {
            // Update existing entry
            map->entries[i].value = value;
            return;
        }
    }

    // Check if we need to expand
    if (map->size >= map->capacity) {
        size_t new_capacity = map->capacity * 2;
        HashMapEntry* new_entries = (HashMapEntry*)realloc(map->entries,
            sizeof(HashMapEntry) * new_capacity);
        if (!new_entries) return;  // OOM

        // Initialize new entries
        for (size_t i = map->size; i < new_capacity; i++) {
            new_entries[i].key = NULL;
            new_entries[i].value = NULL;
        }

        map->entries = new_entries;
        map->capacity = new_capacity;
    }

    // Add new entry
    map->entries[map->size].key = strdup(key);
    map->entries[map->size].value = value;
    map->size++;
}

void* HashMap_get(HashMap* map, const char* key) {
    if (!map || !key) return NULL;

    for (size_t i = 0; i < map->size; i++) {
        if (strcmp(map->entries[i].key, key) == 0) {
            return map->entries[i].value;
        }
    }

    return NULL;
}

int HashMap_contains(HashMap* map, const char* key) {
    if (!map || !key) return 0;

    for (size_t i = 0; i < map->size; i++) {
        if (strcmp(map->entries[i].key, key) == 0) {
            return 1;
        }
    }

    return 0;
}

void* HashMap_remove(HashMap* map, const char* key) {
    if (!map || !key) return NULL;

    for (size_t i = 0; i < map->size; i++) {
        if (strcmp(map->entries[i].key, key) == 0) {
            // Found it
            void* value = map->entries[i].value;
            free(map->entries[i].key);

            // Shift remaining entries
            for (size_t j = i; j < map->size - 1; j++) {
                map->entries[j] = map->entries[j + 1];
            }

            map->size--;
            return value;
        }
    }

    return NULL;
}

int HashMap_size(HashMap* map) {
    return map ? (int)map->size : 0;
}

void HashMap_clear(HashMap* map) {
    if (!map) return;

    for (size_t i = 0; i < map->size; i++) {
        if (map->entries[i].key) {
            free(map->entries[i].key);
        }
    }

    map->size = 0;
}

// ============================================================================
// HashSet Implementation
// ============================================================================

HashSet* HashSet_new() {
    HashSet* set = (HashSet*)malloc(sizeof(HashSet));
    if (!set) return NULL;

    set->entries = (HashSetEntry*)calloc(INITIAL_CAPACITY, sizeof(HashSetEntry));
    if (!set->entries) {
        free(set);
        return NULL;
    }

    set->capacity = INITIAL_CAPACITY;
    set->size = 0;

    return set;
}

void HashSet_drop(HashSet* set) {
    if (!set) return;

    // Free all values
    for (size_t i = 0; i < set->size; i++) {
        if (set->entries[i].value) {
            free(set->entries[i].value);
        }
    }

    free(set->entries);
    free(set);
}

void HashSet_insert(HashSet* set, const char* value) {
    if (!set || !value) return;

    // Check if value already exists
    for (size_t i = 0; i < set->size; i++) {
        if (strcmp(set->entries[i].value, value) == 0) {
            return;  // Already exists
        }
    }

    // Check if we need to expand
    if (set->size >= set->capacity) {
        size_t new_capacity = set->capacity * 2;
        HashSetEntry* new_entries = (HashSetEntry*)realloc(set->entries,
            sizeof(HashSetEntry) * new_capacity);
        if (!new_entries) return;  // OOM

        // Initialize new entries
        for (size_t i = set->size; i < new_capacity; i++) {
            new_entries[i].value = NULL;
        }

        set->entries = new_entries;
        set->capacity = new_capacity;
    }

    // Add new entry
    set->entries[set->size].value = strdup(value);
    set->size++;
}

int HashSet_contains(HashSet* set, const char* value) {
    if (!set || !value) return 0;

    for (size_t i = 0; i < set->size; i++) {
        if (strcmp(set->entries[i].value, value) == 0) {
            return 1;
        }
    }

    return 0;
}

void HashSet_remove(HashSet* set, const char* value) {
    if (!set || !value) return;

    for (size_t i = 0; i < set->size; i++) {
        if (strcmp(set->entries[i].value, value) == 0) {
            // Found it
            free(set->entries[i].value);

            // Shift remaining entries
            for (size_t j = i; j < set->size - 1; j++) {
                set->entries[j] = set->entries[j + 1];
            }

            set->size--;
            return;
        }
    }
}

int HashSet_size(HashSet* set) {
    return set ? (int)set->size : 0;
}

void HashSet_clear(HashSet* set) {
    if (!set) return;

    for (size_t i = 0; i < set->size; i++) {
        if (set->entries[i].value) {
            free(set->entries[i].value);
        }
    }

    set->size = 0;
}
