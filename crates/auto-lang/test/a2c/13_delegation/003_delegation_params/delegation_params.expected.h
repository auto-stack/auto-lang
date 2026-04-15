#pragma once

#include <stdio.h>

typedef struct Calculator_vtable {
    void (*add)(void *self, int a, int b);
    void (*multiply)(void *self, int a, int b);
} Calculator_vtable;

struct MathEngine {
};

int MathEngine_Add(struct MathEngine *self, int, int);
int MathEngine_Multiply(struct MathEngine *self, int, int);
struct Computer {
    struct MathEngine engine;
};
int Computer_add(struct Computer *self, int a, int b);
int Computer_multiply(struct Computer *self, int a, int b);
