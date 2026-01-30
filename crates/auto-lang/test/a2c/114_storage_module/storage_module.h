#pragma once

typedef struct Storage_vtable {
    void (*data)(void *self);
    void (*capacity)(void *self);
    void (*try_grow)(void *self, unknown min_cap);
} Storage_vtable;

struct Heap {
    void** ptr;
    unknown cap;
};
typedef struct Storage_void__vtable {
    void** (*data)(void *self);
    unknown (*capacity)(void *self);
    bool (*try_grow)(void *self, unknown min_cap);
} Storage_void__vtable;

struct InlineInt64 {
    int[64] buffer;
};
typedef struct Storage_int_vtable {
    void** (*data)(void *self);
    unknown (*capacity)(void *self);
    bool (*try_grow)(void *self, unknown min_cap);
} Storage_int_vtable;

struct List {
    unknown len;
    void* store;
};

