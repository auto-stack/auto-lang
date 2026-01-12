#include "basic_spec.h"


void Pigeon_Fly(struct Pigeon *self) {
    printf("%s\n", "Flap");
}
Flyer_vtable Pigeon_Flyer_vtable = {
    .fly = Pigeon_Fly
};


int main(void) {
    struct Pigeon p = {};
    Pigeon_Fly(&p);
    return 0;
}
