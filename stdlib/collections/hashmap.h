#ifndef AUTO_HASHMAP_H
#define AUTO_HASHMAP_H

#include <stddef.h>
#include "may.h"

// Uthash is a header-only library
#define UTHASH_IMPLEMENTATION
#include "uthash.h"

// Hash map entry (for string keys)
typedef struct {
    char* key;              // String key (owned by HashMap)
    void* value;            // Value pointer (not owned by HashMap)
    UT_hash_handle hh;      // Uthash handle
} HashMapEntry;

// Hash map structure
typedef struct {
    HashMapEntry* entries;  // Hash table (uthash manages this)
    size_t count;           // Number of entries
} HashMap;

// ============================================================================
// API - Creation
// ============================================================================

/// Create a new HashMap
May* HashMap_new();

/// Free a HashMap and all its entries
/// @param value_drop Optional callback to free values (NULL if values don't need freeing)
void HashMap_drop(HashMap* map, void (*value_drop)(void*));

// ============================================================================
// API - Operations
// ============================================================================

/// Insert a key-value pair into the map
/// If key already exists, replaces the value (caller responsible for freeing old value)
/// Returns May::error on allocation failure
May* HashMap_insert(HashMap* map, const char* key, void* value);

/// Get a value from the map by key
/// Returns May::value(value) if found, May::nil if not found
May* HashMap_get(HashMap* map, const char* key);

/// Check if a key exists in the map
bool HashMap_contains(HashMap* map, const char* key);

/// Remove a key from the map
/// Returns May::value(removed_value) if key was found (caller must free if needed)
/// Returns May::nil if key was not found
May* HashMap_remove(HashMap* map, const char* key);

// ============================================================================
// API - Utilities
// ============================================================================

/// Get the number of entries in the map
size_t HashMap_len(HashMap* map);

/// Clear all entries from the map
/// @param value_drop Optional callback to free values
void HashMap_clear(HashMap* map, void (*value_drop)(void*));

/// Iterate over all entries in the map
/// Callback function: (key, value, user_data) -> bool (return false to stop iteration)
void HashMap_iter(HashMap* map, bool (*callback)(const char* key, void* value, void* user_data), void* user_data);

#endif
