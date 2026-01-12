#include "multi_delegation.h"



void WarpDrive_Start(struct WarpDrive *self) {
    printf("%s\n", "WarpDrive engaging");
}

void LaserCannon_Fire(struct LaserCannon *self) {
    printf("%s\n", "Pew! Pew!");
}

void Starship_start(struct Starship *self) {
    WarpDrive_start(&self->core);
}
void Starship_fire(struct Starship *self) {
    LaserCannon_fire(&self->weapon);
}

int main(void) {
    struct Starship ship = {};
    Starship_start(&ship);
    Starship_fire(&ship);
    return 0;
}
