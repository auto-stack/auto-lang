#pragma once

typedef struct Storage_vtable {
    void (*get)(void *self);
} Storage_vtable;

struct Heap {
    int* ptr;
};
typedef struct Storage_int_vtable {
    int (*get)(void *self);
} Storage_int_vtable;

