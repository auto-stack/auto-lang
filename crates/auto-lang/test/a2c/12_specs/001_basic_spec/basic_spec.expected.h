#pragma once

#include <stdio.h>

typedef struct Flyer_vtable {
    void (*fly)(void *self);
} Flyer_vtable;

struct Pigeon {
};

void Pigeon_Fly(struct Pigeon *self);
