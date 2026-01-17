#ifndef AUTO_HASHSET_H
#define AUTO_HASHSET_H

#include <stddef.h>
#include "may.h"

// Uthash is a header-only library
#define UTHASH_IMPLEMENTATION
#include "uthash.h"

// Hash set entry (for string values)
typedef struct {
    char* value;            // String value (owned by HashSet)
    UT_hash_handle hh;      // Uthash handle
} HashSetEntry;

// Hash set structure
typedef struct {
    HashSetEntry* entries;  // Hash table (uthash manages this)
    size_t count;           // Number of entries
} HashSet;

// ============================================================================
// API - Creation
// ============================================================================

/// Create a new HashSet
May* HashSet_new();

/// Free a HashSet and all its entries
void HashSet_drop(HashSet* set);

// ============================================================================
// API - Operations
// ============================================================================

/// Insert a value into the set
/// If value already exists, does nothing
/// Returns May::error on allocation failure
May* HashSet_insert(HashSet* set, const char* value);

/// Check if a value exists in the set
bool HashSet_contains(HashSet* set, const char* value);

/// Remove a value from the set
/// Returns May::value(true) if value was found and removed
/// Returns May::nil if value was not found
May* HashSet_remove(HashSet* set, const char* value);

// ============================================================================
// API - Utilities
// ============================================================================

/// Get the number of entries in the set
size_t HashSet_len(HashSet* set);

/// Clear all entries from the set
void HashSet_clear(HashSet* set);

/// Iterate over all values in the set
/// Callback function: (value, user_data) -> bool (return false to stop iteration)
void HashSet_iter(HashSet* set, bool (*callback)(const char* value, void* user_data), void* user_data);

#endif
