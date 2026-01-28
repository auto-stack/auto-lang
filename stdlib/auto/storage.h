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

