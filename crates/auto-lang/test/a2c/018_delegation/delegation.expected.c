#include "delegation.h"


void WarpDrive_Start(struct WarpDrive *self) {
    printf("%s\n", "WarpDrive engaging");
}

void Starship_start(struct Starship *self) {
    WarpDrive_start(&self->core);
}

int main(void) {
    struct Starship ship = {};
    Starship_start(&ship);
    return 0;
}
