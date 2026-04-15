#pragma once

typedef struct Storage_vtable {
    void (*get)(void *self);
} Storage_vtable;

