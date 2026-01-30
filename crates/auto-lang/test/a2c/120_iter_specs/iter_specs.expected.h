#pragma once

typedef struct Iter_vtable {
    void (*next)(void *self);
} Iter_vtable;

typedef struct Iterable_vtable {
    void (*iter)(void *self);
} Iterable_vtable;

