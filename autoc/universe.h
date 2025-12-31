/**
 * Scope and Universe
 * Symbol table and scope management
 */

#ifndef UNIVERSE_H
#define UNIVERSE_H

#include "common.h"
#include "value.h"

typedef enum {
    SCOPE_GLOBAL,
    SCOPE_MOD,
    SCOPE_TYPE,
    SCOPE_FN,
    SCOPE_BLOCK,
} ScopeKind;

typedef struct {
    AutoStr path;
} Sid;

typedef struct {
    ScopeKind kind;
    Sid sid;
    Sid* parent;
    Sid** kids;
    size_t kid_count;
    size_t kid_capacity;
    // symbol tables
    Value** values;
    AutoStr* keys;
    size_t val_count;
    size_t val_capacity;
} Scope;

typedef struct {
    Scope** scopes;
    size_t scope_count;
    size_t scope_capacity;
    Scope* global;
    Scope* current;
    Sid* cur_spot;
} Universe;

Universe* universe_new(void);
void universe_free(Universe* universe);
Scope* universe_enter_scope(Universe* universe, ScopeKind kind);
void universe_exit_scope(Universe* universe);
Value* universe_get(Universe* universe, const char* name);
void universe_set(Universe* universe, const char* name, Value* value);
Value* universe_lookup(Universe* universe, const char* name);

#endif // UNIVERSE_H
