#include "spec.h"


void Pigeon_Fly(struct Pigeon *self) {
    printf("%s\n", "Flap Flap");
}
Flyer_vtable Pigeon_Flyer_vtable = {
    .fly = Pigeon_Fly
};


void Hawk_Fly(struct Hawk *self) {
    printf("%s\n", "Gawk! Gawk!");
}
Flyer_vtable Hawk_Flyer_vtable = {
    .fly = Hawk_Fly
};


int main(void) {

    struct Pigeon b1 = {};
    struct Hawk b2 = {};

    void* arr[2] = {&b1, &b2};
    for (int i = 0; i < 2; i++) {
        void* b = arr[i];
        int_fly(b);
    }
    return 0;
}
