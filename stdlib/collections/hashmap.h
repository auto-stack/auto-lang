#pragma once

#include <stdlib.h>
#include <string.h>

// HashMap entry structure
typedef struct {
    char* key;
    void* value;
} HashMapEntry;

// HashMap structure (simplified - linear search array-based)
typedef struct {
    HashMapEntry* entries;
    size_t capacity;
    size_t size;
} HashMap;

// HashSet entry structure
typedef struct {
    char* value;
} HashSetEntry;

// HashSet structure (simplified - linear search array-based)
typedef struct {
    HashSetEntry* entries;
    size_t capacity;
    size_t size;
} HashSet;

// ============================================================================
// HashMap API
// ============================================================================

HashMap* HashMap_new();
void HashMap_drop(HashMap* map);

void HashMap_insert(HashMap* map, const char* key, void* value);
void* HashMap_get(HashMap* map, const char* key);
int HashMap_contains(HashMap* map, const char* key);
void* HashMap_remove(HashMap* map, const char* key);
int HashMap_size(HashMap* map);
void HashMap_clear(HashMap* map);

// ============================================================================
// HashSet API
// ============================================================================

HashSet* HashSet_new();
void HashSet_drop(HashSet* set);

void HashSet_insert(HashSet* set, const char* value);
int HashSet_contains(HashSet* set, const char* value);
void HashSet_remove(HashSet* set, const char* value);
int HashSet_size(HashSet* set);
void HashSet_clear(HashSet* set);
