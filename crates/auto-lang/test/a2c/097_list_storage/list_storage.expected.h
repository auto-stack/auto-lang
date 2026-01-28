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

struct Heap Heap_New(struct Heap *self);
void* Heap_Data(struct Heap *self);
void Heap_Capacity(struct Heap *self);
bool Heap_TryGrow(struct Heap *self, unknown);
typedef struct Storage_void__vtable {
    void** (*data)(void *self);
    unknown (*capacity)(void *self);
    bool (*try_grow)(void *self, unknown min_cap);
} Storage_void__vtable;

struct InlineInt64 {
    int[64] buffer;
};

struct InlineInt64 InlineInt64_New(struct InlineInt64 *self);
int* InlineInt64_Data(struct InlineInt64 *self);
void InlineInt64_Capacity(struct InlineInt64 *self);
bool InlineInt64_TryGrow(struct InlineInt64 *self, unknown);
typedef struct Storage_int_vtable {
    void** (*data)(void *self);
    unknown (*capacity)(void *self);
    bool (*try_grow)(void *self, unknown min_cap);
} Storage_int_vtable;

struct List {
    unknown len;
    void* store;
};

struct List List_New(struct List *self);
void List_Len(struct List *self);
void List_Capacity(struct List *self);

