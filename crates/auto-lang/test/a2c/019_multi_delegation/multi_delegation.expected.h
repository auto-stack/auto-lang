#pragma once

#include <stdio.h>

typedef struct Engine_vtable {
    void (*start)(void *self);
} Engine_vtable;

typedef struct Weapon_vtable {
    void (*fire)(void *self);
} Weapon_vtable;

struct WarpDrive {
};

void WarpDrive_Start(struct WarpDrive *self);
struct LaserCannon {
};

void LaserCannon_Fire(struct LaserCannon *self);
struct Starship {
    struct WarpDrive core;
    struct LaserCannon weapon;
};
void Starship_start(struct Starship *self);
void Starship_fire(struct Starship *self);
