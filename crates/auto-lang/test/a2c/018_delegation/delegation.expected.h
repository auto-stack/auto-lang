#pragma once

#include <stdio.h>

typedef struct Engine_vtable {
    void (*start)(void *self);
} Engine_vtable;

struct WarpDrive {
};

void WarpDrive_Start(struct WarpDrive *self);
struct Starship {
    struct WarpDrive core;
};
void Starship_start(struct Starship *self);
