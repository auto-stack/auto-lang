/**
 * Universe and Scope Implementation
 * Manages scopes, symbols, and values
 */

#include "autoc.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// ============================================================================
// Sid (Scope ID) Utilities
// ============================================================================

static Sid sid_new(const char* path) {
    Sid sid;
    sid.path = astr_new(path);
    return sid;
}

static Sid sid_kid_of(Sid* parent, const char* name) {
    Sid sid;
    if (parent->path.len == 0 || strcmp(parent->path.data, "") == 0) {
        sid.path = astr_new(name);
    } else {
        char buffer[512];
        snprintf(buffer, sizeof(buffer), "%s.%s", parent->path.data, name);
        sid.path = astr_new(buffer);
    }
    return sid;
}

static void sid_free(Sid* sid) {
    astr_free(&sid->path);
}

// ============================================================================
// Scope Creation
// ============================================================================

static Scope* scope_new(ScopeKind kind, Sid sid) {
    Scope* scope = (Scope*)malloc(sizeof(Scope));
    scope->kind = kind;
    scope->sid = sid;
    scope->parent = NULL;
    scope->kids = NULL;
    scope->kid_count = 0;
    scope->kid_capacity = 0;
    scope->values = NULL;
    scope->keys = NULL;
    scope->val_count = 0;
    scope->val_capacity = 0;
    return scope;
}

static void scope_free(Scope* scope) {
    if (!scope) return;

    // Free values
    for (size_t i = 0; i < scope->val_count; i++) {
        astr_free(&scope->keys[i]);
        value_free(scope->values[i]);
    }
    free(scope->values);
    free(scope->keys);

    // Free kids
    if (scope->kids) {
        for (size_t i = 0; i < scope->kid_count; i++) {
            free(scope->kids[i]);
        }
        free(scope->kids);
    }

    // Free parent Sid (we malloc it in universe_enter_scope)
    if (scope->parent) {
        free(scope->parent);
    }

    // Free sid
    sid_free(&scope->sid);

    free(scope);
}

static void scope_set(Scope* scope, const char* name, Value* value) {
    // Check if key already exists
    for (size_t i = 0; i < scope->val_count; i++) {
        if (strcmp(scope->keys[i].data, name) == 0) {
            value_free(scope->values[i]);
            scope->values[i] = value;
            return;
        }
    }

    // Add new key-value pair
    if (scope->val_count >= scope->val_capacity) {
        scope->val_capacity = scope->val_capacity == 0 ? 16 : scope->val_capacity * 2;
        scope->keys = (AutoStr*)realloc(scope->keys, scope->val_capacity * sizeof(AutoStr));
        scope->values = (Value**)realloc(scope->values, scope->val_capacity * sizeof(Value*));
    }

    scope->keys[scope->val_count] = astr_new(name);
    scope->values[scope->val_count] = value;
    scope->val_count++;
}

static Value* scope_get(Scope* scope, const char* name) {
    for (size_t i = 0; i < scope->val_count; i++) {
        if (strcmp(scope->keys[i].data, name) == 0) {
            return scope->values[i];
        }
    }
    return NULL;
}

static bool scope_has(Scope* scope, const char* name) {
    return scope_get(scope, name) != NULL;
}

// ============================================================================
// Universe Creation
// ============================================================================

Universe* universe_new(void) {
    Universe* universe = (Universe*)malloc(sizeof(Universe));
    universe->scopes = NULL;
    universe->scope_count = 0;
    universe->scope_capacity = 0;

    // Create global scope
    Sid global_sid = sid_new("");
    Scope* global = scope_new(SCOPE_GLOBAL, global_sid);
    universe->global = global;
    universe->current = global;
    universe->cur_spot = (Sid*)malloc(sizeof(Sid));
    *universe->cur_spot = global_sid;

    // Add global scope to scopes list
    universe->scope_capacity = 8;
    universe->scopes = (Scope**)malloc(universe->scope_capacity * sizeof(Scope*));
    universe->scopes[universe->scope_count++] = global;

    return universe;
}

void universe_free(Universe* universe) {
    if (!universe) return;

    // Free all scopes
    for (size_t i = 0; i < universe->scope_count; i++) {
        scope_free(universe->scopes[i]);
    }
    free(universe->scopes);

    // Free cur_spot
    if (universe->cur_spot) {
        free(universe->cur_spot);
    }

    free(universe);
}

Scope* universe_enter_scope(Universe* universe, ScopeKind kind) {
    // Generate new scope ID
    char name[64];
    static int block_counter = 0;

    switch (kind) {
        case SCOPE_BLOCK:
            snprintf(name, sizeof(name), "block_%d", block_counter++);
            break;
        default:
            snprintf(name, sizeof(name), "scope_%d", block_counter++);
            break;
    }

    Sid new_sid = sid_kid_of(universe->cur_spot, name);

    // Create new scope
    Scope* new_scope = scope_new(kind, new_sid);

    // Save parent Sid (copy it BEFORE updating cur_spot)
    Sid* parent_sid = (Sid*)malloc(sizeof(Sid));
    *parent_sid = *universe->cur_spot;
    new_scope->parent = parent_sid;

    // Add to parent's kids
    if (universe->current->kid_count >= universe->current->kid_capacity) {
        universe->current->kid_capacity = universe->current->kid_capacity == 0 ? 8 : universe->current->kid_capacity * 2;
        universe->current->kids = (Sid**)realloc(universe->current->kids, universe->current->kid_capacity * sizeof(Sid*));
    }
    Sid* kid_sid = (Sid*)malloc(sizeof(Sid));
    *kid_sid = new_sid;
    universe->current->kids[universe->current->kid_count++] = kid_sid;

    // Add to universe scopes
    if (universe->scope_count >= universe->scope_capacity) {
        universe->scope_capacity = universe->scope_capacity * 2;
        universe->scopes = (Scope**)realloc(universe->scopes, universe->scope_capacity * sizeof(Scope*));
    }
    universe->scopes[universe->scope_count++] = new_scope;

    // Set as current
    universe->current = new_scope;
    *universe->cur_spot = new_sid;

    return new_scope;
}

void universe_exit_scope(Universe* universe) {
    if (universe->current->parent) {
        Sid parent_sid = *universe->current->parent;

        // Find parent scope
        for (size_t i = 0; i < universe->scope_count; i++) {
            Scope* scope = universe->scopes[i];
            if (astr_eq(&scope->sid.path, &parent_sid.path)) {
                universe->current = scope;
                *universe->cur_spot = parent_sid;
                return;
            }
        }
    }
}

Value* universe_get(Universe* universe, const char* name) {
    Scope* current = universe->current;
    while (current) {
        Value* value = scope_get(current, name);
        if (value) {
            return value;
        }
        if (!current->parent) break;
        // Find parent scope
        for (size_t i = 0; i < universe->scope_count; i++) {
            Scope* scope = universe->scopes[i];
            if (astr_eq(&scope->sid.path, &current->parent->path)) {
                current = scope;
                break;
            }
        }
    }
    return NULL;
}

void universe_set(Universe* universe, const char* name, Value* value) {
    // First, check if variable exists in current or parent scopes
    Scope* current = universe->current;
    while (current) {
        // Check if this scope has the variable
        for (size_t i = 0; i < current->val_count; i++) {
            if (strcmp(current->keys[i].data, name) == 0) {
                // Found it! Update the value in this scope
                value_free(current->values[i]);
                current->values[i] = value;
                return;
            }
        }
        // Check parent
        if (!current->parent) break;
        for (size_t i = 0; i < universe->scope_count; i++) {
            Scope* scope = universe->scopes[i];
            if (astr_eq(&scope->sid.path, &current->parent->path)) {
                current = scope;
                break;
            }
        }
    }

    // Variable doesn't exist in any scope, create it in current scope
    scope_set(universe->current, name, value);
}

Value* universe_lookup(Universe* universe, const char* name) {
    return universe_get(universe, name);
}
